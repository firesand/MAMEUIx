# Dependency review — 2026-09-05

The audit repair updates the lockfile to compatible releases, upgrades direct
quick-xml to 0.41 and webbrowser to 1.2, and pins Tantivy to upstream commit
`5ca39332002c2c87fb5d2abc707cf527b3319d42`. The
unused direct naga, glsl-lang and glslang dependencies were removed: no source
or example used their APIs. Shader helpers keep their existing implementation.
This also removes the unmaintained smartstring and instant dependency paths.

Run `cargo deny --locked check advisories` with cargo-deny 0.20.2 or newer.
The check fails on vulnerabilities, unsoundness, yanked releases, and new
unmaintained advisories. CI runs the same check. Rust 1.88 is the declared
minimum because the application uses Rust 2024 let chains; the lockfile is
resolved for that minimum.

The published Tantivy 0.26.1 still requires lru 0.16, which fails the stricter
transitive unsoundness check (RUSTSEC-2026-0253). The pinned upstream commit
updates lru to >=0.18.2. It identifies itself as development version 0.27.0;
this is deliberately an immutable revision, not a moving branch. Replace
the Git dependency with a published release containing that fix when one is
available. Search tests and release builds must be checked after migration.
[Upstream fix](https://github.com/quickwit-oss/tantivy/commit/5ca39332002c2c87fb5d2abc707cf527b3319d42).

Two maintenance-only advisories remain explicitly recorded in `deny.toml`:

- **RUSTSEC-2024-0436 / paste 1.0.15.** Required by egui_dock 0.17 and image
  codec dependencies. It is a compile-time procedural macro, not a runtime
  input parser. Remove the exception when the compatible docking/image stack
  moves to a maintained replacement. [Advisory](https://rustsec.org/advisories/RUSTSEC-2024-0436.html).
- **RUSTSEC-2026-0192 / ttf-parser 0.25.1.** Required by ab_glyph in egui's
  epaint stack and the Wayland decoration stack. The application font assets
  are bundled; it does not offer user font import. This reduces the input
  exposure but does not restore upstream maintenance. Remove the exception
  when the supported GUI/font stack replaces it. [Advisory](https://rustsec.org/advisories/RUSTSEC-2026-0192.html).

These are not claims that upstream maintenance has been fixed. No vulnerability
or unsoundness advisory is exempted, and an unused exception fails the check
so it must be removed when its dependency disappears. Reassess the two
exceptions during the next GUI/dependency upgrade.
