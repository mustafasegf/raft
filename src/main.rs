use prost::Message;
use raft::message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration};

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

    println!("initial {:?}", &data);
    println!("initial byte {:?}", &data.encode_to_vec());
    println!(
        "initial byte with length {:?}",
        &data.encode_length_delimited_to_vec()
    );

    let ln = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = ln.local_addr().unwrap();

    println!("listening on {}", addr);

    let h1 = tokio::spawn(async move {
        let (mut stream, addr) = ln.accept().await.unwrap();
        println!("accepted from {}", addr.to_string());

        let len = stream.read_u8().await.unwrap();
        let mut buf: Vec<u8> = vec![0; len as usize];
        if let Err(err) = stream.read_exact(&mut buf).await {
            println!("read error: {}", err);
        };

        let req = message::Request::decode(&buf[..]).unwrap();

        println!("request {:?}", &req);

        let res = message::Response {
            term: 1,
            status: true,
        };
        match req.requests {
            Some(message::request::Requests::Vote(..)) => println!("vote request"),
            Some(message::request::Requests::Append(..)) => println!("append request"),
            None => println!("error: no request"),
        };

        stream
            .write_all(&res.encode_length_delimited_to_vec())
            .await
            .unwrap();
        println!("from server {:?}", &req);
    });

    let h2 = tokio::spawn(async move {
        let mut stream = TcpStream::connect(&addr).await.unwrap();
        stream
            .write_all(&data.encode_length_delimited_to_vec())
            .await
            .unwrap();

        let mut buf: Vec<u8> = Vec::with_capacity(1024);

        if let Err(err) = stream.read_to_end(&mut buf).await {
            println!("error {:?}", err);
        };
        let req = message::Response::decode(&buf[1..]).unwrap();
        println!("response {:?}", &req);
    });

    let (h1, h2) = tokio::join!(h1, h2);
    h1.unwrap();
    h2.unwrap();
}
