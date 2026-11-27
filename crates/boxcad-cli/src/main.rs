//! Command-line access to the core, so the document model and geometry can be
//! tested and swept without the app. Subcommands arrive with the features they
//! exercise.

use boxcad_core::{
    Document, Edit, EntityId, Grain, History, Item, Material, MaterialId, Panel, PanelId, Plane,
    SheetSize, Sketch, SketchId, ThicknessSide, file,
};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "boxcad-cli", version, about = "Box CAD from the command line")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the core library's version.
    Version,
    /// Write a project holding one rectangular panel.
    Sample {
        /// Where to write the project, for example panel.boxcad
        path: PathBuf,
        /// Panel width in mm
        #[arg(long, default_value_t = 200.0)]
        width: f64,
        /// Panel depth in mm
        #[arg(long, default_value_t = 120.0)]
        depth: f64,
        /// Material thickness in mm
        #[arg(long, default_value_t = 3.0)]
        thickness: f64,
        /// Laser kerf in mm
        #[arg(long, default_value_t = 0.15)]
        kerf: f64,
    },
    /// Open a project, check it, and list what it holds.
    Info {
        /// The project file to read
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    let result = match Cli::parse().command {
        Command::Version => {
            println!("boxcad-core {}", boxcad_core::version());
            Ok(())
        }
        Command::Sample { path, width, depth, thickness, kerf } => {
            sample(&path, width, depth, thickness, kerf)
        }
        Command::Info { path } => info(&path),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(problem) => {
            eprintln!("boxcad-cli: {problem}");
            ExitCode::FAILURE
        }
    }
}

type Outcome = Result<(), Box<dyn std::error::Error>>;

/// Build a one-panel design the way the app will, through edits, and save it.
fn sample(path: &Path, width: f64, depth: f64, thickness: f64, kerf: f64) -> Outcome {
    let mut history = History::new(Document::new("Sample panel"));

    let material = Material {
        id: MaterialId(history.allocate_id()),
        name: "Birch ply".into(),
        thickness,
        kerf,
        sheet: SheetSize { width: 600.0, height: 400.0 },
        grain: Grain::None,
    };
    let material_id = material.id;
    history.apply("Add material", Edit::add(Item::Material(material)))?;

    let sketch_id = SketchId(history.allocate_id());
    let sides = std::array::from_fn(|_| EntityId(history.allocate_id()));
    let outline =
        Sketch::rectangle(sketch_id, "Panel outline", Plane::ground(), [0.0, 0.0], width, depth, sides);
    let panel = Panel {
        id: PanelId(history.allocate_id()),
        name: "Panel".into(),
        sketch: sketch_id,
        material: material_id,
        thickness_side: ThicknessSide::Positive,
    };
    history.apply(
        "Draw panel",
        Edit::Group(vec![Edit::add(Item::Sketch(outline)), Edit::add(Item::Panel(panel))]),
    )?;

    file::save(path, history.document())?;
    println!("Wrote {}", path.display());
    Ok(())
}

fn info(path: &Path) -> Outcome {
    let doc = file::load(path)?;
    println!("{} ({})", doc.name, path.display());
    println!("  file layout version {}, valid", file::VERSION);
    println!("  materials: {}", doc.materials.len());
    for m in &doc.materials {
        println!(
            "    {}: {} mm thick, kerf {} mm, sheet {} × {} mm",
            m.name, m.thickness, m.kerf, m.sheet.width, m.sheet.height
        );
    }
    println!("  sketches: {}", doc.sketches.len());
    for s in &doc.sketches {
        println!("    {}: {} entities", s.name, s.entities.len());
    }
    println!("  panels: {}", doc.panels.len());
    for p in &doc.panels {
        let material = doc.material(p.material).map_or("?", |m| m.name.as_str());
        let sketch = doc.sketch(p.sketch).map_or("?", |s| s.name.as_str());
        println!("    {}: outline \"{}\", material \"{}\"", p.name, sketch, material);
    }
    println!("  parameters: {}", doc.parameters.len());
    for p in &doc.parameters {
        println!("    {} = {}", p.name, p.value);
    }
    Ok(())
}
