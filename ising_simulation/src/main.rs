use macroquad::prelude::*;
use ising_simulation::Simulation;
#[macroquad::main("2d ising")]
async fn main() 
{
    let mut my_simulation = Simulation::new();
    loop 
    {
        my_simulation.update();
        my_simulation.draw_and_handle_ui();
        next_frame().await;
    }
}