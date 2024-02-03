use futures::join;
use google_calendar3::Error;
use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyper::{Body, Method, Request, Response, StatusCode};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tls_listener::TlsListener;
use tokio::sync::Mutex;
use tokio::{task, time};
use yup_oauth2 as oauth2;

mod calendar;
const CERT: &[u8] = include_bytes!("../darach.cert");
const PKEY: &[u8] = include_bytes!("../darach.key");

fn tls_acceptor() -> tokio_rustls::TlsAcceptor {
    use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
    let key = PrivateKey(PKEY.into());
    let cert = Certificate(CERT.into());
    Arc::new(
        ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap(),
    )
    .into()
}

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
    let addr = ([0, 0, 0, 0], 3020).into();
    let svc = make_service_fn(|_| {
        let state = state.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let state = state.clone();
                async move { route(req, state).await }
            }))
        }
    });
    let incoming = TlsListener::new(tls_acceptor(), AddrIncoming::bind(&addr).unwrap());
    let server = Server::builder(incoming).serve(svc);
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
