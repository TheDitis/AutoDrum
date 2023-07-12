use std::collections::HashMap;
use std::error::Error;
use auto_drum::autodrum::AutoDrum;
use auto_drum::striker::Striker;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = AutoDrum::new().await;

    app.add_drum(84, 4, Striker::SolenoidBig);
    app.add_drum(86, 5, Striker::SolenoidSmall);
    app.add_drum(88, 6, Striker::SolenoidSmall);
    app.add_drum(90, 7, Striker::SolenoidSmall);

    app.run().await;
    app.stop();

    Ok(())
}
