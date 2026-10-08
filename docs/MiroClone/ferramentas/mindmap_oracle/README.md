# Oráculo de mapa mental — mind-elixir · simple-mind-map · markmap, corridos por script, nunca lidos

Corre as três bibliotecas num Chromium headless (Playwright 1.63.0, Chromium 153.0.8010.12, partilhado
com `../tldraw_oracle/` e `../excalidraw_oracle/`) sobre as árvores NOSSAS de [`entradas/`](entradas/),
mede o DOM/SVG em execução e carrega no teclado com o **rato e o teclado reais** do Playwright. Serve a
onda W6 (mapa mental no Quadro): as leis de layout e de teclado que o nosso motor em Rust tem de igualar
ou recusar com razão.

## Licenças (triadas por artefacto instalado, 2026-10-08) — as três ACEITES

| pacote | versão (exacta) | licença do artefacto | ficheiro lido |
|---|---|---|---|
| `mind-elixir` | **6.0.0-next.4** (o `latest` do npm; o último não-pré é 5.15.1) | MIT | `mind-elixir/LICENSE` (1 pacote no fecho: zero dependências) |
| `simple-mind-map` | **0.14.0-fix.3** (`latest`) | MIT | ⚠️ o artefacto **não traz ficheiro de licença**: vale o campo `license: "MIT"` do `package.json` instalado e o banner `@license MIT` do `dist/`; o `LICENSE` do repositório (`wanglin2/mind-map`, ramo `main`) é MIT. O `README_MORE_ZH.md` do repositório **pede** para uso comercial manter o aviso de copyright e a origem («基于SimpleMindMap项目开发») ou pagar para o remover — irrelevante para um oráculo local (nada é distribuído nem copiado) |
| `markmap-lib` / `markmap-view` | **0.18.12** / **0.18.12** (`latest`; `markmap-common` 0.18.9) | MIT | `markmap-*/LICENSE` |
| `esbuild` / `playwright` | 0.28.2 / 1.63.0 | MIT / Apache-2.0 | só ferramentas |

Fecho completo (194 pacotes, [`licencas.mjs`](licencas.mjs) lê o `package.json` E o ficheiro
`LICENSE*/COPYING*` de cada um e atribui a raiz pelo `package-lock`): **126 MIT · 43 ISC (d3 e família)
· 9 BSD-2 · 7 BSD-3 (`quill`, `highlight.js`, `d3-ease`…) · 3 Apache-2.0 · 1 0BSD · 1 Unlicense
(`robust-predicates`) · 1 Python-2.0 (`argparse`, de `markmap-lib`) · 1 BlueOak-1.0.0 (`sax`) ·
1 `(MIT AND Zlib)` (`pako`) · 1 `(MIT OR GPL-3.0-or-later)` (`jszip`, vale a opção MIT)**. Nenhum
não-permissivo; 30 pacotes sem ficheiro de licença valem pelo campo `license` (todos MIT/ISC).
Tabela inteira: `cd ~/Apps/mindmap-oracle && node <esta pasta>/licencas.mjs --md`.

## O que foi LIDO (e só isto)

- **mind-elixir:** `readme.md` instalado; os `.d.ts` instalados `dist/types/index.d.ts`,
  `types/index.d.ts`, `types/dom.d.ts`, `plugin/keypress.d.ts` (nomes de opções e métodos); a página de
  atalhos `docs.mind-elixir.com/docs/guides/shortcuts`.
- **simple-mind-map:** `README.md` instalado (2 linhas, aponta para o GitHub); a documentação em
  `wanglin2.github.io/mind-map-docs` (páginas *constructor options/methods*, *node*, *render*,
  *keyCommand*); do repositório só `LICENSE`, `README.md`, `README_EN.md`, `README_MORE_ZH.md` (secção de
  licença). O pacote instalado não traz `.d.ts`.
- **markmap:** `README.md` de `markmap-lib`/`markmap-view`; os `.d.ts` `markmap-view/dist/view.d.ts`,
  `constants.d.ts`, `markmap-common/dist/types/common.d.ts`.
- **Nenhum código-fonte** foi aberto (nem `src/` — o `simple-mind-map` instala-o —, nem `dist/*.js`, nem
  o `keyMap.js` para onde a doc do simple-mind-map remete). Única leitura de `dist/`: um `grep` de
  `@license` no bundle do simple-mind-map, para a triagem.

