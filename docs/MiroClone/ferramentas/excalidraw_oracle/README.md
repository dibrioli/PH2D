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

Por entrada `<n>`: `<n>.svg` · `<n>.png` (pede `exportScale: 2`, mas o `exportToBlob` IGNORA o
`appState.exportScale` — medido na W8: escala 1/2/3 dão o MESMO PNG byte a byte; estes PNG são **1×**,
do tamanho do SVG; omitido acima de 300 KB) · `<n>.restored.json`
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

### W5 — organizar (molduras, grupos, alinhar, ordem, cadeado): `corre_organizar.mjs`, editor montado

```bash
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/excalidraw_oracle/corre_organizar.mjs \
  [--fotos <pasta>] [--so <caso>] docs/MiroClone/ferramentas/excalidraw_oracle/entradas/organizar_casos.json \
  docs/MiroClone/ferramentas/excalidraw_oracle/saidas
```

Monta o `<Excalidraw>` (1280×800, zoom 1, scroll 0) com as cenas de `organizar_casos.json`, age com o
rato/teclado do Playwright (selecção inicial pelo `updateScene`) e grava a cena ANTES e DEPOIS de cada
passo em `saidas/organizar_<caso>.json` (x, y, w, h, ângulo, `groupIds`, `frameId`, `index`, `locked`,
`isDeleted`, selecção, `editingGroupId`) + `.meta.json`. Precisa do `src/oracle.js` com
`editorApi`/`editorUnmount` (07/10: re-correr `instala.sh`). **Determinismo:** duas corridas dos 51
casos = **104/104 ficheiros idênticos** (o `.meta.json` sem o campo `date`).

- **Moldura, quem é filho** (o `frameId`): **desenhar a moldura** (`F`) por cima de formas só adopta as
  **totalmente dentro** (metade, centro dentro e canto ficam `null`; `A1_A3`). **Desenhar uma forma**:
  decide o **ponto onde o arrasto COMEÇA** — começa dentro ⇒ filho mesmo com o centro fora e 17×20 de
  sobreposição (`A1b`, `A1d`); começa fora ⇒ não, mesmo a cruzar (`A1c`, `A1e`); o filho novo entra na
  ordem **logo antes** da moldura (`Zz` < `a0`). **Arrastar** (entrar e sair): decide o **ponteiro ao
  largar**, não o centro nem a área — ponteiro dentro e centro fora ⇒ filho (`A4_ponteiro_dentro_centro_fora`);
  ponteiro fora e centro dentro ⇒ não (`A4_ponteiro_fora_centro_dentro`); 41,7 % da área no canto com o
  ponteiro dentro ⇒ filho (`A4_canto_centro_dentro`); sair a cruzar com o ponteiro dentro mantém (`A5_*`).
  Arrastar para dentro NÃO muda o `index` (o filho fica depois da moldura). Montar uma cena não adopta
  ninguém pela geometria.
- **API `convertToExcalidrawElements`**: moldura com `children` e sem caixa ⇒ caixa dos filhos **+10** de
  cada lado (`A0`: filhos 437..697×263..413 ⇒ 427,253,280×170); com caixa dada, os `children` ficam filhos
  mesmo FORA dela; sem `children` ⇒ **excepção** (`forEach` de `undefined`) — tem de ser `[]`.
- **Mover a moldura** leva os filhos pelo **mesmo delta** (seta 1, Shift+seta 5, arrastar o título +37,+19;
  `A6`); clicar no título OU no vazio de dentro selecciona a moldura. **Apagar** a moldura: os filhos
  **ficam**, com `frameId` a `null`, e ficam seleccionados (`A7`). **Encolher** pelo canto: quem fica
  totalmente fora perde o `frameId`; quem cruza a borda nova com o centro dentro mantém (`A8`).
- **Molduras não se aninham na UI**: desenhar uma dentro de outra ou à volta de outra ⇒ `frameId null`
  nas duas (`A9`, `A9b`). Pela API o `frameId` numa moldura sobrevive ao restore, mas mover a de fora
  leva só a de dentro, **não os netos** (`A9c`) — estado incoerente, não copiar.
- **Recorte e título** (`A10`): o filho que cruza a borda é **recortado** na moldura — no PNG exportado
  (vermelho em x 549..612 de cena, borda em 613; o resto dos 120 cortado), no canvas do editor (idem) e
  no SVG (`clipPath` = rectângulo da moldura com `rx 8`). A exportação enquadra a MOLDURA, não a união.
  O título no editor é um `div` do DOM: **14 px** Assistant, cor `#999`, `line-height` 17,5, em
  `x = frame.x`, fundo a **3 px** acima da borda (206,5..224 para y 227); no SVG `<text>` 14 px Helvetica e
  uma faixa de **20,5** acima da moldura.
- **Grupos** (`B_grupos`): Ctrl+G dá um id novo a cada membro; clicar num membro selecciona o **grupo
  inteiro** (`selectedGroupIds`); duplo clique entra (`editingGroupId`) e selecciona só o membro; Escape
  NÃO sai (só um clique no vazio). Grupo de grupo: `groupIds` = **[interior, exterior]** (o mais externo
  no FIM); clique ⇒ exterior; 1.º duplo clique ⇒ o grupo interior; 2.º ⇒ o membro. Ctrl+Shift+G tira só
  o **mais externo**.
- **Alinhar** (`C1_*`): referência = **caixa da selecção** (esquerda 400 = min x; direita 632; topo 250;
  fundo 561; centro h 516 e v 405,5 pelo botão). Grupo + solto (`C2_*`): o grupo move **como unidade**
  (desvios internos guardados; Ga 400→337, Gb 450→387). Rodado 0,5 rad (`C4_*`): usa a **AABB rodada**
  (o outro vai a x 497,76 e y 273,68, não a 500/300). **Distribuir** (`C3_*`, botão = Alt+H): **vãos
  iguais**, extremos fixos — 4 larguras 41/113/29/87 em 337..787 ⇒ vão **60** (D2 413→438, D3 601→611).
