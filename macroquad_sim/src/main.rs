
use periodic_array_2d::PeriodicArray2D;
use monte_carlo::ising_state;
use monte_carlo::metropolis;

mod macroquad_rng;
use macroquad_rng::MacroquadRng;

use macroquad::{color::{self}, input::{self, KeyCode}, shapes, text, time::draw_fps, window};


const ROWS:    i32          = 256;     // Spin array has periodic boundary condition, I therefor use signed ints rather than usize
const COLUMNS: i32          = 256;
const INTERACTION_TERM: f32 = -1f32; // noted "J" in the App, as in all stat phys text books, but i don't like single char constants!

#[macroquad::main("2D Ising")]
async fn main() 
{
    let mut my_rng    = MacroquadRng::new();

    let thermal_state = ||{ising_state::thermal_state(&mut my_rng)};
    let spin_down_state  = ||{ising_state::SPINDOWN};
    let spin_up_state    = ||{ising_state::SPINUP};


    let mut spin_array = PeriodicArray2D::new_with(ROWS, COLUMNS, thermal_state)
        .expect("Rows & cols must be > 0.");
    
    let mut temp: f32          = 2.0;
    let delta_temp: f32        = 0.1;

    loop 
    {
        metropolis::perform_metropolis_sweep(&mut spin_array, &mut my_rng, temp, INTERACTION_TERM);
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

        window::clear_background(color::LIGHTGRAY);
            let box_size: f32  = window::screen_width().min(window::screen_height());
            let offset_x: f32  = (window::screen_width() - box_size) / 2. + 10.;
            let offset_y: f32  = (window::screen_height() - box_size) / 2. + 10.;
            let sq_size: f32   = (window::screen_height() - offset_y * 2.) / std::cmp::min(ROWS, COLUMNS) as f32;
        

            let mut magnetization: f32 = 0f32;
            for i in spin_array.rows_range() 
            {
                for j in spin_array.columns_range() 
                {
                    let spin_val= spin_array.at_unchecked(i, j); // correct range is ensured by the use of Ranges
                    let color = match spin_val 
                    {
                        ising_state::SPINUP   => color::DARKBLUE,
                        ising_state::SPINDOWN => color::DARKPURPLE,
                            _                 => color::WHITE
                    };

                    magnetization += spin_val;

                    shapes::draw_rectangle(
                        offset_x + j as f32 * sq_size,
                        offset_y + i as f32 * sq_size,
                        sq_size,
                        sq_size,
                        color);

                }
            }
            
            magnetization /= spin_array.total_number() as f32;

            let font_size: f32 = (0.015*window::screen_width()).max(15f32); 
            let text_y: f32    = 0.05*window::screen_height();
            let text_x: f32    = 0.01*window::screen_width();
             
            draw_fps();
            text::draw_text(&format!("temp: {:.1} (J/kB)", temp),             text_x, offset_y + 2.*text_y, font_size, color::BLACK);
            text::draw_text("[UP]: +0.1",                                     text_x, offset_y + 3.*text_y, font_size, color::BLACK);
            text::draw_text("[DOWN]: -0.1",                                   text_x, offset_y + 4.*text_y, font_size, color::BLACK);
            text::draw_text("[SPACE]: polarize up",                           text_x, offset_y + 5.*text_y, font_size, color::BLACK);
            text::draw_text("[BACKSPACE]: polarize down",                     text_x, offset_y + 6.*text_y, font_size, color::BLACK);
            text::draw_text(&format!("magnetization = {:.2}", magnetization), text_x, offset_y + 7.*text_y, font_size, color::RED);

        window::next_frame().await
    }

}
