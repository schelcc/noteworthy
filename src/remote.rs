use ssh2::Session;
use std::net::TcpStream;

use crate::{config, intern_error};

pub fn connect_to_tablet() -> Result<Session, intern_error::Error> {
    let tcp = TcpStream::connect(config::SETTINGS["host"].as_str()).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_password(
        config::SETTINGS["username"].as_str(),
        config::SETTINGS["password"].as_str(),
    )?;

    Ok(sess)
}
