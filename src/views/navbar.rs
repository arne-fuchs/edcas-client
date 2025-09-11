use crate::debug;
use crate::AppState;
use dioxus::prelude::*;

#[component]
pub fn Navbar(app_state: Signal<AppState>) -> Element {
    rsx! {
        div { class: "flex border-y-3 border-amber-600/60 bg-black fixed w-full h-10",
            div {
                class: if *app_state.read() == AppState::News {"ed-button-active w-25"} else {"ed-button w-25"},
                cursor: "pointer",
                onclick: move |_| {
                    app_state.set(AppState::News);
                },
                p { class: "text-glow",
                    "News"
                }
            }
            div {
                class: "ed-button w-25",
                cursor: "pointer",
                onclick: move |_| {
                    app_state.set(AppState::News);
                },
                p { class: "text-glow",
                    "Explorer"
                }
            }
            div {
                class: "ed-button w-25",
                cursor: "pointer",
                onclick: move |_| {
                    app_state.set(AppState::News);
                },
                p { class: "text-glow",
                    "Mining"
                }
            }
            div {
                class: "ed-button w-25",
                cursor: "pointer",
                onclick: move |_| {
                    app_state.set(AppState::News);
                },
                p { class: "text-glow",
                    "Materials"
                }
            }
            div {
                class: "ed-button w-25",
                cursor: "pointer",
                onclick: move |_| {
                    app_state.set(AppState::News);
                },
                p { class: "text-glow",
                    "About"
                }
            }
            div { class: "flex ml-auto",
                div {
                    class: "ed-button w-10",
                    onclick: move |_| {
                        #[cfg(feature = "desktop")]
                        {
                            let maximized = dioxus::desktop::window().is_maximized();
                            let mut pos = dioxus::desktop::tao::dpi::PhysicalPosition::default();
                            let current_monitor = dioxus::desktop::window().window.current_monitor();
                            debug!("current_monitor {:?}", current_monitor);
                            if let Some(current_monitor) = current_monitor {
                                let outer_position = dioxus::desktop::window().window.outer_position();
                                debug!("outer_position {:?}", outer_position);
                                if let Ok(outer_position) = outer_position {
                                    pos.y = current_monitor.size().height - outer_position.y as u32;
                                    pos.x = outer_position.x as u32;
                                    debug!("new_position {:?}", pos);

                                    dioxus::desktop::window().set_outer_position(pos);
                                    dioxus::desktop::window().set_maximized(maximized);
                                }
                            }
                        }
                    },
                    p { class: "text-glow", "⬆" }
                }
                div {
                    class: "ed-button w-10",
                    onclick: move |_| {
                        #[cfg(feature = "desktop")]
                        {
                            let maximized = dioxus::desktop::window().is_maximized();
                            let mut pos = dioxus::desktop::tao::dpi::PhysicalPosition::default();
                            let current_monitor = dioxus::desktop::window().window.current_monitor();
                            debug!("current_monitor {:?}", current_monitor);
                            if let Some(current_monitor) = current_monitor {
                                let outer_position = dioxus::desktop::window().window.outer_position();
                                debug!("outer_position {:?}", outer_position);
                                if let Ok(outer_position) = outer_position {
                                    pos.y = current_monitor.size().height + outer_position.y as u32;
                                    pos.x = outer_position.x as u32;
                                    debug!("new_position {:?}", pos);

                                    dioxus::desktop::window().set_outer_position(pos);
                                    dioxus::desktop::window().set_maximized(maximized);
                                }
                            }
                        }
                    },
                    p {class: "text-glow", "⬇" }
                }
                div {
                    class: "ed-button w-10",
                    onclick: move |_| {
                        #[cfg(feature = "desktop")]
                        {
                            let maximized = dioxus::desktop::window().is_maximized();
                            let mut pos = dioxus::desktop::tao::dpi::PhysicalPosition::default();
                            let current_monitor = dioxus::desktop::window().window.current_monitor();
                            debug!("current_monitor {:?}", current_monitor);
                            if let Some(current_monitor) = current_monitor {
                                let outer_position = dioxus::desktop::window().window.outer_position();
                                debug!("outer_position {:?}", outer_position);
                                if let Ok(outer_position) = outer_position {
                                    pos.x = current_monitor.size().width - outer_position.x as u32;
                                    pos.y = outer_position.y as u32;
                                    debug!("new_position {:?}", pos);

                                    dioxus::desktop::window().set_outer_position(pos);
                                    dioxus::desktop::window().set_maximized(maximized);
                                }
                            }
                        }
                    },
                    p {class: "text-glow", "⬅" }
                }
                div {
                    class: "ed-button w-10",
                    onclick: move |_| {
                        #[cfg(feature = "desktop")]
                        {
                            let maximized = dioxus::desktop::window().is_maximized();
                            let mut pos = dioxus::desktop::tao::dpi::PhysicalPosition::default();
                            let current_monitor = dioxus::desktop::window().window.current_monitor();
                            debug!("current_monitor {:?}", current_monitor);
                            if let Some(current_monitor) = current_monitor {
                                let outer_position = dioxus::desktop::window().window.outer_position();
                                debug!("outer_position {:?}", outer_position);
                                if let Ok(outer_position) = outer_position {
                                    pos.x = current_monitor.size().width + outer_position.x as u32;
                                    pos.y = outer_position.y as u32;
                                    debug!("new_position {:?}", pos);

                                    dioxus::desktop::window().set_outer_position(pos);
                                    dioxus::desktop::window().set_maximized(maximized);
                                }
                            }
                        }
                    },
                    p {class: "text-glow", "➡" }
                }
                div {
                    class: "ed-button w-10",
                    onclick: move |_| {
                        #[cfg(feature = "desktop")]
                        {
                            use dioxus::desktop::tao::platform::unix::WindowExtUnix;
                            dioxus::desktop::window()
                                .set_decorations(!dioxus::desktop::window().is_decorated());
                            let _ = dioxus::desktop::window().set_skip_taskbar(true);
                        }
                    },
                    p {class: "text-glow", "🗖" }
                }
                div {
                    class: if *app_state.read() == AppState::Settings {"ed-button-active w-10"} else {"ed-button w-10"},
                    onclick: move |_| {
                        app_state.set(AppState::Settings);
                    },
                    p {class: "text-glow", "⚙" }
                }
                div {
                    class: "ed-button w-10",
                    onclick: move |_| { #[cfg(feature = "desktop")] { dioxus::desktop::window().close() } },
                    p {class: "text-glow", "➜]" }
                }
            }
        }
    }
}
