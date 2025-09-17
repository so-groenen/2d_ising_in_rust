

mod real_time_data_handler;
mod spin_img;

// mod xorshifts;
 

// Different fast XOR-shift based RNGs to test performance for WASM.

#[allow(unused_imports)]
use xorshifts::Xoshiro256p;    // Cousin of the "SmallRng" of the rand crate (which is Xoshiro256++).
#[allow(unused_imports)]
use xorshifts::Xoroshiro128p;  // Lightweight & fast alternative, good enough for wasm
          

use spin_img::SpinImage;

use real_time_data_handler::RealTimeDataHandler;
use monte_carlo_lib::{ising_state, metropolis, SpinEnergyFluctuation};
use periodic_array_2d_lib::{PeriodicArray2D};

 

const MIN_TEMP: f32        = 0_f32;
const MAX_TEMP: f32        = 5_f32;
const MIN_MAG: f32         = -0.5_f32;
const MAX_MAG: f32         = 0.5_f32;
const DELTA_TEMP: f32      = 0.05_f32;
const DELTA_MAG: f32       = 0.05_f32;
const INTERATION_TERM: f32 = 1_f32;

const COLUMNS: i32  = 512;  
const ROWS:    i32  = 512;  

pub struct IsingSimulation
{
    spin_img: SpinImage,
    data_handler: RealTimeDataHandler,
    spin_2d: PeriodicArray2D<i8,f32>,
    my_rng: Xoshiro256p,
    spin_total: f32,
    magnetization: f32,
    temperature: f32,
    extern_mag: f32,
}

#[cfg(not(target_arch = "wasm32"))]
fn new_rng() -> Xoshiro256p
{
    Xoshiro256p::from_os()
}
#[cfg(target_arch = "wasm32")]
fn new_rng() -> Xoshiro256p
{
    Xoshiro256p::from_utc()
}


impl IsingSimulation 
{
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self
    {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        // let mut my_rng = if cfg!(target_arch = "wasm32")
        // {
        //     Xoshiro256p::from_utc()
        // }
        // else
        // {
        //     Xoshiro256p::from_os()
        // };

        let mut my_rng    = new_rng();
        let generator     = ||ising_state::thermal_state::<i8, Xoshiro256p>(&mut my_rng);
        let spin_2d       = PeriodicArray2D::new_with(ROWS, COLUMNS, generator).expect("Rows && Cols > 0");
        let spin_img      = SpinImage::new(&spin_2d, &cc.egui_ctx);

        let spin_total    = spin_2d.sum_observable();
        let magnetization = spin_total / spin_2d.total_number() as f32;
        let n_points      = 40_usize;  
        let n_frame_avg   = 25_usize;    
        let data_handler  = RealTimeDataHandler::new(n_points, n_frame_avg); 

        Self 
        {
            spin_img,
            data_handler,
            spin_2d,
            my_rng,
            spin_total,
            magnetization,
            temperature: 2_f32,
            extern_mag: 0_f32
        }
    }


    fn update_physics(&mut self)
    {
        let SpinEnergyFluctuation(delta_spin,_) = metropolis::perform_metropolis_sweep(&mut self.spin_2d,
                                                                                       &mut self.my_rng,
                                                                                       self.temperature,
                                                                                       INTERATION_TERM,
                                                                                       -self.extern_mag); // minus sign for visualization!!!
        self.spin_total   += delta_spin;
        self.magnetization = self.spin_total / self.spin_2d.total_number() as f32;
    }
    fn update_data(&mut self)
    {
        self.data_handler.append_and_update(self.magnetization);
    }
    fn update_spin_image(&mut self)
    {
        self.spin_img.update_image(&self.spin_2d);
    }
    fn draw_and_handle_control_panel_ui(&mut self, ctx: &egui::Context)  
    {
        let dt  = ctx.input(|i| i.stable_dt);
        let fps = 1.0 / dt;

        let temp_range = MIN_TEMP..=MAX_TEMP; 
        let mag_range  = MIN_MAG..=MAX_MAG; 
        egui::SidePanel::left("left panel").show(ctx, |ui|
        {
            ui.label(format!("FPS: {fps:.2}"));
            ui.heading( format!("2D Ising: {}x{} Metropolis algorithm", ROWS, COLUMNS));
            ui.separator();
            ui.heading(format!("Temperature: {:.2}", self.temperature));
            ui.horizontal(|ui: &mut egui::Ui|
            {
                if ui.button("[-]").clicked() && self.temperature > *temp_range.start()
                {
                    self.temperature -= DELTA_TEMP
                }       
                ui.add( egui::Slider::new(&mut self.temperature, temp_range.clone()).show_value(false));       
                if ui.button("[+]").clicked() && self.temperature < temp_range.clone().end() -DELTA_TEMP
                {
                    self.temperature += DELTA_TEMP
                }
            });

            ui.separator();
            ui.heading(format!("External magnetic field: {:.2}", self.extern_mag));
            ui.vertical(|ui|
            {
                ui.horizontal(|ui: &mut egui::Ui|
                {
                    if ui.button("[-]").clicked() && self.extern_mag > *mag_range.start()
                    {
                        self.extern_mag -= DELTA_MAG;    
                    }       
                    ui.add( egui::Slider::new(&mut self.extern_mag, mag_range.clone()).show_value(false));       
                    if ui.button("[+]").clicked() && self.extern_mag < mag_range.clone().end() - DELTA_MAG
                    {
                        self.extern_mag += DELTA_MAG;               
                    }
                });
                ui.vertical_centered_justified(|ui|
                {
                    if ui.button("Reset").clicked()
                    {
                        self.extern_mag = 0_f32;                 
                    }
                })
            });
            
            ui.separator();
            ui.heading(format!("Magnetization: {:.2}", self.magnetization));
            ui.separator();
            self.data_handler.plot_data(ui, "Time", "<S>", -1.1, 1.1);
        });
    }

    fn draw_spins(&self, ctx: &egui::Context)  
    {
        egui::CentralPanel::default()
        .show(ctx, |ui|
        {
            self.spin_img.draw_on_ui(ui);
        });
    }
}

impl eframe::App for IsingSimulation 
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        ctx.request_repaint();

        self.update_physics();
        self.update_data();
        self.update_spin_image();
        self.draw_and_handle_control_panel_ui(ctx);
        self.draw_spins(ctx);
    }
}










 