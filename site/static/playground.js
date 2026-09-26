// The playground: a live editor over the wasm build. Paste columns of data,
// pick a preset and its furniture, and see the plot the engine renders — and
// beside it the Rust program and the kaz command line that would draw the
// same thing, with the data inlined the way `kaz --emit-code` does. The state
// lives in the URL hash, so a playground can be linked.

import init, { expand_preset, render_columns } from "./wasm/malevich_js.js";
import { ansiToHtml, escapeHtml, formatMs } from "./ansi.js";

const PALETTE = ["Cyan", "Yellow", "Green", "Magenta", "Blue", "Red"];

const PRESETS = [
  "line",
  "scatter",
  "bar",
  "hist",
  "density",
  "ecdf",
  "stairs",
  "box",
  "violin",
  "heatmap",
  "hist2d",
  "table",
  "describe",
  "sparkline",
];

const CHARSETS = ["Quadrants", "Octants", "Sextants", "Braille", "HalfBlocks", "Ascii"];
const CHARSET_LABELS = {
  Quadrants: "Quadrants",
  Octants: "Octants",
  Sextants: "Sextants",
  Braille: "Braille",
  HalfBlocks: "Half blocks",
  Ascii: "ASCII",
};
// kaz's names for the same tiers.
const KAZ_CHARSETS = {
  Quadrants: "quad",
  Octants: "octant",
  Sextants: "sextant",
  Braille: "braille",
  HalfBlocks: "half",
  Ascii: "ascii",
};

const COLORS = ["TrueColor", "Ansi256", "Ansi16", "Plain"];
const COLOR_LABELS = { TrueColor: "TrueColor", Ansi256: "256 colors", Ansi16: "16 colors", Plain: "Plain" };

const LIMITS = { width: [12, 240], height: [3, 80] };

// ---------------------------------------------------------------------------
// Sample data: each is text as a reader might paste it, with the preset that
// suits it.

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

const SAMPLES = {
  sine: {
    preset: "line",
    text: () =>
      Array.from({ length: 48 }, (_, i) => (10 + 10 * Math.sin(i * 0.16)).toFixed(2)).join("\n"),
  },
  "two series": {
    preset: "line",
    text: () => {
      const rows = ["train val"];
      for (let s = 0; s < 60; s++) {
        const train = 3.8 * Math.exp(-0.07 * s) + 0.32 + 0.05 * Math.sin(s * 0.7);
        const val = 4.0 * Math.exp(-0.06 * s) + 0.55 + 0.08 * Math.cos(s * 0.35);
        rows.push(`${train.toFixed(3)} ${val.toFixed(3)}`);
      }
      return rows.join("\n");
    },
  },
  latencies: {
    preset: "hist",
    text: () => {
      // Log-normal-ish: a Box–Muller normal in the log, around 40 ms.
      const unit = xorshift(0x9e3779b97f4a7c15n);
      const rows = ["ms"];
      for (let i = 0; i < 200; i++) {
        const u = Math.max(unit(), 1e-12);
        const v = unit();
        const normal = Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v);
        rows.push(Math.exp(Math.log(40) + 0.45 * normal).toFixed(1));
      }
      return rows.join("\n");
    },
  },
  categories: {
    preset: "bar",
    text: () => "rust 68\ngo 41\npython 55\ntypescript 62\nzig 12",
  },
  matrix: {
    preset: "heatmap",
    text: () => {
      const rows = [];
      for (let y = 0; y < 8; y++) {
        const row = [];
        for (let x = 0; x < 8; x++) {
          const bump = Math.exp(-((x - 3.5) ** 2 + (y - 3.5) ** 2) / 6);
          row.push((bump + 0.15 * Math.sin(x * 1.3) * Math.cos(y * 0.9)).toFixed(2));
        }
        rows.push(row.join(" "));
      }
      return rows.join("\n");
    },
  },
};

// ---------------------------------------------------------------------------
// Parsing. Rows are lines; fields are separated by whitespace or commas; a
// `#` line is a comment; a first row with no numeric token is a header that
// names the series. A field that does not parse is a gap in a numeric column,
// and a column with no numbers at all is a label column.

