use glow;
use gtk4::{gdk, glib, prelude::*, subclass::prelude::*, Align};
use std::cell::RefCell;
use std::num::NonZeroU32;
use wgpu::TextureUses;
use wgpu_hal as hal;
use wgpu_types as wgt;
use wgpu_hal::gles::TextureInner;

struct Renderer {
    exposed: wgpu_hal::ExposedAdapter<wgpu_hal::api::Gles>,
    fbo: glow::NativeFramebuffer,
}

impl Renderer {
    fn new() -> Self {
        use glow::HasContext;

        static LOAD_FN: fn(&str) -> *const std::ffi::c_void =
            |s| epoxy::get_proc_addr(s) as *const _;

        let exposed = unsafe {
            wgpu_hal::gles::Adapter::new_external(
                LOAD_FN,
                wgpu::GlBackendOptions::from_env_or_default(),
            )
        }
        .expect("Initializing new wgpu_hal gles Adapter.");

        // also need the nativeframebuffer here
        let fbo = unsafe {
            let ctx = glow::Context::from_loader_function(LOAD_FN);
            let id = NonZeroU32::new(ctx.get_parameter_i32(glow::DRAW_FRAMEBUFFER_BINDING) as u32)
                .expect("No GTK provided framebuffer binding");
            ctx.bind_framebuffer(glow::FRAMEBUFFER, None);
            // the view will be created by glow after binding to the correct framebuffer;
            glow::NativeFramebuffer(id)
        };

        Self { exposed, fbo }
    }

    // From wgpu-hal/examples/raw-gles
    fn fill_screen(&self, width: u32, height: u32) {
        use wgpu_hal::{Adapter as _, CommandEncoder as _, Device as _, Queue as _};

        let od = unsafe {
            self.exposed.adapter.open(
                wgt::Features::empty(),
                &wgt::Limits::downlevel_defaults(),
                &wgt::MemoryHints::default(),
            )
        }
        .unwrap();

        let format = wgt::TextureFormat::Rgba8UnormSrgb;
        let mut texture = <hal::api::Gles as hal::Api>::Texture::default_framebuffer(format);
        // then add the external gl inner texture
        texture.inner = TextureInner::ExternalGlFrameBuffer { inner: self.fbo };


        let view = unsafe {
            od.device
                .create_texture_view(
                    &texture,
                    &hal::TextureViewDescriptor {
                        label: None,
                        format,
                        dimension: wgt::TextureViewDimension::D2,
                        usage: TextureUses::COLOR_TARGET,
                        range: wgt::ImageSubresourceRange::default(),
                    },
                )
                .unwrap()
        };

        println!("Filling the screen");
        let mut encoder = unsafe {
            od.device
                .create_command_encoder(&hal::CommandEncoderDescriptor {
                    label: None,
                    queue: &od.queue,
                })
                .unwrap()
        };
        let mut fence = unsafe { od.device.create_fence().unwrap() };
        let rp_desc = hal::RenderPassDescriptor {
            label: None,
            extent: wgt::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            sample_count: 1,
            color_attachments: &[Some(hal::ColorAttachment {
                target: hal::Attachment {
                    view: &view,
                    usage: TextureUses::COLOR_TARGET,
                },
                resolve_target: None,
                ops: hal::AttachmentOps::STORE,
                clear_value: wgt::Color::BLUE,
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
            od.queue.submit(&[&cmd_buf], &[], (&mut fence, 0)).unwrap();
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
            if self.renderer.borrow().is_some() {
                return;
            }
            self.obj().attach_buffers();
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
