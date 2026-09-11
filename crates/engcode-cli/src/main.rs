mod error_display;

use clap::{Parser, Subcommand};
use colored::*;
use engcode_lexer::Lexer;
use engcode_parser::Parser as EngParser;
use engcode_analyzer::SemanticAnalyzer;
use engcode_runtime::Interpreter;
use error_display::ErrorDisplay;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "engcode")]
#[command(about = "EnglishCode - Programming in Plain English", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an .eng file
    Run {
        /// Path to the .eng file
        file: PathBuf,
    },
    /// Show version information
    Version,
    /// Build mobile app (iOS/Android)
    Mobile {
        #[command(subcommand)]
        action: MobileCommands,
    },
}

#[derive(Subcommand)]
enum MobileCommands {
    /// Initialize new React Native project
    Init {
        /// Project name
        name: String,
        /// Output directory (default: ./mobile)
        #[arg(short, long, default_value = "./mobile")]
        output: String,
    },
    /// Build mobile app from .eng file
    Build {
        /// Source .eng file
        file: PathBuf,
        /// Platform (ios/android/both)
        #[arg(short, long, default_value = "both")]
        platform: String,
        /// Output directory
        #[arg(short, long, default_value = "./mobile")]
        output: String,
    },
    /// Run app on iOS simulator
    RunIos {
        /// Project directory
        #[arg(short, long, default_value = "./mobile")]
        dir: String,
    },
    /// Run app on Android emulator
    RunAndroid {
        /// Project directory
        #[arg(short, long, default_value = "./mobile")]
        dir: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            if let Err(e) = run_file(&file) {
                eprintln!("{} {}", "✗".red().bold(), e.red());
                std::process::exit(1);
            }
            println!("\n{}", "Program completed successfully!".green().bold());
        }
        Commands::Version => {
            println!("{}", "engcode v0.1.0".cyan().bold());
            println!("{}", "EnglishCode - Programming in Plain English".cyan());
            println!("\n{}", "macOS Edition with Mobile Support".bright_black());
        }
        Commands::Mobile { action } => {
            if let Err(e) = handle_mobile_command(action) {
                eprintln!("{} {}", "✗".red().bold(), e.red());
                std::process::exit(1);
            }
        }
    }
}

fn run_file(path: &PathBuf) -> Result<(), String> {
    // Check if file exists
    if !path.exists() {
        return Err(format!("File not found: {}", path.display().to_string().yellow()));
    }

    // Check if file has .eng extension
    if path.extension().and_then(|s| s.to_str()) != Some("eng") {
        return Err(format!("File must have {} extension", ".eng".yellow()));
    }

    println!("{} {}", "→".bright_black(), format!("Running {}", path.display()).bright_black());
    println!();

    // Read file contents
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let error_display = ErrorDisplay::new(
        contents.clone(),
        path.display().to_string(),
    );

    // Tokenize
    let mut lexer = Lexer::new(contents.clone());
    let tokens = lexer.tokenize();

    // Parse
    let mut parser = EngParser::new(tokens);
    let program = parser.parse().map_err(|e| {
        error_display.display_parse_error(&e, None);
        e
    })?;

    // Semantic Analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program).map_err(|e| {
        error_display.display_semantic_error(&e.to_string(), None);
        e.to_string()
    })?;

    // Execute
    let mut interpreter = Interpreter::new();
    interpreter.execute(program).map_err(|e| {
        error_display.display_runtime_error(&e.to_string(), "");
        e.to_string()
    })?;

    Ok(())
}

fn handle_mobile_command(action: MobileCommands) -> Result<(), String> {
    use engcode_mobile::transpiler::MobileTranspiler;
    use engcode_mobile::react_native::ReactNativeProject;

    match action {
        MobileCommands::Init { name, output } => {
            println!("{}", "🚀 Initializing React Native project...".cyan().bold());
            let project = ReactNativeProject::new(name.clone(), output);
            project.init().map_err(|e| e.to_string())?;
            println!("\n{}", format!("✓ Project '{}' created successfully!", name).green().bold());
            println!("\n{}", "Next steps:".yellow().bold());
            println!("  1. {} to build your app from .eng file", "engcode mobile build app.eng".cyan());
            println!("  2. {} to run on iOS", "engcode mobile run-ios".cyan());
            println!("  3. {} to run on Android", "engcode mobile run-android".cyan());
            Ok(())
        }
        MobileCommands::Build { file, platform, output } => {
            println!("{}", "📱 Building mobile app...".cyan().bold());

            // Read and parse .eng file
            if !file.exists() {
                return Err(format!("File not found: {}", file.display()));
            }

            let source = fs::read_to_string(&file).map_err(|e| e.to_string())?;

            println!("{}", "  → Parsing EnglishCode...".bright_black());
            let mut lexer = Lexer::new(source);
            let tokens = lexer.tokenize();

            let mut parser = EngParser::new(tokens);
            let program = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

            println!("{}", "  → Transpiling to React Native...".bright_black());
            let mut transpiler = MobileTranspiler::new();
            let react_code = transpiler.transpile_to_react_native(&program.statements)
                .map_err(|e| e.to_string())?;

            // Create/update React Native project
            let project = ReactNativeProject::new("EnglishCodeApp".to_string(), output.clone());

            // Write transpiled code
            project.write_app_code(&react_code).map_err(|e| e.to_string())?;

            // Install React Navigation dependencies so the app actually runs
            println!("{}", "  → Installing navigation dependencies...".bright_black());
            project.add_dependencies(&[
                "@react-navigation/native",
                "@react-navigation/native-stack",
                "react-native-screens",
                "react-native-safe-area-context",
            ]).map_err(|e| e.to_string())?;

            println!("{}", format!("✓ Mobile app built in {}", output).green().bold());

            if platform == "ios" || platform == "both" {
                println!("\n{}", "iOS:".yellow().bold());
                println!("  Run: {}", "engcode mobile run-ios".cyan());
            }
            if platform == "android" || platform == "both" {
                println!("\n{}", "Android:".yellow().bold());
                println!("  Run: {}", "engcode mobile run-android".cyan());
            }

            Ok(())
        }
        MobileCommands::RunIos { dir } => {
            println!("{}", "🍎 Running on iOS simulator...".cyan().bold());
            let project = ReactNativeProject::new("EnglishCodeApp".to_string(), dir);
            project.run_ios().map_err(|e| e.to_string())?;
            println!("\n{}", "✓ iOS app running!".green().bold());
            Ok(())
        }
        MobileCommands::RunAndroid { dir } => {
            println!("{}", "🤖 Running on Android emulator...".cyan().bold());
            let project = ReactNativeProject::new("EnglishCodeApp".to_string(), dir);
            project.run_android().map_err(|e| e.to_string())?;
            println!("\n{}", "✓ Android app running!".green().bold());
            Ok(())
        }
    }
}
