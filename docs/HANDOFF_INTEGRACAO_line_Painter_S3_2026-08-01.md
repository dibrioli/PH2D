# HANDOFF DE INTEGRAÇÃO — `line/Painter` · 2026-08-01

> **Para o agente INTEGRADOR.** A linha está **FECHADA**. Ela **não integrou e não pushou** — isso é
> ordem explícita do Enio (CLAUDE.md §0.7). Este documento é o que a DIRETRIZ §1.5.9 pede.

## §1 — O que a linha traz

**31 commits**, 47 arquivos, `+5200 / −412`. Quatro crates: `ph2d-tool-painter` ·
`ph2d-wet-paint` · `ph2d-painter-brush` · `shells/desktop`.

Três frentes, em ordem cronológica:

### (A) A água — o fim da frente de CPU (docs 28 §5.55–§5.57 · ADR-0146 Emenda 5)

- O **item 3 do ADR-0146 (a GPU do solver) FECHOU POR MEDIÇÃO**: os dois gatilhos *mensuráveis* dele
  morreram. O K–M custa **1,1-1,4×** (a nota dizia 4,75×) e a razão 1 a 4096² custa **21,18 ms/quadro
  contra o nominal de 25** — *a concessão que o ADR existia para remover não existe mais*. Só sobrevive
  o gatilho hipotético.
- O **upsample** do composite ganhou a fatoração *row-invariant* (`SampleU::row`, a irmã exata do
  `FlowRowSampler`): **1,24× no amostrador, 1,16-1,17× no composite**, **bit-idêntica** contra a rota
  antiga congelada sob `cfg(test)`, com a razão 1 servindo de controle interno.
- ⛔ **O `away 24%` do worker foi construído, medido e REVERTIDO**: devolver o motor na fronteira do
  composite compra **+0,6 Hz (2%) ao preço de +50% no PIOR TICK**. As duas metades já saturam os 32
  núcleos, então sobrepô-las não cria capacidade — *um balde de ESPERA só é oportunidade quando o
  recurso que ele espera está PARADO*. Ficou o split instrumentado (custo zero) e a sonda.
- ⚠️ **Um FLAKE cross-line da `ph2d-painter-brush` fechou junto** (reportado pela `line/Vector`): o
  contador da LUT era global com uma trava cuja doc **enumerava** quem devia segurá-la — 13 sítios
  depositam, 2 seguravam. Reproduzido 1 em 8 sob carga. Contadores por **thread** tornam a poluição
  estruturalmente impossível e a trava morreu com os 2 holders.

### (B) O quadro — o instrumento (doc 28 §5.48–§5.50)

- O `[frame]` publica o **divisor** das entregas de carimbo (`stamps` admitia *um re-stamp de 105 ms*
  ou *cinquenta entregas de 2 ms* — curas opostas), a **poça em M células** e o **`ns/célula`**.
- O `stall` era uma **subtração com nome de medição**; a bbox de upload é `(x,y,w,h)` e eu a li como
  cantos e publiquei o número errado — os dois corrigidos, com o mecanismo escrito.

### (C) O S3 — o journal vira a fonte do `before` do RELEVO (doc 28 §5.58–§5.61)

O degrau 2 (a rota), o 3 (a base) e o **4 (a elisão)**. É a frente com o ganho de produto:

**pen-up 22,22 → 5,57 ms a 4096² (3,99×)** e **10,54 → 3,94 a 2048² (2,68×)**, pen-down intocado,
donos dos três planos de relevo **3 → 1**. ⚠️ **Só as duas elisões juntas entregam.**

Detalhe completo, com a tabela e as três correções de desenho: **[doc 28 §5.61](Painter/28_otimizacoes_o_que_funcionou.md)**.

## §2 — ⚠️ O que o integrador precisa saber ANTES de fundir

### `PROJECT_SCHEMA`, contrato congelado, deps

| | |
|---|---|
| `PROJECT_SCHEMA` | **48, INTOCADO** (conferido por `git diff`, não por auto-relato) |
| Contrato congelado | **4/4 verde** — `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` |
| `Cargo.toml` | **ZERO tocados** — nenhuma dep nova, nenhuma crate nova |
| ids / tokens / i18n | **nenhum novo** |
| ADR | **nenhum novo**; o **0146 ganhou a Emenda 5** (o arquivo é tocado, o número não é reivindicado) |

⇒ **Esta linha fica FORA de toda disputa de número desta janela.**

### O ponto de merge sensível

**`crates/ph2d-tool-painter/src/undo*.rs`** — a família inteira foi reorganizada. Dois arquivos NOVOS
(`undo_elide.rs`, `undo_shape_state.rs`) e uma superfície interna que mudou de forma:

- `ModelSnapshot` ganhou o campo `relief_elided` ⇒ **os três literais** do struct (o `snapshot_model` e
  dois de fixture) precisam dele. Uma linha que tenha criado um quarto literal **não compila** — e é
  bom que não compile.
- `absorb_foreign_writes` mantém a assinatura de um argumento.
- `UndoController` ganhou `base_for_top` (a porta da base do relevo) e, sob `cfg(test)`, `installed`
  e os dois flags de ablação.

