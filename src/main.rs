use nannou::prelude::*;
use nannou::rand;

fn main() {
    nannou::app(model)
        .update(update)
        .run();
}

// --- CONFIGURATION ---
const PALETTE: &[[u8; 4]] = &[
    [0, 0, 0, 255],       // State 0: Dead (Black)
    [255, 65, 54, 255],   // State 1: Red
    [46, 204, 64, 255],   // State 2: Green
    [0, 116, 217, 255],   // State 3: Blue
    [255, 220, 0, 255],   // State 4: Yellow
];

fn solve_cell(current_state: u8, neighbors: &[u8]) -> u8 {
    // Count specific neighbor types
    let mut counts = [0u8; 5];
    for &n in neighbors {
        if (n as usize) < counts.len() {
            counts[n as usize] += 1;
        }
    }

    // 0 = Empty, 1 = Red, 2 = Green, 3 = Blue
    match current_state {
        0 => {
             // 2 Reds spawn a Red
            if counts[1] == 2 { return 1; }
             // 2 Greens spawn a Green
            if counts[2] == 2 { return 2; }
             // 2 Blues spawn a Blue
            if counts[3] == 2 { return 3; }
            0
        },
        1 => {
            // Red dies if surrounded by too many blues
            if counts[2] >= 2 { return 0; }
            // Red dies if surrounded by too many reds
            if counts[1] >= 2 { return 0; }
            1
        },
        2 => {
            // Green dies if surrounded by too many reds
            if counts[3] >= 2 { return 0; }
            // Green dies if surrounded by too many greens
            if counts[2] >= 2 { return 0; }
            2
        },
        3 => {
            // Blue dies if surrounded by too many greens
            if counts[4] >= 2 { return 0; }
            // Green dies if surrounded by too many greens
            if counts[3] >= 2 { return 0; }
            3
        },
        4 => {
            // Blue dies if surrounded by too many greens
            if counts[1] >= 2 { return 0; }
            // Green dies if surrounded by too many greens
            if counts[4] >= 2 { return 0; }
            4
        },
        _ => 0,
    }
}

struct Model {
    texture: wgpu::Texture,
    state_grid: Vec<u8>,
    next_state: Vec<u8>,
    texture_data: Vec<u8>,
    rows: usize,
    cols: usize,
}

fn model(app: &App) -> Model {
    let window_id = app.new_window().size(800, 800).view(view).build().unwrap();
    let window = app.window(window_id).unwrap();

    let rows = 100;
    let cols = 100;

    // Initialize random states (0 to 4)
    let state_grid: Vec<u8> = (0..rows * cols)
        .map(|_| if rand::random::<f32>() > 0.8 { 
            rand::random_range(1, 5)
        } else { 
            0 
        })
        .collect();
    
    let next_state = vec![0; rows * cols];
    let texture_data = vec![0; rows * cols * 4];

    let texture = wgpu::TextureBuilder::new()
        .size([cols as u32, rows as u32])
        .format(wgpu::TextureFormat::Rgba8Unorm)
        .usage(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING)
        .build(window.device());

    Model { 
        texture, 
        state_grid, 
        next_state, 
        texture_data, 
        rows, 
        cols 
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if app.elapsed_frames() % 10 == 0 {
        for y in 0..model.rows {
            for x in 0..model.cols {
                let idx = y * model.cols + x;
                
                // 1. Get Neighbors
                let neighbors = get_neighbor_states(&model.state_grid, x, y, model.rows, model.cols);
                
                // 2. Run Configurable Rules
                let new_state = solve_cell(model.state_grid[idx], &neighbors);
                model.next_state[idx] = new_state;

                // 3. Update Visuals (Map State ID -> Color)
                let color_idx = (new_state as usize).min(PALETTE.len() - 1);
                let color = PALETTE[color_idx];

                let tex_idx = idx * 4;
                model.texture_data[tex_idx] = color[0];     // R
                model.texture_data[tex_idx + 1] = color[1]; // G
                model.texture_data[tex_idx + 2] = color[2]; // B
                model.texture_data[tex_idx + 3] = color[3]; // A
            }
        }

        // Swap the logic grids
        std::mem::swap(&mut model.state_grid, &mut model.next_state);

        // Upload to GPU
        let window = app.main_window();
        window.queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &model.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &model.texture_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some((model.cols * 4) as u32),
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

fn get_neighbor_states(grid: &Vec<u8>, x: usize, y: usize, rows: usize, cols: usize) -> [u8; 8] {
    let mut neighbors = [0u8; 8];
    let mut idx = 0;
    
    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 { continue; }

            let ny = (y as isize + i + rows as isize) as usize % rows;
            let nx = (x as isize + j + cols as isize) as usize % cols;
            
            neighbors[idx] = grid[ny * cols + nx];
            idx += 1;
        }
    }
    neighbors
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK); // Clear the screen to black

    // Calculate the size of each cell
    let win = app.window_rect();
    let cell_w = win.w() / model.cols as f32;
    let cell_h = win.h() / model.rows as f32;

    // Center the grid: Nannou (0,0) is the center of the screen
    let half_w = win.w() / 2.0;
    let half_h = win.h() / 2.0;

    for y in 0..model.rows {
        for x in 0..model.cols {
            let idx = y * model.cols + x;
            let state = model.state_grid[idx];

            if state != 0 {
                let color_idx = (state as usize).min(PALETTE.len() - 1);
                let rgb = PALETTE[color_idx];
                let color = rgba(
                    rgb[0] as f32 / 255.0, 
                    rgb[1] as f32 / 255.0, 
                    rgb[2] as f32 / 255.0, 
                    rgb[3] as f32 / 255.0
                );

                let px = (x as f32 * cell_w) - half_w + (cell_w / 2.0);
                let py = half_h - (y as f32 * cell_h) - (cell_h / 2.0);

                draw.rect()
                    .x_y(px, py)
                    .w_h(cell_w, cell_h)
                    .color(color);
            }
        }
    }

    draw.to_frame(app, &frame).unwrap();
}