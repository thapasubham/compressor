# Compressor

A peak compressor audio plugin built in Rust with [`nice-plug`](https://codeberg.org/RustAudio/nice-plug)
(a `nih_plug`-style plugin framework) and [`nice-plug-iced`](https://codeberg.org/RustAudio/nice-plug)
for the GUI. Ships as VST3 and CLAP, plus a standalone app for quick testing
and an optional offline WAV-batch CLI.

## Controls

| Control       | Range           | Description                                   |
|----------------|-----------------|------------------------------------------------|
| Threshold      | -60 dB to 0 dB  | Level above which compression starts           |
| Ratio          | 1:1 to 20:1     | Amount of gain reduction above threshold        |
| Attack         | 0.1 ms to 500 ms| How fast the compressor reacts to loud signal   |
| Release        | 1 ms to 1000 ms | How fast the compressor recovers after loud signal |
| Makeup Gain    | -12 dB to 24 dB | Gain applied after compression                  |
| Mix            | 0% to 100%      | Blend between the dry and compressed (wet) signal |
| Bypass         | on/off          | Passes audio through unprocessed                |

The GUI also shows a live gain-reduction meter, all bound directly to the
plugin's `nih_plug`-style parameters (no local/fake UI state — everything is
automatable from a host and saved/restored with your project).

## Project layout

```
src/
  dsp/         Core signal processing: envelope follower, gain computer,
               the PeakCompressor, and dB/gain unit conversions.
  plugin/      The nice-plug Plugin/ClapPlugin/Vst3Plugin implementation
               and its parameters (src/plugin/params.rs).
  ui/          The Iced-based editor GUI, including the custom knob widget
               and gain-reduction meter (src/ui/knob.rs).
  cli/         Optional offline WAV batch-processing CLI (behind the `cli`
               feature).
  main.rs      Standalone runner (opens the GUI, streams live audio).
  lib.rs       Library entry point used by the plugin bundles.
```

## Building

Requires a recent stable Rust toolchain (`rustup` recommended).

```shell
cargo build --release
```

### Running standalone (no DAW required)

Useful for quickly testing the GUI and live audio processing against your
system's audio devices:

```shell
cargo run --bin compressor
```

Use `--input-device`/`--output-device` to pick specific audio devices; run
with `--help` to see all standalone options.

### Bundling as a plugin (VST3 / CLAP)

This project uses `nice-plug`'s bundler rather than `nih_plug`'s `cargo xtask`.
Install it once:

```shell
cargo install cargo-nice-plug
```

Then bundle:

```shell
cargo nice-plug bundle compressor --release
```

This produces, under `target/bundled/`:

- `compressor.vst3` — for any VST3 host (FL Studio, Ableton, Cubase, etc.)
- `compressor.clap` — for CLAP hosts (Bitwig, REAPER, etc.)
- `compressor.app` — a standalone macOS app bundle

Install a bundle by copying it into your OS/host's plugin folder, e.g. on
macOS:

```shell
cp -R target/bundled/compressor.vst3 ~/Library/Audio/Plug-Ins/VST3/
cp -R target/bundled/compressor.clap ~/Library/Audio/Plug-Ins/CLAP/
```

Then rescan plugins in your DAW (in FL Studio: `Options` → `Manage Plugins` →
`Find plugins`).

### Offline WAV batch processing (CLI)

Enable the `cli` feature to build a small command-line tool that applies the
compressor to a WAV file without a DAW or GUI:

```shell
cargo build --release --features cli
./target/release/compressor input.wav output.wav --threshold -18 --ratio 4 --attack 10 --release 100 --makeup 0
```

Run with `-h`/`--help` to see all flags and their defaults.

## Testing

```shell
cargo test --lib
```

Covers the DSP core (envelope follower, gain computer, dB/gain conversions,
full `PeakCompressor` behavior) and the CLI argument parser.
