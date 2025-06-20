use petgraph::{
    algo::toposort,
    graph::{DiGraph, NodeIndex},
    visit::{DfsPostOrder, EdgeRef},
};
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    rc::Rc,
};
use tenda_common::{source::IdentifiedSource, span::SourceSpan};
use tenda_parser::{self, ast};

use crate::{loader_error::LoaderError, Cycle, ResolutionError};

const EXTENSIONS: [&str; 2] = ["tenda", "tnd"];

#[derive(Debug)]
pub struct LoadOutput {
    pub modules: indexmap::IndexMap<PathBuf, ModuleData>,
    pub entry_path: PathBuf,
}

#[derive(Debug)]
pub struct PromptOutput {
    pub prompt: ModuleData,
    pub modules: indexmap::IndexMap<PathBuf, ModuleData>,
}

#[derive(Debug, Clone)]
pub struct ModuleData {
    pub unit: ModuleUnit,
    pub source_id: IdentifiedSource,
    pub text: Rc<str>,
}

#[derive(Debug, Clone)]
pub struct ModuleUnit {
    pub ast: ast::Ast,
    pub imports: Vec<ResolvedImport>,
}

#[derive(Debug, Clone)]
pub struct ResolvedImport {
    pub raw_path: String,
    pub alias: String,
    pub canonical: PathBuf,
    pub span: SourceSpan,
}

#[derive(Clone)]
struct ImportCtx {
    span: SourceSpan,
    source_id: IdentifiedSource,
    text: Rc<str>,
}

impl ImportCtx {
    fn new(span: SourceSpan, source_id: IdentifiedSource, text: Rc<str>) -> Self {
        Self {
            span,
            source_id,
            text,
        }
    }
}

pub struct Loader<R: Fn(&Path) -> io::Result<String>> {
    read: R,
    cache: HashMap<PathBuf, ModuleData>,
    virtual_src: HashMap<PathBuf, Rc<str>>,
    graph: DiGraph<PathBuf, ()>,
    node_ix: HashMap<PathBuf, NodeIndex>,
    edge_span: HashMap<(PathBuf, PathBuf), SourceSpan>,
    stack: Vec<PathBuf>,
    counter: usize,
}

impl<R> Loader<R>
where
    R: Fn(&Path) -> io::Result<String>,
{
    pub fn new(reader: R) -> Self {
        Self {
            read: reader,
            cache: HashMap::new(),
            virtual_src: HashMap::new(),
            graph: DiGraph::new(),
            node_ix: HashMap::new(),
            edge_span: HashMap::new(),
            stack: Vec::new(),
            counter: 0,
        }
    }

    pub fn load_entry<P: Into<PathBuf>>(&mut self, entry: P) -> Result<LoadOutput, LoaderError> {
        let entry = canonical(&entry.into());

        self.visit_file(&entry, None)?;
        self.ensure_acyclic()?;

        Ok(LoadOutput {
            modules: self.post_order(),
            entry_path: entry,
        })
    }

    pub fn register_virtual(&mut self, src: String) -> Result<PromptOutput, LoaderError> {
        let path = PathBuf::from(format!("<anônimo {}>", self.counter + 1));

        self.counter += 1;
        self.virtual_src.insert(path.clone(), Rc::from(src));

        let prompt = self.parse_source(&path, None)?;

        for imp in &prompt.unit.imports {
            let ctx = ImportCtx::new(imp.span.clone(), prompt.source_id, Rc::clone(&prompt.text));
            self.visit_file(&imp.canonical, Some(ctx))?;
        }

        self.ensure_acyclic()?;

        Ok(PromptOutput {
            prompt,
            modules: self.post_order(),
        })
    }
}

