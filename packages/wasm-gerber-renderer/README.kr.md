<div align="center">

# wasm-gerber-renderer

[**`English`**](README.md) · **`한국어`** · [**`简体中文`**](README.zh-Hans.md) · [**`繁體中文`**](README.zh-Hant.md)

</div>

---

`wasm-gerber-viewer`의 Rust/WASM parser와 renderer를 사용하는 WebGL2 Gerber renderer입니다.

이 패키지는 다음을 제공합니다.

- Gerber source string, `File`, `Blob`, `ArrayBuffer`, `Uint8Array` 입력을 브라우저 canvas에 렌더링
- headless WebGL2 context를 통한 Node.js PNG 렌더링과 파일/stream 출력
- Gerber 파일 또는 `.tar.gz`/`.tgz` archive를 PNG로 렌더링하는 `gerber-renderer` CLI
- packaging 중 생성되어 포함되는 `wasm-bindgen` output

브라우저 entrypoint는 호출자가 제공한 WebGL2 canvas를 사용합니다. Node.js entrypoint는 같은 WASM/WebGL renderer를 사용하며, 기본 native WebGL2 context provider로 [`node-gles-webgl2`](https://github.com/dsafdsaf132/node-gles-webgl2)를 사용합니다.

## 목차

- [설치](#설치)
- [플랫폼 지원](#플랫폼-지원)
- [브라우저 사용](#브라우저-사용)
- [Node.js 사용](#nodejs-사용)
- [CLI](#cli)
- [주요 API](#주요-api)
- [옵션](#옵션)
- [라이선스](#라이선스)

## 설치

브라우저 사용:

```bash
npm install wasm-gerber-renderer
```

CLI 사용자는 renderer 패키지와 [`node-gles-webgl2`](https://www.npmjs.com/package/node-gles-webgl2)가 필요합니다.

```bash
npm install -g wasm-gerber-renderer node-gles-webgl2
```

GitHub Packages에도 `@dsafdsaf132/wasm-gerber-renderer` 이름으로 배포됩니다.

```bash
npm config set @dsafdsaf132:registry https://npm.pkg.github.com
npm install @dsafdsaf132/wasm-gerber-renderer
```

GitHub Packages에서 설치할 때는 import specifier의 `wasm-gerber-renderer`를 `@dsafdsaf132/wasm-gerber-renderer`로 바꾸세요.

## 플랫폼 지원

브라우저 렌더링은 platform independent이며 호출자가 제공한 WebGL2 canvas를 사용합니다.

Node.js와 CLI 렌더링은 [`node-gles-webgl2`](https://github.com/dsafdsaf132/node-gles-webgl2)를 통해 다음 platform에서 지원됩니다.

- Linux x64
- Linux arm64
- macOS arm64
- Windows x64
- Windows arm64

macOS x64는 기본 `node-gles-webgl2` ANGLE prebuilt archive set에서 지원되지 않습니다.

## 브라우저 사용

```js
import { renderGerberToCanvas } from "wasm-gerber-renderer";

const canvas = document.querySelector("canvas");
const gerber = await file.text();

await renderGerberToCanvas(canvas, gerber, {
  background: "#05070c",
  padding: 24,
});
```

반복 렌더링에는 renderer instance를 재사용하세요.

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

Batch helper는 기본적으로 가능한 모든 valid layer를 렌더링합니다. 한 layer가 parse에 실패해도 나머지 layer는 계속 렌더링됩니다. `onLayerError`로 skipped layer를 확인하거나 `layerErrorMode: "throw"`로 strict behavior를 사용할 수 있습니다.

## Node.js 사용

Node.js entrypoint를 사용하기 전에 `node-gles-webgl2`를 설치하세요.

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

같은 Gerber 입력을 여러 번 렌더링할 때는 prepared layer를 사용하세요.

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

일반적인 generic Gerber/drill 파일은 `--output`을 생략하면 입력 파일 이름에서 PNG 이름을 만듭니다. 여러 파일을 한 번에 렌더링할 때는 `--output`을 지정하는 것이 좋습니다.

## 주요 API

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

## 옵션

`frameOptions`:

- `width`, `height`: output 크기
- `background`: output 배경. `null`이면 transparent
- `fit`, `padding`, `view`: view fitting과 manual camera
- `flipX`, `flipY`: 출력 좌우/상하 반전
- `preserveArcRegions`, `arcTessellationQuality`: region arc 처리 방식
- `minimumFeaturePixels`: line/arc 최소 표시 폭
- `renderDrills`: NC drill overlay 렌더링 여부
- `globalAlpha`: layer alpha가 없는 layer의 기본 투명도
- `layerErrorMode`, `onLayerError`: layer 실패 처리

`layerOptions`:

- `color`, `alpha`
- `offsetX`, `offsetY`
- `kind`: `"gerber"` 또는 `"drill"` 강제 지정
- `name`: layer 표시 이름

`exportOptions`:

- `background`
- `maxBandBytes`: streamed PNG export의 row-buffer budget

`rendererOptions`:

- `wasmModule`, `wasmModuleUrl`, `wasmBinaryUrl`, `wasmInitInput`
- `contextAttributes`, `releaseContext`
- Node 전용 `glesModule`, `glesModuleName`, `gl`

## 라이선스

[MIT License](LICENSE)
