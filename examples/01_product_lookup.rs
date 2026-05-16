use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use storekit::Product;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var_os("STOREKIT_PRODUCT_LOOKUP_BUNDLED").is_none() {
        return relaunch_inside_app_bundle();
    }

    match Product::products_for(["nonexistent.product.id"]) {
        Ok(products) if products.is_empty() => println!("✅ product lookup OK (empty result)"),
        Ok(products) => println!("ℹ️ product lookup returned {} products", products.len()),
        Err(error) => println!("ℹ️ product lookup error: {error}"),
    }
    Ok(())
}

fn relaunch_inside_app_bundle() -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = env::current_exe()?;
    let crate_root = env::current_dir()?;
    let executable = executable_name(&current_exe);
    let app_root = crate_root.join("target/storekit-product-lookup.app");
    let contents_dir = app_root.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let bundle_exe = macos_dir.join(&executable);

    fs::create_dir_all(&macos_dir)?;
    fs::copy(&current_exe, &bundle_exe)?;
    fs::set_permissions(&bundle_exe, fs::metadata(&current_exe)?.permissions())?;
    fs::write(contents_dir.join("Info.plist"), info_plist(&executable))?;

    let status = Command::new(&bundle_exe)
        .env("STOREKIT_PRODUCT_LOOKUP_BUNDLED", "1")
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("bundled product lookup runner exited with status {status}").into())
    }
}

fn executable_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .map_or_else(|| "01_product_lookup".to_owned(), ToOwned::to_owned)
}

fn info_plist(executable_name: &str) -> String {
    [
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
        "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">",
        "<plist version=\"1.0\">",
        "<dict>",
        "  <key>CFBundleExecutable</key>",
        &format!("  <string>{executable_name}</string>"),
        "  <key>CFBundleIdentifier</key>",
        "  <string>fish.doom.storekit.product.lookup</string>",
        "  <key>CFBundleName</key>",
        "  <string>storekit-product-lookup</string>",
        "  <key>CFBundlePackageType</key>",
        "  <string>APPL</string>",
        "  <key>LSMinimumSystemVersion</key>",
        "  <string>12.0</string>",
        "</dict>",
        "</plist>",
    ]
    .join("\n")
}
