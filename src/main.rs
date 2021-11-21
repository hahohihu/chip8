mod decode;
mod chip8;
mod bits;

use chip8::{Chip8, Cycle, SCREEN_HEIGHT, SCREEN_WIDTH};
use std::time::Instant;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, StartCause, VirtualKeyCode};
use winit_input_helper::WinitInputHelper;
use std::time::{Duration};
use env_logger;

fn load_rom(chip8: &mut Chip8) {
    let rom_path = std::env::args().nth(1).expect("No ROM given");
    let file = std::fs::File::open(rom_path).expect("Couldn't find ROM path given");
    chip8.read_program(file).expect("Failed to read ROM");
    chip8.print_program();
}

const KEY_MAPPING: [(VirtualKeyCode, u8); 16] = [
    (VirtualKeyCode::Key1, 1_u8),
    (VirtualKeyCode::Key2, 2),
    (VirtualKeyCode::Key3, 3),
    (VirtualKeyCode::Key4, 0xc),
    (VirtualKeyCode::Q, 4),
    (VirtualKeyCode::W, 5),
    (VirtualKeyCode::E, 6),
    (VirtualKeyCode::R, 0xd),
    (VirtualKeyCode::A, 7),
    (VirtualKeyCode::S, 8),
    (VirtualKeyCode::D, 9),
    (VirtualKeyCode::F, 0xe),
    (VirtualKeyCode::Z, 0xa),
    (VirtualKeyCode::X, 0),
    (VirtualKeyCode::C, 0xb),
    (VirtualKeyCode::V, 0xf),
];

fn main() {
    env_logger::builder().format_timestamp(None).init();
    let mut time = Instant::now();
    let mut chip8 = Chip8::new(time);
    load_rom(&mut chip8);
    chip8.print_program();
    let clock_speed: u32 = 1000000; // TODO: make configurable
    let clock_gap: Duration = Duration::from_secs_f32(1.0) / clock_speed;
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let (window, width, height, mut _hidpi_factor) = create_window("CHIP-8 Emulator", &event_loop);
    let surface_texture = SurfaceTexture::new(width, height, &window);
    let mut pixels = Pixels::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, surface_texture).expect("Failed to start graphics library");
    println!("Starting CHIP-8 emulator");

    let mut key_pressed: Option<u8> = None;
    event_loop.run(move |event, _, control_flow| {
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if let Some(factor) = input.scale_factor_changed() {
                _hidpi_factor = factor;
            }
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            for (key, num) in KEY_MAPPING {
                if input.key_pressed(key) {
                    key_pressed = Some(num);
                }
                if input.key_released(key) {
                    key_pressed = None;
                }
            }
        }

        match event {
            Event::RedrawRequested(_) => {
                chip8.draw(pixels.get_frame());
                pixels.render().expect("Failed to render");
            },
            Event::NewEvents(StartCause::Init) => {
                *control_flow = ControlFlow::WaitUntil(time + clock_gap);
            },
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                if let Cycle::RedrawRequested = chip8.cycle(key_pressed, time) {
                    *control_flow = ControlFlow::WaitUntil(time + clock_gap);
                    window.request_redraw();
                }
                time += clock_gap;
            },
            _ => {}
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