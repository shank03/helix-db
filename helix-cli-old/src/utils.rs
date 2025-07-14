
pub fn get_cfg_deploy_path(cmd_path: Option<String>) -> Result<String, CliError> {
    if let Some(path) = cmd_path {
        return Ok(path);
    }

    let cwd = ".";
    let files = match check_and_read_files(cwd) {
        Ok(files) => files,
        Err(_) => {
            return Ok(DB_DIR.to_string());
        }
    };

    if !files.is_empty() {
        return Ok(cwd.to_string());
    }

    Ok(DB_DIR.to_string())
}

pub fn find_available_port(start_port: u16) -> Option<u16> {
    let mut port = start_port;
    while port < 65535 {
        let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>().unwrap();
        match TcpListener::bind(addr) {
            Ok(listener) => {
                drop(listener);
                let localhost = format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap();
                match TcpListener::bind(localhost) {
                    Ok(local_listener) => {
                        drop(local_listener);
                        return Some(port);
                    }
                    Err(e) => {
                        if e.kind() != ErrorKind::AddrInUse {
                            return None;
                        }
                        port += 1;
                        continue;
                    }
                }
            }
            Err(e) => {
                if e.kind() != ErrorKind::AddrInUse {
                    return None;
                }
                port += 1;
                continue;
            }
        }
    }
    None
}

/// Checks if the path contains a schema.hx and config.hx.json file
/// Returns a vector of DirEntry objects for all .hx files in the path
pub fn check_and_read_files(path: &str) -> Result<Vec<DirEntry>, CliError> {
    if !fs::read_dir(&path)
        .map_err(CliError::Io)?
        .any(|file| file.unwrap().file_name() == "schema.hx")
    {
        return Err(CliError::from(format!(
            "{}",
            "No schema file found".red().bold()
        )));
    }

    if !fs::read_dir(&path)
        .map_err(CliError::Io)?
        .any(|file| file.unwrap().file_name() == "config.hx.json")
    {
        return Err(CliError::from(format!(
            "{}",
            "No config.hx.json file found".red().bold()
        )));
    }

    let files: Vec<DirEntry> = fs::read_dir(&path)?
        .filter_map(|entry| entry.ok())
        .filter(|file| file.file_name().to_string_lossy().ends_with(".hx"))
        .collect();

    // Check for query files (exclude schema.hx)
    let has_queries = files.iter().any(|file| file.file_name() != "schema.hx");
    if !has_queries {
        return Err(CliError::from(format!(
            "{}",
            "No query files (.hx) found".red().bold()
        )));
    }

    Ok(files)
}

/// Generates a Content object from a vector of DirEntry objects
/// Returns a Content object with the files and source
///
/// This essentially makes a full string of all of the files while having a separate vector of the individual files
///
/// This could be changed in the future but keeps the option open for being able to access the files separately or all at once
pub fn generate_content(files: &Vec<DirEntry>) -> Result<Content, CliError> {
    let files: Vec<HxFile> = files
        .iter()
        .map(|file| {
            let name = file.path().to_string_lossy().into_owned();
            let content = fs::read_to_string(file.path()).unwrap();
            HxFile { name, content }
        })
        .collect();

    let content = files
        .clone()
        .iter()
        .map(|file| file.content.clone())
        .collect::<Vec<String>>()
        .join("\n");

    Ok(Content {
        content,
        files,
        source: Source::default(),
    })
}

/// Uses the helix parser to parse the content into a Source object
fn parse_content(content: &Content) -> Result<Source, CliError> {
    let source = match HelixParser::parse_source(&content) {
        Ok(source) => source,
        Err(e) => {
            return Err(CliError::from(format!("{}", e)));
        }
    };

    Ok(source)
}

/// Runs the static analyzer on the parsed source to catch errors and generate diagnostics if any.
/// Otherwise returns the generated source object which is an IR used to transpile the queries to rust.
fn analyze_source(source: Source) -> Result<GeneratedSource, CliError> {
    let (diagnostics, source) = analyze(&source);
    if !diagnostics.is_empty() {
        for diag in diagnostics {
            let filepath = diag.filepath.clone().unwrap_or("queries.hx".to_string());
            println!("{}", diag.render(&source.src, &filepath));
        }
        return Err(CliError::CompileFailed);
    }

    Ok(source)
}

/// Generates a Content and GeneratedSource object from a vector of DirEntry objects
/// Returns a tuple of the Content and GeneratedSource objects
///
/// This function is the main entry point for generating the Content and GeneratedSource objects
///
/// It first generates the content from the files, then parses the content into a Source object, and then analyzes the source to catch errors and generate diagnostics if any.
pub fn generate(files: &Vec<DirEntry>) -> Result<(Content, GeneratedSource), CliError> {
    let mut content = generate_content(&files)?;
    content.source = parse_content(&content)?;
    let analyzed_source = analyze_source(content.source.clone())?;
    Ok((content, analyzed_source))
}

pub fn gen_typescript(source: &GeneratedSource, output_path: &str) -> Result<(), CliError> {
    let mut file = File::create(PathBuf::from(output_path).join("interface.d.ts"))?;

    for node in &source.nodes {
        write!(file, "{}", node.to_typescript())?;
    }
    for edge in &source.edges {
        write!(file, "{}", edge.to_typescript())?;
    }
    for vector in &source.vectors {
        write!(file, "{}", vector.to_typescript())?;
    }

    Ok(())
}

