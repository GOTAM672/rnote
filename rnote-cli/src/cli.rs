use smol::fs::File;
use smol::io::{AsyncReadExt, AsyncWriteExt};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use rnote_engine::RnoteEngine;

/// rnote-cli
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Exports the Rnote file and saves it in the output file.{n}
    /// The export format is recognized from the file extension of the output file.{n}
    /// Currently `.svg`, `.xopp` and `.pdf` are supported.
    Export {
        /// the rnote save file
        rnote_file: PathBuf,
        /// the export output file
        #[arg(short, long)]
        output_file: PathBuf,
    },
}

pub(crate) async fn run() -> anyhow::Result<()> {
    let mut engine = RnoteEngine::default();

    let cli = Cli::parse();

    match cli.command {
        Commands::Export {
            rnote_file,
            output_file,
        } => {
            println!("Converting..");
            convert_file(&mut engine, rnote_file, output_file).await?;
            println!("Finished!");
        }
    }

    Ok(())
}

pub(crate) async fn convert_file(
    engine: &mut RnoteEngine,
    input_file: PathBuf,
    output_file: PathBuf,
) -> anyhow::Result<()> {
    let mut input_bytes = vec![];

    File::open(input_file)
        .await?
        .read_to_end(&mut input_bytes)
        .await?;

    let store_snapshot = engine.open_from_rnote_bytes_p1(input_bytes)?.await??;

    engine.open_from_store_snapshot_p2(&store_snapshot)?;

    let export_title = output_file
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("output_file"));

    let export_bytes = match output_file.extension().and_then(|ext| ext.to_str()) {
        Some("svg") => engine.export_doc_as_svg_bytes(None).await??,
        Some("xopp") => {
            engine
                .export_doc_as_xopp_bytes(export_title, None)
                .await??
        }
        Some("pdf") => engine.export_doc_as_pdf_bytes(export_title, None).await??,
        Some(ext) => {
            return Err(anyhow::anyhow!(
                "unsupported extension `{ext}` for output file"
            ))
        }
        None => {
            return Err(anyhow::anyhow!(
                "Output file needs to have an extension to determine the file type."
            ))
        }
    };

    File::create(output_file)
        .await?
        .write_all(&export_bytes)
        .await?;

    Ok(())
}
