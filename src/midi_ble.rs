use std::error::Error;
use std::future::Future;
use std::io::Read;
use std::pin::Pin;
use bluer::{
    adv::Advertisement,
    gatt::{
        local::{
            characteristic_control, Application, Characteristic, CharacteristicControlEvent,
            CharacteristicNotify, CharacteristicNotifyMethod, CharacteristicWrite, CharacteristicWriteMethod,
            Service, CharacteristicRead
        },
        CharacteristicReader, CharacteristicWriter,
    },
};
use std::time::Duration;
use std::sync::Arc;
use bluer::adv::AdvertisementHandle;
use bluer::agent::{Agent, AgentHandle};
use bluer::gatt::local::ApplicationHandle;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    time::sleep,
    sync::Mutex,
};
use uuid::{Uuid, uuid};
use futures::{future, pin_mut, StreamExt};

const BLE_MIDI_SERVICE_ID: Uuid = uuid!("03B80E5A-EDE8-4B33-A751-6CE34EC4C700");
const BLE_MIDI_CHARACTERISTIC_ID: Uuid = uuid!("7772E5DB-3868-4112-A1A9-F2669D106BF3");

const GPIO_LED: u8 = 4;

type MIDIEvent = (u8, u8, u8);

pub struct MidiBle {
    midi_session: bluer::Session,
    app_handle: Option<ApplicationHandle>,
    advertisement_handle: Option<AdvertisementHandle>,
    agent_handle: Option<AgentHandle>,
    pub tx: tokio::sync::broadcast::Sender<MIDIEvent>,
}

impl MidiBle {
    pub async fn new() -> MidiBle {
        let (tx, _rx) = tokio::sync::broadcast::channel::<MIDIEvent>(120);
        MidiBle {
            midi_session: bluer::Session::new().await.unwrap(),
            app_handle: None,
            advertisement_handle: None,
            agent_handle: None,
            tx,
        }
    }

    pub async fn init(&mut self) -> bluer::Result<()> {
        self.await_pair().await?;
        Ok(())
    }

    fn is_status_byte(byte: u8) -> bool {
        byte & 0b1000_0000 != 0
    }

    /// TODO: THIS DOESN'T AWAIT PAIRING, REFACTOR OR RENAME
    async fn await_pair(&mut self) -> bluer::Result<()> {
        self.agent_handle = Some(
            self.midi_session.register_agent(Agent {
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

        let adapter = self.midi_session.default_adapter().await?;
        adapter.set_powered(true).await?;
        adapter.set_pairable(true).await?;
        adapter.set_discoverable(true).await?;


        println!("Advertising on Bluetooth adapter {} with address {}\n", adapter.name(), adapter.address().await?);
        let le_advertisement = Advertisement {
            advertisement_type: bluer::adv::Type::Peripheral,
            service_uuids: vec!["03B80E5A-EDE8-4B33-A751-6CE34EC4C700".parse().unwrap()].into_iter().collect(),
            discoverable: Some(true),
            local_name: Some("AutoDrum".to_string()),
            min_interval: Some(Duration::from_millis(15)),
            max_interval: Some(Duration::from_millis(15)),
            ..Default::default()
        };
        println!("Advertisement: {:?}\n\n", &le_advertisement);
        self.advertisement_handle = Some(adapter.advertise(le_advertisement).await?);

        match self.tx.send((1, 1, 1)) {
            Ok(_) => println!("Sent over tx"),
            Err(e) => println!("Error sending over tx: {}", e),
        }

        self.setup_midi_gatt_service().await?;

        Ok(())
    }

    async fn setup_midi_gatt_service(&mut self) -> bluer::Result<()> {
        let adapter = self.midi_session.default_adapter().await?;
        let application = self.midi_application().await;

        self.app_handle = Some(adapter.serve_gatt_application(application).await?);
        Ok(())
    }

    async fn midi_application(&self) -> Application {
        let value = Arc::new(Mutex::new(vec![0x10, 0x01, 0x01, 0x10]));
        let value_read = value.clone();
        let value_write = value.clone();
        let value_notify = value.clone();
        let tx_clone = self.tx.clone();

        let mut session = self.midi_session.clone();
        let mut adapter = session.default_adapter().await.unwrap().clone();

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
                                fun: Box::new(move |req| {
                                    let value = value_read.clone();
                                    let adapter = adapter.clone();
                                    Box::pin(async move {
                                        let value = value.lock().await.clone();
                                        let adapter = adapter.clone();

                                        // If it's a pairing request, try to pair and trust
                                        // TODO: Handle case where another device is already connected
                                        if vec![16, 1, 1, 16] == value {
                                            println!("Pairing request");
                                            let device = adapter.device(req.device_address).unwrap();
                                            if let Err(err) = device.pair().await {
                                                println!("Did not pair: {:?}", err.message);
                                            } else {
                                                println!("Paired!");
                                            }
                                            if let Err(err) = device.set_trusted(true).await {
                                                println!("Did not trust: {:?}", err.message);
                                            } else {
                                                println!("Trusted!");
                                            }
                                        } else {
                                            println!("Not pairing request");
                                        }
                                        println!("Read request {:?} with value {:x?}", &req, &value);
                                        Ok([].to_vec())
                                    })
                                }),
                                ..Default::default()
                            }),
                            write: Some(CharacteristicWrite {
                                write: true,
                                write_without_response: true,
                                method: CharacteristicWriteMethod::Fun(Box::new(move |new_value, req| {

                                    println!("\n\n[{:?}]: New write request at", std::time::SystemTime::now());
                                    println!("Write request {:?} with value {:x?}", &req, &new_value);

                                    let value = value_write.clone();
                                    let tx = tx_clone.clone();

                                    Box::pin(async move {
                                        let mut last_status: u8 = 0x00;
                                        let mut midi_data: Vec<u8> = vec![];

                                        // iterate bytes (adding first status byte to end as they trigger send of previous data)
                                        for byte in new_value.iter().chain([new_value.first().unwrap()]) {
                                            // if the byte is a status or timestamp byte (non-data):
                                            if MidiBle::is_status_byte(*byte) {
                                                // if we just finished a note-on message group, send them over tx
                                                if midi_data.len() > 0 {
                                                    if last_status == 0x90 {

                                                        midi_data.chunks(2).for_each(|pair| {
                                                            println!("Pair: {:?}", pair);
                                                            let note_number = pair[0];
                                                            let velocity = pair[1];
                                                            println!("[{:?}]: Sending message on tx: status_byte: {:x?}, note: {:x?}, velocity: {:x?}", std::time::SystemTime::now(), last_status, note_number, velocity);
                                                            &tx.send((last_status.clone(), note_number.clone(), velocity.clone())).unwrap();
                                                        });
                                                    }
                                                    midi_data.clear();
                                                }
                                                last_status = *byte;
                                            } else {
                                                // TODO: HERE, split data into pairs and send each pair over tx
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
                                method: CharacteristicNotifyMethod::Fun(Box::new(move |mut notifier| {
                                    println!("Notification session start with confirming={:?}", notifier.confirming());
                                    Box::pin(async move {
                                        println!("Notification guy run")
                                    })
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
