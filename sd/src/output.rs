use crate::{Error, Result};
use std::{fs, io::Write, path::Path};

pub(crate) fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    let path = fs::canonicalize(path)?;

    let mut temp = tempfile::NamedTempFile::new_in(
        path.parent()
            .ok_or_else(|| Error::InvalidPath(path.to_path_buf()))?,
    )?;

    let file = temp.as_file();
    file.set_len(data.len() as u64)?;

    if let Ok(metadata) = fs::metadata(&path) {
        file.set_permissions(metadata.permissions()).ok();

        // Explicitly retain ownership
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, fchown};
            fchown(file, Some(metadata.uid()), Some(metadata.gid()))?;
            metadata.gid();
        }
    }

    if !data.is_empty() {
        temp.as_file_mut().write_all(data)?;
        temp.as_file_mut().flush()?;
    }

    temp.persist(&path)?;

    Ok(())
}
