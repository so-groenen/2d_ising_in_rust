
pub mod ising_state
{   
    pub const SPINUP: f32   = 1f32;
    pub const SPINDOWN: f32 = -1f32;
    pub fn thermal_state(rng: &mut macroquad::rand::RandGenerator) ->f32
    {
        1f32 - 2f32*rng.gen_range(0f32, 1f32).round()
    }
}

// We use i32 instead of usize because we allow negative indices for periodic boundary conditions.
pub struct Spin2D
{
    data: Vec<f32>,
    rows: i32,
    columns: i32,
    n: i32,
}



#[derive(Debug)]
pub struct Spin2DError
{
    from: String,
    message: String
}
impl std::fmt::Display for Spin2DError 
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result 
    {
        write!(f, "from: {}, msg: {}", self.from, self.message)
    }
}
impl From <std::num::TryFromIntError> for Spin2DError  
{
    fn from(error: std::num::TryFromIntError) -> Self
    {
        Spin2DError 
        {
            from: String::from("TryFromIntError"),
            message: error.to_string(),
        }
    }
}


//helper function
fn get_mod(x: i32, n: i32) -> i32
{   

    let mut x = x%n;
    while x < 0
    {
        x += n;    
    }
    x
}

impl Spin2D 
{   
    const MAX_BETA: f32 = 1E6;
    fn _get_total_elements_usize(rows: i32, columns: i32) -> Result<usize, Spin2DError>
    {
        if rows * columns <= 0
        {
            return Err( Spin2DError 
            {
                from: String::from("Spin2D::new()"),
                message: String::from("Rows & columns need to be > 0.")
            })
        };
        let n_elements: usize = (columns*rows).try_into()?;
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
    pub fn new_with(rows: i32, columns: i32, generator: impl FnMut()->f32) -> Result<Self, Spin2DError>
    {        
        let n_elements: usize  = Self::_get_total_elements_usize(rows, columns)?;
        
        let mut data: Vec<f32> = vec![Default::default(); n_elements];
        data.fill_with(generator);

        Ok(Spin2D { data, rows, columns , n: n_elements.try_into().unwrap()})
    }    
    fn _get_index(&self, i: i32, j: i32) -> Result<usize, Spin2DError>
    {
        assert!(self.rows > 0);
        assert!(self.columns > 0);
        let i: i32       = get_mod(i, self.rows);
        let j: i32       = get_mod(j, self.columns);
        let index: usize = (i*self.columns + j).try_into()?;
        Ok(index)
    }
    pub fn at(&self, i: i32, j: i32) -> Result<f32, Spin2DError>
    {
        let index: usize = self._get_index(i,j)?;
        let Some(value)  = self.data.get(index) else
        {
            return Err(Spin2DError 
                {
                    from: String::from("spin2D::at()"),
                    message: String::from("Access out of bound")
                });
        };
        Ok(*value)
    }
    pub fn at_unchecked(&self, i: i32, j: i32) -> f32
    {
        self.at(i, j).unwrap()
    }
    pub fn at_mut(&mut self, i: i32, j: i32) -> Result<&mut f32, Spin2DError>
    {
        let index: usize = self._get_index(i,j)?;
        let Some(value)  = self.data.get_mut(index) else
        {
            return Err(Spin2DError 
                {
                    from: String::from("Spin2D::at_mut()"),
                    message: String::from("Access out of bound")
                });
        };
        Ok(value)
    }

    fn _accept_state(temp: f32, delta_energy: f32, rng: &mut macroquad::rand::RandGenerator) -> bool
    {
        let beta: f32 = if temp > 0. {1./temp} else {Spin2D::MAX_BETA};
        delta_energy < 0. || (rng.gen_range(0f32, 1f32) < (-beta*delta_energy).exp())
    }

    pub fn get_average_magnetization(&self) -> f32
    {
        assert!(self.n>0);
        self.sum() / self.n as f32
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
    fn _get_random_point(&self, rng: &mut macroquad::rand::RandGenerator) -> (i32, i32)
    {
        let x: i32 = rng.gen_range(0, self.n);
        (x / self.columns, x % self.columns )
    }
    fn _get_delta_energy(&self, i: i32, j: i32, interaction_term: f32) -> f32
    {
        let left_right: f32 = self.at_unchecked(i-1, j) + self.at_unchecked(i+1, j);
        let up_down: f32    = self.at_unchecked(i, j+1) + self.at_unchecked(i, j-1);
        
        -2.*interaction_term*self.at_unchecked(i,j)*(left_right + up_down)
    }
    pub fn get_total_energy(&self, interaction_term: f32) -> f32
    {
        let mut total_energy: f32 = 0f32;
        for i in self.rows_range()
        {
            for j in self.columns_range()
            {
                let up: f32    = self.at_unchecked(i+1, j);
                let right: f32 = self.at_unchecked(i, j+1);
                total_energy  += interaction_term*self.at_unchecked(i,j)*(up + right);
            }
        }
        total_energy
    }
    pub fn perform_monte_carlo_sweep(&mut self, temp: f32, interaction_term: f32, rng: &mut macroquad::rand::RandGenerator)
    {
        for _ in 0..self.n
        {
            let (i_rand, j_rand) = self._get_random_point(rng);
            let delta_energy     = self._get_delta_energy(i_rand, j_rand, interaction_term);
            if Self::_accept_state(temp, delta_energy, rng) 
            {
                *self.at_mut(i_rand, j_rand).unwrap() *= -1.;
            }
        }
    }
}
