use crate::{MobileError, Result};
use std::process::Command;
use std::env;

pub struct AndroidBuilder {
    app_name: String,
    package_name: String,
}

impl AndroidBuilder {
    pub fn new(app_name: String, package_name: String) -> Self {
        Self { app_name, package_name }
    }

    pub fn check_android_sdk(&self) -> Result<()> {
        if env::var("ANDROID_HOME").is_err() && env::var("ANDROID_SDK_ROOT").is_err() {
            return Err(MobileError::AndroidBuildError(
                "Android SDK not found. Set ANDROID_HOME or ANDROID_SDK_ROOT environment variable".to_string()
            ));
        }

        println!("✓ Android SDK found");
        Ok(())
    }

    pub fn check_java(&self) -> Result<()> {
        let output = Command::new("java")
            .arg("-version")
            .output();

        if output.is_err() {
            return Err(MobileError::AndroidBuildError(
                "Java not found. Please install JDK 11 or higher".to_string()
            ));
        }

        println!("✓ Java found");
        Ok(())
    }

    pub fn list_emulators(&self) -> Result<Vec<String>> {
        let android_home = env::var("ANDROID_HOME")
            .or_else(|_| env::var("ANDROID_SDK_ROOT"))
            .map_err(|_| MobileError::AndroidBuildError("ANDROID_HOME not set".to_string()))?;

        let output = Command::new(format!("{}/emulator/emulator", android_home))
            .args(&["-list-avds"])
            .output()?;

        if !output.status.success() {
            return Err(MobileError::AndroidBuildError(
                "Failed to list emulators".to_string()
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let emulators: Vec<String> = output_str
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(emulators)
    }

    pub fn start_emulator(&self, emulator_name: &str) -> Result<()> {
        println!("Starting Android emulator: {}", emulator_name);

        let android_home = env::var("ANDROID_HOME")
            .or_else(|_| env::var("ANDROID_SDK_ROOT"))
            .map_err(|_| MobileError::AndroidBuildError("ANDROID_HOME not set".to_string()))?;

        Command::new(format!("{}/emulator/emulator", android_home))
            .args(&["-avd", emulator_name])
            .spawn()
            .map_err(|e| MobileError::AndroidBuildError(format!("Failed to start emulator: {}", e)))?;

        println!("✓ Emulator starting...");
        Ok(())
    }

    pub fn wait_for_device(&self) -> Result<()> {
        println!("Waiting for device...");
        let output = Command::new("adb")
            .args(&["wait-for-device"])
            .output()?;

        if !output.status.success() {
            return Err(MobileError::AndroidBuildError(
                "Failed to wait for device".to_string()
            ));
        }

        println!("✓ Device ready");
        Ok(())
    }

    pub fn build_debug(&self, project_dir: &str) -> Result<String> {
        println!("Building Android debug APK...");
        let output = Command::new("./gradlew")
            .args(&["assembleDebug"])
            .current_dir(format!("{}/android", project_dir))
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::AndroidBuildError(format!(
                "Failed to build: {}",
                error_msg
            )));
        }

        let apk_path = format!(
            "{}/android/app/build/outputs/apk/debug/app-debug.apk",
            project_dir
        );
        println!("✓ Debug APK built: {}", apk_path);
        Ok(apk_path)
    }

    pub fn build_release(&self, project_dir: &str) -> Result<String> {
        println!("Building Android release APK...");
        let output = Command::new("./gradlew")
            .args(&["assembleRelease"])
            .current_dir(format!("{}/android", project_dir))
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::AndroidBuildError(format!(
                "Failed to build release: {}",
                error_msg
            )));
        }

        let apk_path = format!(
            "{}/android/app/build/outputs/apk/release/app-release.apk",
            project_dir
        );
        println!("✓ Release APK built: {}", apk_path);
        Ok(apk_path)
    }

    pub fn build_bundle(&self, project_dir: &str) -> Result<String> {
        println!("Building Android App Bundle (AAB)...");
        let output = Command::new("./gradlew")
            .args(&["bundleRelease"])
            .current_dir(format!("{}/android", project_dir))
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::AndroidBuildError(format!(
                "Failed to build bundle: {}",
                error_msg
            )));
        }

        let aab_path = format!(
            "{}/android/app/build/outputs/bundle/release/app-release.aab",
            project_dir
        );
        println!("✓ AAB built: {}", aab_path);
        Ok(aab_path)
    }

    pub fn install_apk(&self, apk_path: &str) -> Result<()> {
        println!("Installing APK on device...");
        let output = Command::new("adb")
            .args(&["install", "-r", apk_path])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::AndroidBuildError(format!(
                "Failed to install APK: {}",
                error_msg
            )));
        }

        println!("✓ APK installed");
        Ok(())
    }

    pub fn run_app(&self) -> Result<()> {
        println!("Launching app...");
        let output = Command::new("adb")
            .args(&[
                "shell",
                "am",
                "start",
                "-n",
                &format!("{}/MainActivity", self.package_name),
            ])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(MobileError::AndroidBuildError(format!(
                "Failed to launch app: {}",
                error_msg
            )));
        }

        println!("✓ App launched");
        Ok(())
    }
}
