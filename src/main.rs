use std::vec;

use futures::future::join_all;
use prost::Message;
use raft::message;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let data = message::Request {
        term: 1,
        requests: Some(message::request::Requests::Vote(message::VoteRequest {
            term: 1,
            candidate_id: 1,
            last_log_idx: 1,
            last_log_term: 1,
        })),
    };

    println!("initial {:?}", &data.encode_to_vec());

    let ln = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = ln.local_addr().unwrap();

    let mut handles = vec![];
    println!("listening on {}", addr);

    let h1 = tokio::spawn(async move {
        let (stream, _) = ln.accept().await.unwrap();
        let mut stream = BufReader::new(stream);

        let mut buf: Vec<u8> = Vec::with_capacity(1024);
        stream.read_to_end(&mut buf).await.unwrap();

        let req = message::Request::decode(&buf[..]).unwrap();
        println!("received {:?}", &buf);

        println!(
            "from server {:?}",
            req
        );
    });

    let h2 = tokio::spawn(async move {
        let mut stream = TcpStream::connect(&addr).await.unwrap();
        stream.write_all(&data.encode_to_vec()).await.unwrap();
    });

    handles.push(h1);
    handles.push(h2);

    join_all(handles).await;
}
