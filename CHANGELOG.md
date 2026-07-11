# MAMEUIx Changelog

Notable changes are recorded here. Dates for tagged releases follow the Git tag dates.

## [Unreleased]

### Changed

- Synchronized the repository's Arch packaging with the tested AUR `0.1.6-1` recipe and documented standard AUR installation; Debian and RPM recipes remain pending clean-distribution validation.

## [0.1.6] - 2026-07-11

### Added

- Opt-in experimental redesigned UI shell, available from Preferences or with `--redesign`, with responsive Library, game detail, verification, and settings views. The dock-panel UI remains the default.
- Public Sans regular, medium, semibold, and bold font roles for the redesign.
- Redesign artwork loading for marquee, title, snapshot, flyer, cabinet, and PCB images, including parent fallback for clones.
- Software Lists preview for parsing and searching MAME hash XML, filtering by list, and checking best-effort media-path presence from configured software ROM roots. Archive and CHD contents are not audited.
- Experimental FreeBSD readiness: `/usr/local/bin/mame` file-picker defaults and `/usr/local/share/mame/bgfx/chains` discovery. FreeBSD remains unverified and is not yet an officially supported platform.

### Changed

- Simplified the About dialog to the application name and two stable description lines; removed the stale hard-coded version and feature inventory.
- Moved public installation, folder-layout, and graphics guides into `docs/`; internal memory, implementation journals, generated design handoff, and old standalone release notes are now local-only.
- Made redesign filters, typography, table layout, hover states, path truncation, and settings more consistent and responsive.
- Removed misleading shader preset Apply controls; the redesign Shaders page now reports BGFX status read-only and explains that bundled GLSL effect names are not MAME BGFX chain names.

### Fixed

- Built release AppImages on Ubuntu 22.04 to avoid requiring a newer host glibc.
- Detected Debian/Ubuntu MAME installations under `/usr/games/mame`.
- Prevented repeated artwork filesystem probes for known cache misses.
- Allowed invalid UTF-8 in MAME information files to fall back to lossy decoding instead of aborting the load.
- Corrected redesign search/filter cache invalidation and inclusive decade filtering.
- Made `Ctrl`/`Cmd`+`F` focus the redesign search field.
- Made the redesign Missing collection match only missing ROM sets, while Issues covers incorrect ROMs, CHD problems, and non-working or preliminary drivers.
- Made redesign Settings content scroll when it is taller than the window.
- Kept legacy theme selection from replacing the redesign token palette.

## [0.1.5] - 2026-07-05

### Added

- Dockable panels powered by `egui_dock`.
- CPU, device, sound-chip, and manufacturer filters.
- Toast notifications for loading, launch, and configuration feedback.
- Steam-inspired Preferences, Directories, and Advanced MAME Settings dialogs.
- ROM scan progress callbacks and baseline unit tests for scanning and INI/category loading.

### Changed

- Replaced the manual MAME XML parser with `quick-xml`.
- Isolated icon loading in a per-manager Rayon thread pool.
- Improved table/list keyboard navigation and scrolling.
- Updated Debian, RPM, and Arch packaging metadata and build scripts.
- Removed obsolete example binaries from the default build gate.

## [0.1.4] - 2025-08-03

### Added

- CLRMamePro Lite ROM verification with live progress and status indicators.
- Pause, resume, and stop controls for verification jobs.
- Text, CSV, and HTML verification reports.
- Missing-ROM lookup integration and detailed verification statistics.

### Fixed

- Verification state persistence and background UI responsiveness.
- Report generation and error handling during failed verification.

## [0.1.3] - 2025-07-22

### Added

- Parallel icon loading with adaptive work rates for large libraries.
- Performance monitoring and window size/position persistence.
- BGFX/GLSL configuration, integer scaling, and core performance controls.

### Changed

- Improved game-history layout, resizable dialogs, caching, and error recovery.
- Updated the application to egui 0.32.

## [0.1.2] - 2025-07-20

### Added

- Resizable game-list columns with persisted widths.
- Initial Debian, RPM, and Arch packaging support.

## [0.1.1] - 2025-07-13

### Added

- Ten UI themes and preference-based theme selection.
- Desktop integration assets and Linux installer scripts.

## 0.1.0 - Initial development

- Initial MAME executable integration, game list, filtering, ROM detection, CHD support, artwork, and egui interface.

[Unreleased]: https://github.com/firesand/MAMEUIx/compare/v0.1.6...HEAD
[0.1.6]: https://github.com/firesand/MAMEUIx/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/firesand/MAMEUIx/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/firesand/MAMEUIx/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/firesand/MAMEUIx/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/firesand/MAMEUIx/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/firesand/MAMEUIx/releases/tag/v0.1.1
