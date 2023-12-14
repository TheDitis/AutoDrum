use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs::File;
use serde::{Serialize, Deserialize};

use crate::midi_ble::MidiBle;
use crate::drum::Drum;
use crate::striker::Striker;


#[derive(Debug, Serialize, Deserialize)]
struct HitLogEntry {
    pub time: usize,
    pub time_since_last: usize,
    pub planned_hit_duration: Duration,
    pub actual_hit_duration: Duration,
    pub striker_kind: Striker,
    pub midi_data: (u8, u8, u8),
    pub target_pin: u8,
}

const BASE_HIT_DURATION_SMALL: f32 = 0.0002;
const BASE_HIT_DURATION_BIG: f32 = 0.005;


// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_LED: u8 = 4;

pub struct AutoDrum {
    midi_ble_manager: MidiBle,
    drums: HashMap<u8, Drum>,
    hit_log: Vec<HitLogEntry>,
}

impl AutoDrum {
    pub async fn new() -> Self {
        let mut midi_ble_manager = MidiBle::new().await;
        let mut drums = HashMap::new();
        AutoDrum {
            midi_ble_manager,
            drums,
            hit_log: vec![],
        }
    }

    pub fn add_drum(&mut self, note: u8, pin_num: u8, striker_kind: Striker) {
        self.drums.insert(note, Drum::new(note, pin_num, striker_kind));
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.midi_ble_manager.init().await.expect("Task panicked in MidiBle.init()");
        println!("BLE MIDI service ready. Press enter to quit.");
        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();

        let mut rx = self.midi_ble_manager.tx.subscribe();

        loop {
            tokio::select! {
                _ = lines.next_line() => {
                    // save hit log to new file:
                    let mut file = File::create(format!("./logs/hit_log_{:?}.json", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap())).await?;
                    file.write_all(serde_json::to_string(&self.hit_log).unwrap().as_bytes()).await?;
                    println!("Hit log saved to file");
                    break;
                },
                read_res = rx.recv() => {
                    match read_res {
                        Ok(note) => {
                            self.handle_note(note).await;
                        },
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
            }
        }
        Ok(())
    }


    pub async fn handle_note(&mut self, midi_data: (u8, u8, u8)) -> Result<(), Box<dyn Error>> {
        let (status, note, velocity) = midi_data;
        if status == 0x90 {
            if let Some(drum) = self.drums.get_mut(&note) {
                let start = std::time::Instant::now();
                drum.hit(velocity).await?;
                let end = std::time::Instant::now();
                /// DEBUG LOGGING
                &self.hit_log.push(HitLogEntry {
                    time: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as usize,
                    time_since_last: if self.hit_log.len() > 0 {
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as usize - self.hit_log.last().unwrap().time
                    } else { 0 },
                    planned_hit_duration: drum.get_strike_duration(velocity),
                    actual_hit_duration: end.duration_since(start),
                    striker_kind: drum.get_striker_kind(),
                    midi_data,
                    target_pin: drum.get_pin_num(),
                });
            }
        }
        Ok(())
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