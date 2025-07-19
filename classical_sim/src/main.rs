
use periodic_array_2d::{PeriodicArray2D};
use monte_carlo::{ising_state, metropolis};

mod os_based_rng;
use crate::os_based_rng::OsBasedRng;



const ROWS:    i32          = 256;     // Spin array has periodic boundary condition, I therefor use signed ints rather than usize
const COLUMNS: i32          = 256;
const INTERACTION_TERM: f32 = -1f32; // noted "J" in the App, as in all stat phys text books, but i don't like single char constants!

fn main() 
{
    let mut my_rng          = OsBasedRng::new();

    let thermal_state = ||{ising_state::thermal_state(&mut my_rng)};
    let mut spin_array = PeriodicArray2D::new_with(ROWS, COLUMNS, thermal_state)
        .expect("Rows & cols must be > 0.");
    spin_array.sum();
    println!("TODO!!");
}
