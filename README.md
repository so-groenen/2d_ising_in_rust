# 2d Ising Metropolis Simulation in Rust
This Rust project contains a 
* **Real time 2D Ising Simulation**, which uses the Metropolis algorithm to solve 
a $512\times512$ spin system in real time, and display it live using [egui](https://github.com/emilk/egui) & [WebAssembly](https://en.wikipedia.org/wiki/WebAssembly), as well as 
* **Basic calculation of the phase diagram**, meaning: Magnetization, energy density, specific heat, magnetic susceptibilities & correlation length to estimate the critical temperature<br>

### Goals
The main goal of this project, was to learn more about Rust for: 
* Real time graphics/physics simulation,
* Generic programming (using the [num-traits](https://docs.rs/num-traits/latest/num_traits/) crate), 
* Low-level control of memory/data-strucutres: The spins are encoded as `i8` (1byte), the observables (magnetization etc...) as `f32` for the realtime simulation, but `f64` for the phase diagrams.
Being able to run a "perform_computation" for both `f32` & `f64` while maintining type safety (ie, not implicit casting) was very important.
* Low-level implementations of random number generation (Mainly XORshifts, following the guidlines/C implementations of [https://prng.di.unimi.it/](https://prng.di.unimi.it/))
* Multithreading using [Rayon](https://github.com/rayon-rs/rayon) for Monte-Carlo simultions (and possibly bench mark it against C/C++/FORTRAN libs like OpenMP, or Julia's [Polyester](https://github.com/JuliaSIMD/Polyester.jl)).
* Using Python to interact with high-performance/multithreaded Rust programs. (See [Python Simulation Manager](https://github.com/so-groenen/python_simulation_manager))

## Real Time Simulation
Click the image to try out the interactive WebAssembly simulation!<br>
[![Watch the video](docs/assets/screen_shot.png)](https://so-groenen.github.io/2d_ising_in_rust/)
<br>

You can: 
* change the temperature
* change the external magnetic field
* watch the magnetization change

### Usage (Desktop)
First git clone it:
```
git clone https://github.com/so-groenen/2d_ising_in_rust.git 
cd 2d_ising_in_rust
```
Run the simulation using cargo:
```
cargo run --bin ising_simulation --release 
```
### Usage (Web in your browser)
For the WASM, we use [Trunk](https://trunkrs.dev/) to build a .wasm binary & a folder with all you need.<br>
Follow the guidlines in the description of [https://github.com/emilk/eframe_template/](https://github.com/emilk/eframe_template/), starting from the **"Web Locally"** section. 


## Monte-Carlo calculation in Rust 

Using the same data-structure, we can perform some calcuation for linear system sizes $L=[8, 16, 32, 64, 128]$, and total number of spins $N=L\times L$.<br>
Parameters used where:
* $5\times 10^5$ steps both for thermalization & for measurements for $L\in [8, 16, 32, 64]$
* $1\times 10^5$  steps both for thermalization & for measurements for $L=128$
* no external magnetic field
* Usual ferromagnetic interaction strength $J=1$

Here, one step = one sweep across the lattice = $N=L\times L$ metropolis trials.<br>
The reason for this "convention" is to be able to easier compare with cluster algorithms like the Swendsen-Wang algorithm (repo still private: TODO: Add Plots).<br>

* Computations are done by iterating "in parallel" over temperatures using 8 threads and the [Rayon](https://github.com/rayon-rs/rayon) library.
* Parameters are written in Python, and dispatched to Rust as simple files using [Python Simulation Manager](https://github.com/so-groenen/python_simulation_manager). 
Rust calculations can easily be launched from a python notebook, and the result grabbed for plotting.<br>
* The `calculation_manager` Python module serves in a way as a "control center" to handle the Rust calcultions.
A more thorough anlysis/calcuation will be done using the Swendsen-Wang algorithm. Indeed, because observables in the Metropolis case exhibit long auto-correlation times (in "normal people's speak", everything looks similar, and you have to wait some time for the spin configuration to look notably different), it takes more measure to get more meaningful data.<br>
 
<center><img src="ising_calculation/results/overview/overview.png" width="1200"></center>


## Rough estimation of the critical temperature
One way to get the critical temperatures, is to compute the correlation length $\xi$. The correlation length can be computed from the spin-spin correlator/structure factor defined by
$$S(\vec{q}) = \langle\sigma_{\vec{q}}\sigma_{-\vec{q}}\rangle = \sum_{\vec{r}_1,\vec{r}_2}e^{i\vec{q}(\vec{r}_2-\vec{r}_1)}G(\vec{r}_1,\vec{r}_2), \quad \text{where} \quad G(r) \propto e^{-r/\xi}$$


At the critical temperature, the system exhitibs scale invariance, that is $\xi(T_c;\lambda L) = \lambda\xi(T_c;L)$, where $\lambda$ a scale factor.<br> As a result, $\xi(T_c;\lambda L)/L$ is scale invariant and we can estimate $T_c$ from it, using for instance, scipy's spline interpolation: 

<center><img src="ising_calculation/results/critical_temperatures/correlation_lengths.png" width="1024"></center>
<br>
Notice how the results are noisy! This is the tell-tale sign of long auto-correlation times: When averaging over many many data samples, because of the way metropolis works (it is a single-spin algorithm), many data samples
will look "similar", and the algorithm will not be able to sample many different configurations in a short span. This can be mitigated using cluster algorithms like the Swendsen-Wang algorithm,
or the Wolff algorithm, or other types like the Worm algoritm...

### Using the Rust calculation from the Python notebook:

The python module uses [uv](https://docs.astral.sh/uv/) as package manager.
First $cd$ into the `calculation_manager` folder (if you have git-cloned the repo), and then activate the virtual environnement:
```
cd calculation_manager
uv venv
```
Normally, uv will download the Python Simulation Manager, and other dependencies if not present (numpy, matplotlib & subprocess).
Everything is then handled in the notebook. After importing the files in the notebook header we tell the manager where to find the Rust crate, where to put the results, and the parameters. This is handled by the "experiment builder":
```python
rust_dir        = "../ising_calculation"
folder          = "results"
name            = "overview"

builder = RustIsingExperimentBuilder(name=name, folder=folder, rust_dir=rust_dir)
```
Then we create an "experiment" by providing the necessary parameters & write the parameter files where Rust can find them:
```python
lengths                      = [8, 16, 32, 64, 128]
temperatures                 = np.linspace(0.5, 4.45, 80)
(therm_steps, measure_steps) = get_default_monte_carlo_parameters(lengths)

experiment = builder.new_from_parameters(therm_steps=therm_steps, measure_steps=measure_steps, temperatures=temperatures)
experiment.write_parameter_files()
```

We are ready to launch the calculations:
```python
for L in experiment.get_lengths():
    experiment.perform_rust_computation(L)
```    
which will call `cargo run --release -- results/overview/parameter_LxL.txt` under the hood.
We can then get the results (dicts which maps int "L" to classes containing the data):
```python
results = experiment.get_results()
```
