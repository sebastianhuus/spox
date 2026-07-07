use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use serde_json::{json, Value};

const SKILL_MD: &str = include_str!("../skills/spox/SKILL.md");
const SDD_MD: &str = include_str!("../skills/spox/sdd.md");
const SPEC_TEMPLATE_MD: &str = include_str!("../skills/spox/spec-template.md");
const CHECK_CHAIN_SH: &str = include_str!("../skills/spox/check-chain.sh");
const NEW_SPEC_SKILL_MD: &str = include_str!("../skills/new-spec/SKILL.md");
const FORMAT_MD: &str = include_str!("../format.md");

struct Spec {
    name: String,
    status: String,
    date: String,
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
    let root = find_project_root();
    let skill_dir = root.join(".claude").join("skills").join("spox");
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
            let new_spec_current = fs::read_to_string(
                root.join(".claude").join("skills").join("new-spec").join("SKILL.md"),
            )
            .map(|s| s == NEW_SPEC_SKILL_MD)
            .unwrap_or(false);
            if all_current && new_spec_current { SkillStatus::Current } else { SkillStatus::Outdated }
        }
        Ok(_) => SkillStatus::Outdated,
        Err(_) if old_path.exists() => SkillStatus::Outdated,
        Err(_) => SkillStatus::NotInstalled,
    }
}

fn maybe_suggest_install() {
    match skill_status() {
        SkillStatus::Current | SkillStatus::NotInstalled => {}
        SkillStatus::Outdated => {
            cmd_skill_install();
        }
    }
}

fn parse_spec(path: &std::path::Path) -> Option<Spec> {
    let name = path.file_stem()?.to_string_lossy().into_owned();
    let content = fs::read_to_string(path).ok()?;

    let mut status = "(no status)".to_string();
    let mut date = String::new();
    let mut status_seen = false;
    for line in content.lines() {
        if line.is_empty() || line.starts_with('#') {
            break;
        }
        if let Some(v) = line.strip_prefix("status: ") {
            status = v.trim().to_string();
            status_seen = true;
        } else if let Some(v) = line.strip_prefix("date: ") {
            date = v.trim().to_string();
        } else if !status_seen {
            break;
        }
    }

    let open_criteria = content
        .lines()
        .filter_map(|line| line.strip_prefix("- [ ] ").map(str::to_string))
        .collect();
    Some(Spec { name, status, date, open_criteria })
}

fn sync_format_md(spox_dir: &std::path::Path) {
    let path = spox_dir.join(".format.md");
    let root = spox_dir.parent().unwrap_or(spox_dir);
    let rel = path.strip_prefix(root).unwrap_or(&path);
    let status = match fs::read_to_string(&path) {
        Ok(s) if s == FORMAT_MD => None,
        Ok(_) => Some("updated"),
        Err(_) => Some("created"),
    };
    if let Some(status) = status {
        fs::write(&path, FORMAT_MD).unwrap_or_else(|e| {
            eprintln!("error: could not write {}: {}", path.display(), e);
            std::process::exit(1);
        });
        eprintln!("spox: {} {}", status, rel.display());
    }
}

fn require_spox_dir() -> PathBuf {
    let dir = find_spox_dir().unwrap_or_else(|| {
        eprintln!("error: no .spox directory found in this or any parent directory");
        std::process::exit(1);
    });
    sync_format_md(&dir);
    dir
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

fn criterion_label(text: &str) -> String {
    // FNV-1a 32-bit, XOR-folded to 16 bits → 4 lowercase hex chars.
    // Stable as long as criterion text doesn't change; survives reordering and partial checks.
    let mut h: u32 = 2166136261;
    for b in text.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16777619);
    }
    let folded = ((h >> 16) ^ (h & 0xFFFF)) as u16;
    format!("{:04x}", folded)
}

fn is_hex_label(s: &str) -> bool {
    s.len() == 4 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

fn spec_mtime(path: &std::path::Path) -> Option<u128> {
    fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_nanos())
}

fn cache_path(spox_dir: &std::path::Path, spec_name: &str) -> std::path::PathBuf {
    spox_dir.join(".cache").join(spec_name)
}

fn store_mtime_cache(spox_dir: &std::path::Path, spec_name: &str, spec_path: &std::path::Path) {
    let cache_dir = spox_dir.join(".cache");
    let _ = fs::create_dir_all(&cache_dir);
    if let Some(t) = spec_mtime(spec_path) {
        let _ = fs::write(cache_path(spox_dir, spec_name), t.to_string());
    }
}

enum CacheCheck { Valid, Missing, Stale }

