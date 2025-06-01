// Modules
pub(crate) mod wgpuarea;

// Re-Exports
pub use wgpuarea::WgpuArea;

// Imports
use gtk4::{glib, prelude::*};

fn main() -> glib::ExitCode {
    wgpuarea::init_epoxy();
    let application =
        gtk4::Application::new(Some("com.github.flxzt.gtkwgpuarea"), Default::default());
    application.connect_activate(build_ui);
    application.run()
}

fn build_ui(application: &gtk4::Application) {
    let window = gtk4::ApplicationWindow::builder()
        .application(application)
        .title("GtkWgpuArea")
        .default_width(800)
        .default_height(600)
        .build();
    let wgpu_area = WgpuArea::default();
    window.set_child(Some(&wgpu_area));
    window.present();
}