function parseNumber(token) {
  if (/^[+-]?(nan|inf|infinity)$/i.test(token)) {
    return Number.NaN;
  }
  const value = Number(token);
  return token !== "" && !Number.isNaN(value) ? value : undefined;
}

function parseTable(text) {
  const rows = [];
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) {
      continue;
    }
    rows.push(line.split(/[\s,]+/).filter(Boolean));
  }
  let header = null;
  if (rows.length > 1 && rows[0].every((token) => parseNumber(token) === undefined)) {
    header = rows.shift();
  }
  const width = rows.reduce((max, row) => Math.max(max, row.length), 0);
  const columns = [];
  for (let index = 0; index < width; index++) {
    const text = rows.map((row) => row[index] ?? "");
    const parsed = text.map(parseNumber);
    const numeric = parsed.some((value) => value !== undefined);
    columns.push({
      index,
      name: header?.[index] ?? null,
      text,
      values: Float64Array.from(parsed, (value) => value ?? Number.NaN),
      numeric,
    });
  }
  return { header, rows, width, columns };
}

// ---------------------------------------------------------------------------
// Composition: a preset over a parsed table. Each branch yields the Document
// and its bound columns, the Rust bindings and chart expression, and the kaz
// shape. Furniture is applied afterwards, to all three at once.

function finite(values) {
  return Float64Array.from(values.filter((value) => Number.isFinite(value)));
}

const KEYWORDS = new Set([
  "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if",
  "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static",
  "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async", "await",
  "dyn", "box", "plot", "frame",
]);

// A Rust identifier from a header name, unique within the program.
function identifier(name, fallback, used) {
  let base = (name ?? "").toLowerCase().replace(/[^a-z0-9]+/g, "_").replace(/^_+|_+$/g, "");
  if (!base || /^[0-9]/.test(base)) {
    base = fallback;
  }
  if (KEYWORDS.has(base)) {
    base = `${base}_`;
  }
  let candidate = base;
  for (let n = 2; used.has(candidate); n++) {
    candidate = `${base}${n}`;
  }
  used.add(candidate);
  return candidate;
}

function seriesName(column) {
  return column.name ?? `col ${column.index + 1}`;
}

function seriesLabel(column) {
  return column.name ?? null;
}

function plotDocument(spec) {
  return {
    version: 1,
    kind: "plot",
    spec: {
      title: null,
      x: "Auto",
      y: "Auto",
      x_label: null,
      y_label: null,
      x_domain: null,
      y_domain: null,
      colorbar: false,
      ...spec,
    },
  };
}

function preset(name, options, columns) {
  return JSON.parse(expand_preset(name, JSON.stringify(options), columns));
}

