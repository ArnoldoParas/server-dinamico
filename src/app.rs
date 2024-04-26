use std::sync::mpsc;
use sysinfo::System;
use egui::{RichText, ScrollArea};

// #[derive(Default)]
pub struct App {
  info: SystemInfo,
  receiver: mpsc::Receiver<String>,
  clients_info: Vec<SystemInfo>
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
      clients_info: Vec::new(),
    }
  }

  fn render_results(&self, ui: &mut eframe::egui::Ui) {
    ui.horizontal(|ui| {
      ui.add_space(ui.available_width());
  });
    let mut count:u8 = 0;
    if self.clients_info.len() != 0 {
      for i in &self.clients_info {
        egui::CollapsingHeader::new(RichText::new(&i.client_name)).id_source(count.to_string()).show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
              ui.label("CPU model:");
              ui.label(RichText::new(&i.cpu_model));
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
              ui.label("Frequency:");
              ui.label(RichText::new(&i.cpu_freq));
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
              ui.label("Physical cores:");
              ui.label(RichText::new(&i.physical_cores));
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
              ui.label("Total memory:");
              ui.label(RichText::new(&i.total_memory));
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
              ui.label("Os:");
              ui.label(RichText::new(&i.os));
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
              ui.label("Os version:");
              ui.label(RichText::new(&i.os_version));
            });
        });
        count += 1;
      }
    }
  }

  fn handle_tcp_data(&mut self) {
    while let Ok(data) = self.receiver.try_recv() {
      let mut iter = data.split(',');
      self.clients_info.push(SystemInfo {
        client_name: iter.next().unwrap().to_string(),
        cpu_model: iter.next().unwrap().to_string(),
        cpu_freq: iter.next().unwrap().to_string(),
        physical_cores: iter.next().unwrap().to_string(),
        total_memory: iter.next().unwrap().to_string(),
        os: iter.next().unwrap().to_string(),
        os_version: iter.next().unwrap().to_string()
      });
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
  }
}