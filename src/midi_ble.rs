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
                                        if vec![10, 1, 1, 10] == value { // .iter().enumerate().all(|(i, v)| ) {// matches!(value.as_slice(), [10, 1, 1, 10]) {
                                            println!("Pairing request");
                                            let device = adapter.device(req.device_address).unwrap();
                                            device.pair().await.unwrap();
                                            println!("Paired");
                                            device.set_trusted(true).await.unwrap();
                                            println!("Trusted");
                                        }
                                        println!("Read request {:?} with value {:x?}", &req, &value);
                                        // Ok(value)
                                        Ok([].to_vec())
                                    })
                                }),
                                ..Default::default()
                            }),
                            write: Some(CharacteristicWrite {
                                write: true,
                                write_without_response: true,
                                method: CharacteristicWriteMethod::Fun(Box::new(move |new_value, req| {
                                    if new_value[2] != 0x90 {
                                        return Box::pin(async move {
                                            Ok(())
                                        })
                                    }
                                    println!("\n\n[{:?}]: New write request at", std::time::SystemTime::now());
                                    println!("Write request {:?} with value {:x?}", &req, &new_value);

                                    let value = value_write.clone();
                                    let tx = tx_clone.clone();

                                    Box::pin(async move {
                                        println!("Write request {:?} with value {:x?}", &req, &new_value);
                                        let mut value = value.lock().await;
                                        *value = new_value;
                                        let status_byte = value[2];

                                        // Excluding the timestamp bytes and status byte, iterate over pairs of note number and velocity
                                        value[3..].chunks(2).for_each(|pair| {
                                            println!("Pair: {:?}", pair);
                                            let note_number = pair[0];
                                            let velocity = pair[1];
                                            println!("[{:?}]: Sending message on tx: status_byte: {:x?}, note: {:x?}, velocity: {:x?}", std::time::SystemTime::now(), status_byte, note_number, velocity);
                                            &tx.send((status_byte.clone(), note_number.clone(), velocity.clone())).unwrap();
                                        });
                                        Ok(())
                                    })
                                })),
                                ..Default::default()
                            }),
                            notify: Some(CharacteristicNotify {
                                notify: true,
                                method: CharacteristicNotifyMethod::Fun(Box::new(move |mut notifier| {
                                    // let value = value_notify.clone();
                                    println!("Notification session start with confirming={:?}", notifier.confirming());
                                    Box::pin(async move {
                                        // tokio::spawn(async move {
                                        //     println!(
                                        //         "Notification session start with confirming={:?}",
                                        //         notifier.confirming()
                                        //     );
                                        //     loop {
                                        //         {
                                        //             let mut value = value.lock().await;
                                        //             println!("Notifying with value {:x?}", &*value);
                                        //             if let Err(err) = notifier.notify(value.to_vec()).await {
                                        //                 println!("Notification error: {}", &err);
                                        //                 break;
                                        //             }
                                        //             println!("Decrementing each element by one");
                                        //             for v in &mut *value {
                                        //                 *v = v.saturating_sub(1);
                                        //             }
                                        //         }
                                        //         sleep(Duration::from_secs(5)).await;
                                        //     }
                                        //     println!("Notification session stop");
                                        // });
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
