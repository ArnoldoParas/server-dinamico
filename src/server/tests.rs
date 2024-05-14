#![allow(unused)]
use crate::server::ServerWrapper;
use std::sync::mpsc;

#[test]
fn host_spawn() {
    let (tx, _rx) = mpsc::channel();
    let wrapper = ServerWrapper::new(tx);
    let host_switch = wrapper.server.host_server();

    assert_eq!(host_switch, true);
}
