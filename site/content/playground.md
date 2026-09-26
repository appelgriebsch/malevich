# Playground

Paste numbers, pick a chart, and watch the plot lay itself out as you change the frame. The renderer is the crate compiled to WebAssembly — the same engine the [npm package](../js/) ships — drawing the exact string a terminal would receive, decoded the way a terminal emulator would show it. Beside the plot: the Rust program that draws it, and the `kaz` command line that draws it from a shell.

<div id="playground"><p class="status">Loading the engine…</p></div>

## What this shows

The frame is run state. The plot on this page is built once from your data and re-rendered for every width, height, charset, and color mode you choose — nothing about the plot changes, and the layout re-resolves each time: ticks re-search for a labeling that fits, the legend and title shed when the frame is small, and the labels stay exact decimals throughout. Try `Plain` and read what a pipe would see.

The link in your address bar carries the state: send it to someone and they open the same plot.
