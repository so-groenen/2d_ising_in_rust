#![allow(dead_code)]

mod xorshift_traits;
use core::f64;

pub use xorshift_traits::XorshiftRandomGenerator;
use chrono::prelude::*;

use monte_carlo_lib::MonteCarloRngInterface;
use periodic_array_2d_lib::ArrayRngInterface;

// Different XOR-shift random number generators
// We have Xorshift64, Xoroshiro128p, Xoshiro256p, Xoshiro256pp

// Based on: https://en.wikipedia.org/wiki/Xorshift 
// As well as: https://prng.di.unimi.it/ 
// uses "wrapping_add" && "unbounded_shl" (shift-left) to mimick what you get in C & avoid undef-behaviour

//WASM friendly way of seeding our RNGs
fn get_random_seed_from_utc_time() -> u64
{
    let utc_time: DateTime<Utc> = Utc::now();  
    
    let epoch_utc = DateTime::parse_from_rfc2822(&utc_time.to_rfc2822()).unwrap().timestamp();
    epoch_utc as u64
}

//Thread local approach for use with Rayon (each thread will get its own seed)
#[cfg(not(target_arch = "wasm32"))]
fn get_seed_os() -> u64
{
    use rand::{self, RngCore};

    let mut rng = rand::rng();
    let seed = rng.next_u64();
    seed
}

fn rol64(x: u64, k: u32) -> u64
{
// 	          (x << k)   | (x >> (64 - k))
	(x.unbounded_shl(k)) | (x.unbounded_shr(64 - k))
}
pub struct Xorshift64
{
    state: u64,
}
pub struct Xoroshiro128p
{
    state: [u64; 2],
}
pub struct Xoshiro256p
{
    state: [u64; 4],
}
pub struct Xoshiro256pp
{
    state: [u64; 4],
}
/////////////////////
// RNG algorithms: //
/////////////////////

// https://en.wikipedia.org/wiki/Xorshift#Example_implementation
impl XorshiftRandomGenerator for Xorshift64
{
    #[inline(always)]
    fn generate(&mut self) -> u64
    {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }    
}
// https://xorshift.di.unimi.it/xoroshiro128plus.c
impl XorshiftRandomGenerator for Xoroshiro128p
{
    fn generate(&mut self) -> u64
    {
        let s0     = self.state[0];
        let mut s1 = self.state[1]; 

        // let result = s0 + s1;
        let result = s0.wrapping_add(s1);
        s1 ^= s0;

        // self.state[0] = rol64(s0, 24) ^ s1 ^ (s1 << 16);  
        self.state[0] = rol64(s0, 24)^s1^(s1.unbounded_shl(16));  
        self.state[1] = rol64(s1, 37);  
        result
    }    
}
// https://en.wikipedia.org/wiki/Xorshift#xoshiro256+
impl XorshiftRandomGenerator for Xoshiro256p
{
    fn generate(&mut self) -> u64
    {
        let s  = &mut self.state;

        // let result = s[0] + s[3];
        let result = s[0].wrapping_add(s[3]);
        
        // let t = s[1] << 17;
        let t = s[1].unbounded_shl(17);

        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];

        s[2] ^= t;
        s[3] = rol64(s[3], 45);

        result
    }
}
// https://en.wikipedia.org/wiki/Xorshift#xoshiro256++
impl XorshiftRandomGenerator for Xoshiro256pp
{
    fn generate(&mut self) -> u64
    {
        let s  = &mut self.state;

        // let result = rol64(s[0] + s[3], 23) + s[0];
        let result = rol64(s[0].wrapping_add(s[3]), 23).wrapping_add(s[0]);
        
        // let t = s[1] << 17;
        let t = s[1].unbounded_shl(17);

        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];

        s[2] ^= t;
        s[3] = rol64(s[3], 45);

        result
    }
}

/////////////////////////////////
//       Constructors:        //
////////////////////////////////
impl Xorshift64
{
    
    pub fn new(seed: u64)-> Self
    {
        let state   = seed;
        let mut rng = Self{state};
        rng.generate();
        rng
    }    
    pub fn from_utc_time() -> Self
    {
        let seed    = get_random_seed_from_utc_time();
        let mut rng = Self { state: seed };
        rng.generate();
        rng
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_os() -> Self
    {
        let state   = get_seed_os();
        let mut rng = Self { state };
        rng.generate();
        rng
    }
}
impl Xoroshiro128p
{
    
