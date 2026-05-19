# ⚡ Antigravity Core MCP Server

### *The Unified, High-Performance Developer Tool suite for AI Agents & Coders*

Welcome! If you are a developer, an AI engineer, or just someone looking to give your AI assistants super-powers, you've come to the right place. 

This repository houses a single, lightning-fast, highly optimized Rust executable that implements the **Model Context Protocol (MCP)**. By starting the application with a single CLI flag, it acts as one of four distinct advanced developer servers.

---

## 🤔 What is an MCP Server?

If you've ever used an AI assistant like Claude or ChatGPT, you know they are incredibly smart but lack direct access to the real world. They can't browse a website on demand, read local files, check which version of a programming library is the newest, or run a compiler to see why code fails.

**Model Context Protocol (MCP)** is an open standard developed by Anthropic that acts as a secure USB port for AI. 
*   An **MCP Client** (like Claude Desktop or Cursor) is the host.
*   An **MCP Server** (like this binary) is the tool-belt.
*   By plugging this server into your client, your AI assistant instantly gains active, state-of-the-art tools to perform real-world tasks on your behalf!

---

## 🌟 The 4-in-1 Power Suite

Instead of running four separate, heavy, sluggish applications, **Antigravity Core** consolidates everything into a single, microscopic **~8.3 MB** executable. You simply run it with a flag telling it which server to become:

```powershell
# Invocation syntax:
antigravity_core_mcp.exe --server <mode>
```

| Server Mode (`--server <mode>`) | Core Capability | Key Exposed Tools |
| :--- | :--- | :--- |
| **`firecrawl`** | High-speed web scraping and crawling translation bridge. | `firecrawl_scrape`, `firecrawl_crawl`, `firecrawl_map` |
| **`crates`** | Smart dependency analyzer and crates.io registry librarian. | `optimize_cargo_toml`, `get_latest_version`, `search_crates` |
| **`borrow`** | Structural borrow-checker error visualizer & manual. | `map_lifetimes`, `explain_compiler_error` |
| **`appliance`** | Idiomatic Rust pattern injector and code quality linter. | `inject_pattern`, `lint_idiomatic` |

---

## 🚀 Easy Installation Guide

We have made this server incredibly simple to install whether you are a complete beginner or an expert systems engineer.

### Method 1: The Easy Way (Pre-compiled Binary)
*Perfect for non-programmers or quick setups.*

1. **Download the Binary**: Get the pre-compiled `antigravity_core_mcp.exe` file from our releases page and save it somewhere on your computer (e.g., `C:\MCP\antigravity_core_mcp.exe`).
2. **Open Claude Desktop Config**: Open your Claude Desktop configuration file. You can find it by pressing `Win + R` and pasting:
   ```text
   %appdata%\Claude\claude_desktop_config.json
   ```
