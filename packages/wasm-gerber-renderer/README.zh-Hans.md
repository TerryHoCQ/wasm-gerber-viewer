<div align="center">

# wasm-gerber-renderer

[**`English`**](README.md) · [**`한국어`**](README.kr.md) · **`简体中文`** · [**`繁體中文`**](README.zh-Hant.md)

</div>

---

基于 `wasm-gerber-viewer` Rust/WASM parser 和 renderer 的 WebGL2 Gerber renderer。

这个包提供：

- 在浏览器 canvas 中渲染 Gerber source string、`File`、`Blob`、`ArrayBuffer` 或 `Uint8Array` 输入
- 通过 headless WebGL2 context 在 Node.js 中渲染 PNG，并支持直接写入文件或 stream
- 将 Gerber 文件或 `.tar.gz`/`.tgz` archive 渲染为 PNG 的 `gerber-renderer` CLI
- 打包时生成并包含的 `wasm-bindgen` output

浏览器 entrypoint 使用调用方提供的 WebGL2 canvas。Node.js entrypoint 使用同一套 WASM/WebGL renderer，并默认通过 [`node-gles-webgl2`](https://github.com/dsafdsaf132/node-gles-webgl2) 提供 native WebGL2 context。

## 内容

- [安装](#安装)
- [平台支持](#平台支持)
- [浏览器用法](#浏览器用法)
- [Node.js 用法](#nodejs-用法)
- [CLI](#cli)
- [主要 API](#主要-api)
- [选项](#选项)
- [许可证](#许可证)

## 安装

浏览器用户：

```bash
npm install wasm-gerber-renderer
```

CLI 用户需要 renderer 包和 [`node-gles-webgl2`](https://www.npmjs.com/package/node-gles-webgl2)：

```bash
npm install -g wasm-gerber-renderer node-gles-webgl2
```

同一个包也发布到 GitHub Packages，名称为 `@dsafdsaf132/wasm-gerber-renderer`：

```bash
npm config set @dsafdsaf132:registry https://npm.pkg.github.com
npm install @dsafdsaf132/wasm-gerber-renderer
```

从 GitHub Packages 安装时，请把 import specifier 中的 `wasm-gerber-renderer` 改成 `@dsafdsaf132/wasm-gerber-renderer`。

## 平台支持

浏览器渲染不依赖平台，使用调用方提供的 WebGL2 canvas。

Node.js 和 CLI 渲染通过 [`node-gles-webgl2`](https://github.com/dsafdsaf132/node-gles-webgl2) 支持：

- Linux x64
- Linux arm64
- macOS arm64
- Windows x64
- Windows arm64

默认的 `node-gles-webgl2` ANGLE prebuilt archive set 不支持 macOS x64。

## 浏览器用法

```js
import { renderGerberToCanvas } from "wasm-gerber-renderer";

const canvas = document.querySelector("canvas");
const gerber = await file.text();

await renderGerberToCanvas(canvas, gerber, {
  background: "#05070c",
  padding: 24,
});
```

重复渲染时复用 renderer instance：

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

Batch helper 默认尽可能渲染所有 valid layer。某一层 parse 失败时，其余 layer 仍会继续渲染。使用 `onLayerError` 检查 skipped layer，或设置 `layerErrorMode: "throw"` 使用 strict behavior。

## Node.js 用法

使用 Node.js entrypoint 前请安装 `node-gles-webgl2`。

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

需要多次渲染同一组 Gerber 输入时，请使用 prepared layer。

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

对于常见 generic Gerber/drill 文件，省略 `--output` 时会根据输入文件名生成 PNG 名称。一次渲染多个文件时建议指定 `--output`。

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

## 选项

`frameOptions`:

- `width`, `height`: output 尺寸
- `background`: output 背景，`null` 表示透明
- `fit`, `padding`, `view`: view fitting 和 manual camera
- `flipX`, `flipY`: 水平/垂直翻转输出
- `preserveArcRegions`, `arcTessellationQuality`: region arc 处理方式
- `minimumFeaturePixels`: line/arc 最小可见宽度
- `renderDrills`: 是否渲染 NC drill overlay
- `globalAlpha`: 未设置 layer alpha 时的默认透明度
- `layerErrorMode`, `onLayerError`: layer 失败处理

`layerOptions`:

- `color`, `alpha`
- `offsetX`, `offsetY`
- `kind`: 强制指定 `"gerber"` 或 `"drill"`
- `name`: layer 显示名称

`exportOptions`:

- `background`
- `maxBandBytes`: streamed PNG export 的 row-buffer budget

`rendererOptions`:

- `wasmModule`, `wasmModuleUrl`, `wasmBinaryUrl`, `wasmInitInput`
- `contextAttributes`, `releaseContext`
- Node 专用 `glesModule`, `glesModuleName`, `gl`

## 许可证

[MIT License](LICENSE)
