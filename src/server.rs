use std::{
    collections::HashMap,
    io::{prelude::*, BufReader},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex}, 
    thread,
    time::Duration,
    process, 
};
use crate::sysinfo;
use uuid::Uuid;
use chrono::{DateTime, Utc};

mod tests;

struct Server {
    sender: mpsc::Sender<HashMap<String, Vec<String>>>,
    reciver: Mutex<mpsc::Receiver<String>>,
    id: Arc<Mutex<String>>,
    server_ip: Arc<Mutex<String>>,
    current_ip: Arc<Mutex<String>>,
    fallback_ip: Arc<Mutex<String>>,
    termination_signal: Arc<Mutex<bool>>,
    migration_mode: Arc<Mutex<bool>>,
    host_data: Arc<Mutex<HashMap<String, Vec<String>>>>,
    new_server_id: Arc<Mutex<String>>,
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
    pub fn new(
        tx: mpsc::Sender<HashMap<String, Vec<String>>>,
        rx: mpsc::Receiver<String>,
    ) -> ServerWrapper {
        ServerWrapper {
            server: Arc::new(Server {
                sender: tx,
                reciver: Mutex::new(rx),
                id: Arc::new(Mutex::new(String::new())),
                server_ip: Arc::new(Mutex::new(String::new())),
                current_ip: Arc::new(Mutex::new(String::from("25.55.184.100:3012"))),
                fallback_ip: Arc::new(Mutex::new(String::new())),
                termination_signal: Arc::new(Mutex::new(false)),
                migration_mode: Arc::new(Mutex::new(false)),
                host_data: Arc::new(Mutex::new(HashMap::new())),
                new_server_id: Arc::new(Mutex::new(String::new())),
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
                        thread::spawn(move || {
                            server_clone_2.pulse();
                        });
                        state = server_clone_1.tcp_server()
                    }
                }
            }
        });
    }
}

