// Modules
pub(crate) mod wgpuarea;

// Re-Exports
pub(crate) use wgpuarea::WgpuArea;

// Imports
use gtk4::{glib, prelude::*};

fn main() -> glib::ExitCode {
    std::env::set_var("GDK_DEBUG", "gl-disable-gl");
    std::env::set_var("GSK_RENDERER", "ngl");
    init_epoxy();

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

/// Load GL pointers from epoxy (GL context management library used by GTK).
pub fn init_epoxy() {
    #[cfg(target_os = "macos")]
    let library = unsafe { libloading::os::unix::Library::new("libepoxy.0.dylib") }.unwrap();
    #[cfg(all(unix, not(target_os = "macos")))]
    let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();
    #[cfg(windows)]
    let library = libloading::os::windows::Library::open_already_loaded("libepoxy-0.dll")
        .or_else(|_| libloading::os::windows::Library::open_already_loaded("epoxy-0.dll"))
        .unwrap();

    epoxy::load_with(|name| {
        unsafe { library.get::<_>(name.as_bytes()) }
            .map(|symbol| *symbol)
            .unwrap_or_else(|e| {
                eprintln!("failed to init epoxy, Err: {e:?}");
                std::ptr::null()
            })
    });
}
