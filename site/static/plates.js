// The live gallery: the plots drawn in the browser by the engine compiled to
// WebAssembly, a cell grid beside the pixel panel, the code that built each
// one below it, and a last plate that is an instrument — a long line through
// M4 with a wheel-and-drag window and a render clock.

import init, {
  expand_preset,
  mapping_columns,
  raster_columns,
  render_pixels_columns,
} from "./wasm/malevich_js.js";
import { NAMED, ansi256, escapeHtml, formatMs } from "./ansi.js";

// ---------------------------------------------------------------------------
// The plots. Every figure is a Document plus its out-of-band columns; nothing
// here draws, the engine does.

const ASCII = {
  width: 72,
  height: 16,
  charset: "Ascii",
  color: "TrueColor",
  theme: { palette: ["Cyan", "Yellow", "Green", "Magenta", "Blue", "Red"] },
};

const RED_BLUE = {
  stops: [
    [202, 0, 32],
    [244, 165, 130],
    [247, 247, 247],
    [146, 197, 222],
    [5, 113, 176],
  ],
  midpoint: 0,
};

const VERMILION = { Rgb: [227, 66, 52] };
const SKY = { Rgb: [108, 153, 212] };

function ewma(values, alpha) {
  const out = new Float64Array(values.length);
  let mean = 0;
  let started = false;
  for (let i = 0; i < values.length; i++) {
    const value = values[i];
    if (!Number.isFinite(value)) {
      out[i] = Number.NaN;
      continue;
    }
    mean = started ? alpha * value + (1 - alpha) * mean : value;
    started = true;
    out[i] = mean;
  }
  return out;
}

// A deterministic xorshift so the synthetic data is the same on every load.
function lcg(seed) {
  let state = BigInt(seed);
  return () => {
    state ^= state << 13n;
    state ^= state >> 7n;
    state ^= state << 17n;
    state &= 0xffff_ffff_ffff_ffffn;
    return Number(state >> 11n) / 2 ** 53;
  };
}

function plot(spec, columns = []) {
  return {
    document: { version: 1, kind: "plot", spec },
    columns,
  };
}

function loss() {
  const steps = Float64Array.from({ length: 120 }, (_, i) => i);
  const train = Float64Array.from(
    steps,
    (s) => 3.8 * Math.exp(-0.035 * s) + 0.32 + 0.05 * Math.sin(s * 0.7),
  );
  const val = Float64Array.from(
    steps,
    (s) => 4.0 * Math.exp(-0.03 * s) + 0.55 + 0.08 * Math.cos(s * 0.35),
  );
  const trainSmooth = ewma(train, 0.9);
  const valSmooth = ewma(val, 0.9);
  return plot(
    {
      layers: [
        {
          Area: {
            x: { col: 0 },
            low: null,
            high: { col: 1 },
            horizontal: false,
            color: VERMILION,
            label: null,
            opacity: 0.13,
          },
        },
        {
          Line: {
            x: { col: 0 },
            y: { col: 1 },
            color: VERMILION,
            label: "train",
            style: "Pixels",
            glow: true,
          },
        },
        {
          Line: {
            x: { col: 0 },
            y: { col: 2 },
            color: SKY,
            label: "val",
            style: "Pixels",
            dash: "Dotted",
          },
        },
        {
          Rule: {
            orientation: { Horizontal: 0.5 },
            color: null,
            label: "target",
            dash: "Dashed",
          },
        },
      ],
      title: "loss (synthetic)",
      x: "Auto",
      y: "Auto",
      x_label: "step",
      y_label: "loss",
      x_domain: null,
      y_domain: null,
      colorbar: false,
    },
    [steps, trainSmooth, valSmooth],
  );
}

