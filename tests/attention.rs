use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const AGENT_LINE: &str =
    "Agent: mention this briefly at a natural moment, offer to go through it, and do not change any spec without asking.";

struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let root = std::env::temp_dir().join(format!("spox-attn-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("home/.cache/spox")).unwrap();
        // Canonicalize so paths match what spox sees (macOS /tmp -> /private/tmp).
        let root = fs::canonicalize(&root).unwrap();
        Sandbox { root }
    }

    fn dir(&self, rel: &str) -> PathBuf {
        let p = self.root.join("work").join(rel);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn home(&self) -> PathBuf {
        self.root.join("home")
    }

    // A fresh last_check keeps the daily self-update from running in tests that
    // don't care about it.
    fn fresh_last_check(&self) {
        fs::write(self.home().join(".cache/spox/last_check"), b"").unwrap();
    }

    fn git(&self, dir: &Path, args: &[&str], unix_time: Option<u64>) {
        let date = unix_time.map(|t| format!("{} +0000", t));
        let mut cmd = Command::new("git");
        cmd.args(["-c", "user.name=t", "-c", "user.email=t@example.com", "-c", "commit.gpgsign=false"])
            // No background maintenance after a commit: it touches .git while
            // the read-only test is snapshotting the tree.
            .args(["-c", "gc.auto=0", "-c", "maintenance.auto=false"])
            .args(args)
            .current_dir(dir)
            .env("HOME", self.home());
        if let Some(d) = &date {
            cmd.env("GIT_AUTHOR_DATE", d).env("GIT_COMMITTER_DATE", d);
        }
        let o = cmd.output().unwrap();
        assert!(o.status.success(), "git {:?}: {}", args, String::from_utf8_lossy(&o.stderr));
    }

    fn git_init(&self, dir: &Path) {
        self.git(dir, &["init", "-q"], None);
    }

    // Write `<project>/.spox/<name>.md`; returns its path.
    fn spec(&self, project: &Path, name: &str, status: &str, open: usize) -> PathBuf {
        let spox = project.join(".spox");
        fs::create_dir_all(&spox).unwrap();
        // Pre-synced so spox has nothing to announce.
        fs::write(spox.join(".format.md"), include_str!("../format.md")).unwrap();
        let mut body = format!("status: {}\ndate: 2026-01-01\n\n# T\n\n## Acceptance criteria\n", status);
        for i in 0..open {
            body.push_str(&format!("- [ ] criterion {}\n", i));
        }
        body.push_str("- [x] done one\n");
        let path = spox.join(format!("{}.md", name));
        fs::write(&path, body).unwrap();
        path
    }

    // Commit `path` as if it was committed `days` days (plus an hour) ago.
    fn commit(&self, repo: &Path, path: &Path, days: u64) {
        let t = now() - days * 86400 - 3600;
        self.git(repo, &["add", path.to_str().unwrap()], None);
        self.git(repo, &["commit", "-q", "-m", "c"], Some(t));
    }

    fn attention(&self, cwd: &Path, args: &[&str]) -> Output {
        self.attention_env(cwd, args, &[])
    }

    fn attention_env(&self, cwd: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_spox"));
        cmd.arg("attention")
            .args(args)
            .current_dir(cwd)
            .env("HOME", self.home())
            .env_remove("SPOX_PROJECT")
            .env_remove("SPOX_ATTENTION_SCOPE")
            .env("SPOX_NO_GLOW", "1");
        for (k, v) in env {
            cmd.env(k, v);
        }
        cmd.output().unwrap()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn assert_silent(o: &Output) {
    assert!(o.status.success(), "exit: {:?} stderr: {}", o.status, err(o));
    assert_eq!(out(o), "");
    assert_eq!(err(o), "");
}

#[test]
fn nothing_to_report_is_silent() {
    let sb = Sandbox::new("nothing");
    sb.fresh_last_check();
    let repo = sb.dir("repo");
    sb.git_init(&repo);
    for (name, status) in [("a", "draft"), ("b", "completed"), ("c", "discarded")] {
        let p = sb.spec(&repo, name, status, 1);
        sb.commit(&repo, &p, 30);
    }
    assert_silent(&sb.attention(&repo, &[]));
}

#[test]
fn no_spox_anywhere_is_silent_and_does_not_search_below() {
    let sb = Sandbox::new("nospox");
    sb.fresh_last_check();
    let top = sb.dir("mono");
    sb.git_init(&top);
    // Sub-project with an ongoing spec, but the top folder is not set up.
    let sub = sb.dir("mono/api");
    sb.spec(&sub, "x", "ongoing", 1);
    assert_silent(&sb.attention(&top, &[]));
    // Inside the sub-project it does report.
    let o = sb.attention(&sub, &[]);
    assert!(out(&o).starts_with("spox attention: 1 spec needs a look\n"), "{}", out(&o));
}

#[test]
fn report_lists_oldest_first_with_exact_format() {
    let sb = Sandbox::new("format");
    sb.fresh_last_check();
    let repo = sb.dir("repo");
    sb.git_init(&repo);
    let young = sb.spec(&repo, "young", "ongoing", 2);
    sb.commit(&repo, &young, 3);
    let old = sb.spec(&repo, "old", "pending-verification", 1);
    sb.commit(&repo, &old, 40);
    let mid = sb.spec(&repo, "mid", "ongoing", 0);
    sb.commit(&repo, &mid, 10);
    // Not reported.
    let done = sb.spec(&repo, "done", "completed", 0);
    sb.commit(&repo, &done, 90);

    let o = sb.attention(&repo, &[]);
    assert!(o.status.success());
    assert_eq!(err(&o), "");
    let expected = format!(
        "spox attention: 3 specs need a look\n\
         repo\n\
         \x20 ┣━ old  pending-verification  40d  1 open\n\
         \x20 ┣━ mid  ongoing  10d  0 open\n\
         \x20 ┗━ young  ongoing  3d  2 open\n\
         {}\n",
        AGENT_LINE
    );
    assert_eq!(out(&o), expected);

    // Stable for identical input.
    assert_eq!(out(&sb.attention(&repo, &[])), expected);
}

#[test]
fn more_than_five_are_capped() {
    let sb = Sandbox::new("cap");
    sb.fresh_last_check();
    let repo = sb.dir("repo");
    sb.git_init(&repo);
    for i in 1..=7u64 {
        let p = sb.spec(&repo, &format!("s{}", i), "ongoing", 1);
        sb.commit(&repo, &p, i * 5);
    }
    let o = out(&sb.attention(&repo, &[]));
    let lines: Vec<&str> = o.lines().collect();
    assert_eq!(lines[0], "spox attention: 7 specs need a look");
    assert_eq!(lines[1], "repo");
    // Oldest five: s7 (35d) .. s3 (15d).
    assert_eq!(lines[2], "  ┣━ s7  ongoing  35d  1 open");
    assert_eq!(lines[5], "  ┣━ s4  ongoing  20d  1 open");
    assert_eq!(lines[6], "  ┗━ s3  ongoing  15d  1 open");
    assert_eq!(lines[7], "…and 2 more");
    assert_eq!(lines[8], AGENT_LINE);
    assert_eq!(lines.len(), 9);
}

#[test]
fn singular_count_line() {
    let sb = Sandbox::new("singular");
    sb.fresh_last_check();
    let repo = sb.dir("repo");
    sb.git_init(&repo);
    let p = sb.spec(&repo, "only", "ongoing", 1);
    sb.commit(&repo, &p, 2);
    assert!(out(&sb.attention(&repo, &[])).starts_with("spox attention: 1 spec needs a look\n"));
}

#[test]
fn untracked_and_modified_specs_are_uncommitted() {
    let sb = Sandbox::new("dirty");
    sb.fresh_last_check();
    let repo = sb.dir("repo");
    sb.git_init(&repo);
    let tracked = sb.spec(&repo, "tracked", "ongoing", 1);
    sb.commit(&repo, &tracked, 20);
    let modified = sb.spec(&repo, "modified", "ongoing", 1);
    sb.commit(&repo, &modified, 50);
    fs::write(&modified, format!("{}\nmore\n", fs::read_to_string(&modified).unwrap())).unwrap();
    sb.spec(&repo, "zeta-new", "pending-verification", 1);
    sb.spec(&repo, "alpha-new", "ongoing", 1);

    let o = out(&sb.attention(&repo, &[]));
    let expected = format!(
        "spox attention: 4 specs need a look\n\
         repo\n\
         \x20 ┣━ tracked  ongoing  20d  1 open\n\
         \x20 ┣━ alpha-new  ongoing  uncommitted  1 open\n\
         \x20 ┣━ modified  ongoing  uncommitted  1 open\n\
         \x20 ┗━ zeta-new  pending-verification  uncommitted  1 open\n\
         {}\n",
        AGENT_LINE
    );
    assert_eq!(o, expected);
}

#[test]
fn many_uncommitted_are_capped() {
    let sb = Sandbox::new("dirtycap");
    sb.fresh_last_check();
    let repo = sb.dir("repo");
    sb.git_init(&repo);
    for i in 1..=7 {
        sb.spec(&repo, &format!("n{}", i), "ongoing", 1);
    }
    let o = out(&sb.attention(&repo, &[]));
    assert!(o.starts_with("spox attention: 7 specs need a look\nrepo\n"), "{}", o);
    assert!(o.contains("  ┣━ n1  ongoing  uncommitted  1 open\n"), "{}", o);
    assert!(o.contains("  ┗━ n5  ongoing  uncommitted  1 open\n…and 2 more\n"), "{}", o);
    assert!(!o.contains("n6"), "{}", o);
}

#[test]
fn without_git_falls_back_to_mtime_and_nothing_is_uncommitted() {
    let sb = Sandbox::new("nogit");
    sb.fresh_last_check();
    let dir = sb.dir("plain");
    let p = sb.spec(&dir, "old", "ongoing", 1);
    let f = fs::File::options().write(true).open(&p).unwrap();
    f.set_modified(SystemTime::now() - Duration::from_secs(12 * 86400 + 3600)).unwrap();
    drop(f);
    sb.spec(&dir, "fresh", "pending-verification", 1);

    let o = out(&sb.attention(&dir, &[]));
    let expected = format!(
        "spox attention: 2 specs need a look\n\
         plain\n\
         \x20 ┣━ old  ongoing  12d  1 open\n\
         \x20 ┗━ fresh  pending-verification  0d  1 open\n\
         {}\n",
        AGENT_LINE
    );
    assert_eq!(o, expected);
}

// A monorepo: set-up top folder plus two sub-projects, each its own git repo
// state living in the one top-level repo.
fn monorepo(sb: &Sandbox) -> PathBuf {
    let top = sb.dir("mono");
    sb.git_init(&top);
    let top_spec = sb.spec(&top, "orchestration", "ongoing", 1);
    sb.commit(&top, &top_spec, 5);
    let api = sb.dir("mono/services/api");
    let api_spec = sb.spec(&api, "auth", "ongoing", 2);
    sb.commit(&top, &api_spec, 30);
    let web = sb.dir("mono/services/web");
    let web_spec = sb.spec(&web, "ui", "pending-verification", 1);
    sb.commit(&top, &web_spec, 15);
    top
}

#[test]
fn top_folder_with_spox_reports_all_subprojects_with_prefixes() {
    let sb = Sandbox::new("mono-all");
    sb.fresh_last_check();
    let top = monorepo(&sb);
    let o = out(&sb.attention(&top, &[]));
    let expected = format!(
        "spox attention: 3 specs need a look\n\
         services/api\n\
         \x20 ┗━ auth  ongoing  30d  2 open\n\
         services/web\n\
         \x20 ┗━ ui  pending-verification  15d  1 open\n\
         mono\n\
         \x20 ┗━ orchestration  ongoing  5d  1 open\n\
         {}\n",
        AGENT_LINE
    );
    assert_eq!(o, expected);
}

#[test]
fn inside_subproject_only_that_project_is_scanned() {
    let sb = Sandbox::new("mono-sub");
    sb.fresh_last_check();
    let top = monorepo(&sb);
    let o = out(&sb.attention(&top.join("services/api"), &[]));
    let expected = format!(
        "spox attention: 1 spec needs a look\n\
         api\n\
         \x20 ┗━ auth  ongoing  30d  2 open\n\
         {}\n",
        AGENT_LINE
    );
    assert_eq!(o, expected);
}

#[test]
fn nearest_scope_skips_subprojects_via_flag_and_env() {
    let sb = Sandbox::new("mono-nearest");
    sb.fresh_last_check();
    let top = monorepo(&sb);
    let expected = format!(
        "spox attention: 1 spec needs a look\n\
         mono\n\
         \x20 ┗━ orchestration  ongoing  5d  1 open\n\
         {}\n",
        AGENT_LINE
    );
    assert_eq!(out(&sb.attention(&top, &["--scope", "nearest"])), expected);
    assert_eq!(out(&sb.attention(&top, &["--scope=nearest"])), expected);
    assert_eq!(out(&sb.attention_env(&top, &[], &[("SPOX_ATTENTION_SCOPE", "nearest")])), expected);
    // The flag wins over the env var.
    let o = out(&sb.attention_env(&top, &["--scope", "all"], &[("SPOX_ATTENTION_SCOPE", "nearest")]));
    assert!(o.starts_with("spox attention: 3 specs need a look\n"), "{}", o);
}

#[test]
fn invalid_scope_is_a_usage_error() {
    let sb = Sandbox::new("badscope");
    sb.fresh_last_check();
    let repo = sb.dir("repo");
    let o = sb.attention(&repo, &["--scope", "everything"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).starts_with("error:"), "{}", err(&o));
    assert_eq!(out(&o), "");

    let o = sb.attention_env(&repo, &[], &[("SPOX_ATTENTION_SCOPE", "bogus")]);
    assert_eq!(o.status.code(), Some(1));

    let o = sb.attention(&repo, &["--scope"]);
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn project_option_sets_the_starting_directory() {
    let sb = Sandbox::new("dashc");
    sb.fresh_last_check();
    let top = monorepo(&sb);
    let elsewhere = sb.dir("elsewhere");
    let o = Command::new(env!("CARGO_BIN_EXE_spox"))
        .args(["-C", top.join("services/web").to_str().unwrap(), "attention"])
        .current_dir(&elsewhere)
        .env("HOME", sb.home())
        .env_remove("SPOX_PROJECT")
        .output()
        .unwrap();
    assert!(out(&o).starts_with("spox attention: 1 spec needs a look\nweb\n  ┗━ ui  pending-verification  15d"), "{}", out(&o));
}

#[test]
fn attention_is_read_only_and_never_self_updates() {
    let sb = Sandbox::new("readonly");
    // Deliberately NO last_check: any other command would try the daily
    // self-update and write one.
    let last_check = sb.home().join(".cache/spox/last_check");
    let repo = sb.dir("repo");
    sb.git_init(&repo);
    let p = sb.spec(&repo, "a", "ongoing", 1);
    sb.commit(&repo, &p, 9);
    // A stale .format.md would be rewritten (and announced) by other commands.
    let format = repo.join(".spox/.format.md");
    fs::write(&format, "stale").unwrap();
    let before = snapshot(&repo);

    let o = sb.attention(&repo, &[]);
    assert!(o.status.success());
    assert_eq!(err(&o), "");
    assert!(!last_check.exists(), "self-update check must not run");
    assert_eq!(fs::read_to_string(&format).unwrap(), "stale");
    let after = snapshot(&repo);
    let changed: Vec<_> = after.iter().filter(|e| !before.contains(e)).chain(before.iter().filter(|e| !after.contains(e))).collect();
    assert!(changed.is_empty(), "files under the project changed: {:?}", changed);
}

fn snapshot(dir: &Path) -> Vec<(PathBuf, u64, SystemTime)> {
    let mut v = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap().flatten() {
            let m = e.metadata().unwrap();
            if m.is_dir() {
                stack.push(e.path());
            }
            v.push((e.path(), m.len(), m.modified().unwrap()));
        }
    }
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

// ---- `--hook` mode ----

fn hook_state_path(sb: &Sandbox) -> PathBuf {
    sb.home().join(".cache/spox/attention_hook")
}

// A git project with one committed, reportable spec.
fn reportable_project(sb: &Sandbox, name: &str) -> PathBuf {
    let repo = sb.dir(name);
    sb.git_init(&repo);
    let p = sb.spec(&repo, "a", "ongoing", 1);
    sb.commit(&repo, &p, 9);
    repo
}

fn state_lines(sb: &Sandbox) -> Vec<(u64, String)> {
    fs::read_to_string(hook_state_path(sb))
        .unwrap_or_default()
        .lines()
        .map(|l| {
            let (t, p) = l.split_once('\t').expect("tab-separated line");
            (t.parse().unwrap(), p.to_string())
        })
        .collect()
}

#[test]
fn hook_when_due_prints_the_same_report_and_records_the_time() {
    let sb = Sandbox::new("hook-due");
    let repo = reportable_project(&sb, "repo");
    let plain = out(&sb.attention(&repo, &[]));
    assert!(!plain.is_empty());
    assert!(!hook_state_path(&sb).exists(), "plain attention must not create state");

    let o = sb.attention(&repo, &["--hook"]);
    assert!(o.status.success());
    assert_eq!(err(&o), "");
    assert_eq!(out(&o), plain, "hook output must equal plain attention output");

    let lines = state_lines(&sb);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].1, repo.display().to_string());
    assert!(now().abs_diff(lines[0].0) < 10, "recorded time should be now");
    // The self-update check must not have run (it would create last_check).
    assert!(!sb.home().join(".cache/spox/last_check").exists());
}

#[test]
fn hook_when_not_due_is_silent_and_leaves_the_state_alone() {
    let sb = Sandbox::new("hook-notdue");
    let repo = reportable_project(&sb, "repo");
    assert!(!out(&sb.attention(&repo, &["--hook"])).is_empty());
    let before = fs::read_to_string(hook_state_path(&sb)).unwrap();

    let o = sb.attention(&repo, &["--hook"]);
    assert!(o.status.success());
    assert_eq!((out(&o), err(&o)), (String::new(), String::new()));
    assert_eq!(fs::read_to_string(hook_state_path(&sb)).unwrap(), before);
}

#[test]
fn hook_is_due_again_after_24_hours_and_drops_stale_lines() {
    let sb = Sandbox::new("hook-24h");
    let repo = reportable_project(&sb, "repo");
    let key = repo.display().to_string();
    fs::create_dir_all(hook_state_path(&sb).parent().unwrap()).unwrap();
    let t = now();
    fs::write(
        hook_state_path(&sb),
        format!(
            "{}\t{}\n{}\t/some/old/project\n{}\t/some/recent/project\n",
            t - 25 * 3600,
            key,
            t - 30 * 3600,
            t - 3600
        ),
    )
    .unwrap();

    assert!(!out(&sb.attention(&repo, &["--hook"])).is_empty());
    let lines = state_lines(&sb);
    assert_eq!(lines.len(), 2, "{:?}", lines);
    assert!(lines.iter().any(|(ts, p)| *p == key && now().abs_diff(*ts) < 10));
    assert!(lines.iter().any(|(_, p)| p == "/some/recent/project"));
    assert!(!lines.iter().any(|(_, p)| p == "/some/old/project"));
}

#[test]
fn hook_treats_a_recorded_time_in_the_future_as_due() {
    let sb = Sandbox::new("hook-future");
    let repo = reportable_project(&sb, "repo");
    fs::create_dir_all(hook_state_path(&sb).parent().unwrap()).unwrap();
    fs::write(hook_state_path(&sb), format!("{}\t{}\n", now() + 7200, repo.display())).unwrap();
    assert!(!out(&sb.attention(&repo, &["--hook"])).is_empty());
}

#[test]
fn hook_with_unwritable_state_prints_nothing() {
    let sb = Sandbox::new("hook-unwritable");
    let repo = reportable_project(&sb, "repo");
    // `~/.cache` is a regular file, so the state directory cannot be created.
    fs::remove_dir_all(sb.home().join(".cache")).unwrap();
    fs::write(sb.home().join(".cache"), "not a directory").unwrap();

    let o = sb.attention(&repo, &["--hook"]);
    assert!(o.status.success());
    assert_eq!((out(&o), err(&o)), (String::new(), String::new()));
    // Plain attention does not depend on the state and still prints.
    assert!(!out(&sb.attention(&repo, &[])).is_empty());
}

#[test]
fn hook_is_silent_and_records_nothing_without_spox_or_with_nothing_to_report() {
    let sb = Sandbox::new("hook-quiet");
    let plain_dir = sb.dir("plain");
    let o = sb.attention(&plain_dir, &["--hook"]);
    assert!(o.status.success());
    assert_eq!((out(&o), err(&o)), (String::new(), String::new()));

    let repo = sb.dir("done");
    sb.git_init(&repo);
    let p = sb.spec(&repo, "x", "completed", 1);
    sb.commit(&repo, &p, 30);
    let o = sb.attention(&repo, &["--hook"]);
    assert!(o.status.success());
    assert_eq!((out(&o), err(&o)), (String::new(), String::new()));

    assert!(!hook_state_path(&sb).exists(), "nothing shown, so nothing recorded");

    // Because nothing was recorded, a report that appears later is not delayed.
    let p = sb.spec(&repo, "y", "ongoing", 1);
    sb.commit(&repo, &p, 12);
    assert!(!out(&sb.attention(&repo, &["--hook"])).is_empty());
}

#[test]
fn hook_throttles_each_project_independently() {
    let sb = Sandbox::new("hook-two");
    let one = reportable_project(&sb, "one");
    let two = reportable_project(&sb, "two");
    assert!(!out(&sb.attention(&one, &["--hook"])).is_empty());
    assert!(!out(&sb.attention(&two, &["--hook"])).is_empty());
    assert_eq!(out(&sb.attention(&one, &["--hook"])), "");
    assert_eq!(out(&sb.attention(&two, &["--hook"])), "");
    assert_eq!(state_lines(&sb).len(), 2);
}

#[test]
fn plain_attention_never_touches_the_throttle_state() {
    let sb = Sandbox::new("plain-state");
    let repo = reportable_project(&sb, "repo");
    // Fresh state for this very project: plain attention must still print,
    // and leave the file byte-for-byte alone.
    fs::create_dir_all(hook_state_path(&sb).parent().unwrap()).unwrap();
    let content = format!("{}\t{}\n", now(), repo.display());
    fs::write(hook_state_path(&sb), &content).unwrap();
    assert!(!out(&sb.attention(&repo, &[])).is_empty());
    assert_eq!(fs::read_to_string(hook_state_path(&sb)).unwrap(), content);
}

#[test]
fn hook_follows_scope_rules_but_throttles_by_containing_project() {
    let sb = Sandbox::new("hook-scope");
    let top = monorepo(&sb);
    let nearest_plain = out(&sb.attention(&top, &["--scope", "nearest"]));
    let o = sb.attention(&top, &["--hook", "--scope", "nearest"]);
    assert_eq!(out(&o), nearest_plain);
    // Same containing project, different scope: already shown today.
    assert_eq!(out(&sb.attention(&top, &["--hook"])), "");
    assert_eq!(state_lines(&sb).len(), 1);
}

#[test]
fn hook_usage_errors_are_still_loud_and_record_nothing() {
    let sb = Sandbox::new("hook-usage");
    let repo = reportable_project(&sb, "repo");
    for args in [&["--hook", "--scope", "bogus"][..], &["--hook", "--nope"][..]] {
        let o = sb.attention(&repo, args);
        assert_eq!(o.status.code(), Some(1), "{:?}", args);
        assert!(err(&o).starts_with("error:"), "{}", err(&o));
    }
    assert!(!hook_state_path(&sb).exists());
}
