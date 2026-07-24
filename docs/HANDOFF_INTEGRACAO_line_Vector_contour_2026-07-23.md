# Handoff de integração — `line/Vector` (Contour + rotação do Pattern), 2026-07-23

**11 commits**, base `df91ef6ec`. Duas entregas independentes: a **rotação do Pattern on Path**
(smoke aprovado pelo Enio) e o **CONTOUR** (pesquisa `20_*` item #9, **pendente de smoke**).

---

## §1 — O que entra

### A. Pattern on Path: orientação do motivo (smoke OK)

`VecPatternRotation(f32)` gira cada cópia **dentro do referencial da guia** — `90` põe o motivo de
pé, atravessado na curva; é constante em todas as cópias e relativa à TANGENTE, não ao mundo.
Row bipolar `−180..180` na seção. Piso do **Spacing** baixado `0.25 → 0.01` (pedido do Enio).

⚠️ **Componente PRÓPRIO, não campo do `VecPatternPath`**: o blob é postcard POSICIONAL, então
apender bumparia o `PROJECT_SCHEMA` — e um bump **RECUSA todo projeto já salvo**. É o critério que
a `line/physics` fixou depois de o pagar (W-Offset bumpou; `AreaDrag`/`AreaBuoyancy`/`AreaTorque`
reverteram, cada uma com o mesmo racional).

⚠️ **O piso do Spacing invalidou uma nota:** `MAX_COPIES` agora morde quando
`guia > 40,96 × largura do motivo` e **trunca em silêncio** — a tabela medida está no doc-comment
de `pattern_path.rs` (400/0,25 → 40 cópias/0,08 ms … 4000/0,01 → **4096 capado**/7,53 ms).

### B. Contour — N anéis concêntricos com rampa de cor (pendente de smoke)

O efeito que a Corel publica como sem equivalente no Illustrator. Componente
`ph2d_ecs::VecContour{steps, d, join, side, to, accel}` + cozimento `contour_live.rs` (shell) +
seção completa no painel + `Expand Contour`.

**Duas decisões de arquitetura, ambas saídas de fatos do código:**

1. **Não cabe na pilha de efeitos (ADR-0132).** `PathEffect::apply` é `VecPath → **UM** VecPath`, e
   um contour precisa de N caminhos com tintas DIFERENTES. Não é limitação da pilha: é a pilha a
   dizer que este efeito não é um dos seus. Daí componente + `LiveGeometry`, como o `VecOffset` de
   que é a generalização.
2. **A cor viaja como `[u8; 4]`** porque `ph2d-ecs` não depende de `ph2d-color` e `ph2d-vec-scene`
   o **recusa explicitamente** no Cargo.toml. Quem interpola é a shell, em **Oklab** — em sRGB cru
   o meio da rampa fica lamacento, e o módulo já mostra um picker OKLCH.

**O `d` é POR PASSO (modelo Corel), e a razão é MEDIDA.** A alternativa (`d` como alcance TOTAL,
com o anel externo parado quando a contagem muda) parecia melhor de usar e foi recusada: com
alcance total, mexer nos passos **move todos os anéis** ⇒ o slider mais comum do efeito re-cozinha
N offsets por frame. Por passo, acrescentar um anel custa **UM** offset e o memo reusa o prefixo.

**`MAX_CONTOUR_STEPS = 16`, medido** (custo LINEAR nos anéis; o caso que morde é estrela+Round a
11,24 ms — degradação honesta, em vez de capar todos em 8 por causa da forma mais cara).

---

## §2 — ⚠️ O ACHADO MAIS IMPORTANTE: o offset derrubava o app

Fui medir os números da cena de smoke (política: *a sonda headless roda ANTES de a mensagem ser
escrita*) e **a sonda panicou**. `linesweeper` 0.3.0 tem um `unwrap()` numa curva degenerada
(`curve/mod.rs:302`) e **aborta** em vez de devolver `None`. Varredura de 200 distâncias:

| forma | Miter | Round | Bevel |
|---|---|---|---|
| retângulo | 0/200 | 0/200 | 0/200 |
| hexágono | 8/200 | 26/200 | 26/200 |
| estrela 5 pontas | 0/200 | 83/200 | **118/200** |

Numa estrela com quina Bevel, **59% das distâncias derrubam o app** — e o slider de Offset da
**seção Expand** as alcança arrastando. **É defeito PRÉ-EXISTENTE**, vivo na `main` hoje; nada
nesta linha o introduziu. O Contour apenas o torna certo (N anéis varrem N distâncias de uma vez).

**Cura no nosso lado:** isolar o pânico na fronteira do `offset_path` e devolver o vazio que a doc
dele **já prometia** desde que existe (*"devolve vazio se o sweep falhar"* — a frase valia só para
o `None` do `Region::of`). É o padrão do `ph2d-imageio-avif` com um decoder hostil. **A cura de
verdade é do `linesweeper` e não é desta linha** — fica nomeada aqui.

**Consequência visível:** onde o sweep não responde, o anel sai VAZIO (falta um anel) em vez de
matar o processo. A cena de smoke usa `accel = 1`, onde saem 6 de 6, e a mensagem avisa.

---

## §3 — Números que a integração precisa conferir

| contador | valor | mudou? |
|---|---|---|
| `PROJECT_SCHEMA` | **29** | **NÃO** (`VecContour`/`VecPatternRotation` cunham blob-key própria) |
| `VEC_SCENE_SCHEMA_VERSION` | **13** | NÃO |
| registro `ph2d-ecs` | **37** | 35 → 37 (+2) |
| espelhos `ph2d-render` / `ph2d-script` | **38** | 36 → 38 (+2, **os TRÊS contadores**) |
| `VECTOR_SECTIONS` | **26** | 23 → 26 (+3, ver §4) |

⚠️ **O contador de componentes é TRÊS, não um** — `ph2d-render` e `ph2d-script` afirmam a mesma
contagem e cada um só roda na suíte da própria crate; os dois já ficaram vermelho-latentes em
integrações anteriores. Confira os três.

⚠️ **Números que SOMAM entre linhas se CONTAM, não se escolhem.** Se outra linha registrar
componentes na mesma janela, o valor certo não está em nenhum dos dois lados do conflito.

---

## §4 — ⚠️ Dívida da `main` que esta linha achou e corrigiu

**`VECTOR_SECTION_TEXTPATH` e `VECTOR_SECTION_PATTERNPATH` nunca entraram em `VECTOR_SECTIONS`.**
Os dois chegaram à `main` na integração de 2026-07-23 pintando chevron e **não dobrando**: o
`dispatch` consulta `is_collapsible_section` antes de disparar o toggle, então esquecer a entrada
**não dá erro em lado nenhum**.

O gate que existia (`seam.rs::every_section_header_is_registered_as_collapsible`) percorria a
**LISTA** e provava que tudo nela está marcado — a metade errada, e por isso ficou verde sobre o
bug. Gate novo (`section_headers_are_collapsible.rs`) varre as chamadas de `section_header` no
FONTE e cobra a correspondência, com controle positivo. Mutação: tirar TEXTPATH da lista → RED.

⚠️ `VECTOR_SECTIONS` é **lista partilhada append-only** — foi fundida contra a `main` de hoje; na
integração, só ACRESCENTE.

---

## §5 — Foundational tocado (tudo aditivo, projetado para isolamento)

- **NOVO** `ph2d-editor-core/src/ids/chrome/vector_contour.rs` — os 17 ids da seção, bloco
  append-only (irmão do `vector_patternpath`).
- **NOVO** `ph2d-editor-core/src/ids/chrome/vector_sections.rs` — `VECTOR_SECTIONS` saiu do
  `vector.rs` (704 > 700 LOC). O corte é por responsabilidade: os irmãos declaram IDs, este
  declara uma POLÍTICA sobre eles.
- `ph2d-i18n` — uma chave (`panel.vector.section.contour`).
- `ph2d-vec-boolean::offset_path` — o `catch_unwind` da §2 (+ `offset_path_inner`).
- `ph2d-editor-core/tests/architecture_panel_wiring_parity.rs` — `VECTOR_CONTOUR_TO` na allowlist
  das picker-swatches (a 3ª do mesmo painel, pela mesma porta que Stroke/Fill).
- `ph2d-panel-vector` — 3 splits por LOC cap: `state_expand.rs` (os knobs panel-local do Offset
  Path), `event_contour.rs`, `contour_params.rs`.

**Nenhum contrato congelado tocado** (conferido por grep + os gates de superfície, não por
auto-relato): `NodeOp`/`OpResolver`/`NodeManifest`, `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` e o data-model vetorial estão intactos.

---

## §6 — Smokes

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector
env PH2D_BUILD_SMOKE=25 cargo run -p ph2d-host-desktop --release   # CONTOUR (pendente)
env PH2D_BUILD_SMOKE=24 cargo run -p ph2d-host-desktop --release   # Pattern (aprovado)
```

A cena `=25` tem **duas metades**, e a razão é a cicatriz do `impasto_smoke` (*"o smoke que arma
estado sob a mesa pula justamente o seam que ele existia para provar"*): uma **estrela pelada e
selecionada** (é ela que prova o seam, do `Add Contour` ao Expand, com a mão do artista) e um
**hexágono já armado** (para o olho ter a que comparar).

---

## §7 — Aberto, nomeado

- **A cura de verdade do `linesweeper`** (§2) — anel que falta em certas distâncias. Precisa de
  upgrade/patch do dep, que é decisão com ADR.
- **Multi-seleção com contours diferentes**: os sliders são um número só e escrevem em todos os
  selecionados (mesmo desenho do Offset vivo).
- **Contour + pilha de efeitos na mesma forma**: a resposta é o `LiveGeometry` alimentar um 2º
  estágio, **não** mexer no contrato da pilha (a mesma nota que o Offset já carrega).
- O `Expand` não re-seleciona o resultado (o Blend re-seleciona); a fonte continua selecionada,
  que é defensável — decisão de produto que o smoke pode reverter.

---

## §8 — Gate de fechamento (rodado)

- `cargo test` das 5 crates tocadas: **103 suítes ok, 0 falhas**
- `scripts/nextest-impacted.sh`: **5728 testes, 5728 passaram**
- `cargo clippy --workspace --all-targets`: **0**
- `cargo machete`: limpo · `typos`: limpo · `cargo fmt` (pin 1.95): limpo
- `file_loc_caps` da **shell** (que `cargo test -p` não alcança): ok
- Gates novos: 3 seam + 1 arch-gate de seção + 1 pin cross-crate + 1 de robustez do offset +
  2 unit do mapa. **Mutações: 11, todas sangram.**

⚠️ **A suíte da SHELL foi rodada inteira** (`cargo test -p ph2d-host-desktop`), não só por crate —
é a 3ª ocorrência da lição, e foi ela que pegou o `arch_safe_clamp_only` e o `file_loc_caps`.
