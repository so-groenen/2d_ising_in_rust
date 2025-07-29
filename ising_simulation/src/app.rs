
use macroquad::prelude::*;
use monte_carlo_lib::{ising_state, metropolis};
use periodic_array_2d_lib::{PeriodicArray2D};
use egui_macroquad::egui::{self, epaint::image};

mod texture_finder;
mod rng;
mod real_time_data_handler;

use rng::MyRNG;
use real_time_data_handler::RealTimeDataHandler;

const COLUMNS: i32  = 512;  
const ROWS:    i32  = 512;
// const VIRTUAL_SQR_SIZE: f32 = 2f32;     // This influcences the quality & performance since we are drawing on a (VIRTUAL_SQR_SIZE*ROWS)² pixel texture,
                                        // ...and rendering that texture on a larger eGui UI. Larger VIRTUAL_SQR_SIZE => more quality / less speed.
// const VIRTUAL_WIDTH: f32      = VIRTUAL_SQR_SIZE * (COLUMNS as f32);
// const VIRTUAL_HEIGHT: f32     = VIRTUAL_SQR_SIZE * (ROWS as f32);
const MY_LIGHT_BLUE: Color    = color_u8!(120, 185, 181, 255 );
const MY_DARK_BLUE: Color     = color_u8!(50, 10, 107, 255 );
const INTERATION_TERM: f32 = -1f32;


pub struct Simulation
{
    my_render_target: RenderTarget,
    // render_target_cam: Camera2D, 
    egui_texture: egui::TextureId,
    data_handler: RealTimeDataHandler,
    spin_img: Image,

    spin_2d: PeriodicArray2D,
    my_rng: MyRNG,
    spin_total: f32,
    magnetization: f32,
    temperature: f32,
    extern_mag: f32,
}
impl Simulation
{
    pub fn new() -> Self
    {   
        let my_render_target = macroquad::texture::render_target(COLUMNS as u32, ROWS as u32);
        // let my_render_target = macroquad::texture::render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
        let Some(texture_id) = texture_finder::get_raw_opengl_texture_id_from_framebuffer(&my_render_target) else 
        {
            panic!("No texture found: cannot continue.");
        };
        let egui_texture = egui::TextureId::User(texture_id as u64);
        
        // let mut render_target_cam = Camera2D::from_display_rect(Rect::new(0., 0., VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
        // render_target_cam.render_target = Some(my_render_target.clone());

        let mut my_rng = MyRNG::new();
        let generator   = ||ising_state::thermal_state(&mut my_rng);
        let spin_2d = PeriodicArray2D::new_with(ROWS, COLUMNS, generator).expect("Rows && Cols > 0");
        let spin_total: f32 = spin_2d.sum() as f32;
        let magnetization: f32 = spin_total / spin_2d.total_number() as f32;
        let spin_img = Image::gen_image_color(COLUMNS as u16, ROWS as u16, BLACK);
        
        let n_points: usize = 40;  
        let n_frame_avg: usize = 25;    
        let data_handler: RealTimeDataHandler = RealTimeDataHandler::new(n_points, n_frame_avg); //rename to "RealTimePloter?"

        Self {
            my_render_target,
            // render_target_cam,
            egui_texture,
            data_handler,
            spin_img,
            spin_2d,
            my_rng,
            spin_total,
            magnetization,
            temperature: 2f32,
            extern_mag: 0f32
        }
    }    