function correlation() {
  const features = ["age", "len", "dep", "mass", "veg", "kcal", "spd", "alt"];
  const n = features.length;
  const grid = Float64Array.from({ length: n * n }, (_, i) => {
    const row = Math.floor(i / n);
    const column = i % n;
    if (row === column) {
      return 1;
    }
    return Math.exp(Math.abs(row - column) * -0.35) * Math.cos((row + column) * 0.55);
  });
  const layers = [
    {
      Cells: {
        columns: n,
        values: { col: 0 },
        extents: null,
        colormap: RED_BLUE,
      },
    },
  ];
  for (let index = 0; index < grid.length; index++) {
    const coefficient = grid[index];
    const column = index % n;
    const row = Math.floor(index / n);
    const ink =
      Math.abs(coefficient) > 0.45 ? { Rgb: [235, 235, 230] } : { Rgb: [32, 32, 32] };
    layers.push({
      Text: {
        x: column,
        y: row,
        text: `${coefficient >= 0 ? "+" : ""}${coefficient.toFixed(2)}`,
        color: ink,
        align: "Center",
      },
    });
  }
  return plot(
    {
      layers,
      title: "feature correlation (synthetic)",
      x: { Bands: features },
      y: { Bands: features },
      x_label: null,
      y_label: null,
      x_domain: null,
      y_domain: null,
      colorbar: false,
    },
    [grid],
  );
}

function penguins() {
  const unit = lcg(0x9e3779b97f4a7c15n);
  return {
    adelie: Float64Array.from(
      { length: 80 },
      (_, i) => 170 + ((i * 17) % 23) * 0.4 + unit(),
    ),
    chinstrap: Float64Array.from(
      { length: 60 },
      (_, i) => 185 + ((i * 13) % 19) * 0.5 + unit(),
    ),
    gentoo: Float64Array.from(
      { length: 90 },
      (_, i) => 205 + ((i * 11) % 29) * 0.45 + unit(),
    ),
  };
}

// A preset expansion with a title set afterwards, the way the TypeScript rim
// applies furniture after `expand_preset`.
function titled(json, title) {
  const document = JSON.parse(json);
  document.spec.title = title;
  return { document, columns: [] };
}

function violins(wasm) {
  const { adelie, chinstrap, gentoo } = penguins();
  return titled(
    wasm.expand_preset(
      "violin",
      JSON.stringify({ categories: ["Adelie", "Chinstrap", "Gentoo"] }),
      [adelie, chinstrap, gentoo],
    ),
    "flipper length by species (synthetic)",
  );
}

function landscape(wasm) {
  const size = 24;
  const field = Float64Array.from({ length: size * size }, (_, i) => {
    const x = (i % size) / 4;
    const y = Math.floor(i / size) / 4;
    return (x - 3) ** 2 * 0.4 + (y - 2.6) ** 2 * 0.7 + Math.sin(x * 1.7) * Math.cos(y * 1.3) * 0.8;
  });
  const json = wasm.expand_preset("heatmap", JSON.stringify({ columns: size }), [field]);
  const document = JSON.parse(json);
  document.spec.title = "a loss landscape (synthetic)";
  document.spec.colorbar = true;
  return { document, columns: [] };
}

function languages() {
  return plot(
    {
      layers: [
        {
          Bars: {
            placement: { Bands: ["rust", "go", "python", "typescript", "zig"] },
            values: [68, 41, 55, 62, 12],
            color: null,
            label: null,
          },
        },
      ],
      title: "admired languages, % (synthetic)",
      x: "Auto",
      y: "Auto",
      x_label: null,
      y_label: null,
      x_domain: null,
      y_domain: null,
      colorbar: false,
    },
    [],
  );
}

function boxPlots(wasm) {
  const { adelie, chinstrap, gentoo } = penguins();
  return titled(
    wasm.expand_preset(
      "box_plot",
      JSON.stringify({ categories: ["Adelie", "Chinstrap", "Gentoo"] }),
      [adelie, chinstrap, gentoo],
    ),
    "flipper length, boxes (synthetic)",
  );
}

function scatter() {
  const unit = lcg(0xcafef00ddeadbeefn);
  const species = ["Adelie", "Chinstrap", "Gentoo"];
  const n = 180;
  const x = new Float64Array(n);
  const y = new Float64Array(n);
  const group = [];
  for (let i = 0; i < n; i++) {
    const kind = species[i % 3];
    group.push(kind);
    x[i] = 32 + (i % 3) * 4 + unit() * 3;
    y[i] = 170 + (i % 3) * 18 + unit() * 8;
  }
  return plot(
    {
      layers: [
        {
          Points: {
            x: { col: 0 },
            y: { col: 1 },
            color: null,
            label: null,
            style: "Dot",
            color_by: group,
          },
        },
      ],
      title: "bill depth vs flipper (synthetic)",
      x: "Auto",
      y: "Auto",
      x_label: "bill depth",
      y_label: "flipper",
      x_domain: null,
      y_domain: null,
      colorbar: false,
      palette: {
        colors: [
          { Rgb: [230, 159, 0] },
          { Rgb: [86, 180, 233] },
          { Rgb: [0, 158, 115] },
        ],
      },
    },
    [x, y],
  );
}

