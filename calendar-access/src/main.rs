use google_calendar3 as cal;
use hyper_rustls;
use yup_oauth2 as oauth2;
use hyper::Client;
use cal::Error;

#[tokio::main]
async fn main() {
    // Load the secret
    let secret = oauth2::read_application_secret("credentials.json")
        .await
        .expect("Client secret not loaded from credentials.json");
    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("tokencache.json")
        .build()
        .await
        .unwrap();
    let client = Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().build());

    let hub = cal::CalendarHub::new(client,auth);


    let result = hub.events().list("c12717e59b8cbf4e58b2eb5b0fe0e8aa823cf71943cab642507715cd86db80f8@group.calendar.google.com")
    //let result = hub.events().list("primary")
        .doit()
        .await;

    let page_token = match result {
        Err(e) => match e {
            Error::HttpError(_) 
                | Error::Io(_)
                | Error::MissingAPIKey
                | Error::MissingToken(_)
                | Error::Cancelled
                | Error::UploadSizeLimitExceeded(_,_)
                | Error::Failure(_)
                | Error::BadRequest(_)
                | Error::FieldClash(_)
                | Error::JsonDecodeError(_,_) => { println!("{}",e); None },
        }
            Ok(res) => { /*println!("Success: {:?}",res);*/ res.1.next_page_token },
        };
    if let Some(token) = page_token {
      let result = hub.events().list("c12717e59b8cbf4e58b2eb5b0fe0e8aa823cf71943cab642507715cd86db80f8@group.calendar.google.com")
        .page_token(&token)
        .doit()
        .await;
    let page_token = match result {
        Err(e) => match e {
            Error::HttpError(_) 
                | Error::Io(_)
                | Error::MissingAPIKey
                | Error::MissingToken(_)
                | Error::Cancelled
                | Error::UploadSizeLimitExceeded(_,_)
                | Error::Failure(_)
                | Error::BadRequest(_)
                | Error::FieldClash(_)
                | Error::JsonDecodeError(_,_) => { println!("{}",e); None },
        }
            Ok(res) => { println!("Success: {:?}",res.1.items.unwrap()); res.1.next_page_token },
        };
    }
}
