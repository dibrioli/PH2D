# Handoff de integração — `line/Vector`: O ATLAS DE RASTER (plano 24 W10)

**Linha:** `line/Vector` · **Worktree:** `Worktrees/line-Vector` · **Base:** `78d770370`
**Estado:** fechada, **pendente de smoke**

⚠️ **Esta é a SEXTA wave da mesma abertura de linha,** e as seis **compartilham o bump de schema**
(`PROJECT_SCHEMA` 38). Integre-as juntas — o `main` nunca viu nenhuma:

1. [`…_fx_blend_…`](HANDOFF_INTEGRACAO_line_Vector_fx_blend_2026-07-28.md) — a lei de mistura
2. [`…_turbulence_…`](HANDOFF_INTEGRACAO_line_Vector_turbulence_2026-07-28.md) — a turbulência
3. [`…_morphology_…`](HANDOFF_INTEGRACAO_line_Vector_morphology_2026-07-28.md) — Grow / Shrink
4. [`…_colour_adjust_…`](HANDOFF_INTEGRACAO_line_Vector_colour_adjust_2026-07-28.md) — Color Adjust
5. [`…_duotone_…`](HANDOFF_INTEGRACAO_line_Vector_duotone_2026-07-29.md) — Duotone + Luma to Alpha
6. **esta** — o atlas de raster

⚠️ **É a primeira wave desta linha que NÃO acrescenta um tipo de efeito.** Ela não muda um pixel do
que o artista vê — muda quanto custa vê-lo. O oráculo de todos os gates dela é **o produto de
ontem**, byte a byte.

---

## O que muda

Uma cena de formas filtradas pagava **um render do Vello por forma**. Agora todas as que erram o
memo são rasterizadas **numa passagem só**, num ATLAS, cada uma na sua célula.

**Medido na RTX, o mesmo trabalho pelos dois caminhos:**

| N formas filtradas | ontem | hoje | ganho |
|---|---|---|---|
| 4 | 0,79 ms | 0,45 ms | 1,76× |
| 16 | 3,26 ms | 1,16 ms | **2,81×** |
| **32** | **6,00 ms** | **2,22 ms** | **2,7×** (2,58–2,82 em quatro corridas) |

**32 formas filtradas saem de 39 % de um quadro de 60 fps para 14 %.**

---

## Por que isto importa, e por que só agora

Num **editor** o artista mexe numa forma de cada vez, então o memo protege tudo e o defeito é
invisível. Num **jogo** as formas filtradas ANIMAM, logo erram o memo **todas, todo frame** — e o
que multiplica é o custo por-FORMA, que **nenhum número deste plano tinha medido** (todos eram de
UMA pilha sobre UMA forma).

---

## A medição veio antes do desenho, e derrubou a minha hipótese

⚠️ Eu suspeitei do `VelloPass::ensure_size`, que compara por **igualdade** enquanto o irmão dele
(`FxStackPass::ensure_work`) **cresce e guarda** — então uma cena de formas de tamanhos diferentes
realocaria a textura a cada forma. **Medido, uniformes e variadas custam o mesmo** (6,16 contra 6,40
a N=32). A realocação não é o custo.

**A causa é o custo FIXO de um render do Vello** — ~**0,12 ms** antes de desenhar coisa alguma, e
independente do conteúdo. A MESMA área de arte: **32 renders = 3,82 ms · 1 render = 0,39 ms**.

---

## O desenho, e a porta única que já estava escrita

O comentário do `run` já afirmava: *"o INGEST é o que põe a fonte no espaço de trabalho, então
depois dele **nenhum op volta a ler `src`**"*. Logo há exactamente **um** sítio a deslocar.

| Pergunta | Porta |
|---|---|
| *onde, na fonte, começa a célula desta forma?* | `Globals.src_org` — lido **só** pelo `cs_ingest` |
| *como a pilha lê uma célula?* | `FxStackPass::run_from` (o `run` delega com `[0,0]`) |
| *onde cabe cada célula?* | `fx_atlas::pack` — pura, sem `wgpu`, determinista |

Tudo a jusante do ingest (work textures, segmentos de silhueta, origem do ruído, `dst`) continua a
falar em coordenadas **locais da forma** — a tradução é feita **uma vez, na fronteira**.

---

## Os três defeitos que os gates acharam, todos meus

1. ⚠️ **O `tap_img` limita-se a `g.dims`** (o tamanho da FORMA). Correto para as work textures — o
   `ensure_work` cresce e nunca encolhe, e é esse limite que torna invisíveis os pixels de uma forma
   maior de um frame anterior — e **errado para a fonte**, onde a célula vive em
   `src_org .. src_org + dims`. O gate de paridade nasceu **VERMELHO com 8806 bytes diferentes**.
   Cura: `tap_src`, com limite próprio (`textureDimensions`).
