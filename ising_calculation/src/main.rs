#![allow(non_snake_case)]
use ising_calculation::{self, MonteCarloResults, perform_metropolis_computation_parallel};
use ising_calculation::ExperimentParam;
use rayon;
use std::env;
use parameter_reader::ParameterReader;
use num::{Zero};


const J: f64                        = 1_f64; 
const EXTERN_MAG: f64               = 0_f64; 
const MINIMUM_TEMP: f64             = 1E-6;
const PARAMETERS: [&'static str; 7] = [
    "Lx",
    "Ly", 
    "temperatures",
    "measure_corr_len",
    "therm_steps", 
    "measure_steps",
    "outputfile"
];
// use std::fs;
fn main() 
{
    let args: Vec<String> = env::args().collect();
    if args.len() < 2
    {
        println!("Usage: Cargo run --release -- parameter_file.txt");
        std::process::exit(1);
    }

    let reader = ParameterReader::build(&args[1], &PARAMETERS).unwrap_or_else(|e|
    {   
        println!("Failed to create parameter reader: {e}");
        std::process::exit(1);
    });
    let params = reader.parse_parameters(":").unwrap_or_else(|err|
    {
        println!("Failed to read parameters: {err}");
        std::process::exit(1);
    });

    let Lx: usize                   = params["Lx"].parse().expect("!! Could not parse \"cols\"");
    let Ly: usize                   = params["Ly"].parse().expect("!! Could not parse \"rows\"");
    let thermalisation_steps: usize = params["therm_steps"].parse().expect("!! Could not parse \"therm_steps\"");
    let measurement_steps: usize    = params["measure_steps"].parse().expect("!! Could not parse \"measure_steps\"");
    let outputfile: String          = params["outputfile"].parse().expect("!! Could not parse \"outputfile\"");
    let measure_corr_len: bool      = params["measure_corr_len"].to_lowercase().parse().expect("!! Could not parse structur factor");
    
    let mut temperatures: Vec<f64>  = params["temperatures"].split(", ").map(|t| t.parse().expect("!! failed to parse \"temperatures\"") ).collect();

    temperatures
        .iter_mut()
        .filter(|t| t.is_zero() || t.is_nan() || t.is_sign_negative())
        .for_each(|t|
        {
            println!("Setting temperature t={t} => {MINIMUM_TEMP}");
            *t = MINIMUM_TEMP;
        });
    

    println!("Launching 2D Isig with the Metropolis algorithm for N:{Lx}x{Ly} with therm steps {thermalisation_steps} & measure_steps: {measurement_steps}");
    let &temp_last  = temperatures.last().unwrap();
    let &temp_first = temperatures.first().unwrap();
    let temp_len    = temperatures.len();
    println!("Using: {temp_len} temperatures values from {temp_first} to {temp_last} & {} threads", rayon::current_num_threads());

    let parameters  = ExperimentParam 
    {
        temperatures, 
        extern_mag:             EXTERN_MAG,
        interaction_term:       J, 
        thermalisation_steps, 
        measurement_steps,
        measure_struct_fact: measure_corr_len // we need the structur factor, related to the fourier transform of the spin to get the correlation length!
    };
    
    let now     = std::time::SystemTime::now();
    let results = perform_metropolis_computation_parallel::<i8,f64>(Ly, Lx, &parameters).unwrap_or_else(|e|      // We will use i8 spins and f64 observables:
    {
        println!("Could not perform metropolis computation: {e:?}");
        std::process::exit(1);
    });
    let elapsed_time: std::time::Duration = now.elapsed().unwrap(); 


    println!("Calculation finished after {}s", elapsed_time.as_secs());
    println!("Saving result as \"{outputfile}\".");
    

    MonteCarloResults::write_to_file(&outputfile, &parameters.temperatures, &results, Ly, Lx, elapsed_time).unwrap_or_else(|e|
    {
        println!("Could not write to file: {e}.");
        std::process::exit(1);
    });
    

}
 