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

As dependências do `roughjs` que o crate `ph2d-board-rough` PORTA (W4, triadas pelo `LICENSE` de cada
artefacto em 2026-10-07; os textos estão em `crates/ph2d-board-rough/LICENSE-THIRD-PARTY.md`):
`hachure-fill` 0.5.2 · `points-on-curve` 0.2.0 (a de dentro do `roughjs`) · `points-on-path` 0.2.1 ·
`path-data-parser` 0.1.0 — todas MIT.

### As portas (W4): `corre_portas.mjs`, sem navegador

```bash
node docs/MiroClone/ferramentas/excalidraw_oracle/corre_portas.mjs \
  docs/MiroClone/ferramentas/excalidraw_oracle/entradas/portas.json docs/MiroClone/ferramentas/excalidraw_oracle/saidas
```

Corre o `roughjs` (o `generator` do bundle CJS) e o `perfect-freehand` directamente no Node sobre as
entradas de `portas.json` e grava `saidas/portas_rough.json` (os `sets` de cada caso, com a MESMA
semente que o lado Rust usa) e `saidas/portas_freehand.json` (passo 1 `getStrokePoints`, passo 2
`getStrokeOutlinePoints`). Gate: `ph2d-board-rough::oracle_tests` (29 + 11 casos, tolerância 1e-9).

⛔ **Corrigido 07/10 (triagem por artefacto):** a `Excalifont` instalada diz *«Copyright (c) 2024 by
Excalidraw. All rights reserved.»* e não traz licença (nem na tabela `name`, nem no pacote) — NÃO
embarca. A `Virgil` traz a SIL OFL 1.1 na tabela `name` (sem «Reserved Font Name» declarado): é a letra
à mão do rascunho (`crates/ph2d-text/fonts/Virgil-*`).

