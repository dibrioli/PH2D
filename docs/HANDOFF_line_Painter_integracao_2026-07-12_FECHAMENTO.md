# HANDOFF de INTEGRAÇÃO — `line/Painter` (fechamento, 2026-07-12)

> **Para o agente INTEGRADOR.** A linha está **FECHADA**. Não integrei, não pushei, não rodei `ship.sh`
> — por protocolo (CLAUDE.md §0.7). Este documento é o que você precisa para integrar `line/Painter`
> em `main` **por ordem explícita do Enio**.
>
> Supersede [`HANDOFF_line_Painter_integracao_2026-07-12.md`](HANDOFF_line_Painter_integracao_2026-07-12.md)
> (escrito no meio da jornada; a linha andou **21 commits** depois dele).

---

## 0. TL;DR para o integrador

| | |
|---|---|
| **Branch** | `line/Painter` |
| **Base** | `3805f650` (`main` no início da jornada) |
| **Commits** | **56** à frente de `main` |
| **HEAD** | `4179af71` — *feat(impasto): o RIG — quatro lâmpadas, uma na tela* |
| **Árvore** | limpa |
| **Gates (worktree)** | `cargo test --workspace` → **5684 passed, 0 failed** · clippy `--all-targets` → **0** · LOC caps → verdes |
| **`ship.sh`** | **NÃO rodado** (não é meu — §0.7). Espere 2-4 iterações: o gate per-linha não roda fmt/clippy-all/machete/deny/typos ([[project_integrator_ship_catches_latents_budget_iterations]]) |
| **⚠️ Bug conhecido** | **A UI do rig de luzes está MORTA** (§6). O Enio viu e mandou pra fila de amanhã. **Leia §6 antes de decidir integrar.** |

---

## 1. O que a linha entregou (Impasto, #16 — do zero ao rig)

Ordem cronológica, uma frase cada:

1. **O canal `h` + o passe de luz** — o relevo é o **segundo output da MESMA lista de dabs**, então
   Symmetry / Tiling / Shape / Grain / Jitter / shape-editors sculptam o relevo **de graça**.
2. **A luz é RELATIVA** — tinta plana fica **byte-idêntica** (dividida pela resposta de uma superfície
   plana). Um modelo absoluto escureceria a pintura inteira ao acender a luz.
3. **O relevo é do CAMINHO, não da amostragem** — varredura por **cápsula** até o dab anterior (um disco
   por dab dava traço corrugado, com a ondulação escolhida pelo *spacing*).
4. **Fase 4 — o CORPO:** `body_profile` (platô + parede), slope físico (`DEPTH_UNIT_PX`), *Amount* morto
   (era um 2º ganho sobre o mesmo percepto que Depth).
5. **Todo knob é VIVO** — o traço guarda **ingredientes** (`paint`/`grain`/`push`), e o relevo é
   `derive_height(spec, …)`: Depth/Body/Depth Source/Smoothing/Push re-esculpem o traço **já pintado**.
