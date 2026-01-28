use core::cell::OnceCell;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use esp_alloc as _;
use esp_hal::gpio::{Flex, InputConfig, Level, OutputConfig};
use log::info;
use picoserve::{AppBuilder, response::{File, IntoResponse, StatusCode}, routing::{self, get_service, parse_path_segment}};

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use esp_metadata_generated::for_each_gpio;
use serde::Serialize;

fn generate_gpios() -> Vec<String> {
    let mut gpios: Vec<String> = Vec::new();
    for_each_gpio! {
        (all $( ($n:literal, $gpio:ident ($($digital_input_function:ident => $digital_input_signal:ident)*) ($($digital_output_function:ident => $digital_output_signal:ident)*) ($([$pin_attribute:ident])*)) ),*) => {
            $(
                gpios.push(String::from(concat!("gpio", $n)));
            )*
        };
    }

    gpios
}

#[derive(Debug)]
pub enum GpioDriverMode {
    Input,
    Output
}

#[derive(Debug)]
pub struct GpioDriver {
    pub driver: Flex<'static>,
    pub mode: GpioDriverMode,
}

for_each_gpio! {
    (all $( ($n:literal, $gpio:ident ($($digital_input_function:ident => $digital_input_signal:ident)*) ($($digital_output_function:ident => $digital_output_signal:ident)*) ($([$pin_attribute:ident])*)) ),*) => {
        #[derive(Debug)]
        pub struct GpioDrivers {
            $(
                pub $gpio: GpioDriver,
            )*
        }
    };
}

// type GpioPins = Vec<Flex<'static>>; 

pub static PIN_DRIVERS: Mutex<CriticalSectionRawMutex, OnceCell<GpioDrivers>> = Mutex::new(OnceCell::new());

pub struct Application;

#[derive(Serialize)]
struct LevelResponse {
    pub level: bool
}

#[derive(Serialize)]
struct ModeResponse {
    pub mode: String
}

#[derive(Serialize)]
struct GpiosResponse {
    pub gpios: Vec<String>
}

impl AppBuilder for Application {
    type PathRouter = impl routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        // picoserve::Router::new()
        //     .route(
        //         "/level",
        //         routing::get(|| async {
        //             let mut pins_guard = PIN_DRIVERS.lock().await;
        //             let pins = pins_guard.get_mut().unwrap();

        //             let level = match pins.GPIO8.level() {
        //                 Level::High => true,
        //                 Level::Low => false,
        //             };
        //             picoserve::response::Json(LevelResponse { level })
        //         })
        //     )
        //     .route(
        //         "/toggle",
        //         routing::get(async || {
        //             let mut pins_guard = PIN_DRIVERS.lock().await;
        //             let pins = pins_guard.get_mut().unwrap();

        //             pins.GPIO8.toggle();
        //         })
        //     )
        //     .route(
        //         "/set/high",
        //         routing::get(async || {
        //             let mut pins_guard = PIN_DRIVERS.lock().await;
        //             let pins = pins_guard.get_mut().unwrap();

        //             pins.GPIO8.set_high();
        //         })
        //     )
        //     .route(
        //         "/set/low",
        //         routing::get(async || {
        //             let mut pins_guard = PIN_DRIVERS.lock().await;
        //             let pins = pins_guard.get_mut().unwrap();

        //             pins.GPIO8.set_low();
        //         })
        //     )

        let router = picoserve::Router::new()
            .route(
                "/",
                get_service(File::html(include_str!("../frontend/index.html")))
            )
            .route(
                "/assets/index-C1ON0lHl.js",
                get_service(File::javascript(include_str!("../frontend/assets/index-C1ON0lHl.js")))
            )
            .route(
                "/assets/index-BHnF5g-J.css",
                get_service(File::css(include_str!("../frontend/assets/index-BHnF5g-J.css")))
            )
            .route(
                "/gpios",
                routing::get(|| async {
                    let gpios = generate_gpios();
                    picoserve::response::Json(GpiosResponse { gpios })
                })
            );

