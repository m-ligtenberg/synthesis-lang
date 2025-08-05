#!/usr/bin/env rust-script

//! Synthesis Package Manager
//! 
//! Custom package manager that hides Rust implementation details
//! and provides a clean, language-specific build experience.

use std::env;
use std::process::{Command, exit};
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SynthesisManifest {
    package: PackageInfo,
    dependencies: Option<std::collections::HashMap<String, String>>,
    build: Option<BuildConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PackageInfo {
    name: String,
    version: String,
    description: Option<String>,
    author: Option<String>,
    license: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BuildConfig {
    target: Option<String>,
    optimization: Option<String>,
    features: Option<Vec<String>>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help();
        return;
    }
    
    match args[1].as_str() {
        "new" => create_new_project(&args[2..]),
        "build" => build_project(&args[2..]),
        "run" => run_project(&args[2..]),
        "install" => install_package(&args[2..]),
        "publish" => publish_package(&args[2..]),
        "clean" => clean_project(),
        "version" => print_version(),
        "help" | "-h" | "--help" => print_help(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_help();
            exit(1);
        }
    }
}

fn create_new_project(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: Project name required");
        eprintln!("Usage: syn-pkg new <project-name>");
        exit(1);
    }
    
    let project_name = &args[0];
    let project_dir = Path::new(project_name);
    
    if project_dir.exists() {
        eprintln!("Error: Directory '{}' already exists", project_name);
        exit(1);
    }
    
    println!("Creating new Synthesis project: {}", project_name);
    
    // Create project structure
    fs::create_dir_all(project_dir.join("src")).expect("Failed to create src directory");
    fs::create_dir_all(project_dir.join("examples")).expect("Failed to create examples directory");
    fs::create_dir_all(project_dir.join("assets")).expect("Failed to create assets directory");
    
    // Create package.syn manifest
    let manifest = SynthesisManifest {
        package: PackageInfo {
            name: project_name.clone(),
            version: "0.1.0".to_string(),
            description: Some(format!("A Synthesis language project")),
            author: None,
            license: Some("MIT".to_string()),
        },
        dependencies: None,
        build: Some(BuildConfig {
            target: Some("native".to_string()),
            optimization: Some("debug".to_string()),
            features: None,
        }),
    };
    
    let manifest_toml = toml::to_string_pretty(&manifest).expect("Failed to serialize manifest");
    fs::write(project_dir.join("package.syn"), manifest_toml).expect("Failed to write manifest");
    
    // Create main.syn
    let main_content = r#"// Main Synthesis program
import Audio.{mic_input, analyze_fft}
import Graphics.{clear, plasma}

loop {
    audio = Audio.mic_input()
    fft_data = Audio.analyze_fft(audio, 8)
    
    Graphics.clear(Graphics.black)
    Graphics.plasma({
        speed: fft_data[0],
        intensity: fft_data[1]
    })
}
"#;
    fs::write(project_dir.join("src").join("main.syn"), main_content).expect("Failed to write main.syn");
    
    // Create README
    let readme_content = format!(r#"# {}

A Synthesis language project for creative programming.

## Getting Started

```bash
# Build the project
syn-pkg build

# Run the project
syn-pkg run

# Clean build artifacts
syn-pkg clean
```

## Project Structure

- `src/` - Source code files
- `examples/` - Example programs
- `assets/` - Audio, image, and other assets
- `package.syn` - Project manifest

## Learn More

Visit [synthesis-lang.org](https://synthesis-lang.org) for documentation.
"#, project_name);
    
    fs::write(project_dir.join("README.md"), readme_content).expect("Failed to write README");
    
    println!("✓ Created project structure");
    println!("✓ Generated package.syn manifest");
    println!("✓ Created src/main.syn");
    println!("✓ Added README.md");
    println!();
    println!("Next steps:");
    println!("  cd {}", project_name);
    println!("  syn-pkg run");
}

fn build_project(_args: &[String]) {
    if !Path::new("package.syn").exists() {
        eprintln!("Error: No package.syn found. Are you in a Synthesis project directory?");
        exit(1);
    }
    
    println!("Building Synthesis project...");
    
    // Read manifest
    let manifest_content = fs::read_to_string("package.syn").expect("Failed to read package.syn");
    let manifest: SynthesisManifest = toml::from_str(&manifest_content).expect("Failed to parse package.syn");
    
    // Determine build mode
    let optimization = manifest.build
        .as_ref()
        .and_then(|b| b.optimization.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("debug");
    
    let target = manifest.build
        .as_ref()
        .and_then(|b| b.target.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("native");
    
    let release_flag = if optimization == "release" { vec!["--release"] } else { vec![] };
    
    // Run underlying Rust build (hidden from user)
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    cmd.args(&release_flag);
    
    // Hide Rust output and show clean Synthesis output
    let output = cmd.output().expect("Failed to execute build");
    
    if output.status.success() {
        println!("✓ Build completed successfully");
        println!("✓ Target: {}", target);
        println!("✓ Optimization: {}", optimization);
    } else {
        eprintln!("✗ Build failed");
        // Show clean error messages without Rust internals
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("error") {
            eprintln!("Synthesis compilation errors found:");
            // Filter and clean up error messages
            for line in stderr.lines() {
                if line.contains("error:") && !line.contains("rustc") {
                    eprintln!("  {}", line.trim());
                }
            }
        }
        exit(1);
    }
}

fn run_project(args: &[String]) {
    // Build first
    build_project(&[]);
    
    println!("Running Synthesis project...");
    
    let target_path = if Path::new("target/release").exists() {
        "target/release/synthesis"
    } else {
        "target/debug/synthesis"
    };
    
    let main_file = args.get(0)
        .map(|s| s.as_str())
        .unwrap_or("src/main.syn");
    
    let mut cmd = Command::new(target_path);
    cmd.arg(main_file);
    
    let status = cmd.status().expect("Failed to run project");
    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
}

fn install_package(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: Package name required");
        eprintln!("Usage: syn-pkg install <package-name>");
        exit(1);
    }
    
    let package_name = &args[0];
    println!("Installing package: {}", package_name);
    
    // This would connect to a package registry in the future
    println!("✓ Package registry integration coming soon!");
    println!("  For now, you can manually add dependencies to package.syn");
}

fn publish_package(_args: &[String]) {
    println!("Publishing package to Synthesis registry...");
    println!("✓ Package publishing coming soon!");
    println!("  Visit synthesis-lang.org/publish for early access");
}

fn clean_project() {
    println!("Cleaning build artifacts...");
    
    let status = Command::new("cargo")
        .arg("clean")
        .status()
        .expect("Failed to clean project");
    
    if status.success() {
        println!("✓ Clean completed");
    } else {
        eprintln!("✗ Clean failed");
        exit(1);
    }
}

fn print_version() {
    println!("syn-pkg 0.1.0");
    println!("Synthesis Package Manager");
    println!("Built for creative programming");
}

fn print_help() {
    println!("syn-pkg - Synthesis Package Manager");
    println!();
    println!("USAGE:");
    println!("    syn-pkg <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    new <name>       Create a new Synthesis project");
    println!("    build            Build the current project");
    println!("    run [file]       Build and run the project");
    println!("    install <pkg>    Install a package dependency");
    println!("    publish          Publish package to registry");
    println!("    clean            Remove build artifacts");
    println!("    version          Show version information");
    println!("    help             Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    syn-pkg new my-visualizer    # Create new project");
    println!("    syn-pkg build                # Build current project");
    println!("    syn-pkg run                  # Run main.syn");
    println!("    syn-pkg run examples/demo.syn   # Run specific file");
    println!();
    println!("For more information, visit: https://synthesis-lang.org");
}