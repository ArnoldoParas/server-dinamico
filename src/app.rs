use std::sync::mpsc;
use sysinfo::System;
use egui::{RichText, ScrollArea};
use std::collections::HashMap;

// #[derive(Default)]
pub struct App {
  info: SystemInfo,
  receiver: mpsc::Receiver<String>,
  clients: HashMap<u32, Vec<String>>,
  ranked_clients: Vec<(u32,f32)>
}

struct SystemInfo {
  client_name: String,
  cpu_model: String,
  cpu_freq: String,
  physical_cores: String,
  total_memory: String,
  os: String,
  os_version: String,
}

impl App {
  pub fn new(_cc: &eframe::CreationContext<'_>, rx: mpsc::Receiver<String>) -> Self {
    let mut sys = System::new_all();
    sys.refresh_all();
    let cpu = sys.cpus().get(0).unwrap();

    App {
      info: SystemInfo {
        client_name: System::host_name().unwrap(),
        cpu_model: cpu.brand().into(),
        cpu_freq: format!("{}",cpu.frequency()),
        physical_cores: format!("{}",sys.physical_core_count().unwrap()),
        total_memory: format!("{} bytes",sys.total_memory()),
        os: format!("{}", System::long_os_version().unwrap()),
        os_version: format!("{}", System::kernel_version().unwrap())
      },
      receiver: rx,
      clients: HashMap::new(),
      ranked_clients: Vec::new()
    }
  }

  fn render_results(&self, ui: &mut eframe::egui::Ui) {
    ui.horizontal(|ui| {
      ui.add_space(ui.available_width());
    });
    if self.clients.len() != 0 {
      egui::Grid::new("test")
      .min_col_width(100.0)
      .striped(true)
      .show(ui, |ui|{
        ui.vertical_centered(|ui| {
          ui.label("Rank");
        });
        ui.vertical_centered(|ui| {
          ui.label("Client name");
        });
        ui.vertical_centered(|ui| {
          ui.label("CPU usage");
        });
        ui.vertical_centered(|ui| {
          ui.label("RAM usage (MB)");
        });
        ui.vertical_centered(|ui| {
          ui.label("Free bandwidth (MB)");
        });
        ui.vertical_centered(|ui| {
          ui.label("Free disk memory (GB)");
        });
        ui.vertical_centered(|ui| {
          ui.label("Status");
        });
        ui.end_row();
        #[allow(unused)]
        for (i, (k, s)) in self.ranked_clients.iter().enumerate() {
          ui.vertical_centered(|ui| {
            match i+1 {
              1 => ui.label(RichText::new((i+1).to_string()).color(egui::Color32::GOLD)),
              2 => ui.label(RichText::new((i+1).to_string()).color(egui::Color32::GRAY)),
              3 => ui.label(RichText::new((i+1).to_string()).color(egui::Color32::BROWN)),
              _ =>  ui.label(RichText::new((i+1).to_string()))
            };
          });
          ui.vertical_centered(|ui| {
            ui.label(RichText::new(&self.clients.get(&k).unwrap()[0]));
          });
          ui.vertical_centered(|ui| {
            ui.label(RichText::new(&self.clients.get(&k).unwrap()[1]));
          });
          ui.vertical_centered(|ui| {
            ui.label(RichText::new(&self.clients.get(&k).unwrap()[2]));
          });
          ui.vertical_centered(|ui| {
            ui.label(RichText::new(&self.clients.get(&k).unwrap()[3]));
          });
          ui.vertical_centered(|ui| {
            ui.label(RichText::new(&self.clients.get(&k).unwrap()[4]));
          });
          ui.vertical_centered(|ui| {
            if &self.clients.get(&k).unwrap()[6] == "connected" {
              ui.label(RichText::new(&self.clients.get(&k).unwrap()[6]).color(egui::Color32::GREEN));
            }
            else {
              ui.label(RichText::new(&self.clients.get(&k).unwrap()[6]).color(egui::Color32::RED));
            }
          });
          ui.end_row();
        }
      });
    }
  }

  fn handle_tcp_data(&mut self) {
    while let Ok(data) = self.receiver.try_recv() {
      let mut x = data
        .split(',')
        .map(String::from)
        .collect::<Vec<_>>();
      let key: u32 = x[0].parse().unwrap();
      x.remove(0);
      self.clients.insert(key, x);
      println!("----------------------------------------------------");
      println!("{:?}", self.clients);
      self.rank_clients();
    }
  }