fn check_mtime_cache(spox_dir: &std::path::Path, spec_name: &str, spec_path: &std::path::Path) -> CacheCheck {
    let stored = fs::read_to_string(cache_path(spox_dir, spec_name))
        .ok()
        .and_then(|s| s.trim().parse::<u128>().ok());
    match stored {
        None => CacheCheck::Missing,
        Some(stored_t) => match spec_mtime(spec_path) {
            Some(current_t) if current_t == stored_t => CacheCheck::Valid,
            _ => CacheCheck::Stale,
        },
    }
}

fn invalidate_cache(spox_dir: &std::path::Path, spec_name: &str) {
    let _ = fs::remove_file(cache_path(spox_dir, spec_name));
}

fn terminal_width() -> usize {
    use terminal_size::{Width, terminal_size};
    terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(80)
}

fn glow_available() -> bool {
    env::var_os("PATH")
        .map(|p| env::split_paths(&p).any(|dir| dir.join("glow").is_file()))
        .unwrap_or(false)
}

fn use_glow() -> bool {
    use std::io::IsTerminal;
    env::var_os("SPOX_NO_GLOW").is_none()
        && std::io::stdout().is_terminal()
        && glow_available()
}

fn print_via_glow(content: &str) -> bool {
    use std::io::Write;
    let width = terminal_width();
    let mut child = match std::process::Command::new("glow")
        .args(["--width", &width.to_string(), "-"])
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(content.as_bytes());
    }
    child.wait().map(|s| s.success()).unwrap_or(false)
}

fn print_criteria(open_criteria: &[String]) {
    let n = open_criteria.len();
    let labels: Vec<String> = open_criteria.iter().map(|c| criterion_label(c)).collect();
    let has_collision = {
        let mut seen = std::collections::HashSet::new();
        labels.iter().any(|l| !seen.insert(l.clone()))
    };
    let term_width = terminal_width();
    for (i, criterion) in open_criteria.iter().enumerate() {
        let is_last = i + 1 == n;
        let branch = if is_last { "  ┗━" } else { "  ┣━" };
        let prefix = if has_collision {
            format!("{} {}. [{}] ", branch, i + 1, labels[i])
        } else {
            format!("{} [{}] ", branch, labels[i])
        };
        // Continuation lines: ┃ at the branch column for non-last items, spaces for last
        let indent: String = if is_last {
            " ".repeat(prefix.chars().count())
        } else {
            format!("  ┃{}", " ".repeat(prefix.chars().count().saturating_sub(3)))
        };
        let text_width = term_width.saturating_sub(prefix.chars().count());
        let words: Vec<&str> = criterion.split_whitespace().collect();
        let mut line = String::new();
        let mut first = true;
        for word in &words {
            if line.is_empty() {
                line.push_str(word);
            } else if line.len() + 1 + word.len() <= text_width {
                line.push(' ');
                line.push_str(word);
            } else {
                if first {
                    println!("{}{}", prefix, line);
                    first = false;
                } else {
                    println!("{}{}", indent, line);
                }
                line = word.to_string();
            }
        }
        if first {
            println!("{}{}", prefix, line);
        } else {
            println!("{}{}", indent, line);
        }
    }
}

fn cmd_check_all(spec_name: &str) {
    let spox_dir = require_spox_dir();
    let spec_path = find_spec_path(&spox_dir, spec_name);

    match check_mtime_cache(&spox_dir, spec_name, &spec_path) {
        CacheCheck::Missing => {
            eprintln!("error: run `spox view -c {}` before checking — spec has not been read yet", spec_name);
            std::process::exit(1);
        }
        CacheCheck::Stale => {
            eprintln!("error: '{}' has changed since you last read it — run `spox view -c {}` to refresh", spec_name, spec_name);
            std::process::exit(1);
        }
        CacheCheck::Valid => {}
    }

    let content = read_spec_content(&spec_path);
    let lines: Vec<&str> = content.lines().collect();
    let open_count = lines.iter().filter(|l| l.starts_with("- [ ] ")).count();

    if open_count == 0 {
        println!("{}: no open criteria", spec_name);
        return;
    }

    let mut new_lines: Vec<String> = lines.iter().map(|l| {
        if l.starts_with("- [ ] ") {
            format!("- [x] {}", &l["- [ ] ".len()..])
        } else {
            l.to_string()
        }
    }).collect();

    if new_lines[0].starts_with("status: ") {
        new_lines[0] = "status: completed".to_string();
    }

    let trailing_newline = if content.ends_with('\n') { "\n" } else { "" };
    write_spec_content(&spec_path, &(new_lines.join("\n") + trailing_newline));
    invalidate_cache(&spox_dir, spec_name);

    println!("checked: {} — all {} open criteria done", spec_name, open_count);
    println!("status:  {} → completed", spec_name);
}