6. **PLOW** — a espátula: o Smear arrasta o relevo junto com a cor.
7. **Composite Depth por camada** — o relevo vira parâmetro de composição (`Add`/`Level`).
8. **Persistência** — a pintura (camadas + relevo) sobrevive ao Ctrl+S.
9. **Conservação de volume (Push)** — real-time, conservativa (Σ R₁ = 0), viva, idempotente.
10. **O FILME** (Bug #14) — *"a tinta extravasa o relevo"*: um pincel que não deposita corpo não
    deposita tinta, **e a tinta que ele deposita é OPACA** (Beer-Lambert). Névoa **52% → 13,5%**.
11. **O RIG** — 4 lâmpadas (angle/elev/intensity/cor), **uma na tela**; contrato preservado **por canal**.

Perf no fechamento: **3,41 ms/movimento @2048² · 3,66 @4096²** (alvo ≤4, kill 8).

---

## 2. Superfície tocada — para prever conflito de merge

### 2.1 Crates da linha (conflito improvável)

`ph2d-painter-brush` · `ph2d-tool-painter` · `ph2d-panel-painter-layers` · `ph2d-painter-effects`.

**Arquivos NOVOS** (não conflitam por definição): `height_film.rs`, `height_push.rs`,
`impasto_rig.rs`, `impasto.rs`, `impasto_light.rs`, `impasto_settle.rs`, `impasto_plow.rs`,
`impasto_settings.rs`, `relief_state.rs`, `paint_impasto.rs`, `paint_impasto_rig.rs`,
`paint_rows_relief.rs`, `event/impasto_light_picker.rs`, `impasto_smoke.rs`.

### 2.2 ⚠️ FOUNDATIONAL tocado (ADR-0107 — **é aqui que o merge dói**)

| Arquivo | O que mudou | Risco |
|---|---|---|
| `ph2d-editor-core/src/ids/chrome/painter_impasto.rs` | **arquivo novo** + ids do rig | baixo (novo) |
| `ph2d-editor-core/src/ids/chrome/mod.rs` | `pub mod painter_impasto;` | **mesmo-símbolo**: outra linha que adicione um módulo aqui colide textualmente. Mergiraf resolve. |
| `ph2d-editor-core/src/ids/chrome/painter.rs` | ids do Painter | baixo |
| `ph2d-ecs/src/painted_doc.rs` | `PaintedDoc` ganhou o relevo (`heights`/`covers`) | **médio** — ver 2.3 |
| `ph2d-ecs/src/scene/registry.rs` | **+1 componente registrado** | **ALTO** — ver 2.3 |
| `ph2d-render/src/registry.rs` · `ph2d-script/src/registry.rs` | **contagem** de componentes | **ALTO** — ver 2.3 |
| `ph2d-editor-core/tests/architecture_panel_loc_cap.rs` | allowlist | baixo |
| `shells/desktop/src/**` (9 arquivos) | bridge/persistência/smoke | médio |

### 2.3 🔴 A ARMADILHA QUE ME CUSTOU UM "VERDE" FALSO — leia isto

Registrar um componente novo no ECS **muda a CONTAGEM** que dois gates de registry afirmam:

- `crates/ph2d-render/src/registry.rs`
- `crates/ph2d-script/src/registry.rs`

**`nextest-impacted` NÃO os toca.** Eu reportei "gate batched verde" com esses dois **VERMELHOS**, e só
`cargo test --workspace` pegou. **Se outra linha desta jornada também registrou um componente, a
contagem combinada vai estar errada nas duas e o merge fica verde nos dois lados e vermelho na árvore
integrada.** Rode `cargo test --workspace` na árvore COMBINADA — não confie no impacted.

### 2.4 Contratos congelados (§6 do CLAUDE.md)

**Nenhum tocado.** `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` intactos;
nodes e vector-doc não foram encostados. Zero ADR necessário.

---

## 3. Como verificar a árvore integrada

```bash
cd <arvore-combinada>
cargo test --workspace          # ← NÃO use nextest-impacted (§2.3)
cargo clippy --workspace --all-targets --all-features
./scripts/ship.sh               # só o INTEGRADOR, e só por ordem do Enio
```

Perf (opcional, ~1 min, só em `--release`):

```bash
cargo test --release -p ph2d-tool-painter --lib -- impasto_perf_kill_criterion --ignored --nocapture
# esperado: ~3.4 ms/move @2048, ~3.7 @4096 (alvo <=4, kill 8)
```

---

## 4. Os gates que a linha instalou (e que valem como rede pra você)

Todos com **RED provado por mutação** — se um deles ficar vermelho depois do merge, é regressão real,
não flake:

| Gate | Afirma |
|---|---|
| `impasto_light_leaves_flat_paint_byte_identical` | tinta plana não muda **um byte** |
| `a_coloured_light_rig_leaves_flat_paint_byte_identical` | …nem sob 4 lâmpadas saturadas |
| `a_single_lamp_shifts_brightness_never_hue` | uma lâmpada colorida sozinha muda brilho, **nunca matiz** |
| `the_lights_turned_all_the_way_down_is_an_unlit_canvas` | baixar as luzes não **escurece** o quadro |
| `the_glint_only_ever_adds_light` | Shine só **soma** luz |
| `impasto_paint_has_an_edge_not_a_fringe` | ≤18% da tinta é "nem sólida nem ausente" (a névoa do Bug #14) |
| `impasto_lays_no_pigment_where_the_light_lays_no_shading` | todo pixel pigmentado é modelado pela luz |
| `the_film_never_starves_the_brush_at_low_strength` | Strength baixa ainda **pinta** |
| `the_light_models_a_faint_stroke` | Strength baixa ainda **acende** |
| `impasto_perf_kill_criterion` | ≤4 ms/movimento (kill 8) |

---

## 5. Smoke

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

Canvas branco 1024², Impasto já ligado, Grain = None. Clique no canvas → pill **Painter** → arraste.

**Já validado pelo Enio:** o corpo, a luz, o Body dial, o Shine, os knobs vivos, o Plow, o Composite
Depth, a persistência, o filme/opacidade (*"melhorou o extravasamento"*).

---

## 6. 🔴 ABERTO — e o Enio JÁ VIU

### 6.1 A UI do rig de luzes está MORTA (fila: **amanhã**)

**Enio, 2026-07-12 (print anexado à sessão):** *"UI não funciona, nem o checkbox nem se pode selecionar
outra luz. Mas coloque na fila para amanhã."*

Os chips `1 2 3 4` **pintam** (o print mostra `1` selecionado e `2· 3· 4·` apagados) mas **não
respondem ao clique**, e o checkbox também não.

**Causa: NÃO IDENTIFICADA.** Levantei e **descartei** duas hipóteses (a colisão de id entre o `group_id`
do segmented e o id da opção 1 — o widget **ignora** o `group_id`; e a falta de registro em
`populate.rs` — os segmentos de Depth Source funcionam sem ele). **Não vou adivinhar num handoff.**

**A lição, e ela é a de sempre:** eu gatei a **MATEMÁTICA** do rig com 6 gates e 3 mutações vermelhas —
e **zero gates no SEAM da UI**. O `ph2d-ui-testkit` existe exatamente pra isso. Um teste headless que
**clica no chip** e afirma que `rig.selected` mudou teria saído vermelho antes do Enio abrir o app.
Isto é [[feedback_painted_is_not_populated_paint_gate]] e
[[feedback_tool_unit_green_integration_dead]] pela terceira vez.

**Amanhã:** escrever o gate do seam PRIMEIRO (ele nasce vermelho), depois consertar.

**➡️ RECOMENDAÇÃO AO INTEGRADOR:** o rig entra em `main` com **os chips inertes**. É um knob morto
visível. **Duas saídas, e a escolha é do Enio:**
- **(a)** consertar amanhã **antes** de integrar (é UI pura, não toca a matemática nem os gates); **ou**
- **(b)** integrar já — a matemática está correta e gateada, e o rig **default (1 lâmpada) funciona
  exatamente como antes**; só o *acesso* às lâmpadas 2-4 está morto. Nada regride.

Eu recomendo **(a)**: é uma sessão curta e evita `main` com um controle que não faz nada.

### 6.2 A tinta empurrada (Push) — **FIM DA FILA, por ordem**

**Enio:** *"a tinta empurrada ainda não resolveu. Adiar para o final de toda essa implementação. Fim da
fila."*

A mecânica está certa (real-time, conservativa, viva, idempotente — §13 do plano); o **desenho** da
tinta deslocada não convence. **Não diagnosticado. Não mexer.**

### 6.3 Fila restante (`16_impasto_plano_implementacao.md` §17)

1. ~~Múltiplas luzes~~ ✅ (com a UI aberta, §6.1) · 2. **Passe de luz na GPU** · 3. Persistência do `h`
no `ProjectState` · **Relevo do PAPEL: exige ordem NOVA do Enio** (acopla impasto↔aquarela) ·
**último: Push**.

### 6.4 Herdados (não são desta linha)

- **Bug #11** (Per-Layer Color, listras retangulares intermitentes) — dormente, armadilha armada.
- **Bug #13** abertos — nenhum é crash.
- `HANDOFF_per_layer_color_perf_artifacts.md` — perf de camadas-como-brush.

---

## 7. Docs vivos

- [`docs/Painter/16_impasto_plano_implementacao.md`](Painter/16_impasto_plano_implementacao.md) — o
  plano + **§14 (o filme)**, **§15 (a luz por canal)**, **§16 (opacidade ≠ espessura)**, **§17 (a fila
  do Enio)**, **§18 (o rig)**.
- [`docs/Painter/17_impasto_deposito_pesquisa2.md`](Painter/17_impasto_deposito_pesquisa2.md) — a
  pesquisa (Photoshop/ArtRage/Rebelle/Krita) que decidiu o corpo **e o número de luzes**.
- [`docs/Painter/BUGS_painter.md`](Painter/BUGS_painter.md) — **Bug #14** (o extravasamento, 3 rodadas).

---

## 8. As lições que valem além desta linha (já em `project-memory/`)

1. **Um gate VERDE não prova que você mediu a coisa certa.** *"O pigmento existe exatamente onde a luz
   modela"* era verdade, provada, com mutação vermelha — e **cega** pro sintoma. Quando o Enio
   contradiz um gate verde: **RENDERIZE E OLHE**
   ([[feedback_render_and_look_when_a_green_gate_is_contradicted]]).
2. **Um gate que você não sabe derrubar não é um gate.** Escrevi 3 gates para o rig; rodei as
   mutações; **os 3 passaram**. Reescrevi enunciando o que cada peça de fato compra.
3. **Desfaça mutação com `cp`, NUNCA `git checkout`** — o checkout apaga a feature junto e o gate
   "passa" ([[feedback_mutation_undo_with_cp_never_git_checkout]]). Aconteceu **3×** nesta linha.
4. **Um limiar pertence à FORMA; a dinâmica multiplica depois.** Errei isso em dois lados opostos do
   mesmo cano (o corte do pigmento e o peso da luz), e as duas vezes o sintoma foi um **knob morto** em
   pressão parcial.
5. **`nextest-impacted` não vê os gates de contagem de registry** (§2.3).
