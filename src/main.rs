use std::collections::HashMap;
use std::error::Error;
use auto_drum::autodrum::AutoDrum;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = AutoDrum::new().await;
    app.run().await;
    app.stop();

    Ok(())
}
