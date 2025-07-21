use eframe::egui;
use monte_carlo_lib::{metropolis, ising_state};
use periodic_array_2d_lib::PeriodicArray2D;
 
type Rng = os_based_rng::OsBasedRng;
 

const ROWS: i32             = 200;
const COLS: i32             = 200;
const HALF: i32             = ROWS/2;
const INTERACTION_TERM: f32 = -1f32;
const TEMP_CRITICAL: f32    = 2.269f32;


pub struct MyApp
{
    ising_2d: PeriodicArray2D,
    temperature: f32,
    magnetization: f32,
    my_rng: Rng,
}

impl Default for MyApp
{
    fn default() -> Self 
    {
        let mut my_rng = Rng::new();
        let thermal_state    = ||ising_state::thermal_state(&mut my_rng);
        let ising_2d= match PeriodicArray2D::new_with(ROWS, COLS, thermal_state)
        {
            Ok(array) => array,
            Err(e) => 
            {
                eprintln!("Error creating PeriodicArray: {e}");
                std::process::exit(1);
            }
        };

        MyApp
        {
            ising_2d,
            temperature: 3f32,
            magnetization: 0f32,
            my_rng
        }
    }
    
}

impl MyApp
{
    pub fn new(cc: &eframe::CreationContext) -> MyApp
    {
        cc.egui_ctx.set_theme(egui::Theme::Dark);

        Default::default()
         
    }    
    fn perform_monte_carlo(&mut self)
    {
        metropolis::perform_metropolis_sweep(&mut self.ising_2d, &mut self.my_rng, self.temperature, INTERACTION_TERM);
    }
}

impl eframe::App for MyApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) 
    {
        ctx.request_repaint();
        self.perform_monte_carlo();

        let range = 0f32..=5f32; 

        egui::SidePanel::left("left panel").show(ctx, |ui|
        {
            let dt = ctx.input(|i|i.stable_dt);
            ui.small(format!("FPS: {:.1}", 1.0/dt));
            ui.heading( format!("2D Ising: {}x{} Metropolis algorithm", ROWS, COLS));
            ui.separator();
            ui.label("Temperature:");
            // ui.heading(format!("Temperature [critial temp: {:.3}]:", TEMP_CRITICAL));
            ui.add( egui::Slider::new(&mut self.temperature, range).show_value(true));                
            ui.separator();
            if ui.button("Go to critial temperature").clicked()
            {
                self.temperature = TEMP_CRITICAL;    
            }
            if ui.button("Magnetic pulse [+]").clicked()
            {
                self.ising_2d.reset(||ising_state::SPINUP);               
            }
            if ui.button("Magnetic pulse [-]").clicked()
            {
                self.ising_2d.reset(||ising_state::SPINDOWN);               
            }

            ui.separator();
            ui.heading(format!("Magnetization: {:.2}", self.magnetization))
        });

        egui::CentralPanel::default().show(ctx, |ui|
        {
            // Credit for the drawing : https://github.com/StefanSalewski/tiny-chess/
            let mut spin_total = 0f32;
            let available_size = ui.available_size();
            let central_panel_rect = ui.min_rect();
            let center_x = central_panel_rect.center().x;
            let center_y = central_panel_rect.center().y;
            let array_size = available_size.min_elem();
            let square_size = array_size / ROWS as f32;
            let array_top_left = egui::Pos2 
            {
                x: center_x - (HALF as f32 * square_size),
                y: center_y - (HALF as f32 * square_size)
            };
            let painter = ui.painter();
            for i in self.ising_2d.rows_range()
            {
                for j in self.ising_2d.columns_range()
                {
                    let top_left = egui::Pos2 
                    {
                        x: array_top_left.x + (j as f32 * square_size),
                        y: array_top_left.y + (i as f32 * square_size),
                    };
                    let bottom_right = egui::Pos2 
                    {
                        x: top_left.x + square_size,
                        y: top_left.y + square_size,
                    };
                    let spin: f32 = self.ising_2d.at_unchecked(i, j);
                    let color = match spin
                    {
                        ising_state::SPINUP   => egui::Color32::from_rgb(120, 185, 181), //Bright Blue
                        ising_state::SPINDOWN => egui::Color32::from_rgb(50, 10, 107),  // Dark Blue
                        _                     => egui::Color32::from_rgb(0,0,0)  // black; for debug
                    };
                    spin_total   += spin;
                    let rect = egui::Rect::from_two_pos(top_left, bottom_right);
                    painter.rect_filled(rect, 0.0, color);
                }    
            }
            self.magnetization = spin_total / self.ising_2d.total_number() as f32;
        });
    }    
}
