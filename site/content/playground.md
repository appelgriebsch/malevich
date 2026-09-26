# Playground

Paste numbers, pick a chart, and watch the plot lay itself out as you change the frame. The renderer is the crate compiled to WebAssembly — the same engine the [npm package](../js/) ships. It draws the exact string a terminal would receive, decoded the way a terminal emulator would show it. Beside the plot is the Rust program that draws it, and the `kaz` command line that draws it from a shell.

<div id="playground"><p class="status">Loading the engine…</p></div>

## What this shows

The plot on this page is built once from your data, and nothing about it changes after that. The frame is this run's state: width, height, charset, color mode. Change the frame and the layout runs again. Ticks search again for a labeling that fits. The legend and the title shed when the frame is small. The labels stay exact decimals throughout. Try `Plain` and read what a pipe would see.

The link in your address bar carries the state. Send it to someone and they open the same plot.
