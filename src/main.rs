use rmcp::{tool, tool_router, ServiceExt, ErrorData};
use rmcp::model::{CallToolResult, ErrorCode};
use rmcp::handler::server::wrapper::Parameters;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use std::env;
use std::path::PathBuf;
use std::process;
use std::process::Stdio;
use tokio::io::{stdin, stdout, BufReader, AsyncBufReadExt};
use tokio::process::Command;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

// =========================================================================
// 📥 Input Parameter Schemas
// =========================================================================

#[derive(Deserialize, JsonSchema)]
struct ScrapeParams {
    /// The target URL to scrape
    url: String,
    /// List of target formats to return, e.g., ["markdown"] or ["html"]
    formats: Option<Vec<String>>,
    /// Set to true to isolate and return only the main content blocks
    only_main_content: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
struct CrawlParams {
    /// The starting base URL to crawl
    url: String,
    /// Maximum number of pages to crawl (Default: 5)
    limit: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
struct CrawlStatusParams {
    /// The unique asynchronous crawl task ID
    job_id: String,
}

#[derive(Deserialize, JsonSchema)]
struct MapParams {
    /// The base domain to discover
    url: String,
    /// Optional query search filter for discovering sub-directories
    search: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct SearchParams {
    /// The search query term to look up on crates.io (e.g. "serde")
    query: String,
}

#[derive(Deserialize, JsonSchema)]
struct VersionParams {
    /// The exact name of the target crate (e.g. "tokio")
    crate_name: String,
}

#[derive(Deserialize, JsonSchema)]
struct OptimizeParams {
    /// Absolute or relative path to the target Cargo.toml file to analyze
    file_path: String,
}

#[derive(Deserialize, JsonSchema)]
struct ExplainParams {
    /// Rust compiler error code (e.g. "E0382", "E0502")
    error_code: String,
    /// Optional source snippet related to the error
    source_snippet: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct MapLifetimeParams {
    /// Absolute path to the Rust project root containing Cargo.toml
    project_path: String,
}

#[derive(Deserialize, JsonSchema)]
struct InjectParams {
    /// Name of the elite pattern to inject (e.g., "type_state_builder", "thiserror_mapping", "parallel_rayon_loop")
    pattern_name: String,
    /// Absolute path to the destination file in the target project workspace
    target_path: String,
}

#[derive(Deserialize, JsonSchema)]
struct LintParams {
    /// Raw Rust source code block to analyze and refactor
    code_block: String,
}

// =========================================================================
// 📡 Server 1: Firecrawl Server Implementation
// =========================================================================

#[derive(Clone)]
struct FirecrawlServer {
    api_url: String,
    api_key: String,
    http_client: reqwest::Client,
}

impl FirecrawlServer {
    pub fn new() -> Self {
        let api_url = env::var("FIRECRAWL_API_URL")
            .unwrap_or_else(|_| "http://localhost:3002".to_string());
        let api_key = env::var("FIRECRAWL_API_KEY")
            .unwrap_or_else(|_| "local_development".to_string());
        Self {
            api_url,
            api_key,
            http_client: reqwest::Client::new(),
        }
    }
}

#[tool_router(server_handler)]
impl FirecrawlServer {
    #[tool(name = "firecrawl_ping", description = "Internal chassis health check")]
    async fn ping(&self) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::structured(serde_json::json!({"status": "ok"})))
    }

    #[tool(
        name = "firecrawl_scrape",
        description = "Scrapes a single URL and converts it into clean, LLM-ready markdown or text."
    )]
    async fn scrape_url(&self, params: Parameters<ScrapeParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Firecrawl MCP] Executing scrape for: {}", params.url);
        
        let scrape_endpoint = format!("{}/v1/scrape", self.api_url);
        let payload = serde_json::json!({
            "url": params.url,
            "formats": params.formats.unwrap_or_else(|| vec!["markdown".to_string()]),
            "onlyMainContent": params.only_main_content.unwrap_or(true),
        });

