use google_calendar3::Error;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use yup_oauth2 as oauth2;

mod calendar;

async fn ok_page(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Working server".into()))
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

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(ok_page)) });
    let server = Server::bind(&addr).serve(make_svc);

    let mut event_struct = calendar::Events::new(auth);
    event_struct.scan_calendar().await?;
    event_struct.update_filtered_events().await?;

    let _ = server.await;
    Ok(())
}
