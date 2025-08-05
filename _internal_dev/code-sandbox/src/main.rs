use std::process::{Command, Stdio};
use std::io::{Write, Read};
use std::time::{Instant, Duration};
use std::sync::mpsc;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::env;

use serde::{Deserialize, Serialize};
use warp::{Filter, Reply, Rejection};
use anyhow::{Result, Context};
use tracing::{info, error};

#[derive(Debug, Serialize, Deserialize)]
struct ExecutionRequest {
    code: String,
    module: String,
    example: String,
}

#[derive(Debug, Serialize)]
struct ExecutionResponse {
    output: String,
    error: Option<String>,
    execution_time: u128,
    memory_used: usize,
    visualization: Option<String>,
}

fn limit_code_complexity(code: &str) -> Result<()> {
    // Basic complexity checks
    anyhow::ensure!(code.len() <= 10000, "Code too long");
    anyhow::ensure!(code.lines().count() <= 500, "Too many lines");
    
    // Prevent potentially dangerous operations
    let dangerous_patterns = [
        "unsafe", 
        "std::fs", 
        "std::env", 
        "std::net", 
        "std::process",
        "system(",
        "exec(",
        "shell(",
    ];
    
    for pattern in &dangerous_patterns {
        anyhow::ensure!(!code.contains(pattern), "Potentially unsafe code detected");
    }
    
    Ok(())
}

fn execute_synthesis_code(request: ExecutionRequest) -> Result<ExecutionResponse> {
    // Validate code complexity and safety
    limit_code_complexity(&request.code)?;
    
    // Create a temporary file for code execution
    let temp_file = tempfile::NamedTempFile::new()?;
    let file_path = temp_file.path().to_str().context("Invalid file path")?;
    
    // Write code to temporary file
    std::fs::write(file_path, &request.code)?;
    
    // Set up timeout and output capture
    let (tx, rx) = mpsc::channel();
    
    let start_time = Instant::now();
    
    let handler = thread::spawn(move || {
        let output = Command::new("synthesis")
            .arg("run")
            .arg(file_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute code");
        
        tx.send(output).expect("Could not send output");
    });
    
    // Wait with timeout
    let result = match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(output) => {
            let execution_time = start_time.elapsed().as_millis();
            
            // Process output
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            
            // Determine response based on exit status
            if output.status.success() {
                Ok(ExecutionResponse {
                    output: stdout,
                    error: if !stderr.is_empty() { Some(stderr) } else { None },
                    execution_time,
                    memory_used: get_memory_usage(),
                    visualization: extract_visualization(&stdout),
                })
            } else {
                Err(anyhow::anyhow!(
                    "Execution failed: {}{}",
                    stdout,
                    stderr
                ))
            }
        },
        Err(_) => {
            // Timeout occurred
            Err(anyhow::anyhow!("Code execution timed out"))
        }
    };
    
    // Ensure thread is joined
    handler.join().expect("Thread panicked");
    
    result
}

fn get_memory_usage() -> usize {
    // Placeholder for actual memory usage tracking
    // In a real implementation, use system-specific memory tracking
    42 // KB
}

fn extract_visualization(output: &str) -> Option<String> {
    // Extract visualization URL or data from output
    // This is a placeholder - actual implementation would parse output
    if output.contains("VISUALIZATION:") {
        Some(output.split("VISUALIZATION:").nth(1)?.trim().to_string())
    } else {
        None
    }
}

async fn handle_execution(request: ExecutionRequest) -> Result<impl Reply, Rejection> {
    match execute_synthesis_code(request) {
        Ok(response) => Ok(warp::reply::json(&response)),
        Err(e) => Ok(warp::reply::json(&ExecutionResponse {
            output: String::new(),
            error: Some(e.to_string()),
            execution_time: 0,
            memory_used: 0,
            visualization: None,
        }))
    }
}

// Global readiness and liveness state
static IS_READY: AtomicBool = AtomicBool::new(false);
static IS_HEALTHY: AtomicBool = AtomicBool::new(true);

// Configuration from environment
fn get_config() -> SandboxConfig {
    SandboxConfig {
        max_execution_time: env::var("MAX_EXECUTION_TIME")
            .map(|v| v.parse().unwrap_or(5))
            .unwrap_or(5),
        max_memory: env::var("MAX_MEMORY")
            .map(|v| v.parse().unwrap_or(512))
            .unwrap_or(512),
    }
}

#[derive(Clone)]
struct SandboxConfig {
    max_execution_time: u64,
    max_memory: usize,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load sandbox configuration
    let config = get_config();
    
    // CORS and JSON parsing
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST", "GET"])
        .allow_headers(vec!["Content-Type"]);
    
    // Health and readiness check routes
    let health_route = warp::get()
        .and(warp::path("health"))
        .map(|| {
            if IS_HEALTHY.load(Ordering::Relaxed) {
                warp::reply::with_status("OK", warp::http::StatusCode::OK)
            } else {
                warp::reply::with_status("UNHEALTHY", warp::http::StatusCode::SERVICE_UNAVAILABLE)
            }
        });
    
    let ready_route = warp::get()
        .and(warp::path("ready"))
        .map(|| {
            if IS_READY.load(Ordering::Relaxed) {
                warp::reply::with_status("READY", warp::http::StatusCode::OK)
            } else {
                warp::reply::with_status("NOT_READY", warp::http::StatusCode::SERVICE_UNAVAILABLE)
            }
        });
    
    let execute_route = warp::post()
        .and(warp::path("execute"))
        .and(warp::body::json())
        .and_then(handle_execution)
        .with(cors);
    
    // Combine routes
    let routes = health_route
        .or(ready_route)
        .or(execute_route);
    
    // Mark sandbox as ready
    IS_READY.store(true, Ordering::Relaxed);
    
    info!("Synthesis Sandbox Runtime started on port 8080");
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], 8080))
        .await
}

// Security and performance test harness
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_code_complexity_limits() {
        let long_code = "x".repeat(20000);
        assert!(limit_code_complexity(&long_code).is_err());
        
        let unsafe_code = "import std::fs;";
        assert!(limit_code_complexity(unsafe_code).is_err());
    }
    
    #[test]
    fn test_execution_timeout() {
        let infinite_loop = r#"
        loop {
            // Intentional infinite loop
        }
        "#.to_string();
        
        let result = execute_synthesis_code(ExecutionRequest {
            code: infinite_loop,
            module: "test".to_string(),
            example: "timeout".to_string(),
        });
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timeout"));
    }
}