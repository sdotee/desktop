use crate::qrcode::QrGenerator;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/ee/s/app/ui/qr_dialog.ui")]
    pub struct QrDialog {
        #[template_child]
        pub qr_picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub url_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub save_png_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub save_svg_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub save_pdf_button: TemplateChild<gtk::Button>,
        pub url: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QrDialog {
        const NAME: &'static str = "SeeQrDialog";
        type Type = super::QrDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for QrDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup();
        }
    }

    impl WidgetImpl for QrDialog {}
    impl AdwDialogImpl for QrDialog {}
}

glib::wrapper! {
    pub struct QrDialog(ObjectSubclass<imp::QrDialog>)
        @extends gtk::Widget, adw::Dialog;
}

impl QrDialog {
    pub fn new(url: &str) -> Self {
        let dialog: Self = glib::Object::new();
        dialog.set_url(url);
        dialog
    }

    fn setup(&self) {
        let imp = self.imp();

        // Save PNG button
        imp.save_png_button.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.save_as_png();
            }
        ));

        // Save SVG button
        imp.save_svg_button.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.save_as_svg();
            }
        ));

        // Save PDF button
        imp.save_pdf_button.connect_clicked(glib::clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.save_as_pdf();
            }
        ));
    }

    pub fn set_url(&self, url: &str) {
        let imp = self.imp();
        imp.url.replace(url.to_string());
        imp.url_label.set_label(url);

        // Generate and display QR code
        match QrGenerator::generate_texture(url, 256) {
            Ok(texture) => {
                imp.qr_picture.set_paintable(Some(&texture));
            }
            Err(e) => {
                log::error!("Failed to generate QR code: {}", e);
            }
        }
    }

    fn save_as_png(&self) {
        let url = self.imp().url.borrow().clone();
        let dialog = gtk::FileDialog::builder()
            .title("Save QR Code as PNG")
            .initial_name("qrcode.png")
            .build();

        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.png");
        filter.set_name(Some("PNG Images"));

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));

        dialog.save(
            self.root().and_downcast_ref::<gtk::Window>(),
            None::<&gio::Cancellable>,
            glib::clone!(
                #[strong]
                url,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            if let Err(e) = QrGenerator::save_png(&url, &path, 512) {
                                log::error!("Failed to save PNG: {}", e);
                            }
                        }
                    }
                }
            ),
        );
    }

    fn save_as_svg(&self) {
        let url = self.imp().url.borrow().clone();
        let dialog = gtk::FileDialog::builder()
            .title("Save QR Code as SVG")
            .initial_name("qrcode.svg")
            .build();

        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.svg");
        filter.set_name(Some("SVG Images"));

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));

        dialog.save(
            self.root().and_downcast_ref::<gtk::Window>(),
            None::<&gio::Cancellable>,
            glib::clone!(
                #[strong]
                url,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            if let Err(e) = QrGenerator::save_svg(&url, &path) {
                                log::error!("Failed to save SVG: {}", e);
                            }
                        }
                    }
                }
            ),
        );
    }

    fn save_as_pdf(&self) {
        let url = self.imp().url.borrow().clone();
        let dialog = gtk::FileDialog::builder()
            .title("Save QR Code as PDF")
            .initial_name("qrcode.pdf")
            .build();

        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.pdf");
        filter.set_name(Some("PDF Documents"));

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));

        dialog.save(
            self.root().and_downcast_ref::<gtk::Window>(),
            None::<&gio::Cancellable>,
            glib::clone!(
                #[strong]
                url,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            if let Err(e) = QrGenerator::save_pdf(&url, &path, 256.0) {
                                log::error!("Failed to save PDF: {}", e);
                            }
                        }
                    }
                }
            ),
        );
    }
}
