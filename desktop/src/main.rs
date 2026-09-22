use chip8_core::*;
use sdl3::{event::Event, pixels::Color};
use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    const SCALE: u32 = 15;
    const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
    const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let color_offset = 0;

    'gameloop: loop {
        canvas.set_draw_color(Color::RGB(color_offset, 64, 255 - color_offset));
        canvas.clear();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'gameloop;
                }
                _ => {}
            }
        }
    }

    canvas.present();
}
