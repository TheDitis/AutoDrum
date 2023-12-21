use std::sync::{Arc, Mutex};
use std::time::Duration;

use bluer::{
    adv::Advertisement,
    gatt::{
        local::{
            Application, Characteristic,
            CharacteristicNotify, CharacteristicNotifyMethod, CharacteristicRead, CharacteristicWrite,
            CharacteristicWriteMethod, Service
        },
    },
};
use bluer::adv::AdvertisementHandle;
use bluer::agent::{Agent, AgentHandle};
use bluer::gatt::local::ApplicationHandle;
use uuid::{Uuid, uuid};
use crate::remote_command::Command;
use rand::Rng;

// Specified by MIDI BLE spec, these are the UUIDs for the MIDI service and characteristic and should never change
const BLE_MIDI_SERVICE_ID: Uuid = uuid!("03B80E5A-EDE8-4B33-A751-6CE34EC4C700");
const BLE_MIDI_CHARACTERISTIC_ID: Uuid = uuid!("7772E5DB-3868-4112-A1A9-F2669D106BF3");

type MIDIEvent = (u8, u8, u8);

/// Handles the sending and receiving of MIDI data over BLE, forwarding relevant MIDI events to AutoDrum
pub struct MidiBle {
    /// The BLE session object from bluer (BlueZ wrapper)
    ble_session: bluer::Session,
    /// The handle to the BlueZ agent that handles pairing and authorization
    agent_handle: Option<AgentHandle>,
    /// The handle to the BLE advertisement that advertises the MIDI service
    advertisement_handle: Option<AdvertisementHandle>,
    /// The handle to the GATT application that serves the MIDI service
    app_handle: Option<ApplicationHandle>,
    /// The tokio channel to send MIDI events to the main AutoDrum application
    pub tx: tokio::sync::broadcast::Sender<Command>,
    /// The value of the MIDI characteristic
    read_value: Arc<Mutex<Vec<u8>>>,
}

impl MidiBle {
    pub async fn new() -> MidiBle {
        let ble_session = bluer::Session::new().await.unwrap();
        let (tx, _rx) = tokio::sync::broadcast::channel::<Command>(120);
        let read_value = Arc::new(Mutex::new(vec![0x00, 0x00, 0x00, 0x00]));
        MidiBle {
            ble_session,
            agent_handle: None,
            advertisement_handle: None,
            app_handle: None,
            tx,
            read_value,
        }
    }

    /// Initialize the BLE MIDI service
    ///
    /// 1. Register the agent (as default and with no PIN code)
    /// 2. Make sure the adapter is powered on and ready to go, but not discoverable or pairable outside the midi service
    /// 3. Advertise the MIDI service with a low connection interval (specified <15ms by MIDI BLE spec)
    /// 4. Serve the GATT application
    pub async fn init(&mut self) -> bluer::Result<()> {
        // Register the agent (as default and with no PIN code)
        self.agent_handle = Some(
            self.ble_session.register_agent(Agent {
                request_default: true,
                request_pin_code: None,
                display_pin_code: None,
                request_passkey: None,
                display_passkey: None,
                request_confirmation: None,
                request_authorization: None,
                authorize_service: None,
                ..Default::default()
            }).await?
        );

        // Make sure the adapter is powered on and ready to go, but not discoverable or pairable outside the midi service
        let adapter = self.ble_session.default_adapter().await?;
        adapter.set_powered(true).await?;
        adapter.set_pairable(false).await?;
        adapter.set_discoverable(false).await?;
        adapter.set_alias("AutoDrum".to_string()).await?;

        // Advertise the MIDI service with a low connection interval (specified <15ms by MIDI BLE spec)
        println!("Advertising on Bluetooth adapter {} with address {}", adapter.name(), adapter.address().await?);
        let le_advertisement = Advertisement {
            advertisement_type: bluer::adv::Type::Peripheral,
            service_uuids: vec!["03B80E5A-EDE8-4B33-A751-6CE34EC4C700".parse().unwrap()].into_iter().collect(),
            discoverable: Some(true),
            local_name: Some("AutoDrum".to_string()),
            min_interval: Some(Duration::from_millis(5)),
            max_interval: Some(Duration::from_millis(5)),
            ..Default::default()
        };
        println!("Advertisement: {:#?}\n\n", &le_advertisement);
        self.advertisement_handle = Some(adapter.advertise(le_advertisement).await?);

        // Serve the GATT application
        let application = self.midi_application().await;
        self.app_handle = Some(adapter.serve_gatt_application(application).await?);

        Ok(())
    }

    /// Send MIDI data over BLE by updating the MIDI characteristic value
    pub fn send(&mut self, data: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut value_array = self.read_value.lock().unwrap();
        value_array.clear();
        // create 2 stamp bytes to add to the beginning of the message
        let mut rng = rand::thread_rng();
        let stamp: Vec<u8> = vec![rng.gen_range(1..255), rng.gen_range(1..255)];
        value_array.extend(stamp);
        value_array.extend(data.as_bytes().to_vec());
        Ok(())
    }

    /// Check if a given byte is a status byte (note-on, note-off, aftertouch, etc.)
    pub fn is_status_byte(byte: u8) -> bool {
        byte & 0b1000_0000 != 0
    }

    /// Create the GATT application for the MIDI service
    ///
    /// Characteristics:
    /// - Read: Empty. Characteristic is required but not used. May be used in the future
    /// - Write: MIDI data. This is the characteristic that will be used to send received MIDI data
    ///     to the core AutoDrum application. Currently only sends note-on messages, as duration is
    ///     handled by the AutoDrum application.
    /// - Notify: Empty. Characteristic is required but not used.
    async fn midi_application(&self) -> Application {
        let value_read = self.read_value.clone();
        // Might need this again later:
        // let value_write = self.read_value.clone();
        let tx_clone = self.tx.clone();

        Application {
            services: vec![
                Service {
                    uuid: BLE_MIDI_SERVICE_ID,
                    primary: true,
                    characteristics: vec![
                        Characteristic {
                            uuid: BLE_MIDI_CHARACTERISTIC_ID,
                            broadcast: true,
                            authorize: true,
                            read: Some(CharacteristicRead {
                                read: true,
                                fun: Box::new(move | _req | {
                                    let value_read = value_read.clone();
                                    Box::pin(async move {
                                        let send_value = value_read.lock().unwrap().clone();
                                        println!("Send value: {:?}", send_value);
                                        Ok(send_value)
                                    })
                                }),
                                ..Default::default()
                            }),
                            write: Some(CharacteristicWrite {
                                write: true,
                                write_without_response: true,
                                method: CharacteristicWriteMethod::Fun(Box::new(move |new_value, _req| {
                                    println!("Write value: {:?}", new_value);
                                    let tx = tx_clone.clone();
                                    Box::pin(async move {
                                        if let Ok(command) = Command::try_from(&new_value) {
                                            tx.send(command.clone()).unwrap();
                                        }
                                        Ok(())
                                    })
                                })),
                                ..Default::default()
                            }),
                            notify: Some(CharacteristicNotify {
                                notify: true,
                                method: CharacteristicNotifyMethod::Fun(Box::new(move | _notifier | {
                                    Box::pin(async move {})
                                })),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }
                    ],
                    ..Default::default()
                }
            ],
            ..Default::default()
        }
    }

}