function compose(kind, table) {
  const numeric = table.columns.filter((column) => column.numeric);
  const labels = table.columns.find((column) => !column.numeric) ?? null;
  if (numeric.length === 0) {
    throw new Error("no numeric column in the data");
  }
  const used = new Set();
  const lets = [];
  const f64 = (name, fallback, values) => {
    const id = identifier(name, fallback, used);
    lets.push({ name: id, kind: "f64", values });
    return id;
  };
  // The kaz shape: the subcommand, its flags, the columns the plot used, and
  // an optional replacement for the data rows.
  const kaz = { command: kind, flags: [], uses: null, rows: null };

  switch (kind) {
    case "line": {
      const layers = numeric.map((column, i) => ({
        Line: { x: null, y: { col: i }, color: null, label: seriesLabel(column), style: "Pixels" },
      }));
      let chart;
      if (numeric.length === 1 && !numeric[0].name) {
        const id = f64(null, "values", numeric[0].values);
        chart = `malevich::line(&${id}[..])`;
      } else {
        chart = "malevich::Plot::new()";
        numeric.forEach((column, i) => {
          const id = f64(column.name, `series${i}`, column.values);
          const label = column.name ? `.label(${rustString(column.name)})` : "";
          chart += `\n        .layer(malevich::Line::y(&${id}[..])${label})`;
        });
        if (numeric.length > 1) {
          kaz.flags.push("--fmt", "y");
        }
      }
      kaz.uses = numeric;
      return {
        document: plotDocument({ layers }),
        columns: numeric.map((column) => column.values),
        lets,
        chart,
        kaz,
      };
    }
    case "scatter": {
      if (numeric.length === 1) {
        const id = f64(numeric[0].name, "values", numeric[0].values);
        const label = numeric[0].name ? `.label(${rustString(numeric[0].name)})` : "";
        kaz.uses = numeric;
        return {
          document: plotDocument({
            layers: [
              { Points: { x: null, y: { col: 0 }, color: null, label: seriesLabel(numeric[0]), style: "Dot" } },
            ],
          }),
          columns: [numeric[0].values],
          lets,
          chart: `malevich::Plot::new()\n        .layer(malevich::Points::y(&${id}[..])${label})`,
          kaz,
        };
      }
      const x = numeric[0];
      const ys = numeric.slice(1);
      const colorBy = labels && ys.length === 1 ? labels : null;
      const layers = ys.map((column, i) => {
        const points = { x: { col: 0 }, y: { col: i + 1 }, color: null, label: seriesLabel(column), style: "Dot" };
        if (colorBy) {
          points.color_by = colorBy.text;
        }
        return { Points: points };
      });
      const xId = f64(x.name, "x", x.values);
      let chart;
      if (ys.length === 1 && !colorBy && !ys[0].name) {
        const yId = f64(null, "y", ys[0].values);
        chart = `malevich::scatter(&${xId}[..], &${yId}[..])`;
      } else {
        chart = "malevich::Plot::new()";
        const yIds = ys.map((column, i) => f64(column.name, ys.length === 1 ? "y" : `y${i}`, column.values));
        let groups = null;
        if (colorBy) {
          groups = identifier(null, "groups", used);
          lets.push({ name: groups, kind: "str", values: colorBy.text });
          kaz.flags.push("--by", columnReference(table, colorBy));
        }
        ys.forEach((column, i) => {
          const label = column.name ? `.label(${rustString(column.name)})` : "";
          const by = groups ? `.color_by(${groups})` : "";
          chart += `\n        .layer(malevich::Points::xy(&${xId}[..], &${yIds[i]}[..])${label}${by})`;
        });
      }
      kaz.uses = colorBy ? [...numeric, colorBy] : numeric;
      return {
        document: plotDocument({ layers }),
        columns: [x.values, ...ys.map((column) => column.values)],
        lets,
        chart,
        kaz,
      };
    }
    case "bar": {
      const values = numeric[0];
      const names = labels ? labels.text : table.rows.map((_, i) => String(i + 1));
      const labelsId = identifier(null, "labels", used);
      lets.push({ name: labelsId, kind: "str", values: names });
      const valuesId = f64(values.name, "values", values.values);
      kaz.rows = names.map((name, i) => `${name} ${plain(values.values[i])}`);
      return {
        document: plotDocument({
          layers: [{ Bars: { placement: { Bands: names }, values: { col: 0 }, color: null, label: null } }],
        }),
        columns: [values.values],
        lets,
        chart: `malevich::bar(${labelsId}, &${valuesId}[..])`,
        kaz,
      };
    }
    case "hist":
    case "density":
    case "ecdf":
    case "stairs": {
      const column = numeric[0];
      const id = f64(column.name, "values", column.values);
      kaz.uses = [column];
      if (kind === "stairs") {
        kaz.command = null;
      }
      return {
        document: preset(kind, {}, [column.values]),
        columns: [],
        lets,
        chart: `malevich::${kind}(&${id}[..])`,
        kaz,
      };
    }
    case "box":
    case "violin":
    case "describe": {
      const groups = numeric.map((column) => finite(column.values));
      const names = numeric.map((column, i) => column.name ?? String(i + 1));
      kaz.uses = numeric;
      if (kind === "describe") {
        const namesId = identifier(null, "names", used);
        lets.push({ name: namesId, kind: "str", values: names });
        const ids = numeric.map((column, i) => f64(column.name, `group${i}`, groups[i]));
        return {
          document: preset("describe", { names }, groups),
          columns: [],
          lets,
          chart: `malevich::describe(${namesId}, [${ids.map((id) => `&${id}[..]`).join(", ")}])`,
          kaz,
        };
      }
      const groupsId = identifier(null, "groups", used);
      lets.push({ name: groupsId, kind: "groups", values: groups });
      const categoriesId = identifier(null, "categories", used);
      lets.push({ name: categoriesId, kind: "str", values: names });
      const rust = kind === "box" ? "box_plot" : "violin";
      return {
        document: preset(rust, { categories: names }, groups),
        columns: [],
        lets,
        chart: `malevich::${rust}(${categoriesId}, ${groupsId})`,
        kaz,
      };
    }
    case "heatmap": {
      // Row 0 of a matrix is the bottom row on a numeric y axis, so the rows
      // are reversed: the matrix displays the way it was typed, as kaz does.
      const columns = numeric.length;
      const rows = table.rows.length;
      const values = new Float64Array(columns * rows);
      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < columns; c++) {
          values[r * columns + c] = numeric[c].values[rows - 1 - r];
        }
      }
      const id = f64(null, "values", values);
      const document = preset("heatmap", { columns }, [values]);
      document.spec.colorbar = true;
      return {
        document,
        columns: [],
        lets,
        chart: `malevich::heatmap(${columns}, &${id}[..])\n        .colorbar()`,
        kaz,
      };
    }
    case "hist2d": {
      if (numeric.length < 2) {
        throw new Error("hist2d needs two numeric columns: x and y");
      }
      const xId = f64(numeric[0].name, "x", numeric[0].values);
      const yId = f64(numeric[1].name, "y", numeric[1].values);
      kaz.uses = numeric.slice(0, 2);
      return {
        document: preset("hist2d", {}, [numeric[0].values, numeric[1].values]),
        columns: [],
        lets,
        chart: `malevich::hist2d(&${xId}[..], &${yId}[..])`,
        kaz,
      };
    }
    case "table": {
      // kaz's shape: rows named by a first column that never parses, else
      // by position; columns named by the header, else by position.
      const namedRows =
        table.width > 1 && table.rows.length > 0 && table.rows.every((row) => parseNumber(row[0] ?? "") === undefined);
      const first = namedRows ? 1 : 0;
      const rowNames = table.rows.map((row, i) => (namedRows ? row[0] : String(i + 1)));
      const columnNames = [];
      for (let c = first; c < table.width; c++) {
        columnNames.push(table.header?.[c] ?? String(c + 1 - first));
      }
      const values = new Float64Array(rowNames.length * columnNames.length);
      table.rows.forEach((row, r) => {
        for (let c = first; c < table.width; c++) {
          values[r * columnNames.length + (c - first)] = parseNumber(row[c] ?? "") ?? Number.NaN;
        }
      });
      const rowsId = identifier(null, "rows", used);
      lets.push({ name: rowsId, kind: "str", values: rowNames });
      const columnsId = identifier(null, "columns", used);
      lets.push({ name: columnsId, kind: "str", values: columnNames });
      const valuesId = f64(null, "values", values);
      return {
        document: preset("table", { rows: rowNames, columns: columnNames }, [values]),
        columns: [],
        lets,
        chart: `malevich::table(${rowsId}, ${columnsId}, &${valuesId}[..])`,
        kaz,
      };
    }
    case "sparkline": {
      const column = numeric[0];
      const id = f64(column.name, "values", column.values);
      kaz.command = "spark";
      kaz.uses = [column];
      return {
        document: plotDocument({
          layers: [
            { Bars: { placement: { Spans: { start: 0, width: 1 } }, values: { col: 0 }, color: null, label: null } },
          ],
          axes: false,
        }),
        columns: [column.values],
        lets,
        chart: `malevich::sparkline(&${id}[..])`,
        kaz,
      };
    }
    default:
      throw new Error(`unknown preset ${kind}`);
  }
}

