# wasm-gerber-renderer

WebGL2 Gerber renderer powered by the `wasm-gerber-viewer` Rust/WASM parser and renderer.

The package provides:

- Browser canvas rendering from Gerber source strings, `File`, `Blob`, `ArrayBuffer`, or `Uint8Array` inputs
- Node.js PNG rendering through a headless WebGL2 context
- A `gerber-renderer` CLI for rendering Gerber files to PNG
- Bundled `wasm-bindgen` output generated during packaging

The browser entrypoint uses the caller's WebGL2 canvas. The Node.js entrypoint
uses the same WASM/WebGL renderer, but needs a native WebGL2 context provider.

## Install

Browser users:

```bash
npm install wasm-gerber-renderer
```

CLI users need the renderer package and
[`node-gles-webgl2`](https://www.npmjs.com/package/node-gles-webgl2):

```bash
npm install -g wasm-gerber-renderer node-gles-webgl2
```

The same package is also published to GitHub Packages as
`@dsafdsaf132/wasm-gerber-renderer`:

```bash
npm config set @dsafdsaf132:registry https://npm.pkg.github.com
npm install @dsafdsaf132/wasm-gerber-renderer
```

When installing from GitHub Packages, replace import specifiers such as
`wasm-gerber-renderer` with `@dsafdsaf132/wasm-gerber-renderer`.

Browser usage does not need `node-gles-webgl2`.

## Browser Usage

```js
import { renderGerberToCanvas } from "wasm-gerber-renderer";

const canvas = document.querySelector("canvas");
const gerber = await file.text();

await renderGerberToCanvas(canvas, gerber, {
  background: "#05070c",
  padding: 24,
});
```

For repeated rendering, reuse a renderer instance:

```js
import { createGerberRenderer } from "wasm-gerber-renderer";

const renderer = await createGerberRenderer(canvas);

await renderer.withFrame({ width: 1200, height: 800, padding: 24 }, async () => {
  await renderer.renderLayer(topCopper, { color: [1, 0, 0] });
  await renderer.renderLayer(bottomCopper, { color: [0, 0.7, 1], alpha: 0.8 });
});
```

Browser APIs:

| API | Description |
| --- | --- |
| `renderGerberToCanvas(canvas, layers, frameOptions)` | One-shot render into an existing WebGL2-capable canvas. Use this for simple viewer or preview cases. |
| `renderGerberToPng(canvas, layers, frameOptions, exportOptions)` | Renders through a browser canvas and returns a PNG `Blob`. |
| `createGerberRenderer(canvas, rendererOptions)` | Creates a reusable renderer for rendering multiple frames or layers without reloading the WASM module every time. |
| `renderer.withFrame(frameOptions, callback)` | Starts a render frame, applies canvas/view options, runs the callback, and presents the rendered layers. |
| `renderer.renderLayer(layer, layerOptions)` | Adds one Gerber layer to the active frame. Layer options include `color`, `alpha`, `offsetX`, and `offsetY`. |
| `renderer.exportPng(exportOptions)` | Exports the last rendered browser frame as a PNG `Blob`. |
| `renderer.dispose()` | Releases the WebGL context when the renderer is no longer needed. |

## API Options

`frameOptions` control the output frame and renderer behavior:

| Option | Default | Description |
| --- | --- | --- |
| `width` | Browser canvas width, Node: `1200` | Output width in pixels. |
| `height` | Browser canvas height, Node: `800` | Output height in pixels. |
| `clear` | `true` | Clears the frame before rendering. Node always renders to a fresh buffer and does not support `false`. |
| `background` | `null` | Background color. Use `null` for transparent output, a CSS color string, or `[r, g, b, a]`. |
| `fit` | `true` | Fits all loaded layer bounds into the output frame. |
| `padding` | `0` | Pixel padding applied when `fit` is enabled. |
| `view` | `null` | Manual view override: `{ zoomX, zoomY, offsetX, offsetY }`. When provided, it takes precedence over `fit`. |
| `preserveArcRegions` | `true` | Keeps region arcs for exact arc-region rendering. Set `false` to approximate region arcs. |
| `arcTessellationQuality` | `1` | Arc approximation quality: `0` low, `1` normal, `2` high. |
| `minimumFeaturePixels` | `1` | Minimum rendered line/arc width in screen pixels. |
| `globalAlpha` | `0.7` | Global opacity multiplier applied to rendered layers. |
| `rendererOptions` | `{}` | Browser one-shot helpers only. Passed through when creating the renderer. |

`layerOptions` control a single layer:

| Option | Default | Description |
| --- | --- | --- |
| `color` | Automatic color cycle | Layer color. Browser accepts `[r, g, b]`; Node accepts `[r, g, b]` or a CSS color string. |
| `alpha` | `1` | Per-layer opacity before `globalAlpha` is applied. |
| `offsetX` | `0` | X offset applied while loading the layer geometry. |
| `offsetY` | `0` | Y offset applied while loading the layer geometry. |
| `name` | Source name or `Layer <id>` | Layer display name when using layer config objects such as `{ source, name }` or `{ path, name }`. |

`exportOptions` control PNG export:

| Option | Default | Description |
| --- | --- | --- |
| `type` | `image/png` | Browser export MIME type. Node always writes PNG. |
| `quality` | Browser default | Browser encoder quality passed to `canvas.toBlob`. |
| `background` | Last frame background | Export background override. Use `null` to keep transparency. |

`rendererOptions` control renderer creation:

| Option | Default | Description |
| --- | --- | --- |
| `wasmModule` | Bundled module | Preloaded WASM JS module. Most users do not need this. |
| `wasmModuleUrl` | Bundled module URL | URL used to import the WASM JS module. |
| `wasmBinaryUrl` | Bundled `.wasm` URL | Node-only binary URL used when initializing the WASM module. |
| `wasmInitInput` | `undefined` | Custom value passed to the WASM module initializer. |
| `contextAttributes` | Package defaults | WebGL context attributes. |
| `releaseContext` | `true` | Releases the WebGL context on `dispose()` when supported. |
| `glesModule` | Auto-loaded in Node | Node-only GLES module object. Normal CLI usage uses `node-gles-webgl2`. |
| `glesModuleName` | `node-gles-webgl2` fallback list | Node-only module name to load for the GLES runtime. |
| `gl` | Auto-created in Node | Node-only pre-created WebGL2 context. |

## Node.js Usage

Install `node-gles-webgl2` before using the Node.js entrypoint.

```js
import { renderGerberToPngFile } from "wasm-gerber-renderer/node";

await renderGerberToPngFile(
  "board.png",
  ["top.gbr", "bottom.gbr"],
  {
    width: 1200,
    height: 800,
    background: "#05070c",
    padding: 24,
  },
);
```

## CLI

After global installation, run the CLI directly:

```bash
gerber-renderer board.gbr -o board.png --width 1200 --height 800 --background '#05070c'
```

More complete example:

```bash
gerber-renderer top.gbr bottom.gbr \
  --output board.png \
  --width 1600 \
  --height 1000 \
  --background '#05070c' \
  --padding 32 \
  --alpha 0.7 \
  --minimum-feature-pixels 1
```

CLI options:

| Option | Default | Description |
| --- | --- | --- |
| `<input.gbr...>` | Required | One or more Gerber input files. Multiple files are rendered as separate layers in argument order. |
| `-o, --output <path>` | Required | PNG output path. Parent directories must already exist. |
| `--width <px>` | `1200` | Output canvas width in pixels. Must be a positive integer. |
| `--height <px>` | `800` | Output canvas height in pixels. Must be a positive integer. |
| `--padding <px>` | `0` | Extra screen-space padding used by fit-to-view. |
| `--background <color>` | Transparent | CSS background color, such as `#05070c`, `black`, or `rgba(0,0,0,0)`. |
| `--alpha <0-1>` | `0.7` | Global layer opacity applied while rendering. |
| `--minimum-feature-pixels <px>` | `1` | Minimum rendered line/arc width in screen pixels, useful for keeping very thin features visible. |
| `--approx-region-arcs` | Disabled | Converts region arcs to line segments before rendering instead of using the exact arc-region renderer. |
| `--arc-quality <0\|1\|2>` | `1` | Arc tessellation quality: `0` low, `1` normal, `2` high. Mainly relevant with `--approx-region-arcs`. |
| `--no-fit` | Disabled | Disables fit-to-view and renders with the renderer's identity view. |
| `-h, --help` | - | Prints CLI usage and exits. |

`--arc-quality` is used only with `--approx-region-arcs`. Quality values are
`0` for low, `1` for normal, and `2` for high.

## License

[MIT License](LICENSE)
