use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use storekit::Product;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var_os("STOREKIT_SMOKE_BUNDLED").is_none() {
        return relaunch_inside_app_bundle();
    }

    let products = Product::products_for(["nonexistent.product.id"])?;
    if products.is_empty() {
        println!("✅ storekit product fetch (empty) OK");
        return Ok(());
    }

    Err(format!(
        "expected no products for nonexistent.product.id, received {} entries",
        products.len()
    )
    .into())
}

fn relaunch_inside_app_bundle() -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = env::current_exe()?;
    let crate_root = env::current_dir()?;
    let app_root = crate_root.join("target/storekit-smoke.app");
    let contents_dir = app_root.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let bundle_exe = macos_dir.join(executable_name(&current_exe));

    fs::create_dir_all(&macos_dir)?;
    fs::copy(&current_exe, &bundle_exe)?;
    fs::set_permissions(&bundle_exe, fs::metadata(&current_exe)?.permissions())?;
    fs::write(contents_dir.join("Info.plist"), info_plist())?;

    let status = Command::new(&bundle_exe)
        .env("STOREKIT_SMOKE_BUNDLED", "1")
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("bundled smoke runner exited with status {status}").into())
    }
}

fn executable_name(path: &Path) -> String {
    path.file_name().and_then(|value| value.to_str()).map_or_else(
        || "01_storekit_smoke".to_owned(),
        ToOwned::to_owned,
    )
}

fn info_plist() -> String {
    [
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
        "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">",
        "<plist version=\"1.0\">",
        "<dict>",
        "  <key>CFBundleExecutable</key>",
        "  <string>01_storekit_smoke</string>",
        "  <key>CFBundleIdentifier</key>",
        "  <string>fish.doom.storekit.smoke</string>",
        "  <key>CFBundleName</key>",
        "  <string>storekit-smoke</string>",
        "  <key>CFBundlePackageType</key>",
        "  <string>APPL</string>",
        "  <key>LSMinimumSystemVersion</key>",
        "  <string>12.0</string>",
        "</dict>",
        "</plist>",
    ]
    .join("\n")
}
