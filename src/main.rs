use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use serde_json::{json, Value};

const SKILL_MD: &str = include_str!("../skills/spox/SKILL.md");
const SDD_MD: &str = include_str!("../skills/spox/sdd.md");
const SPEC_TEMPLATE_MD: &str = include_str!("../skills/spox/spec-template.md");
const CHECK_CHAIN_SH: &str = include_str!("../skills/spox/check-chain.sh");
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
    let skill_dir = find_project_root().join(".claude").join("skills").join("spox");
    let new_path = skill_dir.join("SKILL.md");
    let old_path = skill_dir.join("SPOX.md");
    match fs::read_to_string(&new_path) {
        Ok(installed) if installed == SKILL_MD => {
            let supporting = [
                ("sdd.md", SDD_MD),
                ("spec-template.md", SPEC_TEMPLATE_MD),
                ("check-chain.sh", CHECK_CHAIN_SH),
            ];
            let all_current = supporting.iter().all(|(name, expected)| {
                fs::read_to_string(skill_dir.join(name))
                    .map(|s| s == *expected)
                    .unwrap_or(false)
            });
            if all_current { SkillStatus::Current } else { SkillStatus::Outdated }
        }
        Ok(_) => SkillStatus::Outdated,
        Err(_) if old_path.exists() => SkillStatus::Outdated,
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

fn require_spox_dir() -> PathBuf {
    find_spox_dir().unwrap_or_else(|| {
        eprintln!("error: no .spox directory found in this or any parent directory");
        std::process::exit(1);
    })
}

fn find_spec_path(spox_dir: &std::path::Path, spec_name: &str) -> PathBuf {
    let path = spox_dir.join(format!("{}.md", spec_name));
    if !path.exists() {
        eprintln!("error: spec '{}' not found in {}", spec_name, spox_dir.display());
        std::process::exit(1);
    }
    path
}

fn read_spec_content(path: &std::path::Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        std::process::exit(1);
    })
}

fn write_spec_content(path: &std::path::Path, content: &str) {
    fs::write(path, content).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        std::process::exit(1);
    });
}

fn cmd_check(spec_name: &str, n_str: &str) {
    let n: usize = n_str.parse().unwrap_or_else(|_| {
        eprintln!("error: criterion index must be a positive integer");
        std::process::exit(1);
    });
    if n == 0 {
        eprintln!("error: criterion index starts at 1");
        std::process::exit(1);
    }

    let spox_dir = require_spox_dir();
    let spec_path = find_spec_path(&spox_dir, spec_name);
    let content = read_spec_content(&spec_path);

    let lines: Vec<&str> = content.lines().collect();
    let total_open = lines.iter().filter(|l| l.starts_with("- [ ] ")).count();

    let mut open_count = 0;
    let mut target_idx: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("- [ ] ") {
            open_count += 1;
            if open_count == n {
                target_idx = Some(i);
                break;
            }
        }
    }

    let target = target_idx.unwrap_or_else(|| {
        eprintln!(
            "error: spec '{}' has {} open criterion/criteria (you asked for #{})",
            spec_name, total_open, n
        );
        std::process::exit(1);
    });

    let criterion_text = lines[target].strip_prefix("- [ ] ").unwrap_or("").to_string();
    let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    new_lines[target] = format!("- [x] {}", criterion_text);

    let was_last = total_open == 1;
    if was_last && new_lines[0].starts_with("status: ") {
        new_lines[0] = "status: completed".to_string();
    }

    let trailing_newline = if content.ends_with('\n') { "\n" } else { "" };
    write_spec_content(&spec_path, &(new_lines.join("\n") + trailing_newline));

    println!("checked: {} #{} — {}", spec_name, n, criterion_text);
    if was_last {
        println!("status:  {} → completed (all criteria done)", spec_name);
    }
}

