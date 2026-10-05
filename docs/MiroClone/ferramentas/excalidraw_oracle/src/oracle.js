// Página do oráculo: expõe SÓ a API pública documentada do @excalidraw/excalidraw em window.oracle.
// As fontes vêm do servidor local: index.html define EXCALIDRAW_ASSET_PATH="/" (self-hosting
// documentado) ANTES deste módulo avaliar — imports ESM são içados, por isso não pode ser aqui.

import React from "react";
import { createRoot } from "react-dom/client";
import {
  Excalidraw,
  exportToSvg,
  exportToBlob,
  restoreElements,
  convertToExcalidrawElements,
} from "@excalidraw/excalidraw";

async function blobToBase64(blob) {
  const buf = new Uint8Array(await blob.arrayBuffer());
  let s = "";
  for (let i = 0; i < buf.length; i += 0x8000) {
    s += String.fromCharCode.apply(null, buf.subarray(i, i + 0x8000));
  }
  return btoa(s);
}

// entrada = conteúdo de um .excalidraw. Se `elements` traz esqueletos (sem `version`/`seed`),
// passam por convertToExcalidrawElements; depois tudo passa por restoreElements.
window.oracle = {
  async run(input) {
    const raw = input.elements || [];
    // Aquecimento das fontes ANTES de medir texto: o convertToExcalidrawElements mede o texto (quebra
    // de linha, tamanho do contentor) com a fonte que estiver carregada nesse instante. Medido: só o
    // exportToBlob (canvas) regista as FontFace no documento e espera por elas; o exportToSvg embute
    // as fontes sem as registar. Sem isto: 198 px (fonte de recurso) onde Excalifont dá 253 px.
    const texts = [];
    const families = new Set([5]);
    const walk = (el) => {
      if (typeof el.text === "string") texts.push(el.text);
      if (el.label && typeof el.label.text === "string") texts.push(el.label.text);
      if (el.fontFamily) families.add(el.fontFamily);
      if (el.label && el.label.fontFamily) families.add(el.label.fontFamily);
    };
    raw.forEach(walk);
    if (texts.length) {
      // Elementos COMPLETOS com tamanho dado: nada é medido antes de a fonte carregar.
      const warm = [...families].map((fontFamily, i) => ({
        type: "text", id: `aquecimento-${i}`, x: 0, y: i * 40, width: 100, height: 25,
        text: texts.join(" "), originalText: texts.join(" "), fontFamily, fontSize: 20,
        version: 1, versionNonce: 1, seed: 1,
      }));
      await exportToBlob({
        elements: restoreElements(warm, null, { refreshDimensions: false }),
        appState: {},
        files: null,
        mimeType: "image/png",
      });
      // E os subconjuntos unicode-range que cobrem os caracteres das entradas (acentos, CJK…).
      const all = texts.join(" ");
      const fams = [...new Set([...document.fonts].map((f) => f.family.replace(/^"|"$/g, "")))];
      await Promise.all(fams.map((fam) => document.fonts.load(`20px "${fam}"`, all)));
      await document.fonts.ready;
    }
    const isSkeleton = (el) => el.version === undefined || el.seed === undefined;
    const converted = raw.some(isSkeleton)
      ? convertToExcalidrawElements(raw, { regenerateIds: false })
      : raw;
    const elements = restoreElements(converted, null, { refreshDimensions: false, repairBindings: true });
    const appState = {
      exportBackground: true,
      viewBackgroundColor: "#ffffff",
      exportWithDarkMode: false,
      ...(input.appState || {}),
    };
    const files = input.files || null;
    await document.fonts.ready;
    const svgEl = await exportToSvg({ elements, appState, files, exportPadding: 10 });
    const svg = new XMLSerializer().serializeToString(svgEl);
    const blob = await exportToBlob({
      elements,
      appState: { ...appState, exportScale: 2 },
      files,
      mimeType: "image/png",
      exportPadding: 10,
    });
    const fontsLoaded = [...document.fonts].filter((f) => f.status === "loaded").map((f) => f.family);
    return {
      restored: elements,
      svg,
      pngBase64: await blobToBase64(blob),
      fontsLoaded: [...new Set(fontsLoaded)].sort(),
    };
  },
};
// Editor <Excalidraw> montado (API pública excalidrawAPI): para ver o que o EDITOR faz que as
// funções utilitárias não fazem. Medido: convertToExcalidrawElements e restoreElements NÃO
// encaminham setas em cotovelo (saem com os pontos da entrada); quem as encaminha é o editor ao
// mexer numa forma ligada. O corre.mjs monta, selecciona as formas das pontas e empurra-as com o
// teclado (→ depois ←, posição final igual), e lê a cena.
let editor = null;
window.oracle.editorMount = (elements) =>
  new Promise((resolve, reject) => {
    const host = document.createElement("div");
    host.style.cssText = "position:fixed;left:0;top:0;width:1280px;height:800px";
    document.body.appendChild(host);
    const root = createRoot(host);
    const timer = setTimeout(() => reject(new Error("editor não montou em 20 s")), 20000);
    root.render(
      React.createElement(Excalidraw, {
        initialData: { elements, appState: { viewBackgroundColor: "#ffffff" }, scrollToContent: false },
        handleKeyboardGlobally: true,
        excalidrawAPI: (api) => {
          editor = { api, root, host };
          setTimeout(() => { clearTimeout(timer); resolve(true); }, 500);
        },
      }),
    );
  });
// Ids das formas ligadas a setas elbowed (as que o corre.mjs empurra).
window.oracle.elbowEndpoints = () => {
  const ids = new Set();
  for (const e of editor.api.getSceneElements()) {
    if (e.type === "arrow" && e.elbowed) {
      if (e.startBinding) ids.add(e.startBinding.elementId);
      if (e.endBinding) ids.add(e.endBinding.elementId);
    }
  }
  return [...ids];
};
window.oracle.editorSelect = (ids) => {
  editor.api.updateScene({ appState: { selectedElementIds: Object.fromEntries(ids.map((id) => [id, true])) } });
};
window.oracle.editorRead = async () => {
  const elements = JSON.parse(JSON.stringify(editor.api.getSceneElements()));
  editor.root.unmount();
  editor.host.remove();
  editor = null;
  const appState = { exportBackground: true, viewBackgroundColor: "#ffffff", exportWithDarkMode: false };
  const svgEl = await exportToSvg({ elements, appState, files: null, exportPadding: 10 });
  const blob = await exportToBlob({ elements, appState: { ...appState, exportScale: 2 }, files: null, mimeType: "image/png", exportPadding: 10 });
  return { elements, svg: new XMLSerializer().serializeToString(svgEl), pngBase64: await blobToBase64(blob) };
};
// As funções públicas cruas, para sondas ad hoc (page.evaluate).
window.oracle.api = { exportToSvg, exportToBlob, restoreElements, convertToExcalidrawElements };
window.oracleReady = true;
