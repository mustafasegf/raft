#![allow(unused_imports)]
#![allow(dead_code)]

use crate::parser::ArgsPeer;
use core::fmt::{Debug, Display};
use prost::Message;
use rand::RngCore;
use std::net::SocketAddr;
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
pub struct Node {
    pub id: i32,
    pub peers: Vec<Peer>,
    pub server: String,

    // cant use this this
    // use mailbox and channel
    // pub client: String,
    pub curent_term: u64,
    pub voted_for: Option<i32>,
    pub listener: Option<TcpListener>,
    // role: Role,
}

#[derive(Default, Debug)]
pub struct Peer {
    id: i32,
    server: String,
    tx: Option<OwnedWriteHalf>,
    rx: Option<OwnedReadHalf>,
}

impl Node {
    pub fn new(id: i32, server: String, peers: Vec<ArgsPeer>) -> Self {
        Self {
            id,
            server,
            peers: peers
                .into_iter()
                .map(|peer| Peer {
                    id: peer.id,
                    server: peer.server,
                    ..Default::default()
                })
                .collect(),
            curent_term: 1,
            voted_for: None,
            listener: None,
        }
    }

    pub fn to_peer(&self) -> Peer {
        Peer {
            id: self.id,
            server: self.server.clone(),
            tx: None,
            rx: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.server).await?;

        self.listener = Some(listener);
        println!("node {} binding on {}", &self.id, &self.server);
        Ok(())
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("node {} trying to connect", &self.id);
        for peer in &mut self.peers {
            let addr_server = peer.server.parse::<SocketAddr>()?;

            println!("node {} trying to connect to {}", &self.id, &peer.server);

            let stream = TcpStream::connect(addr_server).await?;

            println!(
                "server {} is connected to {}",
                &stream.peer_addr().unwrap(),
                &peer.server
            );
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
            let Some(listener) = self.listener.take() else {
                    continue;
                };
            break listener;
        };

        // self.peers.iter_mut().for_each(|peer| {
        //
        // });

        println!("node {} listener available at {}", &self.id, &self.server);
        // let (stream, addr) = listener.accept().await?;
        // println!("{} is connected to {}", &self.server, &addr);
        // let (rx, tx) = stream.into_split();

        // peer.rx = Some(rx);
        // peer.tx = Some(tx);
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

impl Peer {
    pub fn new(id: i32, server: String) -> Self {
        Self {
            id,
            server,
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
