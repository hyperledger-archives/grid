// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate splinter;

use std::env;
use std::thread;
use std::time::{Duration, Instant};

use splinter::{
    mesh::{Envelope, Mesh},
    transport::{socket::RawTransport, Listener, Transport},
};

// An example of creating a Transport and a Mesh, and doing reads and writes in a single thread.
// A single background thread is spawned for accepting new connections.
//
// To try this out, do
// ```
// cargo run --example mesh-echo-peer \
//   {number of connections to make} \
//   {bind} \
//   {list of endpoints to connect to}
// ```
fn main() {
    let mut args = env::args().skip(1);
    let connections: usize = args.next().unwrap().parse().unwrap();
    let endpoint = args.next().unwrap();
    let peers: Vec<String> = args.collect();

    let mut transport = RawTransport::default();
    let mesh = Mesh::new(512, 128);

    listen(mesh.clone(), transport.listen(&endpoint).unwrap());
    let ids = connect(&mut transport, &mesh, &peers, connections);

    for id in &ids {
        send(&mesh, *id, b"hello");
    }

    let mut tx: usize = 0;
    let mut rx: usize = 0;
    let mut start: Instant = Instant::now();

    loop {
        match mesh.recv() {
            Ok(envelope) => match envelope.payload() {
                b"hello" => {
                    rx += 1;
                    send(&mesh, envelope.id(), b"world");
                    tx += 1;
                }
                b"world" => {
                    rx += 1;
                    send(&mesh, envelope.id(), b"hello");
                    tx += 1;
                }
                _ => (),
            },
            Err(err) => {
                eprintln!("Error receiver: {:?}", err);
                break;
            }
        }

        if start.elapsed().as_secs() > 2 {
            println!(
                "tx = {} kB/s, rx = {} kB/s",
                kbytes_per_sec(tx, start),
                kbytes_per_sec(rx, start)
            );
            start = Instant::now();
            tx = 0;
            rx = 0;
        }
    }
}

fn kbytes_per_sec(x: usize, since: Instant) -> usize {
    (x * b"hello".len()) / (since.elapsed().as_secs() as usize) / 1024
}

fn listen(mesh: Mesh, mut listener: Box<dyn Listener>) {
    thread::spawn(move || {
        println!("Listening on {}...", listener.endpoint());
        loop {
            match listener.accept() {
                Ok(connection) => {
                    println!(
                        "Accepted new connection from {}",
                        connection.remote_endpoint()
                    );
                    if let Err(err) = mesh.add(connection) {
                        eprintln!("Error adding connection to mesh: {:?}", err);
                    }
                }
                Err(err) => {
                    eprintln!("Error accepting connection: {:?}", err);
                }
            }
        }
    });
}

fn connect<T: Transport>(transport: &mut T, mesh: &Mesh, peers: &[String], n: usize) -> Vec<usize> {
    if peers.len() == 0 {
        return Vec::with_capacity(0);
    }

    let mut ids = Vec::with_capacity(n);
    for i in 0..n {
        loop {
            let peer = &peers[i % peers.len()];
            println!("Connecting to {}", peer);
            match transport.connect(peer).map(|conn| mesh.add(conn)) {
                Ok(Ok(id)) => {
                    ids.push(id);
                    break;
                }
                Ok(Err(err)) => {
                    eprintln!("Error adding connection to mesh: {:?}", err);
                    break;
                }
                Err(_err) => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            }
        }
    }
    ids
}

fn send(mesh: &Mesh, id: usize, msg: &[u8]) {
    if let Err(err) = mesh.send(Envelope::new(id, msg.to_vec())) {
        eprintln!("Error sending to {}: {:?}", id, err);
    }
}
