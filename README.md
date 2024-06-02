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
  - [ ] Make a host ui (Deprecated)

---

## Naming/Brodcast

To-do list

- [x] add a ID argument in the server struct.
- [x] Find a way to diferienciate a **ConnectionRefused** error from being the first server and the one when the server is down. [^1]
- [x] Add the re-try feature in case that the server is down. [^2]

### Flow of the feature

Host try to connect to the server.

If server refuse the connection check the last response from the server.
1. There is a response.

>Try to connect to the server 3 more times.
>
> Seek the ip address to the fallback server in the response.
> 1. If there is no a fallback server end application.
> 2. If there is then erace the reponse history and try to connect. 
2. There is not a response.

> Try to connect to the ser ver 3 more times.
> 
> Become the server.

### Errors found

* There's a bug in witch if in the first pulse of the server the server doesnt end up in the first place then  when the server ends up in the first place it's going to try to
switch to himself. This also means that the host in the fist place will never get to switch to a server.

* Seems to be that if the server shuts down in the middle of handeling a connection the host recives a error type not supported in code.

### Possible errors

* The time set to determine the host status is 20 sec but the server pulse is 3 sec so there might be a chance that a host diconnect from the server and
the server in one of the 6 pulses that will pass before the host is set to desconnected it might end up in first or second place and the server might try to 
switch to a host that is already disconnected or set a fallback server to a host that is already disconnected.

[^1]: The **ConnectionRefused** errors may be diferenciated by copying the last response from the server.
In the case that there's no last response it means that you are the first device.
This can also be used to access the last message from the server in witch it should containt the ip address to the fallback server.

[^2]: Maybe it should try 3 times in an inteval of 1.5 sec.
---
