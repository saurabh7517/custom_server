use std::thread;
use std::sync::{Mutex,Arc};
use std::sync::mpsc;
use std::io::{Read, Result, Write};
use std::str;
use std::net::{TcpListener,TcpStream,SocketAddr};
use std::thread::JoinHandle;

fn main() {
    let hostname_and_port: &str = "127.0.0.1:8000";
    let listen = TcpListener::bind(hostname_and_port).expect("Could not bind");
    println!("Starting server....");

    let mut connection_thread_list:Vec<JoinHandle<_>> = Vec::with_capacity(2);

    let wrapped_listener = Arc::new(Mutex::new(listen));

    let (tx,rx) = mpsc::channel();

    let pooling_connection_thread = thread::spawn(move || {
        println!("Waiting for client to establish connection...");
        let unwrap_listener = wrapped_listener.clone();
        loop {
            let listener = unwrap_listener.lock().unwrap();
            println!("Incoming connection");
            let conn: Result<(TcpStream, SocketAddr)> = listener.accept();
            let connection = conn.unwrap();
            println!("Connection {} sent to reading thread",connection.1);
            tx.send(connection).unwrap();
        }
    });

    for connection in rx {
        let mut tcp_stream = connection.0.try_clone().unwrap();
        let socket_address = connection.1.clone();

        connection_thread_list.push(thread::spawn(move || {
            let mut buf = [0; 512];
            //static ack sent to client after receiving a message.
            let message_sent_to_client = format!("Data received at server");
            loop {
                println!("Reading from client address :: {}", socket_address);
                let data_size_read = tcp_stream.read(&mut buf).unwrap();
                println!("Data read in bytes :: {}", data_size_read);
                let message = str::from_utf8(&mut buf[..]).expect("cannot convert to utf-8").to_owned();
                tcp_stream.write(message_sent_to_client.as_bytes()).expect("unable to write data to buffer");
                tcp_stream.flush().unwrap();
                println!("{}",message);
                buf = [0; 512];
                if message.contains("quit") {
                    // Disconnect client from server
                    println!("Shutting down connection with client server :: {}",socket_address);
                    break;
                }
            }
        }));
    }

    for some_thread in connection_thread_list {
        some_thread.join().unwrap();
    }

    pooling_connection_thread.join().unwrap();

}
