//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    io::Write,
    os::unix::net::UnixStream,
    sync::mpsc,
};

use crate::db;
use crate::db::{
    Kind,
};
use crate::err;

use serde_json::Value;
use log::error;

// Deserializes a JSON Value struct into a db::Comm,
// ready for passing to the ledger worker thread.
// Serialize/Deserialize serde traits apparently
// don't play well with enums.
pub fn to_comm(json: &Value, tx: mpsc::Sender<db::Reply>) -> Option<db::Comm> {
    let json_kind = json["kind"].as_str()?;
    let json_kind = json_kind.to_lowercase();
    let kind: db::Kind = match &json_kind[..] {
        "register" => Kind::Register,
        "whoami" => Kind::Whoami,
        "rename" => Kind::Rename,
        "send" => Kind::Send,
        "sign" => Kind::Sign,
        "balance" => Kind::Balance,
        "verify" => Kind::Verify,
        "contest" => Kind::Contest,
        "audit" => Kind::Audit,
        "resolve" => Kind::Resolve,
        "second" => Kind::Second,
        "query" => Kind::Query,             // Query and Disconnect are internal
        "disconnect" => Kind::Disconnect,   // values for miscellaneous database
        &_ => return None,                  // queries and shutting down the DB.
    };

    let args = json["args"].as_str()?
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    Some(
        db::Comm::new(
            Some(kind), 
            Some(args), 
            Some(tx),
        )
    )
}

// Takes a string, outputs JSON.
// If there's an error, sends an error down the socket.
// TODO: This shouldn't do that last bit. Leave that up to the caller.
pub fn from_str(json_in: &str, conn: Option<&mut UnixStream>) -> Option<serde_json::Value> {
    return match serde_json::from_str(&json_in) {
        Ok(val) => Some(val),
        Err(err) => {
            let err = format!("{}", err);
            let out = err::Resp::new(02, "JSON Error", &err);

            error!(
                "\nError {}:\n{}\n{}", 
                out.code(), 
                out.kind(), 
                out.details(),
            );
            
            let out = out.to_bytes();

            if let Some(conn) = conn {
                conn.write_all(&out).unwrap();
            }
            None
        }
    }
}