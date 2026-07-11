# BGFX and GLSL Configuration

MAMEUIx can configure video options that are passed to the external MAME process.
MAME—not MAMEUIx—renders the emulated game and applies BGFX chains or GLSL
shaders.

The MAMEUIx interface itself is rendered by `eframe`/`egui`. Changing a MAME
BGFX backend does not change the renderer used for the frontend window.

## Current implementation

The active launch path consists of:

- `src/models/game_properties.rs` — video mode and per-game BGFX settings.
- `src/ui/components/game_properties.rs` — controls and command preview.
- `src/ui/components/advanced_mame_settings.rs` — advanced MAME option controls.
- `src/mame/launcher.rs` — converts saved settings into arguments for MAME.

When the selected video mode is BGFX, the launcher can pass:

- `-video bgfx`
- `-bgfx_backend <backend>`
- `-bgfx_screen_chains <chain>`
- `-bgfx_debug`
- `-bgfx_shadow_mask <texture>`
- `-bgfx_lut <texture>`

`Auto` leaves backend selection to MAME. The UI currently offers OpenGL and
Vulkan on Linux, Direct3D/OpenGL/Vulkan on Windows, and Metal/OpenGL on macOS.
This list describes the selection logic in the code; it is not a claim that all
of those operating systems have been tested for release.

For example, a configured launch can be equivalent to:

```bash
mame pacman \
  -video bgfx \
  -bgfx_backend vulkan \
  -bgfx_screen_chains crt-geom
```

The chain name and its assets must be available to the installed MAME build.
MAMEUIx does not include MAME or replace MAME's BGFX chain files.

## Experimental and roadmap code

Additional graphics helpers live under:

```text
src/utils/graphics/
├── mod.rs
├── shader_manager.rs
└── shader_templates/

src/embedded_shaders/
```

`src/utils/graphics/mod.rs` defines presets and can generate argument vectors.
`shader_manager.rs` contains basic file loading, validation, template creation,
and BGFX helper functions. `src/embedded_shaders/` embeds GLSL source files in
the binary.

These pieces are not yet connected end-to-end to the active launch path:

- `GraphicsConfig::generate_bgfx_args()` is not called by `src/mame/launcher.rs`.
- The redesign can store a selected embedded shader, but the launcher does not
  currently apply that selection.
- Custom GLSL fields appear in the game-properties UI and command preview, but
  the active launcher does not currently emit their GLSL arguments.
- The embedded `.vert` and `.frag` files are not MAME BGFX chain definitions.

Treat the preset manager, embedded-shader selection, extraction, and custom GLSL
workflow as experimental/roadmap functionality until they are wired into the
launcher and covered by integration tests.

## Setup

1. Install MAME separately and confirm that its build supports the desired video
   option and backend.
2. In MAMEUIx, open the game-properties or advanced MAME settings, select BGFX as
   the video mode, then choose a backend supported by the host.
3. Leave the screen chain at its default unless the chain is present in MAME's
   configured BGFX path.
4. Save the settings and launch a game. Per-game settings override the default
   game properties.

Before troubleshooting MAMEUIx, test the equivalent command directly in a
terminal. This separates a MAME/backend problem from frontend configuration.

## Troubleshooting

### MAME rejects the backend

- Run `mame -showusage` and inspect the video/BGFX options supported by that
  build.
- Test `-video bgfx` without `-bgfx_backend` so MAME can select a backend.
- On Linux, try `opengl` if `vulkan` is unavailable, or vice versa.

### A BGFX chain is missing or fails to load

- Remove `-bgfx_screen_chains` and confirm that plain BGFX starts.
- Check the chain name and MAME's `bgfx_path` in its configuration.
- Use chain assets distributed for the installed MAME version; arbitrary GLSL
  files are not interchangeable with MAME BGFX chains.

### A custom GLSL or embedded shader has no effect

This is a current integration limitation. The UI/helper code exists, but the
active launcher does not yet forward those selections. Supplying validated MAME
arguments through the custom-arguments setting is a temporary developer-level
workaround.

### Inspecting the generated command

A debug build prints the `Command` used to launch MAME to the terminal:

```bash
cargo run
```

Launch a game from the interface, then inspect the terminal output. MAMEUIx does
not currently define a ROM-name command-line argument.

## References

- [MAME BGFX documentation](https://docs.mamedev.org/advanced/bgfx.html)
- [MAME GLSL documentation](https://docs.mamedev.org/advanced/glsl.html)
