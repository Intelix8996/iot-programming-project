use core::net::Ipv4Addr;

use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_time::{Duration, Timer};
use esp_hal::rng::Rng;
use esp_radio::wifi::{WifiApState, WifiController, WifiDevice, WifiEvent};
use log::info;

use crate::mk_static;

const STATIC_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);
const STATIC_IP_MASK: u8 = 24;
const GATEWAY_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);

const PASSWORD: &str = "esp32c3wifi";
const SSID: &str = "ESP32C3_WIFI_AP";

pub async fn start_wifi_ap(
    controller: WifiController<'static>,
    wifi_interface: WifiDevice<'static>,
    rng: Rng,
    spawner: &Spawner,
) -> Stack<'static> {
    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    // Create Network config with IP details
    let net_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(STATIC_IP, STATIC_IP_MASK),
        gateway: Some(GATEWAY_IP),
        dns_servers: Default::default(),
    });

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        net_seed,
    );

    spawner.spawn(connection_task(controller)).ok();
    spawner.spawn(net_task(runner)).ok();

    wait_for_connection(stack).await;

    stack
}

async fn wait_for_connection(stack: Stack<'_>) {
    info!("Waiting for link to be up");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("Connect to the AP `esp-wifi` and point your browser to http://{STATIC_IP}/");
    while !stack.is_config_up() {
        Timer::after(Duration::from_millis(100)).await
    }
    stack.config_v4().inspect(|c| info!("ipv4 config: {c:?}"));
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
async fn connection_task(mut controller: WifiController<'static>) {
    info!("Start connection task");
    info!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_radio::wifi::ap_state() {
            WifiApState::Started => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::ApStop).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let ap_config = esp_radio::wifi::AccessPointConfig::default()
                .with_ssid(SSID.try_into().unwrap())
                .with_password(PASSWORD.try_into().unwrap())
                .with_auth_method(esp_radio::wifi::AuthMethod::Wpa2Personal);
            let mode_config = esp_radio::wifi::ModeConfig::AccessPoint(ap_config);

            controller.set_config(&mode_config).unwrap();
            info!("Starting wifi");
            controller.start_async().await.unwrap();
            info!("Wifi started!");
        }
    }
}
