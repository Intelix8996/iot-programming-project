#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::gpio::{Flex, InputConfig, OutputConfig};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, rng::Rng};
use esp_radio::Controller;
use log::info;
use wifi_ap::mk_static;

extern crate alloc;

use wifi_ap::web::{GpioDrivers, GpioDriver, GpioDriverMode};
use esp_metadata_generated::for_each_gpio;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // generator version: 1.0.1

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let radio_init = mk_static!(
        Controller,
        esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller")
    );
    let (wifi_controller, interfaces) =
        esp_radio::wifi::new(radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let rng = Rng::new();

    // Configure and Start Wi-Fi tasks
    let stack = wifi_ap::wifi_ap::start_wifi_ap(wifi_controller, interfaces.ap, rng, &spawner).await;
    // let stack = wifi_ap::wifi_sta::start_wifi_sta(wifi_controller, interfaces.sta, rng, &spawner).await;

    // let uart_cfg = esp_hal::uart::Config::default()
    //     .with_baudrate(9600);

    // let uart = esp_hal::uart::Uart::new(peripherals.UART0, uart_cfg)
    //     .unwrap()
    //     .with_rx(peripherals.GPIO1)
    //     .with_tx(peripherals.GPIO2);

    // spawner.spawn(wifi_ap::uart::uart_task(uart)).unwrap();

    let mut gpio_drivers: GpioDrivers;

    for_each_gpio! {
        (20, $gpio:ident ($($digital_input_function:ident => $digital_input_signal:ident)*) ($($digital_output_function:ident => $digital_output_signal:ident)*) ($([$pin_attribute:ident])*)) => {};
        (21, $gpio:ident ($($digital_input_function:ident => $digital_input_signal:ident)*) ($($digital_output_function:ident => $digital_output_signal:ident)*) ($([$pin_attribute:ident])*)) => {};
        
        (all $( ($n:literal, $gpio:ident ($($digital_input_function:ident => $digital_input_signal:ident)*) ($($digital_output_function:ident => $digital_output_signal:ident)*) ($([$pin_attribute:ident])*)) ),*) => {
            gpio_drivers = GpioDrivers {
                $(
                    $gpio: GpioDriver {
                        driver: Flex::new(peripherals.$gpio),
                        mode: GpioDriverMode::Input,
                    },
                )*
            };
        };
    }

    for_each_gpio! {
        (all $( ($n:literal, $gpio:ident ($($digital_input_function:ident => $digital_input_signal:ident)*) ($($digital_output_function:ident => $digital_output_signal:ident)*) ($([$pin_attribute:ident])*)) ),*) => {
            $(
                gpio_drivers.$gpio.driver.apply_output_config(&OutputConfig::default());
                gpio_drivers.$gpio.driver.set_output_enable(false);
                gpio_drivers.$gpio.driver.apply_input_config(&InputConfig::default());
                gpio_drivers.$gpio.driver.set_input_enable(true);
            )*
        };
    }

    {
        let drivers_guard = wifi_ap::web::PIN_DRIVERS.lock().await;
        (*drivers_guard).set(gpio_drivers).unwrap();
    }

    // {
    //     let mut flex = Flex::new(peripherals.GPIO8);
    //     flex.apply_output_config(&OutputConfig::default());
    //     flex.set_output_enable(true);
    
    //     let p = wifi_ap::web::PIN.lock().await;
    //     (*p).set(flex).unwrap();
    // }

    // Web Tasks
    let web_app = wifi_ap::web::WebApp::default();
    for id in 0..wifi_ap::web::WEB_TASK_POOL_SIZE {
        spawner.must_spawn(wifi_ap::web::web_task(
            id,
            stack,
            web_app.router,
            web_app.config,
        ));
    }
    info!("Web server started...");

    spawner.spawn(wifi_ap::dhcp::dhcp_server(stack)).ok();
    info!("DHCP server started...");
}
