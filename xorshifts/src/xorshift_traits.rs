 
pub trait XorshiftRandomGenerator
{
    fn generate(&mut self) -> u64;
    fn generate_rand_u32(&mut self, low: u32, high: u32) -> u32
    {
        let rand  = self.generate() as u32;
        let delta = rand%(high - low);
        
        low + delta
    }
    fn rand_f32(&mut self) -> f32
    {
        let rand_float_int = (self.generate() >> 40) as f32; // float has 24bit mantissa, hence 64-40=24 to encode the largest "integer" float  
        let max_float_int  = (1_u32 << 24) as f32;
        rand_float_int / max_float_int
    }
    #[allow(dead_code)] // i keep it for completness
    fn rand_f64(&mut self) -> f64
    {
        let rand_double_int = (self.generate() >> 11) as f64; // double has 53bit mantissa so largest "integer" double has 53bit (64-11)
        let max_double_int  = (1_u64 << 53) as f64;
        rand_double_int / max_double_int
    }
}
