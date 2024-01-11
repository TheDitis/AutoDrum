pub const MIDI_NOTE_ON_BYTE: u8 = 0x90;
pub const MIDI_NOTE_OFF_BYTE: u8 = 0x80;
pub const READ_SYSTEM_CONSTANTS_COMMAND_BYTE: u8 = 0x00;
pub const READ_CONFIG_COMMAND_BYTE: u8 = 0x01;
pub const WRITE_CONFIG_COMMAND_BYTE: u8 = 0x02;

/// Represents a general command received from the remote
#[derive(Debug, Clone)]
pub enum Command {
    MIDI(Vec<u8>),
    ReadSystemConstants(Vec<u8>),
    ReadConfiguration(Vec<u8>),
    WriteConfiguration(Vec<u8>),
}

impl TryFrom<&Vec<u8>> for Command {
    type Error = String;

    fn try_from(message: &Vec<u8>) -> Result<Self, Self::Error> {
        if message.len() < 3 {
            return Err("Message too short".to_string());
        }
        match message.get(2).unwrap().clone() {
            MIDI_NOTE_ON_BYTE | MIDI_NOTE_OFF_BYTE => Ok(Command::MIDI(message.clone())),
            READ_SYSTEM_CONSTANTS_COMMAND_BYTE => Ok(Command::ReadSystemConstants(message.clone())),
            READ_CONFIG_COMMAND_BYTE => Ok(Command::ReadConfiguration(message.clone())),
            WRITE_CONFIG_COMMAND_BYTE => Ok(Command::WriteConfiguration(message.clone())),
            _ => Err("Unknown command".to_string()),
        }
    }
}
