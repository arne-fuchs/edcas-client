use std::option;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Clone)]
pub struct Icon {
    pub char: String,
    pub color: String,
    pub enabled: bool,
}

#[component]
pub fn settings_view_icons(settings: Signal<crate::desktop::settings::Settings>) -> Element {
    let icons = settings.cloned().icons;
    let mut icons: Vec<&String> = icons.iter().map(|f| f.0 ).collect();
    icons.sort();

    rsx! {
        div { class: "ed-value-box mt-10",
            div { class: "flex justify-center",
                p {
                    class: "text-base m-5 \
                    sm:text-xl  \
                    md:text-2xl \
                    lg:text-3xl \
                    xl:text-4xl",
                    "Icons"
                }
            }
            for key in icons {
                div { class: "flex  \
                     row-border \
                    ",
                    div{
                        class: "ml-10 w-[20px] text-center content-center h-12",
                        style:"color: {settings.read().icons.get(key.as_str()).unwrap().color}",
                        "{settings.read().icons.get(key).unwrap().char}"
                    }
                    div{
                        class:"text-center content-center h-12 w-2xs",
                        p {
                            class: "ml-10 mr-10 ",
                            {key.clone()}
                        }
                    }
                    div{
                        class:"text-center content-center h-12",
                        input {
                            class: "mr-10 w-[20px] \
                            ed-input-text
                            ",
                            type: "text",
                            oninput: {
                                let mut settings_cloned = settings.cloned();
                                let key_cloned = key.clone();
                                move |evt| {
                                    settings_cloned.icons.get_mut(key_cloned.as_str()).unwrap().char = evt.value().clone().trim().to_string();
                                    settings.set(settings_cloned.clone());
                                }
                            },
                            value: "{settings.read().icons.get(key).unwrap().char}"
                        }
                    }
                    div{
                        class:"text-center content-center h-12",
                        input {
                            class: "mr-10 w-[70px] \
                            ed-input-text
                            ",
                            type: "text",
                            oninput: {
                                let mut settings_cloned = settings.cloned();
                                let key_cloned = key.clone();
                                move |evt| {
                                    settings_cloned.icons.get_mut(key_cloned.as_str()).unwrap().color = evt.value().clone().trim().to_string();
                                    settings.set(settings_cloned.clone());
                                }
                            },
                            value: "{settings.read().icons.get(key).unwrap().color}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn settings_view_stars(settings: Signal<crate::desktop::settings::Settings>) -> Element {
    let icons = settings.cloned().stars;
    let mut icons: Vec<&String> = icons.iter().map(|f| f.0 ).collect();
    icons.sort();
    rsx! {
        div { class: "ed-value-box mt-10",
            div { class: "flex justify-center",
                p {
                    class: "text-base m-5 \
                    sm:text-xl  \
                    md:text-2xl \
                    lg:text-3xl \
                    xl:text-4xl",
                    "Stars"
                }
            }
            for key in icons {
                div { class: "flex  \
                     row-border \
                    ",
                    div{
                        class: "ml-10 w-[20px] text-center content-center h-12",
                        style:"color: {settings.read().stars.get(key.as_str()).unwrap().color}",
                        "{settings.read().stars.get(key.as_str()).unwrap().char}"
                    }
                    div{
                        class:"text-center content-center h-12 w-2xs",
                        p {
                            class: "ml-10 mr-10 ",
                            {key.clone()}
                        }
                    }
                    div{
                        class:"text-center content-center h-12",
                        input {
                            class: "mr-10 w-[20px] \
                            ed-input-text
                            ",
                            type: "text",
                            oninput: {
                                let mut settings_cloned = settings.cloned();
                                let key_cloned = key.clone();
                                move |evt| {
                                    settings_cloned.stars.get_mut(key_cloned.as_str()).unwrap().char = evt.value().clone().trim().to_string();
                                    settings.set(settings_cloned.clone());
                                }
                            },
                            value: "{settings.read().stars.get(key.as_str()).unwrap().char}"
                        }
                    }
                    div{
                        class:"text-center content-center h-12",
                        input {
                            class: "mr-10 w-[70px] \
                            ed-input-text
                            ",
                            type: "text",
                            oninput: {
                                let mut settings_cloned = settings.cloned();
                                let key_cloned = key.clone();
                                move |evt| {
                                    settings_cloned.stars.get_mut(key_cloned.as_str()).unwrap().color = evt.value().clone().trim().to_string();
                                    settings.set(settings_cloned.clone());
                                }
                            },
                            value: "{settings.read().stars.get(key.as_str()).unwrap().color}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn settings_view_planets(settings: Signal<crate::desktop::settings::Settings>) -> Element {
    let icons = settings.cloned().planets;
    let mut icons: Vec<&String> = icons.iter().map(|f| f.0 ).collect();
    icons.sort();
    rsx! {
        div { class: "ed-value-box mt-10",
            div { class: "flex justify-center",
                p {
                    class: "text-base m-5 \
                    sm:text-xl  \
                    md:text-2xl \
                    lg:text-3xl \
                    xl:text-4xl",
                    "Planets"
                }
            }
            for key in icons {
                div { class: "flex  \
                     row-border \
                    ",
                    div{
                        class: "ml-10 w-[20px] text-center content-center h-12",
                        style:"color: {settings.read().planets.get(key.as_str()).unwrap().color}",
                        "{settings.read().planets.get(key.as_str()).unwrap().char}"
                    }
                    div{
                        class:"text-center content-center h-12 w-2xs",
                        p {
                            class: "ml-10 mr-10 ",
                            {key.clone()}
                        }
                    }
                    div{
                        class:"text-center content-center h-12",
                        input {
                            class: "mr-10 w-[20px] \
                            ed-input-text
                            ",
                            type: "text",
                            oninput: {
                                let mut settings_cloned = settings.cloned();
                                let key_cloned = key.clone();
                                move |evt| {
                                    settings_cloned.planets.get_mut(key_cloned.as_str()).unwrap().char = evt.value().clone().trim().to_string();
                                    settings.set(settings_cloned.clone());
                                }
                            },
                            value: "{settings.read().planets.get(key.as_str()).unwrap().char}"
                        }
                    }
                    div{
                        class:"text-center content-center h-12",
                        input {
                            class: "mr-10 w-[70px] \
                            ed-input-text
                            ",
                            type: "text",
                            oninput: {
                                let mut settings_cloned = settings.cloned();
                                let key_cloned = key.clone();
                                move |evt| {
                                    settings_cloned.planets.get_mut(key_cloned.as_str()).unwrap().color = evt.value().clone().trim().to_string();
                                    settings.set(settings_cloned.clone());
                                }
                            },
                            value: "{settings.read().planets.get(key.as_str()).unwrap().color}"
                        }
                    }
                }
            }
        }
    }
}
