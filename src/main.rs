use simple_logger::SimpleLogger;
use tower_lsp::{LspService, Server};

use crate::server::{HashLanguageServer, LogLayer};

mod server;

use tower::Layer;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| HashLanguageServer { client });

    let layer_withlog = LogLayer {};
    let svc = layer_withlog.layer(service);

    log::info!("Starting server");

    Server::new(stdin, stdout, socket).serve(svc).await;

    log::info!("server stopped");
}
