// Página do oráculo: monta o <Tldraw> (API pública) e expõe o `editor` do onMount em window.oracle.
// Nada aqui lê ou altera o funcionamento interno: só props documentadas (hideUi, onMount) e as
// funções exportadas createShapeId. O corredor (corre_guias.mjs) faz o resto por page.evaluate.
import React from "react";
import { createRoot } from "react-dom/client";
import { Tldraw, createShapeId } from "tldraw";

window.oracle = { editor: null, createShapeId };
createRoot(document.getElementById("host")).render(
  React.createElement(Tldraw, {
    hideUi: true,
    onMount: (editor) => {
      window.oracle.editor = editor;
      window.oracleReady = true;
    },
  }),
);
