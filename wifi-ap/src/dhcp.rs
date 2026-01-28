use core::net::Ipv4Addr;

use embassy_net::Stack;
use embassy_time::Duration;
use esp_hal_dhcp_server::{simple_leaser::SimpleDhcpLeaser, structs::DhcpServerConfig};

#[embassy_executor::task]
pub async fn dhcp_server(stack: Stack<'static>) {
    let config = DhcpServerConfig {
        ip: Ipv4Addr::new(10, 0, 0, 1),
        lease_time: Duration::from_secs(3600),
        gateways: &[],
        subnet: None,
        dns: &[],
        use_captive_portal: false,
    };

    let mut leaser = SimpleDhcpLeaser {
        start: Ipv4Addr::new(10, 0, 0, 50),
        end: Ipv4Addr::new(10, 0, 0, 200),
        leases: Default::default(),
    };
    let _ = esp_hal_dhcp_server::run_dhcp_server(stack, config, &mut leaser).await;
}
