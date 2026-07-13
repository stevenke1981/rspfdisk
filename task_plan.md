# Task Plan - Bootable Guided Installer

## Goal

Make the GRUB2 boot media launch Rust SPFDisk directly into a simple guided workflow for Windows, Linux, macOS, and multiboot partition layouts while preserving image-only automated write tests and explicit confirmation.

## Scope

- Improve GRUB2 and initramfs startup defaults and recovery options.
- Add beginner-oriented install scenarios and clearer TUI navigation.
- Reuse existing layout templates and safety/write pipeline.
- Add automated tests and user-facing boot/install documentation.
- Build release, review all delegated work, commit, and push `master`.

## Safety Boundaries

- Do not write to a real disk during development or tests.
- Do not weaken `ChangePlan`, backup, confirmation, or read-back verification.
- macOS support means preparing GPT/APFS target partitions, not installing macOS or formatting APFS.
- GRUB2 boots SPFDisk media; OS installers remain separate external media unless explicitly integrated later.

## Phases

| Phase | Status | Description |
|---|---|---|
| 1. Inspect | completed | Map boot, TUI, templates, docs, and current gaps |
| 2. Delegate | completed | Assign disjoint boot, TUI/scenario, and docs/test work to Luna xhigh agents |
| 3. Integrate | completed | Review and combine subagent changes; resolve interfaces locally |
| 4. Verify | completed | Unit tests, image tests, scripts, Clippy, release build |
| 5. Publish | completed | Update final evidence, commit, push, confirm clean status |

## Acceptance Criteria

- Boot media defaults to Rust SPFDisk TUI and offers a recovery shell entry.
- First screen presents understandable Windows, Linux, macOS, and multiboot scenarios.
- Scenario selection maps deterministically to existing tested templates.
- Existing backup, confirmation, and image-only write rules remain enforced.
- Automated tests cover scenario mapping and boot bundle expectations.
- Documentation explains what SPFDisk prepares versus what the OS installer completes.
- Workspace release build and required checks pass.

## Errors Encountered

| Error | Attempt | Resolution |
|---|---|---|
| Combined initialization batch returned exit code 1 | 1 | Split calls; session catchup succeeded, missing planning files caused `Get-ChildItem` to exit 1 |
| `rg` rejected `README*.md` on Windows | 1 | Use explicit README paths or enumerate files before searching |
| Subagent message template failed JavaScript parsing | 1 | Remove Markdown backticks from JavaScript template strings and resend as plain text |
| Four CBM symbol reads failed after TUI line drift | 1 | Re-query the graph for current qualified names before reading changed symbols |
| Markdown link checker treated root README parent as empty | 1 | Normalize an empty parent to `.` and rerun separately from Rust tests |
| Rust format check requested multiline test assertions | 1 | Run `cargo fmt --all`, then rerun format and focused tests |
| `bash -n` resolved to WSL without an installed distribution | 1 | Use Git Bash directly when available; keep full Linux boot verification assigned to Ubuntu CI |
| Combined validation tool script had a JavaScript quoting error | 1 | Run Git Bash, PowerShell validators, and link checker as smaller independent calls |
| Reviewer spawn rejected full-history fork with explicit model/role overrides | 1 | Spawn a fresh `code-reviewer` with the requested Luna xhigh model and a self-contained repository brief |