function density2d(wasm) {
  const unit = lcg(0x123456789abcdef0n);
  const x = Float64Array.from({ length: 2000 }, (_, i) => unit() * 4 + (i % 2) * 1.2);
  const y = Float64Array.from({ length: 2000 }, (_, i) => unit() * 3 + (i % 3) * 0.4);
  return titled(
    wasm.expand_preset("hist2d", "{}", [x, y]),
    "2D histogram (synthetic)",
  );
}

// The instrument's document: one line over the bound column, windowed by
// `domain` — a zoom is a domain window, the series never changes.
function waveDocument(n, domain) {
  return {
    version: 1,
    kind: "plot",
    spec: {
      layers: [
        {
          Line: {
            x: null,
            y: { col: 0 },
            color: { Rgb: [227, 66, 52] },
            label: null,
            style: "Pixels",
            glow: true,
          },
        },
      ],
      title: `${n.toLocaleString("en-US")} points through M4`,
      x: "Auto",
      y: "Auto",
      x_label: "index",
      y_label: null,
      x_domain: domain,
      y_domain: null,
      colorbar: false,
    },
  };
}

const CHARTS = [
  {
    kicker: "Fig. 1",
    caption:
      "Fig. 1. Training loss as two series, a wash, and a dashed target.",
    width: 72,
    height: 16,
    rust: `let plot = Plot::new()
    .layer(Area::xy(&steps, &train).color(vermilion).opacity(0.13))
    .layer(Line::xy(&steps, &train).label("train").color(vermilion).glow())
    .layer(Line::xy(&steps, &val).label("val").color(sky).dash(Dash::Dotted))
    .layer(Rule::h(0.5).label("target").dash(Dash::Dashed))
    .title("loss (synthetic)")
    .x_label("step")
    .y_label("loss");`,
    typescript: `new Plot()
  .layer(Area.xy(steps, train).color(vermilion).opacity(0.13))
  .layer(Line.xy(steps, train).label("train").color(vermilion).glow())
  .layer(Line.xy(steps, val).label("val").color(sky).dash("Dotted"))
  .layer(Rule.h(0.5).label("target").dash("Dashed"))
  .title("loss (synthetic)")
  .xLabel("step")
  .yLabel("loss")`,
    build: () => loss(),
  },
  {
    kicker: "Fig. 2",
    caption:
      "Fig. 2. Feature correlation: band axes, a diverging map centered at zero, coefficients in the cells.",
    width: 72,
    height: 12,
    rust: `let colormap = Colormap::RED_BLUE.centered_at(0.0);
let mut plot = Plot::new()
    .layer(Cells::matrix(n, &grid).colormap(colormap.clone()))
    .x_scale(Scale::bands(features))
    .y_scale(Scale::bands(features))
    .title("feature correlation (synthetic)");
for (index, &c) in grid.iter().enumerate() {
    plot = plot.layer(Text::at(col, row, format!("{c:+.2}")).align(Align::Center));
}`,
    typescript: `const colormap = Colormap.centeredAt(Colormap.RED_BLUE, 0);
let plot = new Plot()
  .layer(Cells.matrix(n, grid).colormap(colormap))
  .xScale({ bands: features })
  .yScale({ bands: features })
  .title("feature correlation (synthetic)");
for (const [i, c] of grid.entries()) {
  plot = plot.layer(Text.at(i % n, Math.floor(i / n), format(c)).align("Center"));
}`,
    build: () => correlation(),
  },
  {
    kicker: "Fig. 3",
    caption:
      "Fig. 3. Flipper length as violins — a KDE per species. Pixels keep the shoulder; ascii keeps the shape.",
    width: 64,
    height: 16,
    rust: `violin(
    ["Adelie", "Chinstrap", "Gentoo"],
    [adelie, chinstrap, gentoo],
)
.title("flipper length by species (synthetic)")`,
    typescript: `violin(
  ["Adelie", "Chinstrap", "Gentoo"],
  [adelie, chinstrap, gentoo],
).title("flipper length by species (synthetic)")`,
    build: (wasm) => violins(wasm),
  },
  {
    kicker: "Fig. 4",
    caption:
      "Fig. 4. A loss landscape as a heatmap. Ascii is a shade grid; the pixel panel interpolates the field.",
    width: 56,
    height: 18,
    rust: `heatmap(24, &field)
    .colorbar()
    .title("a loss landscape (synthetic)")`,
    typescript: `heatmap(24, field)
  .colorbar()
  .title("a loss landscape (synthetic)")`,
    build: (wasm) => landscape(wasm),
  },
  {
    kicker: "Fig. 5",
    caption: "Fig. 5. Categorical bars from a zero baseline.",
    width: 64,
    height: 14,
    rust: `bar(
    ["rust", "go", "python", "typescript", "zig"],
    &[68.0, 41.0, 55.0, 62.0, 12.0][..],
)
.title("admired languages, % (synthetic)")`,
    typescript: `bar(
  ["rust", "go", "python", "typescript", "zig"],
  [68, 41, 55, 62, 12],
).title("admired languages, % (synthetic)")`,
    build: () => languages(),
  },
  {
    kicker: "Fig. 6",
    caption: "Fig. 6. The same three groups as box plots: type-7 quartiles, Tukey whiskers.",
    width: 64,
    height: 16,
    rust: `box_plot(
    ["Adelie", "Chinstrap", "Gentoo"],
    [adelie, chinstrap, gentoo],
)
.title("flipper length, boxes (synthetic)")`,
    typescript: `boxPlot(
  ["Adelie", "Chinstrap", "Gentoo"],
  [adelie, chinstrap, gentoo],
).title("flipper length, boxes (synthetic)")`,
    build: (wasm) => boxPlots(wasm),
  },
  {
    kicker: "Fig. 7",
    caption: "Fig. 7. A scatter with a color_by channel. Okabe–Ito colors; the groups stay separable.",
    width: 64,
    height: 16,
    rust: `Plot::new()
    .layer(Points::xy(&x, &y).color_by(&group))
    .title("bill depth vs flipper (synthetic)")
    .x_label("bill depth")
    .y_label("flipper")`,
    typescript: `new Plot()
  .layer(Points.xy(x, y).colorBy(group))
  .title("bill depth vs flipper (synthetic)")
  .xLabel("bill depth")
  .yLabel("flipper")`,
    build: () => scatter(),
  },
  {
    kicker: "Fig. 8",
    caption: "Fig. 8. A 2D histogram: two thousand points reduced onto the raster.",
    width: 56,
    height: 16,
    rust: `hist2d(&x, &y).title("2D histogram (synthetic)")`,
    typescript: `hist2d(x, y).title("2D histogram (synthetic)")`,
    build: (wasm) => density2d(wasm),
  },
  {
    kicker: "Fig. 9",
    interactive: true,
    caption:
      "Fig. 9. A long line through M4. Wheel zooms at the cursor; drag pans. The series does not change — only the window does.",
    width: 72,
    height: 16,
    rust: `line(&wave[..]).title("1,000,000 points through M4")
// a zoom is a domain window; M4 re-aggregates to the columns.`,
    typescript: `line(wave).title("1,000,000 points through M4")
// Plot.viewport({ x: [lo, hi] }) — M4 walks the visible window.`,
    build: () => ({ document: waveDocument(1_000_000, null), columns: [] }),
  },
];