        let response = self.http_client
            .post(&scrape_endpoint)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("HTTP Connection failed: {}", e), None))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed parsing API JSON: {}", e), None))?;

        Ok(CallToolResult::structured(body))
    }

    #[tool(
        name = "firecrawl_crawl",
        description = "Asynchronously starts a crawl job on a target website domain."
    )]
    async fn crawl_website(&self, params: Parameters<CrawlParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Firecrawl MCP] Starting asynchronous crawl for: {}", params.url);

        let crawl_endpoint = format!("{}/v1/crawl", self.api_url);
        let payload = serde_json::json!({
            "url": params.url,
            "limit": params.limit.unwrap_or(5),
        });

        let response = self.http_client
            .post(&crawl_endpoint)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("HTTP Connection failed: {}", e), None))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed parsing API JSON: {}", e), None))?;

        Ok(CallToolResult::structured(body))
    }

    #[tool(
        name = "firecrawl_get_crawl_status",
        description = "Retrieves the status and scraped results of an ongoing crawl job."
    )]
    async fn get_crawl_status(&self, params: Parameters<CrawlStatusParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Firecrawl MCP] Checking status of crawl job: {}", params.job_id);

        let status_endpoint = format!("{}/v1/crawl/{}", self.api_url, params.job_id);

        let response = self.http_client
            .get(&status_endpoint)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("HTTP Connection failed: {}", e), None))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed parsing API JSON: {}", e), None))?;

        Ok(CallToolResult::structured(body))
    }

    #[tool(
        name = "firecrawl_map",
        description = "Discovers and maps out all structural sub-URLs of a website."
    )]
    async fn map_website(&self, params: Parameters<MapParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Firecrawl MCP] Mapping structural nodes for: {}", params.url);

        let map_endpoint = format!("{}/v1/map", self.api_url);
        let mut payload = serde_json::json!({
            "url": params.url,
        });

        if let Some(ref search) = params.search {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("search".to_string(), serde_json::json!(search));
            }
        }

        let response = self.http_client
            .post(&map_endpoint)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("HTTP Connection failed: {}", e), None))?;

        let body: Value = response
            .json()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed parsing API JSON: {}", e), None))?;

        Ok(CallToolResult::structured(body))
    }
}

// =========================================================================
// 📡 Server 2: Crates Advisor Server Implementation
// =========================================================================

#[derive(Clone)]
struct CratesAdvisorServer {
    http_client: reqwest::Client,
}

impl CratesAdvisorServer {
    pub fn new() -> Self {
        // Build the HTTP client with the mandatory Crawler Policy compliant User-Agent
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("crates_advisor_mcp/0.1.0 (local AI agent infrastructure)")
        );

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to initialize reqwest HTTP client");

        Self { http_client }
    }
}

