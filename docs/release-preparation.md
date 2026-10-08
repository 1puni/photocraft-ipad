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
- [x] **Build and distribution:** the initial clean-clone pass covered 19 tests,
  formatting, Clippy, optimized Wasm, the server boundary and packaged notices.
  After integrating the command shelf, all 26 tests, formatting, native/Wasm
  Clippy, optimized build and license packaging passed again. Browser acceptance
  and screenshot provenance are recorded in [verification](verification.md).
  A failed license check preserves the served preview byte-for-byte.
- [x] **Private security reports:** [SECURITY.md](../SECURITY.md) points to
  `gg@1puni.com`, the existing GG inbox; read-only access verified during this pass.
- [x] **Device status:** the [roadmap](ipad-port.md) records physical Pencil and
  iPad microphone tests plus workflow refinements as upcoming v0.1 work. Automated/browser evidence is
  kept separately in the test record. These are development work, not a reason
  to keep the source closed.
- [ ] **Public visibility:** awaiting V's explicit go-ahead.

## Candidate and development handoff

`989c3ae` adds the command shelf and browser speech adapter from implementation
commit `2ba5edf` on top of the reviewed `29bd591` preparation. The earlier
`82be479` integrated the `5eae0db` Layers stack and grouped tool navigation.
Original authorship and source commit provenance are preserved. The engine is
pinned to public baseline `5896f0b` plus five mail patches,
producing tree `7545cfe69924ec4f48fee63aa61ebf3ff82c48ac` (equivalent to `af949b1`).

The publishing checkout is separate from ongoing development. Take in remote
`main` before the next development push. The command batch was cherry-picked, so
its original implementation branch should merge the prepared history:

```sh
git fetch origin
git merge origin/main
```

Preserve any new or uncommitted work before merging; never reset it or replay the
same command batch again. Keep release documentation when resolving overlaps.
Each checkout uses its own `target/` to avoid the stale-UI collision
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
