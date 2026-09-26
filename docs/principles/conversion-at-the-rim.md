# Conversion lives at the rim

The core computes in `f64`, monomorphically. Every other numeric shape
converts exactly once, at ingestion, and never again.

## Why

The tempting design is a core generic over a `Float` trait. Accept `f32`,
integers, decimals, anything, all the way down. The cost shows up everywhere
this library actually earns its keep. Tick placement needs literals (`10.0`,
`0.5`, the SI thresholds). Exact-decimal formatting needs a concrete
mantissa. The KDE needs constants. Every generic bound infects every
signature. The hot loops either monomorphize into code-size bloat, or they
dispatch per point. Topos, this library's sibling, pays a documented
"generic-literal gap" tax for payload genericity, because one engine over
scalars and tensors is its literal thesis. Malevich has no such thesis. A
generic float core would buy nothing. It would cost literals, tick math, and
formatting everywhere.

The other temptation is to accept nothing but `&[f64]`. That taxes every
caller with a conversion loop and an allocation they have to remember to
write, and half of them convert wrongly at the first `NaN`.

## The idea

The boundary is a trait. The interior is a type. Anything series-shaped —
slices, arrays, vectors of any primitive numeric type, iterators — converts
exactly once, at the rim, into contiguous `f64`, where `NaN` is the gap.
A borrowed `f64` slice crosses zero-copy. Inside the rim there is one numeric
world. Monomorphic `f64`, no bounds, no dispatch. Literals and constants are
used freely. The hot loops are code the optimizer can see through.

The rim is also where the ecosystem plugs in without becoming a dependency.
ndarray's contiguous arrays borrow zero-copy, behind a feature. Polars needs
no feature at all, because a contiguous column is already a borrowed slice,
and its null-yielding iterator maps straight onto the gap convention. The
convention is the interface. A big data library integrates by meeting `f64`
plus `NaN`, not by being linked.

`f64` is the right single type. It holds every `f32`, every `u32`, and every
count exactly, and a chart raster cannot resolve the difference that remains
at `u64` extremes.

## Consequences

- Ticks, formatting, statistics, and rasterization each have one
  implementation. There are no generic variants to test in every width.
- The cost of ingestion is explicit, bounded, and paid once. A render loop
  never converts.
- The gap convention is universal, because ingestion is what establishes it.
  After the rim, code may assume `NaN` means gap.
- A new input shape is an `IntoSeries` implementation, not a change to the
  core.
- No `Float` bound ever appears in a public signature.

## Not this

- The core is not generic over a float trait, and a draw loop does not call
  `Into<f64>` per point.
- There is no second numeric path for `f32` "for performance."
- Accepting a dataframe's columns does not mean depending on that library.
- Stats do not convert lazily. The same series does not pay on every use.

See [What earns a concept](what-earns-a-concept.md) for the closed core in
general, and [Vision](../vision.md) rule 2.

## Spelled today

`data::Series` is the contiguous `f64` column; `data::IntoSeries` is the
rim, with zero-copy borrowing for `f64` slices and `FromIterator` for
iterators. The `ndarray` feature adds zero-copy ingestion for contiguous
arrays; the polars recipe in the README is two lines against the public
rim. Function sampling arrives with the marks, not through the rim. This
section may rot; the rest must not.