#[allow(unused)]
impl Server {
    fn host_server(&self) -> bool {
        let mut server_mode_switch = false;

        manage_mutex(self.termination_signal.clone(), Some(false));

        let mut ip: String = manage_mutex(self.current_ip.clone(), None).unwrap();
        let mut request;
        let mut id = String::from("No-Id");

        let mut guard = self.id.lock().unwrap();
        if !guard.is_empty() {
            id = guard.to_owned();
            request = format!("{}\n{}\n{}",
                id,
                manage_mutex(self.server_ip.clone(), None).unwrap(),
                sysinfo::get_info()
            );
        } else {
            request = format!("{}\nNo-Ip\n{}",
                id,
                sysinfo::get_info()
            );
        }

        let mut last_server_response: HashMap<String, String> = HashMap::new();
        let mut connection_attempts: u8 = 0;
        let mut fallback_server_state = false;
        loop {
            let mut stream;
            match TcpStream::connect(&ip) {
                Ok(s) => {
                    connection_attempts += 1;
                    println!("connection attempts: {}, SUCCESS", connection_attempts);
                    connection_attempts = 0;
                    fallback_server_state = false;
                    stream = s
                },
                Err(_) => {
                    connection_attempts += 1;
                    if connection_attempts <= 1 {
                        thread::sleep(Duration::from_millis(500));
                        eprintln!("connection attempts: {}, FAIL", connection_attempts);
                        continue;
                    }

                    if last_server_response.is_empty() {
                        if !fallback_server_state {
                            eprintln!("connection attempts: {}, FAIL", connection_attempts);
                            server_mode_switch = true;
                            break;
                        }
                        eprintln!("connection attempts: {}, FAIL", connection_attempts);
                        eprintln!("No fallback server... aborting");
                        process::abort()
                    } else { 
                        match last_server_response.get("Fallback-Server").unwrap().as_str() {
                            "None" => {
                                eprintln!("connection attempts: {}, FAIL", connection_attempts);
                                eprintln!("No fallback server... aborting");
                                process::abort()
                            },
                            server_ip => {
                                println!("Fallback server ip: {}", server_ip);
                                if *self.server_ip.lock().unwrap() == server_ip {
                                    println!("Switching to server...");
                                    let mut guard = self.current_ip.lock().unwrap();
                                    *guard = format!("{}:3012", server_ip);
                                    server_mode_switch = true;
                                    break;
                                }
                                ip = manage_mutex(self.current_ip.clone(), Some(format!("{}:3012", server_ip))).unwrap();
                                connection_attempts = 0;
                                fallback_server_state = true;
                                last_server_response.clear();
                                continue;
                            }
                        } 
                    }
                }
            }
            
            stream
                .write_all(request.as_bytes())
                .expect("fallo en enviar el mensaje");

            stream.shutdown(Shutdown::Write).unwrap();

            let buf_reader = BufReader::new(&mut stream);
            let http_response: Vec<_> = buf_reader
                .lines()
                .map(|result| result.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();

            let http_response = {
                let mut temp_hashmap = HashMap::new();
                for element in http_response {
                    let (k, v) = element.split_once(':').unwrap();
                    temp_hashmap.insert(k.to_owned(), v.trim().to_owned());
                }
                println!("response: {:#?}", temp_hashmap);
                temp_hashmap
            };

            last_server_response = http_response.clone();

            {
                let mut guard = self.server_ip.lock().unwrap();
                if guard.is_empty() {
                    *guard = http_response.get("Ip").unwrap().to_owned();
                }
            }

            if http_response.get("State").unwrap() != "OK" {
                id = http_response.get("Id").unwrap().to_owned();
                request = id.to_string();
            }
            if http_response.get("Switch-To-Server").unwrap() == "true" {
                {
                    let ip_clone = self.current_ip.clone();
                    let mut guard = ip_clone.lock().unwrap();
                    *guard = format!("{}:3012", stream.local_addr().unwrap().ip());
                }
                server_mode_switch = true;
                break;
            }
            if http_response.get("New-Server").unwrap() != "None" {
                {
                    let ip_clone = self.current_ip.clone();
                    let mut guard = ip_clone.lock().unwrap();
                    *guard = format!("{}:3012", http_response.get("New-Server").unwrap());
                    ip = String::from(&*guard);
                }
            }

            request = format!("{}\n{}\n{}",
                id,
                manage_mutex(self.server_ip.clone(), None).unwrap(),
                sysinfo::get_info()
            );
            thread::sleep(Duration::from_secs(1));
        }
        server_mode_switch
    }

    fn tcp_server(&self) -> bool {
        {
            let mut guard = self.id.lock().unwrap();
            if guard.is_empty() {
                *guard = String::from(Uuid::new_v4());
            }
        }
        
        let addr = manage_mutex(self.current_ip.clone(), None).unwrap();
        
        // let mut hosts_dir: HashMap<String, String> = HashMap::new();
        let listener = TcpListener::bind(&addr).unwrap();
        println!("---\nListening on {}\n---", &addr);
        
        manage_mutex(self.server_ip.clone(), Some(listener.local_addr().unwrap().ip().to_string()));

        for stream in listener.incoming() {
            {
                let signal = self
                    .termination_signal
                    .lock()
                    .expect("Fallo en checar la señal");
                if *signal {
                    break;
                }
            }

            let stream = stream.expect("Fallo en inicial el stream");
            let guard = self.migration_mode.lock().unwrap();
            match *guard {
                true => {
                    self.handle_conecction(stream, true);
                    continue;
                }
                false => self.handle_conecction(stream, false),
            }
        }
        let mut guard = self.host_data.lock().unwrap();
        guard.clear();
        false
    }

    fn pulse(&self) {
        let current_ip = manage_mutex(self.current_ip.clone(), None).unwrap();
        let guard = self.reciver.lock().unwrap();
        let id_guard = self.id.lock().unwrap();

        'outer: loop {
            thread::sleep(Duration::from_secs(3));

            { // Send the server system information to the UI
                let mut host_data_guard = self.host_data.lock().unwrap();
                let server_ip_guard = self.server_ip.lock().unwrap();
                let mut req: Vec<String> = Vec::new();

                req.push(server_ip_guard.to_owned().to_string());
                let server_system_info: Vec<String> = sysinfo::get_info()
                    .lines()
                    .map(|line| line.to_string())
                    .take_while(|line| !line.is_empty())
                    .collect();
                
                for line in server_system_info {
                    req.push(line)
                }
                
                host_data_guard.insert(
                    id_guard.to_owned().to_string(),
                    req
                );
            }
            
            
            { // check if host is still connected
                let mut guard = self.host_data.lock().unwrap();
                let pulse_time: DateTime<Utc> = Utc::now();
                
                for (key, value) in &mut *guard {
                    let mut fecha_datetime = DateTime::parse_from_str(&value[7].as_str(), "%+") // %Y-%m-%d %H:%M:%S%.f %Z
                        .unwrap()
                        .with_timezone(&Utc);
                    let diff = (pulse_time - fecha_datetime).num_seconds();

                    if value.len() == 9 {
                        value.pop();
                    }
                    if diff >= 20 {
                        value.push(String::from("disconnected"));
                    } else {
                        value.push(String::from("connected"));
                    }
                }
            }

            self.sender
                .send(manage_mutex(self.host_data.clone(), None).unwrap())
                .unwrap();
            thread::sleep(Duration::from_millis(500));

            for _ in 0..2 {
                if let Ok(msg) = guard.try_recv() {
                    let msg: Vec<_> = msg
                    .lines()
                    .map(|result| result.to_string())
                    .take_while(|line| !line.is_empty())
                    .collect();
    
                    match msg[0].as_str() {
                        "First" => {
                            manage_mutex(self.new_server_id.clone(), dbg!(Some(msg[1].to_owned())));
                            manage_mutex(self.migration_mode.clone(), Some(true));
                            thread::sleep(Duration::from_secs(3));
            
                            manage_mutex(self.termination_signal.clone(), Some(true));
            
                            let mut stream = TcpStream::connect(&current_ip).unwrap(); // & in current ip
                            stream.write_all("OK\nNone\n".as_bytes()).unwrap(); // probably change request
            
                            manage_mutex(self.migration_mode.clone(), Some(false));
            
                            break 'outer;
                        },
                        "Second" => {
                            if msg[1] == "None" {
                                manage_mutex(self.fallback_ip.clone(), Some("None".to_string()));
                            } else {
                                let fallback_ip;
                                {
                                    let guard = self.host_data.lock().unwrap();

                                    fallback_ip = guard
                                        .get(&msg[1])
                                        .unwrap()[0]
                                        .to_owned();
                                }
    
                                manage_mutex(self.fallback_ip.clone(), Some(fallback_ip));
                            }
                        },
                        _ => ()
                    }
                }
            }
        }
    }