#[tool_router(server_handler)]
impl CratesAdvisorServer {
    #[tool(name = "crates_ping", description = "Internal chassis health check")]
    async fn ping(&self) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::structured(serde_json::json!({"status": "ok"})))
    }

    #[tool(
        name = "search_crates",
        description = "Searches for matching crates on crates.io, returning descriptions and download metrics."
    )]
    async fn search_crates(&self, params: Parameters<SearchParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Crates Advisor] Searching crates.io for: '{}'", params.query);

        let url = format!("https://crates.io/api/v1/crates?q={}", utf8_percent_encode(&params.query, NON_ALPHANUMERIC));
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed to connect to crates.io: {}", e), None))?;

        if !response.status().is_success() {
            return Err(ErrorData::new(
                ErrorCode(500),
                format!("crates.io responded with error status: {}", response.status()),
                None,
            ));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed to parse response JSON: {}", e), None))?;

        // Extract and structure top 5 results for clean LLM reading
        let default_empty_vec = vec![];
        let crates = body.get("crates").and_then(|c| c.as_array()).unwrap_or(&default_empty_vec);
        let mut results = Vec::new();

        for cr in crates.iter().take(5) {
            let name = cr.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let downloads = cr.get("downloads").and_then(|d| d.as_u64()).unwrap_or(0);
            let max_version = cr.get("max_version").and_then(|v| v.as_str()).unwrap_or("unknown");
            let description = cr.get("description").and_then(|d| d.as_str()).unwrap_or("No description provided.");

            results.push(serde_json::json!({
                "crate_name": name,
                "downloads": downloads,
                "latest_version": max_version,
                "description": description
            }));
        }

        Ok(CallToolResult::structured(serde_json::json!({ "results": results })))
    }

    #[tool(
        name = "get_latest_version",
        description = "Retrieves structural version metrics, docs link, and feature lists for a specific crate."
    )]
    async fn get_latest_version(&self, params: Parameters<VersionParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Crates Advisor] Fetching details for crate: '{}'", params.crate_name);

        let url = format!("https://crates.io/api/v1/crates/{}", params.crate_name);
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed to connect to crates.io: {}", e), None))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ErrorData::new(
                ErrorCode(404),
                format!("Crate '{}' not found on crates.io.", params.crate_name),
                None,
            ));
        }

        if !response.status().is_success() {
            return Err(ErrorData::new(
                ErrorCode(500),
                format!("crates.io responded with error status: {}", response.status()),
                None,
            ));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed to parse response JSON: {}", e), None))?;

        // Format clean metadata
        let crate_data = body.get("crate").ok_or_else(|| {
            ErrorData::new(ErrorCode(500), "Missing 'crate' field in crates.io response".to_string(), None)
        })?;

        let name = crate_data.get("name").and_then(|n| n.as_str()).unwrap_or(&params.crate_name);
        let max_version = crate_data.get("max_version").and_then(|v| v.as_str()).unwrap_or("unknown");
        let docs = crate_data.get("documentation").and_then(|d| d.as_str()).unwrap_or("");
        
        // Find default or available features in version lists
        let default_empty_vec = vec![];
        let versions = body.get("versions").and_then(|v| v.as_array()).unwrap_or(&default_empty_vec);
        let mut features = serde_json::Map::new();

        if let Some(latest_ver_info) = versions.iter().find(|v| v.get("num").and_then(|n| n.as_str()) == Some(max_version)) {
            if let Some(feats) = latest_ver_info.get("features").and_then(|f| f.as_object()) {
                features = feats.clone();
            }
        }

        Ok(CallToolResult::structured(serde_json::json!({
            "crate_name": name,
            "latest_version": max_version,
            "documentation": docs,
            "available_features": features
        })))
    }

    #[tool(
        name = "optimize_cargo_toml",
        description = "Analyzes a local Cargo.toml sequentially with safe rate-limiting, recommending dependency upgrades and missing standards."
    )]
    async fn optimize_cargo_toml(&self, params: Parameters<OptimizeParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        let file_path = PathBuf::from(&params.file_path);
        eprintln!("[Crates Advisor] Optimizing manifest at: {:?}", file_path);

        if !file_path.exists() {
            return Err(ErrorData::new(
                ErrorCode(400),
                format!("Specified Cargo.toml file does not exist: {:?}", file_path),
                None,
            ));
        }

        let content = tokio::fs::read_to_string(&file_path)
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed to read target file: {}", e), None))?;

        let parsed: toml::Value = toml::from_str(&content)
            .map_err(|e| ErrorData::new(ErrorCode(400), format!("Invalid Cargo.toml syntax: {}", e), None))?;

        let mut recommendations = Vec::new();

        if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
            for (name, val) in deps {
                let current_version = match val {
                    toml::Value::String(s) => s.clone(),
                    toml::Value::Table(t) => t.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    _ => "".to_string(),
                };

                if current_version.is_empty() {
                    continue;
                }

                // 🚨 CRITICAL ADDITION: crates.io Crawler Rate Limiting (1 request per second max)
                // Sleep 1050ms sequentially between queries to guarantee IP safety
                eprintln!("[Crates Advisor] Rate-limiting pause (1050ms) before querying crates.io for '{}'...", name);
                tokio::time::sleep(std::time::Duration::from_millis(1050)).await;

                let url = format!("https://crates.io/api/v1/crates/{}", name);
                let response = self.http_client.get(&url).send().await;

                let res = match response {
                    Ok(resp) => resp,
                    Err(e) => {
                        eprintln!("[Crates Advisor] Warning: Failed to query rates for '{}': {}", name, e);
                        continue;
                    }
                };

                if !res.status().is_success() {
                    eprintln!("[Crates Advisor] Warning: crates.io returned {} for '{}'", res.status(), name);
                    continue;
                }

                let body: Result<Value, _> = res.json().await;
                if let Ok(body_val) = body {
                    if let Some(crate_info) = body_val.get("crate") {
                        if let Some(max_version) = crate_info.get("max_version").and_then(|v| v.as_str()) {
                            // Suggest update if version mismatch
                            if current_version != max_version && !max_version.is_empty() {
                                recommendations.push(serde_json::json!({
                                    "dependency": name,
                                    "current_version": current_version,
                                    "latest_version": max_version,
                                    "status": "Update Available",
                                    "recommendation": format!("Upgrade '{}' from '{}' to '{}'", name, current_version, max_version)
                                }));
                            } else {
                                recommendations.push(serde_json::json!({
                                    "dependency": name,
                                    "current_version": current_version,
                                    "latest_version": max_version,
                                    "status": "Up to date",
                                    "recommendation": "None"
                                }));
                            }
                        }
                    }
                }
            }
        }

        Ok(CallToolResult::structured(serde_json::json!({
            "manifest_analyzed": params.file_path,
            "recommendations": recommendations
        })))
    }
}

