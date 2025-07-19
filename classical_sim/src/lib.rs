use periodic_array_2d::{PeriodicArray2D, PeriodicArrayError};
use monte_carlo::{ising_state, metropolis};

mod os_based_rng;
use crate::os_based_rng::OsBasedRng;

#[derive(Debug)]
pub enum CalculationError
{
    NegativeTempError,
    ArrayInitError(PeriodicArrayError),
}
pub struct ExperimentParam
{
    pub temperatures: Vec<f32>,
    pub interaction_term: f32,
    pub steps_between_measures: usize,  
    pub thermalisation_steps: usize,     // = 
    pub measurement_steps: usize             // = N: Everytime we do a full sweep we take the measure
}

pub fn create_temp_vector(start: f32, stop: f32, step: f32) -> Vec<f32>
{
    let mut range: Vec<f32> = Vec::new();
    if step <= 0f32
    {
        return range;
    }
    let needed_capacity = ((stop - start).abs() / step) as usize;
    range.reserve(needed_capacity);

    let mut current = start;
    let sign = (stop - start).signum();
    while current < stop
    {
        range.push(current);
        current += sign*step;
    }
    range
}

pub fn perform_magnetization_computation(rows: i32, columns: i32, param: &ExperimentParam) -> Result<Vec<f32>, CalculationError>
{
    let mut my_rng             = OsBasedRng::new();
    let initial_state = ||{ising_state::thermal_state(&mut my_rng)};
    
    if param.temperatures.iter().any(|x|*x < 0f32)
    {
        return Err(CalculationError::NegativeTempError);
    }
    let mut spin_2d_arr   = match PeriodicArray2D::new_with(rows, columns, initial_state)
    {
        Ok(v)     => v,
        Err(e) => return Err(CalculationError::ArrayInitError(e))
    };

    
    let mut magnetizations: Vec<f32> = Vec::new();
    let number_of_measures : usize   = param.measurement_steps / param.steps_between_measures ;

    for temp in &param.temperatures[..]
    {
        spin_2d_arr.reset(||{ising_state::thermal_state(&mut my_rng)});
        for _ in 0..param.thermalisation_steps 
        {
            metropolis::perform_metropolis_proposal(&mut spin_2d_arr, &mut my_rng, *temp, param.interaction_term);
        }

        let mut spin_sum: f32     = spin_2d_arr.sum();
        let mut spin_sum_avg: f32 = 0f32;
        for n in 0..param.measurement_steps 
        {
            spin_sum += metropolis::perform_metropolis_proposal(&mut spin_2d_arr, &mut my_rng, *temp, param.interaction_term);
            if n%param.steps_between_measures == 0
            {
                spin_sum_avg +=  spin_sum.abs()/(number_of_measures as f32);
            }
        }
        magnetizations.push(spin_sum_avg / spin_2d_arr.total_number() as f32);
    }

    Ok(magnetizations)
}