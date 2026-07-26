# HANDOFF DE INTEGRAÇÃO — `line/Painter`: **o histórico de undo guarda a JANELA** (U1)

> Para o **agente integrador**, munido pelo Enio. DIRETRIZ §1.5.9.
> O handoff de **tarefa** que abriu esta wave é
> [`HANDOFF_line_Painter_undo_delta_2026-07-26.md`](HANDOFF_line_Painter_undo_delta_2026-07-26.md);
> o dos 31 commits anteriores é
> [`HANDOFF_INTEGRACAO_line_Painter_impasto_fold_2026-07-25.md`](HANDOFF_INTEGRACAO_line_Painter_impasto_fold_2026-07-25.md).

## 1. Branch / HEAD / base

| | |
|---|---|
| branch | `line/Painter` (worktree `Worktrees/line-Painter/`) |
| HEAD | `9a5b13fbe` |
| base | `main` @ `0afc6bb28` — **já é ancestral**, `git rebase main` é no-op |
| commits à frente | **37** (os 31 já descritos no handoff de 25/07 + **6** desta wave) |
| árvore | limpa |

Os 6 commits desta wave:

```
9a5b13fbe docs(claude): a 5 ganha a entrada da U1 -- com os dois negativos
a28677737 docs(plano 26): a 7.5 registra o que a U1 mediu -- incluindo os DOIS negativos
fa5cce3c8 docs(hr-13): o Painter declara budget de historico -- e tem o gate que MEDE
5c259ead3 perf(painter): o orcamento do historico e funcao do DOCUMENTO, e a medicao derrubou a constante
27de1267a perf(painter): o pen-down NAO era o mesmo defeito, e a receita da cura estava 5% certa
1f35c05a2 perf(painter): o historico de undo guarda a JANELA, nao o documento (U1)
```

## 2. O que a wave faz, numa tabela

| | antes | depois |
|---|---|---|
| retido, 24 traços a 2048² com impasto | **1.627,2 MB** | **242,2 MB** |
| **por passo** | **~67,8 MB** (mais que um documento) | **2,36 MB** (3,7% de um) |
| pico | 1.669,2 MB | 345,8 MB |
| cap | `DEFAULT_MAX_DEPTH = 300` (**contagem**) | `2 × documento + 256 MB` (**bytes**) |
| profundidade útil (1024/2048/4096²) | 9 · 3 · 1 passos | **114 · 46 · 26** |
| custo de um undo | `Arc::clone` (~0) | **0,43 ms @2048² · 13,37 @4096²** |

## 3. Foundational tocado — **NENHUM**

Tudo dentro de `crates/ph2d-tool-painter/`, mais **duas linhas de doc** em arquivos compartilhados:

- `SKILL_Stack_PH2D_Definitiva.md` — **uma linha nova** na tabela do HR-13 (append: a linha do Painter,
  logo antes de `Physics state`) + o `Enforced by:` do HR-13 ganhou os dois gates novos.
- `CLAUDE.md` §5 — **um parágrafo apendado** ao FIM do bloco do Painter.

⚠️ Os dois são listas compartilhadas: **só ADIÇÃO**, nada removido nem reordenado. Se der conflito, é
textual e o `theirs`/`ours` pode ser fundido linha a linha.

## 4. Ids / consts / variants novos (para o integrador detectar colisão)

**Nenhum id de widget, nenhum token, nenhuma chave i18n.** Só API de crate:

| símbolo | o que é |
|---|---|
| `undo::DEFAULT_MAX_BYTES` | ⚠️ **substitui** `DEFAULT_MAX_DEPTH`, que foi REMOVIDO |
| `undo::MAX_HISTORY_STEPS` | guarda de sanidade sobre a contagem (1000); **não é o cap** |
| `undo::history_budget_bytes(w, h)` | o orçamento em função do documento |
| `UndoController::{set_max_bytes, retained_bytes}` | novos |
| `PainterTool::undo_retained_bytes()` | a porta pela qual o HR-13 é observado |
| `ModelSnapshot.canvas_size: (u32, u32)` | **campo novo** (dá o stride ao motor de delta) |

Módulos novos (privados): `undo_delta.rs` (o motor), `undo_planes.rs` (a lista dos 19 planos),
`undo_tests.rs` (os gates, `#[path]`).

## 5. Contratos congelados — **INTACTOS**

`Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` — conferidos por **grep e pelo
gate**, não por auto-relato (`architecture_tool_contract_surface`, 4 passed).

**`PROJECT_SCHEMA` fica em 29.** O histórico de undo **não é serializado** — nada desta wave viaja no
arquivo. Se você vir um bump de schema neste diff, é sinal de que o delta vazou para a persistência:
**pare e reporte**.