fn cmd_set_status(spec_name: &str, value: &str) {
    let spox_dir = require_spox_dir();
    let spec_path = find_spec_path(&spox_dir, spec_name);
    let content = read_spec_content(&spec_path);

    let old_status = content
        .lines()
        .next()
        .and_then(|l| l.strip_prefix("status: "))
        .unwrap_or("(none)")
        .trim()
        .to_string();

    let new_content = if content.starts_with("status: ") {
        let rest_start = content.find('\n').unwrap_or(content.len());
        format!("status: {}{}", value, &content[rest_start..])
    } else {
        format!("status: {}\n{}", value, content)
    };

    write_spec_content(&spec_path, &new_content);
    println!("{}: {} → {}", spec_name, old_status, value);
}

fn merge_settings(root: &PathBuf, skill_dir: &PathBuf) {
    let settings_path = root.join(".claude").join("settings.json");

    let mut settings: Value = fs::read_to_string(&settings_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));

    // Merge permissions.allow
    let spox_cmds = [
        "Bash(spox)",
        "Bash(spox -c)",
        "Bash(spox --criteria)",
        "Bash(spox check *)",
        "Bash(spox status * *)",
        "Bash(spox init)",
        "Bash(spox skill install)",
    ];
    let allow = settings["permissions"]["allow"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut added_cmds = Vec::new();
    let mut new_allow = allow.clone();
    for cmd in spox_cmds {
        let v = Value::String(cmd.to_string());
        if !allow.contains(&v) {
            new_allow.push(v);
            added_cmds.push(cmd);
        }
    }
    settings["permissions"]["allow"] = Value::Array(new_allow);

    // Merge hooks.PreToolUse — add our hook if not already present
    let hook_cmd = skill_dir.join("check-chain.sh").to_string_lossy().to_string();
    let pre_tool_use = settings["hooks"]["PreToolUse"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let already_hooked = pre_tool_use.iter().any(|entry| {
        entry["hooks"]
            .as_array()
            .map(|hooks| hooks.iter().any(|h| h["command"] == hook_cmd))
            .unwrap_or(false)
    });
    let hook_added = if !already_hooked {
        let entry = json!({
            "matcher": "Bash",
            "hooks": [{ "type": "command", "command": hook_cmd, "timeout": 5 }]
        });
        let mut new_hooks = pre_tool_use;
        new_hooks.push(entry);
        settings["hooks"]["PreToolUse"] = Value::Array(new_hooks);
        true
    } else {
        false
    };

    let json = serde_json::to_string_pretty(&settings).unwrap_or_default();
    fs::write(&settings_path, json + "\n").unwrap_or_else(|e| {
        eprintln!("warning: could not write settings.json: {}", e);
    });

    if !added_cmds.is_empty() {
        println!("settings: added {} allowed command(s) to permissions.allow", added_cmds.len());
    }
    if hook_added {
        println!("settings: registered check-chain hook in hooks.PreToolUse");
    }
    if added_cmds.is_empty() && !hook_added {
        println!("settings: already up to date");
    }
}

fn cmd_skill_install() {
    let root = find_project_root();
    let dest_dir = root.join(".claude").join("skills").join("spox");
    let dest_file = dest_dir.join("SKILL.md");

    fs::create_dir_all(&dest_dir).unwrap_or_else(|e| {
        eprintln!("error: could not create {}: {}", dest_dir.display(), e);
        std::process::exit(1);
    });

    fs::write(&dest_file, SKILL_MD).unwrap_or_else(|e| {
        eprintln!("error: could not write {}: {}", dest_file.display(), e);
        std::process::exit(1);
    });

    for (name, content) in [("sdd.md", SDD_MD), ("spec-template.md", SPEC_TEMPLATE_MD), ("check-chain.sh", CHECK_CHAIN_SH)] {
        let path = dest_dir.join(name);
        fs::write(&path, content).unwrap_or_else(|e| {
            eprintln!("error: could not write {}: {}", path.display(), e);
            std::process::exit(1);
        });
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let hook_path = dest_dir.join("check-chain.sh");
        if let Ok(meta) = fs::metadata(&hook_path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms).unwrap_or_else(|e| {
                eprintln!("warning: could not chmod check-chain.sh: {}", e);
            });
        }
    }

    let old_file = dest_dir.join("SPOX.md");
    if old_file.exists() {
        fs::remove_file(&old_file).unwrap_or_else(|e| {
            eprintln!("warning: could not remove {}: {}", old_file.display(), e);
        });
        println!("migrated: removed {}", old_file.display());
    }

    println!("installed: {}", dest_file.display());
    merge_settings(&root, &dest_dir);
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

fn status_group(status: &str) -> u8 {
    let s = status.to_lowercase();
    if s.contains("partial") || s.contains("progress") || s.contains("wip") || s.contains("active") {
        0
    } else if s.contains("complet") || s.contains("done") || s.contains("finish") || s.contains("implement") {
        2
    } else if s.contains("draft") || s.contains("plan") || s.contains("todo") || s.contains("backlog") {
        1
    } else {
        0
    }
}

fn spox_repo_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let real = std::fs::canonicalize(&exe).ok()?;
    // binary lives at $REPO/target/release/spox — go up three levels
    real.parent()?.parent()?.parent().map(PathBuf::from)
}

