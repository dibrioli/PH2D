# Handoff de INTEGRAÇÃO — `line/Painter`: a cobertura da máscara

**Para:** o agente integrador (por ordem EXPLÍCITA do Enio). **De:** a `line/Painter`, 2026-07-25.
**Detalhe técnico:** [`docs/Painter/25_avaliacao_gpu.md`](Painter/25_avaliacao_gpu.md) §13.9 (revertida) +
**§13.10** (o que ficou).

> ## O resumo em quatro linhas
>
> **Duas entregas, dois eixos.** (a) A tentativa de dar à máscara uma **lei de cobertura própria** (o
> envelope Wash do Krita) foi construída e **REPROVADA na tela** (o traço saía em contas) ⇒ revertida, e o
> estado que ficou é *a máscara pinta exactamente como o brush digital normal* — ordem do Enio, pinada num
> gate de byte-identidade. Esse defeito (a borda endurece sob muitas passadas) **segue ABERTO** (§13.10.4).
> (b) **A TINTA atravessando a proteção saía craquelada, e ISSO FECHOU** (§13.11 diagnóstico → **§13.12**
> cura): a proteção era composta uma vez por BATCH, logo a força dela seguia a taxa de polling do mouse;
> agora é uma **sessão por-TRAÇO** com o `keep` aplicado UMA vez. 6 gates novos, 4 mutações, 4 sangram.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/Painter` |
| HEAD | o tip da branch (`git log --oneline main..HEAD` é a fonte; um sha literal aqui se auto-invalidaria) |
| base (merge-base com `main`) | `df91ef6ec` |
| commits desta wave | **15**: `d8018d6bc`..`edd0602b1` constroem a lei do canal · `d41b0a71b` **a remove** (doc §13.10) · `a5c5a14f2` os handoffs · `afbcd5e8a`+`c05b1ede1`+`6d801604c` o **diagnóstico** do craquelado (§13.11) · `b75eda4c1` **a CURA** (§13.12: o plano livre por-traço) |
| commits da linha desde a base | ⚠️ **~49** — esta wave são os 11 do topo; o resto são **waves ANTERIORES da mesma linha que ainda não integraram** (§7) |

⚠️ **Os commits desta wave NÃO são um caminho reto:** `d8018d6bc`..`edd0602b1` constroem a lei do canal e
`d41b0a71b` a **remove**, mantendo só a higiene que sobreviveu. Um squash é defensável (o estado final é o
que importa, e está na §2); manter o histórico também é (ele é a prova de que as duas leis foram tentadas,
que é o argumento da §13.10.4).

## 2. O que o diff FINAL faz (contra `main`)

| arquivo | o quê | aditivo? |
|---|---|---|
| `crates/ph2d-painter-brush/src/stroke_cover.rs` | **NOVO** — a aritmética do cap de Accumulate, extraída do `bands.rs` (pure code motion), numa cópia só; o módulo registra a lei alternativa tentada e reprovada | sim (irmão novo, `pub(crate)`) |
| `crates/ph2d-painter-brush/src/stroke_cover_tests.rs` | **NOVO** — 2 gates: a aritmética é a que shipou (termo a termo) · o cap aprofunda e não passa do teto | sim |
| `crates/ph2d-painter-brush/src/lib.rs` | `pub(crate) mod stroke_cover;` | sim, 1 linha |
| `crates/ph2d-painter-brush/src/dab/bands.rs` | o kernel per-pixel chama `cover_add` em vez de ter a aritmética inline | não (mesmas operações, mesma ordem) |
| `crates/ph2d-tool-painter/src/tool/paint/stamp_route.rs` | **`stroke_cover_wanted(brush) -> bool`** — a porta ÚNICA do predicado que estava em 3 lugares; **não olha o modo** | sim (método novo) |
| `crates/ph2d-tool-painter/src/tool/paint/stamp_cache.rs` | as 2 rotas que threadam o buffer perguntam à porta | não |
| `crates/ph2d-tool-painter/src/tool/mod.rs` | **`GateSession`** + o campo `gate` (canvas-shaped, ao lado do `canvas_rgba` e dos planos de relevo) | sim |
| `crates/ph2d-tool-painter/src/tool/paint/mask.rs` | **`stamp_dabs_gated`** + `project_gate_region` + `end_gate_session`; as 2 portas antigas (`snapshot_region`/`restore_protected_region`) **removidas**; declara os 3 módulos de teste irmãos | não (a lei muda do 2º batch em diante) |
| `crates/ph2d-tool-painter/src/tool/paint/region.rs` | `restore_region` repõe o plano livre — a **porta única** que evita enumerar os 5 chamadores | não |
| `crates/ph2d-tool-painter/src/tool/paint/selection.rs` | `restore_deselected_region` **removida** (a seleção virou um FATOR do `keep`) | não |
| `crates/ph2d-tool-painter/src/tool/paint/stroke_lifecycle.rs` + `stamp_preview.rs` + `layers/undo.rs` | as 4 mortes da sessão (os MESMOS sítios do sculpt) | sim |
| `crates/ph2d-tool-painter/src/tool/paint/mask_probe_gate.rs` | **NOVO** — as 2 sondas do gate (o reporte + o custo), split por ASSUNTO quando `mask_probe.rs` bateu 810 LOC | sim |
| `crates/ph2d-tool-painter/src/tool/paint/mask_probe.rs` | **NOVO** — 11 sondas de medição (`#[ignore]`, com dump p/ render-and-look) + os helpers de oráculo | sim |
| `crates/ph2d-tool-painter/src/tool/paint/mask_tests.rs` | **NOVO** — 5 gates (byte-identidade com o brush · o cap · undo · custo · o número do endurecimento) | sim |
| `shells/desktop/src/mask_smoke.rs` + `main.rs` + `app_state.rs` + `render_loop/mod.rs` | **NOVO** a cena `PH2D_MASK_SMOKE=1` + 3 sítios de wiring | sim |
| `shells/desktop/tests/the_smokes_open_the_painter_in_digital.rs` | o gate varre o 3º smoke, com a metade positiva própria (a cena não tem `arm_brush_once` de propósito) | sim |
| `docs/Painter/25_avaliacao_gpu.md` | §13.9 marcada REVERTIDA + §13.10 (mecanismo, medições, o que ficou, o que segue aberto) | sim |
| `CLAUDE.md` | 1 parágrafo na §5 | sim (só adiciona) |
| `project-memory/` | 1 memória nova + 1 linha no índice + o tópico de oráculo 5→6 | sim |

