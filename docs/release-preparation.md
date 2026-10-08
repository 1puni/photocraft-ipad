# Preparing the first public repository

As of **8 October 2026**, the source is being prepared in a private repository.
Public visibility requires V's explicit go-ahead. The hosted demo and optional
tips are separate, undecided proposals: there is no public endpoint, donation
link, price, release date, or service commitment in this preparation.

## Present source

The extension baseline is `06495ac`: a Rust touch workspace, browser input adapter,
local server and Pencil diagnostic page. The engine baseline and four patches
are pinned in `upstream.env` and `patches/photocraft/`. Further editor development
may continue independently; this preparation does not sweep in uncommitted work.

Presentation direction: lead with the solved work and engineering. A v0.1 label
and a single roadmap communicate maturity; do not repeat disclaimers across the
README. Credit upstream and state our contribution with equal clarity.

The [acceptance matrix](ipad-port.md) owns implementation gaps, and
[verification](verification.md) distinguishes previous results from fresh checks.
The README screenshots show the actual workspace; they are not physical-device
validation. Keep original commit attribution and disclose AI-assisted development.

## Before public visibility

- Review the rendered README and naming with V; obtain the explicit public switch.
- Review all reachable Git history for credentials, personal data and private logs;
  fix any findings before the switch, not merely in the latest files.
- Reproduce setup from a clean clone, preserve dependency/license notices, and
  inspect the actual distributed assets for restricted marks.
- Confirm a private security-reporting channel and document it.
- Publish measured iPad/Pencil results when available. If still pending, retain
  the experimental label and the explicit unverified rows; do not imply completion.

## Next engineering work

Physical Pencil pressure/tilt and palm tests; portrait/landscape/Split View and
keyboard runs; repeated PSD save/reopen; long-session and large-document behavior;
remaining dialog families and recovery. Scope any upstream proposals with the
maintainers before investing in a large upstream integration.

If we later host a demo, decide origin/TLS, asset delivery, browser memory limits,
support and ongoing costs first. Optional tips would support this independent
adaptation and must not imply an upstream fee or an official PhotoCraft service.
