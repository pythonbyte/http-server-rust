## Introduction

This project is an implementation of an HTTP server from scratch using Rust. The server is capable of handling dynamic requests and multiple requests concurrently using threads.

**Key Features**

* 🔧 Built entirely from scratch, no external HTTP libraries
* 🦀 Leverages Rust's safety and concurrency features
* 🌐 Implements core HTTP/1.1 functionalities
* 🚦 Handles multiple concurrent connections efficiently
* 📚 Educational resource for understanding HTTP internals and Rust networking

## Usage

To run the server, execute the following command:

```bash
cargo run
```

The server will start running on `127.0.0.1:4221`.

To test the server, open a browser and navigate to
```bash
curl -v http://localhost:4221/echo/test
```


**Response**

```bash
* Connected to localhost (127.0.0.1) port 4221
> GET /echo/test HTTP/1.1
> Host: localhost:4221
> User-Agent: curl/8.7.1
> Accept: */*
>
* Request completely sent off
< HTTP/1.1 200 OK
< Content-Type: text/plain
< Content-Length: 4
<
* Connection #0 to host localhost left intact
test%
```
