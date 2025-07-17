
mod spin_2d;
use macroquad::{camera, color, input::{self, KeyCode}, math, text, texture, time::draw_fps, window};
use spin_2d::{Spin2D, ising_state, SpinRNG, SpinMacroquadRng};


// The Spin2D methods (Monte carlo methods etc) should be independant of the random number generator
// For this, we define the "SpinRNG" trait with "generate_rand_f32", "generate_rand_i32" & "new" method.
// They act as "high-level" wrapper for the spin2D functions.
// SpinMacroquadRng is a "high-level" wrapper/interface around macroquad::rand::RandGenerator. 

const ROWS:     i32      = 256;
const COLUMNS:  i32      = 256;
const SQR_SIZE: i32      = 2;   // size of a square depicting the spin on the render_target
const RENDER_WIDTH: f32  = SQR_SIZE as f32*ROWS as f32;
const RENDER_HEIGHT: f32 = SQR_SIZE as f32*COLUMNS as f32;

#[macroquad::main("2D Ising")]
async fn main() 
{
    let render_target          = texture::render_target(RENDER_WIDTH as u32, RENDER_HEIGHT as u32);

    let spin_up_img: texture::Image           = texture::Image::gen_image_color(SQR_SIZE as u16, SQR_SIZE as u16, color::DARKBLUE);
    let spin_down_img: texture::Image         = texture::Image::gen_image_color(SQR_SIZE as u16, SQR_SIZE as u16, color::DARKPURPLE);
    let spin_error_img: texture::Image        = texture::Image::gen_image_color(SQR_SIZE as u16, SQR_SIZE as u16, color::WHITE);

    let spin_up_texture: texture::Texture2D    = texture::Texture2D::from_image(&spin_up_img);
    let spin_down_texture: texture::Texture2D  = texture::Texture2D::from_image(&spin_down_img);
    let spin_error_texture: texture::Texture2D = texture::Texture2D::from_image(&spin_error_img);
    texture::build_textures_atlas();

    let mut render_target_cam =
        camera::Camera2D::from_display_rect(math::Rect::new(0., 0., RENDER_WIDTH, RENDER_HEIGHT));
        render_target_cam.render_target = Some(render_target.clone());


    let mut my_rng       = SpinMacroquadRng::new();

    let thermal_state = ||{ising_state::thermal_state(&mut my_rng)};
    let spin_down_state  = ||{ising_state::SPINDOWN};
    let spin_up_state    = ||{ising_state::SPINUP};


    let mut spin_array: Spin2D = Spin2D::new_with(ROWS, COLUMNS, thermal_state).expect("Rows & cols must be > 0.");
    let mut temp: f32          = 2.0;
    let delta_temp: f32        = 0.1;
    let interaction_term: f32  = -1.0; // "noted J" in the App, as in all stat phys text books!

    loop 
    {
        spin_array.perform_monte_carlo_sweep(temp, interaction_term, &mut my_rng); 

        if input::is_key_pressed(KeyCode::Up) 
        {
            temp += delta_temp;
            temp = ((temp*10.).round())/10.    
        }
        if input::is_key_pressed(KeyCode::Down) && temp > 0.
        {
            temp -= delta_temp;
            temp = ((temp*10.).round())/10.    
        }
        if input::is_key_pressed(KeyCode::Space)
        {
            spin_array.reset(spin_up_state);
        }
        if input::is_key_pressed(KeyCode::Backspace)
        {
            spin_array.reset(spin_down_state);
        }

        let scale: f32 = f32::min(
                window::screen_width() / RENDER_WIDTH,
                window::screen_height() / RENDER_HEIGHT,
            );

            
        let mut mag_avg = 0f32;

        camera::set_camera(&render_target_cam);
            window::clear_background(color::LIGHTGRAY);

            for i in spin_array.rows_range() 
            {
                for j in spin_array.columns_range() 
                {
                    let spin_val: f32 = spin_array.at_unchecked(i, j); // correct range is ensured by the use of Ranges
                    let x_pixel: f32  = SQR_SIZE as f32 * j as f32;
                    let y_pixel: f32  = SQR_SIZE as f32 * i as f32;
                    match spin_val
                    {
                        ising_state::SPINUP   => texture::draw_texture(&spin_up_texture, x_pixel, y_pixel, color::WHITE),
                        ising_state::SPINDOWN => texture::draw_texture(&spin_down_texture, x_pixel, y_pixel, color::WHITE),
                        _                     => texture::draw_texture(&spin_error_texture, x_pixel, y_pixel, color::WHITE)
                    }
                    mag_avg += spin_val;
                }
            }

        mag_avg /= spin_array.total_number() as f32;

        camera::set_default_camera();
            window::clear_background(color::LIGHTGRAY);

            texture::draw_texture_ex(
                &render_target.texture,
                (window::screen_width() - (RENDER_WIDTH*scale))*0.5,
                (window::screen_height() - (RENDER_HEIGHT*scale))*0.5,
                color::WHITE,
                texture::DrawTextureParams 
                {
                    dest_size: Some(math::vec2(RENDER_WIDTH * scale, RENDER_HEIGHT * scale)),
                    flip_y: true, // Must flip y otherwise 'render_target' will be upside down
                    ..Default::default()
                },
            );
            
            draw_fps();
            let font_size: f32 = (0.015*window::screen_width()).max(15f32); 
            let text_y: f32    = 0.05*window::screen_height();
            let text_x: f32    = 0.01*window::screen_width();
            let box_size: f32  = window::screen_width().min(window::screen_height());
            let offset_y: f32  = (window::screen_height() - box_size) / 2. + 10.;
        

            text::draw_text(&format!("temp: {:.1} (J/kB)", temp),       text_x, offset_y + 2.*text_y, font_size, color::BLACK);
            text::draw_text("[UP]: +0.1",                               text_x, offset_y + 3.*text_y, font_size, color::BLACK);
            text::draw_text("[DOWN]: -0.1",                             text_x, offset_y + 4.*text_y, font_size, color::BLACK);
            text::draw_text("[SPACE]: polarize up",                     text_x, offset_y + 5.*text_y, font_size, color::BLACK);
            text::draw_text("[BACKSPACE]: polarize down",               text_x, offset_y + 6.*text_y, font_size, color::BLACK);
            text::draw_text(&format!("magnetization = {:.2}", mag_avg), text_x, offset_y + 7.*text_y, font_size, color::RED);

        window::next_frame().await
    }

}
