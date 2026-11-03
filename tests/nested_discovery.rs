use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Sandbox {
        let root = std::env::temp_dir().join(format!("spox-test-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        // Canonicalize so paths match what spox prints (macOS /tmp -> /private/tmp).
        let root = fs::canonicalize(&root).unwrap();
        // A fresh last_check keeps the daily self-update (git pull + cargo
        // build of the spox repo) from running during tests.
        let cache = root.join("home/.cache/spox");
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("last_check"), b"").unwrap();
        Sandbox { root }
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root.join("work").join(rel)
    }

    fn mkdir(&self, rel: &str) -> PathBuf {
        let p = self.path(rel);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn git_repo(&self, rel: &str) -> PathBuf {
        let p = self.mkdir(rel);
        fs::create_dir_all(p.join(".git")).unwrap();
        p
    }

    // Create `<rel>/.spox` containing one spec called `spec`.
    fn project(&self, rel: &str, spec: &str) -> PathBuf {
        let spox = self.mkdir(rel).join(".spox");
        fs::create_dir_all(&spox).unwrap();
        // Pre-synced so spox has nothing to announce on stderr.
        fs::write(spox.join(".format.md"), include_str!("../format.md")).unwrap();
        fs::write(
            spox.join(format!("{}.md", spec)),
            "status: draft\ndate: 2026-01-01\n\n# T\n\n## Acceptance criteria\n- [ ] one\n",
        )
        .unwrap();
        spox
    }

    fn spox(&self, cwd: &Path, args: &[&str]) -> Output {
        self.spox_env(cwd, args, &[])
    }

    fn spox_env(&self, cwd: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_spox"));
        cmd.args(args)
            .current_dir(cwd)
            .env("HOME", self.root.join("home"))
            .env_remove("SPOX_PROJECT")
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

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn spox_at_git_root_is_unchanged() {
    let sb = Sandbox::new("root");
    let repo = sb.git_repo("repo");
    sb.project("repo", "alpha");
    let sub = sb.mkdir("repo/src/deep");

    for cwd in [&repo, &sub] {
        let o = sb.spox(cwd, &["list"]);
        assert!(o.status.success(), "{}", err(&o));
        assert!(out(&o).contains("alpha"));
        assert_eq!(err(&o), "", "no extra output when .spox is at or above cwd");
    }
}

#[test]
fn ancestor_spox_wins_over_nested_below() {
    let sb = Sandbox::new("ancestor");
    let repo = sb.git_repo("repo");
    sb.project("repo", "outer");
    sb.project("repo/pkg", "inner");

    let o = sb.spox(&repo, &["list"]);
    assert!(out(&o).contains("outer") && !out(&o).contains("inner"));
    assert_eq!(err(&o), "");

    // Nearest wins when starting inside the nested project.
    let o = sb.spox(&repo.join("pkg"), &["list"]);
    assert!(out(&o).contains("inner") && !out(&o).contains("outer"));
}

#[test]
fn single_nested_project_found_from_root_and_sibling() {
    let sb = Sandbox::new("single");
    let repo = sb.git_repo("notes");
    sb.project("notes/remote-sensing/quiz-app", "quiz");
    let sibling = sb.mkdir("notes/other/stuff");

    let o = sb.spox(&repo, &["list"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("quiz"));
    assert!(err(&o).contains("using remote-sensing/quiz-app/.spox"), "{}", err(&o));

    let o = sb.spox(&sibling, &["list"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("quiz"));
    // Not under cwd, so the note falls back to an absolute path.
    assert!(err(&o).contains(&sb.path("notes/remote-sensing/quiz-app/.spox").display().to_string()));
}

#[test]
fn write_commands_work_on_nested_project() {
    let sb = Sandbox::new("write");
    let repo = sb.git_repo("notes");
    let spox = sb.project("notes/app", "quiz");

    let o = sb.spox(&repo, &["status", "quiz", "ongoing"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(fs::read_to_string(spox.join("quiz.md")).unwrap().starts_with("status: ongoing"));
}

#[test]
fn multiple_nested_projects_is_an_error_naming_each() {
    let sb = Sandbox::new("multi");
    let repo = sb.git_repo("notes");
    sb.project("notes/a", "one");
    sb.project("notes/group/b", "two");

    for args in [&["list"][..], &["view", "one"], &["status", "one", "completed"], &["path"]] {
        let o = sb.spox(&repo, args);
        assert_eq!(o.status.code(), Some(1), "{:?}", args);
        let e = err(&o);
        assert!(e.contains("multiple spox projects found"), "{}", e);
        assert!(e.contains("  a\n") && e.contains("  group/b\n"), "{}", e);
        assert!(e.contains("-C") && e.contains("SPOX_PROJECT"), "{}", e);
        assert_eq!(out(&o), "");
    }
    // The status call must not have modified anything.
    assert!(fs::read_to_string(sb.path("notes/a/.spox/one.md")).unwrap().starts_with("status: draft"));
}

#[test]
fn no_spox_anywhere_keeps_existing_error() {
    let sb = Sandbox::new("none");
    let repo = sb.git_repo("repo");
    let o = sb.spox(&repo, &["list"]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(err(&o), "error: no .spox directory found in this or any parent directory\n");
}

#[test]
fn no_downward_search_outside_git_repo() {
    let sb = Sandbox::new("nogit");
    let dir = sb.mkdir("plain");
    sb.project("plain/app", "quiz");
    let o = sb.spox(&dir, &["list"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("no .spox directory found"));
}

#[test]
fn dash_c_accepts_project_subdir_and_spox_dir() {
    let sb = Sandbox::new("dashc");
    // Outside any git repo, so -C is the only way in.
    sb.project("plain/app", "quiz");
    let elsewhere = sb.mkdir("elsewhere");
    sb.mkdir("plain/app/src");

    for target in ["plain/app", "plain/app/src", "plain/app/.spox"] {
        let dir = sb.path(target);
        let o = sb.spox(&elsewhere, &["-C", dir.to_str().unwrap(), "list"]);
        assert!(o.status.success(), "{}: {}", target, err(&o));
        assert!(out(&o).contains("quiz"), "{}", target);
    }

    // Relative paths resolve against cwd.
    let o = sb.spox(&sb.path("plain"), &["-C", "app", "list", "-c"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("one"), "open criteria shown: {}", out(&o));
}

#[test]
fn dash_c_picks_one_of_several() {
    let sb = Sandbox::new("pick");
    let repo = sb.git_repo("notes");
    sb.project("notes/a", "one");
    sb.project("notes/b", "two");

    let o = sb.spox(&repo, &["-C", "b", "list"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("two") && !out(&o).contains("one"));
}

#[test]
fn dash_c_errors() {
    let sb = Sandbox::new("dashc-err");
    let cwd = sb.mkdir("x");
    fs::write(sb.path("x/file"), "").unwrap();

    let o = sb.spox(&cwd, &["-C", "missing", "list"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).starts_with("error: -C: cannot use 'missing'"), "{}", err(&o));

    let o = sb.spox(&cwd, &["-C", "file", "list"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("is not a directory"), "{}", err(&o));

    let o = sb.spox(&cwd, &["-C"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("-C requires a directory"));
}

#[test]
fn spox_project_env_and_precedence() {
    let sb = Sandbox::new("env");
    sb.project("plain/a", "one");
    sb.project("plain/b", "two");
    let cwd = sb.mkdir("elsewhere");
    let a = sb.path("plain/a");
    let b = sb.path("plain/b");

    let o = sb.spox_env(&cwd, &["list"], &[("SPOX_PROJECT", a.to_str().unwrap())]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("one") && !out(&o).contains("two"));

    // -C wins over the env var.
    let o = sb.spox_env(
        &cwd,
        &["-C", b.to_str().unwrap(), "list"],
        &[("SPOX_PROJECT", a.to_str().unwrap())],
    );
    assert!(out(&o).contains("two") && !out(&o).contains("one"));

    // A bad env value is reported as such; an empty one is ignored.
    let o = sb.spox_env(&cwd, &["list"], &[("SPOX_PROJECT", "/nope/nothing")]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).starts_with("error: SPOX_PROJECT:"), "{}", err(&o));
    let o = sb.spox_env(&sb.path("plain/a"), &["list"], &[("SPOX_PROJECT", "")]);
    assert!(o.status.success(), "{}", err(&o));
}

#[test]
fn path_and_format_commands() {
    let sb = Sandbox::new("pathfmt");
    let repo = sb.git_repo("notes");
    sb.project("notes/remote-sensing/quiz-app", "quiz");

    let o = sb.spox(&repo, &["path"]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), format!("{}\n", sb.path("notes/remote-sensing/quiz-app/.spox").display()));

    let o = sb.spox(&repo, &["format"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("# Spec format"), "{}", out(&o));
    // Same bytes as the file spox synced into the project.
    let on_disk = fs::read_to_string(sb.path("notes/remote-sensing/quiz-app/.spox/.format.md")).unwrap();
    assert_eq!(out(&o), on_disk);

    // Also reachable through -C in a single command.
    let o = sb.spox(&sb.mkdir("elsewhere"), &["-C", repo.to_str().unwrap(), "path"]);
    assert!(o.status.success(), "{}", err(&o));
}

#[test]
fn search_depth_and_skipped_dirs() {
    let sb = Sandbox::new("depth");
    let repo = sb.git_repo("r");

    // Project dir 6 levels down: out of range.
    sb.project("r/1/2/3/4/5/6", "toodeep");
    let o = sb.spox(&repo, &["list"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));

    // Project dir 5 levels down: found.
    sb.project("r/a/b/c/d/e", "deep");
    let o = sb.spox(&repo, &["list"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("deep") && !out(&o).contains("toodeep"));

    // Hidden dirs, node_modules and target are never searched.
    let sb2 = Sandbox::new("skipped");
    let repo2 = sb2.git_repo("r");
    sb2.project("r/node_modules/pkg", "nm");
    sb2.project("r/target/x", "tg");
    sb2.project("r/.hidden/x", "hd");
    let o = sb2.spox(&repo2, &["list"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    assert!(err(&o).contains("no .spox directory found"));
}

#[cfg(unix)]
#[test]
fn symlinked_dirs_are_not_followed() {
    let sb = Sandbox::new("symlink");
    let repo = sb.git_repo("r");
    sb.project("elsewhere/app", "quiz");
    std::os::unix::fs::symlink(sb.path("elsewhere/app"), repo.join("link")).unwrap();
    let o = sb.spox(&repo, &["list"]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
}
