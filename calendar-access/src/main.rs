use google_calendar3::Error;
use yup_oauth2 as oauth2;

mod calendar;

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

    let mut event_struct = calendar::Events::new(auth);
    event_struct.load_main().await?;
    event_struct.load_filtered().await?;
    event_struct.create_references();
    event_struct.delete_filtered_events().await?;

    let events = event_struct.events_with_description();
    for evt in events {
        if let Some(filter_id) = event_struct.filtered_event_for(evt.id.as_ref().unwrap()) {
            event_struct.update_filtered_event(&evt, &filter_id).await?;
        } else {
            event_struct.add_filtered_event(&evt).await?;
        }
    }
    Ok(())
}
