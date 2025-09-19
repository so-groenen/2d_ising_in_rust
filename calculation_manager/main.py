import numpy as np
from python_simulation_manager.experiment_base.experiment_handler import ExperimentHandler
from python_simulation_manager.experiment_base.experiment_output import ExperimentOutput
from typing import override
import subprocess

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
    def parse_output(self, line_number, lines):
        if line_number == 0:
            slines = lines.split(':')
            try:
                self.observables  = slines[0].split(', ')
                self.elapsed_time = float(slines[1])
            except Exception as _:
                self.observables = lines.split(',')
                print("No elasped time found.")
        else:
            slines = lines.split(", ")
            self.temperatures.append(float(slines[0]))
            self.energy_density.append(float(slines[1]))
            self.magnetisation.append(float(slines[2]))
            self.specific_heat.append(float(slines[3]))
            self.mag_susceptibility.append(float(slines[4]))
            self.correlation_length.append(float(slines[5]))
                       
class RustIsingExperiment(ExperimentHandler):

    @override
    def set_result_type(self, output_file) -> ExperimentOutput:
        return IsingData(output_file)
   
    @override
    def perform_rust_computation(self, L):
        command  = f"cargo run --release --  {self.get_parameter_file_relative(L)}"
        cwd      = self.get_rust_dir()
        time     = -1
        print(f"Command: \"{command}\"")   
        with subprocess.Popen(command, stdout=subprocess.PIPE, bufsize=1, text=True, stderr=subprocess.STDOUT, cwd=cwd) as stream:
            for line in stream.stdout:
                if line.__contains__("Time taken: "):
                    time_str = line.split("Time taken: ")[1]
                    time     = int(time_str.removesuffix("s\n"))
                print(f" * From Rust: {line}", end='') 
        return time
    
    def should_measure_correlation_length(self, val: bool):
        self.set_static_parameter("measure_corr_len", val)
    
class RustIsingExperimentBuilder:
    def __init__(self, name: str, folder: str, rust_dir: str):
        self.name: str     = name
        self.folder: str   = folder
        self.rust_dir: str = rust_dir

    def new_from_parameters(self, therm_steps: dict, measure_steps: dict, temperatures: np.ndarray, measure_corr_length: bool = False) -> RustIsingExperiment:
     
        new_exp = RustIsingExperiment(self.name, self.folder, self.rust_dir)
        new_exp.set_scaling_name(["Lx", "Ly"])
        new_exp.add_static_parameter("temperatures", temperatures)
        new_exp.add_static_parameter("measure_corr_len", measure_corr_length)
        new_exp.add_scaling_parameter("therm_steps", therm_steps)
        new_exp.add_scaling_parameter("measure_steps", measure_steps)
        return new_exp
    
    def load(self, lengths: list[int]) -> RustIsingExperiment:
        new_exp = RustIsingExperiment(self.name, self.folder, self.rust_dir, lengths)
        return new_exp

    
    
if __name__ == "__main":
    None
 