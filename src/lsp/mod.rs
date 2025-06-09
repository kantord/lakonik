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
    DidChangeConfigurationParams, Hover, HoverContents, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, MarkedString, Position, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
    },
};
use tower::ServiceBuilder;
use tracing::Level;
use utils::update_document;

use crate::hir::{Analyzed, AnalyzedPart, AnalyzedSentence};

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
            let hover = analyzed_opt.and_then(|analyzed| {
                find_hover_text(&analyzed, &pos).map(|txt| Hover {
                    contents: HoverContents::Scalar(MarkedString::String(txt.to_string())),
                    range: None,
                })
            });

            Ok(hover)
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
    if analyzed.vocative.get_range().contains_position(pos) {
        return Some(&analyzed.vocative.hover_text);
    }

    if analyzed.verb.get_range().contains_position(pos) {
        return Some(&analyzed.verb.hover_text);
    }

    for part in &analyzed.parts {
        match part {
            AnalyzedPart::Freeform(p) => {
                if p.node.range.contains_position(pos) {
                    return Some(&p.hover_text);
                }
            }
            AnalyzedPart::FilePath(p) => {
                if p.node.range.contains_position(pos) {
                    return Some(&p.hover_text);
                }
            }
            AnalyzedPart::InlineShell(p) => {
                if p.node.range.contains_position(pos) {
                    return Some(&p.hover_text);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    use async_lsp::{
        LanguageServer, MainLoop, ServerSocket, client_monitor::ClientProcessMonitorLayer,
        concurrency::ConcurrencyLayer, panic::CatchUnwindLayer, router::Router,
        server::LifecycleLayer, tracing::TracingLayer,
    };
    use lsp_types::{
        DidOpenTextDocumentParams, HoverContents, HoverParams, InitializeParams, InitializedParams,
        MarkedString, Position, TextDocumentIdentifier, TextDocumentItem,
        TextDocumentPositionParams, Url, WorkDoneProgressParams,
    };
    use regex::Regex;
    use rstest::rstest;
    use tokio::io::duplex;
    use tokio::task::JoinHandle;
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
    use tower::ServiceBuilder;

    pub fn find_hover_position(source: &str) -> (String, Position) {
        if let Some(idx) = source.find("***") {
            let mut clean = String::with_capacity(source.len() - 3);
            clean.push_str(&source[..idx]);
            clean.push_str(&source[idx + 3..]);
            let col = idx as u32;
            (clean, Position::new(0, col))
        } else {
            panic!("No hover marker (***) found in source!");
        }
    }

    pub async fn launch_lsp_server() -> (ServerSocket, JoinHandle<()>, JoinHandle<()>) {
        // Create two in-memory pipes: one for client→server, one for server→client.
        let (client_read, server_write) = duplex(1024);
        let (server_read, client_write) = duplex(1024);

        let (server_mainloop, _) = MainLoop::new_server(|client_socket| {
            ServiceBuilder::new()
                .layer(TracingLayer::default())
                .layer(LifecycleLayer::default())
                .layer(CatchUnwindLayer::default())
                .layer(ConcurrencyLayer::default())
                .layer(ClientProcessMonitorLayer::new(client_socket.clone()))
                .service(ServerState::new_router(client_socket))
        });

        let server_task: JoinHandle<()> = tokio::spawn(async move {
            let mut read = server_read.compat();
            let mut write = server_write.compat_write();
            server_mainloop
                .run_buffered(&mut read, &mut write)
                .await
                .unwrap();
        });

        let (client_mainloop, client_socket) = MainLoop::new_client(|_server| Router::new(()));

        let client_task: JoinHandle<()> = tokio::spawn(async move {
            let mut read = client_read.compat();
            let mut write = client_write.compat_write();
            client_mainloop
                .run_buffered(&mut read, &mut write)
                .await
                .unwrap();
        });

        (client_socket, server_task, client_task)
    }

    pub async fn get_hover_text(source: &str) -> Option<String> {
        let (clean, pos) = find_hover_position(source);

        let (mut client, server_handle, client_handle) = launch_lsp_server().await;

        let uri = Url::parse("file:///testfile").unwrap();

        client
            .initialize(InitializeParams {
                ..Default::default()
            })
            .await
            .unwrap();
        client.initialized(InitializedParams {}).unwrap();

        client
            .did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "test".into(),
                    version: 1,
                    text: clean.clone(),
                },
            })
            .unwrap();

        let hover = client
            .hover(HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    position: pos,
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
            })
            .await
            .unwrap();

        drop(client);
        server_handle.abort();
        client_handle.abort();

        hover.map(|h| match h.contents {
            HoverContents::Scalar(MarkedString::String(s)) => s,
            HoverContents::Markup(m) => m.value,
            HoverContents::Array(arr) => arr
                .into_iter()
                .map(|ms| match ms {
                    MarkedString::String(s) => s,
                    MarkedString::LanguageString(ls) => ls.value,
                })
                .collect::<Vec<_>>()
                .join("\n"),
            _ => String::new(),
        })
    }

    #[rstest]
    #[case("qw***en3 create foobar", Some(r"vocative: qwen3$"))]
    #[case("hell***o create foobar", Some(r"vocative: hello$"))]
    #[case("foobar *** create lorem", None)]
    #[case(
        "test c***reate foobar",
        Some(r"(?s)_Verb_.*create.*create for me a.*")
    )]
    #[case(
        "test ~f***oo=(lorem ipsum) foobar",
        Some(r"(?s)_Verb_.*foo.*lorem ipsum.*")
    )]
    #[case("test ***create foobar", Some(r"_Verb_.*create.*"))]
    #[case("test create foo***bar", Some(r"^This is a part"))]
    #[case(
        "test create test module in $(tre***e .)",
        Some(r"expand to the results of `tree .`")
    )]
    #[tokio::test]
    async fn hover_cases(#[case] raw_input: &str, #[case] expected_pat: Option<&str>) {
        let actual = get_hover_text(raw_input).await;

        match expected_pat {
            // We expect *no* hover text.
            None => assert!(
                actual.is_none(),
                "expected no hover text, but got `{actual:?}`"
            ),

            // We expect hover text that matches the regex pattern.
            Some(pat) => {
                let text = actual.expect("expected some hover text, got `None`");
                let re = Regex::new(pat).expect("invalid regex");
                assert!(
                    re.is_match(&text),
                    "regex `{pat}` did not match hover text `{text}`"
                );
            }
        }
    }
}
