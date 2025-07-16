
mod spin_2d;
use macroquad::{color, input::{self, KeyCode}, prelude::Conf, shapes, text, time::draw_fps, window};
// use rand::rngs::ThreadRng;
use spin_2d::{Spin2D, ising_state};

// fn window_conf() -> Conf
// {
//     Conf{window_title: "2D Ising: Metropolis algorithm".to_owned(),
//         window_width: 1024,
//         window_height: 768,
//         ..Default::default()}
// }

#[macroquad::main("window_conf")]
async fn main() 
{
    let rows:    i32 = 300;
    let columns: i32 = 300;

    // let mut my_rng: ThreadRng  = rand::rng(); 
    let mut my_rng = macroquad::rand::RandGenerator::new();

    let initial_state_generator = ||{ising_state::thermal_state(&mut my_rng)};
    // Alternatively use one of either as initial state:
        // let spin_down = ||{ising_state::SPINDOWN};
        // let spin_up   = ||{ising_state::SPINUP};

    let mut spin_array: Spin2D = Spin2D::new_with(rows, columns, initial_state_generator).expect("Rows & cols must be > 0.");
    let mut temp: f32          = 2.0;
    let delta_temp: f32        = 0.1;
    let interaction_term: f32  = -1.0; // "noted J" in the App, as in all stat phys text books!
    let font_size: f32         = 50f32;

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

        window::clear_background(color::LIGHTGRAY);

            draw_fps();
            let game_size = window::screen_width().min(window::screen_height());
            let offset_x  = (window::screen_width() - game_size) / 2. + 10.;
            let offset_y  = (window::screen_height() - game_size) / 2. + 10.;
            let sq_size   = (window::screen_height() - offset_y * 2.) / std::cmp::min(spin_array.rows(), spin_array.columns()) as f32;
            let text_y    = 0.05*window::screen_height();
            let text_x    = 0.025*window::screen_width();
            
            text::draw_text(&format!("temp: {:.1}(J/kB)", temp), text_x, offset_y + 2.*text_y, font_size, color::BLACK);
            text::draw_text("UP:  +0.1", text_x, offset_y + 3.*text_y, font_size, color::BLACK);
            text::draw_text("DOWN: -0.1", text_x, offset_y + 4.*text_y, font_size, color::BLACK);


            shapes::draw_rectangle(offset_x, offset_y, game_size - 20., game_size - 20., color::WHITE);


            for i in spin_array.rows_range() 
            {
                for j in spin_array.columns_range() 
                {
                    let color = match spin_array.at_unchecked(i, j) // correct range is ensured by the use of iterators
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
