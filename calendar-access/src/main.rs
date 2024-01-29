use google_calendar3 as cal;
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

    match all_events(&hub,"c12717e59b8cbf4e58b2eb5b0fe0e8aa823cf71943cab642507715cd86db80f8@group.calendar.google.com").await {
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
                | Error::JsonDecodeError(_,_) => { println!("{}",e); },
        }
            Ok(events) => { println!("Got {} events", events.len()); 
                            for e in events {
                               println!("{:?} - {:?}",e.summary, e.description); } },
        };
    }

pub async fn all_events(hub: &cal::CalendarHub<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>, cal_id: &str) -> Result<Vec<cal::api::Event>,Error> 
{
    let mut result = Vec::new();
    let mut response;
    response = hub.events().list(cal_id)
        .doit()
        .await?;
    loop {
      if let Some(mut items) = response.1.items {
        result.append(&mut items);
      };
      if let Some(token) = response.1.next_page_token {
        response = hub.events().list(cal_id)
          .page_token(&token)
          .doit()
          .await?;
      } else {
          break; 
      }
    }

    Ok(result)
}
      