fn cmd_check(spec_name: &str, n_str: &str) {
    if n_str == "all" {
        return cmd_check_all(spec_name);
    }

    let use_label = is_hex_label(n_str);
    let n_pos: Option<usize> = if use_label {
        None
    } else {
        match n_str.parse::<usize>() {
            Ok(0) => {
                eprintln!("error: criterion index starts at 1");
                std::process::exit(1);
            }
            Ok(n) => Some(n),
            Err(_) => None,
        }
    };
    if !use_label && n_pos.is_none() {
        eprintln!("error: criterion must be a 4-char hex label (e.g. a3f2) from `spox view -c`, a position number, or 'all'");
        std::process::exit(1);
    }

    let spox_dir = require_spox_dir();
    let spec_path = find_spec_path(&spox_dir, spec_name);

    match check_mtime_cache(&spox_dir, spec_name, &spec_path) {
        CacheCheck::Missing => {
            eprintln!("error: run `spox view -c {}` before checking — spec has not been read yet", spec_name);
            std::process::exit(1);
        }
        CacheCheck::Stale => {
            eprintln!("error: '{}' has changed since you last read it — run `spox view -c {}` to refresh", spec_name, spec_name);
            std::process::exit(1);
        }
        CacheCheck::Valid => {}
    }

    let content = read_spec_content(&spec_path);
    let lines: Vec<&str> = content.lines().collect();
    let total_open = lines.iter().filter(|l| l.starts_with("- [ ] ")).count();

    let (target, criterion_text) = if use_label {
        let mut matches: Vec<(usize, String)> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if let Some(text) = line.strip_prefix("- [ ] ") {
                if criterion_label(text) == n_str {
                    matches.push((i, text.to_string()));
                }
            }
        }
        match matches.len() {
            0 => {
                eprintln!("error: no open criterion with label '{}' in spec '{}'", n_str, spec_name);
                std::process::exit(1);
            }
            1 => matches.remove(0),
            _ => {
                eprintln!("error: label '{}' matches {} criteria — use a position number instead", n_str, matches.len());
                std::process::exit(1);
            }
        }
    } else {
        let n = n_pos.unwrap();
        let mut open_count = 0;
        let mut found: Option<(usize, String)> = None;
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("- [ ] ") {
                open_count += 1;
                if open_count == n {
                    found = Some((i, line.strip_prefix("- [ ] ").unwrap_or("").to_string()));
                    break;
                }
            }
        }
        found.unwrap_or_else(|| {
            eprintln!(
                "error: spec '{}' has {} open criterion/criteria (you asked for #{})",
                spec_name, total_open, n
            );
            std::process::exit(1);
        })
    };

    let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    new_lines[target] = format!("- [x] {}", criterion_text);

    let was_last = total_open == 1;
    if was_last && new_lines[0].starts_with("status: ") {
        new_lines[0] = "status: completed".to_string();
    }

    let trailing_newline = if content.ends_with('\n') { "\n" } else { "" };
    write_spec_content(&spec_path, &(new_lines.join("\n") + trailing_newline));
    invalidate_cache(&spox_dir, spec_name);

    let label = criterion_label(&criterion_text);
    println!("checked: {} [{}] — {}", spec_name, label, criterion_text);
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

fn allow_check_cmd(project_dir: &PathBuf) {
    let claude_dir = project_dir.join(".claude");
    let settings_path = claude_dir.join("settings.json");

    let mut settings: Value = fs::read_to_string(&settings_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));

    let check_cmd = Value::String("Bash(spox check *)".to_string());
    let mut allow = settings["permissions"]["allow"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    if !allow.contains(&check_cmd) {
        allow.push(check_cmd);
        settings["permissions"]["allow"] = Value::Array(allow);

        if let Err(e) = fs::create_dir_all(&claude_dir) {
            eprintln!("warning: could not create .claude/: {}", e);
            return;
        }
        let json = serde_json::to_string_pretty(&settings).unwrap_or_default();
        fs::write(&settings_path, json + "\n").unwrap_or_else(|e| {
            eprintln!("warning: could not write settings.json: {}", e);
        });
        println!("settings: allowed Bash(spox check *)");
    }
}