Se o merge textual sair limpo, **rode a suíte da crate nos dois perfis antes de acreditar** — a família
de defeitos que este repo já pagou é a árvore que funde limpa e não compila
([[feedback_clean_text_merge_can_be_semantically_broken]]).

### ⚠️ O journal do RELEVO agora roda em RELEASE

Era `cfg(any(test, debug_assertions))`. **Ele e a elisão subiram JUNTOS** (§5.58.1) — promover um sem o
outro é regressão pura. O journal do **CANVAS continua em debug** (o `before` do canvas não é elidido,
capturar dele seria custo sem contrapartida).

**Consequência de comportamento:** existe um caminho novo em que a **história de undo é DESCARTADA** —
quando o `before` elidiu o relevo e o journal não descreve o passo. Ele **nunca disparou na suíte**
depois das duas capturas (o eraser e o reset do warp), e quando disparar imprime a causa
(`misturado · incompleto · camada errada · plano trocado`) com um `debug_assert` ao lado.

## §3 — Gate de fechamento (rodado, não auto-relatado)

| gate | resultado |
|---|---|
| `cargo test -p ph2d-tool-painter` (debug) | **950 / 0** |
| `cargo test -p ph2d-tool-painter --release` | **952 / 0** |
| `cargo clippy -p ph2d-tool-painter --release --all-targets` | **0 warnings** |
| `cargo test -p ph2d-wet-paint --release --test fingerprint` | **3 / 3** (ADR-0134 intacto) |
| `architecture_workspace_file_loc_cap` (isolado) | **2 / 2** |
| `shells/desktop/tests/file_loc_caps.rs` (isolado) | **2 / 2** |
| `architecture_tool_contract_surface` | **4 / 4** |

⚠️ Os dois de LOC foram rodados **isolados de propósito**: eles não correm num `cargo test -p` filtrado,
e esta linha já shipou dívida vermelho-latente por isso.

## §4 — Smokes

Todos com `--release`.

| smoke | o que julgar |
|---|---|
| `PH2D_IMPASTO_SMOKE=2 PH2D_PAINT_PERF=1` | **o S3 — ✅ APROVADO pelo Enio em 2026-08-01** (*"smoke OK em Undo/Redo do impasto"*). A tinta e o RELEVO voltam iguais. ⚠️ O log trouxe um outlier de **71,2 ms** que foi **atribuído por medição e NÃO é desta wave** — ver §5 e o [doc 28 §5.62](Painter/28_otimizacoes_o_que_funcionou.md). |
| `PH2D_WETPAINT_SMOKE=1 PH2D_FLUID_PROFILE=1` | a água, canvas **4096**, pincel grande, faixas **SOBREPOSTAS**. No log: `busy + away + sleep ≈ 100%`, `TAXA DA AGUA` perto de 40 Hz (nunca acima) com `sleep > 0`, e a linha `poca:`. |
| `PH2D_MASK_SMOKE=1` | a proteção, se o integrador quiser o controle da wave anterior. |

⚠️ **A regra do §5.49 vale para todos:** nenhum smoke desta máquina significa nada com o
`load average` acima de ~5 — a linha `poca:` é o detector (*um dígito de `ns/célula` = máquina sã; três
dígitos = o log não fala sobre o código*).

## §5 — Aberto (nomeado, com o número; não é dívida escondida)

- ⚠️ **O QUADRO DEPOIS DE UM CTRL+Z é plane-bound: 97,7 ms a 2048² e 381,3 a 4096² no produtor de CPU**
  (3,90× para 4× de área), contra **0,000 ms** de um tick ocioso — o controle que o torna um achado. É
  ele que tem a forma do outlier de 71,2 ms do smoke (`preview` inteiro, `branch=idle`). **PRÉ-EXISTENTE
  e exonerado por ablação**: o braço sem elisão paga os mesmos 381 ms, em duas corridas de sinais
  opostos, com o controle de donos (2 → 1) provando que a ablação era real. A cura nomeada é publicar **a
  janela que o passo reescreveu** — e o S3 é justamente a wave que a tornou explícita —, mas isso muda o
  que a tela repinta ⇒ **wave própria, com smoke próprio**. Números e mecanismo:
  [doc 28 §5.62](Painter/28_otimizacoes_o_que_funcionou.md).
- **O pen-down segue sendo uma cópia de canvas** (§5.16 o pinou; 5,65 ms a 4096²). Não era o alvo desta
  wave e não se moveu. Quem o fecha é a captura do "antes" por **REGIÃO** (o *tile-based undo*), e ela
  quer a **porta única de escrita de canvas**.
- **Semear os planos da luz no `prewarm`** — vale **12,7 ms** medidos no 1º traço com impasto, ao preço
  de VRAM canvas-sized em TODO bind, inclusive de quem nunca liga o impasto. **Decisão de produto.**
- O **carimbo de dab** custa 1,86 / 3,34 / 4,37 ms por entrega nos raios 100 / 200 / 300, sub-linear no
  raio — decompor o depósito de um dab é wave própria, e ela tem alvo (§5.50).
