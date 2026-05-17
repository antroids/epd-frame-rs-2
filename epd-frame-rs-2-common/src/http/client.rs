use crate::types::LimitedString;
use alloc::{format, vec};
use reqwless::client::{TlsConfig, TlsVerify};
use reqwless::headers::ContentType;
use reqwless::request::{Method, RequestBuilder};

const TCP_POOL_SIZE: usize = 1;
const TCP_TX_BUFFER_SIZE: usize = 1024 * 8;
const TCP_RX_BUFFER_SIZE: usize = 1024 * 8;
const TLS_TX_BUFFER_SIZE: usize = 1024 * 32;
const TLS_RX_BUFFER_SIZE: usize = 1024 * 32;

type TcpClient<'a> =
    embassy_net::tcp::client::TcpClient<'a, TCP_POOL_SIZE, TCP_TX_BUFFER_SIZE, TCP_RX_BUFFER_SIZE>;
type InnerHttpClient<'a> =
    reqwless::client::HttpClient<'a, TcpClient<'a>, embassy_net::dns::DnsSocket<'a>>;

pub struct HttpClient {
    tcp_client_state: embassy_net::tcp::client::TcpClientState<
        TCP_POOL_SIZE,
        TCP_TX_BUFFER_SIZE,
        TCP_RX_BUFFER_SIZE,
    >,
    dns_client: embassy_net::dns::DnsSocket<'static>,
    stack: embassy_net::Stack<'static>,
    rand: fastrand::Rng,
}

impl HttpClient {
    pub fn new(stack: embassy_net::Stack<'static>, seed: u64) -> Self {
        let tcp_client_state = embassy_net::tcp::client::TcpClientState::new();
        let dns_client = embassy_net::dns::DnsSocket::new(stack.clone());
        Self {
            tcp_client_state,
            dns_client,
            stack,
            rand: fastrand::Rng::with_seed(seed),
        }
    }

    pub async fn get<RESP>(
        &mut self,
        url: &str,
        headers: &[(&str, &str)],
        response_buffer: &mut [u8],
    ) -> Result<RESP, super::HttpError>
    where
        RESP: serde::de::DeserializeOwned,
    {
        let response_buffer_len = response_buffer.len();
        let tcp_client = TcpClient::new(self.stack, &mut self.tcp_client_state);
        let mut tls_rx_buf = vec![0u8; TLS_RX_BUFFER_SIZE];
        let mut tls_tx_buf = vec![0u8; TLS_TX_BUFFER_SIZE];
        let tls_config = TlsConfig::new(
            self.rand.get_seed(),
            &mut tls_rx_buf,
            &mut tls_tx_buf,
            TlsVerify::None,
        );
        let mut http_client =
            InnerHttpClient::new_with_tls(&tcp_client, &self.dns_client, tls_config);
        let mut request_handle = http_client
            .request(Method::GET, url)
            .await?
            .headers(headers);
        let mut response = request_handle.send(response_buffer).await?;

        if response.status.is_successful() {
            if let Some(content_type) = response.content_type.take() {
                match content_type {
                    ContentType::ApplicationJson | ContentType::ApplicationOctetStream => {
                        let mut body = vec![0u8; response_buffer_len];
                        let read = response.body().reader().read_to_end(&mut body).await?;
                        let deserialized = serde_json::from_slice(&body[..read]).map_err(|e| {
                            super::HttpError::DeserializationError(
                                LimitedString::from_str_truncate(&format!(
                                    "Unsupported content type: {:?}",
                                    e
                                )),
                            )
                        })?;
                        Ok(deserialized)
                    }
                    ct => Err(super::HttpError::HttpClientError(
                        LimitedString::from_str_truncate(&format!(
                            "Unsupported content type: {}",
                            ct.as_str()
                        )),
                    )),
                }
            } else {
                Err(super::HttpError::HttpClientError(
                    LimitedString::from_str_truncate("Unsupported content type"),
                ))
            }
        } else {
            Err(super::HttpError::HttpStatusError(response.status.0))
        }
    }
}

impl From<reqwless::Error> for super::HttpError {
    fn from(value: reqwless::Error) -> Self {
        let msg = format!("{:?}", value);
        Self::HttpClientError(LimitedString::from_str_truncate(&msg))
    }
}
