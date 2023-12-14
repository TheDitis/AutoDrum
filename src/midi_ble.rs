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

// Specified by MIDI BLE spec, these are the UUIDs for the MIDI service and characteristic and should never change
const BLE_MIDI_SERVICE_ID: Uuid = uuid!("03B80E5A-EDE8-4B33-A751-6CE34EC4C700");
const BLE_MIDI_CHARACTERISTIC_ID: Uuid = uuid!("7772E5DB-3868-4112-A1A9-F2669D106BF3");

type MIDIEvent = (u8, u8, u8);

/// Handles the sending and receiving of MIDI data over BLE, forwarding relevant MIDI events to AutoDrum
pub struct MidiBle {
    ble_session: bluer::Session,
    app_handle: Option<ApplicationHandle>,
    advertisement_handle: Option<AdvertisementHandle>,
    agent_handle: Option<AgentHandle>,
    pub tx: tokio::sync::broadcast::Sender<MIDIEvent>,
}

impl MidiBle {
    pub async fn new() -> MidiBle {
        let (tx, _rx) = tokio::sync::broadcast::channel::<MIDIEvent>(120);
        MidiBle {
            ble_session: bluer::Session::new().await.unwrap(),
            app_handle: None,
            advertisement_handle: None,
            agent_handle: None,
            tx,
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

    fn is_status_byte(byte: u8) -> bool {
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
        // Will likely need these later:
        // let value = Arc::new(Mutex::new(vec![0x10, 0x01, 0x01, 0x10]));
        // let value_read = value.clone();
        // let value_write = value.clone();
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
                                    Box::pin(async move {
                                        Ok([].to_vec())
                                    })
                                }),
                                ..Default::default()
                            }),
                            write: Some(CharacteristicWrite {
                                write: true,
                                write_without_response: true,
                                method: CharacteristicWriteMethod::Fun(Box::new(move |new_value, _req| {
                                    // let value = value_write.clone(); // Will likely need this later
                                    let tx = tx_clone.clone();

                                    Box::pin(async move {
                                        let mut last_status: u8 = 0x00;
                                        let mut midi_data: Vec<u8> = vec![];

                                        // iterate bytes (adding first status byte to end as they trigger send of previous data)
                                        for byte in new_value.iter().chain([new_value.first().unwrap()]) {
                                            // if the byte is a status or timestamp byte (non-data):
                                            if MidiBle::is_status_byte(*byte) {
                                                // if we just finished a note-on message group, send them over tx
                                                if !midi_data.is_empty() {
                                                    if last_status == 0x90 {
                                                        // split midi data into chunks of 2 bytes (note number and velocity) and send over tx (to be handled by AutoDrum)
                                                        midi_data.chunks(2).for_each(|pair| {
                                                            let note_number = pair[0];
                                                            let velocity = pair[1];
                                                            tx.send((last_status, note_number, velocity)).unwrap();
                                                        });
                                                    }
                                                    midi_data.clear();
                                                }
                                                last_status = *byte;
                                            } else {
                                                midi_data.push(*byte);
                                            }
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
