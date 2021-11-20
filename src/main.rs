mod decode;
mod chip8;
mod bits;

use chip8::{Chip8, Cycle, SCREEN_HEIGHT, SCREEN_WIDTH};
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, VirtualKeyCode};
use winit_input_helper::WinitInputHelper;

use env_logger;

fn load_rom(chip8: &mut Chip8) {
    let rom_path = std::env::args().nth(1).expect("No ROM given");
    let file = std::fs::File::open(rom_path).expect("Couldn't find ROM path given");
    chip8.read_program(file).expect("Failed to read ROM");

}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let (window, width, height, mut _hidpi_factor) = create_window("CHIP-8 Emulator", &event_loop);
    let surface_texture = SurfaceTexture::new(width, height, &window);
    let mut pixels = Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture).expect("Failed to start graphics library");
    let mut chip8 = Chip8::new();
    load_rom(&mut chip8);
    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            chip8.draw(pixels.get_frame());
            pixels.render().expect("Failed to render");
        }

        let mut key_pressed: Option<u8> = None;
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            
            // Adjust high DPI factor
            if let Some(factor) = input.scale_factor_changed() {
                _hidpi_factor = factor;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            let key2num = vec![
                (VirtualKeyCode::Key1, 1_u8),
                (VirtualKeyCode::Key2, 2),
                (VirtualKeyCode::Key3, 3),
                (VirtualKeyCode::Key4, 4),
                (VirtualKeyCode::Key5, 5),
                (VirtualKeyCode::Key6, 6),
                (VirtualKeyCode::Key7, 7),
                (VirtualKeyCode::Key8, 8),
                (VirtualKeyCode::Key9, 9),
                (VirtualKeyCode::Key0, 0),
                (VirtualKeyCode::A, 0xa),
                (VirtualKeyCode::B, 0xb),
                (VirtualKeyCode::C, 0xc),
                (VirtualKeyCode::D, 0xd),
                (VirtualKeyCode::E, 0xe),
                (VirtualKeyCode::F, 0xf),
            ];

            for (key, num) in key2num {
                if input.key_pressed(key) {
                    key_pressed = Some(num);
                }
            }
        }

        if let Cycle::RedrawRequested = chip8.cycle(key_pressed) {
            window.request_redraw();
        }
    });
}

/// Tuple of `(window, surface, width, height, hidpi_factor)`
/// `width` and `height` are in `PhysicalSize` units.
fn create_window(
    title: &str,
    event_loop: &EventLoop<()>,
) -> (winit::window::Window, u32, u32, f64) {
    // Create a hidden window so we can estimate a good default window size
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(event_loop)
        .unwrap();
    let hidpi_factor = window.scale_factor();

    // Get dimensions
    let width = SCREEN_WIDTH as f64;
    let height = SCREEN_HEIGHT as f64;
    let (monitor_width, monitor_height) = {
        if let Some(monitor) = window.current_monitor() {
            let size = monitor.size().to_logical(hidpi_factor);
            (size.width, size.height)
        } else {
            (width, height)
        }
    };
    let scale = (monitor_height / height * 2.0 / 3.0).round().max(1.0);

    // Resize, center, and display the window
    let min_size: winit::dpi::LogicalSize<f64> =
        PhysicalSize::new(width, height).to_logical(hidpi_factor);
    let default_size = LogicalSize::new(width * scale, height * scale);
    let center = LogicalPosition::new(
        (monitor_width - width * scale) / 2.0,
        (monitor_height - height * scale) / 2.0,
    );
    window.set_inner_size(default_size);
    window.set_min_inner_size(Some(min_size));
    window.set_outer_position(center);
    window.set_visible(true);

    let size = default_size.to_physical::<f64>(hidpi_factor);

    (
        window,
        size.width.round() as u32,
        size.height.round() as u32,
        hidpi_factor,
    )
}