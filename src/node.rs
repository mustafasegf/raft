#![allow(unused)]

use anyhow::{Context, Result};
use core::fmt::{Debug, Display};
use futures::future::join_all;
use prost::Message;
use rand::Rng;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio::sync::broadcast::{Receiver, Sender};

// TODO: change to udp

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
    pub socket: Option<UdpSocket>,
    // role: Role,
}

#[derive(Default, Debug, Clone)]
pub struct Peer {
    pub id: i32,
    pub server: String,
}

impl Node {
    pub fn new(id: i32, server: String, peers: Vec<Peer>) -> Self {
        Self {
            id,
            server,
            peers,
            curent_term: 1,
            voted_for: None,
            socket: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let socket = UdpSocket::bind(&self.server).await?;

        self.socket = Some(socket);
        println!("binding on {}", &self.server);
        Ok(())
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
        let socket = self.socket.as_ref().unwrap();
        loop {
            let mut buf = [0 as u8; 1];
            socket.recv_from(&mut buf).await.unwrap();
            let n = buf[0] as usize;
            println!("received msg of size: {}", n);

            let mut buf = vec![0 as u8; n + 1];
            // let mut buf = Vec::with_capacity(n + 1);
            socket.recv_from(&mut buf).await.unwrap();

            if buf.len() != n + 1 {
                println!("error receiving msg: {:?}", buf);
                continue;
            }

            // let mut buf: Vec<u8> = vec![12, 8, 1, 18, 8, 8, 2, 16, 3, 24, 4, 32, 5];
            println!("received msg: {:?}", &buf);

            let res = message::Request::decode_length_delimited(&buf[..]);
            if let Err(err) = res {
                println!("error decoding msg: {:?}", err);
                continue;
            };

            let res = res.unwrap();
            println!("received msg: {:?} from {}", &res, &self.server);
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
        println!("sending msg: {:?} to {}", &msg, &self.server);
        socket
            .send_to(&msg.encode_length_delimited_to_vec(), &self.server)
            .await
            .context("can't send")?;
        Ok(())
    }
}
