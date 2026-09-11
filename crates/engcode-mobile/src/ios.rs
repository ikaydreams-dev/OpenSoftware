use crate::{MobileError, Result};
use std::process::Command;

pub struct IosBuilder {
    app_name: String,
    bundle_id: String,
}

impl IosBuilder {
    pub fn new(app_name: String, bundle_id: String) -> Self {
        Self { app_name, bundle_id }
    }

    pub fn check_xcode(&self) -> Result<()> {
        let output = Command::new("xcodebuild")
            .arg("-version")
            .output();

        if output.is_err() {
            return Err(MobileError::IosBuildError(
                "Xcode not found. Please install Xcode from the App Store".to_string()
            ));
        }

        println!("✓ Xcode found");
        Ok(())
    }

    pub fn check_cocoapods(&self) -> Result<()> {
        let output = Command::new("pod")
            .arg("--version")
            .output();

        if output.is_err() {
            return Err(MobileError::IosBuildError(
                "CocoaPods not found. Install with: sudo gem install cocoapods".to_string()
            ));
        }

        println!("✓ CocoaPods found");
        Ok(())
    }

    pub fn install_pods(&self, ios_dir: &str) -> Result<()> {
        println!("Installing iOS pods...");
        let output = Command::new("pod")
            .args(&["install"])
            .current_dir(ios_dir)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::IosBuildError(format!(
                "Failed to install pods: {}",
                error_msg
            )));
        }

        println!("✓ Pods installed");
        Ok(())
    }

    pub fn list_simulators(&self) -> Result<Vec<String>> {
        let output = Command::new("xcrun")
            .args(&["simctl", "list", "devices", "available"])
            .output()?;

        if !output.status.success() {
            return Err(MobileError::IosBuildError(
                "Failed to list simulators".to_string()
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let simulators: Vec<String> = output_str
            .lines()
            .filter(|line| line.contains("iPhone"))
            .map(|s| s.trim().to_string())
            .collect();

        Ok(simulators)
    }

    pub fn build_for_simulator(&self, project_dir: &str) -> Result<()> {
        println!("Building for iOS simulator...");
        let output = Command::new("xcodebuild")
            .args(&[
                "-workspace",
                &format!("{}/ios/{}.xcworkspace", project_dir, self.app_name),
                "-scheme",
                &self.app_name,
                "-configuration",
                "Debug",
                "-sdk",
                "iphonesimulator",
                "-destination",
                "platform=iOS Simulator,name=iPhone 14",
            ])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::IosBuildError(format!(
                "Failed to build: {}",
                error_msg
            )));
        }

        println!("✓ iOS build successful");
        Ok(())
    }

    pub fn export_ipa(&self, archive_path: &str, export_path: &str) -> Result<String> {
        println!("Exporting IPA...");

        // Create export options plist
        let export_options = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>method</key>
    <string>app-store</string>
    <key>teamID</key>
    <string>YOUR_TEAM_ID</string>
</dict>
</plist>"#
        );

        std::fs::write(format!("{}/ExportOptions.plist", export_path), export_options)?;

        let output = Command::new("xcodebuild")
            .args(&[
                "-exportArchive",
                "-archivePath",
                archive_path,
                "-exportPath",
                export_path,
                "-exportOptionsPlist",
                &format!("{}/ExportOptions.plist", export_path),
            ])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::IosBuildError(format!(
                "Failed to export IPA: {}",
                error_msg
            )));
        }

        let ipa_path = format!("{}/{}.ipa", export_path, self.app_name);
        println!("✓ IPA exported: {}", ipa_path);
        Ok(ipa_path)
    }
}