// ---------------------------------------------------------------------------
// The packed raster, drawn as HTML. `fg`/`bg` carry four bytes per cell: a
// tag (0 default, 1 named, 2 indexed, 3 rgb) and up to three components.

function hex(value) {
  return value.toString(16).padStart(2, "0");
}

function cssColor(bytes, offset) {
  const tag = bytes[offset] ?? 0;
  if (tag === 1) {
    return NAMED[bytes[offset + 1] ?? 0] ?? null;
  }
  if (tag === 2) {
    return ansi256(bytes[offset + 1] ?? 0);
  }
  if (tag === 3) {
    const r = bytes[offset + 1] ?? 0;
    const g = bytes[offset + 2] ?? 0;
    const b = bytes[offset + 3] ?? 0;
    return `#${hex(r)}${hex(g)}${hex(b)}`;
  }
  return null;
}

function rasterHtml(packed) {
  const width = packed.width;
  const height = packed.height;
  const glyphs = [...packed.glyphs];
  const fg = packed.fg;
  const bg = packed.bg;
  const columns = packed.columns;
  const rows = [];
  for (let row = 0; row < height; row++) {
    let html = "";
    let current = "";
    let run = "";
    const flush = () => {
      if (!run) {
        return;
      }
      if (current) {
        html += `<span style="${current}">${run}</span>`;
      } else {
        html += run;
      }
      run = "";
    };
    for (let column = 0; column < width; column++) {
      const i = row * width + column;
      // A zero-width cell is the tail of a wide glyph; it draws nothing.
      if ((columns[i] ?? 1) === 0) {
        continue;
      }
      const glyph = glyphs[i] ?? " ";
      const foreground = cssColor(fg, i * 4);
      const background = cssColor(bg, i * 4);
      const style = [
        foreground && glyph !== " " ? `color:${foreground}` : "",
        background ? `background-color:${background}` : "",
      ]
        .filter(Boolean)
        .join(";");
      if (style !== current) {
        flush();
        current = style;
      }
      run += escapeHtml(glyph);
    }
    flush();
    rows.push(html.replace(/(?:&nbsp;| )+$/g, "").replace(/ +$/g, ""));
  }
  return rows.join("\n");
}

