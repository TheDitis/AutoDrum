use std::error::Error;
use auto_drum::autodrum::AutoDrum;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = AutoDrum::new().await;

    // Note number reference here: https://computermusicresource.com/midikeys.html

    app.run().await?;
    app.stop();

    Ok(())
}
