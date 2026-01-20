const EVENT_CHANNEL_SIZE: usize = 100;
const RESPONSE_CHANNEL_SIZE: usize = 10;
const UPDATE_SLEEP_DURATION: usize = 10;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt}, 
    net::{TcpListener, tcp::{OwnedReadHalf, OwnedWriteHalf}}, 
    sync::{RwLock, mpsc},
    time::sleep
};

use std::{
    net::SocketAddr, 
    sync::Arc,
    time::Duration,
};

#[allow(unused)]
use log::{info, warn, error};

use anyhow::Result;

use super::network::{PeerManager, ResponseHandler, EventCommand, NetworkMessage, NetworkOption, ResponseCommand};

pub async fn network_server<H, P, M>(
    handler: Arc<RwLock<H>>,
    peer_manager: Arc<RwLock<P>>,
    address: SocketAddr,
) -> Result<()>
where
    H: ResponseHandler<M>,
    P: PeerManager,
    M: NetworkMessage,
{

    let (event_tx, event_rx) = mpsc::channel::<EventCommand>(EVENT_CHANNEL_SIZE);

    tokio::spawn({
        let peer_manager = Arc::clone(&peer_manager);
        async move{
            if let Err(e) = response_handler(
                handler, 
                peer_manager,
                 event_rx
                ).await{
                    error!("Network Server Error: {}", e);
                }

        }
    });

    tokio::spawn({
        let peer_manager = Arc::clone(&peer_manager);
        async {
            if let Err(e) = update_peers(peer_manager).await{
                error!("Peer manager error");
            }
        }
    });

    let listener = TcpListener::bind(address).await?;

    loop{
        let (stream, peer) = listener.accept().await?;
        
        let (reader, writer) = stream.into_split();

        let (response_tx, response_rx) = mpsc::channel::<ResponseCommand>(RESPONSE_CHANNEL_SIZE);

        tokio::spawn({
            let event_tx = event_tx.clone();
            async move{
                if let Err(e) = connection_receiver(event_tx, reader, &peer).await{
                    error!("Connection receiver error: {}", e);
                }
            }
        });

        tokio::spawn(async move {
            if let Err(e) = connection_sender(response_rx, writer).await{
                error!("Connection sender error: {}", e);
            }
        });

        peer_manager.write().await.add_peer(&peer, response_tx)?;

    }
}

async fn response_handler<H, P, M>(
    handler: Arc<RwLock<H>>,
    peer_manager: Arc<RwLock<P>>,
    mut event_rx: mpsc::Receiver<EventCommand>
) -> Result<()>
where
    H: ResponseHandler<M>,
    P: PeerManager,
    M: NetworkMessage,
{

    while let Some(event_command) = event_rx.recv().await{
        match event_command{
            EventCommand::Message(peer, bytes) => {
                let message = M::from_bytes(bytes)?;

                let response_opt = {
                    handler.write().await.handle_message(message)
                };
                match response_opt{
                    NetworkOption::Send(response) => {
                        peer_manager.read().await.send(&peer, response.to_bytes()).await?;
                    }
                    NetworkOption::Broadcast(response) => {
                        peer_manager.read().await.broadcast(response.to_bytes()).await?;
                    }
                    NetworkOption::None => continue
                }
            }

            EventCommand::Close(peer) => {
                peer_manager.write().await.remove_peer(&peer).await?;
            }
        }
    }

    Ok(())
}

async fn connection_sender(
    mut response_rx: mpsc::Receiver<ResponseCommand>, 
    mut writer: OwnedWriteHalf
) -> Result<()>{
    while let Some(response) = response_rx.recv().await{
        match response{
            ResponseCommand::Message(message) => {
                let len = (message.len() as u32).to_be_bytes();
                writer.write_all(&len).await?;
                writer.write_all(&message).await?;
            }

            ResponseCommand::Close => break
        }
    }

    writer.shutdown().await?;
    return Ok(())
}

async fn connection_receiver(
    event_tx: mpsc::Sender<EventCommand>,
    mut reader: OwnedReadHalf,
    peer: &SocketAddr,
) -> Result<()>{
    loop{
        let mut len_bytes = [0u8; 4];

        match reader.read_exact(&mut len_bytes).await{
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                event_tx.send(EventCommand::Close(*peer)).await?;
                return Ok(())
            }
            Err(e) => {
                error!("Error reading from: {} : {}", peer, e);
                event_tx.send(EventCommand::Close(*peer)).await?;
                return Err(e.into());
            }
        }

        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).await?;
        event_tx.send(EventCommand::Message(*peer, buf)).await?;
    }
}

async fn update_peers<P>(peer_manager: Arc<RwLock<P>>) -> Result<()>
where 
    P: PeerManager,
{
    loop{
        sleep(Duration::from_secs(UPDATE_SLEEP_DURATION as u64)).await;
        {
            peer_manager.write().await.update().await?;
        }      
    }
}