    pub fn new(s1: u64, s2: u64) -> Self
    {
        let state   = [s1, s2];
        let mut rng = Self{state};
        rng.generate();
        rng
    }    
    
    pub fn from_utc_time() -> Self
    {
        let mut small_rng = Xorshift64::from_utc_time();
        let s1            = small_rng.generate();
        let s2            = small_rng.generate();

        let mut rng = Self { state: [s1, s2] };
        rng.generate();
        rng
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_os() -> Self
    {
        let mut small_rng = Xorshift64::from_os();
        let s1            = small_rng.generate();
        let s2            = small_rng.generate();

        let mut rng = Self { state: [s1, s2] };
        rng.generate();
        rng 
    }
}
impl Xoshiro256p
{
    pub fn new(s1: u64, s2: u64, s3: u64, s4: u64) -> Self
    {
        let state   = [s1, s2, s3, s4];
        let mut rng = Self{state};

        rng.generate();
        rng
    }    
    pub fn from_utc() -> Self
    {
        let mut small_rng = Xorshift64::from_utc_time();
        let s1 = small_rng.generate();
        let s2 = small_rng.generate();
        let s3 = small_rng.generate();
        let s4 = small_rng.generate(); 
        let state   = [s1, s2, s3, s4];
        let mut rng = Self{state};

        rng.generate();
        rng 
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_os() -> Self
    {
        let mut small_rng = Xorshift64::from_os();
        let s1 = small_rng.generate();
        let s2 = small_rng.generate();
        let s3 = small_rng.generate();
        let s4 = small_rng.generate(); 
        let state   = [s1, s2, s3, s4];
        let mut rng = Self{state};

        rng.generate();
        rng 
    }
}
impl Xoshiro256pp
{
    
    pub fn new(s1: u64, s2: u64, s3: u64, s4: u64) -> Self
    {
        let state   = [s1, s2, s3, s4];
        let mut rng = Self{state};

        rng.generate();
        rng
    }    
    pub fn from_utc() -> Self
    {
        let mut small_rng = Xorshift64::from_utc_time();
        let s1 = small_rng.generate();
        let s2 = small_rng.generate();
        let s3 = small_rng.generate();
        let s4 = small_rng.generate(); 
        let state   = [s1, s2, s3, s4];
        let mut rng = Self{state};

        rng.generate();
        rng 
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_os() -> Self
    {
        let mut small_rng = Xorshift64::from_os();
        let s1 = small_rng.generate();
        let s2 = small_rng.generate();
        let s3 = small_rng.generate();
        let s4 = small_rng.generate(); 
        let state   = [s1, s2, s3, s4];
        let mut rng = Self{state};

        rng.generate();
        rng 
    }
}

// Trait implementations for Array & Montecarlo:
////////////////////////////////////// 
// / TODO: Use macros to implement them automatically
impl ArrayRngInterface for Xorshift64
{
    fn generate_rand_i32(&mut self, low: i32, high: i32) -> i32 {
        self.generate_rand_u32(low as u32, high as u32) as i32
    }    
}

impl MonteCarloRngInterface<f32> for Xorshift64
{
    fn generate_rand_float(&mut self, low: f32, high: f32) -> f32 {
        low + (high-low) * self.rand_f32()
    }    
}
///////////////////////////
impl ArrayRngInterface for Xoroshiro128p
{
    fn generate_rand_i32(&mut self, low: i32, high: i32) -> i32 {
        self.generate_rand_u32(low as u32, high as u32) as i32
    }    
}
impl MonteCarloRngInterface<f32> for Xoroshiro128p
{
    fn generate_rand_float(&mut self, low: f32, high: f32) -> f32 {
        low + (high-low) * self.rand_f32()
    }    
}
/////////////////

impl ArrayRngInterface for Xoshiro256p
{
    fn generate_rand_i32(&mut self, low: i32, high: i32) -> i32 {
        self.generate_rand_u32(low as u32, high as u32) as i32
    }    
}
impl MonteCarloRngInterface<f32> for Xoshiro256p
{
    fn generate_rand_float(&mut self, low: f32, high: f32) -> f32 {
        low + (high-low) * self.rand_f32()
    }    
}
impl MonteCarloRngInterface<f64> for Xoshiro256p
{
    fn generate_rand_float(&mut self, low: f64, high: f64) -> f64 {
        low + (high-low) * self.rand_f64()
    }    
}
/////////////////////
impl ArrayRngInterface for Xoshiro256pp
{
    fn generate_rand_i32(&mut self, low: i32, high: i32) -> i32 {
        self.generate_rand_u32(low as u32, high as u32) as i32
    }    
}
impl MonteCarloRngInterface<f32> for Xoshiro256pp  
{
    fn generate_rand_float(&mut self, low: f32, high: f32) -> f32 {
        low + (high-low) * self.rand_f32()
    }
}
impl MonteCarloRngInterface<f64> for Xoshiro256pp  
{
    fn generate_rand_float(&mut self, low: f64, high: f64) -> f64 {
        low + (high-low) * self.rand_f64()
    }
}
 

#[cfg(test)]
mod tests {

