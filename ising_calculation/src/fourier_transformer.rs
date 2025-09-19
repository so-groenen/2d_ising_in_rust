use periodic_array_2d_lib::PhysicalObservable;
use periodic_array_2d_lib::PeriodicArray2D;
use periodic_array_2d_lib::SpinValue;

pub struct FourierTransformer<P>
{
    fourier_kernels: Vec<Complex<P>>,
}


use num::{Complex};
use std::f64::consts::PI;
impl<P> FourierTransformer<P> where P: PhysicalObservable 
{
    pub fn new(Lx: usize) -> Self 
    {
        let qx  = 2_f64 * PI / Lx as f64;
        let qx = P::from(qx).unwrap(); // cast to f32 if necessary

        let fourier_kernels: Vec<Complex<P>> = (0..Lx).map(|x|  (Complex::<P>::i() * qx * (P::from(x as f64).unwrap())).exp() ).collect();
        Self {fourier_kernels}
    }
    pub fn take_fourier_transform<S>(&self, spins: &PeriodicArray2D<S,P>) -> (P, Complex<P>) 
        where S: SpinValue<P>, 
    {

        let (Ly, Lx)     = spins.shape();
        let factor       = 1_f64 / ((Lx*Ly) as f64).sqrt();
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
                spin_qx  = spin_qx + factor_cmplx * s_cmplx * exp_iqx;
            }
        }
        (spin_q0, spin_qx)
    }
}