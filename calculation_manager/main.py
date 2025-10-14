import numpy as np
from physsm.rust_builder import RustExperimentBuilder, RustExperiment
from physsm.experiment_output import ExperimentOutput 
from typing import override
from pathlib import Path

class IsingData(ExperimentOutput):
    def __init__(self, file_name):
        super().__init__(file_name)

        self.temperatures       = []
        self.energy_density     = []
        self.magnetisation      = []
        self.specific_heat      = []
        self.mag_susceptibility = []
        self.elapsed_time       = -1
        self.correlation_length = []

    @override
    def parse_output(self, line_number: int, line: str):
        if line_number == 0:
            slines = line.split(':')
            try:
                self.observables  = slines[0].split(', ')
                self.elapsed_time = float(slines[1])
            except Exception as _:
                self.observables = line.split(',')
                print("No elasped time found.")
        else:
            slines = line.split(", ")
            self.temperatures.append(float(slines[0]))
            self.energy_density.append(float(slines[1]))
            self.magnetisation.append(float(slines[2]))
            self.specific_heat.append(float(slines[3]))
            self.mag_susceptibility.append(float(slines[4]))
            self.correlation_length.append(float(slines[5]))
                       

class RustIsingExperimentCreator:
    
    def __init__(self, folder: str, name: str):
        self.proj_dir      = Path.cwd().parent / "ising_calculation"
        self.builder       = RustExperimentBuilder(self.proj_dir, folder, name)
        self.builder.set_output_type(IsingData)
        self.builder.set_scale_variable_names(["Lx", "Ly"])
        
    def new_from_parameters(self, therm_steps: dict, measure_steps: dict, temperatures: np.ndarray, measure_corr_length: bool = False) -> RustExperiment:
        
        cargo_toml_path = self.proj_dir  / "Cargo.toml"
        self.builder.set_cargo_toml_path(cargo_toml_path)
        self.builder.add_static_parameter("temperatures", temperatures)
        self.builder.add_static_parameter("measure_corr_len", measure_corr_length)
        self.builder.add_scaling_parameter("therm_steps", therm_steps)
        self.builder.add_scaling_parameter("measure_steps", measure_steps)
        return self.builder.build()
    
    def load(self, lengths: list[int]) -> RustExperiment:
        self.builder.set_scale_variables(lengths)
        return self.builder.build(load_only=True)

def get_lengths(exp: RustExperiment) -> list[int]:
    return exp.get_scale_variables()

def get_results(exp: RustExperiment) -> dict[int, IsingData]:
    return exp.get_results()

    
    
if __name__ == "__main":
    pass  

 