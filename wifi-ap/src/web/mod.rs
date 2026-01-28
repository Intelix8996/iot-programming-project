mod app;

use embassy_net::Stack;
use embassy_time::Duration;
use esp_alloc as _;
use picoserve::{AppBuilder, AppRouter, Router};

pub use app::*;

pub const WEB_TASK_POOL_SIZE: usize = 2;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn web_task(
    id: usize,
    stack: Stack<'static>,
    router: &'static AppRouter<Application>,
    config: &'static picoserve::Config<Duration>,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 2048];
    let mut tcp_tx_buffer = [0; 2048];
    let mut http_buffer = [0; 4096];

    picoserve::Server::new(router, config, &mut http_buffer)
        .listen_and_serve(id, stack, port, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await
        .into_never()
}

pub struct WebApp {
    pub router: &'static Router<<Application as AppBuilder>::PathRouter>,
    pub config: &'static picoserve::Config<Duration>,
}

impl Default for WebApp {
    fn default() -> Self {
        let router = picoserve::make_static!(AppRouter<Application>, Application.build_app());

        let config = picoserve::make_static!(
            picoserve::Config<Duration>,
            picoserve::Config::new(picoserve::Timeouts {
                start_read_request: Some(Duration::from_secs(5)),
                read_request: Some(Duration::from_secs(1)),
                write: Some(Duration::from_secs(1)),
                persistent_start_read_request: Some(Duration::from_secs(1))
            })
            .keep_connection_alive()
        );

        Self { router, config }
    }
}
