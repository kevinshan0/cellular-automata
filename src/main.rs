use nannou::prelude::*;
use nannou::rand;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    nannou::app(model)
        .update(update)
        .run();
}

struct Model {
    texture: wgpu::Texture,
    grid: Vec<u8>,
    buffer: Vec<u8>,
    rows: usize,
    cols: usize,
}

fn model(app: &App) -> Model {
    // 1. Create the window and attach the view function
    let window_id = app.new_window()
        .size(800, 800)
        .view(view)
        .build()
        .unwrap();

    let rows = 200;
    let cols = 200;

    // 2. Initialize grid with random values (0 or 255)
    let grid: Vec<u8> = (0..rows * cols)
        .map(|_| if rand::random::<bool>() { 255 } else { 0 })
        .collect();
    
    // Initialize the buffer with zeros (same size as grid)
    let buffer = vec![0; rows * cols];

    // 3. Create the GPU Texture
    let window = app.window(window_id).unwrap();
    let texture = wgpu::TextureBuilder::new()
        .size([cols as u32, rows as u32])
        .format(wgpu::TextureFormat::R8Unorm) // One channel (grayscale)
        .usage(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING)
        .build(window.device());

    Model { grid, buffer, rows, cols, texture }
}

fn count_neighbors_flat(grid: &Vec<u8>, x: usize, y: usize, rows: usize, cols: usize) -> u8 {
    let mut count = 0;
    for i in -1..=1 {
        for j in -1..=1 {
             // Skip the cell itself
            if i == 0 && j == 0 { continue; }

            // 1. Wrap coordinates (Toroidal logic)
            let ny = (y as isize + i + rows as isize) as usize % rows;
            let nx = (x as isize + j + cols as isize) as usize % cols;

            // 2. Convert 2D (x,y) to 1D index
            let idx = ny * cols + nx;

            // 3. Check if alive
            if grid[idx] == 255 {
                count += 1;
            }
        }
    }
    count
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // Update every frame
    if app.elapsed_frames() % 1 == 0 {
        
        // 1. Calculate next state into the buffer
        for y in 0..model.rows {
            for x in 0..model.cols {
                let idx = y * model.cols + x;
                
                let neighbors = count_neighbors_flat(&model.grid, x, y, model.rows, model.cols);
                let is_alive = model.grid[idx] == 255;
                
                let stays_alive = match (is_alive, neighbors) {
                    (true, 2) | (true, 3) => true,
                    (false, 3) => true,
                    _ => false,
                };
                
                model.buffer[idx] = if stays_alive { 255 } else { 0 };
            }
        }

        // 2. Swap the buffer and the grid
        std::mem::swap(&mut model.grid, &mut model.buffer);

        // 3. UPLOAD TO GPU
        let window = app.main_window();
        window.queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &model.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &model.grid,
            wgpu::ImageDataLayout {
                offset: 0,
                // R8Unorm is 1 byte per pixel, so bytes_per_row is just the width
                bytes_per_row: Some(model.cols as u32),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: model.cols as u32,
                height: model.rows as u32,
                depth_or_array_layers: 1,
            },
        );
    }
}

fn view(app: &App, model: &Model, frame: Frame){
    let draw = app.draw();
    draw.background().color(BLACK);

    // Draw the texture stretched to the window size
    // Using Nearest neighbor sampling avoids blurry pixels
    draw.texture(&model.texture)
        .wh(app.window_rect().wh());

    draw.to_frame(app, &frame).unwrap();
}