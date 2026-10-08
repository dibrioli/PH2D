# Oráculo tldraw — guias inteligentes (snapping), corrido por script, nunca lido

Corre o **tldraw 5.5.2** (fixado; o último estável no npm em 2026-10-08) num Chromium headless, monta o
`<Tldraw hideUi onMount>` e arrasta formas com o **rato real do Playwright** sobre cenas NOSSAS. Só API
pública documentada (a do `DOCS.md` que vem no pacote = tldraw.dev): `editor.createShapes`,
`deleteShapes`, `select`, `selectNone`, `setCamera`, `setCurrentTool`,
`user.updateUserPreferences({ isSnapMode })`, `snaps.getIndicators()`, `getShapePageBounds`,
`getShapePageTransform`, `getShape`, `getSelectionPageBounds`, `getInstanceState().cursor`,
`pageToScreen`. **Nenhum código-fonte do tldraw foi lido** (nem `node_modules`, nem GitHub); do GitHub
leu-se só o `LICENSE.md` da tag `v5.5.2`, para onde o `LICENSE.md` instalado aponta.

## Licença (triada por artefacto instalado, 2026-10-08)

O `LICENSE.md` instalado de `tldraw`, `@tldraw/editor` e `@tldraw/driver` diz só *«This code is
licensed under the [tldraw license](https://github.com/tldraw/tldraw/blob/main/LICENSE.md)»*; o
`package.json` deles diz `"SEE LICENSE IN LICENSE.md"`. O texto da licença na tag `v5.5.2` está
copiado verbatim em [`LICENCA_tldraw_v5.5.2.md`](LICENCA_tldraw_v5.5.2.md).

| pacote | versão | licença | ficheiro lido |
|---|---|---|---|
| `tldraw` | 5.5.2 | **tldraw license** (não-OSI) | `tldraw/LICENSE.md` → `LICENSE.md` da tag v5.5.2 |
| `@tldraw/editor` | 5.5.2 | **tldraw license** | `@tldraw/editor/LICENSE.md` |
| `@tldraw/driver` | 5.5.2 | **tldraw license** | `@tldraw/driver/LICENSE.md` |
| `@tldraw/state` · `state-react` · `store` · `tlschema` · `utils` · `validate` | 5.5.2 | MIT | `@tldraw/<p>/LICENSE.md` |
| `react` / `react-dom` / `scheduler` | 19.2.8 / 19.2.8 / 0.27.0 | MIT | `LICENSE` |
| `@tiptap/*` (30) · `prosemirror-*` (13) · `@radix-ui/*` (60) + `radix-ui` · `@floating-ui/*` (4) · `@types/*` (3) · 24 utilitários (`classnames`, `lz-string`, `rbush`, `lodash.*`, `use-*`, …) | — | MIT | `LICENSE`/`LICENSE.md` de cada |
| `idb` 7.1.1 · `quickselect` 2.0.0 | — | ISC | `LICENSE` |
| `tslib` 2.8.1 | — | 0BSD | `LICENSE.txt` |
| `playwright` / `playwright-core` | 1.63.0 | Apache-2.0 | `LICENSE` |
| `esbuild` / `@esbuild/linux-x64` | 0.28.2 | MIT | `esbuild/LICENSE.md` (o binário não traz ficheiro; campo `license`) |
| Chromium do Playwright | build 1243 (153.0.8010.12) | BSD-3 | partilhado com `../excalidraw_oracle/` |

Contagem dos 154 pacotes instalados: 146 MIT · 3 tldraw license · 2 Apache-2.0 · 2 ISC · 1 0BSD.
A tabela completa (pacote · versão · `license` · ficheiro · 1.ª linha) sai de
`cd ~/Apps/tldraw-oracle && node <esta pasta>/licencas.mjs --md`; dois pacotes não trazem ficheiro de
licença (`@esbuild/linux-x64`, `react-remove-scroll-bar`) e valem pelo campo `license` (MIT).

**O que a tldraw license permite para o nosso uso.** Permite *«Use the Software in Development
Environments»* — definido como hospedagem interna *«for development, testing, or staging purposes …
not accessible to end users, customers, or the public»*; proíbe o uso em *Production Environments* sem
chave, interferir com a verificação da chave e distribuir o Software. Correr o tldraw **localmente**,
num Chromium headless desta máquina, como **oráculo de medição** é uso de desenvolvimento/teste: é
permitido. O `node_modules` e o bundle vivem FORA do repo (`~/Apps/tldraw-oracle`, `.gitignore` aqui) —
nada do tldraw é distribuído nem embarca no PH2D, e **nenhum código foi copiado**: o PH2D só recebe as
LEIS medidas (números abaixo) e as fixtures JSON de comportamento. Os pedidos à CDN do tldraw
(`https://cdn.tldraw.com/5.5.2`, ícones/traduções) **não são bloqueados** (seria interferir com a
verificação) — são registados no `.meta.json`.

## Uso

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone      # ou o primário depois da integração
bash docs/MiroClone/ferramentas/tldraw_oracle/instala.sh              # 1× por máquina: ~/Apps/tldraw-oracle (100 MB) + Chromium (já partilhado)
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh \
  node docs/MiroClone/ferramentas/tldraw_oracle/corre_guias.mjs docs/MiroClone/ferramentas/tldraw_oracle/saidas
# um caso só: ... corre_guias.mjs --so G4 <saida>
```

Por caso `G1..G8`: `saidas/guias_<caso>.json` (cada sonda: `cena`, `gesto` — delta pedido, pontos de
ecrã —, `previsto.proximosX/Y` — que traços ficam a ≤ 8,5 sem guia, i.e. porque a sonda discrimina —,
`durante.indicadores` = `editor.snaps.getIndicators()` com o botão AINDA em baixo, `depois` = geometria
depois de soltar, `nudge` = deslocamento real − pedido, em unidades de página) e
`guias_<caso>.meta.json` (versões, data, comando, sha256 do corredor e da saída, pedidos externos,
consola). Janela 1280×800, `deviceScaleFactor 1`, câmara `{x:0,y:0,z}`, `isSnapMode: true` salvo G8.

**Determinismo:** `Math.random`/`crypto.getRandomValues` semeados pelo sha256 do nome do caso, página
nova por caso. Medido 2026-10-08: duas corridas completas (`saidas/` e `saidas_corrida2/`) = **16/16
ficheiros idênticos byte a byte** (os `.meta.json` excluindo o campo `date`).

⚠️ **Artefacto do gesto por script (medido):** se todos os `mouse.move` caem no MESMO quadro em que o
arrasto começa, a forma chega SEM guia (`durante.chegadaRapida`: 0 indicadores, delta exacto); qualquer
movimento num quadro seguinte prende. Não é velocidade: `G8.velocidade_*` prende de 0,5 a **40 px por
passo de 16 ms**. O corredor por isso pausa 300 ms, anda 0,5 px e volta ao alvo exacto antes de ler.

## As leis medidas (tldraw 5.5.2)

- **G8 — limiar e modos.** Prende com o desvio **≤ 8 px de ECRÃ, inclusivo**, a zoom 1 e a zoom 2
  (varrimento 0,5..15 em passos de 0,5: 8,0 prende, 8,5 não; a zoom 2 o nudge é metade — 4 de página).
  Modos (B a 3 px): `isSnapMode` ligado sem Ctrl **prende**; ligado + Ctrl **não**; desligado sem Ctrl
  **não**; desligado + Ctrl **prende** (Ctrl inverte o modo).
- **G1 — movedor rodado (15°, 30°, 37°, 45°):** os pontos de guia do movedor são os **4 vértices
  RODADOS + o centro**, nunca os cantos da AABB. (b) um vértice INTERIOR (o de cima ou o de baixo) a
  3 px da esquerda de A, com a AABB longe, **prende** (nudge −3, em todos os 8 casos); o indicador liga
  `(100,100)–(100,220)` de A ao próprio vértice (`(100,381)` a 15°, i.e. o y do vértice). (a) com a
  AABB-esquerda a 3 px o indicador marca o vértice mais à esquerda (`(100,477.59)` a 15°), não um canto
  da AABB. (c) centros a 3 px prendem (−3). Concorrência: ganha o traço mais PRÓXIMO — a 30° em (a) o
  centro estava a 2,72 e ganhou ao −3 da esquerda (nudge +2,718).
- **G2 — alvo rodado 30°:** o alvo oferece os **seus vértices rodados** (+ centro): os indicadores
  marcam `(240,253.92)` v3, `(300,150)` v0, `(413.21,353.92)` v2, `(473.21,250)` v1 — os vértices
  interiores à AABB (v0 em x, v1 em y) prendem com nudge −3/+3; nenhum canto da AABB do alvo aparece.
- **G3 — ambos a 30°:** **zero indicadores oblíquos** (0 em todas as saídas). A guia é sempre por EIXO
  da página (x e y independentes) entre vértices/centros; aresta paralela a 3 px pela normal NÃO
  encosta a aresta — prende ao vértice mais próximo em cada eixo (nudge `(0.10, −0.80)`); vértice contra
  vértice a `(3, −3)` prende nos dois eixos (`(−3, +3)`, um ponto `(400,200)` por eixo).
- **G4 — rotação pelo puxador** (achado pelo cursor `nesw-rotate` a `(+7, −7)` px do canto superior
  direito da selecção): **não prende ao ângulo de outras formas nem a 0** (A a 30°: 29,5 → 29,5; 29,7 →
  29,7; 30,3 → 30,3 durante o gesto) e **não mostra indicadores**; ao SOLTAR o ângulo é arredondado ao
  **grau inteiro** (29,5 → 29 · 29,7 → 30 · 10,3 → 10 · 10,7 → 11 · 0,6 → 1 · 0,4 → 0 · 17,25 → 17;
  −1 → 359). Com **Shift: passos de 15°**, para o mais próximo (8 → 15 · 20 → 15 · 37 → 30 · 50 → 45 ·
  97 → 90).
- **G5 — espaços iguais** (`type: 'gaps'`): duplicação do espaço 40 com 43 → 40 (−3), 37 → 40 (+3), 48
  → 40 (−8, dentro do limiar inclusivo), à esquerda também; centrar num vão 200 com B 100: 53/47 →
  50/50 (−3), 57/43 → (−7); vertical igual. Os espaços medem-se pela **AABB** (B rodado 30°: o
  indicador usa a aresta da AABB `x=580`). O indicador lista os DOIS espaços (o de referência e o novo).
- **G6 — molduras:** a moldura é **alvo** (cantos e centro: R prende a `F.right=500` e `F.left=100`) e
  arrasta-se com guia (`F.right → O.right` −3). Um **filho prende ao irmão e à moldura-mãe**, mas
  **NÃO a formas fora da moldura** (c1 → O2 a 3 px: nudge 0, 0 indicadores). **Moldura rodada 30°:**
  os indicadores ficam nos **eixos da PÁGINA** (0 oblíquos) e alvejam os vértices rodados da moldura e
  dos irmãos em página; o alinhamento LOCAL com o irmão não existe (topo local 173 contra 170 acabou em
  171,29 — prendeu ao y de página do vértice v3 da moldura, nudge −1,976).
- **G7 — setas e traços livres:** o traço livre (`draw`) é **alvo pela sua AABB** (cantos
  `(180,150)/(180,200)` e centro `(280,175)`; esquerda e direita prendem −3). A **seta NÃO é alvo**
  (esquerda, direita e centro da AABB a 3 px: nudge 0, 0 indicadores).

Ficheiros: [`corre_guias.mjs`](corre_guias.mjs) (corredor e casos) · [`src/`](src/) (a página) ·
[`instala.sh`](instala.sh) · [`licencas.mjs`](licencas.mjs) · [`saidas/`](saidas/).
