#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use freya::prelude::*;

fn main() {
    launch(app); // Be aware that this will block the thread
}

fn app() -> Element {
    // RSX is a special syntax to define the UI of our components
    // Here we simply show a label element with some text
    rsx!(
        label { "Hello, World!" }
    )
}