2. ⚠️ **Uma forma maior que o teto espalhava o próprio excesso pelo lote inteiro** — os vizinhos,
   que cabiam, passavam a viver numa textura que o device recusa (`lote 9000x104 > 8192`).
3. ⚠️ **O atlas introduz um modo de falha NOVO e MUDO:** arte que passe da caixa calculada cairia
   **dentro da célula da vizinha** (antes, a borda da textura a descartava). Cada célula é
   **RECORTADA** — não é otimização, é a reposição de um limite que já existia.

---

## Superfície tocada

| | |
|---|---|
| **`PROJECT_SCHEMA`** | **fica em 38** — a wave não toca documento nenhum |
| **Contrato congelado** | **intacto** (`architecture_contract_surface` 3 · `_tool_` 4 · `_vector_` 11) |
| **ADR** | nenhum |
| **`Cargo.toml`** | **zero** — nenhuma dep nova, nenhuma crate nova |
| **`ph2d-render`** | `Globals` +`src_org`/`_pad3` (128 → **144 bytes**, pin atualizado) · `cs_ingest` desloca · `tap_src` novo · `FxStackPass::run_from` (`run` delega) |
| **`shells/desktop`** | **`fx_atlas.rs` NOVO** (o empacotador) · `fx_live::recook` em duas varreduras + `cook_batch`/`ensure_output` (o `recook_one` morreu) · o log de perf reporta o nº de RENDERS |
| **LOC** | `fx_live.rs` 594 · `fx_stack.rs` 697 — os dois sob o teto, sem split |

---

## Gates

| onde | quantos | o que provam |
|---|---|---|
| `ph2d-render/tests/fx_stack_atlas_gpu.rs` (⚠️ `#[ignore]`, precisa de adapter) | 2 | a célula filtra **byte a byte** como a forma sozinha (**0 de 27648**) · `run` == `run_from([0,0])` |
| `shells/desktop/src/fx_atlas_tests.rs` | 6 | células disjuntas · cena típica num render só · lotes em vez de formas perdidas · determinismo |
| `shells/desktop/tests/the_atlas_clips_every_cell.rs` | 2 | cada célula é recortada · o lote é UM render |

**8 mutações, 8 sangram.**

| # | mutação | resultado |
|---|---|---|
| M1 | o ingest ignora o `src_org` | RED |
| M2 | o `tap_src` volta a limitar-se a `g.dims` (o defeito original) | RED |
| M3 | o slot do ingest não leva a origem | RED |
| M4 | o empacotador devolve um lote por forma | RED |
| M5 | a forma gigante volta a espalhar-se pelo lote dos vizinhos | RED |
| M6 | a textura do lote vira o TETO em vez do conteúdo | RED (3 gates) |
| M7 | a shell não recorta a célula | RED |
| M8 | o render volta para dentro do laço | RED |

---

## Smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector
env PH2D_BUILD_SMOKE=33 PH2D_FX_PERF=1 cargo run -p ph2d-host-desktop --release
```

A cena **`=33`** monta **16 estrelas filtradas** (Blur · Outer Glow · Drop Shadow). Duas coisas a
conferir, e a segunda é a que importa:

1. **A linha de perf tem de dizer `em 1 render(es)`** — `[fx-perf] 16 pilha(s), 16 re-cozida(s) em
   1 render(es), recook … ms`.
2. **O desenho tem de ficar IGUAL ao de antes.** Esta wave não muda um pixel; se alguma estrela
   ganhou arte que não é dela (um pedaço da vizinha), o recorte da célula falhou — e é o único
   sintoma que a paridade de GPU não consegue ver, porque ela mede uma célula por vez.

As outras cenas de FX (`=34` a `=38`) servem de regressão pelo mesmo critério: **igual ao de antes**.

---

## Aberto, com o preço medido

- ⛔ **O segundo eixo (uma submissão para as `n` pilhas) foi CONSTRUÍDO e REVERTIDO — não refaça.**
  A estimativa de ~1,0 ms saiu de uma sonda com o degrau **mais barato que existe**, onde encode e
  submissão *são* a amostra; numa pilha real o fixo sobrepõe-se a trabalho de GPU e deixa de ser
  aditivo. Medido pelas duas rotas, três corridas: a submissão única é **0,92–0,95× a N=16 e
  0,75–0,86× a N=32** — **mais lenta**, e pior quanto maior o lote (as work textures são
  partilhadas, então um encoder só faz o wgpu **serializar** o que `n` submissões deixam o driver
  pipelinar). Detalhe e tabela no plano 24 §17.6. **Zero código sobreviveu**, de propósito: uma
  porta pública que ninguém deve chamar é pior que porta nenhuma.
- **O empacotador é de prateleiras** (desperdiça alguma área contra um exacto). Área desperdiçada
  custa **preenchimento**, que é a metade barata; o que a wave comprou foi o número de RENDERS.
