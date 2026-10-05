# Oráculo Excalidraw — corrido por script, nunca lido

Corre o **Excalidraw 0.18.1** (fixado) num Chromium headless sobre entradas `.excalidraw` NOSSAS e grava
as saídas como fixtures. Só API pública: `exportToSvg`, `exportToBlob`, `restoreElements`,
`convertToExcalidrawElements` e, para as setas em cotovelo, o `<Excalidraw>` montado
(`excalidrawAPI.getSceneElements`).

## Licença (triada por artefacto instalado, 2026-10-05)

| pacote | versão | licença |
|---|---|---|
| `@excalidraw/excalidraw` | 0.18.1 | MIT |
| `roughjs` | 4.6.4 | MIT |
| `perfect-freehand` | 1.2.0 | MIT |
| `react` / `react-dom` | 19.1.0 | MIT |
| `playwright` | 1.63.0 | Apache-2.0 |
| `esbuild` | 0.28.2 | MIT |
| Chromium do Playwright | build 1243 | BSD-3 (Chromium) |

Fontes servidas pela página: as do `dist/prod/fonts` do pacote (Excalifont/Virgil OFL-1.1; as outras
**não triadas** — não embarcar nenhuma sem triar).

## Uso

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone      # ou o primário depois da integração
bash docs/MiroClone/ferramentas/excalidraw_oracle/instala.sh          # 1× por máquina: ~/Apps/excalidraw-oracle (290 MB) + ~/.cache/ms-playwright (658 MB)
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh \
  bash docs/MiroClone/ferramentas/excalidraw_oracle/corre.sh <pasta_saida> <entrada.excalidraw>...
```

Por entrada `<n>`: `<n>.svg` · `<n>.png` (escala 2; omitido acima de 300 KB) · `<n>.restored.json`
(elementos depois da normalização) · `<n>.meta.json` (versões, data, sha256 da entrada). Setas
`elbowed`: também `<n>.editor.{svg,png,json}` — o `restore` sozinho **não** roteia o cotovelo; o editor
montado sim (as formas das pontas são empurradas `→` `←` para forçar a rota).

**Determinismo:** `Math.random`/`crypto.getRandomValues` semeados pelo sha256 da entrada e relógio fixo.
Medido 2026-10-05: duas corridas das 8 entradas = **30/30 ficheiros idênticos byte a byte**, e os SVG
iguais às fixtures de `saidas/`. `--sem-semente` desliga a semente (diagnóstico).

## O que já mediu

- **A seta em cotovelo NÃO desvia de um obstáculo não ligado:** `seta_cotovelo_obstaculo` — origem
  `(40,160,140×90)`, obstáculo `(280,120,120×170)`, destino `(500,160,140×90)` ⇒ a rota é
  `[[0,0],[320,0]]`, uma recta **através** do obstáculo. Com as caixas desalinhadas
  (`seta_cotovelo_desalinhada`) dá um Z de 4 pontos `[[0,0],[160,0],[160,220],[320,220]]`, cujo segmento
  vertical em `x=340` cai DENTRO do obstáculo (`280..400`). ⇒ no desvio, o oráculo é **pior** que o nosso
  `ph2d-vec-connect` (A* sobre grafo de visibilidade); vale como oráculo da **forma** do cotovelo
  (lados de saída, `fixedPoint`), não do desvio.
- **Ligação de seta normalizada:** recta → `{elementId, focus, gap:1}`; cotovelo → `{elementId,
  fixedPoint:[1,0.5001], focus:0, gap:0}` (a ponta presa ao meio do lado, com o `0.5001` de desempate).
