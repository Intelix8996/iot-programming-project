use embassy_sync::{blocking_mutex::raw::{CriticalSectionRawMutex}, channel::Channel};
use esp_hal::{Blocking, uart::Uart};
use log::info;

static CHANNEL: Channel<CriticalSectionRawMutex, u8, 16> = Channel::new();

pub fn get_channel() -> &'static Channel<CriticalSectionRawMutex, u8, 16> {
    &CHANNEL
}

#[embassy_executor::task]
pub async fn uart_task(mut uart: Uart<'static, Blocking>) {
    loop {
        let value = get_channel().receive().await;
        info!("[UART] Received {value}, writing to UART");

        uart.write(&[value]).unwrap();
    }
}
