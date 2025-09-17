use std::collections::VecDeque;
use egui_plot::{Line, Plot, PlotPoints};
use egui::Ui;

pub struct DataDeque
{
    data: VecDeque<f32>
}

impl DataDeque
{
    pub fn new(n: usize) -> Self
    {
        let data: VecDeque<f32> = VecDeque::from(vec![0_f32; n] ); 
        Self { data }
    }    
    pub fn pop_and_push(&mut self, value: f32)
    {
        self.data.pop_back();
        self.data.push_front(value);
    }
    pub fn at(&self, i: usize) -> f32
    {
        self.data[i]
    }
    pub fn len(&self) -> usize
    {
        self.data.len()
    }
}

pub struct RealTimeDataHandler
{
    data_deque: DataDeque,
    counter: usize,
    n_avg: usize,
    data_cumul: f32
}

impl RealTimeDataHandler 
{
    pub fn new(n_total: usize, n_average: usize) -> Self
    {
        Self { data_deque: DataDeque::new(n_total), counter: 0, n_avg: n_average, data_cumul: 0f32 }
    }
    pub fn append_and_update(&mut self, data: f32)
    {
        if self.counter > 0
        {
            self.data_cumul += data;
            if self.counter%self.n_avg == 0
            {
                let data_avg = self.data_cumul / self.n_avg as f32;
                self.data_deque.pop_and_push(data_avg);
                self.counter = 0;
                self.data_cumul = 0f32;
            }
        }
        self.counter += 1;
    }
    pub fn plot_data(&self, ui: &mut Ui, x_label: &'static str, y_label: &'static str, y_min: f64, y_max: f64)
    {
        let my_series: PlotPoints<'_> = (0..self.data_deque.len()).map(|i|
        {
            let t = i as f64 * -1f64;
            [t, self.data_deque.at(i) as f64]
        }).collect();
        
        let line = Line::new("name", my_series);
        Plot::new("my_plot")
            .view_aspect(2.0)
            .x_axis_label(x_label)
            .y_axis_label(y_label)
            .default_y_bounds(y_min, y_max)
            .show(ui, |plot_ui| plot_ui.line(line));
    }
}