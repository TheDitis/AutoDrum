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
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    time::sleep,
    sync::Mutex,
};
use uuid::{Uuid, uuid};
use futures::{future, pin_mut, StreamExt};
// use btleplug::api::{Characteristic, CharPropFlags};
// use btleplug::api::{bleuuid::uuid_from_u16, Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
// use btleplug::platform::{Adapter, Manager, Peripheral};

const BLE_MIDI_SERVICE_ID: Uuid = uuid!("03B80E5A-EDE8-4B33-A751-6CE34EC4C700");
const BLE_MIDI_CHARACTERISTIC_ID: Uuid = uuid!("7772E5DB-3868-4112-A1A9-F2669D106BF3");

pub struct MidiBle {
    midi_session: bluer::Session,
    advertisement_handle: Option<AdvertisementHandle>,
    // midi_application: Application,
    // midi_service: Service,
    // midi_characteristic: Characteristic,
}

impl MidiBle {
    pub async fn new() -> MidiBle {
        MidiBle {
            midi_session: bluer::Session::new().await.unwrap(),
            advertisement_handle: None,
        }
    }

    #[tokio::main]
    pub async fn init(&mut self) -> bluer::Result<()> {
        self.await_pair().await?;
        Ok(())
    }

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
        self.setup_midi_gatt_service().await?;


        // let manager = Manager::new().await.unwrap();
        //
        // // get the first bluetooth adapter
        // let adapters = manager.adapters().await;
        // let central = adapters.into_iter().nth(0).unwrap()[0].clone();
        // // for _i in 0..50 {
        // //     let peripherals = central.peripherals().await.unwrap();
        // //     println!("peripherals:");
        // //     for peripheral in peripherals {
        // //         println!("peripheral: {:?}", peripheral.properties().await);
        // //     }
        // //     sleep(Duration::from_secs(1)).await;
        // // }
        //
        // central.start_scan(ScanFilter{ services: vec![BLE_MIDI_SERVICE_ID] }).await;
        // sleep(Duration::from_secs(3)).await;
        // let peripherals = central.peripherals().await.unwrap();



        /// CONNECTING TEST (checked only when already connected)
        // // println!("peripherals (midi_ble):");
        // for peripheral in peripherals {
        //     println!("attempting to connect to {:?}", peripheral.properties().await.unwrap().unwrap().local_name);
        //     peripheral.connect().await.unwrap();
        //     println!("connected to {:?}", peripheral.properties().await.unwrap().unwrap().local_name);
        //
        //     let nots = peripheral.notifications().await.unwrap();
        //     println!("notifications: {:?}", nots.unwrap());
        //     if let Ok(_) = peripheral.subscribe(&midi_characteristic).await {
        //         println!("subscribed to {:?} with MIDI MIDI MIDI MIDI MAN characteristic!", peripheral.properties().await);
        //     }
        //     // println!("  {:?}", peripheral.properties().await);
        // }


        // start scanning for devices
        // central.start_scan(ScanFilter::default()).await?;
        // // instead of waiting, you can use central.events() to get a stream which will
        // // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
        // sleep(Duration::from_secs(2)).await;
        //
        // // find the device we're interested in
        // let light = find_light(&central).await.unwrap();

        // connect to the device
        // light.connect().await?;




        println!("Press enter to quit");
        sleep(Duration::from_secs(20)).await;
        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();
        let _ = lines.next_line().await;

        println!("Removing advertisement");
        // drop(&self.advertisement_handle);
        sleep(Duration::from_secs(1)).await;
        // adapter.set_discoverable(false).await?;

