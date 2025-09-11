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
    println!("{}",ascii_art);

    for arg in args {
        match arg {
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
            .with_maximized(true)
        ;
        let config = dioxus::desktop::Config::new()
            .with_icon(icon)
            .with_window(window)
            .with_resource_directory("dist/")
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

#[derive(PartialEq)]
enum AppState {
    News,
    Settings,
}

#[component]
fn App() -> Element {
    let state = use_signal(|| AppState::Settings);
    let settings = use_signal(desktop::settings::Settings::default);
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
