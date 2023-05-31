use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::thread;
use std::time::Duration;

use rppal::gpio::Gpio;
use rppal::system::DeviceInfo;

use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures::TryFutureExt;
use sysfs_gpio::{Pin as GPIOPin};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::time;
use uuid::{Uuid, uuid};
use auto_drum::midi_ble::MidiBle;
use tokio_gpiod::{Chip, Options, Output};

const BASE_HIT_DURATION_SMALL: f32 = 0.0002;
const BASE_HIT_DURATION_BIG: f32 = 0.005;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_LED: u8 = 4;

struct AutoDrum {
    midi_ble_manager: MidiBle,
}

impl AutoDrum {
    pub async fn new() -> Self {
        let mut midi_ble_manager = MidiBle::new().await;
        // tokio::task::spawn_blocking(move || {
        AutoDrum {
            midi_ble_manager,
        }
    }

    pub async fn run(&mut self) {
        // let hit_func = |note| async {
        //     // let mut pin = Gpio::new().unwrap().get(GPIO_LED).unwrap().into_output();
        //     //
        //     // // Blink the LED by setting the pin's logic level high for 500 ms.
        //     // pin.set_high();
        //     // thread::sleep(Duration::from_millis(100));
        //     // pin.set_low();
        //     AutoDrum::handle_note(note)
        // };


        self.midi_ble_manager.init().await.expect("Task panicked in MidiBle.init()");
        println!("Echo service ready. Press enter to quit.");
        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();

        let mut rx = self.midi_ble_manager.tx.subscribe();

        loop {
            tokio::select! {
                _ = lines.next_line() => break,
                read_res = rx.recv() => {
                    match read_res {
                        Ok(note) => {
                            println!("Received note: {:?}", note);
                            AutoDrum::handle_note(note).await;
                        },
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
            }
        }

        // hit_func(1);
    }

    pub async fn handle_note(note: (u8, u8, u8)) {
        let mut pin = Gpio::new().unwrap().get(GPIO_LED).unwrap().into_output();

        // Blink the LED by setting the pin's logic level high for 500 ms.
        pin.set_high();
        tokio::time::sleep(Duration::from_millis(100)).await;
        pin.set_low();
    }

    pub fn stop(&mut self) {
        let mut pin = Gpio::new().unwrap().get(GPIO_LED).unwrap().into_output();
        pin.set_low();
    }
}
impl Drop for AutoDrum {
    fn drop(&mut self) {
        self.stop();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let hit_func = || {
    //     let mut pin = Gpio::new().unwrap().get(GPIO_LED).unwrap().into_output();
    //
    //     // Blink the LED by setting the pin's logic level high for 500 ms.
    //     pin.set_high();
    //     thread::sleep(Duration::from_millis(100));
    //     pin.set_low();
    // };
    //
    // // // setup_midi_bluetooth(hit_func).await?;
    // let mut midi_ble_manager = MidiBle::new().await;
    // // tokio::task::spawn_blocking(move || {
    // midi_ble_manager.init(hit_func).await;
    // //     /// TOOO: HERE, seems like passing the function to turn on/off gpio works
    // // }).await.expect("Task panicked");
    //
    //
    // println!("Echo service ready. Press enter to quit.");
    // let stdin = BufReader::new(tokio::io::stdin());
    // let mut lines = stdin.lines();
    //
    // loop {
    //     tokio::select! {
    //         _ = lines.next_line() => break,
    //     }
    // }
    //
    // hit_func();
    //
    //
    //
    //
    // // let mut pin = Gpio::new()?.get(GPIO_LED)?.into_output();
    // //
    // // // Blink the LED by setting the pin's logic level high for 500 ms.
    // // pin.set_high();
    // // thread::sleep(Duration::from_millis(500));
    // // pin.set_low();
    let mut app = AutoDrum::new().await;
    app.run().await;
    app.stop();

    Ok(())
}
