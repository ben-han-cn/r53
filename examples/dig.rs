use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;

extern crate clap;
extern crate r53;

use clap::{App, Arg};
use r53::{edns::Edns, util::hex::to_hex, Message, MessageBuilder, MessageRender, Name, RRType};

fn main() {
    let matches = App::new("dig")
        .arg(
            Arg::with_name("port")
                .help("target port")
                .short("p")
                .long("port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("server")
                .help("server to send query")
                .value_name("SERVER")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("qname")
                .help("domain to query")
                .value_name("DOMAIN")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("qtype")
                .help("type to query")
                .value_name("TYPE")
                .index(3),
        )
        .get_matches();

    let mut server_addr = matches.value_of("server").unwrap().to_string();
    let port = match matches.value_of("port") {
        Some(p) => p.as_ref(),
        None => "53",
    };
    if server_addr.starts_with("@") == false {
        println!("server address isn't start with @");
        return;
    }
    server_addr.remove(0);
    server_addr.push_str(":");
    server_addr.push_str(port);
    let server_addr = server_addr.parse::<SocketAddrV4>().unwrap();

    let socket = UdpSocket::bind("0.0.0.0:0".parse::<SocketAddrV4>().unwrap())
        .expect("bind udp socket failed");
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
        .unwrap();

    let name = matches.value_of("qname").unwrap();
    let name = Name::new(name).unwrap();

    let qtype = match matches.value_of("qtype") {
        Some(t) => t.as_ref(),
        None => "a",
    };
    let qtype = RRType::from_str(qtype.as_ref()).expect("unknown qtype");

    let mut query = Message::with_query(name, qtype);
    let mut builder = MessageBuilder::new(&mut query);
    builder.edns(Edns {
        versoin: 0,
        extened_rcode: 0,
        udp_size: 4096,
        dnssec_aware: false,
        options: None,
    });
    builder.done();
    let mut render = MessageRender::new();
    query.to_wire(&mut render);
    socket.send_to(render.data(), server_addr).unwrap();

    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((len, _)) if len > 0 => {
            //println!("{}", to_hex(&buf[0..len]));
            let response = Message::from_wire(&buf).unwrap();
            println!("get response: {}", response.to_string());
        }
        _ => println!("timeout"),
    }
}
