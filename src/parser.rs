use clap::Parser;
use itertools::Itertools;

use crate::node::Peer;

/// simple raft implementation for learning
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// node id
    #[arg(long)]
    pub id: i32,

    /// server address
    #[arg(short, long)]
    pub server: String,

    /// peers address
    #[arg(short, long,  num_args = 1.., value_delimiter = ' ',  value_parser = validate_peers)]
    pub peers: Vec<Peer>,
}

fn validate_peers(peer: &str) -> Result<Peer, String> {
    if peer.is_empty() {
        return Err(format!("peer is empty"));
    }

    let Some((id, server)) = peer
        .split('@')
        .map(|s| s.to_string())
        .collect_tuple::<(String, String)>() else {
        return Err(format!("peer {} is invalid", peer));
        
        };

    let Ok(id) = id.parse::<i32>() else  {
        return Err(format!("peer {} id is invalid", peer));
    };

    Ok(Peer {id, server})
}
