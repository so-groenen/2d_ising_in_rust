
use periodic_array_2d_lib::ArrayRngInterface;
use monte_carlo_lib::MonteCarloRngInterface;
use rand::{self, Rng};

pub struct OsBasedRng
{
    generator: rand::rngs::ThreadRng
} 
impl ArrayRngInterface for OsBasedRng
{
    fn generate_rand_i32(&mut self, low: i32, high: i32) -> i32
    {
        self.generator.random_range(low..high)
    }
}
impl MonteCarloRngInterface for OsBasedRng
{
    fn generate_rand_f32(&mut self, low: f32, high: f32) -> f32
    {
        self.generator.random_range(low..high)
    }
}
impl OsBasedRng
{
    pub fn new() -> OsBasedRng 
    {
        OsBasedRng {generator: rand::rng()}
    }
}