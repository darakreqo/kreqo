use std::future::Future;

use bytes::Bytes;
use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt};
use server_fn::client::{Client, get_server_url};
use server_fn::error::{FromServerFnError, IntoAppError, ServerFnErrorErr};

use crate::custom_client::request::{CLIENT, CustomRequest};
use crate::custom_client::response::CustomResponse;

/// Implements [`Client`] for a request made by [`reqwest`].
pub struct CustomClient;

impl<
    Error: FromServerFnError,
    InputStreamError: FromServerFnError,
    OutputStreamError: FromServerFnError,
> Client<Error, InputStreamError, OutputStreamError> for CustomClient
{
    type Request = CustomRequest;
    type Response = CustomResponse;

    fn send(req: Self::Request) -> impl Future<Output = Result<Self::Response, Error>> + Send {
        CLIENT
            .execute(req.0)
            .map(|x| x.map(|res| res.into()))
            .map_err(|e| ServerFnErrorErr::Request(e.to_string()).into_app_error())
    }

    async fn open_websocket(
        path: &str,
    ) -> Result<
        (
            impl futures::Stream<Item = Result<Bytes, Bytes>> + Send + 'static,
            impl futures::Sink<Bytes> + Send + 'static,
        ),
        Error,
    > {
        let mut websocket_server_url = get_server_url().to_string();
        if let Some(postfix) = websocket_server_url.strip_prefix("http://") {
            websocket_server_url = format!("ws://{postfix}");
        } else if let Some(postfix) = websocket_server_url.strip_prefix("https://") {
            websocket_server_url = format!("wss://{postfix}");
        }
        let url = format!("{websocket_server_url}{path}");
        let (ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .map_err(|e| Error::from_server_fn_error(ServerFnErrorErr::Request(e.to_string())))?;

        let (write, read) = ws_stream.split();

        Ok(
            (
                read.map(|msg| match msg {
                    Ok(msg) => Ok(msg.into_data()),
                    Err(e) => Err(OutputStreamError::from_server_fn_error(
                        ServerFnErrorErr::Request(e.to_string()),
                    )
                    .ser()),
                }),
                write.with(|msg: Bytes| async move {
                    Ok::<
                        tokio_tungstenite::tungstenite::Message,
                        tokio_tungstenite::tungstenite::Error,
                    >(tokio_tungstenite::tungstenite::Message::Binary(msg))
                }),
            ),
        )
    }

    fn spawn(future: impl Future<Output = ()> + Send + 'static) {
        tokio::spawn(future);
    }
}