// The pixel path emits the iTerm2 protocol: an OSC 1337 with a base64 PNG.
// That PNG is the panel a capable terminal would place.
function itermPngUrl(payload) {
  const match = payload.match(/\x1b]1337;File=[^:]*:([A-Za-z0-9+/=\n]+)\x07/);
  if (!match) {
    throw new Error("pixel render did not carry an iTerm2 PNG");
  }
  const binary = atob(match[1].replaceAll("\n", ""));
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
}

function frameFor(chart) {
  return {
    ...ASCII,
    width: chart.width,
    height: chart.height,
  };
}

// ---------------------------------------------------------------------------
// The figures.

function slug(text) {
  return text.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}

function listing(chart) {
  const id = slug(chart.kicker);
  const rustId = `${id}-rust`;
  const tsId = `${id}-ts`;
  const tablist = `${id}-tabs`;
  return `<div class="listing">
      <div class="tabs" role="tablist" aria-label="Source language">
        <button type="button" role="tab" id="${tablist}-rust" aria-controls="${rustId}" aria-selected="true">Rust</button>
        <button type="button" role="tab" id="${tablist}-ts" aria-controls="${tsId}" aria-selected="false" tabindex="-1">TypeScript</button>
      </div>
      <pre id="${rustId}" role="tabpanel" aria-labelledby="${tablist}-rust"><code>${escapeHtml(chart.rust)}</code></pre>
      <pre id="${tsId}" role="tabpanel" aria-labelledby="${tablist}-ts" hidden><code>${escapeHtml(chart.typescript)}</code></pre>
    </div>`;
}

// A tablist: click selects; arrow keys move between tabs, as the ARIA pattern
// asks, so the listing is reachable without a mouse.
function bindTabs(figure) {
  const buttons = [...figure.querySelectorAll('[role="tab"]')];
  const panels = [...figure.querySelectorAll('[role="tabpanel"]')];
  const select = (button) => {
    for (const other of buttons) {
      const selected = other === button;
      other.setAttribute("aria-selected", String(selected));
      other.tabIndex = selected ? 0 : -1;
    }
    for (const panel of panels) {
      panel.hidden = panel.id !== button.getAttribute("aria-controls");
    }
  };
  buttons.forEach((button, index) => {
    button.addEventListener("click", () => select(button));
    button.addEventListener("keydown", (event) => {
      const step = event.key === "ArrowRight" ? 1 : event.key === "ArrowLeft" ? -1 : 0;
      if (!step) {
        return;
      }
      event.preventDefault();
      const next = buttons[(index + step + buttons.length) % buttons.length];
      select(next);
      next.focus();
    });
  });
}