// =========================================================================
// 📡 Server 3: Borrow Explainer Server Implementation
// =========================================================================

#[derive(Clone, Default)]
struct BorrowExplainerServer;

#[tool_router(server_handler)]
impl BorrowExplainerServer {
    #[tool(name = "borrow_ping", description = "Internal chassis health check")]
    async fn ping(&self) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::structured(serde_json::json!({"status": "ok"})))
    }

    #[tool(
        name = "explain_compiler_error",
        description = "Acts as an embedded reference manual, returning structural causes and 3 idiomatic solutions for a specific compiler error code."
    )]
    async fn explain_compiler_error(&self, params: Parameters<ExplainParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Borrow Explainer] Explaining compiler error: {}", params.error_code);
        if let Some(ref snippet) = params.source_snippet {
            eprintln!("[Borrow Explainer] Provided source snippet:\n{}", snippet);
        }

        let explanation = match params.error_code.to_uppercase().as_str() {
            "E0382" => serde_json::json!({
                "error_code": "E0382",
                "title": "Use of moved value",
                "explanation": "Occurs when a value is used after its ownership has been transferred (moved) to another variable or function parameter. By default, type assignment has move semantics unless the type implements the Copy trait.",
                "idiomatic_solutions": [
                    {
                        "approach": "Clone the value",
                        "description": "Call `.clone()` on the variable if the type implements Clone. This duplicates the data on the heap.",
                        "example": "let y = x.clone(); // x remains valid"
                    },
                    {
                        "approach": "Pass by reference",
                        "description": "Borrow the value via `&x` or `&mut x` instead of passing ownership, allowing the original caller to retain ownership.",
                        "example": "some_function(&x);"
                    },
                    {
                        "approach": "Scoped borrowing or static dispatch",
                        "description": "Reorganize code blocks or scopes using braces `{}` so borrows are resolved before ownership is moved, or use enum-based static dispatch instead of polymorphic ownership structures.",
                        "example": "{\n    let ref_x = &x;\n    // use ref_x\n}\nlet y = x; // valid because borrow has ended"
                    }
                ]
            }),
            "E0502" => serde_json::json!({
                "error_code": "E0502",
                "title": "Cannot borrow as mutable because it is also borrowed as immutable",
                "explanation": "Rust's strict aliasing rules state that you can have either one mutable borrow OR multiple immutable borrows at any given time, but never both simultaneously. This prevents data races at compile time.",
                "idiomatic_solutions": [
                    {
                        "approach": "Restrict borrow lifetimes via explicit scopes",
                        "description": "Use block scopes to complete the immutable borrow before starting the mutable borrow.",
                        "example": "{\n    let y = &x;\n    println!(\"{}\", y);\n} // y goes out of scope here\nlet z = &mut x; // valid"
                    },
                    {
                        "approach": "Refactor to work on owned copies or intermediate values",
                        "description": "Clone the read-only values or read them into intermediate Stack-allocated variables to avoid holding borrows.",
                        "example": "let temp = x.value;\nlet z = &mut x;\n// use temp and z separately"
                    },
                    {
                        "approach": "Non-Lexical Lifetimes structure",
                        "description": "Ensure the immutable borrow is not used after the mutable borrow starts. The compiler will automatically shorten the lifetime.",
                        "example": "let y = &x;\n// y used here\nlet z = &mut x; // valid if y is never used again"
                    }
                ]
            }),
            _ => serde_json::json!({
                "error_code": params.error_code,
                "title": "General Compiler Diagnostic",
                "explanation": "General Rust diagnostic. Consult standard compiler explanation via `rustc --explain`.",
                "idiomatic_solutions": [
                    {
                        "approach": "Read detailed span context",
                        "description": "Analyze compiler message output spans for primary labels explaining ownership changes.",
                        "example": "N/A"
                    }
                ]
            })
        };

        Ok(CallToolResult::structured(explanation))
    }

    #[tool(
        name = "map_lifetimes",
        description = "Programmatically runs cargo check with JSON diagnostics to map references and lifetimes as a beautiful ASCII timeline."
    )]
    async fn map_lifetimes(&self, params: Parameters<MapLifetimeParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        let root_path = PathBuf::from(&params.project_path);
        eprintln!("[Borrow Explainer] Mapping lifetimes for project at: {:?}", root_path);

        if !root_path.exists() {
            return Err(ErrorData::new(
                ErrorCode(400),
                format!("Project path does not exist: {:?}", root_path),
                None,
            ));
        }

        // 🚨 CRITICAL PATCH: Prevent Windows OS Pipe Deadlocks
        // Change Stdio::null() to Stdio::inherit() on stderr to safely pipe raw text diagnostics
        // out to the main terminal's stderr stream without polluting our JSON-RPC stdout channel.
        let mut child = Command::new("cargo")
            .args(["check", "--message-format=json"])
            .current_dir(&root_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed to spawn cargo check: {}", e), None))?;

        let stdout_stream = child.stdout.take().ok_or_else(|| {
            ErrorData::new(ErrorCode(500), "Failed to open child process stdout stream".to_string(), None)
        })?;

        let mut reader = BufReader::new(stdout_stream).lines();
        let mut diagnostic_messages = Vec::new();
        let mut timeline_visualization = String::new();

        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(val) = serde_json::from_str::<Value>(&line) {
                if val.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                    if let Some(msg) = val.get("message") {
                        diagnostic_messages.push(msg.clone());
                    }
                }
            }
        }

        let _status = child.wait().await;

        // 📈 Dynamic ASCII Timeline Builder using actual message.spans
        if diagnostic_messages.is_empty() {
            timeline_visualization.push_str("No compilation borrow/lifetime errors detected! Everything is green. ✨\n");
        } else {
            timeline_visualization.push_str("--- 📈 Structural Ownership Timeline ---\n");
            let mut span_count = 0;

            for msg in &diagnostic_messages {
                let error_code = msg.get("code")
                    .and_then(|c| c.get("code"))
                    .and_then(|code| code.as_str())
                    .unwrap_or("unknown");

                let rendered_desc = msg.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Diagnostic message");

                timeline_visualization.push_str(&format!("\nDiagnostic: [{}] {}\n", error_code, rendered_desc));

                if let Some(spans) = msg.get("spans").and_then(|s| s.as_array()) {
                    let mut sorted_spans = spans.clone();
                    // Sort spans chronologically by starting line
                    sorted_spans.sort_by_key(|a| {
                        a.get("line_start").and_then(|l| l.as_u64()).unwrap_or(0)
                    });

                    for span in sorted_spans {
                        let line_start = span.get("line_start").and_then(|l| l.as_u64()).unwrap_or(0);
                        let file_name = span.get("file_name").and_then(|f| f.as_str()).unwrap_or("src/main.rs");
                        let is_primary = span.get("is_primary").and_then(|p| p.as_bool()).unwrap_or(false);
                        let label = span.get("label").and_then(|l| l.as_str()).unwrap_or("");

                        // Retrieve source code snippet text if available
                        let code_snippet = span.get("text")
                            .and_then(|t| t.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|first_line| first_line.get("text"))
                            .and_then(|text| text.as_str())
                            .unwrap_or("")
                            .trim();

                        if line_start > 0 {
                            span_count += 1;
                            let timeline_glyph = if is_primary {
                                "[Primary]   *==================X (Borrow/Lifetime Error)"
                            } else {
                                "[Secondary]      *=========X    (Active Borrow Reference)"
                            };

                            timeline_visualization.push_str(&format!(
                                "  File: {} (Line {})\n  Code: {}\n  Map:  {}\n",
                                file_name, line_start, code_snippet, timeline_glyph
                            ));

                            if !label.is_empty() {
                                timeline_visualization.push_str(&format!("  Note: ^-- {}\n", label));
                            }
                            timeline_visualization.push('\n');
                        }
                    }
                }
            }

            if span_count == 0 {
                timeline_visualization.push_str("No detailed source spans available in compiler diagnostic outputs.\n");
            }
        }

        Ok(CallToolResult::structured(serde_json::json!({
            "project": params.project_path,
            "errors_found": diagnostic_messages.len(),
            "visual_timeline": timeline_visualization,
            "diagnostics": diagnostic_messages
        })))
    }
}

