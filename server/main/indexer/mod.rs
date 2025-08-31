mod dir_entry_guesser;
mod index_oneshot;
mod index_series;
mod start_index;
mod utils;

use std::path::Path;

use tantivy::{
    Index, IndexReader, IndexWriter, ReloadPolicy,
    directory::MmapDirectory,
    schema::{FAST, Field, INDEXED, STORED, STRING, Schema, TEXT},
};
use tokio::sync::Mutex;

pub struct Indexer {
    pub fields: Fields,

    pub reader: IndexReader,
    pub writer: Mutex<IndexWriter>,
}

impl std::fmt::Debug for Indexer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Indexer")
            .field("fields", &self.fields)
            .finish()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Fields {
    title: Field,
    author: Field,
    description: Field,
    tag: Field,
    identity_path: Field,
    category_identity_path: Field,
    release_date: Field,
}

impl Indexer {
    pub fn new(
        index_path: impl AsRef<Path>,
        memory_budget_in_bytes: usize,
    ) -> tantivy::Result<Self> {
        let mut schema_builder = Schema::builder();

        let fields = Fields {
            title: schema_builder.add_text_field("title", TEXT),
            author: schema_builder.add_text_field("author", TEXT),
            description: schema_builder.add_text_field("description", TEXT),
            tag: schema_builder.add_text_field("tag", TEXT),
            identity_path: schema_builder.add_text_field("identity_path", STRING | STORED),
            category_identity_path: schema_builder
                .add_text_field("category_identity_path", STRING | STORED),
            release_date: schema_builder.add_date_field("release_date", INDEXED | FAST),
        };

        let schema = schema_builder.build();
        let dir = MmapDirectory::open(index_path)?;
        let index = Index::open_or_create(dir, schema)?;

        Ok(Self {
            fields,
            writer: Mutex::new(index.writer(memory_budget_in_bytes)?),
            reader: index
                .reader_builder()
                .reload_policy(ReloadPolicy::OnCommitWithDelay)
                .try_into()?,
        })
    }
}
