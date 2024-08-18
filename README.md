# http-server-rust
Implementing an HTTP Server from scratch using Rust 🦀

## Introduction

This project is a simple implementation of an HTTP server from scratch using Rust. The server is capable of handling dynamic requests. The server is capable of handling multiple requests concurrently using threads.

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