function paintImage(figure, pixels, alt) {
  const img = figure.querySelector("img");
  const previous = img.src;
  if (alt !== undefined) {
    img.alt = alt;
  }
  img.src = itermPngUrl(pixels);
  if (previous.startsWith("blob:")) {
    URL.revokeObjectURL(previous);
  }
}

function paintPlate(figure, packed, pixels, alt) {
  figure.querySelector(".chart").innerHTML = rasterHtml(packed);
  paintImage(figure, pixels, alt);
}

function pairOf(value) {
  if (!value) {
    return undefined;
  }
  if (Array.isArray(value) && value.length >= 2) {
    return [Number(value[0]), Number(value[1])];
  }
  return undefined;
}

function fillWave(n) {
  const y = new Float64Array(n);
  for (let i = 0; i < n; i++) {
    y[i] = Math.sin(i * 0.0002) * Math.cos(i * 0.000013) * 8;
  }
  return y;
}

// The M4 instrument: size buttons, a reset, wheel zoom anchored at the
// cursor, drag pan, and a readout with the cell and pixel render clocks. The
// cell plate redraws on every gesture; the pixel panel follows after the
// gesture settles.
function mountInstrument(figure, chart) {
  const sizes = [100_000, 1_000_000, 10_000_000];
  const state = {
    n: 1_000_000,
    domain: null,
    series: new Map(),
    mapping: undefined,
    pixelTimer: 0,
    lastPixels: undefined,
  };
  const frame = frameFor(chart);
  const frameJson = JSON.stringify(frame);
  const plate = figure.querySelector(".plate-live");
  const controls = document.createElement("div");
  controls.className = "instrument";
  controls.innerHTML = `
    <div class="sizes" role="group" aria-label="Series length">
      ${sizes
        .map(
          (n) =>
            `<button type="button" data-n="${n}" aria-pressed="${n === state.n}">${n === 100_000 ? "100k" : n === 1_000_000 ? "1M" : "10M"}</button>`,
        )
        .join("")}
    </div>
    <button type="button" data-reset>reset view</button>
    <p class="hint">wheel zooms · drag pans</p>
  `;
  const readout = document.createElement("p");
  readout.className = "readout";
  figure.querySelector("figcaption").after(controls);
  controls.after(readout);

  const series = (n) => {
    let values = state.series.get(n);
    if (!values) {
      values = fillWave(n);
      state.series.set(n, values);
    }
    return values;
  };

  const setMapping = (documentJson, columns) => {
    state.mapping?.free();
    state.mapping = mapping_columns(documentJson, frameJson, columns);
  };

  const paintPixels = (documentJson, columns) => {
    const t0 = performance.now();
    const pixels = render_pixels_columns(
      documentJson,
      frameJson,
      columns,
      "iterm2",
      8,
      16,
    );
    const pixelMs = performance.now() - t0;
    paintImage(figure, pixels);
    return pixelMs;
  };

  const draw = (withPixels) => {
    const values = series(state.n);
    const documentJson = JSON.stringify(waveDocument(state.n, state.domain));
    const columns = [values];
    const t0 = performance.now();
    const packed = raster_columns(documentJson, frameJson, columns);
    const cellMs = performance.now() - t0;
    figure.querySelector(".chart").innerHTML = rasterHtml(packed);
    setMapping(documentJson, columns);
    let pixelMs = state.lastPixels;
    if (withPixels) {
      pixelMs = paintPixels(documentJson, columns);
      state.lastPixels = pixelMs;
    } else {
      window.clearTimeout(state.pixelTimer);
      state.pixelTimer = window.setTimeout(() => draw(true), 90);
    }
    const windowLabel = state.domain
      ? `${state.mapping.formatX(state.domain[0])} – ${state.mapping.formatX(state.domain[1])}`
      : "full";
    const pixelBit =
      pixelMs === undefined ? "" : ` · pixels ${formatMs(pixelMs)}`;
    readout.textContent = `${state.n.toLocaleString("en-US")} points · cells ${formatMs(cellMs)}${pixelBit} · ${windowLabel}`;
  };

  // The data x under a pointer: the frame column under it, asked of the
  // mapping at the middle row.
  const dataXAt = (clientX) => {
    if (!state.mapping) {
      return undefined;
    }
    const pre = figure.querySelector(".chart");
    const rect = pre.getBoundingClientRect();
    if (rect.width <= 0) {
      return undefined;
    }
    const column = Math.max(
      0,
      Math.min(frame.width - 1, Math.floor(((clientX - rect.left) / rect.width) * frame.width)),
    );
    const pair = pairOf(state.mapping.dataAt(column, Math.floor(frame.height / 2)));
    return pair?.[0];
  };

  const applyViewport = (next) => {
    const x = pairOf(next.x);
    next.free();
    state.domain = x ?? null;
    draw(false);
  };

  plate.addEventListener(
    "wheel",
    (event) => {
      event.preventDefault();
      if (!state.mapping) {
        return;
      }
      const anchor = dataXAt(event.clientX);
      if (!Number.isFinite(anchor)) {
        return;
      }
      const factor = event.deltaY > 0 ? 1.18 : 1 / 1.18;
      const viewport = state.mapping.viewport();
      const zoomed = viewport.zoomX(factor, anchor);
      viewport.free();
      applyViewport(zoomed);
    },
    { passive: false },
  );

  let drag = null;
  plate.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) {
      return;
    }
    drag = { x: event.clientX };
    event.currentTarget.setPointerCapture(event.pointerId);
  });
  plate.addEventListener("pointerup", () => {
    drag = null;
  });
  plate.addEventListener("pointercancel", () => {
    drag = null;
  });
  plate.addEventListener("pointermove", (event) => {
    if (!drag || !state.mapping) {
      return;
    }
    const width = plate.getBoundingClientRect().width;
    const fraction = (drag.x - event.clientX) / Math.max(width, 1);
    drag.x = event.clientX;
    if (Math.abs(fraction) < 1e-6) {
      return;
    }
    const viewport = state.mapping.viewport();
    const panned = viewport.panX(fraction);
    viewport.free();
    applyViewport(panned);
  });

  controls.querySelector("[data-reset]").addEventListener("click", () => {
    state.domain = null;
    draw(true);
  });
  for (const button of controls.querySelectorAll("[data-n]")) {
    button.addEventListener("click", () => {
      state.n = Number(button.dataset.n);
      state.domain = null;
      for (const other of controls.querySelectorAll("[data-n]")) {
        other.setAttribute("aria-pressed", String(other === button));
      }
      readout.textContent = "sampling…";
      window.setTimeout(() => draw(true), 20);
    });
  }

  draw(true);
}

