use monte_carlo_lib::MonteCarloRngInterface;
use periodic_array_2d_lib::ArrayRngInterface;

pub struct MyRNG
{
    rng: macroquad::rand::RandGenerator, 
}
impl MyRNG
{
    pub fn new() -> Self
    {
        Self { rng: macroquad::rand::RandGenerator::new() }
    }    
}
impl ArrayRngInterface for MyRNG
{
    fn generate_rand_i32(&mut self, low: i32, high: i32) -> i32 
    {
        self.rng.gen_range(low, high) 
    }
}
impl MonteCarloRngInterface for MyRNG
{
    fn generate_rand_f32(&mut self, low: f32, high: f32) -> f32
    {
        self.rng.gen_range(low, high)
    }
}