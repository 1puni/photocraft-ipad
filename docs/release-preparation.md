# Public source readiness

V approved the presentation and requested licensing review and release checks on
8 October 2026. **The repository remains private until V authorizes the visibility
switch.** The hosted demo and optional tips remain separate future decisions.

## Checklist

- [x] **Presentation and naming:** V reviewed the repository and asked us to proceed
  with preparation. Lead with the work; v0.1 and the roadmap communicate maturity.
- [x] **Licensing:** [review completed](licensing.md). MIT OR Apache-2.0 code,
  upstream notices retained, independent identity, embedded asset terms preserved.
  `cargo-about` resolves the 240-package Wasm runtime/build graph.
- [x] **History:** Gitleaks 8.30.1 scanned all published refs and full history,
  including the latest UI work, with zero findings. The reachable file inventory
  contains source, docs, patches and our screenshots; no credentials, personal
  documents or private operational logs. Original commit authorship is preserved.
- [x] **Clean-clone build and distribution:** 19 tests, formatting, Clippy,
  optimized Wasm build, server boundary test and packaged notices passed. Fresh
  browser captures show the integrated UI. A failed license check preserves the
  served preview byte-for-byte. Results are in [verification](verification.md).
- [x] **Private security reports:** [SECURITY.md](../SECURITY.md) points to
  `gg@1puni.com`, the existing GG inbox; read-only access verified during this pass.
- [x] **Device status:** the [roadmap](ipad-port.md) records physical Pencil tests
  and workflow refinements as upcoming v0.1 work. Automated/browser evidence is
  kept separately in the test record. These are development work, not a reason
  to keep the source closed.
- [ ] **Public visibility:** awaiting V's explicit go-ahead.

## Candidate and development handoff

`82be479` integrates the publishing preparation with the other agent's `5eae0db`
Layers stack and grouped tool navigation. All authorship and both histories are
preserved. The engine is pinned to public baseline `5896f0b` plus five mail patches,
producing tree `7545cfe69924ec4f48fee63aa61ebf3ff82c48ac` (equivalent to `af949b1`).

The publishing checkout is separate from ongoing development. Take in remote
`main` before the next development push. If the development checkout is still at
`5eae0db` and clean, this is a fast-forward:

```sh
git fetch origin
git merge --ff-only origin/main
```

If it has new commits or uncommitted work, preserve that work and merge normally;
never reset it. Each checkout uses its own `target/` to avoid the stale-UI collision
found during development. Update `upstream.env` and the patch series together when
changing the engine, and rerun the license/build checks for the new candidate.

## At the public switch

After V's go-ahead, confirm the reviewed revision is still current, change this
repository's visibility, and verify it loads without authentication. Enable
GitHub private vulnerability reporting then; GitHub makes that feature available
for public repositories ([documentation](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository)).
The email reporting route is already available. Tell the development agent the
public repository and branch are ready, and update this checklist with the receipt.
No release tag, public demo, paid service or upstream submission is implied by
opening the source.
