use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::time::{Instant, UNIX_EPOCH};

use tokio::io::{AsyncBufReadExt, BufReader};

use crate::drum::Drum;
use crate::logger::{HitLogEntry, LogEntry, Logger};
use crate::midi_ble::MidiBle;
use crate::striker::Striker;

pub struct AutoDrum {
    midi_ble_manager: MidiBle,
    drums: HashMap<u8, Drum>,
    debug: bool,
    logger: Logger,
}

impl AutoDrum {
    pub async fn new() -> Self {
        let midi_ble_manager = MidiBle::new().await;
        let drums = HashMap::new();

        let debug = env::args().any(|arg| arg == "--debug");
        if debug {
            println!("----------- RUNNING IN DEBUG MODE -----------");
        }

        AutoDrum {
            midi_ble_manager,
            drums,
            debug,
            logger: Logger::new(),
        }
    }

    pub fn add_drum(&mut self, note: u8, pin_num: u8, name: &str, striker_kind: Striker) {
        self.drums.insert(note, Drum::new(note, pin_num, name, striker_kind));
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
                    if self.debug {
                        self.logger.save().await?;
                    }
                    break;
                },
                read_res = rx.recv() => {
                    match read_res {
                        Ok(note) => {
                            self.handle_note(note).await?
                        },
                        Err(e) => {
                            println!("Error: {:?}", e)
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
            if !self.debug {
                self.hit(note, velocity).await?;
            }
            else {
                self.hit_with_debug(note, velocity, midi_data).await?;
            }
        }
        Ok(())
    }

    pub async fn hit(&mut self, note: u8, velocity: u8) -> Result<(), Box<dyn Error>> {
        if let Some(drum) = self.drums.get_mut(&note) {
            drum.hit(velocity).await?;
        }
        Ok(())
    }

    pub async fn hit_with_debug(&mut self, note: u8, velocity: u8, midi_data: (u8, u8, u8)) -> Result<(), Box<dyn Error>> {
        if let Some(drum) = self.drums.get_mut(&note) {
            let time = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            let start = Instant::now();
            drum.hit(velocity).await?;
            let actual_duration_ns = start.elapsed().as_nanos() as u64;
            // Collect data about the hit and give it to the logger
            let hit_data = HitLogEntry {
                time,
                ms_since_last: if let Some(last_hit_time) = self.logger.last_hit_time() {
                    time - last_hit_time
                } else { 0 },
                planned_duration_ns: drum.get_strike_duration(velocity).as_nanos() as u64,
                actual_duration_ns,
                striker_kind: drum.get_striker_kind(),
                midi_data,
                note_num: note,
                velocity,
                drum_name: drum.get_name(),
                target_pin: drum.get_pin_num(),
            };
            self.logger.log(LogEntry::Hit(hit_data));
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        self.drums.iter_mut().for_each(|(_, drum)| drum.abort());
    }
}

/// Ensure no pins are left in the "on" state when the program exits
impl Drop for AutoDrum {
    fn drop(&mut self) {
        self.stop();
    }
}