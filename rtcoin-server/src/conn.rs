//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    error::Error,
    io::BufRead,
    io::BufReader,
    os::unix::net::{
        SocketAddr,
        UnixStream,
    },
    path::Path,
    sync::mpsc,
};

use crate::db;

// First handler for each new connection.
pub fn init(conn: UnixStream, pipe: mpsc::Sender::<db::Comm>) {
    let stream = BufReader::new(conn);
    for line in stream.lines() {
        println!("{}", line.unwrap());
    }

    let (tx, rx) = mpsc::channel::<db::Reply>();
    pipe.send(
        db::Comm::new(
            db::Kind::BulkQuery, 
            db::Trans::Destination("Henlo".into()), 
            tx,
            )
    ).unwrap();

    let resp: Option<db::Reply> = match rx.recv() {
        Ok(val) => Some(val),
        Err(err) => {
            eprintln!("Error in Ledger Worker Response: {}", err);
            None
        }
    };

    if let None = resp {
        eprintln!("Closing connection");
        return
    } else if let Some(val) = resp {
        println!("{:#?}", val);
    }
}

// Grabs the connection's peer address. Used to
// name the thread spawned for the connection
// so we can better pinpoint which thread caused
// a given problem during debugging.
pub fn addr(addr: &SocketAddr) -> String {
    if let Some(n) = addr.as_pathname() {
        let path = n;
        if let Some(n) = path.to_str() {
            return n.to_string()
        };
    };

    return String::from("Unknown Thread")
}
