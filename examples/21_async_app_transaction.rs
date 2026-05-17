//! Example: async app transaction fetch using [`AsyncAppTransaction`].
//!
//! Run with:
//!   `cargo run --example 21_async_app_transaction --features async`
//!
//! On a headless machine without an active App Store session this call will
//! fail with a framework error. The example handles that gracefully and
//! exits 0.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pollster::block_on(async {
        use storekit::async_api::AsyncAppTransaction;

        match AsyncAppTransaction::shared().await {
            Ok(vr) => {
                let app_tx = vr.payload();
                println!("AsyncAppTransaction: bundle={}", app_tx.bundle_id);
                println!("  app_version   : {}", app_tx.app_version);
                println!("  environment   : {}", app_tx.environment.as_str());
                if !vr.is_verified() {
                    println!(
                        "  (verification failed — expected in dev; failure={:?})",
                        vr.verification_failure()
                    );
                }
            }
            Err(e) => {
                println!(
                    "AppTransaction fetch failed (expected without App Store session): {e}"
                );
            }
        }

        Ok(())
    })
}
