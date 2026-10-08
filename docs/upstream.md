# Upstream and related work

Research checked **8 October 2026** using GitHub's repository and issue/PR search,
the current contribution guide, and the linked project READMEs. This is a bounded
search, not evidence that no other private or unindexed iPad project exists.

## Where this belongs

[PhotoCraft](https://github.com/storytold/photocraft) already has a Rust/WebAssembly
browser target. Its [contribution guide](https://github.com/storytold/photocraft/blob/main/docs/contributing.md)
asks for Rust, engine commands, tests, small commits and visual evidence. The
[agent guide](https://github.com/storytold/photocraft/blob/main/AGENTS.md) excludes
Tauri/Electron/webview shells, not the existing WebAssembly target.

No maintainer statement was found accepting or rejecting a complete iPad
workspace. An [Android NativeActivity proposal (#856)](https://github.com/storytold/photocraft/pull/856)
and its [palm-rejection patch (#857)](https://github.com/storytold/photocraft/pull/857)
are open; at the time checked, neither has a maintainer review. Their existence
does not establish endorsement of our approach.

Our recommendation: maintain the iPad workspace independently for now, then ask
about scope before proposing its inclusion. Offer reusable fixes separately:

| Candidate | Local source commit | Proposed treatment |
|---|---|---|
| Per-point pressure and tilt in batched canvas input | `953dce7` | Small regression-backed bug fix; rebase and check current input work first |
| Responsive shared widgets and New Document | `cf6c45e` | Opt-in touch sizing with desktop unchanged; before/after captures |
| Export and Layer Style touch layouts | `2b4b87d` | Separate focused UI proposals with narrow-width tests |
| Browser host/workspace hooks | `022408e` | Discuss the seam first; this is an architecture decision |
| Shared layer reveal API | `af949b1` | Small host API addition; supports layer targeting in alternate workspaces |
| iPad workspace, navigation, diagnostics and hosting | This repository | Remain here unless upstream requests otherwise |

The five source commits are preserved as mail patches in `patches/photocraft/`.
They apply to the public baseline in `upstream.env`. They have not been submitted.
Any upstream PR must follow the current guide, include the relevant upstream
checks, and acknowledge overlapping work. The existing public `1puni/photocraft`
fork is separate from this repository's reproducible iPad patch series.

## Others working nearby

| Project | What its own documentation establishes | Difference from this work |
|---|---|---|
| [simonwjackson's Android stack](https://github.com/storytold/photocraft/pull/856) | Native Android shell/APK, tablet input and palm-rejection proposals | Android native, not iPad Safari; useful input-policy overlap |
| [tzhazuma/artcraft-mobile-port](https://github.com/tzhazuma/artcraft-mobile-port) | iOS/Android/HarmonyOS porting notes and Android experiments, notably FilmCraft | Includes an iOS route, but no demonstrated PhotoCraft iPad/Pencil workspace in the README |
| [WavJaby/photocraft-web](https://github.com/WavJaby/photocraft-web) | Community host of the unmodified v0.3.0 web build; loading/cache work and small-screen scaling | Its README explicitly says scaling does not change touch/keyboard behavior; mobile checks use emulation |

Searches for `photocraft ipad` and `photocraft tablet` with forks included, plus
`iPad` in upstream issue/PR titles and bodies, did not reveal another dedicated
iPad workspace. Broader mobile/web searches found the projects above. **Do not
market this as the first.** The defensible description is an independent iPad and
Pencil workspace for PhotoCraft.

## Credit and identity

PhotoCraft's engine, canvas, formats and shared controls are upstream's work.
Our work is the iPad workspace and integration, with the reusable patch series
clearly identified. Preserve MIT/Apache licenses, NOTICE, and asset licenses.
The [ArtCraft brand terms](https://github.com/storytold/photocraft/blob/main/docs/brand/LICENSE-brand.txt)
require modified distributions to remove their marks; plain-text upstream credit
is appropriate. Our documentation art is original and carries a small 1puni mark.
No endorsement, partnership, official iPad edition or native iPad binary is claimed.
