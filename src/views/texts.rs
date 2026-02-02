use crate::api::async_bridge::{spawn_api_call, ApiRequest, ApiResponse};
use crate::api::TextType;
use crate::config::Config;
use crate::storage::{HistoryStorage, TextEntry};
use crate::views::QrDialog;
use crate::widgets::TextRow;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use std::cell::{Cell, RefCell};

const ITEMS_PER_PAGE: usize = 10;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct TextsView {
        pub title_entry: RefCell<Option<adw::EntryRow>>,
        pub domain_combo: RefCell<Option<adw::ComboRow>>,
        pub type_combo: RefCell<Option<adw::ComboRow>>,
        pub content_view: RefCell<Option<gtk::TextView>>,
        pub create_button: RefCell<Option<gtk::Button>>,
        pub texts_list: RefCell<Option<gtk::ListBox>>,
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
    impl ObjectSubclass for TextsView {
        const NAME: &'static str = "SeeTextsView";
        type Type = super::TextsView;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for TextsView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for TextsView {}
    impl BoxImpl for TextsView {}
}

glib::wrapper! {
    pub struct TextsView(ObjectSubclass<imp::TextsView>)
        @extends gtk::Widget, gtk::Box;
}

impl TextsView {
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

        // Create Text group
        let create_group = adw::PreferencesGroup::builder()
            .title("Create Text")
            .description("Share text snippets, code, or notes")
            .build();
        create_group.add_css_class("create-card");

        let title_entry = adw::EntryRow::builder()
            .title("Title")
            .build();
        title_entry.set_tooltip_text(Some("Leave empty for 'Untitled'"));

        let domain_combo = adw::ComboRow::builder()
            .title("Domain")
            .subtitle("Loading domains...")
            .build();

        let type_combo = adw::ComboRow::builder()
            .title("Type")
            .subtitle("Content format")
            .build();

        // Set up type options
        let type_list = gtk::StringList::new(&["Plain Text", "Source Code", "Markdown"]);
        type_combo.set_model(Some(&type_list));
        type_combo.set_selected(0); // Default to Plain Text

        create_group.add(&title_entry);
        create_group.add(&domain_combo);
        create_group.add(&type_combo);

        // Text content editor with improved styling
        let text_frame = gtk::Frame::builder()
            .margin_top(12)
            .build();
        text_frame.add_css_class("text-editor-frame");

        let content_scrolled = gtk::ScrolledWindow::builder()
            .min_content_height(180)
            .max_content_height(300)
            .build();

        let content_view = gtk::TextView::builder()
            .wrap_mode(gtk::WrapMode::Word)
            .left_margin(16)
            .right_margin(16)
            .top_margin(16)
            .bottom_margin(16)
            .monospace(true)
            .build();
        content_view.add_css_class("see-textview");
        content_view.buffer().set_text("");

        content_scrolled.set_child(Some(&content_view));
        text_frame.set_child(Some(&content_scrolled));

        let create_button = gtk::Button::builder()
            .label("Create Text")
            .css_classes(["see-primary"])
            .halign(gtk::Align::End)
            .margin_top(16)
            .build();

        let button_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .halign(gtk::Align::End)
            .build();
        button_box.append(&create_button);

        // Recent Texts group with header
        let recent_group = adw::PreferencesGroup::builder()
            .title("Recent Texts")
            .build();
        recent_group.add_css_class("history-section");

        // Clear history button in header
        let clear_button = gtk::Button::builder()
            .label("Clear")
            .css_classes(["flat", "clear-history"])
            .build();
        clear_button.set_tooltip_text(Some("Clear local history"));
        recent_group.set_header_suffix(Some(&clear_button));

        let texts_list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();

        // Empty state placeholder
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
            .icon_name("see-text-symbolic")
            .pixel_size(64)
            .build();
        empty_icon.add_css_class("empty-state-icon");

        let empty_title = gtk::Label::builder()
            .label("No Texts Yet")
            .build();
        empty_title.add_css_class("empty-state-title");

        let empty_desc = gtk::Label::builder()
            .label("Shared texts will appear here")
            .wrap(true)
            .justify(gtk::Justification::Center)
            .build();
        empty_desc.add_css_class("empty-state-description");

        empty_box.append(&empty_icon);
        empty_box.append(&empty_title);
        empty_box.append(&empty_desc);
        texts_list.set_placeholder(Some(&empty_box));

        recent_group.add(&texts_list);

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
        content_box.append(&text_frame);
        content_box.append(&button_box);
        content_box.append(&recent_group);
        content_box.append(&pagination_box);

        clamp.set_child(Some(&content_box));
        scrolled.set_child(Some(&clamp));
        toast_overlay.set_child(Some(&scrolled));

        self.append(&toast_overlay);

        // Store references
        imp.title_entry.replace(Some(title_entry.clone()));
        imp.domain_combo.replace(Some(domain_combo.clone()));
        imp.type_combo.replace(Some(type_combo.clone()));
        imp.content_view.replace(Some(content_view.clone()));
        imp.create_button.replace(Some(create_button.clone()));
        imp.texts_list.replace(Some(texts_list.clone()));
        imp.toast_overlay.replace(Some(toast_overlay.clone()));
        imp.page_label.replace(Some(page_label.clone()));
        imp.prev_button.replace(Some(prev_button.clone()));
        imp.next_button.replace(Some(next_button.clone()));

        // Connect create button
        create_button.connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            move |_| {
                view.create_text();
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

        // Load existing texts
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
            storage.clear_texts();
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
        let receiver = spawn_api_call(config, ApiRequest::GetTextDomains);

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = view)]
            self,
            async move {
                if let Ok(response) = receiver.recv().await {
                    match response {
                        ApiResponse::GetTextDomains(Ok(domains)) => {
                            view.update_domains(domains);
                        }
                        ApiResponse::GetTextDomains(Err(e)) => {
                            log::error!("Failed to fetch text domains: {}", e);
                            view.update_domains(vec!["p.s.ee".to_string()]);
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
            if let Some(default_domain) = config.default_text_domain() {
                for (i, domain) in domains.iter().enumerate() {
                    if domain == default_domain {
                        combo.set_selected(i as u32);
                        break;
                    }
                }
            }
        }
    }

    fn create_text(&self) {
        let imp = self.imp();

        let content = imp
            .content_view
            .borrow()
            .as_ref()
            .map(|tv| {
                let buffer = tv.buffer();
                let (start, end) = buffer.bounds();
                buffer.text(&start, &end, true).to_string()
            })
            .unwrap_or_default();

        if content.is_empty() {
            self.show_toast("Please enter some text");
            return;
        }

        let title = imp
            .title_entry
            .borrow()
            .as_ref()
            .map(|e| e.text().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Untitled".to_string());

        // Get selected domain
        let domains = imp.domains.borrow();
        let domain = imp
            .domain_combo
            .borrow()
            .as_ref()
            .and_then(|c| domains.get(c.selected() as usize))
            .cloned();

        // Get selected type
        let text_type = imp.type_combo.borrow().as_ref().map(|c| {
            match c.selected() {
                0 => TextType::PlainText,
                1 => TextType::SourceCode,
                2 => TextType::Markdown,
                _ => TextType::PlainText,
            }
        });

        let config = Config::load().unwrap_or_default();

        let request = ApiRequest::CreateText {
            content: content.clone(),
            title: title.clone(),
            domain: domain.clone(),
            text_type,
        };

        let receiver = spawn_api_call(config, request);
        let content_preview = content.chars().take(100).collect::<String>();

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            title,
            #[strong]
            content_preview,
            async move {
                if let Ok(response) = receiver.recv().await {
                    match response {
                        ApiResponse::CreateText(Ok(result)) => {
                            // Extract domain from short_url
                            let short_url = &result.data.short_url;
                            let domain = short_url
                                .split('/')
                                .nth(2)
                                .unwrap_or("p.s.ee")
                                .to_string();

                            // Build page URL (share page)
                            let page_url = format!("https://{}/{}", domain, result.data.slug);

                            let entry = TextEntry::new(
                                short_url.clone(),
                                Some(page_url),
                                domain,
                                result.data.slug.clone(),
                                Some(title),
                                None,
                                content_preview,
                            );

                            // Save to storage
                            if let Some(ref mut storage) = *view.imp().storage.borrow_mut() {
                                if let Err(e) = storage.add_text(entry) {
                                    log::error!("Failed to save text: {}", e);
                                }
                            }

                            // Go to first page to see the new text
                            view.imp().current_page.set(0);
                            view.refresh_list();
                            view.clear_form();
                            view.show_toast("Text created successfully!");

                            // Copy to clipboard
                            if let Some(display) = gdk::Display::default() {
                                display.clipboard().set_text(short_url);
                            }
                        }
                        ApiResponse::CreateText(Err(e)) => {
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

        let texts_list = match imp.texts_list.borrow().as_ref() {
            Some(list) => list.clone(),
            None => return,
        };

        // Clear existing rows
        while let Some(child) = texts_list.first_child() {
            texts_list.remove(&child);
        }

        // Get texts from storage with pagination
        if let Some(ref storage) = *imp.storage.borrow() {
            let all_texts = storage.texts();
            let total_items = all_texts.len();
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
            for entry in all_texts.iter().skip(start).take(end - start) {
                let row = TextRow::new(entry);
                self.setup_row_actions(&row);
                texts_list.append(&row);
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

    fn setup_row_actions(&self, row: &TextRow) {
        let entry = row.entry();

        // Click on row to open share page in browser
        let entry_for_activate = entry.clone();
        row.connect_activated(move |_| {
            if let Some(ref entry) = entry_for_activate {
                let url = entry.page_url.as_ref().unwrap_or(&entry.url);
                let _ = gio::AppInfo::launch_default_for_uri(url, None::<&gio::AppLaunchContext>);
            }
        });

        // Copy Page URL button
        row.copy_page_button().connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            entry,
            move |_| {
                if let Some(ref entry) = entry {
                    if let Some(display) = gdk::Display::default() {
                        let url = entry.page_url.as_ref().unwrap_or(&entry.url);
                        display.clipboard().set_text(url);
                        view.show_toast("Share page URL copied");
                    }
                }
            }
        ));

        // Copy Raw URL button (add /raw suffix)
        row.copy_button().connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            entry,
            move |_| {
                if let Some(ref entry) = entry {
                    if let Some(display) = gdk::Display::default() {
                        let raw_url = format!("{}/raw", entry.url);
                        display.clipboard().set_text(&raw_url);
                        view.show_toast("Raw URL copied");
                    }
                }
            }
        ));

        // QR button - use page URL
        row.qr_button().connect_clicked(glib::clone!(
            #[weak(rename_to = view)]
            self,
            #[strong]
            entry,
            move |_| {
                if let Some(ref entry) = entry {
                    let url = entry.page_url.as_ref().unwrap_or(&entry.url);
                    let dialog = QrDialog::new(url);
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
                    view.delete_text(&entry.domain, &entry.slug);
                }
            }
        ));
    }

    fn delete_text(&self, domain: &str, slug: &str) {
        let config = Config::load().unwrap_or_default();
        let domain = domain.to_string();
        let slug = slug.to_string();

        let request = ApiRequest::DeleteText {
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
                        ApiResponse::DeleteText(Ok(())) => {
                            // Remove from storage
                            if let Some(ref mut storage) = *view.imp().storage.borrow_mut() {
                                if let Err(e) = storage.remove_text(&domain, &slug) {
                                    log::error!("Failed to remove text from storage: {}", e);
                                }
                            }

                            view.refresh_list();
                            view.show_toast("Text deleted");
                        }
                        ApiResponse::DeleteText(Err(e)) => {
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
        if let Some(ref entry) = *imp.title_entry.borrow() {
            entry.set_text("");
        }
        if let Some(ref tv) = *imp.content_view.borrow() {
            tv.buffer().set_text("");
        }
    }

    fn show_toast(&self, message: &str) {
        if let Some(ref overlay) = *self.imp().toast_overlay.borrow() {
            let toast = adw::Toast::new(message);
            overlay.add_toast(toast);
        }
    }
}

impl Default for TextsView {
    fn default() -> Self {
        Self::new()
    }
}