    fn handle_conecction(
        &self,
        mut stream: TcpStream,
        migration: bool,
    ) {
        let buf_reader = BufReader::new(&mut stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        println!("Request: {:#?}", http_request);

        let response;
        // If true, the server is in migration mode
        match migration {
            true => {
                let host_data_guard = self.host_data.lock().unwrap();
                let new_server_ip = host_data_guard
                    .get(&manage_mutex(self.new_server_id.clone(), None).unwrap())
                    .unwrap()[0]
                    .to_owned();

                let host_ip = stream.peer_addr().unwrap().ip().to_string();
                println!("host ip: {}", host_ip);
                if host_ip == new_server_ip {
                    let new_ip = stream.peer_addr().unwrap().ip().to_string();
                    response = format!(
                        "State: OK\nSwitch-To-Server: true\nNew-Server: None\nFallback-Server: None\nId: {}",
                        http_request[0]
                    );

                    let mut guard = self.current_ip.lock().unwrap();
                    *guard = format!("{}:3012", new_ip);
                } else {
                    let fallback_server_ip = manage_mutex(self.fallback_ip.clone(), None).unwrap();
                    response = format!(
                        "State: Unauthorized\nSwitchToServer: false\nNew-Server: {}\nFallback-Server: {}\nId: {}",
                        new_server_ip,
                        fallback_server_ip,
                        http_request[0]
                    );
                }
            }
            false => {
                if http_request[0] == "No-Id" {
                    let id = String::from(Uuid::new_v4());
                    response = format!(
                        "State: Unauthorized\nSwitch-To-Server: false\nNew-Server: None\nFallback-Server: None\nId: {}\nIp: {}",
                        id,
                        stream.peer_addr().unwrap().ip().to_string()
                    );
                } else {
                    {
                        let mut req = http_request.clone();
                        let k = req[0].clone();
                        req.remove(0);
                        let mut guard = self.host_data.lock().unwrap();
                        guard.insert(k, req);
                    }
                    let fallback_server_ip = manage_mutex(self.fallback_ip.clone(), None).unwrap();
                    response = format!(
                        "State: OK\nSwitch-To-Server: false\nNew-Server: None\nFallback-Server: {}\nId: {}",
                        fallback_server_ip,
                        http_request[0],
                    );
                }
            }
        }
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn manage_mutex<T>(mutex: Arc<Mutex<T>>, data: Option<T>) -> Option<T>
where
    T: Clone,
{
    let mut guard = mutex.lock().unwrap();
    let x = (*guard).clone();

    match data {
        Some(data) => {
            let d = data.clone();
            *guard = d;
            Some(data)
        }
        None => Some(x),
    }
}