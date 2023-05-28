use bluer::adv::Advertisement;
use bluer::gatt::local::Service;
use std::time::Duration;
use btleplug::api::Characteristic;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    time::sleep,
};
use uuid::{Uuid, uuid};

const BLE_MIDI_SERVICE_ID: Uuid = uuid!("03B80E5A-EDE8-4B33-A751-6CE34EC4C700");
const BLE_MIDI_CHARACTERISTIC_ID: Uuid = uuid!("7772E5DB-3868-4112-A1A9-F2669D106BF3");

pub struct MidiBle {
    midi_session: bluer::Session,
}

impl MidiBle {
    pub async fn new() -> MidiBle {
        MidiBle {
            midi_session: bluer::Session::new().await.unwrap(),
        }
    }

    #[tokio::main]
    pub async fn init(&self) -> bluer::Result<()> {
        self.setup_advertisement().await?;
        Ok(())
    }

    async fn setup_advertisement(&self) -> bluer::Result<()> {
        let adapter = self.midi_session.default_adapter().await?;
        adapter.set_powered(true).await?;
        adapter.set_discoverable(true).await?;

        println!("Advertising on Bluetooth adapter {} with address {}", adapter.name(), adapter.address().await?);
        let le_advertisement = Advertisement {
            advertisement_type: bluer::adv::Type::Peripheral,
            service_uuids: vec!["03B80E5A-EDE8-4B33-A751-6CE34EC4C700".parse().unwrap()].into_iter().collect(),
            discoverable: Some(true),
            local_name: Some("AutoDrum".to_string()),
            ..Default::default()
        };
        println!("{:?}", &le_advertisement);
        let handle = adapter.advertise(le_advertisement).await?;

        println!("Press enter to quit");
        sleep(Duration::from_secs(20)).await;
        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();
        let _ = lines.next_line().await;

        println!("Removing advertisement");
        drop(handle);
        sleep(Duration::from_secs(1)).await;
        adapter.set_discoverable(false).await?;

        Ok(())
    }

    // async fn setup_midi_gatt_service(&self) -> bluer::Result<()> {
    //     let adapter = self.midi_session.default_adapter().await?;
    //     let application = bluer::gatt::local::Application::new();
    //
    //     let service = MidiBle::midi_service();
    //     let app = application.register(service).await?;
    //     adapter.serve_gatt_application(app).await?;
    //     Ok(())
    // }
    //
    // fn midi_service() -> Service {
    //     Service {
    //         uuid: "03B80E5A-EDE8-4B33-A751-6CE34EC4C700".parse().unwrap(),
    //         handle: None,
    //         primary: true,
    //         characteristics: vec![],
    //     }
    // }
    //
    // fn midi_characteristic() -> Characteristic {
    //     Characteristic {
    //         uuid: BLE_MIDI_CHARACTERISTIC_ID,
    //     }
    // }
}