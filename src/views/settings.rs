use dioxus::prelude::*;

use crate::desktop;

#[component]
pub fn Settings(settings: Signal<desktop::settings::Settings>) -> Element {
    rsx! {
        div { class: "flex flex-col pt-20 pd-20",
            div{class: "flex justify-center",
                p {
                    class: "text-base \
                    sm:text-xl \
                    md:text-2xl \
                    lg:text-3xl \
                    xl:text-4xl",
                    "Settings"
                }
            }
            div { class: "flex justify-center",
                div { class: "ed-value-box mt-10",
                    div { class: "flex justify-center",
                        p {
                            class: "text-base m-5 \
                            sm:text-xl  \
                            md:text-2xl \
                            lg:text-3xl \
                            xl:text-4xl",
                            "Journal Reader"
                        }
                    },
                    div { class: "flex  \
                         border-y-4 border-black/80 \
                        ",
                        div{
                            class:"text-center content-center h-10",
                            p {
                                class: "ml-10 mr-10 ",
                                "Journal Directory"
                            }
                        }
                        div{
                            class:"text-center content-center h-10",
                            input {
                                class: "mr-10 w-2xs \
                                focus:outline-2 focus:outline-offset-4 focus:outline-amber-700/60 focus:border-none \
                                hover:bg-linear-to-r hover:from-amber-600/60 hover:via-amber-500/60 hover:to-orange-500/60 \
                                hover:bg-radial-[at_25%_25%] hover:to-75% hover:shadow-[0px_0px_10px_5px_rgba(249,_115,_22,_0.5)] \
                                ",
                                type: "text",
                                oninput: move |evt| {
                                    let mut copy = settings.cloned();
                                    copy.journal_reader.journal_directory = evt.value();
                                    settings.replace(copy);
                                },
                                value: "{settings.read().journal_reader.journal_directory}"
                            }
                        }
                    }
                }
            }
        }
    }
}
