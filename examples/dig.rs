use std::net::{SocketAddrV4, UdpSocket};
use std::env;

extern crate r53;
use r53::{Name, Message, RRType, MessageRender, InputBuffer};


fn main() {
    let server_addr = env::args().nth(1).unwrap_or(
        "114.114.114.114:53".to_string(),
    );
    let server_addr = server_addr.parse::<SocketAddrV4>().unwrap();

    let socket = UdpSocket::bind("0.0.0.0:0".parse::<SocketAddrV4>().unwrap())
        .expect("bind udp socket failed");


    let name = env::args().nth(2).unwrap_or("www.knet.cn".to_string());
    let name = Name::new(name.as_str(), true).unwrap();

    let qtype = env::args().nth(3).unwrap_or("a".to_string());
    let qtype = RRType::from_string(qtype.as_ref()).expect("unknown qtype");

    let query = Message::with_query(name, qtype);
    let mut render = MessageRender::new();
    query.rend(&mut render);
    socket.send_to(render.data(), server_addr).unwrap();

    let mut buf = [0; 512];
    socket.recv_from(&mut buf).unwrap();
    let response = Message::from_wire(&mut InputBuffer::new(&buf)).unwrap();
    println!("get response: {}", response.to_string());
}