// How kaz names a column: its header name under -H, else its 0-based index.
function columnReference(table, column) {
  return table.header ? column.name : String(column.index);
}

// The furniture, applied to the document's spec.
function furnish(spec, state) {
  spec.title = state.title || null;
  spec.x_label = state.xLabel || null;
  spec.y_label = state.yLabel || null;
  if (state.logX) {
    spec.x = "Log";
  }
  if (state.logY) {
    spec.y = "Log";
  }
}

function frameJson(state) {
  return JSON.stringify({
    width: state.width,
    height: state.height,
    charset: state.charset,
    color: state.color,
    theme: { palette: PALETTE },
  });
}

// ---------------------------------------------------------------------------
// The equivalent Rust program.

function rustString(text) {
  return JSON.stringify(text).replace(/\\u([0-9a-fA-F]{4})/g, "\\u{$1}");
}

function rustFloat(value) {
  if (Number.isNaN(value)) {
    return "f64::NAN";
  }
  if (!Number.isFinite(value)) {
    return value > 0 ? "f64::INFINITY" : "f64::NEG_INFINITY";
  }
  const text = String(value);
  return /[.eE]/.test(text) ? text : `${text}.0`;
}

// A `vec![...]` on one line when it fits, else wrapped at a comfortable width.
function rustVec(items, indent) {
  const inline = `vec![${items.join(", ")}]`;
  if (indent.length + inline.length <= 88) {
    return inline;
  }
  const lines = [];
  let line = "";
  for (const item of items) {
    const piece = `${item},`;
    if (line && line.length + 1 + piece.length > 84) {
      lines.push(line);
      line = "";
    }
    line = line ? `${line} ${piece}` : piece;
  }
  if (line) {
    lines.push(line);
  }
  const inner = `${indent}    `;
  return `vec![\n${lines.map((l) => `${inner}${l}`).join("\n")}\n${indent}]`;
}