- **Ordem** (`D*`): Ctrl+]/Ctrl+[ trocam com o **vizinho na lista**, sobreponha ou não (`D1`, `D1b`,
  `D2`); um **grupo vizinho é saltado inteiro** (`D5a`, `D5a2`); dentro do grupo (duplo clique) a frente/
  trás fica **dentro do bloco do grupo**, que continua contíguo (`D5b`, `D5c`). Só o elemento movido
  recebe `index` novo (ex. `a1V`). Molduras: a ordem da entrada é guardada (filhos antes ou depois da
  moldura); Ctrl+Shift+[ num filho manda-o para o fundo absoluto, **atrás da moldura** (`D6`, `D6d`);
  Ctrl+[ num filho logo a seguir à moldura **não faz nada** (`D6e`); Ctrl+Shift+] num filho pára **logo
  depois da moldura** (não passa `O`, `D6c`); Ctrl+Shift+] na moldura leva **moldura + filhos** em bloco
  para o topo, filhos antes dela (`D6b`).
- **Cadeado** (`E_cadeado`, Ctrl+Shift+L): o trancado **não** se selecciona por clique nem por laço (o
  laço sobre os dois apanha só o outro), arrastar sobre ele **não o move**, e **Ctrl+A o exclui**.

### W5b — guias e encaixe: `corre_guias.mjs`

```bash
python3 docs/MiroClone/ferramentas/excalidraw_oracle/entradas/gera_guias_casos.py \
  docs/MiroClone/ferramentas/excalidraw_oracle/entradas/guias_casos.json          # só se mudar as cenas
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/excalidraw_oracle/corre_guias.mjs \
  [--so <caso>] [--plano] docs/MiroClone/ferramentas/excalidraw_oracle/entradas/guias_casos.json \
  docs/MiroClone/ferramentas/excalidraw_oracle/saidas
```

Cada **sonda** monta a cena de novo (1280×800, scroll 0, zoom do caso), liga o modo pelo `updateScene`
(`objectsSnapModeEnabled`, conferido no `getAppState` antes de arrastar), desce no centro do móvel (na
moldura: no título), anda 2 px, 12 passos até ao alvo, **repete o movimento final** e espera 2 quadros
(medido: sob carga o último `pointermove` perdia-se — o móvel ficava 40 px antes), lê o
`appState.snapLines` ANTES de largar e a cena DEPOIS. O alvo diz-se como «a feição `de` do móvel a
`mais` px de ECRÃ da feição `a` do outro» (`aabb.l|c|r|t|m|b`, `cantos.k` = k-ésimo canto rodado no
eixo, `ponto.k` = vértice). `--plano` lista, sem navegador, as feições a ≤ 20 px na posição planeada
(as cenas foram afinadas para não haver candidato concorrente mais perto que o pedido). Saída:
`saidas/guias_<caso>.json` (por sonda: plano, `snapLines` completos, móvel antes de largar, x/y/w/h/
ângulo/AABB/cantos de todos depois, desvio do encaixe = final − sem encaixe) + `.meta.json`.
**Determinismo:** duas corridas dos 15 casos (142 sondas, uma delas a `load` 155) = **30/30 ficheiros
idênticos** (o `.meta.json` sem o campo `date`).

- **Tolerância = 8 px de ECRÃ, constante no zoom** (`G8`): a esquerda de B a `d` px da de A encaixa
  para `|d| ≤ 8` e não para `|d| ≥ 9`, igual a zoom 1 e 2 (⇒ em cena, `8/zoom`). O vão igual encaixa com
  6 px (`G5` 46 → 40). Modo desligado ⇒ nada encaixa (controlos `*_sem_encaixe`). **Alt+S** alterna o
  modo (`G0`: falso → verdadeiro → falso).
- **Rodados encaixam pelos 4 CANTOS RODADOS + centro, não pela AABB** (`G1`, B 160×100 a 15/30/37/45°,
  A 0°): um canto **interior** de B posto a 3 px da esquerda de A (x 300) vai a x **300 exacto** com a
  AABB de B longe (AABB esq. 274,1 · 250 · 239,8 · 229,3 nos quatro ângulos); igual a −3 px; os
  centros encaixam; em y o mesmo. Nos `snapLines` o ponto do móvel é o canto rodado verdadeiro (ex.
  `[300, 587,59]`), não um canto da AABB. **Alvo rodado** (`G2`, A a 30°): os 4 cantos de A são alvos
  — a aresta de B encaixa em x 333,40 / 393,40 / 506,60 / 566,60 (os quatro x dos cantos de A, dois deles interiores) e o centro em 450.
- **Só guias de EIXO** (`G3`, A e B a 30°): B com a aresta PARALELA à de A e desviada 3 px ao longo
  dela ⇒ **nada** encaixa, `snapLines` vazio. Em todas as 142 sondas os tipos são só `points` (todos os
  pontos com o mesmo x ou o mesmo y: 72 em x, 24 em y) e `gap` (`direction: "horizontal"`, 10) — zero
  linhas oblíquas.
- **Rotação não encaixa em ângulo nenhum** (`G4`, B 0° ao lado de A a 30°, modo ligado): o ângulo final
  é o do ponteiro (28,02° · 1,64° · 358,36° · 31,66° · 44,42°), nem 30 nem 0; nenhum `snapLines`
  durante a rotação. **Shift = múltiplo de 15° mais PRÓXIMO** (arredonda, não trunca): 8,25 → 15 ·
  21,80 → 15 · 28,02 → 30 · 36,87 → 30 · 50,19 → 45 · 99,87 → 105. A pega fica **16..24 px** de ecrã
  acima da aresta de cima (cursor `grab`).
- **Vãos** (`G5`): A|40|B e C posto a 43 / 37 / 41 / 46 de B ⇒ C vai a **40** nos quatro; os
  `snapLines` trazem os dois vãos como `gap` (`[[300,340],[340,340]]` e `[[440,340],[480,340]]`, à
  altura do meio da faixa comum). **Meio** (`G5b`): B a 53/47 entre A e C ⇒ 50/50, também `gap`. Sem
  sobreposição em y (`G5c`, C noutra linha) ⇒ **não** há encaixe de vão (fica 43).
- **Molduras** (`G6`): a caixa da moldura é alvo (um livre encaixa na esquerda 400 e na direita 800,
  pontos `[400,200]`/`[400,500]`); a moldura arrastada pelo título encaixa pela **sua caixa** (esq. →
  T, baixo → topo de R; os filhos não aparecem nos `snapLines`); um **filho** encaixa no irmão, nas
  bordas da moldura **e** num elemento de FORA dela (U acima). **A moldura não roda:** nenhum cursor
  `grab` de 0 a 60 px acima dela, e o arrasto ali deixa ângulo 0 e a posição; um filho roda (29,98°).
- **Lineares e traço livre encaixam pela AABB** (`G7`): linha — esquerda, direita e centro da caixa
  encaixam; o vértice do meio **interior** em x (220) **não**; numa linha cujo vértice do meio é o
  extremo (L2) ele encaixa e a ponta final interior **não**. Seta: as duas pontas (= a caixa). Traço
  livre: esquerda/direita da caixa sim, vértice interior não. Os pontos nos `snapLines` são os cantos
  da caixa (`[100,260]` não é vértice da linha).

### W8 — o esquema `.excalidraw` e a ida e volta: `corre_esquema.mjs`

```bash
python3 docs/MiroClone/ferramentas/excalidraw_oracle/entradas/gera_w8_sondas.py \
  docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w8_sondas.json            # só se mudar as sondas
bash docs/MiroClone/ferramentas/excalidraw_oracle/instala.sh   # 1× depois de 09/10: o src/oracle.js expõe restore/serializeAsJSON/loadFromBlob/exportToCanvas
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/excalidraw_oracle/corre_esquema.mjs \
  docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w8_tipos.excalidraw \
  docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w8_sondas.json docs/MiroClone/ferramentas/excalidraw_oracle/saidas
```

(`ORACLE_WWW=<pasta>` serve outra página construída; a porta é FIXA, `ORACLE_PORT` = 47118, porque o
`serializeAsJSON` grava `source` = a origem da página.) A entrada `w8_tipos` traz um elemento MÍNIMO
de cada tipo (só as chaves que importam) — rectângulo/elipse/losango, texto solto, texto num contentor,
seta recta ligada, seta `elbowed` com `fixedPoint`, linha tracejada, traço livre, moldura com 3 filhos
(grupo aninhado `["G1","G0"]` e uma imagem PNG 8×8 em `files`), `magicframe`, `embeddable`, `iframe`.
Saídas `saidas/w8_*`: `.restored.json`, `.serialized.excalidraw` (o ficheiro que o Excalidraw grava),
`.serialized_database.json`, `.svg`/`.png`, `.embed.svg`/`.embed.png`, `.escuro.png`, `.sem_fundo.png`,
`.export.json` (cada variante de export: dimensões, sha, chunks PNG, `<metadata>` SVG, e a volta pelo
`loadFromBlob`), `w8_sondas.json` (117 sondas de elementos + 6 de `appState`, entrada → saída),
`w8_appstate_restaurado.json`. **Determinismo:** duas corridas = **12/12 ficheiros idênticos** (o
`.meta.json` sem `date`).

- **Ficheiro:** `{type:"excalidraw", version:2, source, elements, appState, files}`. O `appState`
  gravado é SÓ `{gridSize, gridStep, gridModeEnabled, viewBackgroundColor}` (local = database);
  `files` = `{<fileId>: {mimeType, id, dataURL, created}}`. Ida e volta: `loadFromBlob(json)` →
  `serializeAsJSON` dá o MESMO texto; os elementos voltam iguais ao `restore`.
- **O que o `restore` PREENCHE** (elemento sem a chave): `strokeColor #1e1e1e` · `backgroundColor
  transparent` · `fillStyle solid` · `strokeWidth 2` (também se vier `0` — falsy) · `strokeStyle solid` ·
  `roughness 1` · `opacity 100` · `angle 0` · `roundness null` · `groupIds []` · `frameId null` ·
  `boundElements []` · `link null` · `locked false` · `isDeleted false` · `seed 1` (FIXO, não sorteia) ·
  `version` = dada+1 (ausente ⇒ 2) · `versionNonce` sorteado · `updated` = agora · `id` sorteado
  (21 car., nanoid) · `index` fraccionário. Texto: `textAlign left` · `verticalAlign top` ·
  `originalText` = `text` · `autoResize true` · `lineHeight` = **altura / (fontSize × linhas)** (1,85
  para 37/20; sem altura ⇒ 1,25) — e **NÃO preenche** `fontSize` nem `fontFamily`. Linear:
  `startBinding/endBinding/lastCommittedPoint null`, seta `endArrowhead "arrow"`/linha `null`.
  Imagem: `status pending` · `scale [1,1]` · `crop null` (sem `fileId` ⇒ chave ausente). Moldura: `name null`.
- **O que ele CORRIGE:** caixa negativa ⇒ normalizada (x −40, w 50); `x null` ⇒ 0; pontos com o 1.º ≠
  (0,0) ⇒ trasladados para (0,0) e o `x,y` ajustado; `width/height` da linear ⇒ a caixa dos pontos;
  linear de 1 ponto ⇒ **descartada**; `selection` e tipo desconhecido ⇒ **descartados**; `link
  javascript:` ⇒ `about:blank`; `containerId`/`boundElements`/`frameId`/`startBinding` para id
  inexistente ⇒ `null`/removido; texto com `containerId` válido ⇒ o contentor ganha o `boundElements`
  (a seta NÃO — fica ligada sem recíproca); id duplicado ⇒ o 2.º ganha id novo; seta `elbowed` com
  ligação SEM `fixedPoint` ⇒ ligações `null`; `elbowed` diagonal ⇒ cotovelo `[[0,0],[50,0],[50,50],[100,50]]`;
  `fixedPoint 0.5` ⇒ `0.5001`; legado `strokeSharpness "round"` ⇒ rectângulo `{type:1}`, losango `{type:2}`.
- **O que ele NÃO valida** (passa tal e qual): `opacity` 150/−5/"50", `roughness` 7, `strokeStyle`/
  `fillStyle`/`textAlign`/`verticalAlign`/`startArrowhead` desconhecidos, `roundness {type:9}`, `angle`
  10, cor inválida, `x:"10"`, `groupIds:"G"`, `locked:"sim"`, `fontFamily` 4/10/99/`"Virgil"`. ⇒ o
  NOSSO import valida (o Excalidraw confia no ficheiro).
- **O que REBENTA o import inteiro** (excepção, nenhum elemento entra): texto sem `text` ou com `text`
  número; `line`/`arrow`/`freedraw` sem `points`; `index` inválido (`"zz!"` ⇒ *invalid order key*).
- **`index`** (fractional-indexing, base 62): ausentes ⇒ `a0..a9 aA..aZ aa..` pela ordem do array;
  válidos e crescentes ⇒ mantidos (`a0`, `a0V`, `b0`); fora de ordem ou duplicados ⇒ só os que violam
  são reescritos (`a5, null, a1` ⇒ `a5, a6, a7`). **A ordem é a do ARRAY**, o `index` segue-a.
- **Constantes** (do bundle): `FONT_FAMILY` Virgil 1 · Helvetica 2 · Cascadia 3 · Excalifont 5 ·
  Nunito 6 · Lilita One 7 · Comic Shanns 8 · Liberation Sans 9 (o 4 não existe); `ROUNDNESS` LEGACY 1 ·
  PROPORTIONAL_RADIUS 2 · ADAPTIVE_RADIUS 3; MIME `application/vnd.excalidraw+json`.
- **`convertToExcalidrawElements`** (esqueleto) preenche MAIS que o `restore`: caixa 100×100, texto
  `fontSize 20`/`fontFamily 5`/medido, seta `[[0.5,0],[99.5,0]]` com `elbowed:false`, imagem
  `strokeColor transparent`; NÃO converte `freedraw` nem `embeddable` (passam crus, `version` ausente).
- **Export (API):** `exportToSvg({exportPadding, exportingFrame, skipInliningFonts, renderEmbeddables})`
  + `appState.{exportBackground, viewBackgroundColor, exportWithDarkMode, exportEmbedScene}`;
  `exportToBlob/exportToCanvas({exportPadding, maxWidthOrHeight, getDimensions, mimeType, quality})`.
  ⛔ `appState.exportScale` é **ignorado** pela API (só a UI o usa); a escala é `getDimensions:
  (w,h)=>({width:2w,height:2h,scale:2})` (1880×1200 medido). Padding 0/10/50 ⇒ 920/940/1020 de largura;
  `exportingFrame` ⇒ a caixa da moldura sem padding (400×200); escuro = `filter="invert(93%)
  hue-rotate(180deg)"` na raiz do SVG (no PNG, o canvas filtrado); sem fundo ⇒ sem o `<rect>` de fundo
  (PNG RGBA). **PDF: nenhum** — zero ocorrências de `pdf` nos `.d.ts`.
- **Embed:** PNG ⇒ chunk **`tEXt`** de chave `application/vnd.excalidraw+json`, valor
  `{"version":"1","encoding":"bstring","compressed":true,"encoded":<zlib deflate como bytes latin1>}`,
  inflado = o JSON do ficheiro (aqui com `appState` só `{viewBackgroundColor}`). SVG ⇒
  `<!-- svg-source:excalidraw --><metadata><!-- payload-type:application/vnd.excalidraw+json -->
  <!-- payload-version:2 --><!-- payload-start -->BASE64(o mesmo invólucro)<!-- payload-end --></metadata>`.
  `loadFromBlob` de um e de outro devolve os 17 elementos iguais ao `restore`; de um SVG sem embed ⇒
  *EncodingError: Image doesn't contain scene*.
- **O SVG exportado:** imagens como `<symbol id="image-<fileId>">` + `<use href>`; fontes como
  `@font-face` woff2 em data-URL; moldura = `<clipPath id=<frameId>>` com `rx 8` e título `<text>`
  Helvetica 14 px `#999999` (a `magicframe` sem nome escreve «AI Frame»); linha tracejada ⇒
  `stroke-width` +0,5 e `stroke-dasharray "8 10"`. ⛔ O recorte vai como **`ns1:clip-path`** (atributo no
  namespace SVG, artefacto do `XMLSerializer`): o **resvg** recorta, o **librsvg** e o **Inkscape NÃO**.

#### Oráculos SVG/PDF do SISTEMA (triagem por artefacto, `LANG=C pacman -Qi`, 09/10) — `fumo_svg_pdf.sh`

```bash
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh bash docs/MiroClone/ferramentas/excalidraw_oracle/fumo_svg_pdf.sh <pasta>
```

| ferramenta | pacote · versão | licença do artefacto | papel | fumo |
|---|---|---|---|---|
| `resvg`/`usvg` | resvg 0.48.1 | Apache-2.0 / MIT | rasterizar o NOSSO SVG (a mesma `usvg` 0.48.1 do lock) | ok; ignora `@font-face` |
| `rsvg-convert` | librsvg 2.63.2 | LGPL-2.1+ (só correr) | 2.º rasterizador SVG | ok; ignora `ns1:clip-path` |
| `inkscape` | 1.4.4 | GPL/LGPL (só correr) | 3.º rasterizador; SVG→PDF (cairo 1.18.6) | ok |
| `magick` | imagemagick 7.1.2-32 | ImageMagick (permissiva) | SVG via librsvg; PDF via gs | ok |
| `pdftoppm`/`pdftocairo`/`pdfinfo` | poppler 26.08.0 | GPL-2/3 (só correr) | rasterizar o NOSSO PDF; PDF→SVG; metadados | ok; os dois iguais (max 0) |
| `qpdf --check`/`--json` | qpdf 12.4.2 | Apache-2.0 / Artistic-2.0 | validar a ESTRUTURA do PDF (xref, streams) | ok |
| `gs` | ghostscript 10.08.0 | **AGPL-3.0+** (só correr) | 2.º rasterizador PDF | ok |
| `xmllint` | libxml2 2.15.4 | MIT | SVG bem-formado | ok |
| `mutool`, `cairosvg`, `veraPDF`, `fitz`, `pypdf` | — | — | ausentes | — |
| Python | pillow 12.3 (MIT-CMU), numpy 2.5.3, lxml 6.1.3 | — | comparar PNG | ok |

Concordância sobre o SVG mínimo: librsvg × Inkscape × magick ≤ 8/255; × resvg máx 29 (1,48 % dos px,
antialias). PDF mínimo: pdftoppm = pdftocairo; gs e magick diferem no texto (Type1 Helvetica). O SVG do
Excalidraw rasterizado × o PNG do canvas: resvg 1,9 % · Inkscape 2,7 % · librsvg 3,7 % dos px > 8 (sobretudo o
texto: o resvg avisa que salta o `@font-face`, as fontes caem no recurso).

### W9 — a fidelidade do `.excalidraw` nas DUAS versões: `corre_fidelidade.mjs` (estudo [07](../../07_fidelidade_excalidraw.md))

O excalidraw.com já grava um formato MAIS NOVO que o 0.18.1 (ligações `{elementId, mode:"orbit",
fixedPoint}` sem `focus`/`gap`, o tipo `stickynote`, …). O npm `latest` continua a ser o 0.18.1 (20/04/2026);
o formato novo só existe no canal `next` (builds `0.18.0-<commit>` do `master`). Triagem por artefacto
instalado (`package.json` de cada pacote em `node_modules`; o `@excalidraw/*` não traz ficheiro LICENSE,
só o campo `license`), 10/10:

| pacote (canal next) | versão | licença |
|---|---|---|
| `@excalidraw/excalidraw` · `element` · `common` · `math` | 0.18.0-143b5b6 (`next` de 08/10/2026) | MIT |
| `@excalidraw/fractional-indexing` | 0.18.0-143b5b6 | CC0-1.0 |
| `@excalidraw/laser-pointer` | 1.3.1 | MIT (LICENSE) |
| `roughjs` 4.6.4 · `perfect-freehand` 1.2.0 · `points-on-curve` 1.0.1 | — | MIT (LICENSE) |
| `react`/`react-dom` 19.1.0 · `esbuild` 0.28.2 · `playwright` 1.63.0 | — | MIT · MIT · Apache-2.0 |

**Bissecção pelo npm** (`npm pack` de cada build, `grep` nos `.d.ts`): o `BindMode = "inside" | "orbit" |
"skip"` entra entre `0.18.0-7f66e1f` (13/11/2025, não tem) e `0.18.0-c141960` (25/11/2025, tem); o tipo
`stickynote` entre `0.18.0-2b1c2a9` (07/09/2026) e `0.18.0-afa3a65` (10/09/2026).

```bash
bash docs/MiroClone/ferramentas/excalidraw_oracle/instala.sh        # se ainda não (a 0.18.1 + Chromium)
bash docs/MiroClone/ferramentas/excalidraw_oracle/instala_w9.sh     # ~/Apps/excalidraw-oracle-next (next/package.json + lock) e www_w9/ nas duas
python3 docs/MiroClone/ferramentas/excalidraw_oracle/entradas/gera_w9_casos.py \
  docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w9_casos.json     # só se mudar os casos
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/excalidraw_oracle/corre_fidelidade.mjs \
  [--so <caso>] [--versao 0181|next] docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w9_casos.json \
  docs/MiroClone/ferramentas/excalidraw_oracle/saidas
python3 docs/MiroClone/ferramentas/excalidraw_oracle/mede_w9.py     # as tabelas do estudo 07, das saídas
```

A página `src/oracle_w9.js` é INDEPENDENTE da versão (`import * as EX`: o `next` já não exporta `restore`,
e renomeou a prop `excalidrawAPI` → `onExcalidrawAPI` — o `src/oracle.js` antigo não monta no next).
70 casos (`entradas/w9_casos.json`, gerados): `B_*` ligações (orbit/inside/skip/legado, fixedPoint no centro,
dentro, na borda, fora; espessuras; cantos redondos; as duas gramáticas juntas), `P_*` pontas (comprimento ×
espessura × tipo, o último trecho), `F_fuste`, `M_*`/`M2_*` setas desenhadas pelo RATO no editor (profundidade,
tempo parado, modificadores, cliques, arrastar a ponta), `T_formato` (um elemento de cada tipo/propriedade),
`dono_bv` (o ficheiro do dono, `entradas/w9_dono_bv.excalidraw`, smoke 10/10). Saídas: `saidas/w9_<caso>.<0181|next>.json`
(cabeçalho com versão, licença, sha das entradas, semente; os elementos depois de CADA passo) e `.svg`.

⚠️ **Relógio:** os casos de desenho à mão correm com o relógio LIVRE (`"relogio": "livre"`): o editor next
decide a ligação por um temporizador (`bindModeHandler`) e o relógio fixo congelava-o. Nesses casos os ids,
a `seed` e o `created` dos elementos novos são normalizados (`novo1…`, 7919·(i+1), sem `created`).
**Determinismo:** duas corridas dos 70 casos × 2 versões = **220/220 ficheiros idênticos** (10/10, `load` 79).

**O NOSSO ficheiro (W9, a cura):** `entradas/w9_nosso_casos.json` — à PARTE do `w9_casos.json` (as 220 saídas
levam o sha dele no cabeçalho) — é o `sample_w9` de `ph2d-board-excalidraw/src/fidelity_tests.rs` escrito pelo
nosso escritor (ponto fixo ⇒ `inside`, órbita e centro ⇒ `orbit`, as duas gramáticas, a curva dele, as pontas
novas, o aspecto). Grava-o o teste ignorado `grava_a_entrada_w9_nosso`; corre-se com o MESMO `corre_fidelidade.mjs`
passando este ficheiro em vez do `w9_casos.json`. Saídas `saidas/w9_X_nosso.{0181,next}.{json,svg}`; o gate
`xv3_both_versions_keep_the_tips_we_wrote` (as duas versões não mexem ao abrir; o next re-liga as 5 pontas presas
a ≤ 0,05 px de onde as escrevemos; o 0.18.1 guarda `mode`/`fixedPoint`). ⚠️ 10/10: a página `www/` instalada
era de 07/10 (sem o `serializeAsJSON` da W8 — o `corre_esquema.mjs` esperava 60 s e caía); reconstrói-se com a
secção «Página» do `instala.sh` (os `package*.json` iguais aos instalados ⇒ sem `npm ci`). Ao regravar
`w8_ph2d_quadro.*` o `corre_esquema` também reescreve `w8_sondas.json`: só ids/sementes sorteados pela entrada
mudam — esse fica o da W8.

- **A ponta presa em `orbit`** (XB1): fica na recta entre os dois FOCOS (o `fixedPoint` de cada ponta no
  mundo, ou a ponta solta), no 1.º ponto a **5 + strokeWidth(da FORMA)/2** do contorno — 5,5 · 6 · 7 para
  1 · 2 · 4 (a espessura da SETA não entra: seta 4 em forma 1 ⇒ 5,5); elipse = semi-eixos + vão; segue os
  cantos redondos; foco FORA da faixa ⇒ a ponta é o próprio foco (`B_orbit_fora`: 50 px fora). Contra os
  pontos GRAVADOS no ficheiro do dono: erro ≤ **0,04** px nas seis pontas. `inside` ⇒ a ponta É o foco.
- **A ponta de seta** (XA2): comprimento = **min(base, último trecho/2)**, sem depender da espessura;
  `arrow` 25 a 20°, `triangle` 15 a 25°, `circle` Ø15, `bar` 15 de cada lado, `diamond` 24. Igual nas duas versões.
- **O fuste** (XA1): «redonda» de 2 pontos = RECTA (controlos a ⅓ e ⅔ da corda); de 3+ = Catmull-Rom
  uniforme (pega = (P₊₁ − P₋₁)/6; nas pontas (P₁ − P₀)/6). Igual nas duas versões.
- **0.18.1 × next lendo o MESMO ficheiro:** o 0.18.1 IGNORA `mode`/`fixedPoint` numa seta não-cotovelo
  (acrescenta `focus:0`, sem `gap` ⇒ re-liga ao centro a encostar); o next converte `{focus,gap}` em
  `orbit` com o `fixedPoint` tirado da ponta gravada e IGNORA o `gap` (usa a lei XB1). Com as DUAS
  gramáticas na mesma ligação cada versão usa a sua (`B_ambos_orbit_e_legado`). O 0.18.1 DESCARTA o
  `stickynote`; o next renomeia `dot` → `circle` e `crowfoot_*` → `cardinality_*`.

### W10 — a lei do legado no 0.18.1 e o realce do next: `corre_fidelidade.mjs --prefixo w10`

```bash
python3 docs/MiroClone/ferramentas/excalidraw_oracle/entradas/gera_w10_casos.py docs/MiroClone/ferramentas/excalidraw_oracle/entradas
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/excalidraw_oracle/corre_fidelidade.mjs --prefixo w10 --versao 0181 docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w10_focus_casos.json docs/MiroClone/ferramentas/excalidraw_oracle/saidas
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/excalidraw_oracle/corre_fidelidade.mjs --prefixo w10 docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w10_realce_casos.json docs/MiroClone/ferramentas/excalidraw_oracle/saidas
cargo test -p ph2d-board-excalidraw grava_a_entrada_w10_nosso -- --ignored   # o NOSSO ficheiro (entradas/w10_nosso_casos.json)
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/excalidraw_oracle/corre_fidelidade.mjs --prefixo w10 docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w10_nosso_casos.json docs/MiroClone/ferramentas/excalidraw_oracle/saidas
python3 docs/MiroClone/ferramentas/excalidraw_oracle/mede_w10.py
```

**Novidades do corredor:** `--prefixo <p>` (o prefixo dos ficheiros de saída); o passo
`captura` no `desenhar` grava um PNG `<prefixo>_<caso>.<versao>.<nome>.png` e o appState de ligação/realce
ANTES de largar (sem os voláteis, também dentro do appState); `"guardar": false` num passo (não entra na
saída); `"compacto": true` num caso (JSON sem indentação).

**Casos:** 34 `F_*` (só o passo `empurrar` guardado, sem indentação: 3,1 MB) — forma × vão 0/0,01/1/6/15/−10/−40/−47
× rotação × cantos × espessura × proporção × foco além de ±1 × rectas paralelas aos eixos (12 ângulos × 7 focos,
presas pelo fim e pelo início), só no 0.18.1 — e 9 `R_*` (a ponta arrastada ou desenhada para a faixa, dentro,
o miolo, a diagonal) nas duas versões.

**Determinismo:** duas corridas iguais, 34/34 e 18/18 ficheiros; mais 2 corridas X10 idênticas (10/10).

**O que mediu** (estudo [07](../../07_fidelidade_excalidraw.md) §2 XV7, §3): ao re-ligar, o 0.18.1 põe a ponta na
recta do ponto vizinho por \|focus\|·K (focus = d/h(n)) a `gap` da forma — \|Δfocus\| 0,00000, gap rect/elipse
0,0000 —; `gap 0` põe a ponta DENTRO; rectas paralelas aos eixos saem espelhadas; o vão do losango e do rectângulo
redondo não tem lei (resíduo até 3,7 px) — ⇒ tem: a lei das peças, §W23 B9. No next, o arrasto de uma ponta NÃO marca o foco: só o contorno azul e
a seta já parada a g (os pontos mostrados são o encaixe no meio do lado), e o foco do gesto na faixa é o cursor.

**Quando o escritor muda, regravam-se as TRÊS entradas nossas:** W8 (`grava_a_entrada_do_oraculo` +
`corre_esquema.mjs entradas/w8_ph2d_quadro.excalidraw`, devolvendo o `w8_sondas.json` ao da W8); W9
(`grava_a_entrada_w9_nosso` + `corre_fidelidade.mjs entradas/w9_nosso_casos.json`); W10 (os comandos acima).

### W14 — a IMAGEM: `corre.sh saidas/w14 entradas/w14_*.excalidraw`

Entradas: `entradas/w14_imagens.excalidraw` (gerada por `entradas/gera_w14_imagens.py`: uma PNG 4×2 de
oito cores, uma por píxel, e seis elementos `image` — inteira, `scale [-1,1]`, `crop` 2×2 de 4×2,
`roundness {type:3}` + `opacity 50` + `angle 0.5`, `fileId` sem ficheiro com `status:"pending"`, e
`scale [1,-1]` + `crop` 2×1) e `entradas/w14_nosso.excalidraw` (escrita pelo NOSSO escritor, o teste
ignorado `grava_a_entrada_do_oraculo_w14`). ⚠️ Antes de correr: `www/` do `$ORACLE_HOME` tem de existir
(o passo do `esbuild` do `instala.sh`; em 11/10 estava vazio e o `corre.sh` recusava).

Medido (SVG + PNG + `restored.json`):

| lei | o que o oráculo faz |
|---|---|
| esticar | `<symbol><image preserveAspectRatio="none" width="100%" height="100%">` + `<use width height>` da caixa: a imagem estica à caixa, sem manter proporção |
| espelho | `translate(w/2 h/2) scale(-1 1) translate(-w/2 -h/2)` no `<use>`: vira à volta do CENTRO da caixa |
| recorte | `crop {x,y,width,height,naturalWidth,naturalHeight}` em píxeis NATURAIS; a imagem inteira escala por `w/crop.width` e uma `<mask>` do tamanho da caixa mostra só o recorte |
| cantos | `roundness {type:3}` = `clipPath` com `rx = min(lado menor/4, 32)` — a regra do rectângulo (`corner_radius`) |
| opacidade | `opacity` no `<use>`: a imagem inteira |
| sem ficheiro | `status:"pending"` sem `files[fileId]`: o SVG OMITE-A; o PNG (canvas) pinta um cinzento com o ícone de imagem |
| o nosso | `w14_nosso`: as três imagens nas caixas, o recorte+espelho e a redonda rodada certas (`w14_nosso.png`); o `restore` mantém `fileId`, `scale`, `crop`, `status` e o `customData` — gate `the_excalidraw_round_trip_returns_our_images` |

Gates: `ph2d-board-excalidraw::image_tests` (o ficheiro dele abre com as imagens; sem descodificador
saltam e contam-se; a ida e volta), `ph2d-board-render::image_tests` (a lei do afim, a camada).

### W15 — moldura, SVG e filtro na imagem: `corre.sh saidas/w15 …` + `corre_esquema.mjs`

```bash
python3 entradas/gera_w15_casos.py      # w15_moldura · w15_svg · w15_sondas.json
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh bash $O/corre.sh $O/saidas/w15 $O/entradas/w15_moldura.excalidraw $O/entradas/w15_svg.excalidraw
for n in w15_moldura w15_svg; do PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node $O/corre_esquema.mjs \
  $O/entradas/$n.excalidraw $O/entradas/w15_sondas.json $O/saidas/w15/esquema_$n; done   # O=docs/MiroClone/ferramentas/excalidraw_oracle
```

Medido 2026-10-11 (PNG 1×, vermelho = `r>150, g<120, b<120`; a PNG da imagem é AZUL lisa):

| pergunta | o que o oráculo faz |
|---|---|
| contorno da imagem | `strokeColor #e03131` + `strokeWidth 4` (tracejado, tracejado+`roundness {type:3}`, sólido): **NÃO desenha**. PNG: **0** píxeis vermelhos em toda a imagem (440×280), 0 não-brancos no anel ±10 px de cada caixa, igual ao controlo; SVG: só o `<use>` (+ `clipPath rx 20` na redonda), nenhum `<rect>`/`<path>` de contorno. Idem nas variantes `escuro`/`sem_fundo`/`embed` |
| os campos do contorno | o `restore`, o `serializeAsJSON` e a volta `loadFromBlob` (json, png e svg embutidos) **guardam** `strokeColor`/`strokeWidth`/`strokeStyle`/`roundness` tal qual (`igual_ao_restore: true`) — dados mudos |
| imagem SVG | `files[..].mimeType "image/svg+xml"` com `data:image/svg+xml;base64,…`: aceita, `status "saved"`, devolve o MESMO `mimeType` e `dataURL` (restore, serialize, volta). SVG export: `<image href="data:image/svg+xml;base64,…" preserveAspectRatio="none" width="100%">`. PNG: com `width/height` no `<svg>` enche a caixa 100×100 (**7 846** píxeis vermelhos, círculo r=50 = 7 854); **só com `viewBox`** o canvas pinta-o a **80×80** no canto (4 995 ≈ r=40) — o PNG e o SVG do oráculo divergem aí |
| filtro P&B | nenhum: o `restore` de uma imagem tem 30 chaves e nenhuma é filtro. Chaves desconhecidas (`filter`, `grayscale`, `imageFilter`) **passam intactas** (o `restore` não as valida nem as usa). O único filtro é do export escuro do appState: `filter="invert(93%) hue-rotate(180deg)"` no `<svg>` inteiro |

Uma imagem mínima (`{type:"image", fileId}`) restaura com `strokeColor "#1e1e1e"` e `status "pending"`.

### W16 — o recorte, o espelho e o GIF no EDITOR: `sonda_imagem_w16.mjs`

```bash
python3 -I docs/MiroClone/ferramentas/excalidraw_oracle/entradas/gera_w16_imagens.py     # só se mudar as entradas
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/excalidraw_oracle/sonda_imagem_w16.mjs \
  [--so <prefixo>[,<prefixo>…]] [<pasta_entradas> <pasta_saida>]       # por omissão entradas/ e saidas/w16/
```

Entradas NOSSAS (`gera_w16_imagens.py`, Pillow): `w16_quadrantes.png` 400×300 — TL `(255,0,0)` · TR
`(0,160,0)` · BL `(0,0,255)` · BR preto (o canal a 0 dá o alfa: sobre branco `v = (1−α)·255`) — e
`w16_anim.gif` 120×120, 3 quadros de 200 ms em ciclo (vermelho · verde · azul). 32 sondas; cada uma monta
o `<Excalidraw>` de novo (1280×800, zoom 1, scroll 0 ⇒ cena = ecrã; imagem em `420,180`, 1:1 com o natural),
`addFiles` + espera a imagem no canvas, age com o rato/teclado do Playwright e grava DEPOIS de cada passo:
o elemento (`x y w h angle scale crop status`), o `appState` relevante, **todas** as chaves do `appState`
que mudaram, o cursor, a dica do rodapé, uma captura (`saidas/w16/w16_<sonda>_<nn>_<passo>.png`) e as
corridas de cor nas linhas `y=240`/`420` e colunas `x=520`/`720` (onde a imagem está no ecrã e com que
alfa). As pegas acham-se pelo CURSOR do `canvas.interactive`. Saída: `saidas/w16/w16_sondas.json`
(cabeçalho: versão, Chromium, data, `loadavg`, sha256 das entradas e do script) + o export do GIF
(`w16_C01_gif_export.{svg,png}`). Relógio LIVRE (o duplo clique precisa dele); PRNG semeado pelas
entradas. **Determinismo:** duas corridas (31 sondas, 12/10, `load` 30–100) = 92/92 capturas/SVG
idênticos byte a byte e o JSON igual fora dos tempos — salvo a amostra do `<img>` de CONTROLO do GIF,
que depende do relógio de parede (é para isso que existe).

Medido 2026-10-12 (Excalidraw 0.18.1, Chromium 153):

| pergunta | o que o oráculo faz |
|---|---|
| entrar | duplo clique na imagem, **Enter** com ela seleccionada, ou «Crop image» (botão do painel e menu de contexto) ⇒ `appState.croppingElementId = <id>`. `isCropping` só é `true` DURANTE o arrasto de uma pega (falso no pan). Dica: *«Click outside the image or press Enter or ESCAPE to finish cropping»* |
| pegas | **8**: cantos (`nwse`/`nesw-resize`, zona de ±6 px de ecrã à volta do canto) e meios dos lados (`ns`/`ew-resize`, ±2 px, sem desenho). Desenha só 4 «L» nos cantos; as MESMAS zonas fora do modo |
| canto SE −80,−60 | `x,y` fixos, `w,h` 400×300 → 320×240, `crop {0,0,320,240,400,300}` (`A02`) |
| proporção | **livre**: SE −80,−30 ⇒ 320×270 (`A02b`); com **Shift** fica a do natural 4:3 ⇒ 360×270 (`A02c`) |
| canto NW +80,+60 | o elemento ANDA (`x,y` 420,180 → 500,240) e `crop.x,y` = 80,60: a imagem fica PARADA no ecrã (o fantasma continua a começar em 420) (`A03`) |
| meios | E −80 ⇒ `w` 320, `crop.width` 320; N +60 ⇒ `y`+60, `crop.y` 60; S/W idem (`A04`; ruído `39.99999999999994`) |
| unidades | o `crop` é em px NATURAIS: imagem a metade (200×150), SE −40,−30 ⇒ caixa 160×120, `crop` 320×240 (`A02d`) |
| pan | arrastar DENTRO desliza a imagem sob a janela (cursor `move`; `x,y,w,h` não mudam), no MESMO sentido do ponteiro — mas **`Δcrop` = −2·Δponteiro em unidades de CENA, qualquer que seja a escala da imagem**: −50,−40 ⇒ `crop` +100,+80 (`A05`); −20,−10 ⇒ +40,+20 com 1, 4 ou 20 `pointermove` (`A05b`) e com a caixa a 0,5×/1×/1,5×/2× do natural (`A05c..g`); a zoom 2, −20,−10 de ecrã ⇒ +20,+10. Só a 0,5× a imagem segue o dedo |
| limites | o pan pára nas bordas naturais (`crop.x ∈ [0, 400−w]`: +300 ⇒ 0; −600 ⇒ 160,120); uma pega para FORA da imagem natural fica presa à borda (inteira: nada muda, nem a versão; recortada e SE +300,+300 ⇒ volta a 400×300 e `crop` passa a **`null`**) (`A06`) |
| mínimo | **10×10 em unidades de CENA** (a 1:1 `crop` 10×10; a metade, `crop` 20×20 naturais) — e arrastar para lá do canto oposto também dá 10×10, não vira (`A06b..d`) |
| sair | **Enter**, **Escape** e clique fora SAEM e **guardam** o recorte (o Escape não desfaz); o clique fora também tira a selecção; **duplo clique dentro NÃO sai** (`A07a..d`) |
| desfazer | por ARRASTO, não o recorte inteiro: depois de sair, o 1.º Ctrl+Z só volta a ENTRAR no modo (geometria igual); o 2.º desfaz o último arrasto (NW) e deixa o do SE; Ctrl+Shift+Z refaz. Dentro do modo, cada Ctrl+Z tira um arrasto até `crop null` (`A07`, `A08b`) |
| redimensionar depois | a pega NORMAL não toca no `crop`: a imagem recortada escala. Canto e lado mantêm a proporção da caixa (SE +80,0 ⇒ 480×360; E +60 ⇒ 540×405 com `y` −22,5, centrado na vertical); **Shift LIBERTA** a proporção e a imagem ESTICA (620×405, o vermelho natural 0..200 ocupa 387 px) — igual sem recorte (`A09`, `A09b`) |
| rodada 30° | suportado: as pegas seguem a rotação (a SE achada no canto rodado `718,560`); SE −80,−60 no referencial da imagem ⇒ `crop` 320,2×239,8 e o canto NW fica parado no ecrã (`x,y` mudam porque são da caixa não rodada) (`A10`) |
| espelhada | o `crop` é em px da imagem natural NÃO espelhada: com `scale [-1,1]`, SE −80,−60 ⇒ **`crop.x = 80`** (contra 0 sem espelho, `A02`) — o lado direito do ecrã é a esquerda natural; NW +40,+30 depois ⇒ `crop.x` fica 80, `width` 280 (`A11`) |
| esbatido | fora do recorte a imagem natural aparece a **α ≈ 0,1**: sobre branco o vermelho fica `(255,228,228)`, o verde `(228,244,228)`, o azul `(228,228,255)`, o preto `228` ⇒ `α = 27/255 = 0,106` (verde por G: 0,116). Ao sair desaparece (branco) |
| espelho | **Shift+H** ⇒ `scale [-1,1]`, **Shift+V** ⇒ `scale [1,-1]` (os dois ⇒ `[-1,-1]`); `x,y,w,h` NÃO mudam (vira à volta do centro); ×2 volta a `[1,1]` exacto. Rodada 30°: Shift+H dá `angle` 2π−θ (`5.7596`) e ×2 volta a `0.5235987755982991` (era `…988`: deriva 3e-16). Recortada: o `crop` NÃO muda, o recorte vira no lugar (`B02`) |
| botões | painel da imagem: «Crop image» (sem botão de espelho). Menu de contexto da imagem: Cut · Copy · Paste · Wrap selection in frame · **Crop image** · Copy to clipboard as PNG/SVG · Copy/Paste styles · Add to library · as 4 de ordem · **Flip horizontal [Shift+H]** · **Flip vertical [Shift+V]** · Add link · Copy link to object · Duplicate · Lock · Delete (`B01`) |
| GIF no canvas | **só o 1.º quadro**: 12 amostras em 2,2 s, sem e com re-render provado (com `scrollX` 1 a borda `x=560` fica branca), sempre `(255,0,0)`. CONTROLO: o mesmo GIF num `<img>` do DOM anima (vermelho → verde → azul a ~200 ms) e o `drawImage` desse `<img>` num canvas novo dá sempre vermelho ⇒ é a regra do `drawImage` do Chromium, não uma escolha só do Excalidraw (`C01`) |
| GIF no export | `getFiles()` guarda `mimeType "image/gif"`; o SVG leva `<image href="data:image/gif;base64,…">` com os bytes **iguais** aos da entrada (anima em quem o abrir); o PNG (`exportToBlob`) sai com o **1.º quadro**, igual nas 3 exportações espaçadas de 250 ms (o mesmo sha256) |

### W17 — a imagem ESTRAGADA e o `Shift` numa imagem esticada ou já recortada: `sonda_imagem_w17.mjs`

```bash
python3 -I docs/MiroClone/ferramentas/excalidraw_oracle/entradas/gera_w17_imagens.py     # só se mudar as entradas
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/excalidraw_oracle/sonda_imagem_w17.mjs \
  [--so <prefixo>[,<prefixo>…]] [<pasta_entradas> <pasta_saida>]       # por omissão entradas/ e saidas/w17/
```

Entradas NOSSAS (`gera_w17_imagens.py`, a partir da `w16_quadrantes.png`, lixo de `random.Random(17)`), todas com o
CABEÇALHO bom: `w17_png_truncado.png` (60 % dos bytes) · `w17_png_idat_lixo.png` (o IDAT trocado por lixo, CRC certo)
· `w17_jpeg_lixo.jpg` (a entropia depois do SOS trocada por lixo sem `0xFF`) · `w17_jpeg_truncado.jpg` (50 %). O
Pillow recusa três (`truncated`, `broken data stream`) e lê o `jpeg_lixo` como ruído. 18 sondas, a mesma infra-estrutura
da W16 (cena = ecrã), mais um `MutationObserver` que guarda TODO o texto que aparece no DOM (toasts que somem) e um
`drop` com `DataTransfer` (o caminho do gestor de ficheiros). Saída: `saidas/w17/w17_sondas.json` (cabeçalho com
versão, Chromium, data, `loadavg`, sha256 das entradas e do script) + capturas.

Medido 2026-10-12 (Excalidraw 0.18.1, Chromium 153, `load` 15 → 5):

| pergunta | o que o oráculo faz |
|---|---|
| a estragada chega COM a cena (`D01`) | `status` fica **`saved`** (igual à boa); **nenhum aviso** — zero toasts, diálogos, `errorMessage` ou erros na consola. O canvas desenha O QUE O NAVEGADOR DESCODIFICAR: `png_truncado` = as linhas de cima (vermelho/verde até `y≈352`), o resto em branco; `png_idat_lixo` = **NADA** (a imagem fica invisível — só a caixa de selecção a mostra); `jpeg_lixo` = ruído (cinzento 128 com franjas); `jpeg_truncado` = a metade de cima |
| a estragada é LARGADA (`D02`) | entra um elemento 400×300 (`status` `pending` também na BOA — é o «por gravar», não o «por ler»); o mesmo desenho; **nenhum aviso** a quem a largou |
| o export (`D01`) | SVG leva o `<image>` com os bytes estragados; o PNG sai em branco onde nada se leu (`png_idat_lixo`) ou com a parte lida |
| `Shift` com a imagem ESTICADA (`E01..E05`, `E08`) | a proporção é a do NATURAL **em píxeis naturais** (a caixa no ecrã guarda a do que se vê): 600×300 (kx 1,5), SE −80,−30 ⇒ `crop` 360×270, caixa **540×270**; NW +80,+30 ⇒ `crop` 40,30,360×270; 400×450 (ky 1,5) ⇒ `crop` 373,33×280, caixa 373,33×420; SE −20,−100 ⇒ `crop` 386,67×290 (manda o eixo com a caixa MAIOR). O caminho do artista (esticar com `Shift` pela pega normal e depois recortar) dá o mesmo (`E08`) |
| `Shift` numa imagem JÁ recortada (`E06`, `E07`) | ⭐ a proporção é a do **recorte de agora**, não a da imagem inteira: `crop` 200×300 (2:3) na caixa 200×300, SE −40,−30 ⇒ `crop` **180×270** (2:3; nós dávamos 360×270, 4:3); e esticada (caixa 300×300) ⇒ `crop` 180×270, caixa 270×270 |

⇒ **Gates:** `ph2d-board-edit` `crop::tests::shift_keeps_the_proportion_of_the_window_stretched_or_already_cropped` (os
números de `E02..E07`). ⚠️ **Divergência deliberada (D):** o Excalidraw não distingue «a chegar» de «ilegível» e não avisa
ninguém; a ordem do dono (12/10) é o contrário — uma recusa que só o terminal vê é botão mudo. Nós: a imagem que o
descodificador não lê desenha-se **riscada na tinta do perigo** e o quadro diz-o na faixa de avisos UMA vez por ficheiro;
não desenhamos a parte lida de um ficheiro cortado (o `image` recusa-o inteiro).

### W18 — o VÍDEO do YouTube colado (`embeddable`) e o `<text>` de um SVG: `sonda_w18.mjs`

```bash
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh timeout 900 node docs/MiroClone/ferramentas/excalidraw_oracle/sonda_w18.mjs --rede nao      # + as 16 sondas T
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh timeout 900 node docs/MiroClone/ferramentas/excalidraw_oracle/sonda_w18.mjs --rede youtube  # só YouTube/Google deixados passar
```

Saídas `saidas/w18/w18_sondas_{offline,online}.json` (45 + 29 sondas, **0 falhas**, 12/10). As fotos dos 24
links colados não se guardam (o JSON tem tudo); as do caminho `Z01..Z05` sim.

| pergunta | o Excalidraw 0.18.1 |
|---|---|
| que links viram vídeo (Ctrl+V de texto) | `watch?v=`, `youtu.be/`, `/shorts/`, `/embed/`, `/live/`, `youtube.com` sem `www`, `http://` → **`embeddable`** com `link` = o colado; `m.youtube.com`, `music.youtube.com`, `youtube-nocookie.com`, sem esquema (`www.…`) e texto com um link dentro → **texto** |
| o tamanho | **560×315** (16:9); `/shorts/` **315×560**; um link do YouTube que não é vídeo (canal, maiúsculas no domínio) **560×840** (o genérico) |
| onde nasce | o canto de cima-esquerdo no rato (640,400 → x 640, y 400) |
| o `src` do iframe | `https://www.youtube.com/embed/<id>?enablejsapi=1` (+ `&start=<t>` **cru**: `90s`, `1m30s` — o embed só entende segundos ⇒ ⛔ não copiar); `playlist?list=` → `embed/videoseries?list=`; `watch?v=…&list=` larga a lista |
| defeitos do oráculo (⛔ não copiar) | `/live/<id>` → `embed/live` (partido); `watch?feature=share&v=<id>` → `embed/watch` (partido); `WWW.YOUTUBE.COM` não é reconhecido; um id de 3 letras (`v=abc`) é aceite |
| parado | o iframe está SEMPRE carregado por cima do canvas, `pointer-events: none` |
| activar | **um clique** no vídeo ⇒ `activeEmbeddable.state = "active"`, o iframe recebe o rato; `Esc` não desactiva; clicar fora desactiva |
| redimensionar | livre (560×315 + (200,40) → 760×355); o **Shift não prende** a proporção |
| pan / zoom / rodar | o iframe segue por CSS (`transform` do contentor; a 30° o rectângulo do iframe roda) |
| export PNG | um rectângulo **cinzento arredondado** (`#d3d3d3`) com o link em texto ao meio (`w18_offline_Z04_export.png`) |
| export SVG | o mesmo, dentro de um `<a href="<link>">` (+ um `<a target=_blank>` à volta); sem `<image>`, sem `foreignObject` |
| `.excalidraw` | `type: "embeddable"`, `link`, `roundness: {type: 3}`, `strokeColor`/`backgroundColor` `transparent`; sem `validated` gravado |
| abrir um gravado | `validated` ausente, `true` ou `false` — os três mostram o iframe |
| sem rede | o rectângulo vazio, só o ícone ↗ no canto; com rede o tocador oficial (o vídeo de teste `dQw4…` diz «This video is unavailable» no Chromium do Playwright) |

### W21 — o clique longo num vídeo e as outras ferramentas sobre ele: `sonda_w21.mjs`

`sonda_w21.mjs` (como a `sonda_w20.mjs`: Playwright, YouTube real, a página como `https://ph2d.local/`); saída `saidas/w21/w21_sondas.json` (24 sondas; só as fotos de rectangle/freedraw/text/arrow ficaram, o JSON nomeia as podadas). A duração conta-se no relógio da PÁGINA (`event.timeStamp`), não no do Node. Medido: segurar parado num vídeo seleccionado activa até 283,5 ms e não activa desde 303,3 ms (nos dois sentidos); com qualquer das 11 ferramentas o clique/arrasto faz o da ferramenta e nunca activa o vídeo; a borracha apaga o vídeo. Leitura e decisão: [`10_video_do_youtube.md`](../../10_video_do_youtube.md) §W21.

### W23 — o vão do losango e do rectângulo redondo: `corre_fidelidade.mjs --prefixo w23`

```bash
python3 docs/MiroClone/ferramentas/excalidraw_oracle/entradas/gera_w23_vao_casos.py docs/MiroClone/ferramentas/excalidraw_oracle/entradas
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh node docs/MiroClone/ferramentas/excalidraw_oracle/corre_fidelidade.mjs --prefixo w23 --versao 0181 docs/MiroClone/ferramentas/excalidraw_oracle/entradas/w23_vao_casos.json docs/MiroClone/ferramentas/excalidraw_oracle/saidas
python3 docs/MiroClone/ferramentas/excalidraw_oracle/mede_w23_vao.py
```

**Casos:** 110 `V_*`, 28 188 setas (entrada 6,4 MB, saídas `saidas/w23_V_*.0181.json` 7,7 MB; a corrida de 20/10 tinha 98 —
a B9 juntou as medidas FRACCIONÁRIAS 173,4×97,8, g 6 · −5 · dentro, em cada forma): losango vivo, losango
`roundness` 2 e 3, rectângulo `roundness` 3 × 200×120 (vãos 0,01 · 1 · 3 · 6 · 10 · 20 · −5), 120×200 · 60×40 · 300×200
(1 · 6 · 20 · −5; o 300×200 passa o limiar 128 do raio adaptativo) × rodado 0,4 rad (6 · −5) × `gap 0` × «ponto fixo
dentro» com o par que o NOSSO escritor grava (focus da recta, gap = −distância ao nosso contorno); 24 ângulos × 11 focos
(−0,9…0,9) presas pelo fim (no 200×120 g 6 também pelo início); controlo: rectângulo vivo g 6 e −5 (W10, exacto).
**Novidade do corredor:** `"campos": [...]` num caso — só essas chaves de cada elemento vão para a saída.
**Controlo da máquina:** `F_diamond_g6` da W10 recorrido hoje = byte a byte igual à fixture. **Determinismo:** duas
corridas em paralelo (portas 47131/47132, `load` ~36): **98/98 ficheiros idênticos**; 218 s e 203 s.

**A lei** (referencial local: centro da CAIXA na origem, sem rotação; `hw, hh` as meias-medidas; `R(x)` = `x/4`, e no
`type 3` 32 acima de `x = 128`):
- **as peças alargadas de g** (g com sinal; `ox = ⌊w/2⌋+1−w/2`, `oy = ⌊h/2⌋+1−h/2` — o vértice do losango NÃO está no meio):
  - losango vivo: vértices `(ox, ∓(hh+g))`, `(±(hw+g), oy)` — cada um +g no SEU eixo;
  - losango redondo: o caminho do desenho — vértices `T(ox,−hh) R(hw,oy) B(ox,hh) L(−hw,oy)`, cantos cúbicos com os DOIS
    controlos no vértice entre `V ± (R(⌊w/2⌋+1), R(⌊h/2⌋+1))` ao longo das arestas (topo `(T.x∓vr, T.y+hr)`, direita
    `(R.x−vr, R.y∓hr)`…), arestas rectas entre as pontas; **cada canto transladado de g·V/|V|**;
  - rectângulo redondo (`r = R(min(w,h))`): cantos quadráticos (controlo no vértice) transladados de
    **(±g·(hw+g), ±g·(hh+g)) / √((hw+g)²+(hh+g)²)** — o lado fica a `hw + g·(hw+g)/|…|` (não `hw+g`).
- **a recta:** do ponto vizinho A pelo ponto do foco `|focus|·K` (XV7, K o vértice do losango simétrico `(0,±hh)`,
  `(±hw,0)` / o canto da caixa), como SEGMENTO longo a partir de A; **cada peça dá no máximo UM ponto** (a curva por
  Newton; as sementes não mudam o veredito);
- **≥ 2 pontos ⇒ o mais perto de A; 1 ⇒ o PONTO DO FOCO** (a corda que entra e sai pela MESMA curva de canto); 0 ⇒ a
  ponta não mexe; `gap 0` ⇒ o ponto do foco;
- **o `empurrar` são DUAS re-ligações** (forma +1 px, depois −1): a única seta que a lei de uma só falhava
  (`V_losango_r3_300x200_gneg5`, ponta em (1, −90)) é a 1.ª re-ligação que a 2.ª (sem pontos) deixou ficar.

**Resíduo máximo (px)** — colunas de hipótese: a 1.ª entrada da recta OBSERVADA (só o vão, sem as pontas no foco);
`LEI`: a lei inteira contra todas as setas:

| forma | gap | setas | fica | foco | hipóteses do vão | LEI |
|---|---|---|---|---|---|---|
| losango vivo | 0,01 · 1 · 3 · 6 · 10 · 20 · −5 | 5 808 | 48 | 0 | (a) Minkowski 4,8–33/∞ · (b) semi-diagonais+g 4,7–32 · (c) mitra 4,8–33/∞ · **(d) +g no eixo, vértice ⌊⌋+1: 0,000** | **0,0000** |
| losango vivo | 0 | 264 | 24 | 240 | — | **0,0000** |
| losango redondo 2 e 3 | 0,01 … 20 · −5 | 2 × 5 808 | 358 | 812 | (a) Minkowski do redondo 9,5–41/∞ · (b) o vivo 5,8–42 · (c) +g no eixo 0,001–1,7/∞ · (e) raios do alargado 0,005–16/∞ · (f) g·û(V) quadrática 7–15/∞ · **(f′) g·û(V) cúbica 0,000** (r3 −5: 4,17 = a seta das duas re-ligações) | **≤ 0,0022** |
| rectângulo redondo 3 | 0,01 … 20 · −5 | 5 808 | 148 | 240 | (a) Minkowski 0,07–53/∞ · (b) Chebyshev+g 12–83 · (c) lados+g mesmo r 0,07–65/∞ · **(d) g·û(hw+g,hh+g) quadrática 0,000** · (d′) cúbica 6,7–10 · (e) r do alargado 0,07–65/∞ | **≤ 0,0022** |
| as três | 0 | 3 × 264 | 72 | 720 | — | **0,0000** |

(o resíduo 0,0022 é a tolerância 1e-3 do Newton; |Δfocus| ≤ 0,00006 em todas, 0,0047 na seta das duas re-ligações.)
**O que a W10 leu como «sem lei»:** o vértice do losango a ⌊w/2⌋+1 (as arestas ajustadas a um losango simétrico davam
«x 4,43, y 5,39» para g 6); no rectângulo redondo g 6 «5,09 no lado, 3,17 no topo» = 6·106/124,87 e 6·66/124,87; e as
pontas que caem no PONTO DO FOCO (1 só cruzamento).

**«Ponto fixo dentro»** (o par do nosso escritor): a ponta cai, de mediana, 21 px (losango 200×120), 24 px (losango
redondo), 65 px (rectângulo redondo 200×120) e até 193 px do ponto gravado — a LEI prevê onde (≤ 0,0017 px): a entrada
da recta na forma ENCOLHIDA de |gap| (`gap = −distância` não é o nível que passa pelo ponto), ou o ponto do foco.

**B9 (20/10) — o escritor e o gate.** A lei está em `ph2d-board-excalidraw/src/legacy_focus.rs` (`pieces`,
`landing_pieces`; o Newton dele portado, sementes 0,5 · 0,2 · 0,8 a 1e-3, o ponto aceite afinado) e o escritor grava o
par que a põe na NOSSA ponta (o nível que passa por ela; inexprimível ⇒ o vão de menor erro). Medidas fraccionárias
confirmam o `⌊w/2⌋+1` e o `R(⌊w/2⌋+1)`: lei ≤ 0,0012 nos 173,4×97,8; as 110 em Python ≤ 0,0022, em Rust ≤ 0,0022.
**No repo só 15 saídas (947 KB)**, o subconjunto do gate `w23_the_pieces_law_reproduces_the_0181_sweep`
(`fidelity_w23_tests.rs`): `V_losango_{173p4x97p8_gneg5, 200x120_rot_g6, 200x120_g0, 173p4x97p8_dentro}`,
`V_losango_r2_{173p4x97p8_g6, 200x120_rot_gneg5, 200x120_g0, 173p4x97p8_dentro}`, `V_losango_r3_300x200_{g6, gneg5}`
(o tecto do raio; a 2.ª re-ligação sem cruzamentos), `V_retangulo_r3_{173p4x97p8_g6, 200x120_rot_gneg5, 300x200_g1,
200x120_g0, 173p4x97p8_dentro}` — cada família × vão + · − · 0 · dentro · rodado · fraccionário, e os ramos «entrada»,
«ponto do foco com um só cruzamento» e «fica». A entrada (6,4 MB) e as outras 95 saídas NÃO se guardam: regeneram-se
pelos três comandos acima (determinístico; o cabeçalho das 15 leva `casos_sha256` `1eb66aa8…c2cef328`, o da entrada que
o gerador de hoje escreve).
