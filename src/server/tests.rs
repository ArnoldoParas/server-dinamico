use crate::server::*;

#[test]
fn host_spawn() {
  let wrapper = ServerWrapper::new();
  let host_switch = wrapper.server.host_server();

  assert_eq!(host_switch, true);
}