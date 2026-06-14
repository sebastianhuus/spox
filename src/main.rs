use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

const SKILL_MD: &str = include_str!("../skills/spox/SPOX.md");
const FORMAT_MD: &str = include_str!("../format.md");

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

fn find_project_root() -> PathBuf {
    let mut dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join(".git").exists() {
            return dir;
        }
        if !dir.pop() {
            return env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }
    }
}

enum SkillStatus {
    Current,
    Outdated,
    NotInstalled,
}

fn skill_status() -> SkillStatus {
    let path = find_project_root()
        .join(".claude")
        .join("skills")
        .join("spox")
        .join("SPOX.md");
    match fs::read_to_string(&path) {
        Ok(installed) if installed == SKILL_MD => SkillStatus::Current,
        Ok(_) => SkillStatus::Outdated,
        Err(_) => SkillStatus::NotInstalled,
    }
}

fn maybe_suggest_install() {
    match skill_status() {
        SkillStatus::Current => {}
        SkillStatus::Outdated => {
            eprintln!("\ntip: spox skill is out of date — run `spox skill install` to update");
        }
        SkillStatus::NotInstalled => {
            let show = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs() % 5 == 0)
                .unwrap_or(false);
            if show {
                eprintln!("\ntip: run `spox skill install` to add the spox skill to Claude Code");
            }
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

fn cmd_skill_install() {
    let root = find_project_root();
    let dest_dir = root.join(".claude").join("skills").join("spox");
    let dest_file = dest_dir.join("SPOX.md");

    fs::create_dir_all(&dest_dir).unwrap_or_else(|e| {
        eprintln!("error: could not create {}: {}", dest_dir.display(), e);
        std::process::exit(1);
    });

    fs::write(&dest_file, SKILL_MD).unwrap_or_else(|e| {
        eprintln!("error: could not write {}: {}", dest_file.display(), e);
        std::process::exit(1);
    });

    println!("installed: {}", dest_file.display());
}

fn cmd_init() {
    let spox_dir = env::current_dir()
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            std::process::exit(1);
        })
        .join(".spox");

    if spox_dir.exists() {
        eprintln!("error: .spox already exists at {}", spox_dir.display());
        std::process::exit(1);
    }

    fs::create_dir(&spox_dir).unwrap_or_else(|e| {
        eprintln!("error: could not create {}: {}", spox_dir.display(), e);
        std::process::exit(1);
    });

    let format_file = spox_dir.join(".format.md");
    fs::write(&format_file, FORMAT_MD).unwrap_or_else(|e| {
        eprintln!("error: could not write {}: {}", format_file.display(), e);
        std::process::exit(1);
    });

    println!("created: {}", spox_dir.display());
    println!("created: {}", format_file.display());
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [a] if a == "init" => {
            cmd_init();
        }
        [a, b] if a == "skill" && b == "install" => {
            cmd_skill_install();
        }
        _ => {
            if let Some(unknown) = args.iter().find(|a| *a != "--criteria" && *a != "-c") {
                eprintln!("error: unknown argument `{}`", unknown);
                eprintln!("usage: spox [-c|--criteria]");
                eprintln!("       spox skill install");
                std::process::exit(1);
            }

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
                .filter(|p| p.file_name().and_then(|n| n.to_str()).map(|n| !n.starts_with('.')).unwrap_or(false))
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("md"))
                .filter_map(|p| parse_spec(&p))
                .collect();

            if specs.is_empty() {
                eprintln!("no specs found in {}", spox_dir.display());
                maybe_suggest_install();
                return;
            }

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

            maybe_suggest_install();
        }
    }
}
