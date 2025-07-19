// mod periodic_array_2d;
// pub use crate::periodic_array_2d::PeriodicArray2D;
// pub use crate::periodic_array_2d::ArrayRngInterface;

pub mod array_rng_interface;
pub use array_rng_interface::ArrayRngInterface;

pub struct PeriodicArray2D
{
    data: Vec<f32>,
    rows: i32,
    columns: i32,
    number_of_spins: i32,
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


//helper function
fn get_mod(x: i32, n: i32) -> i32
{   
    assert!(n > 0);
    let mut x = x%n;
    while x < 0
    {
        x += n;    
    }
    x
}

impl PeriodicArray2D 
{   
    fn _get_total_elements_usize(rows: i32, columns: i32) -> Result<usize, PeriodicArrayError>
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
    pub fn rows_range(&self) -> std::ops::Range<i32>
    {
        0..self.rows()
    }
    pub fn columns_range(&self) -> std::ops::Range<i32>
    {
        0..self.columns()
    }
    pub fn total_number(&self) -> i32
    {
        self.number_of_spins
    }
    pub fn all_range(&self) -> std::ops::Range<i32>
    {
        0..self.number_of_spins
    }
    pub fn new_with(rows: i32, columns: i32, generator: impl FnMut()->f32) -> Result<Self, PeriodicArrayError>
    {        
        let n_elements: usize  = Self::_get_total_elements_usize(rows, columns)?;
        
        let mut data: Vec<f32> = vec![Default::default(); n_elements];
        data.fill_with(generator);

        Ok(PeriodicArray2D { data, rows, columns, number_of_spins: n_elements as i32})
    }    
    fn _get_index(&self, i: i32, j: i32) -> usize
    {
        // assert!(self.rows > 0);
        // assert!(self.columns > 0);
        let i: i32       = get_mod(i, self.rows);
        let j: i32       = get_mod(j, self.columns);
        (i*self.columns + j) as usize
    }
    pub fn reset(&mut self, generator: impl FnMut()->f32)
    {
        self.data.fill_with(generator);
    }
    pub fn at(&self, i: i32, j: i32) -> Result<f32, PeriodicArrayError>
    {
        let index: usize     = self._get_index(i,j);
        let Some(value)= self.data.get(index) else
        {
            return Err(
            PeriodicArrayError
            {
                from: String::from("spin2D::at()"),
                message: String::from("Access out of bound")
            });
        };
        Ok(*value)
    }
    pub fn at_unchecked(&self, i: i32, j: i32) -> f32
    {
        self.data[self._get_index(i,j)]
    }

    pub fn at_mut(&mut self, i: i32, j: i32) -> Result<&mut f32, PeriodicArrayError>
    {
        let index: usize = self._get_index(i,j);
        let Some(value)  = self.data.get_mut(index) else
        {
            return Err(PeriodicArrayError 
            {
                from: String::from("PeriodicArray2D::at_mut()"),
                message: String::from("Access out of bound")
            });
        };
        Ok(value)
    }
    pub fn at_mut_unchecked(&mut self, i: i32, j: i32) -> &mut f32
    {
        let index: usize = self._get_index(i,j);
        &mut self.data[index]
    }
    pub fn get_average_magnetization(&self) -> f32
    {
        // assert!(self.number_of_spins>0);
        self.sum() / self.number_of_spins as f32
    }
    pub fn sum(&self) -> f32
    {
        let mut sum = 0f32;
        for v in &self.data
        {
            sum += *v;
        }
        sum
    }
    pub fn get_random_point<R: ArrayRngInterface>(&self, rng: &mut R) -> (i32, i32)
    {
        let x: i32 = rng.generate_rand_i32(0, self.number_of_spins);
        (x / self.columns, x % self.columns )
    }
}
