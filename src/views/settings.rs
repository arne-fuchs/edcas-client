use dioxus::prelude::*;

#[component]
pub fn Settings(settings: Signal<crate::desktop::settings::Settings>) -> Element {
    rsx! {
        div { class: "flex flex-col pt-20 pd-20",
            div{class: "flex justify-center",
                p {
                    class: "text-base \
                    sm:text-xl \
                    md:text-2xl \
                    lg:text-3xl \
                    xl:text-4xl blur-none",
                    "Settings"
                }
            }
            div { class: "flex justify-center",
                crate::desktop::settings::journal_reader::settings_view {settings: settings}
            }

            div { class: "flex flex-col sm:flex-row md:flex-row lg:flex-row xl:flex-row 2xl:flex-row justify-center",
                div{crate::desktop::settings::icons::settings_view_icons {settings: settings}}
                div{crate::desktop::settings::icons::settings_view_stars {settings: settings}}
                div{crate::desktop::settings::icons::settings_view_planets {settings: settings}}
            }
        }
    }
}
