# stunner

Yet another set of STUN ([RFC5389](https://datatracker.ietf.org/doc/html/)) client and server using [`stun-coder`](https://github.com/Vagr9K/rust-stun-coder)

# stunner-client

inspired by [`stunclient`](https://github.com/NATTools/stunclient), instructions:
```
stunner_client

USAGE:
    stunner_client [OPTIONS] <REMOTE_ADDR> <REMOTE_PORT>

ARGS:
    <REMOTE_ADDR>    Destination STUN server
    <REMOTE_PORT>    Destination STUN port

OPTIONS:
    -h, --help                     Print help information
        --localaddr <LOCALADDR>    Specify one of the available IP addresses assigned to a network
                                   interface present on the host [default: 0]
        --localport <LOCALPORT>    Specify the UDP or TCP port that the primary and alternate
                                   interfaces listen on as the primary port for binding requests. If
                                   not specified, a randomly available port chosen by the system is
                                   used [default: 0]
    -V, --version                  Print version information
```
example:\
`$ stunner-client stun.l.google.com 19302  `

# stunner-server

```
stunner_server 0.1.0

USAGE:
    stunner_server [OPTIONS]

OPTIONS:
    -h, --help           Print help information
        --port <PORT>    Specify the listening port where the server should run, by default 19302 is
                         used [default: 3478]
    -V, --version        Print version information
```
