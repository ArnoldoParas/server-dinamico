use std::{
  net::TcpStream, 
  sync::{ Arc, Mutex}, 
  thread, 
  time::Duration
};


mod tests;

struct Server {
  default_ip: String,
  current_ip: Arc<Mutex<String>>,
  termination_signal: Arc<Mutex<bool>>
}

pub struct ServerWrapper { server: Arc<Server> }


impl ServerWrapper {
  /// Create a new TcpServer. When you create a nwe server it will keep rotating between
  /// a TcpListener and a TcpStream that will send data.
  /// 
  /// # Examples
  /// 
  /// ```
  /// use server::server::Server;
  /// 
  /// let server = Server::new();
  /// ```
  pub fn new() -> ServerWrapper {
    ServerWrapper {
      server: Arc::new(Server {
        default_ip: String::from("10.100.42.211:3012"),
        current_ip: Arc::new(Mutex::new(String::from(""))),
        termination_signal: Arc::new(Mutex::new(false))
      })
    }
  }

  pub fn run(&self) {
    let server_clone = self.server.clone();
    // let wrapper_clone = self.clone();
    thread::spawn(move|| {
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
  fn host_server(&self) -> bool {
    println!("hi from host");
    let mut server_mode_switch = false;
    let ip ;
    {
      let ip_clone = self.current_ip.clone();
      let ip_locked = ip_clone.lock().unwrap();
      ip = String::from(&*ip_locked);
    };
    let mut response = String::from(" \nH1\nH2");
    let mut stream;
      match TcpStream::connect(&ip) {
        Ok(s) => stream = s,
        Err(_) => {
          server_mode_switch = true;
        }
      }
      loop {
        if server_mode_switch {
          break;
        }

      }
    server_mode_switch
  }
  
  fn tcp_server(&self) -> bool {
    println!("hi from tcp");
    false
  }
}

// loop {
//     let mut stream;
//     match TcpStream::connect(&test) {
//         Ok(s) => stream = s,
//         Err(_) => {
//             thread::spawn(move ||{
//                 let termination_signal = Arc::new(Mutex::new(false));
//                 tcp_listener_thread(termination_signal, ip);
//                 println!("----------\nFINALIZADO\n----------");
//             });
//             break;
//         }
//     };

//     stream.write_all(response.as_bytes()).expect("fallo en enviar el error");
//     stream.shutdown(std::net::Shutdown::Write).unwrap();

//     let buf_reader = BufReader::new(&mut stream);
//     let http_response: Vec<_> = buf_reader
//         .lines()
//         .map(|result| result.unwrap())
//         .take_while(|line| !line.is_empty())
//         .collect();

//     if http_response[0] != "OK" {
//         response = format!("{}\nHeader 1\nHeader 2\nBody", http_response[2]);
//     } 
//     if http_response[1] != "None" {
//         {
//             let ip_clone = ip.clone();
//             let mut ip_locked = ip_clone.lock().unwrap();
//             *ip_locked = format!("{}:3012",&http_response[1]);
//         }
//         thread::spawn(move ||{
//             println!("Response: {:#?}", http_response);
            
//             let termination_signal = Arc::new(Mutex::new(false));
//             tcp_listener_thread(termination_signal, ip);
//             println!("FINALIZADO");
//         });
//         break;
//     }
//     println!("Response: {:#?}", http_response);
//     // println!("----------\nhost ip: {}\n----------",stream.peer_addr().unwrap());
//     thread::sleep(Duration::from_millis(1500));
// }
