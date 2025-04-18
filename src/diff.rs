use similar::TextDiff;
use std::path::Path;

use crate::error::{Error, Result};

pub(crate) fn create_udiff<UO: AsRef<[u8]>, UN: AsRef<[u8]>>(
    old: UO,
    new: UN,
    context: usize,
    path: Option<&Path>,
) -> Result<String> {
    let diff = TextDiff::from_lines(old.as_ref(), new.as_ref());
    let path = if let Some(path) = path {
        if let Some(spath) = path.to_str() {
            spath
        } else {
            return Err(Error::InvalidPath(path.into()));
        }
    } else {
        ""
    };

    let mut udiff = diff.unified_diff();
    udiff.header(path, path);
    udiff.context_radius(context);

    Ok(udiff.to_string())
}
