use server::app::App;
use std::{
    io::{prelude::*, BufReader}, 
    net::TcpListener,  
    thread,
    sync::mpsc
};


fn main() -> eframe::Result<()> {
    let (tx, rx) = mpsc::channel();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0,550.0]),
            ..Default::default()
    };

    let listener = TcpListener::bind("0.0.0.0:5432").unwrap();
    let listener_clone = listener.try_clone().expect("Failed to clone listener");
    println!("Server listening on port 5432");

    thread::spawn(move || {
        tcp_listener_thread(listener_clone, tx)
    });

    eframe::run_native(
        "status",
        native_options, 
        Box::new(|cc| Box::new(App::new(cc, rx)))
    )
}

fn tcp_listener_thread(listener: TcpListener, tx: mpsc::Sender<String>) {
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buf_reader = BufReader::new(&mut stream);
        let mut request_data = String::new();
        buf_reader.read_to_string(&mut request_data).unwrap();

        println!("{request_data}");
        tx.send(request_data).unwrap();
    }
}