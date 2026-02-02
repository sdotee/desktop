use crate::application::SeeApplication;
use crate::views::{FilesView, LinksView, TextsView};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/ee/s/app/ui/window.ui")]
    pub struct SeeWindow {
        #[template_child]
        pub view_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub links_page: TemplateChild<gtk::Box>,
        #[template_child]
        pub texts_page: TemplateChild<gtk::Box>,
        #[template_child]
        pub files_page: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SeeWindow {
        const NAME: &'static str = "SeeWindow";
        type Type = super::SeeWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SeeWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_views();
            obj.setup_actions();
            obj.load_window_state();
        }
    }

    impl WidgetImpl for SeeWindow {}

    impl WindowImpl for SeeWindow {
        fn close_request(&self) -> glib::Propagation {
            let obj = self.obj();
            obj.save_window_state();
            glib::Propagation::Proceed
        }
    }

    impl ApplicationWindowImpl for SeeWindow {}
    impl AdwApplicationWindowImpl for SeeWindow {}
}

glib::wrapper! {
    pub struct SeeWindow(ObjectSubclass<imp::SeeWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl SeeWindow {
    pub fn new(app: &SeeApplication) -> Self {
        glib::Object::builder()
            .property("application", app)
            .build()
    }

    fn setup_views(&self) {
        let imp = self.imp();

        // Links View
        let links_view = LinksView::new();
        imp.links_page.append(&links_view);

        // Texts View
        let texts_view = TextsView::new();
        imp.texts_page.append(&texts_view);

        // Files View
        let files_view = FilesView::new();
        imp.files_page.append(&files_view);
    }

    fn setup_actions(&self) {
        let action_go_links = gio::ActionEntry::builder("go-links")
            .activate(|win: &Self, _, _| {
                win.imp().view_stack.set_visible_child_name("links");
            })
            .build();

        let action_go_texts = gio::ActionEntry::builder("go-texts")
            .activate(|win: &Self, _, _| {
                win.imp().view_stack.set_visible_child_name("texts");
            })
            .build();

        let action_go_files = gio::ActionEntry::builder("go-files")
            .activate(|win: &Self, _, _| {
                win.imp().view_stack.set_visible_child_name("files");
            })
            .build();

        self.add_action_entries([action_go_links, action_go_texts, action_go_files]);

        // Set up shortcuts window
        let builder = gtk::Builder::from_resource("/ee/s/app/ui/shortcuts.ui");
        let shortcuts_window: gtk::ShortcutsWindow = builder.object("help_overlay").unwrap();
        self.set_help_overlay(Some(&shortcuts_window));
    }

    fn load_window_state(&self) {
        // Try to load from GSettings, fall back to defaults if schema not installed
        if let Some(schema_source) = gio::SettingsSchemaSource::default() {
            if schema_source.lookup(crate::APP_ID, true).is_some() {
                let settings = gio::Settings::new(crate::APP_ID);
                let width = settings.int("window-width");
                let height = settings.int("window-height");
                let maximized = settings.boolean("window-maximized");
                self.set_default_size(width, height);
                if maximized {
                    self.maximize();
                }
                return;
            }
        }
        // Default size if no settings
        self.set_default_size(800, 600);
    }

    fn save_window_state(&self) {
        // Only save if schema is installed
        if let Some(schema_source) = gio::SettingsSchemaSource::default() {
            if schema_source.lookup(crate::APP_ID, true).is_some() {
                let settings = gio::Settings::new(crate::APP_ID);
                let (width, height) = self.default_size();
                let maximized = self.is_maximized();
                let _ = settings.set_int("window-width", width);
                let _ = settings.set_int("window-height", height);
                let _ = settings.set_boolean("window-maximized", maximized);
            }
        }
    }
}
