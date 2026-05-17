use crate::Registry;
use std::path::Path;

/// Detect the language of a file using the registry. v1: extension-only.
/// Shebang and content sniffing can be layered on top later without breaking
/// the call site.
pub fn detect<'r>(registry: &'r Registry, path: &Path) -> Option<&'r crate::LanguageEntry> {
    registry.lookup_by_path(path)
}
