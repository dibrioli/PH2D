// Página do oráculo de colaboração: <Tldraw store={useSync(...)}> ligado ao servidor local
// (servidor.mjs, ws://127.0.0.1:5202/connect/<sala>). Só API pública de USO: useSync({uri, assets}),
// <Tldraw store assetUrls onMount>, getAssetUrls (@tldraw/assets/selfHosted),
// editor.user.updateUserPreferences({name, color}). A interface fica VISÍVEL (os gestos do caso são de
// rato/teclado). Query: ?sala=<id>&nome=<A|B|C>&cor=<#hex>.
// Expõe window.oracle = { editor, status, connectionStatus } para o corredor ler o estado.
import React from "react";
import { createRoot } from "react-dom/client";
import { Tldraw, inlineBase64AssetStore, createShapeId, toRichText } from "tldraw";
import { useSync } from "@tldraw/sync";
import { getAssetUrls } from "@tldraw/assets/selfHosted";

// Ícones, fontes e traduções servidos pelo servidor local (documentado: «Self-hosting assets»), para
// que NADA saia do localhost. A marca d'água e a chave de licença não são tocadas.
const assetUrls = getAssetUrls({ baseUrl: "/" });

const q = new URLSearchParams(location.search);
const sala = q.get("sala") || "sala0";
const nome = q.get("nome") || "anon";
const cor = q.get("cor") || "#888888";
window.oracle = { editor: null, status: "init", connectionStatus: null, sala, nome, createShapeId, toRichText };

function App() {
  const store = useSync({ uri: `ws://127.0.0.1:5202/connect/${encodeURIComponent(sala)}`, assets: inlineBase64AssetStore });
  window.oracle.status = store.status;
  window.oracle.connectionStatus = store.connectionStatus ?? null;
  if (store.status === "error") window.oracle.error = String(store.error?.message || store.error);
  return React.createElement(Tldraw, {
    store,
    assetUrls,
    onMount: (editor) => {
      editor.user.updateUserPreferences({ name: nome, color: cor });
      window.oracle.editor = editor;
      window.oracleReady = true;
    },
  });
}
createRoot(document.getElementById("host")).render(React.createElement(App));
