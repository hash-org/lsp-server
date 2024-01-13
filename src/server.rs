use std::task::{Context, Poll};

use futures::future::{BoxFuture, Inspect};
use futures::FutureExt;
use tower::Layer;
use tower_lsp::jsonrpc::{Request, Response};
use tower_lsp::{lsp_types::*, ExitedError, LspService};
use tower_lsp::{Client, LanguageServer};
use tower_service::Service;

pub struct LogLayer {}

impl<S> Layer<S> for LogLayer {
    type Service = ServiceWithLogLayer<S>;

    fn layer(&self, service: S) -> Self::Service {
        ServiceWithLogLayer { service }
    }
}

// This service implements the Log behavior
pub struct ServiceWithLogLayer<S> {
    service: S,
}

impl Service<Request> for ServiceWithLogLayer<LspService<HashLanguageServer>> {
    type Response = Option<Response>;
    type Error = ExitedError;
    type Future = Inspect<
        BoxFuture<'static, Result<Self::Response, Self::Error>>,
        fn(&Result<Option<Response>, ExitedError>),
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        log::info!("request = {:?}", request);

        // Insert log statement here or other functionality
        self.service.call(request).inspect(|resp| {
            log::info!("response = {:?}\n\r", resp);
        })
    }
}

#[derive(Debug, Clone)]
pub struct HashLanguageServer {
    pub client: Client,
}
type LspResult<T> = tower_lsp::jsonrpc::Result<T>;
#[tower_lsp::async_trait]
impl LanguageServer for HashLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        log::info!("Client triggered Initialize");
        let initialize_result = InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                diagnostic_provider: None,
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "hash-lsp".to_string(),
                version: Some("1.0".to_string()),
            }),
            ..Default::default()
        };
        self.client
            .log_message(
                MessageType::INFO,
                format!("config: {:#?}", initialize_result),
            )
            .await;
        Ok(initialize_result)
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
        log::info!("Server initialized");
    }

    async fn shutdown(&self) -> LspResult<()> {
        log::info!("Server shutdown");
        Ok(())
    }

    async fn completion_resolve(&self, params: CompletionItem) -> LspResult<CompletionItem> {
        Ok(params)
    }

    async fn completion(&self, _: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        self.client
            .log_message(MessageType::INFO, "completion triggered!")
            .await;
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem {
                kind: Some(CompletionItemKind::TEXT),
                label: "Hello".to_string(),
                detail: Some("Some detail".to_string()),
                ..Default::default()
            },
            CompletionItem {
                kind: Some(CompletionItemKind::TEXT),
                label: "Bye".to_string(),
                detail: Some("Some detail".to_string()),
                ..Default::default()
            },
        ])))
    }

    async fn hover(&self, _: HoverParams) -> LspResult<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
            range: None,
        }))
    }
}
