// ANSI to HTML: the string a terminal would receive, decoded into colored
// spans the way a terminal emulator would show it. This is how the live pages
// show a color mode honestly — the encoder's own SGR bytes, drawn — since the
// packed raster always carries the resolved color. The named colors take the
// xterm defaults, the same table site/src/ansi.rs uses for the static plates.

/// xterm's default rendering of the sixteen ANSI colors.
export const NAMED = [
  "#000000",
  "#cd0000",
  "#00cd00",
  "#cdcd00",
  "#0000ee",
  "#cd00cd",
  "#00cdcd",
  "#e5e5e5",
  "#7f7f7f",
  "#ff0000",
  "#00ff00",
  "#ffff00",
  "#5c5cff",
  "#ff00ff",
  "#00ffff",
  "#ffffff",
];

/// The xterm 256-color palette: the sixteen named colors, the 6×6×6 cube,
/// and the 24-step grey ramp.
export function ansi256(index) {
  if (index < 16) {
    return NAMED[index];
  }
  if (index >= 232) {
    const level = 8 + (index - 232) * 10;
    return `rgb(${level}, ${level}, ${level})`;
  }
  const cube = index - 16;
  const ramp = [0, 95, 135, 175, 215, 255];
  const r = ramp[Math.floor(cube / 36) % 6];
  const g = ramp[Math.floor(cube / 6) % 6];
  const b = ramp[cube % 6];
  return `rgb(${r}, ${g}, ${b})`;
}

const ESCAPES = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#39;",
};

export function escapeHtml(text) {
  return String(text).replace(/[&<>"']/g, (character) => ESCAPES[character]);
}

/// A render clock for the readouts: two decimals under 10 ms, one under 100.
export function formatMs(ms) {
  if (ms < 10) {
    return `${ms.toFixed(2)} ms`;
  }
  if (ms < 100) {
    return `${ms.toFixed(1)} ms`;
  }
  return `${Math.round(ms)} ms`;
}

// The SGR state a run of text is drawn with.
function freshStyle() {
  return { foreground: null, background: null, bold: false };
}

function styleCss(style) {
  const parts = [];
  if (style.foreground) {
    parts.push(`color:${style.foreground}`);
  }
  if (style.background) {
    parts.push(`background:${style.background}`);
  }
  if (style.bold) {
    parts.push("font-weight:600");
  }
  return parts.join(";");
}

// Applies one SGR parameter list to `style`, in place. Handles reset, bold,
// the sixteen named colors on both channels, default-color resets, and the
// 256-color and truecolor extended forms.
function applySgr(style, codes) {
  if (codes.length === 0) {
    Object.assign(style, freshStyle());
    return;
  }
  for (let i = 0; i < codes.length; i++) {
    const code = codes[i];
    if (code === 0) {
      Object.assign(style, freshStyle());
    } else if (code === 1) {
      style.bold = true;
    } else if (code === 22) {
      style.bold = false;
    } else if (code >= 30 && code <= 37) {
      style.foreground = NAMED[code - 30];
    } else if (code >= 90 && code <= 97) {
      style.foreground = NAMED[code - 90 + 8];
    } else if (code >= 40 && code <= 47) {
      style.background = NAMED[code - 40];
    } else if (code >= 100 && code <= 107) {
      style.background = NAMED[code - 100 + 8];
    } else if (code === 39) {
      style.foreground = null;
    } else if (code === 49) {
      style.background = null;
    } else if (code === 38 || code === 48) {
      const channel = code === 38 ? "foreground" : "background";
      if (codes[i + 1] === 5 && codes.length > i + 2) {
        style[channel] = ansi256(codes[i + 2] & 0xff);
        i += 2;
      } else if (codes[i + 1] === 2 && codes.length > i + 4) {
        const r = codes[i + 2] & 0xff;
        const g = codes[i + 3] & 0xff;
        const b = codes[i + 4] & 0xff;
        style[channel] = `rgb(${r}, ${g}, ${b})`;
        i += 4;
      }
    }
  }
}

/// Decodes SGR-colored text into HTML: runs of text in spans carrying their
/// color as inline style, everything escaped. Other CSI sequences and OSC
/// strings (hyperlinks, images) are dropped.
export function ansiToHtml(text) {
  let html = "";
  let run = "";
  const style = freshStyle();
  const flush = () => {
    if (!run) {
      return;
    }
    const css = styleCss(style);
    html += css ? `<span style="${css}">${escapeHtml(run)}</span>` : escapeHtml(run);
    run = "";
  };
  let i = 0;
  while (i < text.length) {
    const character = text[i];
    if (character !== "\x1b") {
      run += character;
      i += 1;
      continue;
    }
    const next = text[i + 1];
    if (next === "[") {
      // CSI: parameters up to a final byte in 0x40–0x7e.
      let end = i + 2;
      while (end < text.length && !(text.charCodeAt(end) >= 0x40 && text.charCodeAt(end) <= 0x7e)) {
        end += 1;
      }
      if (text[end] === "m") {
        flush();
        const body = text.slice(i + 2, end);
        const codes = body === "" ? [] : body.split(";").map((code) => Number.parseInt(code, 10)).filter(Number.isFinite);
        applySgr(style, codes);
      }
      i = end + 1;
    } else if (next === "]") {
      // OSC: runs to BEL or ST (ESC \).
      let end = i + 2;
      while (end < text.length && text[end] !== "\x07" && !(text[end] === "\x1b" && text[end + 1] === "\\")) {
        end += 1;
      }
      i = text[end] === "\x07" ? end + 1 : end + 2;
    } else {
      i += 1;
    }
  }
  flush();
  return html;
}
