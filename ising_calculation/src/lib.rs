#![allow(non_snake_case)] // use dS & dE like in statistical physics textbooks

use num_traits::{AsPrimitive};
use rayon::{self, iter::{IntoParallelIterator, ParallelIterator}};
mod monte_carlo_results;
mod fourier_transformer;

pub use monte_carlo_results::MonteCarloResults;
use fourier_transformer::FourierTransformer;

use periodic_array_2d_lib::{PeriodicArray2D, PeriodicArrayError, SpinValue, PhysicalObservable};
use monte_carlo_lib::{ising_state, metropolis, SpinEnergyFluctuation, MonteCarloRngInterface};
use xorshifts::Xoshiro256pp;


#[derive(Debug)]
pub enum CalculationError
{
    NegativeTempError,
    ArrayInitError(PeriodicArrayError),
}

pub struct ExperimentParam<P> where P: PhysicalObservable  
{
    pub temperatures: Vec<P>,
    pub interaction_term: P,
    pub extern_mag: P,
    pub thermalisation_steps: usize,      
    pub measurement_steps: usize,
    pub measure_struct_fact: bool,
}


pub fn perform_metropolis_computation_parallel<S,P>(rows: usize, columns: usize, param: &ExperimentParam<P>) -> Result<Vec<MonteCarloResults<P>>, CalculationError>
    where P:     PhysicalObservable + Send + Sync,     // Send&Sync: to work with parallelIterator
          usize: AsPrimitive<P>,                        
          S:     SpinValue<P>,
          Xoshiro256pp: MonteCarloRngInterface<P>
{   
    if param.temperatures.iter().any(|x| x.is_sign_negative())
    {
        return Err(CalculationError::NegativeTempError);
    }

    let n_values           = param.temperatures.len();
    let mut results        = vec![MonteCarloResults::<P>::default(); n_values];
    let number_of_measures = param.measurement_steps;
    let take_fourier       = param.measure_struct_fact;

    (&param.temperatures, &mut results).into_par_iter().for_each(|(&temp, result)|
    {
        let mut my_rng      = Xoshiro256pp::from_os(); 
        let init_state      = ||ising_state::spin_up::<S>();
        let mut spin_2d_arr = PeriodicArray2D::new_with(rows as i32, columns as i32, init_state).unwrap();
        let fourier_transf  = FourierTransformer::new(columns as usize);

        let mut spin_sum_avg   = P::zero();
        let mut energy_avg     = P::zero();
        let mut spin_sqr_avg   = P::zero();
        let mut energy_sqr_avg = P::zero();

        let mut re_spin_q0_sqr_avg = P::zero(); //  <Re[sigma_q0]²>  
        let mut re_spin_qx_sqr_avg = P::zero(); //  <Re[sigma_qx]²>  
        let mut im_spin_qx_sqr_avg = P::zero();
        for _ in 0..param.thermalisation_steps 
        {
            metropolis::perform_metropolis_sweep(&mut spin_2d_arr, &mut my_rng, temp, param.interaction_term, param.extern_mag);
        }

        let mut spin_sum: P     = spin_2d_arr.sum_observable();  
        let mut total_energy: P = metropolis::get_total_energy(&spin_2d_arr, param.interaction_term, param.extern_mag);

        for _ in 0..param.measurement_steps 
        {                                    
         
            spin_sum_avg     += spin_sum.abs() /(number_of_measures.as_());
            energy_avg       += total_energy/(number_of_measures.as_());
            energy_sqr_avg   += (total_energy * total_energy) /(number_of_measures.as_());
            spin_sqr_avg     += (spin_sum * spin_sum) /(number_of_measures.as_());
           
            if take_fourier
            {
                let (spin_q0, spin_qx) = fourier_transf.take_fourier_transform(&spin_2d_arr);
                re_spin_q0_sqr_avg += spin_q0*spin_q0 /number_of_measures.as_();
                re_spin_qx_sqr_avg += spin_qx.re*spin_qx.re /number_of_measures.as_();
                im_spin_qx_sqr_avg += spin_qx.im*spin_qx.im /number_of_measures.as_();
            }


            let SpinEnergyFluctuation(dS, dE) = metropolis::perform_metropolis_sweep(&mut spin_2d_arr,
                                                                                    &mut my_rng,
                                                                                    temp,
                                                                                    param.interaction_term,
                                                                                    param.extern_mag);   

            spin_sum     += dS; 
            total_energy += dE;
        }

        result.spins_sum_avg  = spin_sum_avg;
        result.energy_avg     = energy_avg;
        result.spins_sqr_avg  = spin_sqr_avg;
        result.energy_sqr_avg = energy_sqr_avg;
        result.struct_fact_q0 = re_spin_q0_sqr_avg;                           //S(q0) =  <Re[sigma_q0]²>
        result.struct_fact_qx = re_spin_qx_sqr_avg + im_spin_qx_sqr_avg;      //S(qx) =  <Im[sigma_qx]²> + <Im[sigma_qx]²>
    });
    Ok(results)
}


 

