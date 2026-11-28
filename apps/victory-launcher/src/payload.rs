use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const FILES: &[(&str, &[u8])] = &[
    ("LICENSE.txt", include_bytes!("../../../LICENSE")),
    ("THIRD_PARTY.md", include_bytes!("../../../THIRD_PARTY.md")),
    (
        "LICENSE-Streamline.txt",
        include_bytes!("../../../LICENSE-Streamline.txt"),
    ),
    ("LICENSE-zip.txt", include_bytes!("../LICENSE-zip.txt")),
    (
        "tools/setup-agility.ps1",
        include_bytes!("../../../tools/setup-agility.ps1"),
    ),
    (
        "tools/setup-dlss.ps1",
        include_bytes!("../../../tools/setup-dlss.ps1"),
    ),
    (
        "tools/setup-streamline.ps1",
        include_bytes!("../../../tools/setup-streamline.ps1"),
    ),
];

pub fn data_home() -> io::Result<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(|path| PathBuf::from(path).join("VIctoryUnified").join("v2.0"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "LOCALAPPDATA is unavailable"))
}

pub fn install(root: &Path) -> io::Result<()> {
    let launcher = std::env::current_exe()?;
    let runtime = launcher.with_file_name("victory.exe");
    let bytes = fs::read(&runtime)
        .map_err(|error| io::Error::new(error.kind(), format!("{}: {error}", runtime.display())))?;
    for (relative, bytes) in [("bin/victory.exe", bytes.as_slice())]
        .into_iter()
        .chain(FILES.iter().copied())
    {
        let path = root.join(relative);
        if fs::read(&path).is_ok_and(|installed| installed == bytes) {
            continue;
        }
        fs::create_dir_all(path.parent().expect("payload path has a parent"))?;
        let tmp = path.with_extension(format!("install-{}", std::process::id()));
        let result = (|| {
            let mut file = fs::File::create(&tmp)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&tmp, &path)
        })();
        if let Err(error) = result {
            let _ = fs::remove_file(&tmp);
            return Err(io::Error::new(
                error.kind(),
                format!("{}: {error}", path.display()),
            ));
        }
    }
    Ok(())
}
