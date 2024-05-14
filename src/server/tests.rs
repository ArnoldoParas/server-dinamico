#![allow(unused)]
use crate::server::ServerWrapper;
use std::sync::{mpsc, Arc, Mutex};

use super::manage_mutex;

// #[test]
// fn host_spawn() {
//     let (tx, _rx) = mpsc::channel();
//     let wrapper = ServerWrapper::new(tx);
//     let host_switch = wrapper.server.host_server();

//     assert_eq!(host_switch, true);
// }

#[test]
fn m_mutex() {
    let mut modify = Arc::new(Mutex::new(false));
    let see = Arc::new(Mutex::new("Hi".to_string()));
    // let see_cpy = see.clone();

    // manage_mutex(see_cpy, Some("?".to_string()));

    let mut modify_copy = modify.clone();
    assert_eq!(manage_mutex(modify, None), Some(false));
    assert_eq!(manage_mutex(modify_copy, Some(true)), Some(true));
    assert_eq!(manage_mutex(see, Some("Hi :D".to_string())), Some("Hi :D".to_string()));
}
