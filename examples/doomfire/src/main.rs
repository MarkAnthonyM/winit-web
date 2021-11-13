use log::error;
use pixels::{PixelsBuilder, SurfaceTexture};
use rand::prelude::*;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use winit_web::WinitWeb;
use std::rc::Rc;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

struct Scene {
    scene_buffer: Vec<usize>,
    scene_palette: Vec<(u8, u8, u8, u8)>,
}

impl Scene {
    fn new() -> Self {
        let scene_palette = vec![
            (7, 7, 7, 1),
            (31, 7, 7, 1),
            (47, 15, 7, 1),
            (71, 15, 7, 1),
            (87, 23, 7, 1),
            (103, 31, 7, 1),
            (119, 31, 7, 1),
            (143, 39, 7, 1),
            (159, 47, 7, 1),
            (175, 63, 7, 1),
            (191, 71, 7, 1),
            (199, 71, 7, 1),
            (223, 79, 7, 1),
            (223, 87, 7, 1),
            (223, 87, 7, 1),
            (215, 95, 7, 1),
            (215, 103, 15, 1),
            (207, 111, 15, 1),
            (207, 119, 15, 1),
            (207, 127, 15, 1),
            (207, 135, 23, 1),
            (199, 135, 23, 1),
            (199, 143, 23, 1),
            (199, 151, 23, 1),
            (191, 159, 31, 1),    
            (191, 159, 31, 1),
            (191, 167, 39, 1),
            (191, 167, 39, 1),
            (191, 175, 47, 1),
            (183, 175, 47, 1),
            (183, 183, 47, 1),
            (183, 183, 55, 1),
            (207, 207, 111, 1),
            (223, 223, 159, 1),
            (239, 239, 199, 1),
            (255, 255, 255, 1)
        ];

        // Initialize doom scene
        let mut scene_buffer = vec![0; (WIDTH * HEIGHT) as usize];

        // Setup fire igniter row
        for x in 0..WIDTH {
            let y = HEIGHT - 1;
            let pixel = y * WIDTH + x;
            scene_buffer[pixel as usize] = scene_palette.len() - 1;
        }
        
        Self {
            scene_buffer,
            scene_palette,
        }
    }

    fn update(&mut self) {
        //Nothing here yet
    }

    fn draw(&mut self, frame: &mut [u8]) {
        self.propagate_fire();
        
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let color = self.scene_buffer[i];
            let swatch = self.convert_swatch(self.scene_palette[color]);

            let rgba = swatch;
            
            pixel.copy_from_slice(&rgba);
        }
    }

    // Convenience method, converts tuple to array 
    fn convert_swatch(&self, swatch: (u8, u8, u8, u8)) -> [u8; 4] {
        [swatch.0, swatch.1, swatch.2, swatch.3]
    }

    // Iterate through scene buffer. Spread fire from
    // bottom-most row, upwards
    fn propagate_fire(&mut self) {
        let mut rng = thread_rng();
        
        for y in 5..HEIGHT {
            for x in 0..WIDTH {
                let src = (y * WIDTH + x) as usize;
                let pixel = self.scene_buffer[src];

                if pixel == 0 {
                    self.scene_buffer[src - WIDTH as usize] = pixel;
                } else {
                    let ran_num: usize = rng.gen_range(0..2);
                    self.scene_buffer[src - WIDTH as usize] = pixel - ran_num;
                }
            }
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

    let mut scene = Scene::new();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            scene.draw(pixels.get_frame());
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
            scene.update();
            window.request_redraw();
        }
    })
}