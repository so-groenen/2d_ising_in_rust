
use std::{ops::{Add, AddAssign}};

use num_traits::AsPrimitive;
use num_traits::Float;
use periodic_array_2d_lib::{PeriodicArray2D, ArrayRngInterface};
use periodic_array_2d_lib::{SpinValue,PhysicalObservable};


pub trait MonteCarloRngInterface<T>  where T: Float
{
    fn generate_rand_float(&mut self, low: T, high: T) -> T;
}

pub mod ising_state
{   
    use std::ops::Neg;
    use num_traits::{Num, Signed};
    use super::*;
    pub fn spin_up<S>() -> S where S: Num 
    {
        S::one()
    }
    pub fn spin_down<S>() -> S where S: Num + Neg<Output = S>
    {
        -S::one()
    }

    pub mod i8_spins 
    {
        pub const SPINUP: i8   = 1_i8;
        pub const SPINDOWN: i8 = -1_i8;
    }
    // pub mod i32_spins 
    // {
    //     pub const SPINUP: i32   = 1_i32;
    //     pub const SPINDOWN: i32 = -1_i32;
    // }
    // pub const SPINUP: i32   = 1_i32;
    // pub const SPINDOWN: i32 = -1_i32;
    pub fn thermal_state<T, R>(rng: &mut R) -> T  
        where R: MonteCarloRngInterface<f32>, 
              T: Signed + Copy + 'static,
            f32: AsPrimitive<T>
    {
        ( 1_f32 - 2_f32 * rng.generate_rand_float(0_f32, 1_f32).round()).as_()
    }
}

// pub trait PhysicalObservable: Float + AddAssign + std::default::Default +'static
// {    
// }

// impl<E> PhysicalObservable for E where E: Float + AddAssign + std::default::Default + 'static
// {    
// }

#[derive(Default, Clone, Copy)]
pub struct SpinEnergyFluctuation<P>(pub P, pub P) where P: PhysicalObservable; // We will use the spin fluctuation dS to construct the magnitization

impl<P> Add for SpinEnergyFluctuation<P>  where P: PhysicalObservable
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self
    {
        Self(self.0 + rhs.0, self.1 + rhs.1) 
    }
}

impl<P> AddAssign for SpinEnergyFluctuation<P>  where P: PhysicalObservable
{
    fn add_assign(&mut self, rhs: Self) 
    {
        *self = *self + rhs; 
    }
}


pub mod metropolis
{
    use super::*;
    pub const MAX_BETA: f32 = 1E6;

    #[allow(non_snake_case)]
    fn get_delta_energy<S, P>(spin_2d_arr: &PeriodicArray2D<S,P>, i: i32, j: i32, interaction_term: P, extern_mag: P) -> P 
        where S: SpinValue<P>, 
              P: PhysicalObservable,
    {
        let spin_LR = (spin_2d_arr.at_unchecked(i-1, j) + spin_2d_arr.at_unchecked(i+1, j)).as_();
        let spin_UP = (spin_2d_arr.at_unchecked(i, j+1) + spin_2d_arr.at_unchecked(i, j-1)).as_();
        let spin_ij = (spin_2d_arr.at_unchecked(i,j)).as_();


        P::from(2.).unwrap() * spin_ij * (interaction_term*(spin_LR + spin_UP) - extern_mag)
    }


    fn accept_state<R, P>(temp: P, delta_energy: P, rng: &mut R) -> bool 
        where R: MonteCarloRngInterface<P> + ArrayRngInterface, 
              P: PhysicalObservable,
    {
        let beta = if temp.is_sign_positive()  {P::one()/temp} else {unsafe{P::from(MAX_BETA).unwrap_unchecked()}};
        
        delta_energy.is_sign_negative() || rng.generate_rand_float(P::zero(), P::one()) < (-beta*delta_energy).exp()
    }

    pub fn perform_metropolis_proposal<R, S, P>(spin_2d_arr: &mut PeriodicArray2D<S,P>, rng: &mut R, temp: P, interaction_term: P, extern_mag: P) -> SpinEnergyFluctuation<P>
        where R: MonteCarloRngInterface<P> + ArrayRngInterface, 
              S: SpinValue<P>, 
              P: PhysicalObservable,
    {
        let (i, j)        = spin_2d_arr.get_random_point(rng);
        let delta_energy  = get_delta_energy(spin_2d_arr, i, j, interaction_term, extern_mag);

        if accept_state(temp, delta_energy, rng) 
        {
            let s = spin_2d_arr.at_mut_unchecked(i, j);
            (*s)  = s.neg();

            let delta_spin = ((*s) + (*s)).as_();

            return SpinEnergyFluctuation(delta_spin, delta_energy);
        }
        SpinEnergyFluctuation::default()
    }


 
    #[allow(non_snake_case)]
    pub fn perform_metropolis_sweep<R, S, P>(spin_2d_arr: &mut PeriodicArray2D<S,P>, rng: &mut R, temp: P, interaction_term: P, extern_mag: P) -> SpinEnergyFluctuation<P>
        where R: MonteCarloRngInterface<P> + ArrayRngInterface, 
              S: SpinValue<P>, 
              P: PhysicalObservable,
    {
        let mut dS_and_dE = SpinEnergyFluctuation::default();
        for _ in 0..spin_2d_arr.total_number()
        {
            dS_and_dE += perform_metropolis_proposal(spin_2d_arr, rng, temp, interaction_term, extern_mag);
        }

        dS_and_dE
    }

 

    pub fn get_total_energy<S, P>(spin_2d_arr: &PeriodicArray2D<S,P>, interaction_term: P, extern_mag: P) -> P  
        where S: SpinValue<P>,// + AsPrimitive<P>, 
              P: PhysicalObservable
    {
        let mut total_energy = P::zero();
        for i in spin_2d_arr.rows_range()
        {
            for j in spin_2d_arr.columns_range()
            {
                let right = spin_2d_arr.at_unchecked(i+1, j).as_();
                let below = spin_2d_arr.at_unchecked(i, j+1).as_();
                let spin  = spin_2d_arr.at_unchecked(i,j).as_();
                total_energy +=  - spin*(interaction_term*(right + below) - extern_mag);
            }
        }

        total_energy
    }

 
}