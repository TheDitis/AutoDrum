use std::error::Error;
use std::future::Future;
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
    pub tx: tokio::sync::broadcast::Sender<MIDIEvent>,
    // rx: Arc<Mutex<tokio::sync::mpsc::Receiver<MIDIEvent>>>,
}

impl MidiBle {
    pub async fn new() -> MidiBle {
        let (tx, _rx) = tokio::sync::broadcast::channel::<MIDIEvent>(50);
        MidiBle {
            midi_session: bluer::Session::new().await.unwrap(),
            app_handle: None,
            advertisement_handle: None,
            tx,
            // rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub async fn init(&mut self) -> bluer::Result<()> {
        self.await_pair().await?;
        Ok(())
    }

    /// TODO: THIS DOESN'T AWAIT PAIRING, REFACTOR OR RENAME
    async fn await_pair(&mut self) -> bluer::Result<()> {
        let adapter = self.midi_session.default_adapter().await?;
        adapter.set_powered(true).await?;
        adapter.set_pairable(true).await?;
        adapter.set_discoverable(true).await?;

        println!("Advertising on Bluetooth adapter {} with address {}\n", adapter.name(), adapter.address().await?);
        let le_advertisement = Advertisement {
            advertisement_type: bluer::adv::Type::Peripheral,
            service_uuids: vec!["03B80E5A-EDE8-4B33-A751-6CE34EC4C700".parse().unwrap()].into_iter().collect(),
            // solicit_uuids: vec!["03B80E5A-EDE8-4B33-A751-6CE34EC4C700".parse().unwrap()].into_iter().collect(),
            discoverable: Some(true),
            local_name: Some("AutoDrum".to_string()),
            ..Default::default()
        };
        println!("Advertisement: {:?}\n\n", &le_advertisement);
        self.advertisement_handle = Some(adapter.advertise(le_advertisement).await?);

        // hit(4, &u8::from(0x90));
        // hit(1);
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


        // let status_byte = 0x90;  // Note on event on channel 1 (channels are 0-indexed)
        // let timestamp_byte = 0x80;  // A timestamp value. In practice, you should calculate this properly.
        // let note_number = 60;
        // let velocity = 127;
        //
        // let characteristic = MidiBler::midi_characteristic().await;
        //
        // let devices = adapter.discover_devices().await?;



        /// MINIMAL LOOP THAT STILL WORKS
        // println!("Echo service ready. Press enter to quit.");
        // let stdin = BufReader::new(tokio::io::stdin());
        // let mut lines = stdin.lines();
        //
        // loop {
        //     tokio::select! {
        //         _ = lines.next_line() => break,
        //     }
        // }


        // /// _-----------
        // /// THIS SHOULD BE PUT BACK
        // /// _-----------
        //
        // println!("Serving GATT echo service on Bluetooth adapter {}", adapter.name());
        // let (char_control, char_handle) = characteristic_control();
        //
        //
        // println!("Echo service ready. Press enter to quit.");
        // let stdin = BufReader::new(tokio::io::stdin());
        // let mut lines = stdin.lines();
        //
        // let mut read_buf = Vec::new();
        // let mut reader_opt: Option<CharacteristicReader> = None;
        // let mut writer_opt: Option<CharacteristicWriter> = None;
        // pin_mut!(char_control);
        //
        // loop {
        //     tokio::select! {
        //         _ = lines.next_line() => break,
        //         evt = char_control.next() => {
        //             match evt {
        //                 Some(CharacteristicControlEvent::Write(req)) => {
        //                     println!("Accepting write request event with MTU {}", req.mtu());
        //                     read_buf = vec![0; req.mtu()];
        //                     reader_opt = Some(req.accept()?);
        //                 },
        //                 Some(CharacteristicControlEvent::Notify(notifier)) => {
        //                     println!("Accepting notify request event with MTU {}", notifier.mtu());
        //                     writer_opt = Some(notifier);
        //                 },
        //                 None => break,
        //             }
        //         },
        //         read_res = async {
        //             match &mut reader_opt {
        //                 Some(reader) if writer_opt.is_some() => reader.read(&mut read_buf).await,
        //                 _ => future::pending().await,
        //             }
        //         } => {
        //             match read_res {
        //                 Ok(0) => {
        //                     println!("Read stream ended");
        //                     reader_opt = None;
        //                 }
        //                 Ok(n) => {
        //                     let value = read_buf[..n].to_vec();
        //                     println!("Echoing {} bytes: {:x?} ... {:x?}", value.len(), &value[0..4.min(value.len())], &value[value.len().saturating_sub(4) ..]);
        //                     if value.len() < 512 {
        //                         println!();
        //                     }
        //                     if let Err(err) = writer_opt.as_mut().unwrap().write_all(&value).await {
        //                         println!("Write failed: {}", &err);
        //                         writer_opt = None;
        //                     }
        //                 }
        //                 Err(err) => {
        //                     println!("Read stream error: {}", &err);
        //                     reader_opt = None;
        //                 }
        //             }
        //         }
        //     }
        // }
        //
        // println!("Removing service and advertisement");
        // // drop(app_handle);
        // sleep(Duration::from_secs(1)).await;

        Ok(())
    }

    async fn midi_application(&self) -> Application {
        let value = Arc::new(Mutex::new(vec![0x10, 0x01, 0x01, 0x10]));
        let value_read = value.clone();
        let value_write = value.clone();
        let value_notify = value.clone();
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
                                fun: Box::new(move |req| {
                                    let value = value_read.clone();
                                    Box::pin(async move {
                                        let value = value.lock().await.clone();
                                        println!("Read request {:?} with value {:x?}", &req, &value);
                                        Ok(value)
                                    })
                                }),
                                ..Default::default()
                            }),
                            write: Some(CharacteristicWrite {
                                write: true,
                                write_without_response: true,
                                method: CharacteristicWriteMethod::Fun(Box::new(move |new_value, req| {
                                    let value = value_write.clone();
                                    let tx = tx_clone.clone();

                                    Box::pin(async move {
                                        println!("Write request {:?} with value {:x?}", &req, &new_value);
                                        let mut value = value.lock().await;
                                        *value = new_value;
                                        if let [.., status_byte, note_number, velocity] = &value[..] {
                                            println!("status_byte: {:x?}, note: {:x?}, velocity: {:x?}", status_byte, note_number, velocity);
                                            &tx.send((status_byte.clone(), note_number.clone(), velocity.clone())).unwrap();
                                        }
                                        Ok(())
                                    })
                                })),
                                ..Default::default()
                            }),
                            notify: Some(CharacteristicNotify {
                                notify: true,
                                method: CharacteristicNotifyMethod::Fun(Box::new(move |mut notifier| {
                                    let value = value_notify.clone();
                                    Box::pin(async move {
                                        tokio::spawn(async move {
                                            println!(
                                                "Notification session start with confirming={:?}",
                                                notifier.confirming()
                                            );
                                            loop {
                                                {
                                                    let mut value = value.lock().await;
                                                    println!("Notifying with value {:x?}", &*value);
                                                    if let Err(err) = notifier.notify(value.to_vec()).await {
                                                        println!("Notification error: {}", &err);
                                                        break;
                                                    }
                                                    println!("Decrementing each element by one");
                                                    for v in &mut *value {
                                                        *v = v.saturating_sub(1);
                                                    }
                                                }
                                                sleep(Duration::from_secs(5)).await;
                                            }
                                            println!("Notification session stop");
                                        });
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