function rustLet(binding) {
  const indent = "    ";
  switch (binding.kind) {
    case "f64":
      return `${indent}let ${binding.name}: Vec<f64> = ${rustVec([...binding.values].map(rustFloat), indent)};`;
    case "str":
      return `${indent}let ${binding.name}: Vec<&str> = ${rustVec(binding.values.map(rustString), indent)};`;
    case "groups": {
      const groups = binding.values.map(
        (group) => `${indent}    ${rustVec([...group].map(rustFloat), `${indent}    `)},`,
      );
      return `${indent}let ${binding.name}: Vec<Vec<f64>> = vec![\n${groups.join("\n")}\n${indent}];`;
    }
    default:
      return "";
  }
}

function rustProgram(composed, state) {
  const lines = [
    "//! The equivalent malevich program, with the playground's data inlined.",
    "",
    "use malevich::Frame;",
    "",
    "fn main() {",
  ];
  for (const binding of composed.lets) {
    lines.push(rustLet(binding));
  }
  let chart = composed.chart;
  if (state.title) {
    chart += `\n        .title(${rustString(state.title)})`;
  }
  if (state.xLabel) {
    chart += `\n        .x_label(${rustString(state.xLabel)})`;
  }
  if (state.yLabel) {
    chart += `\n        .y_label(${rustString(state.yLabel)})`;
  }
  if (state.logX) {
    chart += "\n        .log_x()";
  }
  if (state.logY) {
    chart += "\n        .log_y()";
  }
  lines.push(`    let plot = ${chart};`);
  lines.push("    let mut frame = Frame::detect();");
  lines.push(`    frame.width = ${state.width};`);
  lines.push(`    frame.height = ${state.height};`);
  if (state.charset !== "Quadrants") {
    lines.push(`    frame.charset = malevich::Charset::${state.charset};`);
  }
  if (state.color !== "TrueColor") {
    lines.push(`    frame.color = malevich::ColorMode::${state.color};`);
  }
  lines.push('    println!("{}", plot.render(&frame));');
  lines.push("}");
  return lines.join("\n");
}

// ---------------------------------------------------------------------------
// The equivalent kaz command line: the data through a heredoc, the furniture
// as flags.

function shellQuote(text) {
  return /^[A-Za-z0-9_.,=:\/-]+$/.test(text) ? text : `'${text.replaceAll("'", "'\\''")}'`;
}

