use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose};
use clap::{Arg, Command as ClapCommand};
use octocrab::Octocrab;
use sha2::{Digest, Sha256};
use std::env;
use std::path::Path;
use std::process::Command;
use tokio::fs;

#[derive(Debug, Clone)]
struct GitHubConfig {
    token: String,
    owner: String,
    repo: String,
}

impl GitHubConfig {
    fn from_env() -> Result<Self> {
        let token =
            env::var("GITHUB_TOKEN").context("GITHUB_TOKEN environment variable not set")?;
        let owner = env::var("GITHUB_OWNER").unwrap_or_else(|_| "HelixDB".to_string());
        let repo = env::var("GITHUB_REPO").unwrap_or_else(|_| "helix-db".to_string());

        Ok(GitHubConfig { token, owner, repo })
    }
}

fn generate_error_hash(error_type: &str, error_message: &str, _file_num: u32) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!(
        "{}:{}",
        error_type,
        error_message.lines().take(5).collect::<Vec<_>>().join("\n")
    ));
    let hash = hasher.finalize();
    general_purpose::STANDARD.encode(hash)[0..12].to_string()
}

fn extract_cargo_errors(stderr: &str, stdout: &str) -> String {
    let mut errors = Vec::new();
    
    // Parse both stderr and stdout for error messages
    let combined_output = format!("{}\n{}", stderr, stdout);
    
    let lines: Vec<&str> = combined_output.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        
        // Look for error lines
        if line.starts_with("error:") || line.contains("error:") {
            // Start collecting this error
            let mut error_block = Vec::new();
            error_block.push(line);
            
            // Continue collecting related lines until we hit a blank line or another error/warning
            i += 1;
            while i < lines.len() {
                let next_line = lines[i];
                
                // Stop if we hit another error or warning
                if next_line.starts_with("error:") || next_line.starts_with("warning:") || next_line.starts_with("note:") {
                    break;
                }
                
                // Stop if we hit compilation result lines
                if next_line.contains("error: could not compile") || 
                   next_line.contains("error: aborting due to") ||
                   next_line.contains("For more information about this error") {
                    break;
                }
                
                // Include the line if it's not empty or if it's providing error context
                if !next_line.trim().is_empty() || 
                   next_line.starts_with("  ") || 
                   next_line.starts_with("   ") ||
                   next_line.contains("-->") ||
                   next_line.contains("|") {
                    error_block.push(next_line);
                }
                
                i += 1;
            }
            
            // Add this error block to our collection
            if !error_block.is_empty() {
                errors.push(error_block.join("\n"));
            }
        } else {
            i += 1;
        }
    }
    
    // If no structured errors found, look for any line containing "error:"
    if errors.is_empty() {
        for line in lines {
            if line.contains("error:") && 
               !line.contains("error: could not compile") &&
               !line.contains("error: aborting due to") {
                errors.push(line.to_string());
            }
        }
    }
    
    // Join all errors with double newlines
    if errors.is_empty() {
        "Unknown compilation error".to_string()
    } else {
        errors.join("\n\n")
    }
}

async fn check_issue_exists(github_config: &GitHubConfig, error_hash: &str) -> Result<bool> {
    println!("DEBUG: Checking if issue exists with hash: {}", error_hash);

    let octocrab = Octocrab::builder()
        .personal_token(github_config.token.clone())
        .build()?;

    let search_query = format!(
        "repo:{}/{} is:issue ERROR_HASH:{}",
        github_config.owner, github_config.repo, error_hash
    );

    println!("DEBUG: GitHub search query: {}", search_query);

    let issues = octocrab
        .search()
        .issues_and_pull_requests(&search_query)
        .send()
        .await?;

    let count = issues.total_count.unwrap_or(0);
    println!("DEBUG: Found {} existing issues", count);

    Ok(count > 0)
}

