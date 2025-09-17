use ising_calculation::{self, arange, MonteCarloResults, perform_metropolis_computation_parallel};
use ising_calculation::ExperimentParam;
use rayon;
use std::env;

const J: f64          = 1_f64; 
const EXTERN_MAG: f64 = 0_f64; 

 
fn main() 
{
    let usage = "ERROR: Usage example: \"cargo run ROWS COLUMNS THERM_STEPS MEASURE_STEPS\" where all are ints";
    let args: Vec<String> = env::args().collect();
    if args.len() < 3
    {
        println!("{usage}");
        std::process::exit(1);
    }
    let rows: i32                   = args[1].parse().expect(usage);
    let columns: i32                = args[2].parse().expect(usage);
    let thermalisation_steps: usize = args[3].parse().expect(usage);
    let measurement_steps: usize    = args[4].parse().expect(usage);

    // Zoom around the critical temps Tc ~ 2.269
    let temp_start = 1.9_f64; 
    let step       = 0.02f64; 

    let temp_critical_start = 2.1_f64; 
    let temp_critical_stop  = 2.5_f64; 
    let step_critical       = 0.01_f64; 
    let temp_stop           = 2.78_f64;
    
    
    let mut temperatures = arange(temp_start, temp_critical_start, step).unwrap();
    let mut temp2        = arange(temp_critical_start, temp_critical_stop, step_critical).unwrap();
    let mut temp3        = arange(temp_critical_stop, temp_stop, step).unwrap();
    temperatures.append(&mut temp2);
    temperatures.append(&mut temp3);


    let param  = ExperimentParam 
    {
        temperatures, 
        extern_mag:             EXTERN_MAG,
        interaction_term:       J, 
        steps_between_measures: 1, 
        thermalisation_steps, 
        measurement_steps 
    };

    println!("Computing magnetization & energy density & energy/spin fluctuations for {rows}x{columns} spins with temp from {temp_start:.2} to {:.2} J/kB", param.temperatures.last().unwrap());
    println!("N values: {}", param.temperatures.len());
    println!("Num cores: {}", rayon::current_num_threads());
    
    let now     = std::time::SystemTime::now();
    // We will use i8 spins and f64 observables:
    let results = perform_metropolis_computation_parallel::<i8,f64>(rows, columns, &param).unwrap_or_else(|e| 
    {
        println!("Could not perform metropolis computation: {e:?}");
        std::process::exit(1);
    });
    let elapsed_time: std::time::Duration = now.elapsed().unwrap(); 


    println!("Calculation finished after {}s", elapsed_time.as_secs());

    let file_name = format!("results/montecarlo_parallel_{rows}x{columns}_temp.txt");
    println!("Saving result as \"{file_name}\".");
    
    let num_spins = (rows * columns) as usize;

    MonteCarloResults::write_to_file(&file_name, &param.temperatures, &results, num_spins, elapsed_time).unwrap_or_else(|e|
    {
        println!("Could not write to file: {e}.");
        std::process::exit(1);
    });
    

}
 