fn last_check_path() -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".cache").join("spox").join("last_check"))
}

fn should_check_updates() -> bool {
    let path = match last_check_path() {
        Some(p) => p,
        None => return false,
    };
    fs::metadata(&path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|d| d.as_secs() > 86400)
        .unwrap_or(true)
}

fn mark_checked() {
    if let Some(path) = last_check_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, b"");
    }
}

fn git_head(repo: &PathBuf) -> Option<Vec<u8>> {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .ok()
        .map(|o| o.stdout)
}

fn maybe_self_update() {
    if !should_check_updates() {
        return;
    }
    mark_checked();

    let repo = match spox_repo_dir() {
        Some(d) if d.join(".git").exists() => d,
        _ => return,
    };

    let before = git_head(&repo);

    let ok = std::process::Command::new("git")
        .args(["pull", "--quiet"])
        .current_dir(&repo)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !ok {
        return;
    }

    if before.is_some() && git_head(&repo) != before {
        eprintln!("spox: new version pulled, rebuilding...");
        let _ = std::process::Command::new("cargo")
            .args(["build", "--release", "--quiet"])
            .current_dir(&repo)
            .status();
        eprintln!("spox: updated");
    }
}

fn main() {
    maybe_self_update();
    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [a] if a == "init" => {
            cmd_init();
        }
        [a, b] if a == "skill" && b == "install" => {
            cmd_skill_install();
        }
        [a, b, c] if a == "check" => {
            cmd_check(b, c);
        }
        [a, b, c] if a == "status" => {
            cmd_set_status(b, c);
        }
        _ => {
            if let Some(unknown) = args.iter().find(|a| *a != "--criteria" && *a != "-c") {
                eprintln!("error: unknown argument `{}`", unknown);
                eprintln!("usage: spox [-c|--criteria]");
                eprintln!("       spox check <spec> <n>");
                eprintln!("       spox status <spec> <value>");
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

            specs.sort_by(|a, b| {
                status_group(&a.status)
                    .cmp(&status_group(&b.status))
                    .then(a.name.cmp(&b.name))
            });

            let name_width = specs.iter().map(|s| s.name.len()).max().unwrap_or(0);
            for spec in &specs {
                println!("{:<width$}  {}", spec.name, spec.status, width = name_width);
                if show_criteria {
                    let n = spec.open_criteria.len();
                    for (i, criterion) in spec.open_criteria.iter().enumerate() {
                        let branch = if i + 1 == n { "  ┗━" } else { "  ┣━" };
                        println!("{} {}. {}", branch, i + 1, criterion);
                    }
                }
            }

            maybe_suggest_install();
        }
    }
}