impl<R> Loader<R>
where
    R: Fn(&Path) -> io::Result<String>,
{
    fn visit_file(&mut self, path: &Path, ctx: Option<ImportCtx>) -> Result<(), LoaderError> {
        if self.cache.contains_key(path) {
            return Ok(());
        }

        if self.stack.iter().any(|p| p == path) {
            let start = self.stack.iter().position(|p| p == path).unwrap();
            let mut cycle = self.stack[start..].to_vec();

            cycle.push(path.to_path_buf());

            let ctx = ctx.expect("imported files always provide context");

            return Err(LoaderError::Resolution {
                error: Box::new(ResolutionError::Circular {
                    cycle: Cycle(cycle),
                    span: ctx.span.clone(),
                }),
                source_id: ctx.source_id,
                source_code: ctx.text,
            });
        }

        self.stack.push(path.to_path_buf());

        let md = self.parse_source(path, ctx.clone())?;
        let importer_ix = self.get_node(path);

        for imp in &md.unit.imports {
            let dep_ix = self.get_node(&imp.canonical);

            self.graph.add_edge(importer_ix, dep_ix, ());
            self.edge_span.insert(
                (path.to_path_buf(), imp.canonical.clone()),
                imp.span.clone(),
            );

            let next_ctx = ImportCtx::new(imp.span.clone(), md.source_id, Rc::clone(&md.text));

            self.visit_file(&imp.canonical, Some(next_ctx))?;
        }

        self.cache.insert(path.to_path_buf(), md);
        self.stack.pop();

        Ok(())
    }

    fn parse_source(&self, path: &Path, ctx: Option<ImportCtx>) -> Result<ModuleData, LoaderError> {
        let text: Rc<str> = match self.virtual_src.get(path) {
            Some(t) => Rc::clone(t),
            None => match self.read_file(path) {
                Ok(s) => Rc::from(s),
                Err(e) => match ctx {
                    Some(ctx) => {
                        return Err(LoaderError::Resolution {
                            error: Box::new(ResolutionError::Io {
                                path: path.to_path_buf(),
                                kind: e.kind(),
                                span: ctx.span,
                            }),
                            source_id: ctx.source_id,
                            source_code: ctx.text,
                        })
                    }
                    None => {
                        return Err(LoaderError::EntryFileNotFound {
                            path: path.to_path_buf(),
                            source: e,
                        })
                    }
                },
            },
        };

        let mut sid = IdentifiedSource::new();

        let sid_name = Box::leak(path.to_string_lossy().into_owned().into_boxed_str());
        sid.set_name(sid_name);

        let tokens = tenda_scanner::Scanner::new(&text, sid)
            .scan()
            .map_err(|e| LoaderError::Lexical {
                path: path.into(),
                source_id: sid,
                source_code: Rc::clone(&text),
                errors: e,
            })?;

        let po = tenda_parser::Parser::new(&tokens, sid)
            .parse()
            .map_err(|e| LoaderError::Parse {
                path: path.into(),
                source_id: sid,
                source_code: Rc::clone(&text),
                errors: e,
            })?;

        let imports = po
            .imports
            .into_iter()
            .map(|spec| ResolvedImport {
                raw_path: spec.raw_path.clone(),
                alias: spec.alias,
                canonical: canonical(&path.parent().unwrap().join(&spec.raw_path)),
                span: spec.span,
            })
            .collect();

        Ok(ModuleData {
            unit: ModuleUnit {
                ast: po.ast,
                imports,
            },
            source_id: sid,
            text,
        })
    }

    fn read_file(&self, path: &Path) -> io::Result<String> {
        match (self.read)(path) {
            Ok(txt) => Ok(txt),
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            Err(err) if path.extension().is_some() => Err(err),
            Err(first_err) => {
                for ext in EXTENSIONS {
                    let mut alt = path.to_path_buf();
                    alt.set_extension(ext);

                    match (self.read)(&alt) {
                        Ok(txt) => return Ok(txt),
                        Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                        Err(e) => return Err(e),
                    }
                }

                Err(first_err)
            }
        }
    }

    fn get_node(&mut self, p: &Path) -> NodeIndex {
        *self
            .node_ix
            .entry(p.to_path_buf())
            .or_insert_with(|| self.graph.add_node(p.to_path_buf()))
    }

    fn ensure_acyclic(&self) -> Result<(), LoaderError> {
        if toposort(&self.graph, None).is_ok() {
            return Ok(());
        }

        let bad_idx = toposort(&self.graph, None).unwrap_err().node_id();
        let bad_path = &self.graph[bad_idx];

        let incoming = self
            .graph
            .edges_directed(bad_idx, petgraph::Incoming)
            .next()
            .expect("cycle implies >=1 incoming edge");

        let importer = &self.graph[incoming.source()];
        let span = self.edge_span[&(importer.clone(), bad_path.clone())].clone();
        let md = &self.cache[importer];

        Err(LoaderError::Resolution {
            error: Box::new(ResolutionError::Circular {
                cycle: Cycle(vec![importer.clone(), bad_path.clone()]),
                span,
            }),
            source_id: md.source_id,
            source_code: Rc::clone(&md.text),
        })
    }

    fn post_order(&self) -> indexmap::IndexMap<PathBuf, ModuleData> {
        let mut vec = Vec::new();
        let mut dfs = DfsPostOrder::empty(&self.graph);

        for n in self.graph.node_indices() {
            dfs.move_to(n);

            while let Some(ix) = dfs.next(&self.graph) {
                vec.push(self.graph[ix].clone());
            }
        }

        vec.dedup();
        vec.into_iter()
            .filter_map(|p| self.cache.get(&p).cloned().map(|m| (p, m)))
            .collect()
    }
}

fn canonical(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}