fn merge_settings(root: &PathBuf, skill_dir: &PathBuf) {
    let settings_path = root.join(".claude").join("settings.json");

    let mut settings: Value = fs::read_to_string(&settings_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));

    // Merge permissions.allow
    let spox_cmds = [
        "Bash(spox list)",
        "Bash(spox list -c)",
        "Bash(spox list --criteria)",
        "Bash(spox view *)",
        "Bash(spox view -c *)",
        "Bash(spox view --criteria *)",
        "Bash(spox check *)",
        "Bash(spox check * all)",
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

fn write_skill_file(path: &PathBuf, content: &str) -> &'static str {
    let existing = fs::read_to_string(path).ok();
    let status = match existing {
        Some(ref s) if s == content => "unchanged",
        Some(_) => "updated",
        None => "installed",
    };
    if status != "unchanged" {
        fs::write(path, content).unwrap_or_else(|e| {
            eprintln!("error: could not write {}: {}", path.display(), e);
            std::process::exit(1);
        });
    }
    status
}

fn cmd_skill_install() {
    let root = find_project_root();
    let dest_dir = root.join(".claude").join("skills").join("spox");
    let dest_file = dest_dir.join("SKILL.md");

    fs::create_dir_all(&dest_dir).unwrap_or_else(|e| {
        eprintln!("error: could not create {}: {}", dest_dir.display(), e);
        std::process::exit(1);
    });

    let main_status = write_skill_file(&dest_file, SKILL_MD);

    let mut any_changed = main_status != "unchanged";
    for (name, content) in [("sdd.md", SDD_MD), ("spec-template.md", SPEC_TEMPLATE_MD), ("check-chain.sh", CHECK_CHAIN_SH)] {
        let path = dest_dir.join(name);
        if write_skill_file(&path, content) != "unchanged" {
            any_changed = true;
        }
    }

    let new_spec_dir = root.join(".claude").join("skills").join("new-spec");
    fs::create_dir_all(&new_spec_dir).unwrap_or_else(|e| {
        eprintln!("error: could not create {}: {}", new_spec_dir.display(), e);
        std::process::exit(1);
    });
    let new_spec_file = new_spec_dir.join("SKILL.md");
    let new_spec_status = write_skill_file(&new_spec_file, NEW_SPEC_SKILL_MD);
    if new_spec_status != "unchanged" {
        any_changed = true;
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

    if any_changed {
        if main_status != "unchanged" {
            let rel = dest_file.strip_prefix(&root).unwrap_or(&dest_file);
            eprintln!("spox: skill {} {}", main_status, rel.display());
        }
        if new_spec_status != "unchanged" {
            let rel = new_spec_file.strip_prefix(&root).unwrap_or(&new_spec_file);
            eprintln!("spox: skill {} {}", new_spec_status, rel.display());
        }
    } else {
        println!("skill: already up to date");
    }
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

    let project_dir = spox_dir.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| spox_dir.clone());

    // Add .spox/.cache/ to .gitignore if we're inside a git repo.
    let git_root = {
        let mut dir = project_dir.clone();
        loop {
            if dir.join(".git").exists() {
                break Some(dir);
            }
            if !dir.pop() {
                break None;
            }
        }
    };
    if let Some(root) = git_root {
        let gitignore = root.join(".gitignore");
        let entry = ".spox/.cache/";
        let existing = fs::read_to_string(&gitignore).unwrap_or_default();
        let already_present = existing.lines().any(|l| l.trim() == entry);
        if !already_present {
            use std::io::Write;
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&gitignore)
                .unwrap_or_else(|e| {
                    eprintln!("error: could not open {}: {}", gitignore.display(), e);
                    std::process::exit(1);
                });
            if !existing.is_empty() && !existing.ends_with('\n') {
                writeln!(file).unwrap_or(());
            }
            writeln!(file, "{}", entry).unwrap_or_else(|e| {
                eprintln!("error: could not write {}: {}", gitignore.display(), e);
                std::process::exit(1);
            });
            println!("updated: {}", gitignore.display());
        }
    }

    allow_check_cmd(&project_dir);
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

fn cmd_list(show_criteria: bool, filter: Option<&str>, active_only: bool) {
    let spox_dir = require_spox_dir();

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

    if let Some(name) = filter {
        specs.retain(|s| s.name == name);
        if specs.is_empty() {
            eprintln!("error: spec '{}' not found in {}", name, spox_dir.display());
            std::process::exit(1);
        }
    }

    if active_only {
        specs.retain(|s| {
            let lower = s.status.to_lowercase();
            status_group(&s.status) != 2 && !lower.contains("discard")
        });
    }

    if specs.is_empty() {
        if active_only {
            eprintln!("no active specs in {}", spox_dir.display());
        } else {
            eprintln!("no specs found in {}", spox_dir.display());
        }
        maybe_suggest_install();
        return;
    }

    specs.sort_by(|a, b| {
        status_group(&a.status)
            .cmp(&status_group(&b.status))
            .then(b.date.cmp(&a.date))
            .then(a.name.cmp(&b.name))
    });

    let name_width = specs.iter().map(|s| s.name.len()).max().unwrap_or(0);
    for spec in &specs {
        println!("{:<width$}  {}", spec.name, spec.status, width = name_width);
        if show_criteria {
            print_criteria(&spec.open_criteria);
        }
        let spec_path = spox_dir.join(format!("{}.md", spec.name));
        store_mtime_cache(&spox_dir, &spec.name, &spec_path);
    }

    maybe_suggest_install();
}

