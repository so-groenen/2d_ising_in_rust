
use periodic_array_2d::{PeriodicArray2D, ArrayRngInterface};

// The monte carlo functions should NOT depend on specific RNGs, it should be flexible enough that the user can chooose which RNG to use.
// therefore we make it a Trait (or interface if you have a OOP background)
pub trait MonteCarloRngInterface
{
    fn generate_rand_f32(&mut self, low: f32, high: f32) -> f32;
}

pub mod ising_state
{
    use super::*;
    pub const SPINUP: f32   = 1f32;
    pub const SPINDOWN: f32 = -1f32;
    pub fn thermal_state<T>(rng: &mut T) -> f32 where T: MonteCarloRngInterface
    {
        1f32 - 2f32*rng.generate_rand_f32(0f32, 1f32).round()
    }
}
 
pub mod metropolis
{
    use super::*;
    pub const MAX_BETA: f32 = 1E6;
    fn _get_delta_energy(spin_2d_arr: &PeriodicArray2D, i: i32, j: i32, interaction_term: f32) -> f32
    {
        let left_right: f32 = spin_2d_arr.at_unchecked(i-1, j) + spin_2d_arr.at_unchecked(i+1, j);
        let up_down: f32    = spin_2d_arr.at_unchecked(i, j+1) + spin_2d_arr.at_unchecked(i, j-1);
        
        -2.*interaction_term*spin_2d_arr.at_unchecked(i,j)*(left_right + up_down)
    }
    fn _accept_state<R>(temp: f32, delta_energy: f32, rng: &mut R) -> bool where R: MonteCarloRngInterface + ArrayRngInterface
    {
        let beta: f32 = if temp > 0. {1./temp} else {MAX_BETA};

        delta_energy < 0. || (rng.generate_rand_f32(0f32, 1f32) < (-beta*delta_energy).exp())
    }

    pub fn perform_metropolis_sweep<R>(spin_2d_arr: &mut PeriodicArray2D, rng: &mut R, temp: f32, interaction_term: f32)
        where R: MonteCarloRngInterface + ArrayRngInterface
    {
        for _ in 0..spin_2d_arr.total_number()
        {
            let (i_rand, j_rand) = spin_2d_arr.get_random_point(rng);
            let delta_energy          = _get_delta_energy(spin_2d_arr, i_rand, j_rand, interaction_term);
            if _accept_state(temp, delta_energy, rng) 
            {
                *spin_2d_arr.at_mut_unchecked(i_rand, j_rand) *= -1.;
            }
        }
    }
    pub fn perform_metropolis_proposal<R>(spin_2d_arr: &mut PeriodicArray2D, rng: &mut R, temp: f32, interaction_term: f32) -> f32
        where R: MonteCarloRngInterface + ArrayRngInterface
    {
        let (i_rand, j_rand) = spin_2d_arr.get_random_point(rng);
        let delta_energy          = _get_delta_energy(spin_2d_arr, i_rand, j_rand, interaction_term);
        let mut delta_spin        = 0f32;
        if _accept_state(temp, delta_energy, rng) 
        {
            let s: &mut f32 = spin_2d_arr.at_mut_unchecked(i_rand, j_rand);
            // *spin_2d_arr.at_mut_unchecked(i_rand, j_rand) *= -1.;
            (*s) *= -1.;
            delta_spin = 2.*(*s);
        }
        delta_spin
    }
}