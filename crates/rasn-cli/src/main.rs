use anyhow::Result;
mod batch;

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use serde::Serialize;
use std::sync::Arc;

/// High-performance ASN mapper with Apache Arrow columnar storage
#[derive(Parser)]
#[command(name = "rasn")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value = "human", global = true)]
    output: OutputFormat,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lookup ASN information for IP address, ASN, or domain
    Lookup(LookupArgs),
    /// Batch process multiple inputs from file or stdin
    Batch(BatchArgs),
    /// Start MCP server for AI agent integration
    Mcp(McpArgs),
}

#[derive(Parser)]
struct LookupArgs {
    /// IP address, ASN number (e.g., AS15169), or domain name
    #[arg(value_name = "TARGET")]
    target: String,
}

#[derive(Parser)]
struct BatchArgs {
    /// Input file (use '-' for stdin)
    #[arg(short, long, value_name = "FILE")]
    file: Option<String>,

    /// Number of concurrent workers
    #[arg(short, long, default_value = "10")]
    workers: usize,
}

#[derive(Parser)]
struct McpArgs {
    /// Transport mode
    #[arg(value_enum, default_value = "stdio")]
    transport: TransportMode,

    /// HTTP port (only for http transport)
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    /// Human-readable table output
    Human,
    /// JSON output (pretty-printed)
    Json,
    /// JSON output (compact)
    JsonCompact,
    /// CSV output
    Csv,
}

#[derive(Debug, Clone, ValueEnum)]
enum TransportMode {
    /// Standard I/O (for Claude Desktop)
    Stdio,
    /// HTTP server
    Http,
}

#[derive(Serialize)]
struct LookupResult {
    target: String,
    asn: Option<u32>,
    organization: Option<String>,
    country: Option<String>,
    description: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lookup(args) => handle_lookup(args, cli.output, cli.verbose)?,
        Commands::Batch(args) => handle_batch(args, cli.output, cli.verbose)?,
        Commands::Mcp(args) => handle_mcp(args, cli.verbose)?,
    }

    Ok(())
}

fn handle_lookup(args: LookupArgs, format: OutputFormat, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("{} Looking up: {}", "›".blue(), args.target);
    }

    // Placeholder response - will be replaced with real lookup in Phase 2
    let result = LookupResult {
        target: args.target.clone(),
        asn: Some(15169),
        organization: Some("Google LLC".to_string()),
        country: Some("US".to_string()),
        description: Some("Google".to_string()),
    };

    print_result(&result, format)?;
    Ok(())
}

fn handle_bulk(args: BulkArgs, _format: OutputFormat, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("{} Bulk processing from: {}", "›".blue(), args.input);
        if let Some(ref output) = args.output {
            eprintln!("{} Writing to: {}", "›".blue(), output);
        } else {
            eprintln!("{} Writing to stdout", "›".blue());
        }
    }

    println!("{}", "Bulk processing - Coming soon! (Phase 3.1)".yellow());
    Ok(())
}

fn handle_batch(args: BatchArgs, _format: OutputFormat, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!(
            "{} Batch processing with {} workers",
            "›".blue(),
            args.workers
        );
        if let Some(ref file) = args.file {
            eprintln!("{} Reading from: {}", "›".blue(), file);
        } else {
            eprintln!("{} Reading from stdin", "›".blue());
        }
    }

    println!("{}", "Batch processing - Coming soon! (Phase 3.2)".yellow());
    Ok(())
}

fn handle_mcp(args: McpArgs, verbose: bool) -> Result<()> {
    let server = rasn_mcp::McpServer::new(None)
        .map_err(|e| anyhow::anyhow!("Failed to create MCP server: {}", e))?;
    let server = std::sync::Arc::new(server);

    match args.transport {
        TransportMode::Stdio => {
            if verbose {
                eprintln!("{} Starting MCP server on STDIO", "›".blue());
            }
            let transport = rasn_mcp::transport::StdioTransport::new(server);
            transport
                .run_blocking()
                .map_err(|e| anyhow::anyhow!("STDIO transport error: {}", e))?;
        }
        TransportMode::Http => {
            if verbose {
                eprintln!(
                    "{} Starting MCP server on HTTP port {}",
                    "›".blue(),
                    args.port
                );
            }
            println!("{}", "HTTP transport - Coming soon!".yellow());
        }
    }

    Ok(())
}

fn print_result(result: &LookupResult, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Human => print_human(result),
        OutputFormat::Json => print_json(result, true)?,
        OutputFormat::JsonCompact => print_json(result, false)?,
        OutputFormat::Csv => print_csv(result)?,
    }
    Ok(())
}

fn print_human(result: &LookupResult) {
    println!();
    println!("{}", "ASN Lookup Result".bold().cyan());
    println!("{}", "─".repeat(50).dimmed());
    println!("{:>15}: {}", "Target".bold(), result.target);

    if let Some(asn) = result.asn {
        println!("{:>15}: {}", "ASN".bold(), format!("AS{}", asn).green());
    }

    if let Some(ref org) = result.organization {
        println!("{:>15}: {}", "Organization".bold(), org);
    }

    if let Some(ref country) = result.country {
        println!("{:>15}: {}", "Country".bold(), country);
    }

    if let Some(ref desc) = result.description {
        println!("{:>15}: {}", "Description".bold(), desc);
    }
    println!();
}

fn print_json(result: &LookupResult, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(result)?);
    } else {
        println!("{}", serde_json::to_string(result)?);
    }
    Ok(())
}

fn print_csv(result: &LookupResult) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    wtr.write_record(["target", "asn", "organization", "country", "description"])?;
    wtr.write_record([
        &result.target,
        &result.asn.map_or(String::new(), |a| a.to_string()),
        result.organization.as_deref().unwrap_or(""),
        result.country.as_deref().unwrap_or(""),
        result.description.as_deref().unwrap_or(""),
    ])?;
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::parse_from(["rasn", "lookup", "8.8.8.8"]);
        assert!(matches!(cli.command, Commands::Lookup(_)));
        assert!(matches!(cli.output, OutputFormat::Human));
    }

    #[test]
    fn test_output_format_json() {
        let cli = Cli::parse_from(["rasn", "--output", "json", "lookup", "AS15169"]);
        assert!(matches!(cli.output, OutputFormat::Json));
    }

    #[test]
    fn test_batch_command() {
        let cli = Cli::parse_from(["rasn", "batch", "--file", "ips.txt", "--workers", "5"]);
        if let Commands::Batch(args) = cli.command {
            assert_eq!(args.file, Some("ips.txt".to_string()));
            assert_eq!(args.workers, 5);
        } else {
            panic!("Expected Batch command");
        }
    }

    #[test]
    fn test_mcp_command() {
        let cli = Cli::parse_from(["rasn", "mcp", "http", "--port", "9090"]);
        if let Commands::Mcp(args) = cli.command {
            assert!(matches!(args.transport, TransportMode::Http));
            assert_eq!(args.port, 9090);
        } else {
            panic!("Expected Mcp command");
        }
    }

    #[test]
    fn test_verbose_flag() {
        let cli = Cli::parse_from(["rasn", "-v", "lookup", "1.1.1.1"]);
        assert!(cli.verbose);
    }
}
