use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use rppal::gpio::{Gpio, OutputPin};
use rppal::system::DeviceInfo;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::time;
use uuid::{Uuid, uuid};
use tokio_gpiod::{Chip, Options, Output};
use crate::midi_ble::MidiBle;
use crate::drum::Drum;

const BASE_HIT_DURATION_SMALL: f32 = 0.0002;
const BASE_HIT_DURATION_BIG: f32 = 0.005;


// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_LED: u8 = 4;

pub struct AutoDrum {
    midi_ble_manager: MidiBle,
    // drums: HashMap<u8, Drum>
}

impl AutoDrum {
    pub async fn new() -> Self {
        let mut midi_ble_manager = MidiBle::new().await;
        AutoDrum {
            midi_ble_manager,
        }
    }

    pub async fn run(&mut self) {
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
                            AutoDrum::handle_note(note).await
                        },
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
            }
        }
    }

    pub async fn handle_note(midi_data: (u8, u8, u8)) {
        let mut drum = Drum::new(GPIO_LED);
        let (status, note, velocity) = midi_data;
        if (status == 0x90) {
            drum.hit(midi_data.2).await;
        }
        // let mut pin = Gpio::new().unwrap().get(GPIO_LED).unwrap().into_output();
        //
        // if (status == 0x90) {
        //     println!("Note on: {:?} {:?}", note, velocity);
        //     pin.set_high();
        // } else if (status == 0x80) {
        //     println!("Note off: {:?} {:?}", note, velocity);
        //     pin.set_low();
        // } else {
        //     println!("Unknown status: {:?}", status);
        //     pin.set_low();
        // }
        // // Blink the LED by setting the pin's logic level high for 500 ms.
        // // tokio::time::sleep(Duration::from_millis(100)).await;
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