Fontes servidas pela página: as do `dist/prod/fonts` do pacote (Virgil OFL-1.1; a Excalifont sem licença no artefacto; as outras
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

- ⭐ **As opções do rough.js que o Excalidraw usa no rascunho** (W4, 07/10 — depois do smoke do dono:
  *«os traços não ficaram tão naturais e belos como do Excalidraw»*). Ajuste EXACTO do `d` (duas
  casas) dos SVG com sementes fixas (`ajuste/ajusta.mjs`, `ajusta2.mjs`, `cantos.mjs`, `setas.mjs`;
  entradas `rascunho_*`): **vértices presos** com tremor < 2 (`preserveVertices`); a elipse com
  `curveFitting 1`; o losango com os vértices a `⌊w/2⌋+1` (um desvio dele); **o tremor a METADE** quando
  o lado maior < 50 ou o lado menor < 20 (49×49 / 51×51, 19×300 / 20×300, 60×49 inteiro; abaixo de 10
  outra regra, não medida); a espessura não o muda; o rectângulo de cantos redondos é o caminho de
  quadráticas a partir de `(r, 0)`; as setas: arredondada = `curve` pelos pontos, de cantos vivos =
  `linearPath` com os vértices presos. **Depois lido no código dele** (`dist/dev`, por ordem do dono
  07/10): o `adjustRoughness` confirma a regra e acrescenta os ramos (redonda com o menor ≥ 15 e
  linhas com o maior ≥ 50 ficam inteiras; abaixo de 10 divide por 3); o tracejado desliga o traço
  duplo e soma 0,5 à espessura; `STROKE_WIDTH` = fina 1 · grossa 2 · muito grossa 4 (a de nascença é
  2 — no quadro passa a 1, ordem do dono); a ponta `arrow` tem 25 de comprimento a 20°, sem depender
  da espessura (`getArrowheadSize`). Entradas `rascunho_ramos`. Gate:
  `ph2d-board-render::sketch::oracle_tests` (≥ 50 formas e 4 setas, número a número).

- **A seta em cotovelo NÃO desvia de um obstáculo não ligado:** `seta_cotovelo_obstaculo` — origem
  `(40,160,140×90)`, obstáculo `(280,120,120×170)`, destino `(500,160,140×90)` ⇒ a rota é
  `[[0,0],[320,0]]`, uma recta **através** do obstáculo. Com as caixas desalinhadas
  (`seta_cotovelo_desalinhada`) dá um Z de 4 pontos `[[0,0],[160,0],[160,220],[320,220]]`, cujo segmento
  vertical em `x=340` cai DENTRO do obstáculo (`280..400`). ⇒ no desvio, o oráculo é **pior** que o nosso
  `ph2d-vec-connect` (A* sobre grafo de visibilidade); vale como oráculo da **forma** do cotovelo
  (lados de saída, `fixedPoint`), não do desvio.
- **O recuo do cotovelo antes de dobrar é FIXO: 40** (W2, 2026-10-06). `seta_cotovelo_volta`
  (origem 300,60 140×90 → destino 40,300 140×90, a seta sai pela direita e entra pela esquerda de
  um destino que está à esquerda) ⇒ o editor traça `(440,105) (480,105) (480,225) (0,225) (0,345)
  (40,345)`: avança **40** nas duas pontas e dá a volta a **meio** do vão vertical (225). Com as
  caixas 2× maiores (`seta_cotovelo_volta_grande`, 280×180) o recuo continua **40** e a volta a meio
  (380) ⇒ é uma constante, não uma fracção da caixa. Gate: `ph2d-board-route::oracle_tests` (o
  `JETTY` do quadro), mais a forma do Z a meio do vão de `seta_cotovelo_desalinhada`.
- **A ponta `arrow` com `strokeWidth 2`** (`seta_reta_ligada.svg`): as duas riscas recuam **23,49**
  ao longo da linha (meia-abertura ~20°) — o `HEAD_SCALE` do quadro (`23,49 / (4·2) = 2,94` na caixa
  do catálogo de pontas do vectorial, que mantém a abertura dele, 26,6°).
- **Ligação de seta normalizada:** recta → `{elementId, focus, gap:1}`; cotovelo → `{elementId,
  fixedPoint:[1,0.5001], focus:0, gap:0}` (a ponta presa ao meio do lado, com o `0.5001` de desempate).
- **Cantos e elipse das formas** (`roughness 0`, `strokeWidth 2`; tabela re-medível:
  `python3 mede_cantos.py`; gate `crates/ph2d-board-geom/src/oracle_tests.rs`):
  - **O que o editor escreve** ao DESENHAR (`node sonda_editor_arredondamento.mjs`, 2026-10-06):
    rectângulo `roundness {type:3}`, losango e elipse `{type:2}`. O `restoreElements` sozinho não
    arredonda nada (`formas_arredondamento_padrao`: esqueleto, campo ausente e o legado
    `strokeSharpness:"round"` saem todos `roundness: null`).
  - **Rectângulo** (`formas_canto_retangulo`, lado menor S = 20..400, deitados e em pé): `{type:3}`
    ⇒ raio **`min(S/4, 32)`** exacto (S=128 → 32; 136 → 32; 400 → 32); `{type:2}` ⇒ `S/4` sem tecto.
    O canto é uma **quadrática com o controlo no vértice** (no SVG, a cúbica elevada), não um arco.
  - **Losango** (`formas_canto_losango`, 80×60 · 160×120 · 400×300 · 200×200 · 400×80 · 81×61):
    vértices em `(⌊w/2⌋+1, 0)`, `(w, ⌊h/2⌋+1)`… (desvio de +1, assimétrico — **recusado**, o nosso é
    simétrico); `{type:2}` ⇒ corte de **¼ da meia-extensão em cada eixo** (= ¼ de cada aresta),
    canto = **cúbica com c1 = c2 = vértice**; `{type:3}` limita cada eixo a 32 em separado e as
    marcas saem da aresta (até 6,1 un. em 400×300).
  - **Elipse** (`formas_contorno_elipse`): elipse verdadeira — 73 cúbicas, desvio máx **0,075 un.**
    da analítica em 300×160 (0,004 no círculo 120×120). O traço é a linha central (`stroke-width 2`);
    cada `<g>` leva `translate(x − minX + 10, …)` (`exportPadding 10`).
