use std::collections::HashMap;
use std::error::Error;
use auto_drum::autodrum::AutoDrum;
use auto_drum::striker::Striker;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = AutoDrum::new().await;

    app.add_drum(84, 4, Striker::SolenoidBig);

    app.run().await;
    app.stop();

    Ok(())
}
