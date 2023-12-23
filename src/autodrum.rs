use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::time::{Instant, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::drum::{Drum, DrumRaw};
use crate::logger::{HitLogEntry, LogEntry, Logger};
use crate::midi_ble::MidiBle;
use crate::modifier::{Modifier, ModifierHardwareKind};
use crate::remote_command::Command;
use crate::striker_hardware_util::StrikerHardwareKind;

#[derive(Serialize, Deserialize)]
struct Configuration {
    drums: Vec<DrumRaw>,
}

/// Main application struct
pub struct AutoDrum {
    /// The BLE MIDI manager that brings us any relevant MIDI data sent to the BLE MIDI service
    midi_ble_manager: MidiBle,
    /// A map of drum names to their respective MIDI note numbers (mainly for linking modifiers to drums with human-readable names)
    drum_name_to_note: HashMap<String, u8>,
    /// A map of MIDI note numbers to their respective drums
    drums: HashMap<u8, Drum>,
    /// A map of MIDI note numbers to their respective modifiers
    /// Modifiers are hardware outputs (Strikers) that change the behavior of a drum, e.g. "open" for a hi-hat
    modifiers: HashMap<u8, Modifier>,
    /// A map of MIDI note numbers for a given modified hit to the MIDI note number of the drum that the modifier is modifying
    modifier_targets: HashMap<u8, u8>,
    /// A map of drum note numbers to a vec of their respective modifier note numbers
    drum_modifiers: HashMap<u8, Vec<u8>>,
    /// Whether or not to collect log data to save on exit
    debug: bool,
    /// The logger that collects and saves log data
    logger: Logger,
}

impl AutoDrum {
    /// Create a new AutoDrum instance
    pub async fn new() -> Self {
        let midi_ble_manager = MidiBle::new().await;
        let drum_name_to_note = HashMap::new();
        let drums = HashMap::new();
        let modifiers = HashMap::new();
        let modifier_targets = HashMap::new();
        let drum_modifiers = HashMap::new();

        let debug = env::args().any(|arg| arg == "--debug");
        if debug {
            println!(
                "\n---------------------------------------------\
                 \n          - RUNNING IN DEBUG MODE -         \
                 \n---------------------------------------------\n"
            );
        }

        AutoDrum {
            midi_ble_manager,
            drum_name_to_note,
            drums,
            modifiers,
            modifier_targets,
            drum_modifiers,
            debug,
            logger: Logger::new(),
        }
    }

    /// Ensure that a given note number is not already in use by a drum or modifier
    pub fn enforce_unique_note_num(&mut self, note: u8) -> Result<(), Box<dyn Error>> {
        if self.drums.contains_key(&note) {
            return Err(format!("Drum with note number {} already exists", note).into());
        } else if self.modifiers.contains_key(&note) {
            return Err(format!("Modifier with note number {} already exists", note).into());
        }
        Ok(())
    }

    /// Ensure that a given name is not already in use by a drum
    pub fn enforce_unique_name(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        if self.drum_name_to_note.contains_key(name) {
            return Err(format!("Drum with name {} already exists", name).into());
        }
        Ok(())
    }

    /// Add a new drum to the AutoDrum instance
    pub fn add_drum(
        &mut self,
        note: u8,
        pin_num: u8,
        name: &str,
        striker_kind: StrikerHardwareKind,
    ) -> Result<(), Box<dyn Error>> {
        self.enforce_unique_note_num(note)?;
        self.enforce_unique_name(name)?;
        self.drum_name_to_note.insert(name.to_string(), note);
        self.drums.insert(note, Drum::new(note, pin_num, name, striker_kind));
        Ok(())
    }

    /// Add a new modifier to the AutoDrum instance
    pub fn add_modifier(
        &mut self,
        target_drum_name: &str,
        note: u8,
        pin_num: u8,
        name: &str,
        striker_kind: ModifierHardwareKind,
    ) -> Result<(), Box<dyn Error>> {
        self.enforce_unique_note_num(note)?;
        self.enforce_unique_name(name)?;
        self.modifiers.insert(note, Modifier::new(name, note, pin_num, striker_kind));
        self.modifier_targets.insert(note, self.drum_name_to_note.get(target_drum_name).ok_or(format!("No drum with name {} exists", target_drum_name))?.clone());
        Ok(())
    }

    /// The main loop of the AutoDrum application
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.midi_ble_manager.init().await.expect("Task panicked in MidiBle.init()");
        println!("BLE MIDI service ready. Press enter to quit.");
        let stdin = BufReader::new(tokio::io::stdin());
        // Get a stream of lines from stdin
        let mut lines = stdin.lines();
        // Get a receiver for the MIDI BLE manager's tx channel so we can receive MIDI events
        let mut rx = self.midi_ble_manager.tx.subscribe();

        loop {
            tokio::select! {
                // If we get a line from stdin, exit the program
                _ = lines.next_line() => {
                    if self.debug {
                        self.logger.save().await?;
                    }
                    break;
                },
                // If we get a command from the MIDI BLE manager, route it to the appropriate handler
                read_res = rx.recv() => {
                    match read_res {
                        Ok(command) => {
                            self.route_command(&command).await?
                        },
                        Err(e) => {
                            eprintln!("Error: {:?}", e)
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Route a command to the appropriate handler
    pub async fn route_command(&mut self, command: &Command) -> Result<(), Box<dyn Error>> {
        match command {
            Command::MIDI(new_value) => self.handle_midi_command(new_value).await?,
            Command::ReadConfiguration(new_value) => self.handle_read_configuration_command(new_value)?,
            Command::WriteConfiguration(new_value) => {
                println!("Received write configuration command: {:?}", new_value);
                // if &new_value.first().unwrap().clone() == &WRITE_CONFIG_COMMAND_BYTE {
                //     response_value.lock().unwrap().clear();
                //     let return_value = vec![0x10, 0x01, 0x01, 0x10];
                //     response_value.lock().unwrap().extend(return_value);
                //     return Ok(());
                // }
            },
        }
        Ok(())
    }


    //--------------------------------------------------------------------------------
    // COMMAND HANDLERS (called by route_command)
    //--------------------------------------------------------------------------------

    /// Parse a MIDI command into its subcommands and handle each one
    pub async fn handle_midi_command(&mut self, message_data: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        println!("Received MIDI command: {:?}", message_data);
        let mut last_status: u8 = 0x00;
        let mut midi_data: Vec<u8> = vec![];
        // iterate bytes (adding first status byte to end as they trigger send of previous data)
        for byte in message_data.iter().chain([message_data.first().unwrap()]) {
            // if the byte is a status or timestamp byte (non-data):
            if MidiBle::is_status_byte(*byte) {
                // if we just finished a note-on or note-off message group, send them over tx
                if !midi_data.is_empty() {
                    if last_status == 0x90 || last_status == 0x80 {
                        // split midi data into chunks of 2 bytes (note number and velocity) and send over tx (to be handled by AutoDrum)
                        for pair in midi_data.chunks(2) {
                            let note_number = pair[0];
                            let velocity = pair[1];
                            self.handle_note(
                                (last_status, note_number, velocity)
                            ).await?;
                        }
                    }
                    midi_data.clear();
                }
                last_status = *byte;
            } else {
                midi_data.push(*byte);
            }
        }
        Ok(())
    }

    /// Collect & serialize the current configuration of the AutoDrum instance then send it over BLE
    fn handle_read_configuration_command(&mut self, value: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        println!("Received read configuration command: {:?}", value);
        let config = Configuration {
            drums: self.drums.iter().map(|(_, drum)| drum.export_raw()).collect(),
        };
        let stringified_config = serde_json::to_string(&config).unwrap();
        self.midi_ble_manager.send(&stringified_config)
    }


    //--------------------------------------------------------------------------------
    // MIDI HANDLERS (downstream of handle_midi_command)
    //--------------------------------------------------------------------------------

    /// Handle a note-on or note-off event
    pub async fn handle_note(&mut self, midi_data: (u8, u8, u8)) -> Result<(), Box<dyn Error>> {
        let (status, note, velocity) = midi_data;
        // If it's a note on event, hit the drum
        if status == 0x90 {
            if !self.debug {
                self.hit(note, velocity).await?;
            }
            else {
                self.hit_with_debug(note, velocity, midi_data).await?;
            }
        }
        // If it's a note off event and the note is a modifier, deactivate it
        else if status == 0x80 {
            if let Some(modifier) = self.modifiers.get_mut(&note) {
                modifier.deactivate();
            }
        }
        Ok(())
    }

    /// Hit a drum, activating any modifiers linked to the given note in tandem
    pub async fn hit(&mut self, note: u8, velocity: u8) -> Result<(), Box<dyn Error>> {
        // If hitting the drum directly, not a modified version of it:
        if let Some(drum) = self.drums.get_mut(&note) {
            // Deactivate any modifiers that are currently active for this drum
            if self.drum_modifiers.contains_key(&note) {
                for modifier_note in self.drum_modifiers.get(&note).unwrap() {
                    if let Some(modifier) = self.modifiers.get_mut(modifier_note) {
                        modifier.deactivate();
                    }
                }
            }
            // Hit the drum
            drum.hit(velocity).await?;
        }
        // If hitting a modified version of a drum:
        else if let Some(modifier) = self.modifiers.get_mut(&note) {
            if let Some(target_note) = self.modifier_targets.get(&note) {
                if let Some(target_drum) = self.drums.get_mut(target_note) {
                    // Activate the modifier, then hit the drum, then start a timer to deactivate the modifier
                    modifier.activate();
                    // TODO: May need to add a delay here to ensure the modifier has time to activate before the drum is hit
                    // modifier.start_deactivation_timer() // May need to add this back in
                    target_drum.hit(velocity).await?;
                }
            }
        }
        Ok(())
    }

    /// Hit a drum, logging data about the hit
    /// TODO: add modifiers to logging once they're fully implemented
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

    //--------------------------------------------------------------------------------
    // CONFIGURATION HELPERS
    //--------------------------------------------------------------------------------

    /// Export the current configuration of the AutoDrum instance
    fn export_configuration(&self) -> Configuration {
        let mut drums = vec![];
        for (note, drum) in self.drums.iter() {
            drums.push(DrumRaw {
                name: drum.get_name(),
                note: *note,
                pin: drum.get_pin_num(),
                striker: drum.get_striker_kind(),
                min_hit_duration: drum.get_min_hit_duration(),
                max_hit_duration: drum.get_max_hit_duration(),
            });
        }
        Configuration {
            drums,
        }
    }

    /// Load a configuration into the AutoDrum instance
    fn load_configuration(&mut self, config: Configuration) {
        for drum in config.drums {
            self.add_drum(drum.note, drum.pin, &drum.name, drum.striker).unwrap();
        }
    }

    /// Save the current configuration of the AutoDrum instance to a file
    pub async fn save_configuration_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let config = self.export_configuration();
        let stringified_config = serde_json::to_string(&config).unwrap();
        let mut file = File::create(path).await?;
        file.write_all(stringified_config.as_bytes()).await?;
        Ok(())
    }

    //--------------------------------------------------------------------------------
    // LIFE CYCLE FUNCTIONS
    //--------------------------------------------------------------------------------

    /// Make sure no pins are left in the "on" state when the program exits
    pub fn stop(&mut self) {
        self.drums.iter_mut().for_each(|(_, drum)| drum.abort());
        self.modifiers.iter_mut().for_each(|(_, modifier)| modifier.deactivate());
    }
}

/// Ensure no pins are left in the "on" state when the program exits
impl Drop for AutoDrum {
    fn drop(&mut self) {
        self.stop();
    }
}