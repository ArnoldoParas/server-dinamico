use std::{
    io::{prelude::*, BufReader},
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

mod tests;

struct Server {
    default_ip: String,
    current_ip: Arc<Mutex<String>>,
    termination_signal: Arc<Mutex<bool>>,
}

pub struct ServerWrapper {
    server: Arc<Server>,
}

impl ServerWrapper {
    /// Create a new TcpServer. When you create a nwe server it will keep rotating between
    /// a TcpListener and a TcpStream that will send data.
    ///
    /// # Examples
    ///
    /// ```
    /// use server::server::ServerWrapper;
    ///
    /// let server = ServerWrapper::new();
    /// ```
    pub fn new() -> ServerWrapper {
        ServerWrapper {
            server: Arc::new(Server {
                default_ip: String::from("10.100.42.211:3012"),
                current_ip: Arc::new(Mutex::new(String::from(""))),
                termination_signal: Arc::new(Mutex::new(false)),
            }),
        }
    }

    pub fn run(&self) {
        let server_clone = self.server.clone();
        thread::spawn(move || {
            let mut state = false;
            loop {
                match state {
                    false => state = server_clone.host_server(),
                    true => state = server_clone.tcp_server(),
                }
                thread::sleep(Duration::from_secs(1));
            }
        });
    }
}

impl Server {
    #[allow(unused)]
    fn host_server(&self) -> bool {
        println!("hi from host");
        {
            let termination_signal_clone = self.termination_signal.clone();
            let mut guard = termination_signal_clone.lock().unwrap();
            *guard = false;
        }
        let mut server_mode_switch = false;
        let ip;
        {
            let ip_clone = self.current_ip.clone();
            let guard = ip_clone.lock().unwrap();
            ip = String::from(&*guard);
        };
        let mut request = String::from(" \nH1\nH2");

        loop {
            let mut stream;
            match TcpStream::connect(&ip) {
                Ok(s) => stream = s,
                Err(_) => {
                    server_mode_switch = true;
                    break;
                }
            }
            stream
                .write_all(request.as_bytes())
                .expect("fallo en enviar el mensaje");
            stream.shutdown(std::net::Shutdown::Write).unwrap();

            let buf_reader = BufReader::new(&mut stream);
            let http_response: Vec<_> = buf_reader
                .lines()
                .map(|result| result.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();

            if http_response[0] != "OK" {
                request = format!("{}\nH1\nH2", http_response[2]);
            }
            if http_response[1] != "None" {
                {
                    let ip_clone = self.current_ip.clone();
                    let mut guard = ip_clone.lock().unwrap();
                    *guard = format!("{}:3012", &http_response[1]);
                }
                server_mode_switch = true;
                break;
            }
            println!("Response: {:#?}", http_response);
            thread::sleep(Duration::from_millis(1500));
        }
        server_mode_switch
    }

    fn tcp_server(&self) -> bool {
        println!("hi from tcp");
        false
    }
}
