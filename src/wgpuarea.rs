use gtk4::{gdk, glib, prelude::*, subclass::prelude::*, Align};
use std::cell::RefCell;

struct Renderer {
    exposed: wgpu_hal::ExposedAdapter<wgpu_hal::api::Gles>,
}

impl Renderer {
    fn new() -> Self {
        static LOAD_FN: fn(&str) -> *const std::ffi::c_void =
            |s| epoxy::get_proc_addr(s) as *const _;

        let exposed = unsafe { wgpu_hal::gles::Adapter::new_external(LOAD_FN) }
            .expect("Initializing new wgpu_hal gles Adapter.");

        Self { exposed }
    }

    // From wgpu-hal/examples/raw-gles
    fn fill_screen(&self, width: u32, height: u32) {
        use wgpu_hal::{Adapter as _, CommandEncoder as _, Device as _, Queue as _};

        let od = unsafe {
            self.exposed.adapter.open(
                wgpu_types::Features::empty(),
                &wgpu_types::Limits::downlevel_defaults(),
            )
        }
        .unwrap();

        let format = wgpu_types::TextureFormat::Rgba8UnormSrgb;
        let texture = <wgpu_hal::api::Gles as wgpu_hal::Api>::Texture::default_framebuffer(format);
        let view = unsafe {
            od.device
                .create_texture_view(
                    &texture,
                    &wgpu_hal::TextureViewDescriptor {
                        label: None,
                        format,
                        dimension: wgpu_types::TextureViewDimension::D2,
                        usage: wgpu_hal::TextureUses::COLOR_TARGET,
                        range: wgpu_types::ImageSubresourceRange::default(),
                    },
                )
                .unwrap()
        };

        println!("Filling the screen. Dimensions: ({width}, {height})");
        let mut encoder = unsafe {
            od.device
                .create_command_encoder(&wgpu_hal::CommandEncoderDescriptor {
                    label: None,
                    queue: &od.queue,
                })
                .unwrap()
        };
        let rp_desc = wgpu_hal::RenderPassDescriptor {
            label: None,
            extent: wgpu_types::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            sample_count: 1,
            color_attachments: &[Some(wgpu_hal::ColorAttachment {
                target: wgpu_hal::Attachment {
                    view: &view,
                    usage: wgpu_hal::TextureUses::COLOR_TARGET,
                },
                resolve_target: None,
                ops: wgpu_hal::AttachmentOps::STORE,
                clear_value: wgpu_types::Color::BLUE,
            })],
            depth_stencil_attachment: None,
            multiview: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        unsafe {
            encoder.begin_encoding(None).unwrap();
            encoder.begin_render_pass(&rp_desc);
            encoder.end_render_pass();
            let cmd_buf = encoder.end_encoding().unwrap();
            od.queue.submit(&[&cmd_buf], None).unwrap();
        }
    }
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct WgpuArea {
        renderer: RefCell<Option<Renderer>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WgpuArea {
        const NAME: &'static str = "WgpuArea";
        type Type = super::WgpuArea;
        type ParentType = gtk4::GLArea;
    }

    impl ObjectImpl for WgpuArea {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().set_allowed_apis(gdk::GLAPI::GLES);
            self.obj().set_has_stencil_buffer(true);
            self.obj().set_has_depth_buffer(true);

            self.obj().set_halign(Align::Fill);
            self.obj().set_valign(Align::Fill);
            self.obj().set_hexpand(true);
            self.obj().set_vexpand(true);
        }
    }

    impl WidgetImpl for WgpuArea {
        fn realize(&self) {
            self.parent_realize();

            if let Some(e) = self.obj().error() {
                eprintln!("error in WgpuArea realize, Err: {e:?}");
                return;
            }
        }

        fn unrealize(&self) {
            self.renderer.replace(None);
            self.parent_unrealize();
        }
    }

    impl GLAreaImpl for WgpuArea {
        fn resize(&self, _width: i32, _height: i32) {
            self.ensure_renderer();
        }

        fn render(&self, _context: &gdk::GLContext) -> glib::Propagation {
            self.ensure_renderer();
            let (width, height) = self.get_dimensions();
            if let Some(renderer) = self.renderer.borrow().as_ref() {
                renderer.fill_screen(width, height);
            }
            glib::Propagation::Stop
        }
    }

    impl WgpuArea {
        fn ensure_renderer(&self) {
            self.obj().attach_buffers();
            if self.renderer.borrow().is_some() {
                return;
            }
            self.renderer.replace(Some(Renderer::new()));
        }

        fn get_dimensions(&self) -> (u32, u32) {
            let scale = self.obj().scale_factor();
            let width = self.obj().width();
            let height = self.obj().height();
            ((width * scale) as u32, (height * scale) as u32)
        }
    }
}

glib::wrapper! {
    pub struct WgpuArea(ObjectSubclass<imp::WgpuArea>)
        @extends gtk4::Widget, gtk4::GLArea;
}

impl Default for WgpuArea {
    fn default() -> Self {
        glib::Object::new()
    }
}
