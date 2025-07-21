
use periodic_array_2d_lib::ArrayRngInterface;
use monte_carlo_lib::MonteCarloRngInterface;

pub struct MacroquadRng
{
    generator: macroquad::rand::RandGenerator
}
impl ArrayRngInterface for MacroquadRng
{
    fn generate_rand_i32(&mut self, low: i32, high: i32) -> i32
    {
        self.generator.gen_range(low, high)
    }
}
impl MonteCarloRngInterface for MacroquadRng
{
    fn generate_rand_f32(&mut self, low: f32, high: f32) -> f32
    {
        self.generator.gen_range(low, high)
    }
}
impl MacroquadRng
{
    pub fn new() -> MacroquadRng 
    {
        MacroquadRng {generator: macroquad::rand::RandGenerator::new()}
    }
}