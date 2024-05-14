use server::app::App;
use server::server::ServerWrapper;
use std::{net::TcpListener, sync::mpsc, thread, time::Duration};
use tungstenite::accept;

fn main() -> eframe::Result<()> {
    let (tx, rx) = mpsc::channel();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 550.0]),
        ..Default::default()
    };

    // let listener = TcpListener::bind("0.0.0.0:5432").unwrap();
    // let listener_clone = listener.try_clone().expect("Failed to clone listener");
    // thread::spawn(move || {
    //     tcp_listener_thread(listener_clone, tx)
    // });
    let server = ServerWrapper::new();
    server.run();

    eframe::run_native(
        "status",
        native_options,
        Box::new(|cc| Box::new(App::new(cc, rx))),
    )
}

fn tcp_listener_thread(listener: TcpListener, tx: mpsc::Sender<String>) {
    let mut id: u32 = 0;
    for stream in listener.incoming() {
        let ws = accept(stream.unwrap()).expect("Failed to accept");
        let tx_clone = tx.clone();
        thread::spawn(move || handle_connection(ws, tx_clone, id));
        id += 1;
    }
}

fn handle_connection(
    mut ws: tungstenite::WebSocket<std::net::TcpStream>,
    tx: mpsc::Sender<String>,
    id: u32,
) {
    let mut register = String::from("");
    loop {
        let msg = match ws.read() {
            Ok(msg) => msg,
            Err(_) => {
                let msg = tungstenite::Message::Text(register);
                let msg = format!("{id},{msg},disconnected");
                eprintln!("Host disconnected");
                println!("{msg}");
                tx.send(msg).unwrap();
                thread::sleep(Duration::from_secs(1));
                break;
            }
        };
        register = msg.to_string().to_owned();
        let msg = format!("{id},{},connected", msg);
        tx.send(msg).unwrap();
    }
}
