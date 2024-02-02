use google_calendar3::Error;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use yup_oauth2 as oauth2;

mod calendar;

async fn route(
    req: Request<Body>,
    state: Arc<Mutex<calendar::Events>>,
) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let event_struct = state.lock().await;

            let json = serde_json::to_string_pretty(&*event_struct).unwrap();
            *response.body_mut() = Body::from(json);
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load the secret
    let secret = oauth2::read_application_secret("credentials.json")
        .await
        .expect("Client secret not loaded from credentials.json");
    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("tokencache.json")
    .build()
    .await
    .unwrap();

    let state = Arc::new(Mutex::new(calendar::Events::new(auth)));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let make_svc = make_service_fn(|_| {
        let state = state.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let state = state.clone();
                async move { route(req, state).await }
            }))
        }
    });
    let server = Server::bind(&addr).serve(make_svc);

    let _ = server.await;
    Ok(())
}
