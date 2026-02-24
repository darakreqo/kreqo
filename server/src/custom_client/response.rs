use bytes::Bytes;
use futures::{Stream, TryStreamExt};
use reqwest::Response;
use server_fn::error::{FromServerFnError, IntoAppError, ServerFnErrorErr};
use server_fn::response::ClientRes;

pub struct CustomResponse(pub Response);

impl From<Response> for CustomResponse {
    fn from(value: Response) -> Self {
        Self(value)
    }
}

impl<E: FromServerFnError> ClientRes<E> for CustomResponse {
    async fn try_into_string(self) -> Result<String, E> {
        self.0
            .text()
            .await
            .map_err(|e| ServerFnErrorErr::Deserialization(e.to_string()).into_app_error())
    }

    async fn try_into_bytes(self) -> Result<Bytes, E> {
        self.0
            .bytes()
            .await
            .map_err(|e| ServerFnErrorErr::Deserialization(e.to_string()).into_app_error())
    }

    fn try_into_stream(
        self,
    ) -> Result<impl Stream<Item = Result<Bytes, Bytes>> + Send + 'static, E> {
        Ok(self
            .0
            .bytes_stream()
            .map_err(|e| E::from_server_fn_error(ServerFnErrorErr::Response(e.to_string())).ser()))
    }

    fn status(&self) -> u16 {
        self.0.status().as_u16()
    }

    fn status_text(&self) -> String {
        self.0.status().to_string()
    }

    fn location(&self) -> String {
        self.0
            .headers()
            .get("Location")
            .map(|value| String::from_utf8_lossy(value.as_bytes()).to_string())
            .unwrap_or_else(|| self.0.url().to_string())
    }

    fn has_redirect(&self) -> bool {
        self.0.headers().get("Location").is_some()
    }
}