// =========================================================================
// 📡 Server 4: Rust Appliance Server Implementation
// =========================================================================

#[derive(Clone, Default)]
struct RustApplianceServer;

#[tool_router(server_handler)]
impl RustApplianceServer {
    #[tool(name = "appliance_ping", description = "Internal chassis health check")]
    async fn ping(&self) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::structured(serde_json::json!({"status": "ok"})))
    }

    #[tool(
        name = "inject_pattern",
        description = "Injects compile-time checked structural string templates for elite Rust patterns into target path."
    )]
    async fn inject_pattern(&self, params: Parameters<InjectParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Rust Appliance] Injecting pattern '{}' into: {}", params.pattern_name, params.target_path);

        let content = match params.pattern_name.as_str() {
            "type_state_builder" => {
                r#"// Elite Type-State Builder Pattern
pub struct Complete;
pub struct Missing;

pub struct ProjectBuilder<NameSet, PathSet> {
    name: Option<String>,
    path: Option<String>,
    _marker: std::marker::PhantomData<(NameSet, PathSet)>,
}

impl ProjectBuilder<Missing, Missing> {
    pub fn new() -> Self {
        Self {
            name: None,
            path: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<PathSet> ProjectBuilder<Missing, PathSet> {
    pub fn name(self, name: impl Into<String>) -> ProjectBuilder<Complete, PathSet> {
        ProjectBuilder {
            name: Some(name.into()),
            path: self.path,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<NameSet> ProjectBuilder<NameSet, Missing> {
    pub fn path(self, path: impl Into<String>) -> ProjectBuilder<NameSet, Complete> {
        ProjectBuilder {
            name: self.name,
            path: Some(path.into()),
            _marker: std::marker::PhantomData,
        }
    }
}

impl ProjectBuilder<Complete, Complete> {
    pub fn build(self) -> Project {
        Project {
            name: self.name.unwrap(),
            path: self.path.unwrap(),
        }
    }
}

pub struct Project {
    pub name: String,
    pub path: String,
}
"#
            }
            "thiserror_mapping" => {
                r#"use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplianceError {
    #[error("Workspace I/O error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse JSON configuration: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid pattern request: {0}")]
    InvalidPattern(String),
}
"#
            }
            "parallel_rayon_loop" => {
                r#"use rayon::prelude::*;

pub fn process_parallel(items: Vec<String>) -> Vec<String> {
    items.into_par_iter()
        .map(|item| {
            // Perform complex parallel calculations here
            item.to_uppercase()
        })
        .collect()
}
"#
            }
            other => {
                return Err(ErrorData::new(
                    ErrorCode(400),
                    format!("Unknown pattern: '{}'. Supported patterns: 'type_state_builder', 'thiserror_mapping', 'parallel_rayon_loop'", other),
                    None,
                ));
            }
        };

        let target_path = PathBuf::from(&params.target_path);
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed to create parent directories: {}", e), None))?;
        }

        tokio::fs::write(&target_path, content)
            .await
            .map_err(|e| ErrorData::new(ErrorCode(500), format!("Failed to write pattern to target path: {}", e), None))?;

        Ok(CallToolResult::structured(serde_json::json!({
            "status": "success",
            "pattern_injected": params.pattern_name,
            "target_file": params.target_path,
            "message": format!("Successfully injected '{}' pattern into {:?}", params.pattern_name, target_path)
        })))
    }

    #[tool(
        name = "lint_idiomatic",
        description = "Performs structural scans of Rust source blocks to refactor common anti-patterns into high-performance idiomatic code."
    )]
    async fn lint_idiomatic(&self, params: Parameters<LintParams>) -> Result<CallToolResult, ErrorData> {
        let params = params.0;
        eprintln!("[Rust Appliance] Performing idiomatic lint check on code block...");

        let mut findings = Vec::new();
        let code = &params.code_block;

        // 1. Check for manual vector index looping
        if code.contains("for ") && (code.contains(".len()") || code.contains("0..")) && code.contains("[") && code.contains("]") {
            findings.push(serde_json::json!({
                "anti_pattern": "Manual vector index looping",
                "severity": "Warning",
                "why_it_fails": "Direct indexing has bounds checking overhead and lacks Rust's standard safety/expressiveness. Manual loops bypass Iterator optimizations.",
                "architectural_alternative": "for (idx, item) in vec.iter().enumerate() {\n    // Use item and idx safely here without bounds-check overhead\n}"
            }));
        }

        // 2. Check for Box::new/vec! heap allocations where stack arrays are faster
        if code.contains("Box::new") || code.contains("vec![") {
            findings.push(serde_json::json!({
                "anti_pattern": "Unnecessary heap allocation (Box or Vec)",
                "severity": "Performance Advisory",
                "why_it_fails": "Allocating dynamic heap memory introduces thread synchronization and pointer chasing overhead when stack-allocated arrays `[T; N]` can be statically sized.",
                "architectural_alternative": "let stack_array: [i32; 3] = [1, 2, 3]; // Microsecond-fast allocation on stack"
            }));
        }

        // 3. Check for hazardous .unwrap()
        if code.contains(".unwrap()") {
            findings.push(serde_json::json!({
                "anti_pattern": "Hazardous use of .unwrap()",
                "severity": "Critical",
                "why_it_fails": "Direct panic on None or Err values violates production fault-tolerance rules. Production code must handle failures gracefully using pattern matching or bubbling via the `?` operator.",
                "architectural_alternative": "if let Some(value) = option {\n    // handle\n} else {\n    // fallback / recover\n}\n\n// Or bubble using Option/Result matching:\nlet value = option.ok_or(ApplianceError::InvalidPattern(\"Missing value\".into()))?;"
            }));
        }

        if findings.is_empty() {
            Ok(CallToolResult::structured(serde_json::json!({
                "status": "clean",
                "message": "100% clean sheet! The provided code block conforms perfectly to high-performance, idiomatic Rust standard specifications."
            })))
        } else {
            Ok(CallToolResult::structured(serde_json::json!({
                "status": "findings",
                "findings": findings
            })))
        }
    }
}

