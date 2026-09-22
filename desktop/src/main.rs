use chip8_core::*;
use sdl3::rect::Rect;
use sdl3::{VideoSubsystem, event::Event, pixels::Color, render::Canvas, video::Window};
use std::env;
use std::fs::File;
use std::io::Read;

const SCALE: u32 = 15;
const TICKS_PER_FRAME: usize = 10;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let color_offset = 0;

    let mut canvas = create_canvas(&video_subsystem);
    let mut chip8 = Emu::new();
    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

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

        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }

        chip8.tick_timers();
        draw_screen(&chip8, &mut canvas);

        canvas.present();
    }
}

fn draw_screen(emu: &Emu, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    let screen_buf = emu.get_display();
    canvas.set_draw_color(Color::WHITE);
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;
            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
}

fn create_canvas(video_subsystem: &VideoSubsystem) -> Canvas<Window> {
    const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
    const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let canvas = window.into_canvas();

    canvas
}
