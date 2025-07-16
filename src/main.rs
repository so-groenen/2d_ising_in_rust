
mod spin_2d;
use macroquad::{color, input::{self, KeyCode}, shapes, text, time::draw_fps, window};
use spin_2d::{Spin2D, ising_state};


#[macroquad::main("2D Ising")]
async fn main() 
{
    let rows:    i32 = 300; // Spin array has periodic boundary condition: can use negative indices etc
    let columns: i32 = 300;

    let mut my_rng = macroquad::rand::RandGenerator::new();

    let thermal_state = ||{ising_state::thermal_state(&mut my_rng)};
    let spin_down_state  = ||{ising_state::SPINDOWN};
    let spin_up_state    = ||{ising_state::SPINUP};


    let mut spin_array: Spin2D = Spin2D::new_with(rows, columns, thermal_state).expect("Rows & cols must be > 0.");
    let mut temp: f32          = 2.0;
    let delta_temp: f32        = 0.1;
    let interaction_term: f32  = -1.0; // "noted J" in the App, as in all stat phys text books!
    let mut avg_mag: f32       = 0.0;

    loop 
    {
        spin_array.perform_monte_carlo_sweep(temp, interaction_term, &mut my_rng);
        avg_mag = spin_array.get_average_magnetization();

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

            draw_fps();
            let font_size: f32 = (0.015*window::screen_width()).max(15f32); 
            let text_y: f32    = 0.05*window::screen_height();
            let text_x: f32    = 0.01*window::screen_width();
            let game_size: f32 = window::screen_width().min(window::screen_height());
            let offset_x: f32  = (window::screen_width() - game_size) / 2. + 10.;
            let offset_y: f32  = (window::screen_height() - game_size) / 2. + 10.;
            let sq_size: f32   = (window::screen_height() - offset_y * 2.) / std::cmp::min(spin_array.rows(), spin_array.columns()) as f32;

            
            text::draw_text(&format!("temp: {:.1}(J/kB)", temp), text_x, offset_y + 2.*text_y, font_size, color::BLACK);
            text::draw_text("[UP]: +0.1", text_x, offset_y + 3.*text_y, font_size, color::BLACK);
            text::draw_text("[DOWN]: -0.1", text_x, offset_y + 4.*text_y, font_size, color::BLACK);
            text::draw_text("[SPACE]: polarize up", text_x, offset_y + 5.*text_y, font_size, color::BLACK);
            text::draw_text("[BACKSPACE]: polarize down", text_x, offset_y + 6.*text_y, font_size, color::BLACK);
            text::draw_text(&format!("magnetization = {:.2}", avg_mag), text_x, offset_y + 7.*text_y, font_size, color::RED);


            for i in spin_array.rows_range() 
            {
                for j in spin_array.columns_range() 
                {
                    let color = match spin_array.at_unchecked(i, j) // correct range is ensured by the use of Ranges
                    {
                        ising_state::SPINUP   => color::DARKBLUE,
                        ising_state::SPINDOWN => color::DARKPURPLE,
                            _                 => color::WHITE
                    };

                    shapes::draw_rectangle(
                        offset_x + j as f32 * sq_size,
                        offset_y + i as f32 * sq_size,
                        sq_size,
                        sq_size,
                        color);
                }
            }
    

        window::next_frame().await
    }

}