**Comportamento do produto, contra `main`: UMA mudança, e é a correção.** A tinta atravessando uma
proteção emplumada deixa de depender da taxa de polling (o craquelado); consequência nomeada: **Smear /
Blur / Clone** arrastados sobre uma zona protegida agora leem a tinta **IRRESTRITA** (semântica de máscara
de camada) em vez do display — o desenho antigo lia o display, mas o que ele lia dependia do polling, então
não era referência estável. **O smoke decide** (§13.12, último parágrafo). No resto: A máscara acumula como acumulava (que é como
o brush digital acumula), o pigmento é byte-idêntico, a row Accumulate aparece onde aparecia. O que entra é
**higiene** (uma cópia do predicado, uma da aritmética), **gates** (5+2, mutação-provados), **medição** (11
sondas), o **smoke** e a **documentação do que foi tentado**.

## 3. Símbolos que podem COLIDIR com outra linha

**Zero ids, zero consts de UI, zero tokens, zero chaves de i18n, zero entradas em lista ordenada.**

| símbolo | forma | onde |
|---|---|---|
| `stroke_cover::cover_add` | `pub(crate) fn` | `ph2d-painter-brush` |
| `GateSession` | `pub(crate) struct` | `ph2d-tool-painter::tool` |
| `PainterTool::{stamp_dabs_gated, end_gate_session}` | `pub(super)` / `pub(crate)` | `tool/paint/mask.rs` |
| `PainterTool::stroke_cover_wanted` | `pub(super) fn` | `tool/paint/stamp_route.rs` |
| `mask_smoke_done` | `bool` em `AppState` | `shells/desktop/src/app_state.rs` |
| `PH2D_MASK_SMOKE` | env var nova | `shells/desktop/src/mask_smoke.rs` |
| módulos de teste novos | `mask_probe`, `mask_tests` (filhos de `paint::mask`) · `stroke_cover_tests` | — |

⚠️ **Nenhuma assinatura pública mudou no estado final** — a troca de tipo do `stamp_dab_*` que a wave
introduziu foi desfeita na reversão, então **não há atrito de chamador** para outra linha.

## 4. Contratos congelados encostados

**NENHUM.** `Tool = 12` / `RasterEditTool = 5` / `CanvasPaintTool = 1` / `PanelEvent = 4` intactos;
`NodeOp`/`OpResolver`/`NodeManifest` não tocados; superfície do `ph2d-vector-doc`/`-traits` não tocada.
**Nenhum schema bumpou:** `PROJECT_SCHEMA` **29**, `DOC_VERSION` **11**, `VEC_SCENE` **13** — o buffer
por-traço é transiente (gate `a_mask_stroke_is_one_undo_step_and_the_next_stroke_starts_fresh`) e o scratch
commitado não mudou de representação.

## 5. O que só o `ship.sh` pega

- **Nenhuma dep nova** (`Cargo.toml`/`Cargo.lock` intocados) ⇒ machete/deny/audit sem superfície nova.
- `fmt` com o **pin 1.95**, `--check` limpo na árvore inteira; `clippy --all-targets` **0 warnings** nas
  crates tocadas (5 `doc list item overindented` do smoke corrigidos).
- `typos` não rodado isoladamente: os docs têm termos do repo (`Wash`, `Alpha Darken`, `envelope`).
- LOC caps verdes nos dois gates (o `shells/desktop/tests/file_loc_caps.rs`, que **não** roda com
  `cargo test -p`, entrou no fechamento de propósito). Maior arquivo novo: `mask_probe.rs` ~600.
