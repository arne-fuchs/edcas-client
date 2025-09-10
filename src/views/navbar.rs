use crate::debug;
use dioxus::prelude::*;
use crate::AppState;

#[component]
pub fn Navbar(app_state: Signal<AppState>) -> Element {
    rsx!{
        div{
            class:"flex border-y-4 border-purple-500/50 bg-purple-800/15 fixed w-full",
            div{
                 class: "cursor-pointer bg-clip-border w-24 flex-initial ml-1 mr-1 \
                         bg-linear-to-r from-purple-800/25 via-purple-500/25 to-orange-500/10 \
                         text-center \
                         ",
                cursor: "pointer",
                onclick: move |_| {
                    app_state.set(AppState::News);
                },
                a{
                    "News"
                }
            },
            div{
                class: "cursor-pointer bg-clip-border w-24 flex-initial ml-1 mr-1 \
                         bg-linear-to-r from-purple-800/25 via-purple-500/25 to-orange-500/10 \
                         text-center \
                         ",
                onclick: move |_| {
                    app_state.set(AppState::Settings);
                },
                a{
                    "Settings"
                }
            }
            div{
                class: "flex ml-auto",
                div{
                    class: "cursor-pointer bg-clip-border w-10 flex-initial ml-1 mr-1 \
                             bg-linear-to-r from-purple-800/25 via-purple-500/25 to-orange-500/10 \
                             text-center \
                             ",
                    onclick: move |_| {
                        #[cfg(feature = "desktop")]
                        {
                            let maximized = dioxus::desktop::window().is_maximized();
                            let mut pos = dioxus::desktop::tao::dpi::PhysicalPosition::default();
                            let current_monitor = dioxus::desktop::window().window.current_monitor();
                            debug!("current_monitor {:?}",current_monitor);
                            if let Some(current_monitor) = current_monitor{
                                let outer_position = dioxus::desktop::window().window.outer_position();
                                debug!("outer_position {:?}",outer_position);
                                if let Ok(outer_position) = outer_position{
                                    pos.y = current_monitor.size().height - outer_position.y as u32;
                                    pos.x = outer_position.x as u32;
                                    debug!("new_position {:?}",pos);

                                    dioxus::desktop::window().set_outer_position(pos);
                                    dioxus::desktop::window().set_maximized(maximized);
                                }
                            }
                        }
                    },
                    a{
                        "⬆"
                    }
                }
                div{
                    class: "cursor-pointer bg-clip-border w-10 flex-initial ml-1 mr-1 \
                             bg-linear-to-r from-purple-800/25 via-purple-500/25 to-orange-500/10 \
                             text-center \
                             ",
                    onclick: move |_| {
                        #[cfg(feature = "desktop")]
                        {
                            let maximized = dioxus::desktop::window().is_maximized();
                            let mut pos = dioxus::desktop::tao::dpi::PhysicalPosition::default();
                            let current_monitor = dioxus::desktop::window().window.current_monitor();
                            debug!("current_monitor {:?}",current_monitor);
                            if let Some(current_monitor) = current_monitor{
                                let outer_position = dioxus::desktop::window().window.outer_position();
                                debug!("outer_position {:?}",outer_position);
                                if let Ok(outer_position) = outer_position{
                                    pos.y = current_monitor.size().height + outer_position.y as u32;
                                    pos.x = outer_position.x as u32;
                                    debug!("new_position {:?}",pos);

                                    dioxus::desktop::window().set_outer_position(pos);
                                    dioxus::desktop::window().set_maximized(maximized);
                                }
                            }
                        }
                    },
                    a{
                        "⬇"
                    }
                }
                div{
                    class: "cursor-pointer bg-clip-border w-10 flex-initial ml-1 mr-1 \
                             bg-linear-to-r from-purple-800/25 via-purple-500/25 to-orange-500/10 \
                             text-center \
                             ",
                    onclick: move |_| {
                        #[cfg(feature = "desktop")]
                        {
                            let maximized = dioxus::desktop::window().is_maximized();
                            let mut pos = dioxus::desktop::tao::dpi::PhysicalPosition::default();
                            let current_monitor = dioxus::desktop::window().window.current_monitor();
                            debug!("current_monitor {:?}",current_monitor);
                            if let Some(current_monitor) = current_monitor{
                                let outer_position = dioxus::desktop::window().window.outer_position();
                                debug!("outer_position {:?}",outer_position);
                                if let Ok(outer_position) = outer_position{
                                    pos.x = current_monitor.size().width - outer_position.x as u32;
                                    pos.y = outer_position.y as u32;
                                    debug!("new_position {:?}",pos);

                                    dioxus::desktop::window().set_outer_position(pos);
                                    dioxus::desktop::window().set_maximized(maximized);
                                }
                            }
                        }
                    },
                    a{
                        "⬅"
                    }
                }
                div{
                    class: "cursor-pointer bg-clip-border w-10 flex-initial ml-1 mr-1 \
                             bg-linear-to-r from-purple-800/25 via-purple-500/25 to-orange-500/10 \
                             text-center \
                             ",
                    onclick: move |_| {
                        #[cfg(feature = "desktop")]
                        {
                            let maximized = dioxus::desktop::window().is_maximized();
                            let mut pos = dioxus::desktop::tao::dpi::PhysicalPosition::default();
                            let current_monitor = dioxus::desktop::window().window.current_monitor();
                            debug!("current_monitor {:?}",current_monitor);
                            if let Some(current_monitor) = current_monitor{
                                let outer_position = dioxus::desktop::window().window.outer_position();
                                debug!("outer_position {:?}",outer_position);
                                if let Ok(outer_position) = outer_position{
                                    pos.x = current_monitor.size().width + outer_position.x as u32;
                                    pos.y = outer_position.y as u32;
                                    debug!("new_position {:?}",pos);

                                    dioxus::desktop::window().set_outer_position(pos);
                                    dioxus::desktop::window().set_maximized(maximized);
                                }
                            }
                        }
                    },
                    a{
                        "➡"
                    }
                }
                div{
                    class: "cursor-pointer bg-clip-border w-10 flex-initial ml-1 mr-1 \
                             bg-linear-to-r from-purple-800/25 via-purple-500/25 to-orange-500/10 \
                             text-center \
                             ",
                    onclick: move |_| {
                        #[cfg(feature = "desktop")]
                        {
                            use dioxus::desktop::tao::platform::unix::WindowExtUnix;
                            dioxus::desktop::window().set_decorations(!dioxus::desktop::window().is_decorated());
                            dioxus::desktop::window().set_skip_taskbar(true);
                        }
                    },
                    a{
                        "🗖"
                    }
                }
                div{
                    class: "cursor-pointer bg-clip-border w-24 flex-initial ml-1 mr-1 \
                             bg-linear-to-r from-purple-800/25 via-purple-500/25 to-orange-500/10 \
                             text-center \
                             ",
                    onclick: move |_| {
                        #[cfg(feature = "desktop")]
                        {
                            dioxus::desktop::window().close()
                        }
                    },
                    a{
                        "➜] Exit"
                    }
                }
            }
        }
    }
}