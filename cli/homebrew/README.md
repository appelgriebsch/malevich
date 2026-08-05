# Homebrew formula for `kaz`

[`kaz.rb`](kaz.rb) is the canonical copy of the formula. The live one is
`Formula/kaz.rb` in the tap repo [`shergin/homebrew-tap`](https://github.com/shergin/homebrew-tap).
It builds `kaz` from source with `cargo` and installs the binary, shell
completions, and the man page.

## Install

```sh
brew install shergin/tap/kaz            # stable, from the cli-vX.Y.Z tag
brew install --HEAD shergin/tap/kaz     # track main
```

## Cut a new release

1. Tag the CLI release on this repo: `git tag cli-vX.Y.Z && git push origin cli-vX.Y.Z`.
2. Update `url` + `sha256` in both this file and the tap's `Formula/kaz.rb`:

   ```sh
   url="https://github.com/shergin/malevich/archive/refs/tags/cli-vX.Y.Z.tar.gz"
   curl -sL "$url" | shasum -a 256
   ```

3. `brew audit --strict shergin/tap/kaz` and `brew test shergin/tap/kaz`, then
   commit the tap.

## Later: prebuilt bottles

Building from source needs a Rust toolchain. To install with no toolchain, add a
release workflow (e.g. `cargo-dist`) that builds per-platform binaries on a tag,
publishes a GitHub release, and auto-updates the formula with bottle SHAs — the
remaining M-C4 reach item. The source formula works in the meantime.
