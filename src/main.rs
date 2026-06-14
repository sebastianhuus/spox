use std::env;
use std::fs;
use std::path::PathBuf;

struct Spec {
    name: String,
    status: String,
    open_criteria: Vec<String>,
}

fn find_spox_dir() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    loop {
        let candidate = dir.join(".spox");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn parse_spec(path: &std::path::Path) -> Option<Spec> {
    let name = path.file_stem()?.to_string_lossy().into_owned();
    let content = fs::read_to_string(path).ok()?;
    let first_line = content.lines().next().unwrap_or("");
    let status = first_line
        .strip_prefix("status: ")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "(no status)".to_string());
    let open_criteria = content
        .lines()
        .filter_map(|line| line.strip_prefix("- [ ] ").map(str::to_string))
        .collect();
    Some(Spec { name, status, open_criteria })
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let show_criteria = args.iter().any(|a| a == "--criteria" || a == "-c");

    let spox_dir = find_spox_dir().unwrap_or_else(|| {
        eprintln!("error: no .spox directory found in this or any parent directory");
        std::process::exit(1);
    });

    let mut specs: Vec<Spec> = fs::read_dir(&spox_dir)
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            std::process::exit(1);
        })
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("md"))
        .filter_map(|p| parse_spec(&p))
        .collect();

    specs.sort_by(|a, b| a.name.cmp(&b.name));

    let name_width = specs.iter().map(|s| s.name.len()).max().unwrap_or(0);
    for spec in &specs {
        println!("{:<width$}  {}", spec.name, spec.status, width = name_width);
        if show_criteria {
            let n = spec.open_criteria.len();
            for (i, criterion) in spec.open_criteria.iter().enumerate() {
                let branch = if i + 1 == n { "  ┗━" } else { "  ┣━" };
                println!("{} {}", branch, criterion);
            }
        }
    }
}
