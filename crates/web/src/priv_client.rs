use pi_kiosk_privileged::proto::{PrivRequest, PrivResponse};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/run/pi-kiosk/priv.sock";

pub async fn send_request(request: &PrivRequest) -> Result<PrivResponse, String> {
    let mut stream = UnixStream::connect(SOCKET_PATH)
        .await
        .map_err(|e| format!("failed to connect to privileged helper: {e}"))?;

    let data = serde_json::to_vec(request)
        .map_err(|e| format!("failed to serialize request: {e}"))?;

    let len = data.len() as u32;
    stream.write_u32(len).await.map_err(|e| format!("write len: {e}"))?;
    stream.write_all(&data).await.map_err(|e| format!("write data: {e}"))?;

    let resp_len = stream.read_u32().await.map_err(|e| format!("read len: {e}"))?;
    let mut resp_data = vec![0u8; resp_len as usize];
    stream.read_exact(&mut resp_data).await.map_err(|e| format!("read data: {e}"))?;

    let response: PrivResponse = serde_json::from_slice(&resp_data)
        .map_err(|e| format!("failed to deserialize response: {e}"))?;

    Ok(response)
}

pub async fn send_request_ok(request: &PrivRequest) -> Result<(), String> {
    match send_request(request).await {
        Ok(PrivResponse::Ok) => Ok(()),
        Ok(PrivResponse::Pong) => Ok(()),
        Ok(PrivResponse::Error(e)) => Err(e),
        Err(e) => Err(e),
    }
}
