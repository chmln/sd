use crate::{Error, Result};
use memmap2::MmapMut;
use std::{fs, io::Write, ops::DerefMut, path::Path};

pub(crate) fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    let path = fs::canonicalize(path)?;

    let temp = tempfile::NamedTempFile::new_in(
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
        let mut mmap_temp = unsafe { MmapMut::map_mut(file)? };
        mmap_temp.deref_mut().write_all(data)?;
        mmap_temp.flush_async()?;
    }

    temp.persist(&path)?;

    Ok(())
}
