#![no_main]
use libfuzzer_sys::fuzz_target;

use deno_doc::DocNodeKind;
use deno_doc::DocParser;
use deno_doc::DocPrinter;
use deno_graph::create_type_graph;
use deno_graph::source::LoadFuture;
use deno_graph::source::LoadResponse;
use deno_graph::source::Loader;
use deno_graph::CapturingModuleAnalyzer;
use deno_graph::DefaultParsedSourceStore;
use deno_graph::ModuleSpecifier;
use futures::executor::block_on;
use futures::future;

fuzz_target!(|source: &str| {
  let mut loader = SourceFileLoader { source };

  let future = async move {
    let source_parser = DefaultParsedSourceStore::default();
    let analyzer = CapturingModuleAnalyzer::default();
    let graph = create_type_graph(
      vec!["file://fuzz.ts".try_into().unwrap()],
      false,
      None,
      &mut loader,
      None,
      None,
      Some(&analyzer),
      None,
    )
    .await;
    let parser = DocParser::new(graph, false, &source_parser);
    let parse_result = parser.parse(&"file://fuzz.ts".try_into().unwrap());

    let mut doc_nodes = match parse_result {
      Ok(nodes) => nodes,
      Err(_) => return,
    };

    doc_nodes.retain(|doc_node| doc_node.kind != DocNodeKind::Import);
    let _ = DocPrinter::new(&doc_nodes, true, false);
  };

  block_on(future);
});

struct SourceFileLoader<'a> {
  source: &'a str,
}

impl Loader for SourceFileLoader<'_> {
  fn load(
    &mut self,
    specifier: &ModuleSpecifier,
    _is_dynamic: bool,
  ) -> LoadFuture {
    let result = {
      Ok(Some(LoadResponse::Module {
        specifier: specifier.clone(),
        maybe_headers: None,
        content: self.source.into(),
      }))
    };
    Box::pin(future::ready(result))
  }
}
