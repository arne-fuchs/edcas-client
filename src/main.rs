#![allow(unreachable_code)]
extern crate core;

use std::env;
use dioxus::desktop::tao::platform::unix::WindowBuilderExtUnix;
use dioxus::desktop::tao::window::{Icon, Theme};
use dioxus::desktop::WindowBuilder;
use dioxus::prelude::*;
use num_format::Locale::tr;
//use crate::edcas::EliteRustClient;

//mod cli;
//mod edcas;
use views::{Home};
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

    for i in 0..args.len() {
        match args[i].as_str() {
            _ => {}
        }
    }

    //let client = Box::<EliteRustClient>::default();

    // Set the url of the server where server functions are hosted.
    //#[cfg(not(feature = "server"))]
    //dioxus::fullstack::set_server_url("http://127.0.0.1:8080");

    #[cfg(feature = "desktop")]
    {
        let background = (0,0,0,255);
        let file = std::fs::read("./assets/graphics/logo/edcas_128_rgba.png").expect("Missing logo");
        let icon = Icon::from_rgba(file,32,32).expect("Couldn't load icon");
        let window = WindowBuilder::new()
            .with_decorations(true)
            .with_theme(Some(Theme::Dark))
            .with_background_color(background)
            .with_window_icon(Some(icon.clone()))
            .with_title("EDCAS")
            //.with_skip_taskbar(true)
            //.with_maximized(true)
            ;
        let config = dioxus::desktop::Config::new()
            .with_icon(icon)
            .with_background_color(background)
            .with_window(window)
            .with_resource_directory("dist/")
            //.with_menu()
            //.with_resource_directory()
            ;

        dioxus::LaunchBuilder::desktop().with_cfg(config).launch(App)
    }
    #[cfg(feature = "web")]
    dioxus::launch(App);
    #[cfg(feature = "server")]
    dioxus::launch(App);
}

const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

#[component]
fn App() -> Element {
    rsx!{
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        img { src: asset!("/assets/graphics/logo/edcas.png", AssetOptions::image().with_avif()), id: "logo-img", draggable: false }
    }
}