// A number the way the data would be typed: shortest round-trip, gaps as nan.
function plain(value) {
  return Number.isNaN(value) ? "nan" : String(value);
}

function kazCommand(composed, state, table) {
  const { kaz } = composed;
  if (!kaz.command) {
    return `# kaz has no ${state.preset} command; the Rust program above is the route.`;
  }
  const args = ["kaz", kaz.command];
  const header = table.header && !kaz.rows;
  if (header) {
    args.push("-H");
  }
  args.push(...kaz.flags);
  // Columns the plot used, when the table carries others kaz would pool.
  if (kaz.uses && !kaz.rows && kaz.uses.length < table.width) {
    args.push("--cols", kaz.uses.map((column) => columnReference(table, column)).join(","));
  }
  if (state.title) {
    args.push("-t", state.title);
  }
  if (state.xLabel) {
    args.push("--xlabel", state.xLabel);
  }
  if (state.yLabel) {
    args.push("--ylabel", state.yLabel);
  }
  if (state.logX) {
    args.push("--log-x");
  }
  if (state.logY) {
    args.push("--log-y");
  }
  args.push("-w", String(state.width), "-h", String(state.height));
  if (state.charset !== "Quadrants") {
    args.push("--charset", KAZ_CHARSETS[state.charset]);
  }
  if (state.color === "Plain") {
    args.push("--color", "never");
  }
  const rows = kaz.rows ?? [...(header ? [table.header.join(" ")] : []), ...table.rows.map((row) => row.join(" "))];
  const delimiter = rows.includes("EOF") ? "DATA" : "EOF";
  return `${args.map(shellQuote).join(" ")} <<'${delimiter}'\n${rows.join("\n")}\n${delimiter}`;
}

// ---------------------------------------------------------------------------
// State: the URL hash carries it as JSON, so a playground can be linked.

const DEFAULT = {
  data: SAMPLES.sine.text(),
  preset: "line",
  title: "sine",
  xLabel: "",
  yLabel: "",
  logX: false,
  logY: false,
  width: 72,
  height: 18,
  charset: "Quadrants",
  color: "TrueColor",
};

function clampNumber(value, [min, max], fallback) {
  const number = Math.round(Number(value));
  return Number.isFinite(number) ? Math.min(max, Math.max(min, number)) : fallback;
}

// Only well-typed fields make it in; anything else keeps the default.
function sanitize(raw) {
  const state = { ...DEFAULT };
  if (!raw || typeof raw !== "object") {
    return state;
  }
  for (const key of ["data", "title", "xLabel", "yLabel"]) {
    if (typeof raw[key] === "string") {
      state[key] = raw[key];
    }
  }
  for (const key of ["logX", "logY"]) {
    if (typeof raw[key] === "boolean") {
      state[key] = raw[key];
    }
  }
  if (PRESETS.includes(raw.preset)) {
    state.preset = raw.preset;
  }
  if (CHARSETS.includes(raw.charset)) {
    state.charset = raw.charset;
  }
  if (COLORS.includes(raw.color)) {
    state.color = raw.color;
  }
  state.width = clampNumber(raw.width, LIMITS.width, DEFAULT.width);
  state.height = clampNumber(raw.height, LIMITS.height, DEFAULT.height);
  return state;
}

function readHash() {
  const raw = window.location.hash.slice(1);
  if (!raw) {
    return null;
  }
  try {
    return sanitize(JSON.parse(decodeURIComponent(raw)));
  } catch {
    return null;
  }
}

function encodeHash(state) {
  return `#${encodeURIComponent(JSON.stringify(state))}`;
}

// ---------------------------------------------------------------------------
// The page.

function element(html) {
  const template = document.createElement("template");
  template.innerHTML = html.trim();
  return template.content.firstElementChild;
}

function options(values, labels, selected) {
  return values
    .map(
      (value) =>
        `<option value="${escapeHtml(value)}"${value === selected ? " selected" : ""}>${escapeHtml(labels[value] ?? value)}</option>`,
    )
    .join("");
}