    fn update_physics(&mut self)
    {
        let mut delta_spin = 0f32;
        for _ in self.spin_2d.all_range()
        {
            delta_spin += metropolis::perform_metropolis_proposal(&mut self.spin_2d, &mut self.my_rng, self.temperature, INTERATION_TERM, self.extern_mag);
        }
        self.spin_total += delta_spin;
        self.magnetization = self.spin_total / self.spin_2d.total_number() as f32;
    }
    fn update_data(&mut self)
    {
        self.data_handler.append_and_update(self.magnetization);
    }
    fn draw_spins_on_render_target(&mut self)
    {
        // set_camera(&self.render_target_cam);
            // clear_background(WHITE);
            for i in self.spin_2d.rows_range()
            {
                for j in self.spin_2d.columns_range()
                {
                    // let x = j as f32 * VIRTUAL_SQR_SIZE;
                    // let y = i as f32 * VIRTUAL_SQR_SIZE;
                    
                    let spin_color = match self.spin_2d.at_unchecked(i, j) 
                    {
                        ising_state::SPINUP   => MY_LIGHT_BLUE,
                        ising_state::SPINDOWN => MY_DARK_BLUE,
                        _ => WHITE    
                    };
                    // draw_rectangle(x, y, VIRTUAL_SQR_SIZE, VIRTUAL_SQR_SIZE, spin_color);
                    self.spin_img.set_pixel(j as u32, i as u32, spin_color);
                }
            }
        // set_default_camera();
        self.my_render_target.texture.update(&self.spin_img);
    }

    fn draw_and_handle_control_panel_ui(&mut self, ctx: &egui::Context)  
    {
        let fps = get_fps();
        let delta_temp: f32 = 0.05f32;
        let delta_mag: f32 = 0.05f32;

        let temp_range = 0f32..=5f32; 
        let mag_range = -0.5f32..=0.5f32; 
        egui::SidePanel::left("left panel").show(ctx, |ui|
        {
            // let temp_range = 0f32..=5f32; 
            ui.label(format!("FPS: {fps}"));
            ui.heading( format!("2D Ising: {}x{} Metropolis algorithm", ROWS, COLUMNS));
            ui.separator();
            ui.heading(format!("Temperature: {:.2}", self.temperature));
            ui.horizontal(|ui: &mut egui::Ui|
            {
                if ui.button("[-]").clicked() && self.temperature > *temp_range.start()
                {
                    self.temperature -= delta_temp
                }       
                ui.add( egui::Slider::new(&mut self.temperature, temp_range.clone()).show_value(false));       
                if ui.button("[+]").clicked() && self.temperature < temp_range.clone().end() -delta_temp
                {
                    self.temperature += delta_temp
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
                        self.extern_mag -= delta_mag;    
                    }       
                    ui.add( egui::Slider::new(&mut self.extern_mag, mag_range.clone()).show_value(false));       
                    if ui.button("[+]").clicked() && self.extern_mag < mag_range.clone().end() - delta_mag
                    {
                        self.extern_mag += delta_mag;               
                    }
                });
                ui.vertical_centered_justified(|ui|
                {
                    if ui.button("Reset").clicked()
                    {
                        self.extern_mag = 0f32;                 
                    }
                })
            });
            
            ui.separator();
            ui.heading(format!("Magnetization: {:.2}", self.magnetization));
            ui.separator();
            self.data_handler.plot_data(ui, "Time", "<S>", -1.1, 1.1);
        });
    }

    fn draw_render_target_on_ui(&self, ctx: &egui::Context)  
    {
        egui::CentralPanel::default()
        .show(ctx, |ui|
        {
            let central_panel_rect = ui.min_rect();
            let available_size = ui.available_size();
            let array_size = available_size.min_elem();
            let center_x = central_panel_rect.center().x;
            let center_y: f32 = central_panel_rect.center().y;
            let array_top_left = egui::Pos2 
            {
                x: center_x - 0.5*array_size, 
                y: center_y - 0.5*array_size 
            };
            let rect = egui::Rect::from_min_size(array_top_left, egui::Vec2 { x: array_size, y: array_size });
            let uv = egui::Rect{ min:egui::pos2(0.0, 0.0), max:egui::pos2(1.0, 1.0)};
            ui.painter().image(self.egui_texture, rect, uv, egui::Color32::WHITE)

        });
    }

    pub fn update(&mut self)
    {
        self.update_physics();
        self.update_data();
    }

    pub fn draw_and_handle_ui(&mut self)
    {
        self.draw_spins_on_render_target();

        egui_macroquad::ui(|ctx|
        {
            self.draw_and_handle_control_panel_ui(ctx);
            self.draw_render_target_on_ui(ctx);
        });
        egui_macroquad::draw();        
    }
}
