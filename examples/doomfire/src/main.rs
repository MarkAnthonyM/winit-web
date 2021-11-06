use log::error;
use pixels::{PixelsBuilder, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use winit_web::WinitWeb;
use std::rc::Rc;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

struct World {
    
}

impl World {
    fn new() -> Self {
        Self {

        }
    }

    fn update(&mut self) {
        //Nothing here yet
    }

    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let rgba = [0xff, 0xff, 0xff, 0xff];
            pixel.copy_from_slice(&rgba);
        }
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        wasm_bindgen_futures::spawn_local(proto_run());
    }
}

async fn proto_run() {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Winit-Web Test")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .expect("WindowBuilder error")
    };

    let window = Rc::new(window);
    window.init_web();

    let mut input = WinitInputHelper::new();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
            .wgpu_backend(pixels::wgpu::Backends::all())
            .device_descriptor(pixels::wgpu::DeviceDescriptor {
                limits: pixels::wgpu::Limits::downlevel_webgl2_defaults(),
                ..Default::default()
            })
            .build_async()
            .await
            .expect("PixelsBuilder error")
    };

    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    })
}