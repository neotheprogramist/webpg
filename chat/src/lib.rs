use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    ReadableStreamDefaultReader, ReadableStreamReadResult, WebTransport,
    WebTransportBidirectionalStream, WritableStreamDefaultWriter,
};

#[wasm_bindgen]
pub struct ChatClient {
    transport: WebTransport,
    on_message: js_sys::Function,
}

#[wasm_bindgen]
impl ChatClient {
    #[wasm_bindgen(constructor)]
    pub fn new(transport: WebTransport, on_message: js_sys::Function) -> Self {
        let client = Self {
            transport,
            on_message,
        };
        client.spawn_listener();
        client
    }

    fn spawn_listener(&self) {
        let streams_reader: ReadableStreamDefaultReader = self
            .transport
            .incoming_unidirectional_streams()
            .get_reader()
            .unchecked_into();
        let on_message = self.on_message.clone();

        wasm_bindgen_futures::spawn_local(async move {
            loop {
                let result: ReadableStreamReadResult = JsFuture::from(streams_reader.read())
                    .await
                    .unwrap()
                    .unchecked_into();

                if result.get_done().unwrap_or(false) {
                    break;
                }

                let stream_reader: ReadableStreamDefaultReader = result
                    .get_value()
                    .unchecked_into::<web_sys::ReadableStream>()
                    .get_reader()
                    .unchecked_into();

                let text = read_stream_to_string(&stream_reader).await;
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();

                let js_obj = js_sys::Object::new();
                js_sys::Reflect::set(
                    &js_obj,
                    &"id".into(),
                    &parsed["id"].as_i64().unwrap().into(),
                )
                .unwrap();
                js_sys::Reflect::set(
                    &js_obj,
                    &"body".into(),
                    &parsed["body"].as_str().unwrap().into(),
                )
                .unwrap();
                js_sys::Reflect::set(
                    &js_obj,
                    &"created_at".into(),
                    &parsed["created_at"].as_str().unwrap().into(),
                )
                .unwrap();

                on_message.call1(&JsValue::NULL, &js_obj).unwrap();
            }
        });
    }

    pub async fn send(&self, body: &str) {
        let bidi: WebTransportBidirectionalStream =
            JsFuture::from(self.transport.create_bidirectional_stream())
                .await
                .unwrap()
                .unchecked_into();

        let writer: WritableStreamDefaultWriter = bidi.writable().get_writer().unwrap();

        let payload = serde_json::to_string(&serde_json::json!({ "body": body })).unwrap();
        let bytes = js_sys::Uint8Array::from(payload.as_bytes());

        JsFuture::from(writer.write_with_chunk(&bytes))
            .await
            .unwrap();
        JsFuture::from(writer.close()).await.unwrap();
    }
}

async fn read_stream_to_string(reader: &ReadableStreamDefaultReader) -> String {
    let mut chunks: Vec<u8> = Vec::new();

    loop {
        let result: ReadableStreamReadResult = JsFuture::from(reader.read())
            .await
            .unwrap()
            .unchecked_into();

        if result.get_done().unwrap_or(false) {
            break;
        }

        let chunk = js_sys::Uint8Array::new(&result.get_value());
        let mut bytes = vec![0u8; chunk.length() as usize];
        chunk.copy_to(&mut bytes);
        chunks.extend_from_slice(&bytes);
    }

    String::from_utf8(chunks).unwrap()
}
