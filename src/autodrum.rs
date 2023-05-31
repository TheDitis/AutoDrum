use std::collections::HashMap;
use std::error::Error;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use rppal::gpio::{Gpio, OutputPin};
use rppal::system::DeviceInfo;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use uuid::{Uuid, uuid};
use crate::midi_ble::MidiBle;
use crate::drum::Drum;

const BASE_HIT_DURATION_SMALL: f32 = 0.0002;
const BASE_HIT_DURATION_BIG: f32 = 0.005;


// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_LED: u8 = 4;

pub struct AutoDrum {
    midi_ble_manager: MidiBle,
    drums: HashMap<u8, Drum>
}

impl AutoDrum {
    pub async fn new() -> Self {
        let mut midi_ble_manager = MidiBle::new().await;
        let mut drums = HashMap::new();
        drums.insert(84, Drum::new(4));
        AutoDrum {
            midi_ble_manager,
            drums,
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
                            self.handle_note(note).await
                        },
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
            }
        }
    }

    pub async fn handle_note(&mut self, midi_data: (u8, u8, u8)) {
        let (status, note, velocity) = midi_data;
        if status == 0x90 {
            if let Some(drum) = self.drums.get_mut(&note) {
                drum.hit(velocity).await;
            }
        }
    }

    pub fn stop(&mut self) {
        self.drums.iter_mut().for_each(|(_, drum)| drum.abort());
    }
}
impl Drop for AutoDrum {
    fn drop(&mut self) {
        self.stop();
    }
}