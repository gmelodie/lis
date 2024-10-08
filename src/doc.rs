use crate::{objects::FromNamespaceId, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LisDoc {
    doc_id: NamespaceId,
}

impl LisDoc {
    pub async fn new(node: &Iroh) -> Result<Self> {
        let doc = node.docs().create().await?;

        Ok(Self { doc_id: doc.id() })
    }

    pub async fn load(node: &Iroh, id: NamespaceId) -> Result<Self> {
        Self::from_namespace_id(node, id).await
    }

    pub async fn doc_type(&self, node: &Iroh) -> Result<DocType> {
        Ok(DocType::from(self.get(node, ".type").await?))
    }

    pub async fn set<T: Into<Bytes>>(&self, node: &Iroh, key: Key, value: T) -> Result<()> {
        let doc = Self::load(node, self.doc_id).await?;
        doc.set_bytes(node.authors().default().await?, key.into(), value.into())
            .await?;
        Ok(())
    }

    pub async fn get<T: Into<Bytes>>(&self, node: &Iroh, key: Key) -> Result<Bytes> {
        let key = Key::from(key)?;
        match doc
            .iroh_doc(node)
            .get_exact(node.authors().default().await?, key, false)
            .await?
        {
            Some(entry) => entry.content_bytes(&node.clone()).await,
            None => Err(anyhow!("key not in doc")),
        }
    }

    async fn iroh_doc(&self, node: &Iroh) -> Result<Doc> {
        node.docs()
            .open(self.doc_id)
            .await?
            .ok_or(anyhow!("could not open iroh doc"))
    }
}

impl FromNamespaceId for LisDoc {
    async fn from_namespace_id(node: &Iroh, id: NamespaceId) -> Result<Self> {
        Ok(Self { doc_id: id })
    }
}

#[derive(Debug, PartialEq)]
pub enum DocType {
    DirDoc,
    ChildrenDoc,
    MetadataDoc,
    FileChunkDoc,
    FileDoc,
    RootDoc,
    Unknown,
}

impl From<Bytes> for DocType {
    fn from(bytes: Bytes) -> Self {
        match String::from_utf8(bytes.to_vec())?.as_ref() {
            "root" => DocType::RootDoc,
            "dir" => DocType::DirDoc,
            "children" => DocType::ChildrenDoc,
            "metadata" => DocType::MetadataDoc,
            "file" => DocType::FileDoc,
            "fileChunk" => DocType::FileChunkDoc,
            _ => DocType::Unknown,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_doc_type() {
        let node = iroh::node::Node::memory().spawn().await.unwrap();
        let doc = LisDoc::new(&node);

        // set type to "children"
        doc.set(node, ".type", "children").await.unwrap();
        assert_eq!(doc.doc_type(&node).await.unwrap(), DocType::ChildrenDoc);
    }
}