function buildControls(state) {
  return element(`
    <div class="controls">
      <label>Data
        <textarea id="playground-data" rows="12" spellcheck="false" autocapitalize="off" autocomplete="off">${escapeHtml(state.data)}</textarea>
      </label>
      <div class="samples" role="group" aria-label="Sample data">
        ${Object.keys(SAMPLES)
          .map((name) => `<button type="button" data-sample="${escapeHtml(name)}">${escapeHtml(name)}</button>`)
          .join("")}
      </div>
      <div class="presets" role="group" aria-label="Preset">
        ${PRESETS.map(
          (name) =>
            `<button type="button" data-preset="${name}" aria-pressed="${name === state.preset}">${name}</button>`,
        ).join("")}
      </div>
      <label>Title <input type="text" id="playground-title" value="${escapeHtml(state.title)}"></label>
      <div class="row">
        <label>x label <input type="text" id="playground-xlabel" value="${escapeHtml(state.xLabel)}"></label>
        <label>y label <input type="text" id="playground-ylabel" value="${escapeHtml(state.yLabel)}"></label>
      </div>
      <div class="row">
        <label class="check"><input type="checkbox" id="playground-logx"${state.logX ? " checked" : ""}> log x</label>
        <label class="check"><input type="checkbox" id="playground-logy"${state.logY ? " checked" : ""}> log y</label>
      </div>
      <div class="row">
        <label>Width <input type="number" id="playground-width" min="${LIMITS.width[0]}" max="${LIMITS.width[1]}" value="${state.width}"></label>
        <label>Height <input type="number" id="playground-height" min="${LIMITS.height[0]}" max="${LIMITS.height[1]}" value="${state.height}"></label>
      </div>
      <div class="row">
        <label>Charset <select id="playground-charset">${options(CHARSETS, CHARSET_LABELS, state.charset)}</select></label>
        <label>Color <select id="playground-color">${options(COLORS, COLOR_LABELS, state.color)}</select></label>
      </div>
    </div>
  `);
}

function buildOutput() {
  return element(`
    <div class="output">
      <figure class="plate">
        <pre class="term" id="playground-plate">Loading the engine…</pre>
        <figcaption id="playground-caption"></figcaption>
      </figure>
      <p class="error" id="playground-error" hidden></p>
      <div class="code" data-lang="Rust"><pre><code id="playground-rust"></code></pre></div>
      <div class="code" data-lang="shell"><pre><code id="playground-kaz"></code></pre></div>
    </div>
  `);
}

// The copy button site.js gives code blocks; ours are built after it ran.
function addCopyButton(block) {
  const pre = block.querySelector("pre");
  const button = document.createElement("button");
  button.type = "button";
  button.className = "copy";
  button.textContent = "copy";
  button.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(pre.textContent ?? "");
      button.textContent = "copied";
    } catch {
      button.textContent = "select it";
    }
    window.setTimeout(() => {
      button.textContent = "copy";
    }, 1400);
  });
  block.append(button);
}