async function main() {
  const status = document.getElementById("status");
  const root = document.getElementById("figures");
  if (!root) {
    return;
  }
  const fail = (error) => {
    const message = error instanceof Error ? error.message : String(error);
    const node = status ?? document.createElement("p");
    node.className = "status error";
    node.textContent =
      message.includes("Failed to fetch") || /wasm/i.test(message)
        ? "WebAssembly is missing. From the repo root: ./site/build.sh, then serve site/dist."
        : message;
    if (!status) {
      root.prepend(node);
    }
    console.error(error);
  };
  try {
    await init();
    const api = { expand_preset, raster_columns, render_pixels_columns };
    for (const chart of CHARTS) {
      const built = chart.build(api);
      const figure = document.createElement("figure");
      figure.innerHTML = `
        <div class="plate-live">
          <div class="pane">
            <p class="label">ascii</p>
            <pre class="chart"></pre>
          </div>
          <div class="pane">
            <p class="label">pixels</p>
            <img alt="${escapeHtml(chart.kicker)}: pixel panel" />
          </div>
        </div>
        <figcaption>${escapeHtml(chart.caption)}</figcaption>
        ${listing(chart)}
      `;
      bindTabs(figure);
      if (chart.interactive) {
        figure.id = "live";
      }
      root.append(figure);
      if (chart.interactive) {
        mountInstrument(figure, chart);
        continue;
      }
      const documentJson = JSON.stringify(built.document);
      const frameJson = JSON.stringify(frameFor(chart));
      const packed = raster_columns(documentJson, frameJson, built.columns);
      const pixels = render_pixels_columns(
        documentJson,
        frameJson,
        built.columns,
        "iterm2",
        8,
        16,
      );
      paintPlate(figure, packed, pixels, `${chart.kicker}: pixel panel`);
    }
    status?.remove();
  } catch (error) {
    fail(error);
  }
}

main();
