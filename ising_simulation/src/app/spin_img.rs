const MY_LIGHT_BLUE: egui::Color32 = egui::Color32::from_rgb(120, 185, 181);
const MY_DARK_BLUE: egui::Color32  = egui::Color32::from_rgb(50, 10, 107);

use monte_carlo_lib::ising_state;
use periodic_array_2d_lib::PeriodicArray2D;

pub struct SpinImage 
{
    pixel_buffer: Vec<egui::Color32>,
    texture_handle: egui::TextureHandle,
    img_size: [usize; 2]
}

impl SpinImage
{
    pub fn new(spin_2d: &PeriodicArray2D<i8,f32>, egui_ctx: &egui::Context) -> Self
    {
        let rows     = spin_2d.rows() as usize;
        let column   = spin_2d.columns() as usize;
        let img_size = [rows, column];

        let pixel_buffer   = vec![egui::Color32::WHITE; rows*column];
        let texture_handle = egui_ctx.load_texture(
            "spin_image",
            egui::ColorImage::example(),
            Default::default()
        );
        
        let mut spin_img = Self {pixel_buffer, texture_handle, img_size};
        spin_img.update_image(&spin_2d);
        spin_img
    }
    pub fn update_image(&mut self, spin_2d: &PeriodicArray2D<i8,f32>)
    {
        for i in spin_2d.rows_range()
        {
            for j in spin_2d.columns_range()
            {
                let n = i * spin_2d.columns() + j;

                self.pixel_buffer[n as usize] = match spin_2d.at_unchecked(i, j)
                {
                    ising_state::i8_spins::SPINUP   => MY_LIGHT_BLUE,
                    ising_state::i8_spins::SPINDOWN => MY_DARK_BLUE,
                    _ => egui::Color32::WHITE,
                };
            }
        }
        
        self.texture_handle.set(
            egui::ColorImage
            {
                size: self.img_size,
                pixels: self.pixel_buffer.clone(),
            ..Default::default()
            },
            egui::TextureOptions::NEAREST);
    }
    pub fn draw_on_ui(&self, ui: &egui::Ui)
    {
        let central_panel_rect = ui.min_rect();
        let available_size     = ui.available_size();
        let array_size         = available_size.min_elem();
        let center_x           = central_panel_rect.center().x;
        let center_y           = central_panel_rect.center().y;
        let array_top_left = egui::Pos2 
        {
            x: center_x - 0.5*array_size, 
            y: center_y - 0.5*array_size 
        };
        let rect = egui::Rect::from_min_size(array_top_left, egui::Vec2 { x: array_size, y: array_size });
        let uv   = egui::Rect{ min:egui::pos2(0.0, 0.0), max:egui::pos2(1.0, 1.0)};
        ui.painter().image(self.texture_handle.id(), rect, uv, egui::Color32::WHITE);
    }
}