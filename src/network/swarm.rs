use crate::network::client::{FileRequest, FileResponse};
use anyhow::Result;
use async_trait::async_trait;
use libp2p::{
    core::upgrade::{read_length_prefixed, write_length_prefixed, ProtocolName},
    futures::{AsyncRead, AsyncWrite, AsyncWriteExt},
    identity::Keypair,
    kad::{record::store::MemoryStore, Kademlia, KademliaEvent},
    request_response::{
        ProtocolSupport, RequestResponse, RequestResponseCodec, RequestResponseEvent,
    },
    swarm::{NetworkBehaviour, Swarm},
};
use std::{io, iter};

pub async fn build_swarm(keypair: Keypair) -> Result<Swarm<ComposedBehaviour>> {
    let peer_id = keypair.public().to_peer_id();

    Ok(Swarm::with_threadpool_executor(
        libp2p::development_transport(keypair).await?,
        ComposedBehaviour {
            kademlia: Kademlia::new(peer_id, MemoryStore::new(peer_id)),
            request_response: RequestResponse::new(
                FileExchangeCodec(),
                iter::once((FileExchangeProtocol(), ProtocolSupport::Full)),
                Default::default(),
            ),
        },
        peer_id,
    ))
}

#[derive(Debug)]
pub enum ComposedEvent {
    RequestResponse(RequestResponseEvent<FileRequest, FileResponse>),
    Kademlia(KademliaEvent),
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "ComposedEvent")]
pub struct ComposedBehaviour {
    pub request_response: RequestResponse<FileExchangeCodec>,
    pub kademlia: Kademlia<MemoryStore>,
}

impl From<RequestResponseEvent<FileRequest, FileResponse>> for ComposedEvent {
    fn from(event: RequestResponseEvent<FileRequest, FileResponse>) -> Self {
        ComposedEvent::RequestResponse(event)
    }
}

impl From<KademliaEvent> for ComposedEvent {
    fn from(event: KademliaEvent) -> Self {
        ComposedEvent::Kademlia(event)
    }
}

#[derive(Debug, Clone)]
pub struct FileExchangeProtocol();

#[derive(Clone)]
pub struct FileExchangeCodec();

impl ProtocolName for FileExchangeProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/file-exchange/1".as_bytes()
    }
}

#[async_trait]
impl RequestResponseCodec for FileExchangeCodec {
    type Protocol = FileExchangeProtocol;
    type Request = FileRequest;
    type Response = FileResponse;

    async fn read_request<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(FileRequest(String::from_utf8(vec).unwrap()))
    }

    async fn read_response<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 500_000_000).await?; // update transfer maximum

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(FileResponse(vec))
    }

    async fn write_request<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
        FileRequest(data): FileRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &FileExchangeProtocol,
        io: &mut T,
        FileResponse(data): FileResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }
}
