#![allow(unused)]

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use core::fmt::{Debug, Display};
use futures::future::join_all;
use prost::Message;
use rand::rngs::{StdRng, ThreadRng};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{ToSocketAddrs, UdpSocket};
use tokio::sync::broadcast::{Receiver, Sender};

use crate::message;

#[derive(Debug, Clone)]
pub enum Role {
    Leader,
    Follower,
    Candidate { votes: u64, vote_set: HashSet<u64> },
}

#[derive(Default, Debug)]
pub struct NodeBuilder<Soc, Ser> {
    pub id: u64,
    pub peers: Vec<Peer>,
    pub server: Ser,

    pub curent_term: u64,
    pub voted_for: Option<u64>,
    pub socket: Soc,

    pub log_index: u64,
    pub log_term: u64,
}

pub struct NoSocket;
pub struct Socket(UdpSocket);

pub struct NoServer;

pub struct Leader;
pub struct Candidate;
pub struct Follower;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u64,
    // TODO: change to map
    pub peers: Vec<Peer>,
    pub server: SocketAddr,

    pub current_term: u64,
    pub voted_for: Option<u64>,
    pub role: Role,

    pub log_index: u64,
    pub log_term: u64,
}

pub struct NodeSocket {
    pub id: u64,
    pub socket: UdpSocket,
    pub server: SocketAddr,
}

#[derive(Default, Debug, Clone)]
pub struct Peer {
    pub id: u64,
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
            log_index: 0,
            log_term: 0,
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
            log_index,
            log_term,
            ..
        } = self;
        Ok(NodeBuilder {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            log_index,
            log_term,
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
            log_index,
            log_term,
            ..
        } = self;
        NodeBuilder {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            socket: Socket(socket),
            log_index,
            log_term,
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
            log_index,
            log_term,
            ..
        } = self;
        Ok(NodeBuilder {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            socket: Socket(socket),
            log_index,
            log_term,
        })
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
            log_index,
            log_term,
            ..
        } = self;

        NodeBuilder {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            socket,
            log_index,
            log_term,
        }
    }
}

impl NodeBuilder<Socket, SocketAddr> {
    pub fn build(self) -> (Node, NodeSocket) {
        let Self {
            id,
            server,
            peers,
            curent_term,
            voted_for,
            socket,
            log_index,
            log_term,
        } = self;

        let node = Node {
            id,
            server,
            peers,
            current_term: curent_term,
            voted_for,
            role: Role::Follower,
            log_index,
            log_term,
        };

        let node_socket = NodeSocket {
            id,
            socket: socket.0,
            server,
        };

        (node, node_socket)
    }
}

pub async fn timer(
    rng: &mut StdRng,
    tx_timer: &Sender<()>,
    rx_msg: &mut Receiver<message::Data>,
) -> Result<()> {
    let random = rng.gen_range(2_000..3_000);
    let duration = Duration::from_millis(random);

    println!("timer set to {:?}", &duration);

    tokio::select! {
        _ = tokio::time::sleep(duration) => {
            println!("timer expired");
            tx_timer.send(())?;
        },
        msg = rx_msg.recv() => {
            println!("timer reset");
        },

    };
    Ok(())
}

