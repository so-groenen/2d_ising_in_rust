use std::{io::Write, iter::zip};
use ising_calculation;
use ising_calculation::ExperimentParam;

const ROWS: i32              = 2;     // Spin array has periodic boundary condition, I therefor use signed ints rather than usize
const COLUMNS: i32           = 2;
const J: f32                 = -1f32; 
const EXTERN_MAG: f32        = 0f32; 
const NUMBER_OF_SPINS: usize = (ROWS as usize)*(COLUMNS as usize);

fn main() 
{
    let temp_start = 1f32;
    let temp_stop  = 3f32;
    let step       = 0.05f32;
    let param: ExperimentParam = ExperimentParam 
    {
        temperatures:           ising_calculation::create_temp_vector(temp_start, temp_stop, step),
        extern_mag:             EXTERN_MAG,
        interaction_term:       J, 
        steps_between_measures: NUMBER_OF_SPINS, 
        thermalisation_steps:   (NUMBER_OF_SPINS * 1E2 as usize),  //1E6 sweeps accross the lattice
        measurement_steps:      (NUMBER_OF_SPINS * 1E2 as usize)     //1E6 sweeps accross the lattice
    };

    println!("Computing magnetization for {ROWS}x{COLUMNS} spins with temp from {temp_start:.2} to {temp_stop:.2} J/kB");
    let now = std::time::SystemTime::now();
    let magnetization = match ising_calculation::perform_magnetization_computation(ROWS, COLUMNS, &param)
    {
        Ok(v) => v,
        Err(e) =>
        {
            println!("Error {e:?}");
            std::process::exit(1);
        }
    };
    let elapsed_time = now.elapsed().unwrap(); 
    println!("Calculation finished after {}s", elapsed_time.as_secs());

    let file_name      = format!("result/magnetization_{ROWS}x{COLUMNS}_temp_{:.2}_to_{:.2}.txt", param.temperatures.first().unwrap(), param.temperatures.last().unwrap());
    println!("Saving result as \"{file_name}\"");

    let mut file         = std::fs::File::create(file_name).expect("Cannot create file");
    for (t, m) in zip(param.temperatures.iter(), magnetization.iter())
    {
        writeln!(&mut file, "{}, {}", *t, *m).unwrap();
    } 

}
 