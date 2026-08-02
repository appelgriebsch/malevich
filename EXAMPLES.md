# Gallery

The showcase and the system test in one artifact. Regenerate with
`cargo run --example regen_gallery`; CI fails when this file is stale.
Every example renders a fixed `Frame::plain`, so output is deterministic.

## sine

Function sampling: curves drawn from `f(x)`, one sample per subpixel column.
Source: [examples/sine.rs](examples/sine.rs)

```text
                        sin(x) and 0.6 cos(x/2)
 1.0 ┤     ⢀⠔⠊⠉⠑⢄                           ⢀⠔⠉⠉⠒⢄
     │    ⡠⠃     ⠑⢄                        ⡔⠁     ⠣⡀
     │⠤⠤⢄⣰⡁       ⠈⢆                     ⢀⠜        ⠘⡄              ⢀⣀⡠⠤⠤
 0.5 ┤  ⢰⠁⠈⠉⠒⠢⣄    ⠘⡄                    ⡸          ⢸          ⣠⠔⠒⠉⠁
     │ ⢀⠎      ⠉⠒⢄⡀ ⠸⡀                  ⡰⠁           ⢣     ⢀⠤⠒⠉
     │⢀⠎          ⠈⠱⢄⠱⡀                ⢠⠃             ⢣ ⢀⡠⠊⠁
 0.0 ┤⠎              ⠉⢳⢄              ⢠⠃             ⢀⡨⢖⠁              ⡰
     │                 ⢣⠉⠦⡀          ⢠⠃            ⣀⠔⠁ ⠈⢆             ⡰⠁
     │                  ⢣ ⠈⠑⠢⣀      ⢀⠎         ⢀⡠⠔⠊     ⠈⢆           ⡰⠁
-0.5 ┤                   ⡇    ⠙⠒⠤⣀⡀ ⡎      ⣀⡠⠤⠒⠁         ⠈⡆          ⡇
     │                   ⠘⡄       ⠈⡹⠑⠒⠒⠒⠒⠉⠉               ⠱⡀       ⢀⠎
     │                    ⠈⢆     ⢀⠜                        ⠑⡄     ⡠⠊
-1.0 ┤                      ⠑⠤⣀⡠⠔⠁                          ⠈⠢⢄⣀⡠⠊
     └┬─────────┬──────────┬─────────┬─────────┬──────────┬─────────┬───
      0         2          4         6         8         10        12
```

## loss

The training-loop story: two series on shared scales; unrecorded steps are gaps.
Source: [examples/loss.rs](examples/loss.rs)

```text
                     loss per training step (synthetic)
5 ┤
  │⠁
  │⢄ ⠠
4 ┤⠈⠢⢄
  │  ⠈⢆ ⠁
  │    ⠣⠤⡀⠈ ⢀
3 ┤      ⠈⠒⡄   ⠄
  │        ⠈⠢⣀⡀  ⠠
2 ┤           ⠈⠑⠤⡀  ⠂ ⠠
  │              ⠈⠢⠤⠤⡀   ⠁ ⠄
  │                  ⠈⠒⠤⠤⢄⣀  ⠐  ⠄ ⠠  ⡀
1 ┤                        ⠑⠒⠤⠤⠤⢄      ⠐ ⠠  ⠄ ⠠
  │                              ⠉⠑⠊⠉⠑⠢⠤⠤⠤⠤⢄⡀  ⣀ ⠁ ⠐  ⠂ ⠁ ⠐  ⠄ ⢀  ⠄ ⠠ ⢀
  │                                         ⠈⠉⠉ ⠉⠉⠒⠒⠊⠉⠑⠢⠤⠔⠒⠒⠢⠤⠤⠔⠢⠤⢄⡠⠤⠤⠤⣀⣀⡡⠤⠄
0 ┤
  └┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬
   0    10    20    30    40    50    60    70    80    90    100   110  120
```