## Como reproduzir

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone          # ou o primário depois da integração
bash docs/MiroClone/ferramentas/mindmap_oracle/instala.sh                # 1× por máquina: ~/Apps/mindmap-oracle (156 MB)
node docs/MiroClone/ferramentas/mindmap_oracle/gera_entradas.mjs         # só se mudar as árvores (determinístico)
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh \
  node docs/MiroClone/ferramentas/mindmap_oracle/corre.mjs docs/MiroClone/ferramentas/mindmap_oracle/saidas
# um caso só: ... corre.mjs --so layout_me_SIDE,teclas_smm_tab <saida>
# só as leis, a partir de saídas existentes: node .../leis.mjs <saida>
```

A página ([`src/`](src/)) é um bundle esbuild que importa os pontos de entrada PÚBLICOS
(`mind-elixir`, `simple-mind-map/dist/simpleMindMap.esm.js`, `markmap-view`, `markmap-lib`) e o CSS que o
README manda usar, servida por um http local do próprio script. Janela 1600×1000, `deviceScaleFactor 1`,
`en-US`, UTC. Uma corrida inteira: **~5 min** (39 casos).

**Saídas** (`saidas/<caso>.json` + `<caso>.meta.json` — versões, data, comando, sha256 do corredor, das
leis, das entradas e da saída, consola, carga):
- `layout_<app>_<layout>.json` (17): por árvore T1..T6, cada nó `[id, texto, pai, prof, x, y, w, h, lado,
  recolhido]` (caixa em coordenadas do mundo, **centro da raiz = (0,0), zoom 1**; ME: `getBoundingClientRect`
  do `me-tpc` / `scaleVal`; SMM: `Node.left/top/width/height`; markmap: `state.rect` — o `g` do nó no SVG
  tem exactamente essa translação) e cada ramo `{de, para, ini, fim, d, m}` (o `d` em bruto + a matriz
  local→mundo; as pontas casadas com o nó mais próximo, `folga` = distância).
- `teclas_<app>_<grupo>.json` (2 × 9): cada sonda = árvore, nó inicial (seleccionado por **clique real**),
  teclas, estado antes/depois (nós, selecção, `editando`, foco).
- `leis.json` — as leis destiladas por [`leis.mjs`](leis.mjs); `custo.json` (+ os tempos no `.meta`).

**Determinismo — medido 2026-10-08:** duas corridas completas (`saidas/` e `saidas_corrida2/`): **39/39
`.json` idênticos byte a byte (`cmp`)**, 39/39 `.meta.json` iguais excluídos `date`, `loadavg_1min` e
`tempos_ms`. Normalizações declaradas: `Math.random`/`crypto.getRandomValues`/`crypto.randomUUID`
semeados pelo nome do caso; ids de nós criados pelo teclado → `novo1, novo2…` (pré-ordem); números a
1e-3 (também dentro do `d`). Os tempos não são determinísticos e só vivem no `custo.meta.json`.

⚠️ **Artefactos de sonda medidos (e contornados):** (1) no mind-elixir um **2.º clique no nó já
seleccionado a 150 ms entra em edição** (a 700 ms não; `teclas_me_clique`) — as setas remontam o mapa a
cada sonda; (2) no simple-mind-map os atalhos só valem **com o rato dentro do SVG**
(`enableShortcutOnlyWhenMouseInSvg: true`) — o clique real deixa-o lá; (3) com 60 ms entre teclas o
simple-mind-map não desfazia o `Delete` — as sequências usam **400 ms** (ritmo humano).

## Leis de layout (árvores T2 = 13 nós desequilibrada · T3 = 50 nós · T6 = 7 filhos)

Vão «borda» = da borda do pai à borda do filho no eixo do layout; «faixa» = da caixa que envolve uma
subárvore inteira à da subárvore irmã seguinte. Centragem = centro do pai − centro de referência.

| app · layout | vão entre níveis, borda (0→1 / 1→2 / 2→3) | irmãos: faixa a faixa (prof. 1 / ≥2) | centragem do pai | compacta (tidy)? | equilíbrio (dois lados) |
|---|---|---|---|---|---|
| **ME** `RIGHT` / `LEFT` | **75 / 40 / 60** (filhos alinham a borda próxima: espalhamento 0) | **61–67 / 22** (caixa a caixa varia 22–313) | no **meio da faixa das subárvores** dos filhos (−5 nos pais de prof. 1, −3/−5 na raiz, 0 abaixo); média 1.º/último varia −44..47 | **não** (faixas ≥ 22; nenhuma sobreposição em T2/T3) | — |
| **ME** `SIDE` | 75 / 40 / 60 | 61–67 / 22 | idem, por lado | não | **alterna por índice, 1.º à ESQUERDA: E D E D…** — ignora o tamanho (T2: A(9 nós) E, B D, C E, D D; T6: 4 E / 3 D) |
| **ME** `DOWN` | **120 / 40 / 30** (eixo y) | **110 / 20** | meio da faixa (−0,01) | não | ramos em linha por baixo da raiz |
| **SMM** `logicalStructure` / `…Left` | **104 / 54 / 54** | **44 / 4** | meio da faixa: **0** exacto | não | — |
| **SMM** `mindMap` | 104 / 54 / 54 | 44 / 4 | 0 | não | **pela CONTAGEM e pela ordem: os ⌈n/2⌉ primeiros à DIREITA**, o resto à esquerda (T1 D D E · T2 A(9),B D / C,D E · T6 4 D / 3 E) — ignora a altura |
| **SMM** `organizationStructure` | 104 / 54 / 54 (eixo y) | 44 / 4 | 0 | não | — |
| **markmap** (padrão, só à direita) | **80 / 80 / 80** (= `spacingHorizontal`; a caixa já inclui `paddingX` 8 de cada lado) | caixa a caixa **6,19–6,375** em prof. ≥ 2; faixas **negativas** | **na média do centro do 1.º e do último filho: 0 exacto** (não na faixa: T2 raiz a 26,4 do meio da faixa) | **SIM** — subárvores entram na faixa da vizinha: T2 `A2|A3` −19,8, `A3|A4` −19,8, `A|B` −30,7; T3 7 pares até −32,9 | — |

Layouts mistos do SMM (`catalogOrganization`, `timeline`, `timeline2`, `fishbone`, `fishbone2`,
`verticalTimeline`, `…2`, `…3`) estão medidos (`layout_smm_*.json`) mas não destilados: o eixo muda por
nível; o `leis.json` dá-lhes números aresta a aresta só como descrição.

**Caixas por profundidade** (T3): ME raiz 118,9×60, prof. 1 h 44, prof. ≥ 2 h 30 (`DOWN`: 38); SMM raiz
h 30, prof. 1 h 31, ≥ 2 h 27; markmap h 20 em todas. **Texto (T4):** 80 caracteres → ME 560×68 (quebra
em 2 linhas), SMM 531×50 (quebra; `max-width` 500 do texto), markmap 578×20 (`maxWidth: 0` = não quebra).
Duas linhas (`\n`; no markmap `<br>`): ME 152×68 (2 linhas) · **SMM 192×31 (o `\n` é ignorado: 1 linha)**
· markmap 113×40.

**Recolher (T5 − T2, A recolhido):** o nó recolhido fica no sítio e os que vêm DEPOIS dele no mesmo lado
sobem a altura libertada — ME `SIDE`: só C sobe 150 (B e D, do outro lado, 0); ME `RIGHT`: B, C, D −150;
SMM `mindMap`: só B −91; SMM `logicalStructure`: B, C, D −91. **markmap re-centra tudo:** A +15,3, B 0,
C e D −15,3.

**Custo** (só referência; corrida 1 com a máquina a `load 36` — o nosso mede-se em Rust; mínimo de 5
rodadas intercaladas, página nova por rodada; montar = construtor + layout + DOM + reflow; refazer =
relayout da mesma árvore):

| ms | ME `SIDE` | ME `RIGHT` | SMM `mindMap` | SMM `logical` | markmap |
|---|---|---|---|---|---|
| 100 nós, montar / refazer | 3,6 / 2,6 | 3,3 / 2,4 | 19,7 / 26,9 | 21,2 / 26,8 | 9,1 / 15,1 |
| 1000 nós, montar / refazer | 38 / 26,7 | 25,3 / 23,3 | 271 / 263 | 262 / 275 | 38,2 / 38,1 |

## Leis de teclado (ME = mind-elixir em `SIDE`; SMM = simple-mind-map em `mindMap`)

| tecla | mind-elixir | simple-mind-map |
|---|---|---|
| **Tab** | filho novo como **ÚLTIMO** filho, do lado do pai (na raiz T1: o 4.º ramo cai à DIREITA pela alternância e B, o outro desse lado, sobe 54,5 para re-centrar); **seleccionado e em edição** (`#input-box`, texto «New Node») | idem: **último** filho, seleccionado, **em edição** (`ql-editor`, texto «分支主题»/«二级节点»); na raiz T1 o 4.º ramo cai à ESQUERDA pela contagem e C, o outro desse lado, sobe 37,5 para re-centrar |
| **Enter** | irmão **imediatamente DEPOIS** do actual (A → entre A e B), seleccionado + edição; **na raiz = filho (último)** | irmão imediatamente depois, seleccionado + edição; **na raiz: nada** |
| **Shift+Enter** | irmão **imediatamente ANTES**; na raiz = filho (último) | nada |
| **Shift+Tab** | = Tab (filho último) | **pai novo** inserido entre o nó e o pai (B → `root > novo > B`), seleccionado + edição; na raiz nada |
| **Ctrl+Enter** | **pai novo** (como o Shift+Tab do SMM) | nada |
| **↑ / ↓** | irmão anterior/seguinte **do mesmo lado** (A↓C em `SIDE`, salta B); **no 1.º/último irmão fica** (não pula para o primo) | **espacial**: o nó mais próximo nessa direcção, atravessando subárvores e lados (A3a↑A2, A3c↓A4, root↓D) |
| **→ / ←** | para fora da raiz = **1.º filho** (A3 ← A3a no lado esquerdo); para dentro = **pai**; folha para fora fica; **na raiz = o filho do MEIO desse lado** (⌊(n−1)/2⌋: T1 RIGHT → B, T6 RIGHT → C4, T6 SIDE ← C3 / → C4) | espacial: → em A dá A3 (o filho à altura), em A1/A2 dá A3a; ← na folha A3c dá a raiz; na raiz `mindMap` T6 → C2, ← C6 |
| **Delete / Backspace** | apaga a **subárvore inteira**; selecção → **PAI** (sempre); na raiz nada; Shift+ igual | apaga a subárvore; selecção → **irmão SEGUINTE** (ou o anterior se era o último: A5→A4, A3c→A3b); **na raiz apaga TODOS os filhos**; **Shift+Backspace apaga SÓ o nó e promove os filhos** para o lugar dele; Shift+Delete nada |
| **recolher** | **nenhuma tecla simples** (Espaço, `/`, Ctrl+/, Alt+/, `-`: nada); `Ctrl+K Ctrl+0` recolhe TUDO e limpa a selecção | **`/` alterna** recolher/expandir o seleccionado; Espaço nada |
| **Ctrl+Z depois de Tab** | ainda em edição: vai para o texto (a árvore fica); depois de `Esc` ou `Enter`: **remove o nó novo e volta a seleccionar A**; `Ctrl+Y` e `Ctrl+Shift+Z` refazem (nó novo seleccionado); `Delete`+`Ctrl+Z` repõe a subárvore com A3 seleccionado | `Esc` **não sai** da edição (Tab Esc Ctrl+Z: tudo igual); depois de `Enter`: remove o nó novo, **selecção vazia**; `Delete`+`Ctrl+Z` repõe a subárvore, selecção vazia |
| **2.º clique no seleccionado** | a 150 ms **entra em edição**; a 700 ms não | não entra em edição |

Ficheiros: [`corre.mjs`](corre.mjs) (corredor, casos, lado da página) · [`leis.mjs`](leis.mjs) ·
[`gera_entradas.mjs`](gera_entradas.mjs) · [`entradas/`](entradas/) (T1..T6, N100, N1000) ·
[`src/`](src/) · [`instala.sh`](instala.sh) · [`licencas.mjs`](licencas.mjs) ·
[`package.json`](package.json) + [`package-lock.json`](package-lock.json) · [`saidas/`](saidas/) ·
[`saidas_corrida2/`](saidas_corrida2/).