async fn create_github_issue(
    github_config: &GitHubConfig,
    error_type: &str,
    error_message: &str,
    file_num: u32,
    error_hash: &str,
    query: &str,
    schema: &str,
    generated_rust_code: &str,
) -> Result<()> {
    println!(
        "DEBUG: Creating GitHub issue for {}/{}",
        github_config.owner, github_config.repo
    );

    let octocrab = Octocrab::builder()
        .personal_token(github_config.token.clone())
        .build()?;

    let title = format!("Auto-generated: {} Error in file{}", error_type, file_num);

    let body = format!(
        "## Automatic Error Report\n\n\
        **Error Type:** {}\n\
        **File:** file{}\n\
        **Error Hash:** ERROR_HASH:{}\n\n\
        ### Query\n\
        ```js\n{}\n```\n\n\
        ### Schema\n\
        ```js\n{}\n```\n\n\
        ### Generated Rust Code\n\
        ```rust\n{}\n```\n\n\
        ### Error Details\n\
        ```\n{}\n```\n\n\
        ---\n\
        *This issue was automatically generated by the hql-tests runner.*",
        error_type, file_num, error_hash, query, schema, generated_rust_code, error_message
    );

    let labels = vec![
        "bug".to_string(),
        "automated".to_string(),
        "hql-tests".to_string(),
    ];

    println!("DEBUG: Issue title: {}", title);
    println!("DEBUG: Issue body length: {} chars", body.len());
    println!("DEBUG: Issue labels: {:?}", labels);

    let issue = octocrab
        .issues(&github_config.owner, &github_config.repo)
        .create(&title)
        .body(&body)
        .labels(Some(labels))
        .send()
        .await?;

    println!(
        "Created GitHub issue #{} for {} error in file{}",
        issue.number, error_type, file_num
    );
    Ok(())
}

