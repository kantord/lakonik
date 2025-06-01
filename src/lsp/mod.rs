mod utils;
use std::collections::HashMap;
use std::ops::ControlFlow;

use crate::ast::utils::RangeContainsPosition;
use async_lsp::client_monitor::ClientProcessMonitorLayer;
use async_lsp::concurrency::ConcurrencyLayer;
use async_lsp::panic::CatchUnwindLayer;
use async_lsp::router::Router;
use async_lsp::server::LifecycleLayer;
use async_lsp::tracing::TracingLayer;
use async_lsp::{ClientSocket, LanguageServer, ResponseError};
use futures::future::BoxFuture;
use lsp_types::{
    DidChangeConfigurationParams, Hover, HoverContents, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult, MarkedString, Position,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
    },
};
use tower::ServiceBuilder;
use tracing::Level;
use utils::update_document;

use crate::hir::AnalyzedSentence;

pub struct DocumentState {
    analyzed: AnalyzedSentence,
}

pub struct ServerState {
    _client: ClientSocket,
    docs: HashMap<Url, DocumentState>,
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
                        TextDocumentSyncKind::FULL,
                    )),
                    hover_provider: Some(HoverProviderCapability::Simple(true)),
                    ..ServerCapabilities::default()
                },
                server_info: None,
            })
        })
    }

    fn hover(
        &mut self,
        params: HoverParams,
    ) -> BoxFuture<'static, Result<Option<Hover>, Self::Error>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();
        let pos = params.text_document_position_params.position;
        let analyzed_opt = self.docs.get(&uri).map(|doc| doc.analyzed.clone());

        Box::pin(async move {
            if let Some(analyzed) = analyzed_opt {
                if let Some(hover_text) = find_hover_text(&analyzed, &pos) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Scalar(MarkedString::String(
                            hover_text.to_string(),
                        )),
                        range: None,
                    }));
                }
            }
            Ok(None)
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
            docs: HashMap::new(),
        });

        router.notification::<DidOpenTextDocument>(Self::on_did_open);
        router.notification::<DidChangeTextDocument>(Self::on_did_change);
        router.notification::<DidCloseTextDocument>(Self::on_did_close);
        router.notification::<DidSaveTextDocument>(Self::on_did_save);

        router
    }

    fn on_did_open(
        &mut self,
        params: lsp_types::DidOpenTextDocumentParams,
    ) -> ControlFlow<async_lsp::Result<()>> {
        let text = params.text_document.text.clone();
        let uri = params.text_document.uri.clone();
        update_document(&mut self.docs, uri, text);
        ControlFlow::Continue(())
    }

    fn on_did_change(
        &mut self,
        params: lsp_types::DidChangeTextDocumentParams,
    ) -> ControlFlow<async_lsp::Result<()>> {
        if let Some(change) = params.content_changes.into_iter().next_back() {
            let text = change.text;
            let uri = params.text_document.uri.clone();
            eprintln!(
                "LSP: didChange {} ({} chars) text: '{}'",
                uri,
                text.len(),
                text
            );
            update_document(&mut self.docs, uri, text);
        }
        ControlFlow::Continue(())
    }

    fn on_did_close(
        &mut self,
        params: lsp_types::DidCloseTextDocumentParams,
    ) -> ControlFlow<async_lsp::Result<()>> {
        self.docs.remove(&params.text_document.uri);
        ControlFlow::Continue(())
    }

    fn on_did_save(
        &mut self,
        _params: lsp_types::DidSaveTextDocumentParams,
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

fn find_hover_text<'a>(analyzed: &'a AnalyzedSentence, pos: &Position) -> Option<&'a str> {
    if analyzed.vocative.node.range.contains_position(pos) {
        return Some(&analyzed.vocative.hover_text);
    }

    if analyzed.verb.node.range.contains_position(pos) {
        return Some(&analyzed.verb.hover_text);
    }

    for part in &analyzed.parts {
        match &part.node {
            crate::ast::Part::Freeform(f) => {
                if f.range.contains_position(pos) {
                    return Some(&part.hover_text);
                }
            }
            crate::ast::Part::FilePath(f) => {
                if f.range.contains_position(pos) {
                    return Some(&part.hover_text);
                }
            }
            crate::ast::Part::InlineShell(f) => {
                if f.range.contains_position(pos) {
                    return Some(&part.hover_text);
                }
            }
        }
    }
    None
}
