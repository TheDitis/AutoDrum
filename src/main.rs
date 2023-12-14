use std::error::Error;
use auto_drum::autodrum::AutoDrum;
use auto_drum::striker::Striker;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = AutoDrum::new().await;

    // Note number reference here: https://computermusicresource.com/midikeys.html
    // app.add_drum(84, 4, "BROKEN MOSFET CHANNEL" Striker::SolenoidBig); // C5
    app.add_drum(36, 7, "Kick", Striker::SolenoidBig); // C1
    app.add_drum(37, 6, "Snare", Striker::SolenoidSmall); // C#1
    app.add_drum(38, 5, "HiHat", Striker::SolenoidSmall); // D1

    app.run().await;
    app.stop();

    Ok(())
}
