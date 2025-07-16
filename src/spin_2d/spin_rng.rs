// use macroquad::rand::RandomRange;

pub trait SpinRNG
{
    fn generate_rand_f32(&mut self, low: f32, high: f32) -> f32;
    fn generate_rand_i32(&mut self, low: i32, high: i32) -> i32;    
    fn new() -> Self;
}

pub struct SpinMacroquadRng
{
    generator: macroquad::rand::RandGenerator
}

impl SpinRNG for SpinMacroquadRng
{
    fn generate_rand_f32(&mut self, low: f32, high: f32) -> f32
    {
        self.generator.gen_range(low, high)
    }
    fn generate_rand_i32(&mut self, low: i32, high: i32) -> i32
    {
        self.generator.gen_range(low, high)
    }
    fn new() -> Self 
    {
        SpinMacroquadRng {generator: macroquad::rand::RandGenerator::new()}
    }
}