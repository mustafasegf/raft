#![allow(unused_imports)]
#![allow(dead_code)]

use core::fmt::{Debug, Display};
use prost::Message;
use rand::RngCore;
use std::net::{SocketAddr, SocketAddrV4};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpSocket, TcpStream};

use crate::message;

// pub enum Role<NodeId> {
//     Leader(NodeId),
//     Follower(NodeId),
//     Candidate(NodeId),
// }

#[derive(Default, Debug)]
pub struct Node<NodeId> {
    pub id: NodeId,
    pub peers: Vec<Peer<NodeId>>,
    pub server: String,
    pub client: String,
    pub socket: Option<TcpSocket>,

    pub curent_term: u64,
    pub voted_for: Option<NodeId>,
    pub listener: Option<TcpListener>,
    // role: Role,
}

#[derive(Default, Debug)]
pub struct Peer<NodeId> {
    id: NodeId,
    server: String,
    client: String,
    tx: Option<OwnedWriteHalf>,
    rx: Option<OwnedReadHalf>,
}

impl<NodeId> Node<NodeId>
where
    NodeId: Ord + Copy + Display + Debug,
{
    pub fn new(id: NodeId, server: String, client: String, peers: Vec<Peer<NodeId>>) -> Self {
        Self {
            id,
            server,
            client,
            peers,
            curent_term: 1,
            voted_for: None,
            listener: None,
            socket: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.server).await?;

        self.listener = Some(listener);
        println!("node {} listening on {}", &self.id, &self.server);
        Ok(())
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("node {} trying to connect", &self.id);
        for peer in &mut self.peers {
            let addr_server = peer.server.parse()?;
            let addr_client = self.client.parse()?;

            println!("node {} trying to connect to {}", &self.id, &addr_server);

            let stream = loop {
                let socket = TcpSocket::new_v4()?;

                socket.bind(addr_client)?;
                let Ok(stream) = socket.connect(addr_server).await else {
                    continue;
                };
                println!(
                    "client {} is connected to {}",
                    addr_server,
                    stream.peer_addr()?
                );
                break stream;
            };

            println!("server {} is connected to {}", &self.client, &peer.server);
            let (rx, tx) = stream.into_split();
            peer.rx = Some(rx);
            peer.tx = Some(tx);
        }
        println!("node {} connected to all peers", &self.id);
        Ok(())
    }

    pub async fn listen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("node {} trying to listen", &self.id);

        let listener = loop {
            let Some(mut listener) = self.listener.take() else {
                    continue;
                };
            break listener;
        };
        println!("node {} listener available at {}", &self.id, &self.server);
        let (mut stream, addr) = listener.accept().await?;
        println!("{} is connected to {}", &self.server, &addr);
        let (rx, tx) = stream.into_split();

        // address is different
        println!(
            "peer server: {}, self host: {}",
            &addr.to_string(),
            &self.server
        );

        // print all peer connect
        for peer in &self.peers {
            println!("peer connect: {}", &peer.client);
        }

        let mut peer = self
            .peers
            .iter_mut()
            .find(|peer| peer.client == addr.to_string())
            .unwrap();

        peer.rx = Some(rx);
        peer.tx = Some(tx);
        Ok(())
    }

    pub async fn send_msg(
        &mut self,
        msg: &message::Request,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for peer in &mut self.peers {
            peer.send_msg(msg).await?;
        }
        Ok(())
    }
}

impl<NodeId> Peer<NodeId>
where
    NodeId: Ord + Copy + Display + Debug,
{
    pub fn new(id: NodeId, server: String, client: String) -> Self {
        Self {
            id,
            server,
            client,
            tx: None,
            rx: None,
        }
    }

    pub async fn send_msg(
        &mut self,
        msg: &message::Request,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = &mut self.tx {
            println!("sending msg: {:?} to {}", &msg, &self.server);
            tx.write_all(&msg.encode_length_delimited_to_vec()).await?;
        }
        Ok(())
    }

    pub async fn read_msg(&mut self) -> Result<message::Response, Box<dyn std::error::Error>> {
        if let Some(rx) = &mut self.rx {
            let mut buf: Vec<u8> = Vec::with_capacity(1024);
            rx.read_to_end(&mut buf).await?;
            let res = message::Response::decode(&buf[1..])?;
            println!("received msg: {:?} from {}", &res, &self.server);
            return Ok(res);
        }
        Ok(message::Response::default())
    }
}
