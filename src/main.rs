#![allow(unused)]
use server::app::App;
use server::server::ServerWrapper;
use std::{net::TcpListener, 
    sync::mpsc::{self, Receiver, Sender}, 
    thread, time::Duration};

fn main() -> eframe::Result<()>{
    let (tx, rx) = mpsc::channel();

    let server = ServerWrapper::new(tx);
    server.run();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 550.0]),
        ..Default::default()
    };

    eframe::run_native(
        "status",
        native_options,
        Box::new(|cc| Box::new(App::new(cc, rx))),
    )
}
