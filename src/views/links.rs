use crate::api::async_bridge::{spawn_api_call, ApiRequest, ApiResponse};
use crate::config::Config;
use crate::storage::{HistoryStorage, LinkEntry};
use crate::views::QrDialog;
use crate::widgets::LinkRow;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use std::cell::{Cell, RefCell};

const ITEMS_PER_PAGE: usize = 10;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct LinksView {
        pub url_entry: RefCell<Option<adw::EntryRow>>,
        pub domain_combo: RefCell<Option<adw::ComboRow>>,
        pub slug_entry: RefCell<Option<adw::EntryRow>>,
        pub shorten_button: RefCell<Option<gtk::Button>>,
        pub links_list: RefCell<Option<gtk::ListBox>>,
        pub toast_overlay: RefCell<Option<adw::ToastOverlay>>,
        pub storage: RefCell<Option<HistoryStorage>>,
        pub domains: RefCell<Vec<String>>,
        // Pagination
        pub current_page: Cell<usize>,
        pub page_label: RefCell<Option<gtk::Label>>,
        pub prev_button: RefCell<Option<gtk::Button>>,
        pub next_button: RefCell<Option<gtk::Button>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LinksView {
        const NAME: &'static str = "SeeLinksView";
        type Type = super::LinksView;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for LinksView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for LinksView {}
    impl BoxImpl for LinksView {}
}

glib::wrapper! {
    pub struct LinksView(ObjectSubclass<imp::LinksView>)
        @extends gtk::Widget, gtk::Box;
}

impl LinksView {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("orientation", gtk::Orientation::Vertical)
            .property("hexpand", true)
            .property("vexpand", true)
            .build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        // Load storage
        match HistoryStorage::load() {
            Ok(storage) => {
                imp.storage.replace(Some(storage));
            }
            Err(e) => {
                log::error!("Failed to load history: {}", e);
            }
        }

        // Toast overlay wraps everything
        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_hexpand(true);
        toast_overlay.set_vexpand(true);

        // Scrolled window for content
        let scrolled = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        let clamp = adw::Clamp::builder()
            .maximum_size(700)
            .margin_start(16)
            .margin_end(16)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        let content_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(24)
            .build();

        // Create Short Link group with card styling
        let create_group = adw::PreferencesGroup::builder()
            .title("Create Short Link")
            .description("Paste a long URL to shorten it")
            .build();
        create_group.add_css_class("create-card");

        let url_entry = adw::EntryRow::builder()
            .title("URL")
            .build();
        url_entry.add_css_class("url-entry");

        let domain_combo = adw::ComboRow::builder()
            .title("Domain")
            .subtitle("Loading domains...")
            .build();

        let slug_entry = adw::EntryRow::builder()
            .title("Custom Alias")
            .build();
        slug_entry.set_tooltip_text(Some("Leave empty for auto-generated alias"));

        let shorten_button = gtk::Button::builder()
            .label("Shorten URL")
            .css_classes(["see-primary"])
            .halign(gtk::Align::End)
            .margin_top(16)
            .build();

        create_group.add(&url_entry);
        create_group.add(&domain_combo);
        create_group.add(&slug_entry);

        let button_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .halign(gtk::Align::End)
            .build();
        button_box.append(&shorten_button);

        // Recent Links group with header
        let recent_group = adw::PreferencesGroup::builder()
            .title("Recent Links")
            .build();
        recent_group.add_css_class("history-section");

        // Clear history button in header
        let clear_button = gtk::Button::builder()
            .label("Clear")
            .css_classes(["flat", "clear-history"])
            .build();
        clear_button.set_tooltip_text(Some("Clear local history"));
        recent_group.set_header_suffix(Some(&clear_button));

        let links_list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();

        // Set placeholder for empty state
        let empty_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .spacing(8)
            .margin_top(32)
            .margin_bottom(32)
            .build();
        empty_box.add_css_class("empty-state");

        let empty_icon = gtk::Image::builder()
            .icon_name("see-link-symbolic")
            .pixel_size(64)
            .build();
        empty_icon.add_css_class("empty-state-icon");

        let empty_title = gtk::Label::builder()
            .label("No Links Yet")
            .build();
        empty_title.add_css_class("empty-state-title");

        let empty_desc = gtk::Label::builder()
            .label("Shortened URLs will appear here")
            .wrap(true)
            .justify(gtk::Justification::Center)
            .build();
        empty_desc.add_css_class("empty-state-description");

        empty_box.append(&empty_icon);
        empty_box.append(&empty_title);
        empty_box.append(&empty_desc);
        links_list.set_placeholder(Some(&empty_box));

        recent_group.add(&links_list);

        // Pagination controls
        let pagination_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .halign(gtk::Align::Center)
            .spacing(16)
            .margin_top(16)
            .build();
        pagination_box.add_css_class("pagination-box");

        let prev_button = gtk::Button::builder()
            .icon_name("go-previous-symbolic")
            .css_classes(["flat", "circular"])
            .sensitive(false)
            .build();
        prev_button.set_tooltip_text(Some("Previous page"));

        let page_label = gtk::Label::builder()
            .label("Page 1 / 1")
            .build();
        page_label.add_css_class("pagination-label");

        let next_button = gtk::Button::builder()
            .icon_name("go-next-symbolic")
            .css_classes(["flat", "circular"])
            .sensitive(false)
            .build();
        next_button.set_tooltip_text(Some("Next page"));

        pagination_box.append(&prev_button);
        pagination_box.append(&page_label);
        pagination_box.append(&next_button);

        content_box.append(&create_group);
        content_box.append(&button_box);
        content_box.append(&recent_group);
        content_box.append(&pagination_box);

        clamp.set_child(Some(&content_box));
        scrolled.set_child(Some(&clamp));
        toast_overlay.set_child(Some(&scrolled));

        self.append(&toast_overlay);

        // Store references
        imp.url_entry.replace(Some(url_entry.clone()));
        imp.domain_combo.replace(Some(domain_combo.clone()));
        imp.slug_entry.replace(Some(slug_entry.clone()));
        imp.shorten_button.replace(Some(shorten_button.clone()));
        imp.links_list.replace(Some(links_list.clone()));
        imp.toast_overlay.replace(Some(toast_overlay.clone()));
        imp.page_label.replace(Some(page_label.clone()));
        imp.prev_button.replace(Some(prev_button.clone()));
        imp.next_button.replace(Some(next_button.clone()));

        // Connect shorten button
        shorten_button.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                view.shorten_url();
            }
        ));

        // Connect clear history button
        clear_button.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                view.show_clear_history_dialog();
            }
        ));

        // Connect pagination buttons
        prev_button.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                let current = view.imp().current_page.get();
                if current > 0 {
                    view.imp().current_page.set(current - 1);
                    view.refresh_list();
                }
            }
        ));

        next_button.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                let current = view.imp().current_page.get();
                view.imp().current_page.set(current + 1);
                view.refresh_list();
            }
        ));

        // Load existing links
        self.refresh_list();

        // Fetch domains from API
        self.fetch_domains();
    }

    fn show_clear_history_dialog(&self) {
        let dialog = adw::AlertDialog::builder()
            .heading("Clear Local History?")
            .body("This will only clear the local history on this device.\n\nTo delete your data from S.EE servers, please visit s.ee/user/dashboard")
            .build();

        dialog.add_response("cancel", "Cancel");
        dialog.add_response("clear", "Clear");
        dialog.set_response_appearance("clear", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");

        dialog.connect_response(
            None,
            glib::clone!(
                #[weak(rename_to = view)]
                self,
                move |_, response| {
                    if response == "clear" {
                        view.clear_local_history();
                    }
                }
            ),
        );

        if let Some(window) = self.root().and_downcast_ref::<gtk::Window>() {
            dialog.present(Some(window));
        }
    }

    fn clear_local_history(&self) {
        let imp = self.imp();
        if let Some(ref mut storage) = *imp.storage.borrow_mut() {
            storage.clear_links();
            if let Err(e) = storage.save() {
                log::error!("Failed to save after clearing history: {}", e);
            }
        }
        imp.current_page.set(0);
        self.refresh_list();
        self.show_toast("Local history cleared");
    }

    fn fetch_domains(&self) {
        let config = Config::load().unwrap_or_default();
        let receiver = spawn_api_call(config, ApiRequest::GetUrlDomains);

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = view)]
            self,
            async move {
                if let Ok(response) = receiver.recv().await {
                    match response {
                        ApiResponse::GetUrlDomains(Ok(domains)) => {
                            view.update_domains(domains);
                        }
                        ApiResponse::GetUrlDomains(Err(e)) => {
                            log::error!("Failed to fetch domains: {}", e);
                            // Use default domain as fallback
                            view.update_domains(vec!["s.ee".to_string()]);
                        }
                        _ => {}
                    }
                }
            }
        ));
    }

    fn update_domains(&self, domains: Vec<String>) {
        let imp = self.imp();
        imp.domains.replace(domains.clone());

        if let Some(combo) = imp.domain_combo.borrow().as_ref() {
            let string_list =
                gtk::StringList::new(&domains.iter().map(|s| s.as_str()).collect::<Vec<_>>());
            combo.set_model(Some(&string_list));
            combo.set_subtitle("");

            // Select default domain from config
            let config = Config::load().unwrap_or_default();
            if let Some(default_domain) = config.default_link_domain() {
                for (i, domain) in domains.iter().enumerate() {
                    if domain == default_domain {
                        combo.set_selected(i as u32);
                        break;
                    }
                }
            }
        }
    }

    fn shorten_url(&self) {
        let imp = self.imp();

        let url_entry = imp.url_entry.borrow();
        let domain_combo = imp.domain_combo.borrow();
        let slug_entry = imp.slug_entry.borrow();

        let url = url_entry
            .as_ref()
            .map(|e| e.text().to_string())
            .unwrap_or_default();
        if url.is_empty() {
            self.show_toast("Please enter a URL");
            return;
        }

        let domains = imp.domains.borrow();
        let domain = domain_combo
            .as_ref()
            .and_then(|c| domains.get(c.selected() as usize))
            .cloned()
            .unwrap_or_else(|| "s.ee".to_string());

        let slug = slug_entry
            .as_ref()
            .map(|e| e.text().to_string())
            .filter(|s| !s.is_empty());

        let config = Config::load().unwrap_or_default();

        let request = ApiRequest::ShortenUrl {
            url: url.clone(),
            domain: Some(domain.clone()),
            slug: slug.clone(),
        };

        let receiver = spawn_api_call(config, request);

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            url,
            #[strong]
            domain,
            async move {
                if let Ok(response) = receiver.recv().await {
                    match response {
                        ApiResponse::ShortenUrl(Ok(result)) => {
                            let entry = LinkEntry::new(
                                url,
                                result.data.short_url.clone(),
                                domain,
                                result.data.slug.clone(),
                                None,
                            );

                            // Save to storage
                            if let Some(ref mut storage) = *view.imp().storage.borrow_mut() {
                                if let Err(e) = storage.add_link(entry) {
                                    log::error!("Failed to save link: {}", e);
                                }
                            }

                            // Go to first page to see the new link
                            view.imp().current_page.set(0);
                            view.refresh_list();
                            view.clear_form();
                            view.show_toast("Link shortened successfully!");

                            // Copy to clipboard
                            if let Some(display) = gdk::Display::default() {
                                display.clipboard().set_text(&result.data.short_url);
                            }
                        }
                        ApiResponse::ShortenUrl(Err(e)) => {
                            view.show_toast(&format!("Error: {}", e));
                        }
                        _ => {}
                    }
                }
            }
        ));
    }

    fn refresh_list(&self) {
        let imp = self.imp();

        let links_list = match imp.links_list.borrow().as_ref() {
            Some(list) => list.clone(),
            None => return,
        };

        // Clear existing rows
        while let Some(child) = links_list.first_child() {
            links_list.remove(&child);
        }

        // Get links from storage with pagination
        if let Some(ref storage) = *imp.storage.borrow() {
            let all_links = storage.links();
            let total_items = all_links.len();
            let total_pages = (total_items + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE.max(1);
            let current_page = imp.current_page.get();

            // Ensure current page is valid
            let current_page = if total_pages == 0 {
                0
            } else {
                current_page.min(total_pages - 1)
            };
            imp.current_page.set(current_page);

            // Calculate slice
            let start = current_page * ITEMS_PER_PAGE;
            let end = (start + ITEMS_PER_PAGE).min(total_items);

            // Add rows for current page
            for entry in all_links.iter().skip(start).take(end - start) {
                let row = LinkRow::new(entry);
                self.setup_row_actions(&row);
                links_list.append(&row);
            }

            // Update pagination controls
            self.update_pagination(current_page, total_pages);
        }
    }

    fn update_pagination(&self, current_page: usize, total_pages: usize) {
        let imp = self.imp();

        if let Some(label) = imp.page_label.borrow().as_ref() {
            if total_pages == 0 {
                label.set_label("No items");
            } else {
                label.set_label(&format!("Page {} / {}", current_page + 1, total_pages));
            }
        }

        if let Some(prev) = imp.prev_button.borrow().as_ref() {
            prev.set_sensitive(current_page > 0);
        }

        if let Some(next) = imp.next_button.borrow().as_ref() {
            next.set_sensitive(total_pages > 0 && current_page < total_pages - 1);
        }
    }

    fn setup_row_actions(&self, row: &LinkRow) {
        let entry = row.entry();

        // Click on row to open in browser
        let entry_for_activate = entry.clone();
        row.connect_activated(move |_| {
            if let Some(ref entry) = entry_for_activate {
                let _ = gio::AppInfo::launch_default_for_uri(&entry.short_url, None::<&gio::AppLaunchContext>);
            }
        });

        // Copy button
        row.copy_button().connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            entry,
            move |_| {
                if let Some(ref entry) = entry {
                    if let Some(display) = gdk::Display::default() {
                        display.clipboard().set_text(&entry.short_url);
                        view.show_toast("Copied to clipboard");
                    }
                }
            }
        ));

        // QR button
        row.qr_button().connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            entry,
            move |_| {
                if let Some(ref entry) = entry {
                    let dialog = QrDialog::new(&entry.short_url);
                    if let Some(window) = view.root().and_downcast_ref::<gtk::Window>() {
                        dialog.present(Some(window));
                    }
                }
            }
        ));

        // Delete button
        row.delete_button().connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            entry,
            move |_| {
                if let Some(ref entry) = entry {
                    view.delete_link(&entry.domain, &entry.slug);
                }
            }
        ));
    }

    fn delete_link(&self, domain: &str, slug: &str) {
        let config = Config::load().unwrap_or_default();
        let domain = domain.to_string();
        let slug = slug.to_string();

        let request = ApiRequest::DeleteUrl {
            domain: domain.clone(),
            slug: slug.clone(),
        };

        let receiver = spawn_api_call(config, request);

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            domain,
            #[strong]
            slug,
            async move {
                if let Ok(response) = receiver.recv().await {
                    match response {
                        ApiResponse::DeleteUrl(Ok(())) => {
                            // Remove from storage
                            if let Some(ref mut storage) = *view.imp().storage.borrow_mut() {
                                if let Err(e) = storage.remove_link(&domain, &slug) {
                                    log::error!("Failed to remove link from storage: {}", e);
                                }
                            }

                            view.refresh_list();
                            view.show_toast("Link deleted");
                        }
                        ApiResponse::DeleteUrl(Err(e)) => {
                            view.show_toast(&format!("Error: {}", e));
                        }
                        _ => {}
                    }
                }
            }
        ));
    }

    fn clear_form(&self) {
        let imp = self.imp();
        if let Some(ref entry) = *imp.url_entry.borrow() {
            entry.set_text("");
        }
        if let Some(ref entry) = *imp.slug_entry.borrow() {
            entry.set_text("");
        }
    }

    fn show_toast(&self, message: &str) {
        if let Some(ref overlay) = *self.imp().toast_overlay.borrow() {
            let toast = adw::Toast::new(message);
            overlay.add_toast(toast);
        }
    }
}

impl Default for LinksView {
    fn default() -> Self {
        Self::new()
    }
}
