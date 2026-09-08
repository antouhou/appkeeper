use serde::{Deserialize, Serialize};

/// A serializable application ID with a namespace and a backend-defined value.
///
/// Equality compares both strings. Unsupported IDs can still be saved and loaded.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AppId {
    namespace: String,
    value: String,
}

impl AppId {
    /// Stores both strings unchanged, without validation.
    pub fn from_parts(namespace: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            value: value.into(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests;
