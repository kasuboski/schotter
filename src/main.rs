use nannou::prelude::*;

use std::fs;
use std::io::ErrorKind;
use std::process::exit;

use log::debug;

const ROWS: u32 = 22;
const COLS: u32 = 12;
const SIZE: u32 = 30;
const MARGIN: u32 = 35;
const WIDTH: u32 = COLS * SIZE + 2 * MARGIN;
const HEIGHT: u32 = ROWS * SIZE + 2 * MARGIN;
const LINE_WIDTH: f32 = 0.06;

const SECONDS: usize = 30;
const FRAMES: usize = 60 * SECONDS;

fn main() {
    env_logger::init();
    debug!("starting...");
    // run for frames + 1; +1 for time to exit
    nannou::app(model)
        .update(update)
        .loop_mode(LoopMode::loop_ntimes(FRAMES + 1))
        .run();
}

#[derive(Debug)]
struct Stone {
    x: f32,
    y: f32,
    x_offset: f32,
    y_offset: f32,
    rotation: f32,
    x_velocity: f32,
    y_velocity: f32,
    rot_velocity: f32,
    cycles: u32,
}

impl Stone {
    fn new(x: f32, y: f32) -> Self {
        let x_offset = 0.0;
        let y_offset = 0.0;
        let rotation = 0.0;
        let x_velocity = 0.0;
        let y_velocity = 0.0;
        let rot_velocity = 0.0;
        let cycles = 0;
        Stone {
            x,
            y,
            x_offset,
            y_offset,
            rotation,
            x_velocity,
            y_velocity,
            rot_velocity,
            cycles,
        }
    }
}

struct Model {
    main_window: WindowId,

    frames_dir: String,
    cur_frame: u32,
    recording: bool,

    motion: f32,
    disp_adj: f32,
    rot_adj: f32,
    gravel: Vec<Stone>,
}

fn model(app: &App) -> Model {
    let main_window = app
        .new_window()
        .title(app.exe_name().expect("No exe name"))
        .size(WIDTH, HEIGHT)
        .view(view)
        .key_pressed(key_pressed)
        .build()
        .expect("Couldn't build window");

    let motion = 1.0;
    let disp_adj = 1.0;
    let rot_adj = 1.0;

    let mut gravel = Vec::new();
    for y in 0..ROWS {
        for x in 0..COLS {
            let stone = Stone::new(x as f32, y as f32);
            gravel.push(stone);
        }
    }

    let frames_dir = app.exe_name().expect("couldn't get app name") + "_frames";
    let recording = true;
    let cur_frame = 0;

    Model {
        main_window,
        frames_dir,
        recording,
        cur_frame,
        motion,
        disp_adj,
        rot_adj,
        gravel,
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let gdraw = draw
        .scale(SIZE as f32)
        .scale_y(-1.0)
        .x_y(COLS as f32 / -2.0 + 0.5, ROWS as f32 / -2.0 + 0.5);

    for stone in &model.gravel {
        let cdraw = gdraw.x_y(stone.x, stone.y);
        let basis = abs_normalize(stone.x, COLS as f32) + abs_normalize(stone.y, ROWS as f32) + abs_normalize(stone.rotation, PI / 4.0) + abs_normalize(stone.x_offset, 0.5) + abs_normalize(stone.y_offset, 0.5);
        let hue = basis / 5.0;

        // debug!("basis: {}, hue: {}", basis, hue);
        
        let stroke_color = nannou::color::hsl(hue, 1.0, 0.5);
        cdraw
            .rect()
            .no_fill()
            .stroke(stroke_color)
            .stroke_weight(LINE_WIDTH)
            .w_h(1.0, 1.0)
            .x_y(stone.x_offset, stone.y_offset)
            .rotate(stone.rotation);
    }

    draw.background().color(BLACK);

    draw.to_frame(app, &frame).unwrap();
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let elapsed_frames = app.elapsed_frames();

    if elapsed_frames >= FRAMES as u64 / 2 {
        model.disp_adj = 0.0;
        model.rot_adj = 0.0;
    }

    for stone in &mut model.gravel {
        if stone.cycles == 0 {
            if random_f32() > model.motion {
                stone.x_velocity = 0.0;
                stone.y_velocity = 0.0;
                stone.rot_velocity = 0.0;
                stone.cycles = random_range(50, 300);
                continue;
            }
            let factor = stone.y / ROWS as f32;
            let disp_factor = factor * model.disp_adj;
            let rot_factor = factor * model.rot_adj;

            let new_x = disp_factor * random_range(-0.5, 0.5);
            let new_y = disp_factor * random_range(-0.5, 0.5);
            let new_rot = rot_factor * random_range(-PI / 4.0, PI / 4.0);
            let new_cycles = random_range(50, 300);

            stone.x_velocity = (new_x - stone.x_offset) / new_cycles as f32;
            stone.y_velocity = (new_y - stone.y_offset) / new_cycles as f32;
            stone.rot_velocity = (new_rot - stone.rotation) / new_cycles as f32;
            stone.cycles = new_cycles;
        } else {
            stone.x_offset += stone.x_velocity;
            stone.y_offset += stone.y_velocity;
            stone.rotation += stone.rot_velocity;
            stone.cycles -= 1;
        }
    }

    if model.recording && elapsed_frames % 2 == 0 {
        model.cur_frame += 1;
        if model.cur_frame > 9999 {
            model.recording = false;
        } else {
            let filename = format!("{}/shotter{:>04}.png",
                model.frames_dir,
                model.cur_frame);
            match app.window(model.main_window) {
                Some(window) => {
                    window.capture_frame(filename);
                }
                None => {}
            }
        }
    }

    if elapsed_frames >= FRAMES.try_into().expect("frames can't be u64") {
        exit(0);
    }
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    match key {
        Key::R => {
            if model.recording {
                model.recording = false;
            } else {
                fs::create_dir(&model.frames_dir).unwrap_or_else(|error| {
                    if error.kind() != ErrorKind::AlreadyExists {
                        panic!("Problem creating director {:?}", model.frames_dir);
                    }
                });
                model.recording = true;
                model.cur_frame = 0;
            }
        }
        Key::S => match app.window(model.main_window) {
            Some(window) => {
                let app_name = app.exe_name().expect("couldn't get app name");
                window.capture_frame(app_name + ".png")
            }
            None => {}
        },
        Key::Up => {
            model.disp_adj += 0.1;
        }
        Key::Down => {
            if model.disp_adj > 0.0 {
                model.disp_adj -= 0.1;
            }
        }
        Key::Right => {
            model.rot_adj += 0.1;
        }
        Key::Left => {
            if model.rot_adj > 0.0 {
                model.rot_adj -= 0.1;
            }
        }
        _ => {}
    }
}

fn abs_normalize(orig: f32, max: f32) -> f32 {
    orig.abs() / max
}