        for_each_gpio! {
            (all $( ($n:literal, $gpio:ident ($($digital_input_function:ident => $digital_input_signal:ident)*) ($($digital_output_function:ident => $digital_output_signal:ident)*) ($([$pin_attribute:ident])*)) ),*) => {
                return router
                $(
                    .route(
                        concat!("/gpio", $n, "/level"),
                        routing::get(|| async {
                            let mut pins_guard = PIN_DRIVERS.lock().await;
                            let pins = pins_guard.get_mut().unwrap();

                            let level = match pins.$gpio.driver.level() {
                                Level::High => true,
                                Level::Low => false,
                            };
                            picoserve::response::Json(LevelResponse { level })
                        })
                    )
                    .route(
                        concat!("/gpio", $n, "/toggle"),
                        routing::get(async || {
                            let mut pins_guard = PIN_DRIVERS.lock().await;
                            let pins = pins_guard.get_mut().unwrap();

                            pins.$gpio.driver.toggle();
                        })
                    )
                    .route(
                        concat!("/gpio", $n, "/set/high"),
                        routing::get(async || {
                            let mut pins_guard = PIN_DRIVERS.lock().await;
                            let pins = pins_guard.get_mut().unwrap();

                            pins.$gpio.driver.set_high();
                        })
                    )
                    .route(
                        concat!("/gpio", $n, "/set/low"),
                        routing::get(async || {
                            let mut pins_guard = PIN_DRIVERS.lock().await;
                            let pins = pins_guard.get_mut().unwrap();

                            pins.$gpio.driver.set_low();
                        })
                    )
                    .route(
                        concat!("/gpio", $n, "/mode"),
                        routing::get(async || {
                            let mut pins_guard = PIN_DRIVERS.lock().await;
                            let pins = pins_guard.get_mut().unwrap();

                            let mode = match pins.$gpio.mode {
                                GpioDriverMode::Input => "input",
                                GpioDriverMode::Output => "output"
                            };
                            picoserve::response::Json(ModeResponse { mode: String::from(mode) })
                        })
                    )
                    .route(
                        concat!("/gpio", $n, "/mode/output"),
                        routing::get(async || {
                            let mut pins_guard = PIN_DRIVERS.lock().await;
                            let pins = pins_guard.get_mut().unwrap();

                            pins.$gpio.driver.set_output_enable(true);
                            pins.$gpio.mode = GpioDriverMode::Output;
                        })
                    )
                    .route(
                        concat!("/gpio", $n, "/mode/input"),
                        routing::get(async || {
                            let mut pins_guard = PIN_DRIVERS.lock().await;
                            let pins = pins_guard.get_mut().unwrap();

                            pins.$gpio.driver.set_output_enable(false);
                            pins.$gpio.mode = GpioDriverMode::Input;
                        })
                    )
                )*
            };
        }

        // for_each_gpio! {
        //     (all $( ($n:literal, $gpio:ident ($($digital_input_function:ident => $digital_input_signal:ident)*) ($($digital_output_function:ident => $digital_output_signal:ident)*) ($([$pin_attribute:ident])*)) ),*) => {
        //         return picoserve::Router::new()
        //         $(
        //             .route(
        //                 concat!("/gpio", $n),
        //                 routing::get(|| async move { stringify!($gpio) }),
        //             )
        //             .route(
        //                 concat!("/gpio", $n, "/level"),
        //                 routing::get(|| async move {
        //                     let mut pin_guard = PIN.lock().await;
        //                     let pin = pin_guard.get_mut().unwrap();

        //                     pin.level().to_string()
        //                 }),
        //             )
        //             // .route(
        //             //     concat!("/", stringify!($gpio), "/get"),
        //             //     get($gpio.read()),
        //             // )
        //             // .route(
        //             //     (concat!("/", stringify!($gpio), "/set"), parse_path_segment::<bool>()),
        //             //     post(|input| async move {
        //             //         $gpio.set(input)
        //             //     }),
        //             // )
        //         )*
        //     };
        // }

        // picoserve::Router::new()
        //     .route(
        //         "/",
        //         routing::get_service(File::html(include_str!("../html/index.html"))),
        //     )
        //     .route(
        //         "/hello",
        //         routing::get(|| async move { "Hello World" }),
        //     )
        //     // .route(
        //     //     "/update",
        //     //     routing::post(async |picoserve::extract::Form(UpdateForm { value })| {
        //     //         info!("[HTTP] Got value {value}, putting into queue");
        //     //         crate::uart::get_channel().send(value).await;
        //     //         (picoserve::response::StatusCode::OK, picoserve::response::NoContent)
        //     //     }),
        //     // )
        //     .route(
        //         "/toggle",
        //         routing::get(async || {
        //             info!("[HTTP] TOGGGLE T HEE LEEED");
                    
        //             let mut pin = PIN.lock().await;

        //             let p = pin.get_mut().unwrap();

        //             p.level();

        //             p.toggle();
        //         })
        //     )
        //     // .route(
        //     //     ("/set", parse_path_segment::<bool>()),
        //     //     routing::get(async |val: bool| {
        //     //         info!("[HTTP] TOGGGLE T HEE LEEED");
                    
        //     //         let mut pin = PIN.lock().await;

        //     //         let a = pin.as_mut().unwrap();
        //     //         a.set_level(val.into());
        //     //     })
        //     // )
    }
}
