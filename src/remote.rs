/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

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
