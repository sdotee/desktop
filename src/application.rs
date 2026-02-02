use crate::config::Config;
use crate::views::preferences::SeePreferencesWindow;
use crate::views::window::SeeWindow;
use crate::APP_ID;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use std::cell::OnceCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SeeApplication {
        pub config: OnceCell<Config>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SeeApplication {
        const NAME: &'static str = "SeeApplication";
        type Type = super::SeeApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for SeeApplication {}

    impl ApplicationImpl for SeeApplication {
        fn activate(&self) {
            let app = self.obj();
            let window = if let Some(window) = app.active_window() {
                window
            } else {
                let window = SeeWindow::new(&app);
                window.upcast()
            };
            window.present();
        }

        fn startup(&self) {
            self.parent_startup();

            let app = self.obj();

            // Load custom CSS
            app.load_css();

            app.setup_gactions();
            app.setup_accels();

            // Load configuration
            match Config::load() {
                Ok(config) => {
                    let _ = self.config.set(config);
                }
                Err(e) => {
                    log::error!("Failed to load config: {}", e);
                    let _ = self.config.set(Config::default());
                }
            }
        }
    }

    impl GtkApplicationImpl for SeeApplication {}

    impl AdwApplicationImpl for SeeApplication {}
}

glib::wrapper! {
    pub struct SeeApplication(ObjectSubclass<imp::SeeApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl SeeApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("flags", gio::ApplicationFlags::FLAGS_NONE)
            .build()
    }

    pub fn config(&self) -> &Config {
        self.imp()
            .config
            .get()
            .expect("Config should be initialized")
    }

    pub fn update_config<F>(&self, f: F)
    where
        F: FnOnce(&mut Config),
    {
        let imp = self.imp();
        if let Some(config) = imp.config.get() {
            let mut new_config = config.clone();
            f(&mut new_config);
            if let Err(e) = new_config.save() {
                log::error!("Failed to save config: {}", e);
            }
            // Note: OnceCell doesn't allow mutation, so we reload on next access
        }
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(|app: &Self, _, _| app.quit())
            .build();

        let about_action = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| app.show_about())
            .build();

        let preferences_action = gio::ActionEntry::builder("preferences")
            .activate(|app: &Self, _, _| app.show_preferences())
            .build();

        self.add_action_entries([quit_action, about_action, preferences_action]);
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("app.preferences", &["<Control>comma"]);
        self.set_accels_for_action("win.show-help-overlay", &["<Control>question"]);
        self.set_accels_for_action("win.go-links", &["<Control>1"]);
        self.set_accels_for_action("win.go-texts", &["<Control>2"]);
        self.set_accels_for_action("win.go-files", &["<Control>3"]);
    }

    fn load_css(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/ee/s/app/style.css");

        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            // Register custom icons from resources
            let icon_theme = gtk::IconTheme::for_display(&display);
            icon_theme.add_resource_path("/ee/s/app/icons");

            // For development: add local icon directories
            // This allows finding the app icon without installing
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(project_root) = exe_path
                    .parent() // target/debug or target/release
                    .and_then(|p| p.parent()) // target
                    .and_then(|p| p.parent()) // project root
                {
                    let icons_path = project_root.join("data/icons");
                    if icons_path.exists() {
                        icon_theme.add_search_path(&icons_path);
                    }

                    // Also try adding hicolor directly for the app icon
                    let hicolor_path = project_root.join("data/icons/hicolor");
                    if hicolor_path.exists() {
                        icon_theme.add_search_path(&hicolor_path);
                    }
                }
            }

            // Set current working directory icons path as fallback
            let cwd_icons = std::path::PathBuf::from("data/icons");
            if cwd_icons.exists() {
                icon_theme.add_search_path(&cwd_icons);
                icon_theme.add_search_path(&cwd_icons.join("hicolor"));
            }
        }
    }

    fn show_about(&self) {
        let window = self.active_window();

        let about = adw::AboutDialog::builder()
            .application_name("S.EE")
            .application_icon(APP_ID)
            .developer_name("S.EE Team")
            .version(env!("CARGO_PKG_VERSION"))
            .website("https://s.ee")
            .issue_url("https://github.com/sdotee/desktop/issues")
            .license_type(gtk::License::MitX11)
            .developers(vec!["S.EE Team"])
            .copyright("© 2026 S.EE")
            .build();

        about.present(window.as_ref());
    }

    fn show_preferences(&self) {
        let window = self.active_window();
        let preferences = SeePreferencesWindow::new(self);
        if let Some(win) = window {
            preferences.set_transient_for(Some(&win));
        }
        preferences.present();
    }
}

impl Default for SeeApplication {
    fn default() -> Self {
        Self::new()
    }
}
