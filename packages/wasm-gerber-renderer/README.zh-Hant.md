<div align="center">

# wasm-gerber-renderer

[**`English`**](README.md) · [**`한국어`**](README.kr.md) · [**`简体中文`**](README.zh-Hans.md) · **`繁體中文`**

</div>

---

基於 `wasm-gerber-viewer` Rust/WASM parser 與 renderer 的 WebGL2 Gerber renderer。

這個套件提供：

- 在瀏覽器 canvas 中渲染 Gerber source string、`File`、`Blob`、`ArrayBuffer` 或 `Uint8Array` 輸入
- 透過 headless WebGL2 context 在 Node.js 中渲染 PNG，並支援直接寫入檔案或 stream
- 將 Gerber 檔案或 `.tar.gz`/`.tgz` archive 渲染成 PNG 的 `gerber-renderer` CLI
- 打包時產生並包含的 `wasm-bindgen` output

瀏覽器 entrypoint 使用呼叫方提供的 WebGL2 canvas。Node.js entrypoint 使用同一套 WASM/WebGL renderer，並預設透過 [`node-gles-webgl2`](https://github.com/dsafdsaf132/node-gles-webgl2) 提供 native WebGL2 context。

## 內容

- [安裝](#安裝)
- [平台支援](#平台支援)
- [瀏覽器用法](#瀏覽器用法)
- [Node.js 用法](#nodejs-用法)
- [CLI](#cli)
- [主要 API](#主要-api)
- [選項](#選項)
- [授權](#授權)

## 安裝

瀏覽器使用者：

```bash
npm install wasm-gerber-renderer
```

CLI 使用者需要 renderer 套件與 [`node-gles-webgl2`](https://www.npmjs.com/package/node-gles-webgl2)：

```bash
npm install -g wasm-gerber-renderer node-gles-webgl2
```

同一個套件也發布到 GitHub Packages，名稱為 `@dsafdsaf132/wasm-gerber-renderer`：

```bash
npm config set @dsafdsaf132:registry https://npm.pkg.github.com
npm install @dsafdsaf132/wasm-gerber-renderer
```

從 GitHub Packages 安裝時，請把 import specifier 中的 `wasm-gerber-renderer` 改成 `@dsafdsaf132/wasm-gerber-renderer`。

## 平台支援

瀏覽器渲染不依賴平台，使用呼叫方提供的 WebGL2 canvas。

Node.js 與 CLI 渲染透過 [`node-gles-webgl2`](https://github.com/dsafdsaf132/node-gles-webgl2) 支援：

- Linux x64
- Linux arm64
- macOS arm64
- Windows x64
- Windows arm64

預設的 `node-gles-webgl2` ANGLE prebuilt archive set 不支援 macOS x64。

## 瀏覽器用法

```js
import { renderGerberToCanvas } from "wasm-gerber-renderer";

const canvas = document.querySelector("canvas");
const gerber = await file.text();

await renderGerberToCanvas(canvas, gerber, {
  background: "#05070c",
  padding: 24,
});
```

重複渲染時請重用 renderer instance：

```js
import { createGerberRenderer } from "wasm-gerber-renderer";

const renderer = await createGerberRenderer(canvas);

await renderer.withFrame({ width: 1200, height: 800, padding: 24 }, async () => {
  await renderer.renderLayers([
    { source: topCopper, color: [1, 0, 0] },
    { source: bottomCopper, color: [0, 0.7, 1], alpha: 0.8 },
  ]);
});
```

Batch helper 預設會盡可能渲染所有 valid layer。某一層 parse 失敗時，其餘 layer 仍會繼續渲染。使用 `onLayerError` 檢查 skipped layer，或設定 `layerErrorMode: "throw"` 使用 strict behavior。

## Node.js 用法

使用 Node.js entrypoint 前請安裝 `node-gles-webgl2`。

```js
import { fileLayer, renderGerberToPngFile } from "wasm-gerber-renderer/node";

await renderGerberToPngFile(
  "board.png",
  [
    fileLayer("top.gbr", { name: "Top copper", color: "#ff3b30" }),
    fileLayer("bottom.gbr", { name: "Bottom copper", color: "#007aff" }),
  ],
  {
    width: 1200,
    height: 800,
    background: "#05070c",
    padding: 24,
  },
);
```

需要多次渲染同一組 Gerber 輸入時，請使用 prepared layer。

```js
const renderer = await createNodeGerberRenderer();

try {
  const prepared = await renderer.loadLayers([
    fileLayer("top.gbr", { color: "#ff4040" }),
    fileLayer("bottom.gbr", { color: "#40ff40" }),
  ]);

  await renderer.withFrame({ width: 1920, height: 1080, background: "#000" }, async () => {
    await renderer.renderLayers(prepared.layers);
  });
  const preview = await renderer.exportPng();
} finally {
  renderer.dispose();
}
```

## CLI

```bash
gerber-renderer board.gbr --output board.png --width 1600 --height 1000
gerber-renderer gerbers.tar.gz --output board.png --background "#05070c"
```

對常見 generic Gerber/drill 檔案，省略 `--output` 時會依輸入檔名產生 PNG 名稱。一次渲染多個檔案時建議指定 `--output`。

## 主要 API

Browser:

- `renderGerberToCanvas(canvas, layers, frameOptions)`
- `renderGerberToPng(canvas, layers, frameOptions, exportOptions)`
- `renderGerberToPngStream(canvas, writable, layers, frameOptions, exportOptions)`
- `createGerberRenderer(canvas, rendererOptions)`
- `renderer.withFrame(frameOptions, callback)`
- `renderer.renderLayer(layer, layerOptions)`
- `renderer.renderLayers(layers, options)`
- `renderer.exportPng(exportOptions)`
- `renderer.exportPngStream(writable, exportOptions)`
- `renderer.dispose()`

Node.js:

- `createNodeGerberRenderer(rendererOptions)`
- `renderGerberToPngBuffer(layers, frameOptions, exportOptions, rendererOptions)`
- `renderGerberToPngFile(outputPath, layers, frameOptions, exportOptions, rendererOptions)`
- `renderGerberToPngStream(writable, layers, frameOptions, exportOptions, rendererOptions)`
- `fileLayer(path, options)`
- `renderer.loadLayer(layer, layerOptions)`
- `renderer.loadLayers(layers, options)`
- `renderer.exportPngFile(outputPath, exportOptions)`

## 選項

`frameOptions`:

- `width`, `height`: output 尺寸
- `background`: output 背景，`null` 表示透明
- `fit`, `padding`, `view`: view fitting 與 manual camera
- `flipX`, `flipY`: 水平/垂直翻轉輸出
- `preserveArcRegions`, `arcTessellationQuality`: region arc 處理方式
- `minimumFeaturePixels`: line/arc 最小可見寬度
- `renderDrills`: 是否渲染 NC drill overlay
- `globalAlpha`: 未設定 layer alpha 時的預設透明度
- `layerErrorMode`, `onLayerError`: layer 失敗處理

`layerOptions`:

- `color`, `alpha`
- `offsetX`, `offsetY`
- `kind`: 強制指定 `"gerber"` 或 `"drill"`
- `name`: layer 顯示名稱

`exportOptions`:

- `background`
- `maxBandBytes`: streamed PNG export 的 row-buffer budget

`rendererOptions`:

- `wasmModule`, `wasmModuleUrl`, `wasmBinaryUrl`, `wasmInitInput`
- `contextAttributes`, `releaseContext`
- Node 專用 `glesModule`, `glesModuleName`, `gl`

## 授權

[MIT License](LICENSE)