async fn handle_error_with_github(
    github_config: &GitHubConfig,
    error_type: &str,
    error_message: &str,
    file_num: u32,
    query: &str,
    schema: &str,
    generated_rust_code: &str,
) -> Result<()> {
    let error_hash = generate_error_hash(error_type, error_message, file_num);

    println!(
        "DEBUG: Handling error with GitHub - Type: {}, File: {}, Hash: {}",
        error_type, file_num, error_hash
    );

    match check_issue_exists(github_config, &error_hash).await {
        Ok(exists) => {
            println!("DEBUG: Issue exists check result: {}", exists);
            if !exists {
                println!("DEBUG: Creating new GitHub issue...");
                if let Err(e) = create_github_issue(
                    github_config,
                    error_type,
                    error_message,
                    file_num,
                    &error_hash,
                    query,
                    schema,
                    generated_rust_code,
                )
                .await
                {
                    eprintln!("Failed to create GitHub issue: {}", e);
                }
            } else {
                println!(
                    "Issue already exists for {} error in file{} (hash: {})",
                    error_type, file_num, error_hash
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to check existing issues: {}", e);
            // Try to create the issue anyway if we can't check for duplicates
            println!("DEBUG: Attempting to create issue despite check failure...");
            if let Err(e) = create_github_issue(
                github_config,
                error_type,
                error_message,
                file_num,
                &error_hash,
                query,
                schema,
                generated_rust_code,
            )
            .await
            {
                eprintln!("Failed to create GitHub issue: {}", e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = ClapCommand::new("queries-test")
        .about("Process helix files")
        .arg(
            Arg::new("file_number")
                .help("File number to process (1-100)")
                .value_parser(clap::value_parser!(u32))
                .required(false),
        )
        .arg(
            Arg::new("batch")
                .long("batch")
                .help("Enable batch processing with total batches and current batch")
                .num_args(2)
                .value_names(&["TOTAL_BATCHES", "CURRENT_BATCH"])
                .value_parser(clap::value_parser!(u32))
                .required(false),
        )
        .arg(
            Arg::new("branch")
                .long("branch")
                .help("Branch to process")
                .value_parser(clap::value_parser!(String))
                .required(false),
        )
        .get_matches();

    let current_dir = env::current_dir().context("Failed to get current directory")?;

    // Initialize GitHub configuration (optional - will print warning if not available)
    let github_config = match GitHubConfig::from_env() {
        Ok(config) => {
            println!(
                "GitHub integration enabled for {}/{}",
                config.owner, config.repo
            );
            Some(config)
        }
        Err(e) => {
            println!("GitHub integration disabled: {}", e);
            println!("Set GITHUB_TOKEN environment variable to enable automatic issue creation");
            None
        }
    };

    // pull repo to copy to all folders
    let temp_repo = env::temp_dir().join("temp_repo");
    if !temp_repo.exists() {
        fs::create_dir_all(&temp_repo)
            .await
            .context("Failed to create temp directory")?;
    }

    let branch = matches.get_one::<String>("branch").map(|s| s.as_str()).unwrap_or("dev");
    println!("DEBUG: Branch: {}", branch);
    // Run helix init command
    let output = Command::new("helix")
        .arg("install")
        .arg("-p")
        .arg(&temp_repo)
        .arg("--branch")
        .arg(branch)
        .output()
        .context("Failed to execute helix init command")?;

    if !output.status.success() {
        fs::remove_dir_all(&temp_repo).await.ok();
        bail!(
            "Error: Helix init failed for {}\nStderr: {}\nStdout: {}",
            temp_repo.display(),
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
    }

    if let Some(file_num) = matches.get_one::<u32>("file_number") {
        // Process single file
        if *file_num < 1 || *file_num > 100 {
            bail!("Error: Please provide a number between 1 and 100");
        }

        process_file_parallel(*file_num, &current_dir, &temp_repo, &github_config).await?;
    } else if let Some(batch_args) = matches.get_many::<u32>("batch") {
        // Process in batch mode
        let batch_values: Vec<u32> = batch_args.copied().collect();
        if batch_values.len() != 2 {
            bail!("Error: --batch requires exactly 2 arguments: total_batches current_batch");
        }
        
        let total_batches = batch_values[0];
        let current_batch = batch_values[1];
        
        if current_batch < 1 || current_batch > total_batches {
            bail!("Error: Current batch ({}) must be between 1 and {}", current_batch, total_batches);
        }
        
        if total_batches == 0 {
            bail!("Error: Total batches must be greater than 0");
        }

        // Calculate which files this batch should process
        let files_per_batch = 100 / total_batches;
        let remainder = 100 % total_batches;
        
        // Calculate start and end for this batch
        let start_file = ((current_batch - 1) * files_per_batch) + 1;
        let mut end_file = current_batch * files_per_batch;
        
        // Add remainder files to the last batch
        if current_batch == total_batches {
            end_file += remainder;
        }
        
        println!("Processing batch {}/{}: files {}-{}", current_batch, total_batches, start_file, end_file);

        let tasks: Vec<_> = (start_file..=end_file)
            .map(|file_num| {
                let current_dir = current_dir.clone();
                let temp_repo = temp_repo.clone();
                let github_config = github_config.clone();
                tokio::spawn(async move {
                    match process_file_parallel(file_num, &current_dir, &temp_repo, &github_config)
                        .await
                    {
                        Ok(()) => {
                            println!("Successfully processed file{}", file_num);
                        }
                        Err(e) => {
                            eprintln!("Error processing file{}: {}", file_num, e);
                        }
                    }
                })
            })
            .collect();

        // Wait for all tasks to complete
        for task in tasks {
            let _ = task.await;
        }

        println!("Finished processing batch {}/{}", current_batch, total_batches);
    } else {
        // Process all files in parallel (default behavior)
        println!("Processing all files 1-100 in parallel...");

        let tasks: Vec<_> = (1..=100)
            .map(|file_num| {
                let current_dir = current_dir.clone();
                let temp_repo = temp_repo.clone();
                let github_config = github_config.clone();
                tokio::spawn(async move {
                    match process_file_parallel(file_num, &current_dir, &temp_repo, &github_config)
                        .await
                    {
                        Ok(()) => {
                            println!("Successfully processed file{}", file_num);
                        }
                        Err(e) => {
                            eprintln!("Error processing file{}: {}", file_num, e);
                        }
                    }
                })
            })
            .collect();

        // Wait for all tasks to complete
        for task in tasks {
            let _ = task.await;
        }

        println!("Finished processing all files");
    }

    Ok(())
}

async fn process_file_parallel(
    file_num: u32,
    current_dir: &Path,
    temp_repo: &Path,
    github_config: &Option<GitHubConfig>,
) -> Result<()> {
    let folder = format!("file{}", file_num);
    let folder_path = current_dir.join(&folder);

    if !folder_path.exists() {
        // Skip non-existent files silently in parallel mode
        return Ok(());
    }

    // if queries.hx is empty, skip
    let queries_hx_path = folder_path.join(format!("{}.hx", folder));
    let schema_hx_path = folder_path.join("schema.hx");
    if queries_hx_path.exists() && queries_hx_path.is_file() {
        let content = fs::read_to_string(&queries_hx_path).await?;
        if content.is_empty() {
            return Ok(());
        }
    }

    // Create a temporary directory for this file
    let temp_dir = env::temp_dir().join(format!("helix_temp_{}", file_num));
    // let temp_dir = current_dir.join(format!("helix_temp_{}", file_num));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)
            .await
            .context("Failed to remove existing temp directory")?;
    }
    fs::create_dir_all(&temp_dir)
        .await
        .context("Failed to create temp directory")?;

    // Copy the file contents to temp directory
    copy_dir_recursive(&folder_path, &temp_dir).await?;
    // copy repo to folder
    copy_dir_recursive(&temp_repo, &temp_dir).await?;

    // Run helix compile command
    let compile_output_path = temp_dir.join(".helix/repo/helix-db/helix-container/src");
    fs::create_dir_all(&compile_output_path)
        .await
        .context("Failed to create compile output directory")?;

    let output = Command::new("helix")
        .arg("compile")
        .arg("--path")
        .arg(&temp_dir)
        .arg("--output")
        .arg(&compile_output_path)
        .output()
        .context("Failed to execute helix compile command")?;

    if !output.status.success() {
        fs::remove_dir_all(&temp_dir).await.ok();
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        // For helix compilation, we'll show the raw output since it's not cargo format
        let error_message = format!(
            "Helix compile failed for file{}\nStderr: {}\nStdout: {}",
            file_num,
            stderr,
            stdout
        );

        // Create GitHub issue if configuration is available
        if let Some(config) = github_config {
            println!("DEBUG: Helix compilation failed in parellel mode, creating GitHub issue...");
            let query_content = fs::read_to_string(&queries_hx_path).await.map_err(|e| {
                println!("DEBUG: Failed to read queries.hx: {}", e);
                e
            })?;
            let schema_content = fs::read_to_string(&schema_hx_path).await.map_err(|e| {
                println!("DEBUG: Failed to read schema.hx: {}", e);
                e
            })?;
            let generated_rust_code =
                fs::read_to_string(&compile_output_path.join("queries.rs")).await.map_err(|e| {
                    println!("DEBUG: Failed to read queries.rs: {}", e);
                    e
                })?;
            handle_error_with_github(
                config,
                "Helix Compilation",
                &error_message,
                file_num,
                &query_content,
                &schema_content,
                &generated_rust_code,
            )
            .await?;
        } else {
            println!("DEBUG: GitHub integration not configured, skipping issue creation");
        }

        bail!("Error: {}", error_message);
    }

    // Run cargo check on the helix container path
    let helix_container_path = temp_dir.join(".helix/repo/helix-db/helix-container");
    if helix_container_path.exists() {
        let output = Command::new("cargo")
            .arg("check")
            .current_dir(&helix_container_path)
            .output()
            .context("Failed to execute cargo check")?;

        if !output.status.success() {
            
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            // let filtered_errors = extract_cargo_errors(&stderr, &stdout);
            let error_message = format!(
                "Cargo check failed for file{}\n{}",
                file_num,
                stderr
            );

            // Create GitHub issue if configuration is available
            if let Some(config) = github_config {
                println!("DEBUG: Cargo check failed in parallel mode, creating GitHub issue...");
                let query_content = fs::read_to_string(&queries_hx_path).await.map_err(|e| {
                    println!("DEBUG: Failed to read queries.hx: {}", e);
                    e
                })? ;
                let schema_content = fs::read_to_string(&schema_hx_path).await.map_err(|e| {
                    println!("DEBUG: Failed to read schema.hx: {}", e);
                    e
                })?;
                let generated_rust_code = fs::read_to_string(&compile_output_path.join("queries.rs")).await.map_err(|e| {
                    println!("DEBUG: Failed to read queries.rs: {}", e);
                    e
                })?;
                handle_error_with_github(
                    config,
                    "Cargo Check",
                    &error_message,
                    file_num,
                    &query_content,
                    &schema_content,
                    &generated_rust_code,
                )
                .await?;
            } else {
                println!("DEBUG: GitHub integration not configured, skipping issue creation");
            }
            fs::remove_dir_all(&temp_dir).await.ok();
            bail!("Error: {}", error_message);
        }
    }

    println!("Cargo check passed for {}", file_num);
    // Clean up temp directory
    fs::remove_dir_all(&temp_dir).await.ok();

    Ok(())
}

async fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst).await?;
    }

    let mut entries = fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let dest_path = dst.join(file_name);

        if path.is_dir() {
            Box::pin(copy_dir_recursive(&path, &dest_path)).await?;
        } else {
            fs::copy(&path, &dest_path).await?;
        }
    }

    Ok(())
}
