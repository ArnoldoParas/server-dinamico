use std::{
    collections::HashMap, 
    io::{prelude::*, BufReader}, 
    net::{TcpListener, TcpStream}, 
    sync::{Arc, Mutex}, thread, 
    time::Duration
};

mod tests;

struct Server {
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
                current_ip: Arc::new(Mutex::new(String::from("192.168.100.31:3012"))),
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
        let mut server_mode_switch = false;
        {
            let termination_signal_clone = self.termination_signal.clone();
            let mut guard = termination_signal_clone.lock().unwrap();
            *guard = false;
        }
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
   
    #[allow(unused)]
    fn tcp_server(&self) -> bool {
        println!("hi from tcp");
        let addr;
        {
            let ip_clone = self.current_ip.clone();
            let guard = ip_clone.lock().unwrap();
            addr = String::from(&*guard);
        }
        let mut hosts_dir: HashMap<String, String> = HashMap::new();

        let listener = TcpListener::bind(&addr).unwrap();
        println!("---\nListening on {}\n---", &addr);

        let switch = Arc::new(Mutex::new(false));
        let switch_clone = switch.clone();
        let srv = Arc::new(self);

        thread::spawn(move || {
            pulse(srv);
        });

        false
    }
}

fn pulse(srv: Arc<&Server>) {
    ()
}

//     thread::spawn(move ||{
//         clk(switch_clone, termination_signal_clone, ip_clonee);
//     });

//     for stream in listener.incoming() {
//         {
//             let signal = termination_signal.lock().expect("Fallo en checar la señal");
//             if *signal {
//                 break;
//             }
//         }
        
//         let stream = stream.expect("Fallo en inicial el strea?");
//         let lock = switch.lock().unwrap();
//         if *lock{
//             let ip_clone = ip.clone();
//             switch_connection(stream, &mut hosts, ip_clone);
//             continue; 
//         };
//         handle_conecction(stream, &mut hosts);
//     }
//     thread::spawn(|| {
//         host(ip);
//     });