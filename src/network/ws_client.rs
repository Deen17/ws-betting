use tokio_tungstenite::{connect_async, WebSocketStream, stream::Stream};
// use tokio_native_tls::*;
use tokio::net::TcpStream;
use log::*;
use http::Request;
use super::config::AUTHTOKEN;
// use super::message::Message;

pub type WSStream = WebSocketStream<Stream<TcpStream,tokio_native_tls::TlsStream<tokio::net::TcpStream>>>;

pub async fn connect() 
-> Result
    < WSStream, 
    Box<dyn std::error::Error + Send + Sync> > 
{
    info!("Connecting to DGG");
    let req = Request::builder()
        .uri("wss://chat.destiny.gg/ws")
        .header("Cookie",format!("authtoken={}", AUTHTOKEN))
        .body(())
        .unwrap();
    let (socket, res) = connect_async(req).await?;
    info!("res:\n{:?}", res);
    
    Ok(socket)
}
