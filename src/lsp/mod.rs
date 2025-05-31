use std::ops::ControlFlow;

use async_lsp::client_monitor::ClientProcessMonitorLayer;
use async_lsp::concurrency::ConcurrencyLayer;
use async_lsp::panic::CatchUnwindLayer;
use async_lsp::router::Router;
use async_lsp::server::LifecycleLayer;
use async_lsp::tracing::TracingLayer;
use async_lsp::{ClientSocket, LanguageServer, ResponseError};
use futures::future::BoxFuture;
use lsp_types::{
    DidChangeConfigurationParams, DidSaveTextDocumentParams, Hover, HoverContents, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult, MarkedString, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
    },
};
use tower::ServiceBuilder;
use tracing::Level;

struct ServerState {
    _client: ClientSocket,
    counter: i32,
}

impl LanguageServer for ServerState {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<async_lsp::Result<()>>;

    fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> BoxFuture<'static, Result<InitializeResult, Self::Error>> {
        eprintln!("Initialize with {params:?}");
        Box::pin(async move {
            Ok(InitializeResult {
                capabilities: ServerCapabilities {
                    text_document_sync: Some(TextDocumentSyncCapability::Kind(
                        TextDocumentSyncKind::INCREMENTAL,
                    )),

                    hover_provider: Some(HoverProviderCapability::Simple(true)),
                    ..ServerCapabilities::default()
                },
                server_info: None,
            })
        })
    }

    fn hover(&mut self, _: HoverParams) -> BoxFuture<'static, Result<Option<Hover>, Self::Error>> {
        let counter = self.counter;
        Box::pin(async move {
            Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "I am a dummy hover {counter}!"
                ))),
                range: None,
            }))
        })
    }

    fn did_change_configuration(
        &mut self,
        _: DidChangeConfigurationParams,
    ) -> ControlFlow<async_lsp::Result<()>> {
        ControlFlow::Continue(())
    }
}

impl ServerState {
    fn new_router(client: ClientSocket) -> Router<Self> {
        let mut router = Router::from_language_server(Self {
            _client: client,
            counter: 0,
        });

        router.notification::<DidOpenTextDocument>(Self::on_did_open);
        router.notification::<DidChangeTextDocument>(Self::on_did_change);
        router.notification::<DidSaveTextDocument>(Self::on_did_save);
        router.notification::<DidCloseTextDocument>(Self::on_did_close);

        router
    }

    fn on_did_open(
        &mut self,
        _params: lsp_types::DidOpenTextDocumentParams,
    ) -> ControlFlow<async_lsp::Result<()>> {
        ControlFlow::Continue(())
    }

    fn on_did_change(
        &mut self,
        _params: lsp_types::DidChangeTextDocumentParams,
    ) -> ControlFlow<async_lsp::Result<()>> {
        ControlFlow::Continue(())
    }

    fn on_did_save(
        &mut self,
        _params: DidSaveTextDocumentParams,
    ) -> ControlFlow<async_lsp::Result<()>> {
        ControlFlow::Continue(())
    }

    fn on_did_close(
        &mut self,
        _params: lsp_types::DidCloseTextDocumentParams,
    ) -> ControlFlow<async_lsp::Result<()>> {
        ControlFlow::Continue(())
    }
}

pub async fn run_lsp_server() {
    let (server, _) = async_lsp::MainLoop::new_server(|client| {
        ServiceBuilder::new()
            .layer(TracingLayer::default())
            .layer(LifecycleLayer::default())
            .layer(CatchUnwindLayer::default())
            .layer(ConcurrencyLayer::default())
            .layer(ClientProcessMonitorLayer::new(client.clone()))
            .service(ServerState::new_router(client))
    });

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();

    let (stdin, stdout) = (
        tokio_util::compat::TokioAsyncReadCompatExt::compat(tokio::io::stdin()),
        tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(tokio::io::stdout()),
    );

    server.run_buffered(stdin, stdout).await.unwrap();
}