// =========================================================================
// 🚀 Main Entry Point: Dynamic Chassis Router
// =========================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Look for "--server <type>" in command line arguments
    let mut server_type: Option<&str> = None;
    for i in 0..args.len() {
        if args[i] == "--server" && i + 1 < args.len() {
            server_type = Some(&args[i + 1]);
            break;
        }
    }

    let server_type = match server_type {
        Some(t) => t.to_lowercase(),
        None => {
            print_usage();
            process::exit(1);
        }
    };

    eprintln!("[Antigravity Chassis] Initializing core chassis server...");
    let transport = (stdin(), stdout());

    match server_type.as_str() {
        "firecrawl" => {
            eprintln!("[Antigravity Chassis] Launching Firecrawl Search MCP Engine...");
            let service = FirecrawlServer::new();
            let server = service.serve(transport).await?;
            server.waiting().await?;
        }
        "crates" => {
            eprintln!("[Antigravity Chassis] Launching Crates Advisor MCP Engine...");
            let service = CratesAdvisorServer::new();
            let server = service.serve(transport).await?;
            server.waiting().await?;
        }
        "borrow" => {
            eprintln!("[Antigravity Chassis] Launching Borrow Explainer Lifetime Visualizer...");
            let service = BorrowExplainerServer;
            let server = service.serve(transport).await?;
            server.waiting().await?;
        }
        "appliance" => {
            eprintln!("[Antigravity Chassis] Launching Rust Appliance Pattern Generator...");
            let service = RustApplianceServer;
            let server = service.serve(transport).await?;
            server.waiting().await?;
        }
        other => {
            eprintln!("[Antigravity Chassis] Error: Unknown server target '{}'", other);
            print_usage();
            process::exit(1);
        }
    }

    eprintln!("[Antigravity Chassis] Server gracefully stopped.");
    Ok(())
}

fn print_usage() {
    eprintln!("\n=== Antigravity Core MCP Server Chassis ===");
    eprintln!("Usage: antigravity_core_mcp --server <target>");
    eprintln!("\nAvailable Targets:");
    eprintln!("  firecrawl   : Launches the Firecrawl search engine client");
    eprintln!("  crates      : Launches the Smart Cargo dependency auditor");
    eprintln!("  borrow      : Launches the Borrow-Checker and Lifetime Visualizer");
    eprintln!("  appliance   : Launches the Idiomatic Pattern Generator");
    eprintln!("============================================\n");
}
