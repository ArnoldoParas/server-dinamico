use std::sync::mpsc;
use sysinfo::System;
use egui::{RichText, ScrollArea};
use std::collections::HashMap;

// #[derive(Default)]
pub struct App {
  info: SystemInfo,
  receiver: mpsc::Receiver<String>,
  clients: HashMap<u32, Vec<String>>
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
      clients: HashMap::new()
    }
  }

  fn render_results(&self, ui: &mut eframe::egui::Ui) {
    ui.horizontal(|ui| {
      ui.add_space(ui.available_width());
  });

    // let mut count:u8 = 0;
    if self.clients.len() != 0 {
      egui::Grid::new("test")
      .min_col_width(100.0)
      .striped(true)
      .show(ui, |ui|{
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
        for (k, v) in &self.clients {
          ui.vertical_centered(|ui| {
            ui.label(RichText::new(&v[0]));
          });
          ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
              ui.label(RichText::new(&v[1]));
              ui.label("%");
            });
          });
          ui.vertical_centered(|ui| {
            ui.label(RichText::new(&v[2]));
          });
          ui.vertical_centered(|ui| {
            ui.label(RichText::new(&v[3]));
          });
          ui.vertical_centered(|ui| {
            ui.label(RichText::new(&v[4]));
          });
          ui.vertical_centered(|ui| {
            if &v[5] == "connected" {
              ui.label(RichText::new(&v[5]).color(egui::Color32::GREEN));
            }
            else {
              ui.label(RichText::new(&v[5]).color(egui::Color32::RED));
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
    }
  }
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