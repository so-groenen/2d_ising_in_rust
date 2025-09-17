#![allow(non_snake_case)] // use dS & dE like in statistical physics textbooks


use std::iter::zip;
use num_traits::{Float,AsPrimitive};
use std::io::{Write};
use rayon::{self, iter::{IntoParallelIterator, ParallelIterator}};


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
    pub steps_between_measures: usize,  
    pub thermalisation_steps: usize,      
    pub measurement_steps: usize       
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MonteCarloResults<T> where T: Float
{
    spins_sum_avg: T,   
    spins_sqr_avg: T,  
    energy_avg: T,       
    energy_sqr_avg: T,        
}
struct FourierTransformer<P>
{
    fourier_kernels: Vec<Complex<P>>,
}

fn t()
{

}

use num::{Complex};
use std::f64::consts::PI;
impl<P> FourierTransformer<P> where P: PhysicalObservable 
{
    fn new(Lx: usize) -> Self 
    {
        let qx  = 2_f64 * PI / Lx as f64;
        let qx = P::from(qx).unwrap(); // cast to f32 if necessary

        let fourier_kernels: Vec<Complex<P>> = (0..Lx).map(|x|  (Complex::<P>::i() * qx * (P::from(x as f64).unwrap())).exp() ).collect();
        Self {fourier_kernels}
    }
    fn take_fourier_transform<S>(&self, spins: &PeriodicArray2D<S,P>) -> (P, Complex<P>) 
        where S: SpinValue<P>, 
    {

        let (Ly, Lx)    = spins.shape();
        let factor      = 1_f64 / ((Lx*Ly) as f64).sqrt();
        let factor_real  = P::from(factor).unwrap();
        let factor_cmplx = Complex { re: factor_real, im: P::default() };

        let mut spin_q0 = P::default();
        let mut spin_qx = Complex::<P>::default();

        for y in spins.rows_range()
        {
            for x in spins.columns_range()
            {
                let s_real  = spins.at_unchecked(y, x).as_();
                let s_cmplx = Complex { re: s_real, im: P::default() };

                let exp_iqx = self.fourier_kernels[x as usize];  
                
                spin_q0 += factor_real * s_real;
                spin_qx = spin_qx + factor_cmplx * s_cmplx * exp_iqx;
            }
        }
        (spin_q0, spin_qx)
    }
}



impl<T> MonteCarloResults<T> where T: Float + std::fmt::Display
{
    pub fn write_to_file(file_name: &String, temperatures: &[T], results: &[MonteCarloResults<T>], num_spins: usize, elapsed_time: std::time::Duration ) -> std::io::Result<()>
    {
        if temperatures.len() != results.len()
        {
            return Err(std::io::Error::other("Results length should match temperature length"));
        }

        let mut file= std::fs::File::create(file_name)?;
        writeln!(&mut file, "temp, energy_density, magnetisation, specific_heat, susceptibility, elapsed_time: {}", elapsed_time.as_secs())?;

        let num_spins = T::from(num_spins).unwrap();

        for  (&temp, res) in zip(temperatures, results)
        {
            let specific_heat = (res.energy_sqr_avg - res.energy_avg.powi(2) ) / (temp.powi(2) * num_spins);
            let energy_density = res.energy_avg / num_spins;
            let magnetisation = res.spins_sum_avg / num_spins;
            let susceptibility = (res.spins_sqr_avg - res.spins_sum_avg.powi(2)) / (temp * num_spins);
            
            writeln!(&mut file, "{temp}, {energy_density}, {magnetisation}, {specific_heat}, {susceptibility}")?;
        }
    
        Ok(())
    }
}

 

pub fn arange<T>(start: T, stop: T, step: T) -> Result<Vec<T>, &'static str> where T: Float
{
    if step == T::zero()
    {
        return Err("Arange Error: Step must be non zero")
    }

    let direction_sign = (stop - start).signum();
    if step.signum() != direction_sign
    {   
        return Err("Arange Error: if stop > (<) start then step must be positive (negative).")
    }   
    let num_of_values = ((stop - start).abs() / step).round().to_usize().unwrap();
    let my_arange  = (0..num_of_values).map(|val| start + step * T::from(val).unwrap() ).collect::<Vec<_>>();
    Ok(my_arange)
}




pub fn perform_metropolis_computation_parallel<S,P>(rows: i32, columns: i32, param: &ExperimentParam<P>) -> Result<Vec<MonteCarloResults<P>>, CalculationError>
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
    let number_of_measures = param.measurement_steps / param.steps_between_measures ;
    let take_fourier       = false;
    (&param.temperatures, &mut results).into_par_iter().for_each(|(&temp, result)|
    {
        let mut my_rng      = Xoshiro256pp::from_os(); 
        let init_state      = ||ising_state::spin_up::<S>();
        let mut spin_2d_arr = PeriodicArray2D::new_with(rows, columns, init_state).unwrap();
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

        for n in 0..param.measurement_steps 
        {                                    
            if n%param.steps_between_measures == 0
            {
                spin_sum_avg     += spin_sum.abs() /(number_of_measures.as_());
                energy_avg       += total_energy/(number_of_measures.as_());
                energy_sqr_avg   += (total_energy * total_energy) /(number_of_measures.as_());
                spin_sqr_avg     += (spin_sum * spin_sum) /(number_of_measures.as_());
            }
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
        // result.struct_fact_q0 = re_spin_q0_sqr_acc;                          //S(q0) =  <Re[sigma_q0]²>
        // result.struct_fact_qx = (re_spin_qx_sqr_acc + im_spin_qx_sqr_acc); 
    });
    Ok(results)
}


 