        Ok(())
    }

    async fn setup_midi_gatt_service(&self) -> bluer::Result<()> {
        let adapter = self.midi_session.default_adapter().await?;
        let application = MidiBle::midi_application().await;

        let app_handle = adapter.serve_gatt_application(application).await?;

        // let status_byte = 0x90;  // Note on event on channel 1 (channels are 0-indexed)
        // let timestamp_byte = 0x80;  // A timestamp value. In practice, you should calculate this properly.
        // let note_number = 60;
        // let velocity = 127;
        //
        // let characteristic = MidiBler::midi_characteristic().await;
        //
        // let devices = adapter.discover_devices().await?;





        /// _-----------
        /// THIS SHOULD BE PUT BACK
        /// _-----------

        println!("Serving GATT echo service on Bluetooth adapter {}", adapter.name());
        let (char_control, char_handle) = characteristic_control();


        println!("Echo service ready. Press enter to quit.");
        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();

        let mut read_buf = Vec::new();
        let mut reader_opt: Option<CharacteristicReader> = None;
        let mut writer_opt: Option<CharacteristicWriter> = None;
        pin_mut!(char_control);

        loop {
            tokio::select! {
                _ = lines.next_line() => break,
                evt = char_control.next() => {
                    match evt {
                        Some(CharacteristicControlEvent::Write(req)) => {
                            println!("Accepting write request event with MTU {}", req.mtu());
                            read_buf = vec![0; req.mtu()];
                            reader_opt = Some(req.accept()?);
                        },
                        Some(CharacteristicControlEvent::Notify(notifier)) => {
                            println!("Accepting notify request event with MTU {}", notifier.mtu());
                            writer_opt = Some(notifier);
                        },
                        None => break,
                    }
                },
                read_res = async {
                    match &mut reader_opt {
                        Some(reader) if writer_opt.is_some() => reader.read(&mut read_buf).await,
                        _ => future::pending().await,
                    }
                } => {
                    match read_res {
                        Ok(0) => {
                            println!("Read stream ended");
                            reader_opt = None;
                        }
                        Ok(n) => {
                            let value = read_buf[..n].to_vec();
                            println!("Echoing {} bytes: {:x?} ... {:x?}", value.len(), &value[0..4.min(value.len())], &value[value.len().saturating_sub(4) ..]);
                            if value.len() < 512 {
                                println!();
                            }
                            if let Err(err) = writer_opt.as_mut().unwrap().write_all(&value).await {
                                println!("Write failed: {}", &err);
                                writer_opt = None;
                            }
                        }
                        Err(err) => {
                            println!("Read stream error: {}", &err);
                            reader_opt = None;
                        }
                    }
                }
            }
        }

        println!("Removing service and advertisement");
        drop(app_handle);
        sleep(Duration::from_secs(1)).await;


        /// _-----------
        /// ABOVE SHOULD BE PUT BACK
        /// _-----------

        Ok(())









        // for _i in 0..50 {
        //     // application.services[0].characteristics[0].write(&[timestamp_byte, status_byte, note_number, velocity], WriteType::WithoutResponse).await?;
        //     // application.services().await[0].characteristics().await[0].write(&[0x01, 0x02, 0x03], WriteType::WithoutResponse).await?;
        //     sleep(Duration::from_secs(2)).await;
        // }

        // println!("Service ready. Press enter to quit.");
        // let stdin = BufReader::new(tokio::io::stdin());
        // let mut lines = stdin.lines();
        // let _ = lines.next_line().await;
        //
        // println!("Removing service and advertisement");
        // drop(app_handle);
        // // drop(adv_handle);
        // sleep(Duration::from_secs(1)).await;
        //
        // Ok(())
    }

    async fn midi_application() -> Application {
        Application {
            services: vec![MidiBle::midi_service().await],
            ..Default::default()
        }
    }

    async fn midi_service() -> Service {
        Service {
            uuid: BLE_MIDI_SERVICE_ID,
            primary: true,
            characteristics: vec![MidiBle::midi_characteristic().await],
            ..Default::default()
        }
    }

    // #[tokio::main(flavor = "current_thread")]
    async fn midi_characteristic() -> Characteristic {
        let value = Arc::new(Mutex::new(vec![0x10, 0x01, 0x01, 0x10]));
        let value_read = value.clone();
        let value_write = value.clone();
        let value_notify = value.clone();
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
                    Box::pin(async move {
                        println!("Write request {:?} with value {:x?}", &req, &new_value);
                        let mut value = value.lock().await;
                        *value = new_value;
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
    }
}