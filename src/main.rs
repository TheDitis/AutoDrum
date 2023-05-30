use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::gpio::Gpio;
use rppal::system::DeviceInfo;

use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use sysfs_gpio::Pin;
use tokio::time;
use uuid::{Uuid, uuid};
use AutoDrum::midi_ble::MidiBle;use tokio_gpiod::{Chip, Options};

const BASE_HIT_DURATION_SMALL: f32 = 0.0002;
const BASE_HIT_DURATION_BIG: f32 = 0.005;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_LED: u8 = 4;

const BLE_MIDI_SERVICE_ID: Uuid = uuid!("03B80E5A-EDE8-4B33-A751-6CE34EC4C700");
const BLE_MIDI_CHARACTERISTIC_ID: Uuid = uuid!("7772E5DB-3868-4112-A1A9-F2669D106BF3");

async fn setup_midi_bluetooth(hit: fn() -> ()) -> Result<(), Box<dyn Error>> {


    // let chip = Chip::new("gpiochip0").await?; // open chip
    //
    // let opts = Options::output([4]) // configure lines offsets
    //     .values([true]) // optionally set initial values
    //     .consumer("my-outputs"); // optionally set consumer string
    //
    // let outputs = chip.request_lines(opts).await?;
    //
    // outputs.set_values([true]).await?;



    let mut midi_ble_manager = MidiBle::new().await;
    tokio::task::spawn_blocking(move || {
        /// TOOO: HERE, seems like passing the function to turn on/off gpio works
        midi_ble_manager.init(hit);
    }).await.expect("Task panicked");


    // outputs.set_values([false]).await?;


    // let manager = Manager::new().await?;
    // let adapter_list = manager.adapters().await?;
    // if adapter_list.is_empty() {
    //     eprintln!("No Bluetooth adapters found");
    // }
    // for adapter in adapter_list.iter() {
    //     println!("Starting scan...");
    //     adapter
    //         .start_scan(ScanFilter::default())
    //         .await
    //         .expect("Can't scan BLE adapter for connected devices...");
    //     time::sleep(Duration::from_secs(2)).await;
    //     let peripherals = adapter.peripherals().await?;
    //     if peripherals.is_empty() {
    //         eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
    //     } else {
    //         // All peripheral devices in range.
    //         for peripheral in peripherals.iter() {
    //             let properties = peripheral.properties().await?.unwrap();
    //             // if properties.manufacturer_data.len() > 0 {
    //             println!("peripheral : {:?}", peripheral.properties().await?);
    //             peripheral.discover_services().await?;
    //             let characteristics = peripheral.characteristics();
    //             for characteristic in characteristics.iter() {
    //                 if characteristic.uuid == BLE_MIDI_CHARACTERISTIC_ID {
    //                     let data = peripheral.read(characteristic).await?;
    //                     println!("MIDI data: {:?}", data);
    //                 }
    //             }
    //             // }
    //         }
    //     }
    // }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let hit_func = || {
        let mut pin = Gpio::new().unwrap().get(GPIO_LED).unwrap().into_output();

        // Blink the LED by setting the pin's logic level high for 500 ms.
        pin.set_high();
        thread::sleep(Duration::from_millis(500));
        pin.set_low();
    };

    // hit_func();

    // let chip = Chip::new("gpiochip0").await?; // open chip
    //
    // let mut opts = Options::output([4]) // configure lines offsets
    //     .values([true]) // optionally set initial values
    //     .consumer("my-outputs"); // optionally set consumer string
    //
    // let mut outputs = chip.request_lines(opts).await?;
    // outputs.set_values([true]).await?;

    setup_midi_bluetooth(hit_func).await?;

    hit_func();

    // outputs.set_values([false]).await?;

    println!("Blinking an LED on a {}.", DeviceInfo::new()?.model());



    // let mut pin = Gpio::new()?.get(GPIO_LED)?.into_output();
    //
    // // Blink the LED by setting the pin's logic level high for 500 ms.
    // pin.set_high();
    // thread::sleep(Duration::from_millis(500));
    // pin.set_low();



    // let my_led = Pin::new(GPIO_LED as u64); // number depends on chip, etc.
    // let status = &u8::from(0x90);
    // if status == &u8::from(0x90) {
    //     my_led.set_value(u8::MAX);
    // } else {
    //     my_led.set_value(0);
    // }



    Ok(())
}
