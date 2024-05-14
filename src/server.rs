use std::{
    collections::HashMap, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, ops::Deref, sync::{mpsc, Arc, Mutex}, thread, time::Duration
};
use uuid::Uuid;

mod tests;

struct Server {
    sender: mpsc::Sender<String>,
    current_ip: Arc<Mutex<String>>,
    termination_signal: Arc<Mutex<bool>>,
    switch_mode: Arc<Mutex<bool>>,
}

pub struct ServerWrapper {
    server: Arc<Server>,
}

impl ServerWrapper {
    // / Create a new TcpServer. When you create a nwe server it will keep rotating between
    // / a TcpListener and a TcpStream that will send data.
    // /
    // / # Examples
    // /
    // / ```
    // / use server::server::ServerWrapper;
    // /
    // / let server = ServerWrapper::new();
    // / ```
    pub fn new(tx: mpsc::Sender<String>) -> ServerWrapper {
        ServerWrapper {
            server: Arc::new(Server {
                sender: tx,
                current_ip: Arc::new(Mutex::new(String::from("25.55.184.100:3012"))),
                termination_signal: Arc::new(Mutex::new(false)),
                switch_mode: Arc::new(Mutex::new(false))
            }),
        }
    }

    pub fn run(&self) {
        let server_clone_1 = self.server.clone();
        thread::spawn(move || {
            let mut state = false;
            loop {
                let server_clone_2 = server_clone_1.clone();
                match state {
                    false => state = server_clone_1.host_server(),
                    true => {
                        thread::spawn(move|| {
                            server_clone_2.pulse();
                        });
                        state = server_clone_1.tcp_server()
                    },
                }
                thread::sleep(Duration::from_secs(1));
            }
        });
    }
}

impl Server {
    #[allow(unused)]
    fn host_server(&self) -> bool {
        let mut server_mode_switch = false;

        manage_mutex(self.termination_signal.clone(), Some(false));

        let ip;
        ip = manage_mutex(self.current_ip.clone(), None).unwrap();
        
        let mut request = String::from("NoId\nH1\nH2");
        
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

            let http_response = {
                let mut temp_hashmap = HashMap::new();
                for element in http_response {
                    let (k, v) = element
                        .split_once(':')
                        .unwrap();
                    temp_hashmap.insert(k.to_owned(), v.trim().to_owned());
                }
                println!("{:#?}", temp_hashmap);
                temp_hashmap
            };
            // println!("Response: {:#?}", http_response);

            if http_response.get("State").unwrap() != "OK" {
                request = format!("{}\nH1\nH2", http_response.get("Id").unwrap());
            }
            if http_response.get("SwitchToServer").unwrap() == "true" {
                {
                    let ip_clone = self.current_ip.clone();
                    let mut guard = ip_clone.lock().unwrap();
                    *guard = format!("{}:3012", stream.local_addr().unwrap().ip().to_string());
                }
                server_mode_switch = true;
                break;
            }
            thread::sleep(Duration::from_millis(1500));
        }
        server_mode_switch
    }
   
    #[allow(unused)]
    fn tcp_server(&self) -> bool {
        let addr;

        addr = manage_mutex(self.current_ip.clone(), None).unwrap();

        let mut hosts_dir: HashMap<String, String> = HashMap::new();

        let listener = TcpListener::bind(&addr).unwrap();
        println!("---\nListening on {}\n---", &addr);

        for stream in listener.incoming() {
            {
                let signal = self.termination_signal.lock().expect("Fallo en checar la señal");
                if *signal {
                    break;
                }
            }

            let stream = stream.expect("Fallo en inicial el stream");
            let guard = self.switch_mode.lock().unwrap();
            match *guard {
                true => {
                    self.handle_conecction(stream, &mut hosts_dir, true);
                    continue;
                    },
                false => self.handle_conecction(stream, &mut hosts_dir, false)
            }
        }
        false
    }

    fn pulse(&self) {
        let current_ip = manage_mutex(self.current_ip.clone(), None).unwrap();

        loop {
            thread::sleep(Duration::from_secs(15));
            // If server switch
            manage_mutex(self.switch_mode.clone(), Some(true));

            thread::sleep(Duration::from_secs(2));

            manage_mutex(self.termination_signal.clone(), Some(true));

            let mut stream = TcpStream::connect(&current_ip).unwrap();
            stream.write_all("OK\nNone\n".as_bytes()).unwrap(); // probably change request

            manage_mutex(self.switch_mode.clone(), Some(false));
            break;
        }
    }

    fn handle_conecction(&self, mut stream: TcpStream, hosts_dir: &mut HashMap<String, String>, migration: bool) {
        let buf_reader = BufReader::new(&mut stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        println!("Request: {:#?}", http_request);

        let response;

        match migration {
            true => {
                let host_ip = stream
                    .peer_addr()
                    .unwrap()
                    .ip()
                    .to_string();
                println!("host ip: {}", host_ip);
                if host_ip == hosts_dir.get(&http_request[0]).unwrap().to_owned() {
                    let new_ip = stream.peer_addr().unwrap().ip().to_string();
                    response = format!("State: OK\nSwitchToServer: true\nNewServer: None\nId: {}", http_request[0]);

                    let mut guard = self.current_ip.lock().unwrap();
                    *guard = format!("{}:3012",new_ip);
                } else {
                    response = format!("State: Unauthorized\nSwitchToServer: false\nNewServer: {}\nId: {}", host_ip, http_request[0]);
                }
            },
            false => {
                if http_request[0] == "NoId" {
                    let id = String::from(Uuid::new_v4());
                    response = format!("State: Unauthorized\nSwitchToServer: false\nNewServer: None\nId: {}", id);
                    hosts_dir.insert(
                        id, 
                        stream
                            .peer_addr()
                            .unwrap()
                            .ip()
                            .to_string()
                    );
                } else {
                    response = format!("State: OK\nSwitchToServer: false\nNewServer: None\nId: {}", http_request[0]);
                    println!("----------\nhost ip: {}\n----------\n",stream.peer_addr().unwrap());
                }
            }
        }
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn manage_mutex<T>(mutex: Arc<Mutex<T>>, data: Option<T>) ->Option<T> 
where
    T: Clone
{
    let mut guard = mutex.lock().unwrap();
    let x = (*guard).clone();
    
    match data {
        Some(data) => {
            let d = data.clone();
            *guard = d;
            return Some(data)
        },
        None => Some(x)
    }
}