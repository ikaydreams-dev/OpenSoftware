use crate::{MobileError, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct ReactNativeProject {
    project_name: String,
    output_dir: String,
}

impl ReactNativeProject {
    pub fn new(project_name: String, output_dir: String) -> Self {
        Self {
            project_name,
            output_dir,
        }
    }

    pub fn init(&self) -> Result<()> {
        println!("Initializing React Native project: {}", self.project_name);

        // Check if npx is available
        let npx_check = Command::new("npx")
            .arg("--version")
            .output();

        if npx_check.is_err() {
            return Err(MobileError::ReactNativeError(
                "npx not found. Please install Node.js and npm".to_string()
            ));
        }

        // Create React Native app
        let output = Command::new("npx")
            .args(&[
                "react-native",
                "init",
                &self.project_name,
                "--directory",
                &self.output_dir,
            ])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::ReactNativeError(format!(
                "Failed to initialize React Native project: {}",
                error_msg
            )));
        }

        println!("✓ React Native project created");
        Ok(())
    }

    pub fn write_app_code(&self, code: &str) -> Result<()> {
        let app_path = Path::new(&self.output_dir).join("App.tsx");
        fs::write(app_path, code)?;
        println!("✓ App code written to App.tsx");
        Ok(())
    }

    pub fn add_dependencies(&self, deps: &[&str]) -> Result<()> {
        println!("Installing dependencies...");
        let output = Command::new("npm")
            .args(&["install", "--prefix", &self.output_dir])
            .args(deps)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::ReactNativeError(format!(
                "Failed to install dependencies: {}",
                error_msg
            )));
        }

        println!("✓ Dependencies installed");
        Ok(())
    }

    pub fn run_ios(&self) -> Result<()> {
        println!("Running iOS simulator...");
        let output = Command::new("npx")
            .args(&["react-native", "run-ios"])
            .current_dir(&self.output_dir)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::IosBuildError(format!(
                "Failed to run iOS: {}",
                error_msg
            )));
        }

        println!("✓ iOS app running");
        Ok(())
    }

    pub fn run_android(&self) -> Result<()> {
        println!("Running Android emulator...");
        let output = Command::new("npx")
            .args(&["react-native", "run-android"])
            .current_dir(&self.output_dir)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::AndroidBuildError(format!(
                "Failed to run Android: {}",
                error_msg
            )));
        }

        println!("✓ Android app running");
        Ok(())
    }

    pub fn build_ios_release(&self) -> Result<String> {
        println!("Building iOS release...");
        let output = Command::new("xcodebuild")
            .args(&[
                "-workspace",
                &format!("{}/ios/{}.xcworkspace", self.output_dir, self.project_name),
                "-scheme",
                &self.project_name,
                "-configuration",
                "Release",
                "-archivePath",
                &format!("{}/build/{}.xcarchive", self.output_dir, self.project_name),
                "archive",
            ])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::IosBuildError(format!(
                "Failed to build iOS: {}",
                error_msg
            )));
        }

        let archive_path = format!("{}/build/{}.xcarchive", self.output_dir, self.project_name);
        println!("✓ iOS release built: {}", archive_path);
        Ok(archive_path)
    }

    pub fn build_android_release(&self) -> Result<String> {
        println!("Building Android release APK...");
        let output = Command::new("./gradlew")
            .args(&["assembleRelease"])
            .current_dir(format!("{}/android", self.output_dir))
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::AndroidBuildError(format!(
                "Failed to build Android: {}",
                error_msg
            )));
        }

        let apk_path = format!(
            "{}/android/app/build/outputs/apk/release/app-release.apk",
            self.output_dir
        );
        println!("✓ Android APK built: {}", apk_path);
        Ok(apk_path)
    }
}