3. **Plug it in**: Add any (or all) of the server modes you want to run under the `mcpServers` object in the JSON file (see the [Claude Configuration Section](#-claude-desktop-configuration) below).
4. **Restart Claude**: Fully restart Claude Desktop. Look for the little **Hammer 🛠️** icon in the chat box—that means your new tools are live and ready!

---

### Method 2: The Developer Way (Build from Source)
*For developers who want the latest changes compiled locally.*

1. **Prerequisites**: Ensure you have Git and Rust installed. If not, get Rust from [rustup.rs](https://rustup.rs/).
2. **Clone the Repo**:
   ```bash
   git clone https://github.com/Dahc-Dragyn/antigravity_core_mcp.git
   cd antigravity_core_mcp
   ```
3. **Compile the Release Binary**:
   ```bash
   cargo build --release
   ```
   This will compile a highly optimized, minified executable located at:
   `./target/release/antigravity_core_mcp.exe`

---

## 🛠️ Claude Desktop Configuration

Paste the following configuration into your `%appdata%\Claude\claude_desktop_config.json` file. Make sure to replace `C:\\path\\to\\bin\\antigravity_core_mcp.exe` with the exact path where your compiled or downloaded executable is saved!

> [!IMPORTANT]
> Because Windows JSON paths require double backslashes, make sure your path is written like: `C:\\MCP\\antigravity_core_mcp.exe`.

```json
{
  "mcpServers": {
    "antigravity-appliance": {
      "command": "C:\\path\\to\\bin\\antigravity_core_mcp.exe",
      "args": ["--server", "appliance"]
    },
    "antigravity-borrow": {
      "command": "C:\\path\\to\\bin\\antigravity_core_mcp.exe",
      "args": ["--server", "borrow"]
    },
    "antigravity-crates": {
      "command": "C:\\path\\to\\bin\\antigravity_core_mcp.exe",
      "args": ["--server", "crates"]
    },
    "antigravity-firecrawl": {
      "command": "C:\\path\\to\\bin\\antigravity_core_mcp.exe",
      "args": ["--server", "firecrawl"]
    }
  }
}
```

---

## 📖 Server Modes & Tools Reference

Here is a detailed breakdown of what each server mode does, the tools it gives your AI, and how they work.

### 1. 🔥 Firecrawl Mode (`--server firecrawl`)
Turns the server into a translation bridge for the Firecrawl service, enabling the AI to harvest websites into perfect, readable markdown while stripping out cookie banners, ads, and clutter.

*   **`firecrawl_scrape`**
    *   *What it does*: Converts a single web page into clean, LLM-ready markdown.
    *   *Parameters*:
        *   `url` (String): The page to scrape.
        *   `formats` (Optional Array of Strings): e.g., `["markdown"]`.
        *   `only_main_content` (Optional Boolean): Isolate and return only the main article blocks.
*   **`firecrawl_crawl`**
    *   *What it does*: Initiates an asynchronous crawl queue across an entire domain.
    *   *Parameters*:
        *   `url` (String): Starting URL.
        *   `limit` (Optional Integer): Max pages to crawl.
*   **`firecrawl_get_crawl_status`**
    *   *What it does*: Monitors the status of a crawl job and returns scraped pages.
    *   *Parameters*: `job_id` (String).
*   **`firecrawl_map`**
    *   *What it does*: Discovers and maps out the structural URL tree of a domain.

---

### 📦 Crates Mode (`--server crates`)
A smart dependency advisor that queries the Crates.io registry to check for new, stable versions of Rust packages, inspect features, and optimize project manifests.

> [!NOTE]
> **Safety Invariant**: This mode has a built-in sequential rate-limiter that sleeps for **1050ms** between successive queries to ensure full crawler compliance with crates.io and prevent your IP from being throttled.

*   **`search_crates`**
    *   *What it does*: Search crates.io for package descriptions and downloads.
    *   *Parameters*: `query` (String).
*   **`get_latest_version`**
    *   *What it does*: Retrieves version info, feature lists, and documentation links for a specific crate.
    *   *Parameters*: `crate_name` (String).
*   **`optimize_cargo_toml`**
    *   *What it does*: Scans a local `Cargo.toml` manifest file, analyzes its dependencies safely, and recommends stable upgrades.
    *   *Parameters*: `file_path` (String) - Absolute path to the Cargo.toml file.

---

### 📈 Borrow Mode (`--server borrow`)
The ultimate lifesaver for anyone fighting the Rust compiler's borrow-checker! It parses compilation diagnostics and creates a structural ASCII lifecycle chart of references.

> [!TIP]
> **Deadlock Protection**: Designed with `Stdio::inherit()` stderr redirection to prevent Windows stream deadlocks when analyzing heavy diagnostic logs.

*   **`explain_compiler_error`**
    *   *What it does*: Returns a detailed root-cause explanation and three idiomatic solutions for specific Rust compiler errors (like `E0382` or `E0502`).
    *   *Parameters*: 
        *   `error_code` (String): e.g. `"E0382"`.
        *   `source_snippet` (Optional String): Code block causing the error.
*   **`map_lifetimes`**
    *   *What it does*: Programmatically compiles a local crate, extracts AST JSON spans, and prints a chronological ASCII timeline of references so you can see exactly where a value was borrowed, moved, or dropped.
    *   *Parameters*: `project_path` (String) - Absolute path to the Rust project containing the `Cargo.toml`.

---

### 🔌 Appliance Mode (`--server appliance`)
The ultimate code scaffold and linter that helps you write high-performance, idiomatic, zero-panic Rust applications.

*   **`inject_pattern`**
    *   *What it does*: Injects pre-compiled, production-grade, compiler-checked structural string templates for elite design patterns directly into a file.
    *   *Parameters*:
        *   `pattern_name` (String): One of `"type_state_builder"`, `"thiserror_mapping"`, or `"parallel_rayon_loop"`.
        *   `target_path` (String): Absolute path to the destination file.
*   **`lint_idiomatic`**
    *   *What it does*: Scans custom Rust code blocks for common anti-patterns (unnecessary heap allocations, raw `.unwrap()`, manual loops) and outputs structural rewrite recommendations.
    *   *Parameters*: `code_block` (String) - The raw Rust source block to scan.

---

## 🏆 Standards & Hardening

*   **Zero-Panic Execution**: Designed with comprehensive error boundaries to guarantee the server never crashes or panics at runtime.
*   **Ultra-Lightweight Footprint**: Compiled using release size-optimization profiles, delivering a self-contained single binary with zero external DLL dependencies.
*   **Thread-Safe**: Built on top of the asynchronous `tokio` multi-threaded runtime to handle multiple parallel requests flawlessly.

---
> **Project Status**: 🚀 **Production-Ready** | **Maintained by Dahc-Dragyn**
