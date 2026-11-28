use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};

use tempfile::{TempDir, tempdir};
use zip::ZipArchive;
use zip::read::{ExtractOptions, SymlinkExtractAction};

const SAVE_ROOT: &str = "data/7952987A/saves";
const PROFILE: &str = "0009000000000001/00000000-0000-0000-0000-00007952987a";
const PLAYER: &str = "Player~333dc56ddffd8ea0";

#[derive(Debug)]
pub struct Imported {
    pub slots: Vec<String>,
    pub player_added: bool,
    pub backup: Option<PathBuf>,
}

fn is_slot(name: &str) -> bool {
    let Some((stem, hash)) = name.split_once('~') else {
        return false;
    };
    stem.get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("SARC"))
        && stem.get(4..).is_some_and(|number| {
            !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())
        })
        && !hash.is_empty()
        && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_link(metadata: &fs::Metadata) -> bool {
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

fn find_slots(dir: &Path, slots: &mut BTreeMap<String, (String, PathBuf)>) -> io::Result<()> {
    if let Some(name) = dir.file_name().and_then(|name| name.to_str())
        && is_slot(name)
    {
        if slots
            .insert(name.to_ascii_lowercase(), (name.into(), dir.into()))
            .is_some()
        {
            return Err(io::Error::other(format!(
                "Save contains more than one {name} folder."
            )));
        }
        return Ok(());
    }
    for item in fs::read_dir(dir)? {
        let item = item?;
        let metadata = fs::symlink_metadata(item.path())?;
        if metadata.is_dir() && !is_link(&metadata) {
            find_slots(&item.path(), slots)?;
        }
    }
    Ok(())
}

/// Copy an ordinary directory tree to a new location and count the regular files.
fn copy_tree(source: &Path, destination: &Path) -> io::Result<usize> {
    if is_link(&fs::symlink_metadata(source)?) {
        return Err(io::Error::other(format!(
            "Save folder contains a link: {}",
            source.display()
        )));
    }
    fs::create_dir(destination)?;
    let mut files = 0;
    for item in fs::read_dir(source)? {
        let item = item?;
        let from = item.path();
        let to = destination.join(item.file_name());
        let metadata = fs::symlink_metadata(&from)?;
        if is_link(&metadata) {
            return Err(io::Error::other(format!(
                "Save folder contains a link: {}",
                from.display()
            )));
        }
        if metadata.is_dir() {
            files += copy_tree(&from, &to)?;
        } else if metadata.is_file() {
            fs::copy(&from, &to)?;
            files += 1;
        }
    }
    Ok(files)
}

fn extract_zip(source: &Path) -> io::Result<TempDir> {
    let scratch = tempdir()?;
    let mut zip = ZipArchive::new(BufReader::new(File::open(source)?))?;
    zip.extract_with_options(
        scratch.path(),
        ExtractOptions::default()
            .symlink_action(SymlinkExtractAction::NoExtract)
            .enforce_header_consistency(true)
            .with_size_limit(2 << 30),
    )?;
    Ok(scratch)
}

pub fn import(root: &Path, source: &Path) -> io::Result<Imported> {
    let metadata = fs::symlink_metadata(source)?;
    if is_link(&metadata) {
        return Err(io::Error::other(
            "Choose the actual save folder or ZIP file.",
        ));
    }
    let archive = if metadata.is_dir() {
        None
    } else if source
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
    {
        Some(extract_zip(source)?)
    } else {
        return Err(io::Error::other("Choose a save folder or ZIP file."));
    };

    let input = archive.as_ref().map_or(source, TempDir::path);
    let mut slots = BTreeMap::new();
    find_slots(input, &mut slots)?;
    if slots.is_empty() {
        return Err(io::Error::other(
            "No Story Mode save folders named SARC… were found.",
        ));
    }

    // Prepare the complete replacement beside the save directory so the final moves stay on
    // the same filesystem. A failed read, malformed slot, or full disk leaves existing saves intact.
    let live = root.join(SAVE_ROOT);
    let parent = live.parent().expect("save root has a parent");
    fs::create_dir_all(parent)?;
    let staging = tempfile::Builder::new()
        .prefix(".save-import-")
        .tempdir_in(parent)?;
    let proposed = staging.path().join("saves");
    let has_existing = match fs::symlink_metadata(&live) {
        Ok(_) => {
            copy_tree(&live, &proposed)?;
            true
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir(&proposed)?;
            false
        }
        Err(error) => return Err(error),
    };

    let profile = proposed.join(PROFILE);
    fs::create_dir_all(&profile)?;
    let player = profile.join(PLAYER);
    let mut player_added = false;
    // A fresh profile can use the Player container shipped beside its slots. An
    // established profile keeps its own Player container, and mixed exports are ambiguous.
    if !player.try_exists()?
        && let Some(parent) = slots.values().next().and_then(|(_, slot)| slot.parent())
        && parent.starts_with(input)
        && slots
            .values()
            .all(|(_, slot)| slot.parent() == Some(parent))
        && parent.join(PLAYER).is_dir()
    {
        if copy_tree(&parent.join(PLAYER), &player)? == 0 {
            return Err(io::Error::other("The imported Player container is empty."));
        }
        player_added = true;
    }
    let mut names = Vec::new();
    for (name, slot) in slots.into_values() {
        let dest = profile.join(&name);
        match fs::symlink_metadata(&dest) {
            Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(&dest)?,
            Ok(_) => {
                return Err(io::Error::other(format!(
                    "Expected a save directory at {}.",
                    dest.display()
                )));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        if copy_tree(&slot, &dest)? == 0 {
            return Err(io::Error::other(format!("{name} contains no save files.")));
        }
        names.push(name);
    }

    let backup = if has_existing {
        let dir = root.join("save-backups");
        fs::create_dir_all(&dir)?;
        let backup = tempfile::Builder::new()
            .prefix("before-import-")
            .tempdir_in(&dir)?;
        fs::rename(&live, backup.path().join("saves"))?;
        Some(backup.keep())
    } else {
        None
    };
    if let Err(error) = fs::rename(&proposed, &live) {
        if let Some(backup) = &backup
            && let Err(rollback) = fs::rename(backup.join("saves"), &live)
        {
            return Err(io::Error::other(format!(
                "Could not install the save ({error}) or restore the original ({rollback}). The original saves are at {}.",
                backup.join("saves").display()
            )));
        }
        return Err(error);
    }

    Ok(Imported {
        slots: names,
        player_added,
        backup,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    fn put(path: &Path, bytes: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn folder_import_backs_up_overwritten_slot_and_preserves_profile() {
        let root = tempdir().unwrap();
        let profile = root.path().join(SAVE_ROOT).join(PROFILE);
        let original = profile.join("SARC0000~abc/old");
        put(&original, b"original");
        let profile_data = profile.join("Player~333dc56ddffd8ea0/profile");
        put(&profile_data, b"player");

        let input = tempdir().unwrap();
        let slot = input.path().join("SARC0000~abc");
        put(&slot.join("new"), b"replacement");
        put(
            &input.path().join("Player~333dc56ddffd8ea0/profile"),
            b"foreign player",
        );

        let result = import(root.path(), input.path()).unwrap();
        assert_eq!(result.slots, ["SARC0000~abc"]);
        assert!(!result.player_added);
        let backup = result.backup.unwrap().join("saves").join(PROFILE);
        assert_eq!(
            fs::read(backup.join("SARC0000~abc/old")).unwrap(),
            b"original"
        );
        assert_eq!(
            fs::read(backup.join("Player~333dc56ddffd8ea0/profile")).unwrap(),
            b"player"
        );
        assert!(!original.exists());
        assert_eq!(
            fs::read(profile.join("SARC0000~abc/new")).unwrap(),
            b"replacement"
        );
        assert_eq!(fs::read(profile_data).unwrap(), b"player");
        assert_eq!(fs::read(slot.join("new")).unwrap(), b"replacement");
    }

    #[test]
    fn compressed_zip_imports_a_nested_slot_without_explicit_directory_entries() {
        let source = tempdir().unwrap();
        let archive = source.path().join("story.zip");
        let mut writer = ZipWriter::new(File::create(&archive).unwrap());
        writer
            .start_file(
                "outer/inner/SARC0001~123abc/slot.bin",
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
            )
            .unwrap();
        writer.write_all(b"compressed save").unwrap();
        writer
            .start_file(
                "outer/inner/Player~333dc56ddffd8ea0/v5/0",
                SimpleFileOptions::default(),
            )
            .unwrap();
        writer.write_all(b"player metadata").unwrap();
        writer
            .start_file(
                "outer/inner/SettingsProfileSettings~0b4e25fca06268d0/v1/0",
                SimpleFileOptions::default(),
            )
            .unwrap();
        writer.write_all(b"foreign settings").unwrap();
        writer.finish().unwrap();

        let root = tempdir().unwrap();
        let result = import(root.path(), &archive).unwrap();
        assert_eq!(result.slots, ["SARC0001~123abc"]);
        assert!(result.backup.is_none());
        assert!(result.player_added);
        let profile = root.path().join(SAVE_ROOT).join(PROFILE);
        assert_eq!(
            fs::read(profile.join(PLAYER).join("v5/0")).unwrap(),
            b"player metadata"
        );
        assert!(
            !profile
                .join("SettingsProfileSettings~0b4e25fca06268d0")
                .exists()
        );
        assert_eq!(
            fs::read(
                root.path()
                    .join(SAVE_ROOT)
                    .join(PROFILE)
                    .join("SARC0001~123abc/slot.bin")
            )
            .unwrap(),
            b"compressed save"
        );
    }

    #[test]
    fn empty_slot_does_not_replace_a_valid_save() {
        let root = tempdir().unwrap();
        let original = root
            .path()
            .join(SAVE_ROOT)
            .join(PROFILE)
            .join("SARC0000~abc/game");
        put(&original, b"existing save");
        let input = tempdir().unwrap();
        let slot = input.path().join("SARC0000~abc");
        fs::create_dir(&slot).unwrap();
        assert!(
            import(root.path(), &slot)
                .unwrap_err()
                .to_string()
                .contains("contains no save files")
        );
        assert_eq!(fs::read(original).unwrap(), b"existing save");
        assert!(!root.path().join("save-backups").exists());
    }
}