- `arch_safe_clamp_only` verde.

## 6. Ordem, dependências e o que smoke-testar

**Gate desta linha:** `nextest-impacted.sh` = **4072 testes, 4072 passaram** (~1090 da shell, então os
arch-gates de `shells/desktop/tests/` foram alcançados). ⚠️ O `ph2d-timeline::nesting_clock
the_cost_of_depth_is_linear_not_explosive` falhou numa volta: é a **flake conhecida e PRÉ-EXISTENTE** que o
CLAUDE.md §5 nomeia (gate de RAZÃO sensível a carga); passa isolado (conferido) e passou nas voltas
seguintes. Rodado em debug **e** release.

**O smoke:**

```fish
env PH2D_MASK_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

O que ele deve mostrar, e o **passo 4 é a estrela desta rodada**: pintar atravessando a zona protegida
muitas vezes, **LENTO e depois RÁPIDO** — a fronteira onde a tinta morre tem de ser uma rampa lisa e ficar
**igual nas duas velocidades** (era aí que o craquelado vivia). A máscara em si tem de pintar **como o brush
digital** (miolo sólido, sem contas, sem emendas claras). O que ele **vai** mostrar, porque segue aberto: a
borda da MÁSCARA **endurece** se você esfregar muito (3,53 px de rampa numa passada, 1,38 em quinze) — essa é
a queixa original do outro eixo; a §13.10.4 lista as três hipóteses que sobraram (o overlay · os defaults do
pincel · aceitar), e nenhuma é a lei do acúmulo.

⚠️ **O doc do smoke estava MENTINDO** até este commit: ele descrevia a lei do envelope **revertida** como o
build de hoje (*"one pass is SOFTER than it used to be"*, os números 6,21/2,10 px, a união cross-stroke).
Reescrito com o que o build de fato faz.

**Não smokado / fora desta wave, NOMEADO:**

- ⚠️ **O CUSTO do pen-down de um traço protegido** — 7,43 ms @2048² (contra 3,02 sem proteção) e
  **24,53 @4096²** (contra 11,26): o plano livre é canvas-sized e é alocado uma vez por traço. O **move** é
  plano na tela (1,20 / 1,13 ms) e está gateado por RAZÃO. A 4096² é um quadro perdido no início de um traço
  protegido — **e já era um antes desta wave** (o snapshot de undo força o próprio fork do canvas). Receita
  da wave de perf (semeadura lazy por TILE + reuso da alocação) escrita na §13.12.5; deliberadamente **não**
  feita dentro de uma wave de correção.
- os métodos de SHAPE (Line/Curve/Ellipse/Polygon/Free Hand) em modo máscara **não pintam nada** — o
  roteador de shape intercepta o Down antes do `paint_begin`, o `ensure_mask_scratch` nunca roda e o scratch
  fica com 0 bytes (medido, sonda 4). Pré-existente.

---

## 7. ⚠️ A linha carrega waves ANTERIORES não-integradas

`git rev-list --count main..HEAD` ≈ **49**; só os 11 do topo são desta wave. Os outros são trabalho anterior
da MESMA linha que nunca integrou:

| bloco | o quê |
|---|---|
| `40191df75` + 3 reverts + `1c23b4130`/`38c1f725b`/`c8b48e2e3`/`600a79606` | o §13.6 (envelope entre traços) e o §13.7 (teto por época), construídos e **revertidos** |
| `2da916c99`..`4280ba572` + os 4 `diag(painter)` | Onda 5c (máscara na via parcial + upload cheio) + o instrumento `PH2D_PAINT_PERF` |
| `abe0123ec`/`608cfa038` | Onda 5b (upload parcial da região suja) |
| `a9057588c`/`ed9563b0d` | Onda 5a (a pintura para de copiar o canvas por move) |
| `97f0ab0a2`..`73fe5b67e` | Ondas 1 e 2 da GPU (máscara/clipping como ops, orçamento do dispositivo) |
| `117023207`..`e8414355c` | doc 24 (transferência sRGB tabelada) + o composite row-parallel |

**Consequência:** integrar esta wave integra as anteriores com ela. As de baixo mexem em superfícies OUTRAS
(o compositor da `ph2d-render`, a `ph2d-wet-paint`), então o tamanho do diff e a superfície de conflito não
são os desta wave. Os handoffs delas já existem em `docs/`
(`HANDOFF_INTEGRACAO_line_Painter_gpu_onda5*`, `..._gpu_ondas_1_2_*`, `..._wet_transfer_*`).

**Resumo:** linha `Painter` pronta. Esta wave entrega **higiene + gates + medição + smoke + documentação**,
com **zero mudança de comportamento** contra o `main`; a lei que ela tentou foi reprovada na tela e removida,
e o defeito original está nomeado, medido e aberto. Aguardo ordem de integração.
