mod api;
mod application;
mod config;
mod error;
mod qrcode;
mod storage;
mod views;
mod widgets;

use application::SeeApplication;
use gtk::{gio, glib, prelude::*};

const APP_ID: &str = "ee.s.app";

fn main() -> glib::ExitCode {
    env_logger::init();

    // Initialize GTK first
    gtk::init().expect("Failed to initialize GTK");

    // Disable the deprecated gtk-application-prefer-dark-theme setting
    // to prevent libadwaita warning
    if let Some(settings) = gtk::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(false);
    }

    // Initialize libadwaita
    adw::init().expect("Failed to initialize libadwaita");

    gio::resources_register_include!("see.gresource").expect("Failed to register resources");

    let app = SeeApplication::new();
    app.run()
}
