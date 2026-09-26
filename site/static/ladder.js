// The ladder, live: two optional figures on the terminals page, each driven
// by the wasm build. The explorer re-encodes one plot value for the charset
// and color mode chosen — nothing is redrawn, only the frame changes — and
// the resizer re-renders a plot as its frame shrinks and grows. Either figure
// may be absent from the page; both are guarded.

import init, { expand_preset, render_columns } from "./wasm/malevich_js.js";
import { ansiToHtml, formatMs } from "./ansi.js";

const PALETTE = ["Cyan", "Yellow", "Green", "Magenta", "Blue", "Red"];

function frameJson(width, height, charset, color) {
  return JSON.stringify({ width, height, charset, color, theme: { palette: PALETTE } });
}

// A deterministic xorshift so the synthetic data is the same on every load.
function xorshift(seed) {
  let state = BigInt(seed);
  return () => {
    state ^= state << 13n;
    state ^= state >> 7n;
    state ^= state << 17n;
    state &= 0xffff_ffff_ffff_ffffn;
    return Number(state >> 11n) / 2 ** 53;
  };
}

function ewma(values, alpha) {
  const out = new Float64Array(values.length);
  let mean = 0;
  for (let i = 0; i < values.length; i++) {
    mean = i === 0 ? values[i] : alpha * values[i] + (1 - alpha) * mean;
    out[i] = mean;
  }
  return out;
}

// ---------------------------------------------------------------------------
// The three plot values. Each is a Document JSON string plus the columns its
// `{ col: N }` series bind to; the stats preset inlines its own series.

// Training loss: two lines with labels, a wash under train, a dashed target.
function lines() {
  const steps = Float64Array.from({ length: 120 }, (_, i) => i);
  const train = ewma(
    Float64Array.from(steps, (s) => 3.8 * Math.exp(-0.035 * s) + 0.32 + 0.05 * Math.sin(s * 0.7)),
    0.9,
  );
  const val = ewma(
    Float64Array.from(steps, (s) => 4.0 * Math.exp(-0.03 * s) + 0.55 + 0.08 * Math.cos(s * 0.35)),
    0.9,
  );
  const vermilion = { Rgb: [227, 66, 52] };
  const sky = { Rgb: [108, 153, 212] };
  const document = {
    version: 1,
    kind: "plot",
    spec: {
      layers: [
        {
          Area: {
            x: { col: 0 },
            low: null,
            high: { col: 1 },
            horizontal: false,
            color: vermilion,
            label: null,
            opacity: 0.15,
          },
        },
        {
          Line: { x: { col: 0 }, y: { col: 1 }, color: vermilion, label: "train", style: "Pixels" },
        },
        {
          Line: { x: { col: 0 }, y: { col: 2 }, color: sky, label: "val", style: "Pixels" },
        },
        {
          Rule: { orientation: { Horizontal: 0.5 }, color: null, label: "target", dash: "Dashed" },
        },
      ],
      title: "loss",
      x: "Auto",
      y: "Auto",
      x_label: "step",
      y_label: "loss",
      x_domain: null,
      y_domain: null,
      colorbar: false,
    },
  };
  return { json: JSON.stringify(document), columns: [steps, train, val] };
}

// A 24×24 field through the heatmap preset, with a colorbar and a title.
function heatmap() {
  const size = 24;
  const field = Float64Array.from({ length: size * size }, (_, i) => {
    const x = (i % size) / 4;
    const y = Math.floor(i / size) / 4;
    return (x - 3) ** 2 * 0.4 + (y - 2.6) ** 2 * 0.7 + Math.sin(x * 1.7) * Math.cos(y * 1.3) * 0.8;
  });
  const document = JSON.parse(expand_preset("heatmap", JSON.stringify({ columns: size }), [field]));
  document.spec.title = "a loss landscape";
  document.spec.colorbar = true;
  return { json: JSON.stringify(document), columns: [] };
}

