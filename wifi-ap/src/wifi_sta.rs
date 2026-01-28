use embassy_executor::Spawner;
use embassy_net::{DhcpConfig, Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::rng::Rng;
use esp_radio::wifi::{WifiController, WifiDevice, WifiEvent, WifiStaState};
use log::{error, info};

use crate::mk_static;

const SSID: &str = "ssid";
const PASSWORD: &str = "password";

pub async fn start_wifi_sta(
    controller: WifiController<'static>,
    wifi_interface: WifiDevice<'static>,
    rng: Rng,
    spawner: &Spawner,
) -> Stack<'static> {   
    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    let net_config = embassy_net::Config::dhcpv4(DhcpConfig::default());

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

    info!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            info!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
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
        match esp_radio::wifi::sta_state() {
            WifiStaState::Connected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = esp_radio::wifi::ModeConfig::Client(
                esp_radio::wifi::ClientConfig::default()
                    .with_ssid(SSID.into())
                    .with_password(PASSWORD.into()),
            );
            controller.set_config(&client_config).unwrap();
            info!("Starting wifi");
            controller.start_async().await.unwrap();
            info!("Wifi started!");

            // info!("Scan");
            // let scan_config = esp_radio::wifi::ScanConfig::default().with_max(10);
            // let result = controller
            //     .scan_with_config_async(scan_config)
            //     .await
            //     .unwrap();
            // for ap in result {
            //     info!("{:?}", ap);
            // }

            info!("About to connect...");

            match controller.connect_async().await {
                Ok(_) => info!("Wifi connected!"),
                Err(e) => {
                    error!("Failed to connect to wifi: {:?}", e);
                    Timer::after(Duration::from_millis(5000)).await
                }
            }
        }
    }
}
