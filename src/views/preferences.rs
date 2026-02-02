use crate::api::async_bridge::{spawn_api_call, ApiRequest, ApiResponse};
use crate::application::SeeApplication;
use crate::config::Config;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/ee/s/app/ui/preferences.ui")]
    pub struct SeePreferencesWindow {
        #[template_child]
        pub api_key_entry: TemplateChild<adw::PasswordEntryRow>,
        #[template_child]
        pub base_url_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub default_link_domain_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub default_text_domain_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub default_file_domain_combo: TemplateChild<adw::ComboRow>,
        pub link_domains: RefCell<Vec<String>>,
        pub text_domains: RefCell<Vec<String>>,
        pub file_domains: RefCell<Vec<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SeePreferencesWindow {
        const NAME: &'static str = "SeePreferencesWindow";
        type Type = super::SeePreferencesWindow;
        type ParentType = adw::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SeePreferencesWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup();
        }
    }

    impl WidgetImpl for SeePreferencesWindow {}
    impl WindowImpl for SeePreferencesWindow {}
    impl AdwWindowImpl for SeePreferencesWindow {}
    impl PreferencesWindowImpl for SeePreferencesWindow {}
}

glib::wrapper! {
    pub struct SeePreferencesWindow(ObjectSubclass<imp::SeePreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow;
}

impl SeePreferencesWindow {
    pub fn new(app: &SeeApplication) -> Self {
        let window: Self = glib::Object::new();

        // Load current config values
        let config = app.config();
        let imp = window.imp();

        if let Some(api_key) = config.api_key() {
            imp.api_key_entry.set_text(api_key);
        }

        imp.base_url_entry.set_text(config.base_url());

        // Fetch domains from API for each type
        window.fetch_link_domains(config.default_link_domain().map(|s| s.to_string()));
        window.fetch_text_domains(config.default_text_domain().map(|s| s.to_string()));
        window.fetch_file_domains(config.default_file_domain().map(|s| s.to_string()));

        window
    }

    fn fetch_link_domains(&self, current_default: Option<String>) {
        let config = Config::load().unwrap_or_default();
        let receiver = spawn_api_call(config, ApiRequest::GetUrlDomains);

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = window)]
            self,
            async move {
                if let Ok(response) = receiver.recv().await {
                    match response {
                        ApiResponse::GetUrlDomains(Ok(domains)) => {
                            window.update_link_domains(domains, current_default.as_deref());
                        }
                        ApiResponse::GetUrlDomains(Err(e)) => {
                            log::error!("Failed to fetch link domains: {}", e);
                            window.update_link_domains(vec!["s.ee".to_string()], current_default.as_deref());
                        }
                        _ => {}
                    }
                }
            }
        ));
    }

    fn fetch_text_domains(&self, current_default: Option<String>) {
        let config = Config::load().unwrap_or_default();
        let receiver = spawn_api_call(config, ApiRequest::GetTextDomains);

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = window)]
            self,
            async move {
                if let Ok(response) = receiver.recv().await {
                    match response {
                        ApiResponse::GetTextDomains(Ok(domains)) => {
                            window.update_text_domains(domains, current_default.as_deref());
                        }
                        ApiResponse::GetTextDomains(Err(e)) => {
                            log::error!("Failed to fetch text domains: {}", e);
                            window.update_text_domains(vec!["p.s.ee".to_string()], current_default.as_deref());
                        }
                        _ => {}
                    }
                }
            }
        ));
    }

    fn fetch_file_domains(&self, current_default: Option<String>) {
        let config = Config::load().unwrap_or_default();
        let receiver = spawn_api_call(config, ApiRequest::GetFileDomains);

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = window)]
            self,
            async move {
                if let Ok(response) = receiver.recv().await {
                    match response {
                        ApiResponse::GetFileDomains(Ok(domains)) => {
                            window.update_file_domains(domains, current_default.as_deref());
                        }
                        ApiResponse::GetFileDomains(Err(e)) => {
                            log::error!("Failed to fetch file domains: {}", e);
                            window.update_file_domains(vec!["i.s.ee".to_string()], current_default.as_deref());
                        }
                        _ => {}
                    }
                }
            }
        ));
    }

    fn update_link_domains(&self, domains: Vec<String>, current_default: Option<&str>) {
        let imp = self.imp();
        imp.link_domains.replace(domains.clone());

        let string_list =
            gtk::StringList::new(&domains.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        imp.default_link_domain_combo.set_model(Some(&string_list));
        imp.default_link_domain_combo.set_subtitle("URL shortening");

        // Select current default domain if set
        if let Some(default) = current_default {
            for (i, domain) in domains.iter().enumerate() {
                if domain == default {
                    imp.default_link_domain_combo.set_selected(i as u32);
                    break;
                }
            }
        }
    }

    fn update_text_domains(&self, domains: Vec<String>, current_default: Option<&str>) {
        let imp = self.imp();
        imp.text_domains.replace(domains.clone());

        let string_list =
            gtk::StringList::new(&domains.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        imp.default_text_domain_combo.set_model(Some(&string_list));
        imp.default_text_domain_combo.set_subtitle("Text sharing");

        // Select current default domain if set
        if let Some(default) = current_default {
            for (i, domain) in domains.iter().enumerate() {
                if domain == default {
                    imp.default_text_domain_combo.set_selected(i as u32);
                    break;
                }
            }
        }
    }

    fn update_file_domains(&self, domains: Vec<String>, current_default: Option<&str>) {
        let imp = self.imp();
        imp.file_domains.replace(domains.clone());

        let string_list =
            gtk::StringList::new(&domains.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        imp.default_file_domain_combo.set_model(Some(&string_list));
        imp.default_file_domain_combo.set_subtitle("File uploads");

        // Select current default domain if set
        if let Some(default) = current_default {
            for (i, domain) in domains.iter().enumerate() {
                if domain == default {
                    imp.default_file_domain_combo.set_selected(i as u32);
                    break;
                }
            }
        }
    }

    fn setup(&self) {
        let imp = self.imp();

        // Save API key on change
        imp.api_key_entry.connect_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |entry| {
                window.save_config(|config| {
                    let text = entry.text();
                    config.api_key = if text.is_empty() {
                        None
                    } else {
                        Some(text.to_string())
                    };
                });
            }
        ));

        // Save base URL on change
        imp.base_url_entry.connect_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |entry| {
                window.save_config(|config| {
                    let text = entry.text();
                    config.base_url = if text.is_empty() {
                        None
                    } else {
                        Some(text.to_string())
                    };
                });
            }
        ));

        // Save default link domain on change
        imp.default_link_domain_combo
            .connect_selected_notify(glib::clone!(
                #[weak(rename_to = window)]
                self,
                move |combo| {
                    let imp = window.imp();
                    let domains = imp.link_domains.borrow();
                    let selected = combo.selected() as usize;
                    if let Some(domain) = domains.get(selected) {
                        window.save_config(|config| {
                            config.default_link_domain = Some(domain.clone());
                        });
                    }
                }
            ));

        // Save default text domain on change
        imp.default_text_domain_combo
            .connect_selected_notify(glib::clone!(
                #[weak(rename_to = window)]
                self,
                move |combo| {
                    let imp = window.imp();
                    let domains = imp.text_domains.borrow();
                    let selected = combo.selected() as usize;
                    if let Some(domain) = domains.get(selected) {
                        window.save_config(|config| {
                            config.default_text_domain = Some(domain.clone());
                        });
                    }
                }
            ));

        // Save default file domain on change
        imp.default_file_domain_combo
            .connect_selected_notify(glib::clone!(
                #[weak(rename_to = window)]
                self,
                move |combo| {
                    let imp = window.imp();
                    let domains = imp.file_domains.borrow();
                    let selected = combo.selected() as usize;
                    if let Some(domain) = domains.get(selected) {
                        window.save_config(|config| {
                            config.default_file_domain = Some(domain.clone());
                        });
                    }
                }
            ));
    }

    fn save_config<F>(&self, f: F)
    where
        F: FnOnce(&mut Config),
    {
        let mut config = Config::load().unwrap_or_default();
        f(&mut config);
        if let Err(e) = config.save() {
            log::error!("Failed to save config: {}", e);
        }
    }
}