impl NodeSocket {
    pub async fn listen(&self, tx_msg: &Sender<message::Data>) -> Result<()> {
        let mut socket = &self.socket;

        let mut buf = [0 as u8; 1];
        socket.recv_from(&mut buf).await?;
        let n = buf[0] as usize;

        let mut buf = vec![0 as u8; n + 1];
        // let mut buf = Vec::with_capacity(n + 1);
        socket.recv_from(&mut buf).await?;

        if buf.len() != n + 1 {
            println!("error receiving msg: {:?}", buf);
            return Err(anyhow!("error receiving msg"));
        }

        let res = message::Data::decode_length_delimited(&buf[..]);
        let msg = match res {
            Ok(msg) => msg,
            Err(err) => {
                println!("error decoding msg: {:?}", err);
                return Err(anyhow!("error decoding msg"));
            }
        };

        // println!("received msg: {:?} from {}", &res, &self.server);
        println!("received msg");
        tx_msg.send(msg)?;

        Ok(())
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
            log_index: 0,
            log_term: 0,
        }
    }

    pub fn builder_with_data(id: u64) -> NodeBuilder<NoSocket, NoServer> {
        NodeBuilder {
            id,
            server: NoServer,
            peers: Vec::new(),
            curent_term: 1,
            voted_for: None,
            socket: NoSocket,
            log_index: 0,
            log_term: 0,
        }
    }

    // pub async fn sender(&self, mut rx_msg: Receiver<()>) -> Result<()> {
    //     rx_msg.recv().await.unwrap();
    //     let msg = message::Data {
    //         term: 1,
    //         requests: Some(message::request::Requests::Vote(message::VoteRequest {
    //             term: 2,
    //             candidate_id: 3,
    //             last_log_idx: 4,
    //             last_log_term: 5,
    //         })),
    //     };
    //
    //     match self.send_msg(&msg).await {
    //         Err(err) => {
    //             println!("error sending msg: {:?}", err);
    //             Err(anyhow!("error sending msg"))
    //         }
    //         _ => Ok(()),
    //     }
    // }

    pub async fn send_msg(&self, msg: &message::Data) -> Result<()> {
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

    pub async fn step(
        &mut self,
        tx_msg: &Sender<message::Data>,
        rx_msg: &mut Receiver<message::Data>,
        tx_timer: &Sender<()>,
        rx_timer: &mut Receiver<()>,
    ) -> Result<()> {
        match &self {
            Node {
                role: Role::Follower,
                ..
            } => {
                println!("follower");
                // NOTE: do we need tokio select?
                tokio::select! {
                    msg = rx_timer.recv() => {
                        if msg.is_err() {
                          println!("error receiving msg: {:?}", msg);
                          return Err(anyhow!("error receiving msg"));
                        };

                        println!("sending vote request");
                        self.role = Role::Candidate{votes: 1, vote_set: HashSet::new()};

                        let msg = message::Data {
                            msg_type: Some(message::data::MsgType::Vote(message::VoteRequest {
                                term: self.current_term + 1,
                                candidate_id: self.id,
                                last_log_idx: self.log_index,
                                last_log_term: self.log_term,
                            }))

                        };

                        self.send_msg(&msg).await?;
                    },
                    msg = rx_msg.recv() => {
                        let msg = msg?;
                        match msg.msg_type {
                            Some(message::data::MsgType::Vote(vote_request)) => {
                                // TODO: send message back to the correct node
                                println!("vote request: {:?}", vote_request);
                                if vote_request.term > self.current_term {
                                    self.current_term = vote_request.term;
                                    self.voted_for = Some(vote_request.candidate_id);
                                    self.role = Role::Follower;

                                    let msg = message::Data {
                                        msg_type: Some(message::data::MsgType::Response(message::Response {
                                            term: self.current_term,
                                            status: true,
                                        }))
                                    };

                                    self.send_msg(&msg).await?;
                                }
                            },
                            Some(message::data::MsgType::Append(append_request)) => {
                                println!("append request: {:?}", append_request);

                                if append_request.term > self.current_term && append_request.prev_log_idx == self.log_index && append_request.prev_log_term == self.log_term {
                                    if append_request.entries.len() > 0 {
                                        self.log_index += 1;
                                        self.log_term = append_request.term;
                                    }

                                    let msg = message::Data {
                                        msg_type: Some(message::data::MsgType::Response(message::Response {
                                            term: self.current_term,
                                            status: true,
                                        }))
                                    };

                                    self.send_msg(&msg).await?;
                                } else {
                                    let msg = message::Data {
                                        msg_type: Some(message::data::MsgType::Response(message::Response {
                                            term: self.current_term,
                                            status: false,
                                        }))
                                    };

                                    self.send_msg(&msg).await?;
                                }
                            },
                            _ => {},
                        }
                    },
                };

                Ok(())
            }
            Node {
                role: Role::Candidate { votes, vote_set },
                ..
            } => {
                println!("candidate");
                tokio::select! {
                    msg = rx_timer.recv() => {
                        if msg.is_err() {
                          println!("error receiving msg: {:?}", msg);
                          return Err(anyhow!("error receiving msg, {:?}", msg));
                        };

                        println!("sending vote request");

                        let msg = message::Data {
                            msg_type: Some(message::data::MsgType::Vote(message::VoteRequest {
                                term: self.current_term + 1,
                                candidate_id: self.id,
                                last_log_idx: self.log_index,
                                last_log_term: self.log_term,
                            }))
                        };
                        self.send_msg(&msg).await?;
                    },

                    msg = rx_msg.recv() => {
                        let msg = msg?;
                        match msg.msg_type {
                            Some(message::data::MsgType::Vote(vote_request)) => {
                                println!("vote request: {:?}", vote_request);

                                if vote_request.term > self.current_term {
                                    self.current_term = vote_request.term;
                                    self.voted_for = Some(vote_request.candidate_id);
                                    self.role = Role::Follower;

                                    let msg = message::Data {
                                        msg_type: Some(message::data::MsgType::Response(message::Response {
                                            term: self.current_term,
                                            status: true,
                                        }))
                                    };

                                    self.send_msg(&msg).await?;
                                } else {
                                    let msg = match self.voted_for {
                                        None => {
                                            self.voted_for = Some(vote_request.candidate_id);
                                            self.role = Role::Follower;

                                            message::Data {
                                                msg_type: Some(message::data::MsgType::Response(message::Response {
                                                    term: self.current_term,
                                                    status: true,
                                                }))
                                            }
                                        },
                                        Some(voted_for) if voted_for == vote_request.candidate_id => {

                                            message::Data {
                                                msg_type: Some(message::data::MsgType::Response(message::Response {
                                                    term: self.current_term,
                                                    status: true,
                                                }))
                                            }
                                        },
                                        _ => {
                                            message::Data {
                                                msg_type: Some(message::data::MsgType::Response(message::Response {
                                                    term: self.current_term,
                                                    status: false,
                                                }))
                                            }
                                        },
                                    };

                                    self.send_msg(&msg).await?;
                                }
                            },
                            Some(message::data::MsgType::Response(response_request)) => {
                                println!("response request: {:?}", response_request);
                                match response_request.term.cmp(&self.current_term) {
                                    Ordering::Greater => {
                                        self.current_term = response_request.term;
                                        self.role = Role::Follower;
                                    },
                                    Ordering::Less | Ordering::Equal => {
                                       if response_request.status {
                                           if (votes + 1) > (self.peers.len() as u64 / 2) {
                                                self.role = Role::Leader;
                                            } else {
                                                let votes = votes + 1;
                                                println!("votes: {:?}", &votes);

                                                self.role = Role::Candidate {
                                                    votes: votes,
                                                    vote_set: vote_set.clone(),
                                                };
                                            }

                                       }
                                    },
                                }

                            },
                            _ => {},
                        }
                    },
                };

                Ok(())
            }
            Node {
                role: Role::Leader, ..
            } => {
                println!("leader");
                Ok(())
            }
        }
    }
}

impl Peer {
    pub fn new(id: u64, server: String) -> Self {
        Self { id, server }
    }

    async fn send_msg(&self, socket: &UdpSocket, msg: &message::Data) -> Result<()> {
        // println!("sending msg: {:?} to {}", &msg, &self.server);
        println!("sending msg to {:?}", &self.server);
        socket
            .send_to(&msg.encode_length_delimited_to_vec(), &self.server)
            .await
            .context("can't send")?;
        Ok(())
    }
}