## 6. O que rodou aqui, e o que só o `ship.sh` pega

Rodado nesta árvore, tudo verde:

| gate | resultado |
|---|---|
| `cargo test -p ph2d-tool-painter --release` | **849 passed, 0 failed** |
| `cargo test -p ph2d-host-desktop --release` | **1162 passed, 0 failed** (0 ocorrências de FAILED no log) |
| `cargo check --workspace` | limpo |
| `cargo clippy -p ph2d-tool-painter --all-targets` | **0 warnings** |
| `cargo fmt --all --check` | limpo |
| `cargo machete` | limpo |
| `typos` (project-wide) | limpo |
| `architecture_tool_contract_surface` · `architecture_workspace_file_loc_cap` · `arch_safe_clamp_only` · `architecture_docs_reference_live_gates` · `architecture_panel_wiring_parity` | 4 · 2 · 2 · 1 · 1 passed |
| `architecture_panel_loc_cap` (editor-core) · `file_loc_caps` (shell) | 3 · 2 passed |

**Só o `ship.sh` pega:** `cargo deny` / `cargo audit` (nenhuma dep nova nesta wave — o `Cargo.lock` não
foi tocado) e a matriz 3-OS do CI.

⚠️ **Flake conhecida, PRÉ-EXISTENTE, não desta wave:**
`the_brush_snapshot_costs_the_same_on_a_canvas_sixteen_times_bigger`
(`ph2d-tool-painter`, `measure_window_premise`) é um gate de RAZÃO sobre números **sub-microssegundo**
(0,0008 vs 0,0003 ms) e é sensível a carga: falhou **1 vez em 4** rodadas da suíte completa e passa
isolado e nas 3 rodadas seguintes. Mesma família da flake `the_cost_of_depth_is_linear_not_explosive` da
timeline. **Re-rode sozinho antes de suspeitar do merge.**

## 7. LOC

Três splits, todos **por responsabilidade**, nenhum por tamanho:

| arquivo | LOC | o corte |
|---|---|---|
| `undo_delta.rs` | 582 | o MOTOR: janela, diff, fallback |
| `undo_planes.rs` | 189 | a LISTA dos 19 planos e seus strides |
| `undo.rs` | 602 | o controller |
| `undo_tests.rs` | 528 | os gates (mod filho por `#[path]` ⇒ alcança privados) |

## 8. O que smoke-testar (o julgamento é do Enio)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && cargo build --release -p ph2d-host-desktop
```

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && env PH2D_IMPASTO_SMOKE=2 PH2D_PAINT_PERF=1 ./target/release/ph2d-host-desktop
```

**O que julgar, e é UMA coisa:** **o undo tem de continuar byte-idêntico.** Pinte vários traços com
impasto, desfaça todos até o começo, refaça todos até o fim — a tinta **e o relevo** voltam iguais, sem
resíduo e sem crista órfã. Depois intercale: pinte, desfaça, pinte de novo, desfaça duas vezes. É o
segundo undo em diante que exercita o cursor (o primeiro está sempre certo).

Vale repetir com **shape editors** (Line/Curve: criar, editar ponto, Offset, Apply & Keep, undo) e com
**Sculpt** — os dois têm caminhos de re-carimbo que o delta atravessa.

⚠️ **O que NÃO vai melhorar neste smoke, e é esperado:** *"o primeiro traço tem um delay"*. Ele não é
esta wave (§7.5.2 do plano 26): medido, **16,49 → 16,07 ms** a 4096². A cura é a captura por região, que
é wave própria — o número está pinado num gate para o dia em que ela chegar.

## 9. Aberto, nomeado

- **A latência do pen-down** (§7.5.2) — a receita corrigida: captura do "antes" por **REGIÃO sob
  demanda**, com uma **porta única de escrita de canvas** contra os ~25 sítios que hoje chamam
  `Arc::make_mut(&mut self.canvas_rgba)` direto. ⚠️ A metade *"reuso da alocação"* da receita antiga foi
  **medida e vale 5%** — não a persiga.
- **O delta guarda os DOIS lados da janela.** Guardar só o lado que não está vivo (a *mesma troca* do
  ADR-0117) cortaria ~40% do retido, ao custo de mutar a entrada no undo/redo. Não feito: a entrada
  auto-contida e imutável é o que torna o coalescing e o cap simples, e a memória já está dentro da
  barra. Se algum dia apertar, é aqui.
- **`the_brush_snapshot_*`** — a flake pré-existente do §6 mede sub-microssegundos; se incomodar, o
  conserto é dar-lhe carga suficiente para sair do ruído (não é desta linha).
