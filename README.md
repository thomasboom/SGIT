# SGIT
SGIT (Simple Git) is a lightning-fast Rust wrapper around the Git CLI that exposes streamlined workflows for initializing repositories, staging files, creating commits, and pushing in a few simple commands.

## Building

```sh
cd sgit
cargo build --release
```

Copy `target/release/sgit` into your `PATH`, or run it via `cargo run --bin sgit -- <command>`.

## Usage

```
sgit <command> [options]
```

### Simplified commands

- `sgit init` — run `git init`
- `sgit stage [path ...]` — add files (defaults to `.`)
- `sgit unstage [path ...]` — drop files from the staging area (`git restore --staged`)
- `sgit commit -m "message" [--all | --unstaged | --staged] [--push] [--amend]` — create commits with helpers to stage tracked/unstaged changes and optionally push immediately
- `sgit status [--short]` — show `git status` (`-sb` with `--short`)
- `sgit log [--short]` — compact or detailed log
- `sgit diff [path] [--staged]` — diff working tree (or staged snapshot)
- `sgit branch` — list local branches
- `sgit push [remote] [branch]` — push with the same defaults as `git push`, but allow overriding remote/branch if you need to force a specific ref
- `sgit pull [remote] [branch]` — pull with optional remote/branch

When using `--push`, SGIT now runs `git push` without hard-coding `origin`, so your repository’s configured upstream and `push.default` still take precedence. `--all` stages tracked and untracked files before committing, `--unstaged` stages tracked-but-uncommitted changes, and the plain commit command commits only what you already staged.

`sgit status` accepts `--short` to show the compact `git status -sb` view, and `sgit push` respects the default `git push` behavior (add `remote`/`branch` only if you explicitly pass them).

Set `--explain` on any `sgit` invocation (even without a subcommand) to print a friendly “noob explanation” of each command and its common options instead of running the command you normally would.

## Local installation

Use the provided scripts to install or remove the binary:

```
./install.sh       # builds SGIT and copies it to $HOME/.local/bin (or $SGIT_INSTALL_DIR)
./uninstall.sh     # deletes the installed binary from the same location
```

Set the `SGIT_INSTALL_DIR` environment variable before running `install.sh`/`uninstall.sh` if you prefer installing somewhere else in your PATH. Re-running `install.sh` rebuilds and updates the binary.