    use super::*;
    const TEST_NUMS: usize = 25;

    #[test]
    fn test_float_xorshift64() 
    {
        let mut rng = Xorshift64::new(42);
        let start   = 1_f32;
        let end     = 2_f32; 
        for _ in 0..TEST_NUMS
        {
            let float = rng.generate_rand_float(start, end);
            println!("float = {float}");
            assert!(float > start && float < end, "float between {start} & {end}");
        }
    }

    #[test]
    fn test_int_xorshift64() 
    {
        let mut rng = Xorshift64::new(42);
        let start   = 0;
        let end     = 10; 
        for _ in 0..TEST_NUMS
        {
            let num = rng.generate_rand_i32(start, end);
            println!("num = {num}");
            assert!(num >= start && num < end, "num between {start} & {end}");
        }
    }


    #[test]
    fn test_float_xoroshiro128p() 
    {
        let mut rng = Xoroshiro128p::new(12, 42);
        let start   = 1_f32;
        let end     = 2_f32; 
        for _ in 0..TEST_NUMS
        {
            let float = rng.generate_rand_float(start, end);
            println!("float = {float}");
            assert!(float > start && float < end, "float between {start} & {end}");
        }
    }

    #[test]
    fn test_int_xoroshiro128p() 
    {
        let mut rng = Xoroshiro128p::new(12, 42);
        let start   = 0;
        let end     = 10; 
        for _ in 0..TEST_NUMS
        {
            let num = rng.generate_rand_i32(start, end);
            println!("num = {num}");
            assert!(num >= start && num < end, "num between {start} & {end}");
        }
    }

    
    #[test]
    fn test_float_xoshiro256p() 
    {
        let mut rng = Xoshiro256p::new(12, 42, 123, 12345);
        let start   = 1_f32;
        let end     = 2_f32; 
        for _ in 0..TEST_NUMS
        {
            let float = rng.generate_rand_float(start, end);
            println!("float = {float}");
            assert!(float > start && float < end, "float between {start} & {end}");
        }
    }

    #[test]
    fn test_int_xoshiro256p() 
    {
        let mut rng = Xoshiro256p::new(12, 42, 123, 12345);
        let start   = 0;
        let end     = 10; 
        for _ in 0..TEST_NUMS
        {
            let num = rng.generate_rand_i32(start, end);
            println!("num = {num}");
            assert!(num >= start, "num >= {start}");
            assert!(num < end, "num < {end}");

        }
    }

        #[test]
    fn test_float_xoshiro256pp() 
    {
        let mut rng = Xoshiro256pp::from_utc();
        let start   = 1_f32;
        let end     = 2_f32; 
        for _ in 0..TEST_NUMS
        {
            let float = rng.generate_rand_float(start, end);
            println!("float = {float}");
            assert!(float > start && float < end, "float between {start} & {end}");
        }
    }

    #[test]
    fn test_int_xoshiro256pp() 
    {
        let mut rng = Xoshiro256pp::from_utc();
        let start   = 0;
        let end     = 10; 
        for _ in 0..TEST_NUMS
        {
            let num = rng.generate_rand_i32(start, end);
            println!("num = {num}");
            assert!(num >= start, "num >= {start}");
            assert!(num < end, "num < {end}");

        }
    }
}

 