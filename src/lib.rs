use anyhow::Result;
use futures_lite::StreamExt;
use iroh::blobs::store::Store;
use iroh::{client::docs::Doc, node::Node, util::fs::path_to_key};
use std::path::{Path, PathBuf};

mod cli;
pub use cli::Cli;

pub struct Lis<D: Store + Sized> {
    pub iroh_node: Node<D>,
    author: iroh::docs::AuthorId,
}

/// In memory node.
pub type MemNode = Node<iroh_blobs::store::mem::Store>;

/// Persistent node.
pub type FsNode = Node<iroh_blobs::store::fs::Store>;

impl MemNode {
    /// Returns a new builder for the [`Node`], by default configured to run in memory.
    ///
    /// Once done with the builder call [`Builder::spawn`] to create the node.
    pub fn memory() -> Builder<iroh_blobs::store::mem::Store> {
        Builder::default()
    }
}

impl FsNode {
    /// Returns a new builder for the [`Node`], configured to persist all data
    /// from the given path.
    ///
    /// Once done with the builder call [`Builder::spawn`] to create the node.
    pub async fn persistent(
        root: impl AsRef<Path>,
    ) -> Result<Builder<iroh_blobs::store::fs::Store>> {
        Builder::default().persist(root).await
    }
}

enum IrohNode {
    Mem(Node<iroh::blobs::store::mem::Store>),
    Disk(Node<iroh::blobs::store::fs::Store>),
}

pub enum IrohNodeType {
    Mem,
    Disk(PathBuf),
}

// /// Extracts node from IrohNode enum
// fn get_node(iroh_node: IrohNode) -> Box<dyn Store> {
//     match iroh_node {
//         IrohNode::Mem(node) => Box::new(node),
//         IrohNode::Disk(node) => Box::new(node),
//     }
// }

impl Lis {
    pub async fn new(node_type: IrohNodeType) -> Result<Self> {
        let iroh_node = match node_type {
            IrohNodeType::Mem => iroh::node::Node::memory().spawn().await?,
            IrohNodeType::Disk(root) => iroh::node::Node::persistent(root).await?.spawn().await?,
        };
        let author = iroh_node.authors().create().await?; // TODO: add this to Lis
        let lis = Lis {
            iroh_node, // TODO: option to move to disk node
            author,    // TODO: add this to Lis
        };
        Ok(lis)
    }

    /// Adds a file to new doc
    /// Creates new doc
    pub async fn add_file(&mut self, path: &Path) -> Result<()> {
        // Create document
        let mut doc = self.iroh_node.docs().create().await?;

        self.add_file_to_doc(path, &mut doc).await?;

        Ok(())
    }

    /// Adds a file to a previously created document
    pub async fn add_file_to_doc(&mut self, path: &Path, doc: &mut Doc) -> Result<()> {
        // read file
        let bytes = std::fs::read(path)?;

        let key = path_to_key(&path, None, None)?; // TODO: use prefix and root (see path_to_key
                                                   // docs)
        doc.import_file(self.author, key, path, false)
            .await?
            .collect::<Vec<_>>()
            .await;

        Ok(())
    }

    /// Removes a doc
    pub async fn rm_doc(&mut self, doc: &Doc) -> Result<()> {
        self.iroh_node.docs().drop_doc(doc.id()).await
    }
}