fn cmd_view_raw(spec_name: &str) {
    let spox_dir = require_spox_dir();
    let spec_path = find_spec_path(&spox_dir, spec_name);
    let content = read_spec_content(&spec_path);
    if use_glow() && print_via_glow(&content) {
        return;
    }
    print!("{}", content);
}

fn cmd_view_criteria(spec_name: &str) {
    let spox_dir = require_spox_dir();
    let spec_path = find_spec_path(&spox_dir, spec_name);
    let spec = parse_spec(&spec_path).unwrap_or_else(|| {
        eprintln!("error: could not parse spec '{}'", spec_name);
        std::process::exit(1);
    });
    println!("{:<width$}  {}", spec.name, spec.status, width = spec.name.len());
    print_criteria(&spec.open_criteria);
    store_mtime_cache(&spox_dir, spec_name, &spec_path);
    maybe_suggest_install();
}

fn cmd_help() {
    println!("usage: spox <command> [options]");
    println!();
    println!("Commands:");
    println!("  list [-c] [-a]         list all specs (-c: open criteria, -a: hide completed/discarded)");
    println!("  view [-c] <spec>       show a spec (raw file, or -c for criteria dashboard)");
    println!("  check <spec> <label>   check off a criterion by stable hex label");
    println!("  check <spec> all       check off all remaining open criteria");
    println!("  status <spec> <value>  set a spec's status field");
    println!("  init                   create .spox/ in the current directory");
    println!("  skill install          install the spox skill into .claude/skills/");
    println!("  version                print the installed version");
    println!("  help                   show this message");
}

fn main() {
    maybe_self_update();
    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [a] if a == "version" || a == "--version" || a == "-V" => {
            println!("spox {}", env!("CARGO_PKG_VERSION"));
        }
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
        [a] if a == "list" => {
            cmd_list(false, None, false);
        }
        [a, b] if a == "list" && (b == "-c" || b == "--criteria") => {
            cmd_list(true, None, false);
        }
        [a, b] if a == "list" && (b == "-a" || b == "--active") => {
            cmd_list(false, None, true);
        }
        [a, b, c] if a == "list" && (b == "-a" || b == "--active") && (c == "-c" || c == "--criteria") => {
            cmd_list(true, None, true);
        }
        [a, b, c] if a == "list" && (b == "-c" || b == "--criteria") && (c == "-a" || c == "--active") => {
            cmd_list(true, None, true);
        }
        [a, b] if a == "view" => {
            cmd_view_raw(b);
        }
        [a, b, c] if a == "view" && (b == "-c" || b == "--criteria") => {
            cmd_view_criteria(c);
        }
        [a] if a == "help" || a == "--help" || a == "-h" => {
            cmd_help();
        }
        _ => {
            // Deprecated positional forms (bare `spox` and `spox <spec>`) plus error handling.
            let show_criteria = args.iter().any(|a| a == "--criteria" || a == "-c");
            let positional: Vec<&str> = args.iter()
                .filter(|a| *a != "--criteria" && *a != "-c")
                .map(|s| s.as_str())
                .collect();

            // Known commands that didn't match their arm = wrong number of arguments.
            let known_cmds = ["check", "status", "skill", "list", "view", "init", "version", "help"];
            let first_pos = positional.first().copied();
            if positional.len() > 1 || first_pos.map(|p| known_cmds.contains(&p)).unwrap_or(false) {
                eprintln!("error: unexpected arguments");
                eprintln!("run `spox help` for usage");
                std::process::exit(1);
            }

            match first_pos {
                None => {
                    eprintln!("error: bare `spox` is no longer supported — use `spox list`");
                    std::process::exit(1);
                }
                Some(name) => {
                    if show_criteria {
                        eprintln!("error: `spox -c {}` is no longer supported — use `spox view -c {}`", name, name);
                    } else {
                        eprintln!("error: `spox {}` is no longer supported — use `spox view {}`", name, name);
                    }
                    std::process::exit(1);
                }
            }
        }
    }
}
