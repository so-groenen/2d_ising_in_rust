import numpy as np
import subprocess

class RustMonteCarloData:
    def __init__(self, file_name):
        self.file_name = file_name
        self.temperatures       = []
        self.energy_density     = []
        self.magnetisation      = []
        self.specific_heat      = []
        self.mag_susceptibility = []
        self.elapsed_time       = -1
        self.observables        = []
        self.compute_observables()
        
    def to_nd_array(self, name):
        my_lists = getattr(self, name)
        
        if not isinstance(my_lists, np.ndarray):            
            setattr(self, name, np.asarray(my_lists))

    def all_lists_to_array(self):
        for name in vars(self).keys():
            if name != "elapsed_time":
                self.to_nd_array(name)

    def does_file_existst(self) -> bool:
        try:
            with open(self.file_name, "r") as file:
                pass 
            return True
        except OSError as e:
            print(f"Error opening file: {e}")
            return False

    def compute_observables(self):
        if not self.does_file_existst():
            return
        with open(self.file_name, "r") as file:
            for (n, lines) in enumerate(file):
                if n == 0:
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
        
        self.all_lists_to_array()
        
def perform_rust_computation(command: str):
    print(f"Command: \"{command}\"")   
    with subprocess.Popen(command, stdout=subprocess.PIPE, bufsize=1, universal_newlines=True, stderr=subprocess.STDOUT) as stream:
        for line in stream.stdout:
            print(f">> {line}", end='') 
        if stream.stderr is not None:
            for line in stream.stdout:
                print(f">> {line}", end='') 
    
        stdout, stderr = stream.communicate()
        if stream.returncode != 0:
            print(stdout)
            print(stderr)
        else:
            print("=> execution successful.")
    print("")
    
if __name__ == "__main__":
    test = RustMonteCarloData("Error no file")