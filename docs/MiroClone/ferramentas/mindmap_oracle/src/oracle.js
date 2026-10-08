// Página do oráculo de mapa mental: expõe as TRÊS bibliotecas pelos pontos de entrada PÚBLICOS dos
// pacotes (o que o README de cada uma manda importar) em window.oracle. Nada aqui lê nem altera o
// funcionamento interno; o corredor (corre.mjs) monta, mede o DOM e carrega no teclado.
import MindElixir from "mind-elixir";
import MindMap from "simple-mind-map/dist/simpleMindMap.esm.js";
import { Markmap, defaultOptions as markmapDefaults } from "markmap-view";
import { Transformer } from "markmap-lib";

window.oracle = {
  MindElixir,
  MindMap,
  Markmap,
  markmapDefaults,
  Transformer,
};
window.oracleReady = true;
