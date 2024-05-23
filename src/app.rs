use egui::{RichText, ScrollArea};
use std::{
    collections::HashMap, fmt::format, sync::mpsc::{self, Sender}
};
use sysinfo::System;

// #[derive(Default)]
pub struct App {
    info: SystemInfo,
    clients: HashMap<String, Vec<String>>,
    ranked_clients: Vec<(String, f32)>,
    clear_hash: bool,
    receiver: mpsc::Receiver<HashMap<String, Vec<String>>>,
    sender: mpsc::Sender<String>,
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
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        rx: mpsc::Receiver<HashMap<String, Vec<String>>>,
        tx: Sender<String>,
    ) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let cpu = sys.cpus().first().unwrap();

        App {
            info: SystemInfo {
                client_name: System::host_name().unwrap(),
                cpu_model: cpu.brand().into(),
                cpu_freq: cpu.frequency().to_string(),
                physical_cores: format!("{}", sys.physical_core_count().unwrap()),
                total_memory: format!("{} bytes", sys.total_memory()),
                os: System::long_os_version().unwrap().to_string(),
                os_version: System::kernel_version().unwrap().to_string(),
            },
            clients: HashMap::new(),
            ranked_clients: Vec::new(),
            clear_hash: false,
            receiver: rx,
            sender: tx,
        }
    }

    fn render_results(&mut self, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(ui.available_width());
        });
        if !self.clients.is_empty() {
            egui::Grid::new("test")
                .min_col_width(100.0)
                .striped(true)
                .show(ui, |ui| {
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
                            match i + 1 {
                                1 => ui.label(
                                    RichText::new((i + 1).to_string()).color(egui::Color32::GOLD),
                                ),
                                2 => ui.label(
                                    RichText::new((i + 1).to_string()).color(egui::Color32::GRAY),
                                ),
                                3 => ui.label(
                                    RichText::new((i + 1).to_string()).color(egui::Color32::BROWN),
                                ),
                                _ => ui.label(RichText::new((i + 1).to_string())),
                            };
                        });
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(&self.clients.get(k).unwrap()[0]));
                        });
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(&self.clients.get(k).unwrap()[1]));
                        });
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(&self.clients.get(k).unwrap()[2]));
                        });
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(&self.clients.get(k).unwrap()[3]));
                        });
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(&self.clients.get(k).unwrap()[4]));
                        });
                        ui.vertical_centered(|ui| {
                            if &self.clients.get(k).unwrap()[6] == "connected" {
                                ui.label(
                                    RichText::new(&self.clients.get(k).unwrap()[6])
                                        .color(egui::Color32::GREEN),
                                );
                            } else {
                                ui.label(
                                    RichText::new(&self.clients.get(k).unwrap()[6])
                                        .color(egui::Color32::RED),
                                );
                            }
                        });
                        ui.end_row();
                    }
                });
        }
        if self.clear_hash {
            self.clients.clear();
            self.ranked_clients.clear();
            self.clear_hash = false;
        }
    }

    fn rank_clients(&mut self) {
        let mut first_place_key = String::new();
        match self.ranked_clients.is_empty() {
            true => (),
            false => first_place_key = self.ranked_clients[0].0.clone(),
        }

        self.ranked_clients.clear();
        for (k, v) in &self.clients {
            let total_mem = &v[5].to_owned().parse::<f32>().unwrap();
            let current_mem = &v[2].to_owned().parse::<f32>().unwrap();
            let ram_percentage = (current_mem / total_mem * 10000.0).trunc() / 100.0;

            let cpu_score = match &v[1].parse::<f32>().unwrap() {
                x if (0.0..=10.0).contains(x) => 3.0 * 0.6,
                x if (11.0..=20.0).contains(x) => 5.0 * 0.6,
                x if (21.0..=30.0).contains(x) => 6.0 * 0.6,
                x if (31.0..=40.0).contains(x) => 7.0 * 0.6,
                x if (41.0..=50.0).contains(x) => 8.0 * 0.6,
                x if (51.0..=60.0).contains(x) => 9.0 * 0.6,
                x if (61.0..=70.0).contains(x) => 10.0 * 0.6,
                x if (71.0..=80.0).contains(x) => 9.0 * 0.6,
                x if (81.0..=90.0).contains(x) => 8.0 * 0.6,
                x if (91.0..=100.0).contains(x) => 6.0 * 0.6,
                _ => 0.0,
            };

            let ram_score = match ram_percentage {
                x if (0.0..=10.0).contains(&x) => 6.0 * 0.4,
                x if (11.0..=20.0).contains(&x) => 7.0 * 0.4,
                x if (21.0..=30.0).contains(&x) => 8.0 * 0.4,
                x if (31.0..=40.0).contains(&x) => 9.0 * 0.4,
                x if (41.0..=50.0).contains(&x) => 10.0 * 0.4,
                x if (51.0..=60.0).contains(&x) => 9.0 * 0.4,
                x if (61.0..=70.0).contains(&x) => 8.0 * 0.4,
                x if (71.0..=80.0).contains(&x) => 6.0 * 0.4,
                x if (81.0..=90.0).contains(&x) => 4.0 * 0.4,
                x if (91.0..=100.0).contains(&x) => 2.0 * 0.4,
                _ => 0.0,
            };
            let score = cpu_score + ram_score;

            self.ranked_clients.push((k.to_owned(), score));
        }
        self.ranked_clients
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        match first_place_key.is_empty() {
            true => (),
            false => {
                if dbg!(self.ranked_clients[0].0 != first_place_key) { // hay un cambio lo que significa que minimo hay 2
                    let msg = format!("First\n{}", self.ranked_clients[0].0.clone());
                    self.sender.send(msg).unwrap();
                    self.clear_hash = true;
                } 
            }
        }

        if self.ranked_clients.len() >= 2 {
            let msg = format!("Second\n{}", self.ranked_clients[1].0.clone());
            self.sender.send(msg).unwrap();
        }

    }

    fn handle_tcp_data(&mut self) {
        if let Ok(msg) = self.receiver.try_recv() {
            self.clients = msg;
            self.rank_clients()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("Left-panel")
            .exact_width(250.0)
            .show(ctx, |ui| {
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
