#![allow(unreachable_code)]
extern crate core;

use dioxus::logger::tracing::debug;
use dioxus::prelude::*;
use std::env;
//use crate::edcas::EliteRustClient;

//mod cli;
//mod edcas;
use views::{Navbar, News, Settings};

mod desktop;
mod edcas;
#[cfg(feature = "eddn")]
mod eddn;
mod views;

fn main() {
    let args: Vec<String> = env::args().collect();
    let ascii_art = r#"
  ______    ____      ____       _       ______
 |  ____|  |  _ \   / ____|     / \     /  ____|
 | |__     | | | |  | |        / _ \    |  (___
 |  __|    | | | |  | |       / /_\ \   \___   \
 | |____   | |_| |  | |____  /  / \  \   ____) |
 |______|  |____/   \_____| /__/   \__\ |_____/

"#;
    println!("{}", ascii_art);
    if !args.is_empty() {
        debug!("Arguments: {:?}", args);
        println!("Arguments: {:?}", args);
    }

    for arg in args {
        match arg.as_str() {
            #[cfg(feature = "eddn")]
            "--eddn-listener" => {
                use std::process::exit;

                eddn::run_listener();
                exit(0);
            }
            #[cfg(feature = "eddn")]
            "--eddn-parser" => {
                use std::process::exit;

                eddn::run_parser();
                exit(0);
            }
            _ => {}
        }
    }

    //let client = Box::<EliteRustClient>::default();

    // Set the url of the server where server functions are hosted.
    //#[cfg(not(feature = "server"))]
    //dioxus::fullstack::set_server_url("http://127.0.0.1:8080");

    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::tao::platform::unix::WindowBuilderExtUnix;
        use dioxus::desktop::tao::window::{Icon, Theme};
        use dioxus::desktop::WindowBuilder;

        let icon_bytes = include_bytes!("../assets/graphics/logo/edcas_128_rgba.png");
        let icon = Icon::from_rgba(icon_bytes.to_vec(), 128, 128).expect("Couldn't load icon");
        let window = WindowBuilder::new()
            //.with_decorations(false)
            .with_theme(Some(Theme::Dark))
            .with_window_icon(Some(icon.clone()))
            .with_title("EDCAS")
            .with_skip_taskbar(true)
            .with_resizable(true)
            .with_maximized(true);
        let config = dioxus::desktop::Config::new()
            .with_icon(icon)
            .with_window(window)
            .with_resource_directory("dist/")
            .with_exits_when_last_window_closes(true)
            //.with_menu()
            ;

        dioxus::LaunchBuilder::desktop()
            .with_cfg(config)
            .launch(App)
    }
    #[cfg(feature = "web")]
    dioxus::launch(App);
    #[cfg(feature = "server")]
    dioxus::launch(App);
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
enum AppState {
    News = 0,
    Settings = 1,
}

impl AppState {
    fn next(&self) -> Self {
        match &self {
            AppState::News => AppState::Settings,
            AppState::Settings => AppState::News,
        }
    }

    fn prev(&self) -> Self {
        match &self {
            AppState::News => AppState::Settings,
            AppState::Settings => AppState::News,
        }
    }
}

#[component]
fn App() -> Element {
    let mut state = use_signal(|| AppState::Settings);
    let settings = use_signal(desktop::settings::Settings::default);

    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::tao::event::Event as WryEvent;
        use dioxus::desktop::tao::event::WindowEvent;
        use dioxus::desktop::use_wry_event_handler;

        use_wry_event_handler(move |event, _| {
            if let WryEvent::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        event,
                        is_synthetic: _,
                        ..
                    },
                ..
            } = event
            {
                if event.state == dioxus::desktop::tao::event::ElementState::Released {
                    match event.physical_key {
                        dioxus::desktop::tao::keyboard::KeyCode::KeyE => {
                            let current = state.read().clone();
                            state.set(current.next());
                        }
                        dioxus::desktop::tao::keyboard::KeyCode::KeyQ => {
                            let current = state.read().clone();
                            state.set(current.prev());
                        }
                        dioxus::desktop::tao::keyboard::KeyCode::ArrowUp => {}
                        dioxus::desktop::tao::keyboard::KeyCode::ArrowDown => {}
                        dioxus::desktop::tao::keyboard::KeyCode::ArrowLeft => {}
                        dioxus::desktop::tao::keyboard::KeyCode::ArrowRight => {}
                        _ => {}
                    }
                    debug!("key_state: {:?}", event);
                }
            }
        });
    }

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }
        Navbar { app_state: state }
        match state.try_read_unchecked() {
            Ok(state) => {
                match &*state {
                    AppState::News => rsx! {
                        News {}
                    },
                    AppState::Settings => rsx! {
                        Settings {settings: settings}
                    },
                }
            }
            Err(err) => rsx! {
                {err.to_string()}
            },
        }

    }
}