function mount(root, initial) {
  const state = { ...initial };
  root.classList.add("playground");
  root.replaceChildren(buildControls(state), buildOutput());

  const field = (id) => root.querySelector(`#playground-${id}`);
  const data = field("data");
  const title = field("title");
  const xLabel = field("xlabel");
  const yLabel = field("ylabel");
  const logX = field("logx");
  const logY = field("logy");
  const width = field("width");
  const height = field("height");
  const charset = field("charset");
  const color = field("color");
  const plate = field("plate");
  const caption = field("caption");
  const error = field("error");
  const rust = field("rust");
  const kaz = field("kaz");
  for (const block of root.querySelectorAll(".code")) {
    addCopyButton(block);
  }

  let lastHash = "";
  const render = () => {
    lastHash = encodeHash(state);
    window.history.replaceState(null, "", lastHash);
    try {
      const table = parseTable(state.data);
      const composed = compose(state.preset, table);
      furnish(composed.document.spec, state);
      // The code follows the request even when the engine then refuses it:
      // the reader sees what was asked and, below, why it was declined.
      rust.textContent = rustProgram(composed, state);
      kaz.textContent = kazCommand(composed, state, table);
      const t0 = performance.now();
      const ansi = render_columns(JSON.stringify(composed.document), frameJson(state), composed.columns);
      const ms = performance.now() - t0;
      plate.innerHTML = ansiToHtml(ansi);
      caption.textContent = `${state.preset} · ${state.width}×${state.height} · ${state.charset} · ${state.color} · ${formatMs(ms)}`;
      error.hidden = true;
      error.textContent = "";
    } catch (failure) {
      // A decode or render refusal, verbatim; the last good plate stays up.
      error.textContent = failure instanceof Error ? failure.message : String(failure);
      error.hidden = false;
    }
  };

  // Typing is debounced; everything else renders at once.
  let timer = 0;
  const soon = () => {
    window.clearTimeout(timer);
    timer = window.setTimeout(render, 80);
  };
  const now = () => {
    window.clearTimeout(timer);
    render();
  };

  data.addEventListener("input", () => {
    state.data = data.value;
    soon();
  });
  title.addEventListener("input", () => {
    state.title = title.value;
    soon();
  });
  xLabel.addEventListener("input", () => {
    state.xLabel = xLabel.value;
    soon();
  });
  yLabel.addEventListener("input", () => {
    state.yLabel = yLabel.value;
    soon();
  });
  logX.addEventListener("change", () => {
    state.logX = logX.checked;
    now();
  });
  logY.addEventListener("change", () => {
    state.logY = logY.checked;
    now();
  });
  const size = (input, key) => () => {
    state[key] = clampNumber(input.value, LIMITS[key], DEFAULT[key]);
    now();
  };
  width.addEventListener("input", size(width, "width"));
  height.addEventListener("input", size(height, "height"));
  charset.addEventListener("change", () => {
    state.charset = CHARSETS.includes(charset.value) ? charset.value : DEFAULT.charset;
    now();
  });
  color.addEventListener("change", () => {
    state.color = COLORS.includes(color.value) ? color.value : DEFAULT.color;
    now();
  });

  const presetButtons = [...root.querySelectorAll("[data-preset]")];
  const setPreset = (name) => {
    state.preset = name;
    for (const button of presetButtons) {
      button.setAttribute("aria-pressed", String(button.dataset.preset === name));
    }
  };
  for (const button of presetButtons) {
    button.addEventListener("click", () => {
      setPreset(button.dataset.preset);
      now();
    });
  }

  // A sample replaces the data, suggests its preset, and titles the plot
  // after itself unless the reader has written a title of their own.
  const sampleNames = new Set(Object.keys(SAMPLES));
  for (const button of root.querySelectorAll("[data-sample]")) {
    button.addEventListener("click", () => {
      const sample = SAMPLES[button.dataset.sample];
      if (!sample) {
        return;
      }
      state.data = sample.text();
      data.value = state.data;
      setPreset(sample.preset);
      if (!state.title || sampleNames.has(state.title)) {
        state.title = button.dataset.sample;
        title.value = state.title;
      }
      now();
    });
  }

  // A link pasted while the page is open: take its state.
  window.addEventListener("hashchange", () => {
    if (window.location.hash === lastHash) {
      return;
    }
    const next = readHash();
    if (!next) {
      return;
    }
    Object.assign(state, next);
    data.value = state.data;
    title.value = state.title;
    xLabel.value = state.xLabel;
    yLabel.value = state.yLabel;
    logX.checked = state.logX;
    logY.checked = state.logY;
    width.value = String(state.width);
    height.value = String(state.height);
    charset.value = state.charset;
    color.value = state.color;
    setPreset(state.preset);
    now();
  });

  render();
}

async function main() {
  const root = document.getElementById("playground");
  if (!root) {
    return;
  }
  const initial = readHash() ?? { ...DEFAULT };
  try {
    await init();
  } catch (failure) {
    const text = failure instanceof Error ? failure.message : String(failure);
    const note = document.createElement("p");
    note.className = "status error";
    note.textContent =
      text.includes("Failed to fetch") || /wasm/i.test(text)
        ? "WebAssembly is missing. From the repo root: ./site/build.sh, then serve site/dist."
        : text;
    root.replaceChildren(note);
    console.error(failure);
    return;
  }
  mount(root, initial);
}

main();
