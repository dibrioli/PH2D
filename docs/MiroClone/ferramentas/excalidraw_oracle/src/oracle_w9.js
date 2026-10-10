// W9 (fidelidade): página do oráculo INDEPENDENTE DA VERSÃO — o mesmo fonte compila contra o 0.18.1
// e contra o canal `next` (o do excalidraw.com), que já não exporta `restore`. Só a API pública:
// `import * as EX` e o que existir. Monta o <Excalidraw> (1280×800, zoom 1, scroll 0 ⇒ ecrã = cena)
// para os passos que só o EDITOR faz (re-ligar setas ao mexer uma forma; desenhar uma seta à mão).
import React from "react";
import { createRoot } from "react-dom/client";
import * as EX from "@excalidraw/excalidraw";

async function blobToBase64(blob) {
  const buf = new Uint8Array(await blob.arrayBuffer());
  let s = "";
  for (let i = 0; i < buf.length; i += 0x8000) s += String.fromCharCode.apply(null, buf.subarray(i, i + 0x8000));
  return btoa(s);
}

let editor = null;
const mount = (elements, appState = {}) =>
  new Promise((resolve, reject) => {
    const host = document.createElement("div");
    host.style.cssText = "position:fixed;left:0;top:0;width:1280px;height:800px";
    document.body.appendChild(host);
    const root = createRoot(host);
    const timer = setTimeout(() => reject(new Error("editor não montou em 20 s")), 20000);
    let done = false;
    const onApi = (api) => {
      if (!api || done) return;
      done = true;
      editor = { api, root, host };
      setTimeout(() => {
        clearTimeout(timer);
        api.updateScene({ appState: { scrollX: 0, scrollY: 0, zoom: { value: 1 } } });
        resolve(true);
      }, 500);
    };
    root.render(
      React.createElement(EX.Excalidraw, {
        initialData: { elements, appState: { viewBackgroundColor: "#ffffff", ...appState }, scrollToContent: false },
        handleKeyboardGlobally: true,
        // 0.18.1: `excalidrawAPI`; o canal next renomeou-o `onExcalidrawAPI` (medido: o antigo não chama).
        excalidrawAPI: onApi,
        onExcalidrawAPI: onApi,
      }),
    );
  });

window.oracle = {
  api: EX,
  exports: Object.keys(EX).sort(),
  blobToBase64,
  restoreEls(raw, opts = { refreshDimensions: false, repairBindings: true }) {
    return EX.restoreElements(raw, null, opts);
  },
  async svg(elements, files = null) {
    const svgEl = await EX.exportToSvg({
      elements,
      appState: { exportBackground: true, viewBackgroundColor: "#ffffff", exportWithDarkMode: false },
      files,
      exportPadding: 10,
    });
    return new XMLSerializer().serializeToString(svgEl);
  },
  async png(elements, files = null) {
    const blob = await EX.exportToBlob({
      elements,
      appState: { exportBackground: true, viewBackgroundColor: "#ffffff", exportWithDarkMode: false },
      files,
      mimeType: "image/png",
      exportPadding: 10,
    });
    return blobToBase64(blob);
  },
  mount,
  editorApi: () => (editor ? editor.api : null),
  select(ids) {
    editor.api.updateScene({ appState: { selectedElementIds: Object.fromEntries(ids.map((id) => [id, true])), selectedGroupIds: {} } });
  },
  scene: () => JSON.parse(JSON.stringify(editor.api.getSceneElements())),
  appState: () => {
    const a = editor.api.getAppState();
    return { activeTool: a.activeTool?.type, scroll: [a.scrollX, a.scrollY], zoom: a.zoom?.value, offset: [a.offsetLeft, a.offsetTop], selected: Object.keys(a.selectedElementIds || {}) };
  },
  unmount() {
    if (editor) { editor.root.unmount(); editor.host.remove(); editor = null; }
  },
};
window.oracleReady = true;
