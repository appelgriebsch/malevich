# The frame is run state

A plot describes a chart. A frame describes one rendering of it. The spec
never learns where it will be drawn.

## Why

Put the terminal inside the chart object and everything downstream pays. The
plot cannot be built on one thread and rendered on another. It cannot be
rendered twice, at two sizes. It cannot be serialized without lying about a
file descriptor. Tests need a fake terminal instead of a string to compare.
Global registries and ambient configuration follow. Once one hidden input is
allowed, each new feature wants its own.

The quieter failure is a render that reads the environment. A function that
consults `$TERM` on every call is not reproducible. A snapshot test against
it pins the CI machine, not the library.

## The idea

Split the description from the run. The plot is layers, scales, and furniture:
data, complete and serializable. The frame is where and how to render. Width
and height in cells, charset, color mode, theme. Rendering is a pure function
of the two values. Call it with a different frame and the same spec resolves
its domains, places its ticks, and lays out its furniture again, for that
frame.

Nothing in the plot references a terminal, a thread, or a global, so
`Send + Sync` falls out of the design. It is not bolted on. Build in a worker
and render in the UI. Snapshot-test the string. Ship the spec over a socket
and render it on the other side.

Reading the environment is not banned. It is named. Detection lives in
explicit conveniences that construct values: a detected frame, a detected
graphics choice. Those values then drive pure calls. The documented boundary
is the constructor, never the render.

Errors split the same way. Construction panics on documented programmer
invariants, at the caller's line. Rendering never fails. It sheds what it
cannot draw, because a dashboard has to survive a small terminal. A spec that
arrives from data — deserialization, or configuration — gets the checked
twins, which report the first problem as a typed error instead.

## Consequences

- One spec renders concurrently at many sizes, with no locks. Live and TUI
  code snapshots the value and renders on its own schedule.
- Every render path can be snapshot-tested with a fixed frame. Determinism is
  the default, not a test mode.
- Serialization writes everything there is. No field is "except this one,
  which is a handle."
- A detection result is a value you can log, cache, or override. It is not a
  side effect inside a render.
- The library never owns the terminal. No raw mode, no event loop, no cleanup
  obligations on panic. An in-place repaint is one buffered write.

## Not this

- No `Chart::show()` that grabs stdout, and no plot that holds a writer.
- Rendering does not consult environment variables, the locale, or the
  terminal size directly.
- No global theme, no palette registry, no default-size setting.
- Render does not panic because the terminal is small. Shedding is the
  contract. Panics belong to construction.

See [Degradation is the contract](degradation-is-the-contract.md) for what
shedding means, and [Vision](../vision.md) rule 1.

## Spelled today

`Plot` is the spec (`Clone + Send + Sync`); `Frame` is the run state, with
`Frame::detect` the environment-reading constructor and `Frame::plain` /
`Frame::portable` the deterministic forms. `Plot::render` is the pure
function; `Plot::validate` and `Plot::try_render` are the checked twins for
foreign specs. In the `pixel` feature, `Capabilities::detect_for` constructs
the detection value and `render_with_capabilities` consumes it purely;
`render_best` is the documented stdout convenience. This section may rot; the
rest must not.
