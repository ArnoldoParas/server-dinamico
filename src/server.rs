use std::{
    collections::HashMap, 
    fmt::Error, 
    io::{prelude::*, BufReader}, 
    net::{Shutdown, TcpListener, TcpStream}, 
    sync::{mpsc, Arc, Mutex}, 
    thread, 
    time::Duration
};
use sysinfo::{Disks, Networks, System};
use uuid::Uuid;

mod tests;

struct Server {
    sender: mpsc::Sender<HashMap<String, Vec<String>>>,
    reciver: Mutex<mpsc::Receiver<String>>,
    current_ip: Arc<Mutex<String>>,
    termination_signal: Arc<Mutex<bool>>,
    switch_mode: Arc<Mutex<bool>>,
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
    pub fn new(tx: mpsc::Sender<HashMap<String, Vec<String>>>, rx: mpsc::Receiver<String>) -> ServerWrapper {
        ServerWrapper {
            server: Arc::new(Server {
                sender: tx,
                reciver: Mutex::new(rx),
                current_ip: Arc::new(Mutex::new(String::from("25.55.184.100:3012"))),
                termination_signal: Arc::new(Mutex::new(false)),
                switch_mode: Arc::new(Mutex::new(false)),
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
                thread::sleep(Duration::from_secs(1));
            }
        });
    }
}

#[allow(unused)]
impl Server {
    fn host_server(&self) -> bool {
        let mut server_mode_switch = false;

        manage_mutex(self.termination_signal.clone(), Some(false));

        let mut ip = manage_mutex(self.current_ip.clone(), None).unwrap();

        let mut id = String::from("NoId");
        let mut request = String::from(&id);
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
                println!("{:#?}", temp_hashmap);
                temp_hashmap
            };

            if http_response.get("State").unwrap() != "OK" {
                id = http_response.get("Id").unwrap().to_owned();
                request = format!("{}", id);
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
            if http_response.get("NewServer").unwrap() != "None" {
                {
                    let ip_clone = self.current_ip.clone();
                    let mut guard = ip_clone.lock().unwrap();
                    *guard = format!("{}:3012", http_response.get("NewServer").unwrap());
                    ip = String::from(&*guard);
                }
            }
            request = format!("{}\n{}", id, sysinfo());
            thread::sleep(Duration::from_millis(1500));
        }
        server_mode_switch
    }

    fn tcp_server(&self) -> bool {
        let addr;

        addr = manage_mutex(self.current_ip.clone(), None).unwrap();

        let mut hosts_dir: HashMap<String, String> = HashMap::new();

        let listener = TcpListener::bind(&addr).unwrap();
        println!("---\nListening on {}\n---", &addr);

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
            let guard = self.switch_mode.lock().unwrap();
            match *guard {
                true => {
                    self.handle_conecction(stream, &mut hosts_dir, true);
                    continue;
                }
                false => self.handle_conecction(stream, &mut hosts_dir, false),
            }
        }
        false
    }

    fn pulse(&self) {
        let current_ip = manage_mutex(self.current_ip.clone(), None).unwrap();
        let guard = self.reciver.lock().unwrap();
        
        loop {
            thread::sleep(Duration::from_secs(3));
            self.sender
            .send(manage_mutex(self.host_data.clone(), None).unwrap())
            .unwrap();
            thread::sleep(Duration::from_millis(500));
            match dbg!(guard.try_recv()) {
                Ok(msg) => {
                    manage_mutex(self.switch_mode.clone(), Some(true));
                    manage_mutex(self.new_server_id.clone(), Some(msg));
                    thread::sleep(Duration::from_secs(3));

                    manage_mutex(self.termination_signal.clone(), Some(true));
        
                    let mut stream = TcpStream::connect(&current_ip).unwrap();
                    stream.write_all("OK\nNone\n".as_bytes()).unwrap(); // probably change request
        
                    manage_mutex(self.switch_mode.clone(), Some(false));
                    break;
                },
                Err(_) => (),
            }

        }
    }

    fn handle_conecction(
        &self,
        mut stream: TcpStream,
        hosts_dir: &mut HashMap<String, String>,
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

        match migration {
            true => {
                let new_server_ip = hosts_dir.get(&manage_mutex(self.new_server_id.clone(), None).unwrap())
                    .unwrap()
                    .to_owned();

                let host_ip = stream.peer_addr().unwrap().ip().to_string();
                println!("host ip: {}", host_ip);
                if host_ip == new_server_ip {
                    let new_ip = stream.peer_addr().unwrap().ip().to_string();
                    response = format!(
                        "State: OK\nSwitchToServer: true\nNewServer: None\nId: {}",
                        http_request[0]
                    );

                    let mut guard = self.current_ip.lock().unwrap();
                    *guard = format!("{}:3012", new_ip);
                } else {
                    response = format!(
                        "State: Unauthorized\nSwitchToServer: false\nNewServer: {}\nId: {}",
                        new_server_ip, http_request[0]
                    );
                }
            }
            false => {
                if http_request[0] == "NoId" {
                    let id = String::from(Uuid::new_v4());
                    response = format!(
                        "State: Unauthorized\nSwitchToServer: false\nNewServer: None\nId: {}",
                        id
                    );
                    hosts_dir.insert(id, stream.peer_addr().unwrap().ip().to_string());
                } else {
                    {
                        let mut req = http_request.clone();
                        let k = req[0].clone();
                        req.remove(0);
                        req.push("connected".to_string());
                        let mut guard = self.host_data
                            .lock()
                            .unwrap();
                        guard.insert(k, req);
                    }
                    response = format!(
                        "State: OK\nSwitchToServer: false\nNewServer: None\nId: {}",
                        http_request[0]
                    );
                    println!(
                        "----------\nhost ip: {}\n----------\n",
                        stream.peer_addr().unwrap()
                    );
                    // self.sender.send(http_request).unwrap();
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
            return Some(data);
        }
        None => Some(x),
    }
}

fn sysinfo() -> String {
    #![allow(unused)]
    let mut sys = System::new_all();
    let mut bandwith: u64 = 0;
    let mut freebandwith: u64 = 0;
    let mut disk_space = 0;
    sys.refresh_all();

    let mut networks = Networks::new_with_refreshed_list();
    for (interface_name, network) in &networks {
        bandwith = network.total_transmitted() + network.total_received();
    }

    networks.refresh();
    for (interface_name, network) in &networks {
        freebandwith = bandwith - (network.transmitted() + network.received());
    }

    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        disk_space = disk.available_space() / 1_000_000_000;
        break;
    }

    let cpu = sys.cpus().get(0).unwrap();

    let sysinfo = format!(
        "{}\n{:.2}\n{}\n{}\n{}\n{}",
        System::host_name().unwrap(),
        cpu.cpu_usage(),
        sys.used_memory() / 1_000_000,
        freebandwith / 1_000_000,
        disk_space,
        sys.total_memory() / 1_000_000
    );
    sysinfo
}
