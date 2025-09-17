pub mod array_rng_interface;
use std::marker::PhantomData;


use std::{ops::AddAssign};
pub use array_rng_interface::ArrayRngInterface;
use num_traits::{AsPrimitive, Float, FromPrimitive, Num};

pub trait PhysicalObservable: Float + AddAssign + std::default::Default +'static
{    
}

impl<P> PhysicalObservable for P where P: Float + AddAssign + std::default::Default + 'static
{    
}

pub trait SpinValue<P>: Num + Copy + Default + std::ops::Neg<Output = Self> + 'static + FromPrimitive + AsPrimitive<P> where P: PhysicalObservable
{
}
impl<S,P> SpinValue<P> for S where S: Num + Copy  + Default + std::ops::Neg<Output = Self> + 'static + FromPrimitive + AsPrimitive<P>,  P: PhysicalObservable
{    
} 



pub struct PeriodicArray2D<S, P> where S: SpinValue<P>, P: PhysicalObservable
{
    data: Vec<S>,
    rows: i32,
    columns: i32,
    number_of_spins: i32,
    _phantom: PhantomData<P>
}

#[derive(Debug)]
pub struct PeriodicArrayError
{
    from: String,
    message: String
}
impl std::fmt::Display for PeriodicArrayError 
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result 
    {
        write!(f, "from: {}, msg: {}", self.from, self.message)
    }
}
impl From <std::num::TryFromIntError> for PeriodicArrayError  
{
    fn from(error: std::num::TryFromIntError) -> Self
    {
        PeriodicArrayError 
        {
            from: String::from("TryFromIntError"),
            message: error.to_string(),
        }
    }
}
impl From <&str> for PeriodicArrayError  
{
    fn from(error: &str) -> Self
    {
        PeriodicArrayError 
        {
            from: String::from("TryFromIntError"),
            message: error.to_string(),
        }
    }
}

//helper function: modulo without % operator
trait MonteCarloModulo
{
    fn modulo(self, other: Self) -> Self;
}

impl MonteCarloModulo for i32
{
    #[inline(always)]
    fn modulo(self, other: Self) -> Self
    {
        debug_assert!(other > 0, "Modulo: X%N, N must be > 0");
        if self >= other
        {
            return self-other
        }
        else if self < 0
        {
            return self+other
        }
        self
    }    
}


impl<S,P> PeriodicArray2D<S,P> where S: SpinValue<P>, P: PhysicalObservable
{   
    fn get_total_elements_usize(rows: i32, columns: i32) -> Result<usize, PeriodicArrayError>
    {
        if rows*columns <= 0 || (rows <0 && columns < 0)
        {
            return Err(PeriodicArrayError 
            {
                from: String::from("PeriodicArray2D::new()"),
                message: String::from("Rows & columns need to be > 0.")
            })
        };
        let n_elements: usize = (columns*rows) as usize;
        Ok(n_elements)
    }
    pub fn rows(&self) -> i32
    {
        self.rows
    }
    pub fn columns(&self) -> i32
    {
        self.columns
    }
    #[inline(always)]
    pub fn shape(&self) -> (i32, i32)
    {
        (self.rows, self.columns)
    }
    #[inline(always)]
    pub fn rows_range(&self) -> std::ops::Range<i32>
    {
        0..self.rows()
    }
    #[inline(always)]
    pub fn columns_range(&self) -> std::ops::Range<i32>
    {
        0..self.columns()
    }
    #[inline(always)]
    pub fn total_number(&self) -> i32
    {
        self.number_of_spins
    }
    #[inline(always)]
    pub fn all_range(&self) -> std::ops::Range<i32>
    {
        0..self.number_of_spins
    }
    pub fn new_with(rows: i32, columns: i32, generator: impl FnMut()-> S) -> Result<Self, PeriodicArrayError>
    {        
        let n_elements: usize  = Self::get_total_elements_usize(rows, columns)?;
        
        let mut data: Vec<S> = vec![S::zero(); n_elements];
        data.fill_with(generator);

        Ok(PeriodicArray2D {data, rows, columns, number_of_spins: n_elements as i32, _phantom: PhantomData})
    }    
    #[inline(always)]
    fn get_index(&self, i: i32, j: i32) -> usize
    {
        let i = i.modulo(self.rows);
        let j = j.modulo(self.columns);
        (i*self.columns + j) as usize
    }
    #[inline(always)]
    pub fn reset(&mut self, generator: impl FnMut()-> S)
    {
        self.data.fill_with(generator);
    }
    // pub fn at(&self, i: i32, j: i32) -> Result<S, PeriodicArrayError>
    // {
    //     let index: usize     = self.get_index(i,j);
    //     let Some(value)= self.data.get(index) else
    //     {
    //         return Err(
    //         PeriodicArrayError
    //         {
    //             from: String::from("spin2D::at()"),
    //             message: String::from("Access out of bound")
    //         });
    //     };
    //     Ok(*value)
    // }
    #[inline(always)]
    pub fn at_unchecked(&self, i: i32, j: i32) -> S
    {
        self.data[self.get_index(i,j)]
    }

    // pub fn at_mut(&mut self, i: i32, j: i32) -> Result<&mut S, PeriodicArrayError>
    // {
    //     let index = self.get_index(i,j);
    //     let Some(value)  = self.data.get_mut(index) else
    //     {
    //         return Err(PeriodicArrayError 
    //         {
    //             from: String::from("PeriodicArray2D::at_mut()"),
    //             message: String::from("Access out of bound")
    //         });
    //     };
    //     Ok(value)
    // }
    #[inline(always)]
    pub fn at_mut_unchecked(&mut self, i: i32, j: i32) -> &mut S
    {
        let index = self.get_index(i,j);
        &mut self.data[index]
    }
    #[inline(always)]
    pub fn sum(&self) -> S
    {
        self.data.iter().fold(S::zero(), |acc, &x| acc + x)
    }
    #[inline(always)]
    pub fn sum_observable(&self) -> P
    {
        self.data.iter().fold(P::default(), |acc, &x| acc + x.as_())
    }
    pub fn get_random_point<R: ArrayRngInterface>(&self, rng: &mut R) -> (i32, i32)
    {
        let x: i32 = rng.generate_rand_i32(0, self.number_of_spins);
        (x / self.columns, x % self.columns )
    }
}