  fn rank_clients(&mut self) {
    self.ranked_clients.clear();
    for (k, v) in &self.clients {
      let total_mem = &v[5].to_owned().parse::<f32>().unwrap();
      let current_mem = &v[2].to_owned().parse::<f32>().unwrap();
      let ram_percentage = (current_mem / total_mem * 10000.0).trunc()/100.0;
      println!("cpu pecentaje: {}\nram percentaje: {}\n",&v[1], &ram_percentage);
      
      let cpu_score = match &v[1].parse::<f32>().unwrap() {
        x if *x >= 0.0 && *x <= 10.0 => 3.0 * 0.6,
        x if *x > 10.0 && *x <= 20.0 => 5.0 * 0.6,
        x if *x > 20.0 && *x <= 30.0 => 6.0 * 0.6,
        x if *x > 30.0 && *x <= 40.0 => 7.0 * 0.6,
        x if *x > 40.0 && *x <= 50.0 => 8.0 * 0.6,
        x if *x > 50.0 && *x <= 60.0 => 9.0 * 0.6,
        x if *x > 60.0 && *x <= 70.0 => 10.0 * 0.6,
        x if *x > 70.0 && *x <= 80.0 => 9.0 * 0.6,
        x if *x > 80.0 && *x <= 90.0 => 8.0 * 0.6,
        x if *x > 90.0 && *x <= 100.0 => 6.0 * 0.6,
        _ => 0.0
      };

      let ram_score = match ram_percentage {
        x if x >= 0.0 && x <= 10.0 => 6.0 * 0.4,
        x if x > 10.0 && x <= 20.0 => 7.0 * 0.4,
        x if x > 20.0 && x <= 30.0 => 8.0 * 0.4,
        x if x > 30.0 && x <= 40.0 => 9.0 * 0.4,
        x if x > 40.0 && x <= 50.0 => 10.0 * 0.4,
        x if x > 50.0 && x <= 60.0 => 9.0 * 0.4,
        x if x > 60.0 && x <= 70.0 => 8.0 * 0.4,
        x if x > 70.0 && x <= 80.0 => 6.0 * 0.4,
        x if x > 80.0 && x <= 90.0 => 4.0 * 0.4,
        x if x > 90.0 && x <= 100.0 => 2.0 * 0.4,
        _ => 0.0
      };

      let score = cpu_score + ram_score;

      self.ranked_clients.push((k.to_owned(), score));
    }
    // Ordenar de mayor a menor por los valores f32
    self.ranked_clients.sort_by(
      |a, b| 
      b.1.partial_cmp(&a.1).unwrap()
    );
    println!("{:?}", self.ranked_clients);
  }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::SidePanel::left("Left-panel").exact_width(250.0).show(ctx, |ui| {
      ui.vertical_centered(|ui| {
        ui.add_space(30.0);
        ui.heading("Informacion del dispositivo");
        ui.add_space(30.0);
      });
      ui.horizontal(|ui| {
        ui.label("System name:");
        ui.label(RichText::new(&self.info.client_name));
      });
      ui.add_space(10.0);
      ui.horizontal(|ui| {
        ui.label("CPU model:");
        ui.label(RichText::new(&self.info.cpu_model));
      });
      ui.add_space(10.0);
      ui.horizontal(|ui| {
        ui.label("Frequency:");
        ui.label(RichText::new(&self.info.cpu_freq));
      });
      ui.add_space(10.0);
      ui.horizontal(|ui| {
        ui.label("Physical cores:");
        ui.label(RichText::new(&self.info.physical_cores));
      });
      ui.add_space(10.0);
      ui.horizontal(|ui| {
        ui.label("Total memory:");
        ui.label(RichText::new(&self.info.total_memory));
      });
      ui.add_space(10.0);
      ui.horizontal(|ui| {
        ui.label("Os:");
        ui.label(RichText::new(&self.info.os));
      });
      ui.add_space(10.0);
      ui.horizontal(|ui| {
        ui.label("Os version:");
        ui.label(RichText::new(&self.info.os_version));
      });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
      ui.vertical_centered(|ui| {
        ui.add_space(30.0);
        ui.heading("Clientes");
        ui.add_space(10.0);
      });

      ScrollArea::vertical().show(ui, |ui| {
        self.render_results(ui);
      });
    });

    self.handle_tcp_data();
    ctx.request_repaint();
  }
}