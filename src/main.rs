#![allow(unreachable_code)]
extern crate core;

use std::env;
use dioxus::logger::tracing::{debug, info, error};
use dioxus::prelude::*;
//use crate::edcas::EliteRustClient;

//mod cli;
//mod edcas;
use views::News;
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
        use dioxus::desktop::tao::window::RGBA;
        use dioxus::desktop::tao::window::{Icon, Theme};
        use dioxus::desktop::WindowBuilder;
        let background = (0,0,0,255);
        let icon_bytes = include_bytes!("../assets/graphics/logo/edcas_128_rgba.png");
        //let icon_asset:Asset = asset!("/assets/graphics/logo/edcas.png", AssetOptions::image().with_png().with_size(ImageSize::Manual { width: 64, height: 64 }));
        //let rgba = RGBA::from(icon_bytes.as_slice());
        let icon = Icon::from_rgba(icon_bytes.to_vec(),128,128).expect("Couldn't load icon");
        let window = WindowBuilder::new()
            .with_decorations(true)
            .with_theme(Some(Theme::Dark))
            //.with_background_color(background)
            .with_window_icon(Some(icon.clone()))
            .with_title("EDCAS")
            //.with_skip_taskbar(true)
            //.with_maximized(true)
            ;
        let config = dioxus::desktop::Config::new()
            .with_icon(icon)
            //.with_background_color(background)
            .with_window(window)
            .with_resource_directory("dist/")
            //.with_menu()
            ;

        dioxus::LaunchBuilder::desktop().with_cfg(config).launch(App)
    }
    #[cfg(feature = "web")]
    dioxus::launch(App);
    #[cfg(feature = "server")]
    dioxus::launch(App);
}


pub struct Article {
    pub title: String,
    pub date: String,
    pub text: String,
}

#[component]
fn App() -> Element {
    rsx!{
        News {}
    }
}