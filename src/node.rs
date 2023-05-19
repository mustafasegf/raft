#![allow(unused)]

use anyhow::{Context, Result};
use core::fmt::{Debug, Display};
use futures::future::join_all;
use prost::Message;
use rand::Rng;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{ToSocketAddrs, UdpSocket};
use tokio::sync::broadcast::{Receiver, Sender};

// TODO: change to udp

use crate::message;

// pub enum Role<NodeId> {
//     Leader(NodeId),
//     Follower(NodeId),
//     Candidate(NodeId),
// }
//

#[derive(Default, Debug)]
pub struct NodeBuilder<Soc, Ser> {
    pub id: i32,
    pub peers: Vec<Peer>,
    pub server: Ser,

    pub curent_term: u64,
    pub voted_for: Option<i32>,
    pub socket: Soc,
}

pub struct NoSocket;
pub struct Socket(UdpSocket);

pub struct NoServer;

#[derive(Debug)]
pub struct Node {
    pub id: i32,
    pub peers: Vec<Peer>,
    pub server: SocketAddr,

    // cant use this this
    // use mailbox and channel
    // pub client: String,
    pub curent_term: u64,
    pub voted_for: Option<i32>,
    pub socket: UdpSocket,
    // role: Role,
}

#[derive(Default, Debug, Clone)]
pub struct Peer {
    pub id: i32,
    pub server: String,
}

impl NodeBuilder<NoSocket, NoServer> {
    pub fn new() -> Self {
        Self {
            id: 0,
            server: NoServer,
            peers: Vec::new(),
            curent_term: 1,
            voted_for: None,
            socket: NoSocket,
        }
    }

    pub async fn server(
        self,
        server: impl ToSocketAddrs,
    ) -> Result<NodeBuilder<NoSocket, SocketAddr>> {
        let server = tokio::net::lookup_host(server)
            .await?
            .next()
            .context("no server")?;
        let Self {
            id,
            peers,
            curent_term,
            voted_for,
            ..
        } = self;
        Ok(NodeBuilder {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            socket: NoSocket,
        })
    }
}

impl NodeBuilder<NoSocket, SocketAddr> {
    pub fn socket(self, socket: UdpSocket) -> NodeBuilder<Socket, SocketAddr> {
        let Self {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            ..
        } = self;
        NodeBuilder {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            socket: Socket(socket),
        }
    }

    pub async fn start(self) -> Result<NodeBuilder<Socket, SocketAddr>> {
        let socket = UdpSocket::bind(&self.server).await?;
        let Self {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            ..
        } = self;
        Ok(NodeBuilder {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            socket: Socket(socket),
        })
    }
}

impl NodeBuilder<Socket, SocketAddr> {
    pub fn build(self) -> Node {
        let Self {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            socket,
        } = self;
        Node {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            socket: socket.0,
        }
    }
}

impl<Soc, Ser> NodeBuilder<Soc, Ser> {
    pub fn peers(self, peers: Vec<Peer>) -> NodeBuilder<Soc, Ser> {
        let Self {
            id,
            server,
            curent_term,
            voted_for,
            socket,
            ..
        } = self;

        NodeBuilder {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            socket,
        }
    }
}

impl Node {
    pub fn builder() -> NodeBuilder<NoSocket, NoServer> {
        NodeBuilder {
            id: 0,
            server: NoServer,
            peers: Vec::new(),
            curent_term: 0,
            voted_for: None,
            socket: NoSocket,
        }
    }

    pub fn builder_with_data(id: i32) -> NodeBuilder<NoSocket, NoServer> {
        NodeBuilder {
            id,
            server: NoServer,
            peers : Vec::new(),
            curent_term: 1,
            voted_for: None,
            socket: NoSocket,
        }
    }

    pub async fn timer(&self, tx_msg: Sender<()>, mut rx_timer: Receiver<()>) {
        let mut rng = rand::thread_rng();
        loop {
            let random = rng.gen_range(1_000..3_000);
            let duration = Duration::from_millis(random);

            println!("timer set to {:?}", &duration);

            tokio::select! {
                _ = tokio::time::sleep(duration) => {
                    println!("timer expired");

                    tx_msg.send(()).unwrap();
                },
                _ = rx_timer.recv() => {
                    println!("timer reset");
                    continue;
                },

            };
        }
    }

    pub async fn listen(&self, tx_timer: Sender<()>) {
        let socket = &self.socket;
        loop {
            let mut buf = [0 as u8; 1];
            socket.recv_from(&mut buf).await.unwrap();
            let n = buf[0] as usize;

            let mut buf = vec![0 as u8; n + 1];
            // let mut buf = Vec::with_capacity(n + 1);
            socket.recv_from(&mut buf).await.unwrap();

            if buf.len() != n + 1 {
                println!("error receiving msg: {:?}", buf);
                continue;
            }

            let res = message::Request::decode_length_delimited(&buf[..]);
            if let Err(err) = res {
                println!("error decoding msg: {:?}", err);
                continue;
            };

            let res = res.unwrap();
            // println!("received msg: {:?} from {}", &res, &self.server);
            println!("received msg");
            tx_timer.send(()).unwrap();
        }
    }

    pub async fn sender(&self, mut rx_msg: Receiver<()>) {
        loop {
            rx_msg.recv().await.unwrap();
            let msg = message::Request {
                term: 1,
                requests: Some(message::request::Requests::Vote(message::VoteRequest {
                    term: 2,
                    candidate_id: 3,
                    last_log_idx: 4,
                    last_log_term: 5,
                })),
            };

            if let Err(err) = self.send_msg(&msg).await {
                println!("error sending msg: {:?}", err);
            };
        }
    }

    pub async fn send_msg(&self, msg: &message::Request) -> Result<()> {
        let socket = UdpSocket::bind("127.0.0.1:0").await?;
        let mut handles = Vec::with_capacity(self.peers.len());

        for (i, peer) in self.peers.iter().enumerate() {
            handles.push(peer.send_msg(&socket, msg));
        }
        // TODO: handle errors
        join_all(handles)
            .await
            .iter()
            .filter_map(|r| r.as_ref().err())
            .for_each(|e| println!("error sending msg: {:?}", e));

        Ok(())
    }
}

impl Peer {
    pub fn new(id: i32, server: String) -> Self {
        Self { id, server }
    }

    async fn send_msg(&self, socket: &UdpSocket, msg: &message::Request) -> Result<()> {
        // println!("sending msg: {:?} to {}", &msg, &self.server);
        println!("sending msg to {:?}", &self.server);
        socket
            .send_to(&msg.encode_length_delimited_to_vec(), &self.server)
            .await
            .context("can't send")?;
        Ok(())
    }
}
