use futures::join;
use google_calendar3::Error;
use hyper::service::{make_service_fn, service_fn};
use hyper::Client;
use hyper::Server;
use hyper::{Body, Method, Request, Response, StatusCode};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::{task, time};
use yup_oauth2::parse_service_account_key;

mod calendar;

async fn route(
    req: Request<Body>,
    state: Arc<Mutex<calendar::Events>>,
) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());

    let uuid = {
        let es = state.lock().await;
        es.uuid.clone()
    };
    let change = format!("/change/{}", uuid);
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let event_struct = state.lock().await;

            let json = serde_json::to_string_pretty(&*event_struct).unwrap();
            *response.body_mut() = Body::from(json);
        }
        (&Method::POST, c) if c == change => {
            let mut event_struct = state.lock().await;
            event_struct.scan_calendar().await.unwrap();
            event_struct.update_filtered_events().await.unwrap();
        }
        (&Method::GET, "/check") => {
            let mut event_struct = state.lock().await;
            event_struct.scan_calendar().await.unwrap();
            event_struct.update_filtered_events().await.unwrap();
            let json = serde_json::to_string_pretty(&*event_struct).unwrap();
            *response.body_mut() = Body::from(json);
        }
        _ => {
            *response.body_mut() = Body::from("<HTML><body><h1>Not found</h1></body></HTML>");
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}

static SERVICE_CREDENTIALS: &[u8] = include_bytes!("../film-festival.json");

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load the secretn
    let service_key =
        parse_service_account_key(SERVICE_CREDENTIALS).expect("bad gmail credentials");

    let client = Client::builder().build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .build(),
    );
    let auth = yup_oauth2::ServiceAccountAuthenticator::with_client(service_key, client.clone())
        .build()
        .await
        .expect("failed to create authenticator");
    let state = Arc::new(Mutex::new(calendar::Events::new(client, auth)));
    let addr = ([127, 0, 0, 1], 3020).into();
    let svc = make_service_fn(|_| {
        let state = state.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let state = state.clone();
                async move { route(req, state).await }
            }))
        }
    });
    let server = Server::bind(&addr).serve(svc);
    let state = state.clone();
    let renew = task::spawn(async move {
        let state = state.clone();
        let mut interval = time::interval(Duration::from_millis(1000000));
        loop {
            interval.tick().await;
            {
                let mut event_struct = state.lock().await;
                let _ = event_struct
                    .renew_watch(interval.period().as_millis())
                    .await;
            }
        }
    });

    let _ = join!(server, renew);
    Ok(())
}
