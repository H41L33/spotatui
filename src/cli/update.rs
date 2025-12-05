use anyhow::Result;
use self_update::cargo_crate_version;

/// Check for updates and optionally install the latest version
pub fn check_for_update(do_update: bool) -> Result<()> {
  let current_version = cargo_crate_version!();

  println!("Current version: v{}", current_version);
  println!("Checking for updates...");

  let status = self_update::backends::github::Update::configure()
    .repo_owner("LargeModGames")
    .repo_name("spotatui")
    .bin_name("spotatui")
    .show_download_progress(true)
    .current_version(current_version)
    .no_confirm(false)
    .build()?;

  let latest = status.get_latest_release()?;

  // Remove 'v' prefix if present for comparison
  let latest_version = latest.version.trim_start_matches('v');

  if latest_version == current_version {
    println!("✓ You are already running the latest version!");
    return Ok(());
  }

  println!("New version available: v{}", latest_version);

  if do_update {
    println!("\nDownloading and installing update...");

    let result = status.update()?;
    match result {
      self_update::Status::UpToDate(_) => {
        println!("✓ Already up to date!");
      }
      self_update::Status::Updated(v) => {
        println!("✓ Successfully updated to v{}!", v);
        println!("\nPlease restart spotatui to use the new version.");
      }
    }
  } else {
    println!("\nRun `spotatui update --install` to install the update.");
  }

  Ok(())
}
