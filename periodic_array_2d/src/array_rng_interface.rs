// We want our array random functions to be independant of specific RNG generator, it should only depend on a basic function.
pub trait ArrayRngInterface
{
    fn generate_rand_i32(&mut self, low: i32, high: i32) -> i32;    
}
 