// Three species-like groups through a color_by channel, in Okabe–Ito.
function scatter() {
  const unit = xorshift(0xcafef00ddeadbeefn);
  const species = ["Adelie", "Chinstrap", "Gentoo"];
  const n = 150;
  const x = new Float64Array(n);
  const y = new Float64Array(n);
  const group = [];
  for (let i = 0; i < n; i++) {
    group.push(species[i % 3]);
    x[i] = 32 + (i % 3) * 4 + unit() * 3;
    y[i] = 170 + (i % 3) * 18 + unit() * 8;
  }
  const document = {
    version: 1,
    kind: "plot",
    spec: {
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
      title: "bill depth vs flipper",
      x: "Auto",
      y: "Auto",
      x_label: "bill depth",
      y_label: "flipper",
      x_domain: null,
      y_domain: null,
      colorbar: false,
      palette: {
        colors: [{ Rgb: [230, 159, 0] }, { Rgb: [86, 180, 233] }, { Rgb: [0, 158, 115] }],
      },
    },
  };
  return { json: JSON.stringify(document), columns: [x, y] };
}

// ---------------------------------------------------------------------------
// The figures.

function message(error) {
  return error instanceof Error ? error.message : String(error);
}

function checked(figure, name, fallback) {
  return figure.querySelector(`input[name="${name}"]:checked`)?.value ?? fallback;
}

// The explorer: radios for charset, color, and chart; one render per change,
// timed, decoded from the encoder's own escape bytes.
function mountExplorer(figure, charts) {
  const plate = document.getElementById("explorer-plate");
  const readout = document.getElementById("explorer-readout");
  if (!plate) {
    return;
  }
  const draw = () => {
    const charset = checked(figure, "charset", "Quadrants");
    const color = checked(figure, "color", "TrueColor");
    const chart = charts[checked(figure, "chart", "lines")] ?? charts.lines;
    try {
      const t0 = performance.now();
      const ansi = render_columns(chart.json, frameJson(72, 16, charset, color), chart.columns);
      const ms = performance.now() - t0;
      plate.innerHTML = ansiToHtml(ansi);
      if (readout) {
        readout.textContent = `${charset} · ${color} · ${formatMs(ms)}`;
      }
    } catch (error) {
      plate.textContent = message(error);
      if (readout) {
        readout.textContent = `${charset} · ${color} · refused`;
      }
    }
  };
  figure.addEventListener("change", (event) => {
    if (event.target instanceof HTMLInputElement && event.target.type === "radio") {
      draw();
    }
  });
  draw();
}

// The resizer: two ranges, the lines chart re-rendered at their size, with
// renders coalesced to one per animation frame.
function mountResizer(figure, chart) {
  const plate = document.getElementById("resizer-plate");
  const width = document.getElementById("resizer-width");
  const height = document.getElementById("resizer-height");
  const widthOut = document.getElementById("resizer-width-out");
  const heightOut = document.getElementById("resizer-height-out");
  if (!plate || !width || !height) {
    return;
  }
  const clamp = (input, fallback) => {
    const value = Number.parseInt(input.value, 10);
    const min = Number.parseInt(input.min, 10) || 1;
    const max = Number.parseInt(input.max, 10) || 4096;
    return Number.isFinite(value) ? Math.min(max, Math.max(min, value)) : fallback;
  };
  const draw = () => {
    const w = clamp(width, 80);
    const h = clamp(height, 18);
    if (widthOut) {
      widthOut.textContent = String(w);
    }
    if (heightOut) {
      heightOut.textContent = String(h);
    }
    try {
      const ansi = render_columns(chart.json, frameJson(w, h, "Quadrants", "TrueColor"), chart.columns);
      plate.innerHTML = ansiToHtml(ansi);
    } catch (error) {
      plate.textContent = message(error);
    }
  };
  let pending = 0;
  const schedule = () => {
    if (pending) {
      return;
    }
    pending = window.requestAnimationFrame(() => {
      pending = 0;
      draw();
    });
  };
  width.addEventListener("input", schedule);
  height.addEventListener("input", schedule);
  draw();
}

async function main() {
  const explorer = document.getElementById("explorer");
  const resizer = document.getElementById("resizer");
  if (!explorer && !resizer) {
    return;
  }
  try {
    await init();
  } catch (error) {
    const text = message(error);
    const note =
      text.includes("Failed to fetch") || /wasm/i.test(text)
        ? "WebAssembly is missing. From the repo root: ./site/build.sh, then serve site/dist."
        : text;
    for (const id of ["explorer-plate", "resizer-plate"]) {
      const plate = document.getElementById(id);
      if (plate) {
        plate.textContent = note;
      }
    }
    console.error(error);
    return;
  }
  const charts = { lines: lines(), heatmap: heatmap(), scatter: scatter() };
  if (explorer) {
    mountExplorer(explorer, charts);
  }
  if (resizer) {
    mountResizer(resizer, charts.lines);
  }
}

main();
