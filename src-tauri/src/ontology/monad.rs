use crate::geometry::polar_space::PolarCoord;
use crate::language::Language;
use serde::{Deserialize, Serialize};

/// Represents the kind of code entity that a Monad encapsulates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MonadKind {
    Function,
    Struct,
    Enum,
    Impl,
    Trait,
    Module,
    Constant,
    TypeAlias,
    Unknown,
}

impl std::fmt::Display for MonadKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonadKind::Function => write!(f, "fn"),
            MonadKind::Struct => write!(f, "struct"),
            MonadKind::Enum => write!(f, "enum"),
            MonadKind::Impl => write!(f, "impl"),
            MonadKind::Trait => write!(f, "trait"),
            MonadKind::Module => write!(f, "mod"),
            MonadKind::Constant => write!(f, "const"),
            MonadKind::TypeAlias => write!(f, "type"),
            MonadKind::Unknown => write!(f, "unknown"),
        }
    }
}

/// Categorization of the change when a Monad evolves between rings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeltaType {
    /// Brand new monad with no ancestor in the previous ring.
    Added,
    /// Content changed but semantic identity (name) persists.
    Modified,
    /// Name changed but semantic hash matches a previous monad.
    Renamed,
    /// Monad existed in previous ring but is absent in the new one.
    Deleted,
    /// No change detected.
    Unchanged,
}

/// The minimal functional unit — a logical code entity, not a text line.
/// Each Monad is located at an exact coordinate on the circumference
/// of a temporal ring in the Mandala.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monad {
    pub id: String,              // Unique identifier (semantic hash)
    pub coord: PolarCoord,       // Position in radial space
    pub content: String,         // Source code content
    pub name: String,            // Display name (function name, struct name, etc.)
    pub ring: u32,               // Ring level (expansion number)
    pub kind: MonadKind,         // Entity type (fn, struct, enum, impl, trait, etc.)
    pub semantic_hash: String,   // blake3 hash of whitespace-stripped content
    pub line_start: u32,         // Starting line in original source
    pub line_end: u32,           // Ending line in original source
    pub language: String,        // Source language ("rust", "typescript", etc.)
    #[serde(default)]
    pub is_archived: bool,       // Soft delete flag for archival
}

impl Monad {
    /// Creates a new monad with full metadata.
    pub fn spawn(
        id: String,
        name: String,
        coord: PolarCoord,
        content: String,
        ring: u32,
    ) -> Self {
        let semantic_hash = crate::ontology::semantic_hash::generate_pure_hash(&content);
        Self {
            id,
            coord,
            content,
            name,
            ring,
            kind: MonadKind::Unknown,
            semantic_hash,
            line_start: 0,
            line_end: 0,
            language: "rust".to_string(),
            is_archived: false,
        }
    }

    /// Creates a new monad with explicit kind and line span.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_typed(
        id: String,
        name: String,
        coord: PolarCoord,
        content: String,
        ring: u32,
        kind: MonadKind,
        line_start: u32,
        line_end: u32,
        language: &str,
    ) -> Self {
        let semantic_hash = crate::ontology::semantic_hash::generate_pure_hash(&content);
        Self {
            id,
            coord,
            content,
            name,
            ring,
            kind,
            semantic_hash,
            line_start,
            line_end,
            language: language.to_string(),
            is_archived: false,
        }
    }

    /// Returns true if this monad has semantically different content from another.
    pub fn is_semantically_different(&self, other: &Monad) -> bool {
        self.semantic_hash != other.semantic_hash
    }

    
    pub fn language_enum(&self) -> Language {
        Language::from_extension(&self.language)
    }
}
