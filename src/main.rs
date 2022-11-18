use prost::Message;
use raft::message;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
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
    println!("initial {:?}", &data.encode_to_vec());

    let ln = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = ln.local_addr().unwrap();

    println!("listening on {}", addr);

    let h1 = tokio::spawn(async move {
        let (stream, addr) = ln.accept().await.unwrap();
        println!("accepted from {}", addr.to_string());
        let mut stream = BufReader::new(stream);

        let mut buf: Vec<u8> = Vec::with_capacity(1024);
        loop {
            tokio::select! {
                res = stream.read_u8() => {
                    match res {
                        Ok(b) => buf.push(b),
                        Err(e)  if e.kind() == tokio::io::ErrorKind::UnexpectedEof => break,
                        Err(e) => {
                            println!("error: {}", e);
                            break;
                        },
                    };
                },
                _ = sleep(Duration::from_millis(10)) => break,
            };
        }

        let req = message::Request::decode(&buf[..]).unwrap();

        println!("request {:?}", &req);

        match req.requests {
            Some(message::request::Requests::Vote(..)) => {
                println!("vote request");
                let res = message::Response {
                    term: 1,
                    status: true,
                };

                stream.write_all(&res.encode_to_vec()).await.unwrap();
            }
            Some(message::request::Requests::Append(..)) => {
                println!("append request");
                let res = message::Response {
                    term: 1,
                    status: true,
                };

                stream.write_all(&res.encode_to_vec()).await.unwrap();
            }
            None => println!("error: no request"),
        };
        println!("from server {:?}", &req);
    });

    let h2 = tokio::spawn(async move {
        let mut stream = TcpStream::connect(&addr).await.unwrap();
        stream.write_all(&data.encode_to_vec()).await.unwrap();

        let mut buf: Vec<u8> = Vec::with_capacity(1024);

        if let Err(err) = stream.read_to_end(&mut buf).await {
            println!("error {:?}", err);
        };
        let req = message::Response::decode(&buf[..]).unwrap();
        println!("response {:?}", &req);
    });

    let (h1, h2) = tokio::join!(h1, h2);
    h1.unwrap();
    h2.unwrap();
}
