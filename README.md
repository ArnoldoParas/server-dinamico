# servidor dinamico para clase de sistemas distribuidos y paralelos


## Merge server and client 

To do list
- [x] Switch from web sockets to generic TCP pull/request format
  - [x] Create module fot the Tcp server
  - [x] Create test module
  - [x] Create the TCP pull/request template
  - [x] Implement class metods and test
  - [x] Optimize the prototype
  - [x] Replace web socket logic with the new Tcp module
  - [x] Test the new module
- [ ] Configure the GUI 
  - [x] Set the GUI refresh rate to 3-4 sec
  - [x] Add Sender/Reciver from the GUI to the Tcp module
  - [ ] Make a host ui

---

## Naming/Brodcast

To-do list

- [ ] add a ID argument in the server struct.
- [ ] Find a way to diferienciate a **ConnectionRefused** error from being the first server and the one when the server is down. [^1]
- [ ] Add the re-try feature in case that the server is down. [^2]


[^1]: The **ConnectionRefused** errors may be diferenciated by copying the last response from the server.
In the case that there's no last response it means that you are the first device.
This can also be used to access the last message from the server in witch it should containt the ip address to the fallback server.

[^2]: Maybe it should try 3 times in an inteval of 1.5 sec.
