# HANDOFF — implementação do Deform (transform/deformação do Painter)

> **Ponto de entrada único** para o agente que vai assumir a implementação numa **nova máquina
> desktop Linux**. Escrito 2026-07-04. Design 100% fechado em [`docs/Deform/`](.).
> Leia este arquivo inteiro primeiro.

---

## STATUS (atualizado 2026-07-04)

- **Wave 1 (Reshape brush) — LANDOU + validada no smoke do Enio.** Push/Twist/Pinch/Wrinkle/Fold/
  Reconstruct + Freeze + painel mode-exclusivo. Kernel inverse-warp single-resample (`warp/apply.rs`),
  campos por-modo (`warp/field.rs`, HR-5 sem transcendentais), `disp` de sessão no `ModelSnapshot`.
  ADR-0105 subiu o cap de LOC de arquivo 600→700 no caminho.
- **Wave 2A+2B (Transform gizmo Uniform/Free) — LANDOU (commit local, sem push).** Toggle
  **Reshape/Transform** no topo do painel troca o corpo; Transform mostra picker **Uniform/Free** + um
  gizmo de bounding-box (8 quadrados de escala + anel de rotação + mover-centro) no canvas. Kernel:
  frame pristina `F0` + frame arrastada `F` → afim `M = A1∘A0⁻¹` (`warp/transform.rs`
  `affine_from_frames`), escrita como `D(p)=p−M⁻¹·p` no mesmo `disp`; `F==F0 ⇒ M=I ⇒ byte-idêntico`.
  Gizmo **tool-side** (`on_canvas_pointer`, espelha `selection_gizmo.rs`) — zero foundational, zero
  contrato congelado. Frame no `ModelSnapshot` → undo rola caixa+pixels juntos. Overlay no shell:
  `painter_bridge_deform_gizmo.rs`.
- **PENDENTE — Wave 2C (Distort/homografia 3×3 dos 4 cantos) + Wave 2D (Warp mesh Coons).**
- **PENDENTE — perf 4K:** cada Move do gizmo re-resampleia o canvas inteiro na CPU; medir vs. o
  budget ≤16ms e migrar p/ GPU se estourar (kill-criterion do plano).

---

## 0. TL;DR (o que fazer)
1. Ler, nesta ordem: [`00_README.md`](00_README.md) → [`04_implementation_plan.md`](04_implementation_plan.md) → [`02_design_and_architecture.md`](02_design_and_architecture.md) → [`03_ui_ux_panel_spec.md`](03_ui_ux_panel_spec.md) → [`01_research.md`](01_research.md).
2. Rodar o sanity check (DIRETRIZ §0) na máquina Linux — **rebaseline de ambiente** (§4 aqui).
3. Conferir se o **sistema de Selection já landou** (§3) — ele é pré-requisito da Wave 1.
4. Começar pela **Wave 1** ([`04` §W1](04_implementation_plan.md)): 1A (botão rail, isolado) → 1B (kernel no painter) → 1C (painel inspector). Sites file:line exatos estão no plano.

**Contrato congelado tocado:** **NÃO** (Wave 1). `PaintMode`/`BrushSettings`/`PanelEvent` genérico + `CanvasPaintTool` cap=1 reusado. Sem ADR para W1.

---

## 1. O que é (contexto de 30 segundos)
Ferramenta **Deform** no Painter = **Transform + Liquify + Puppet** unificados sob **um kernel
de inverse-warp** (`out[dst]=sample(dst−D(dst))`, backward-gather). Só o campo `D` muda entre
modos → zero redundância. Botão novo na **rail esquerda do Painter, acima do Mask**; UI mora no
**inspector direito** (`ph2d-panel-painter-layers`), com cards, responsivo a tablet. Objetivo:
**superior ao Procreate** (freeze mask, não-destrutivo, puppet/MLS, warps-como-nós). Detalhe e
os 7 diferenciais em [`02`](02_design_and_architecture.md) §4.

---

## 2. Estado exato do repositório (no momento do handoff)
- **Branch:** `main`, sincronizada com `origin/main`.
- **Último commit:** `79ef0a46 fix(painter): Stroke multi-shape smoke fixes`.
- **Código do Deform:** **nenhuma linha escrita.** Só os docs desta pasta.
- **`docs/Deform/` está UNTRACKED** (não commitado) — ver §5 (transferência para o Linux).
- **WIP de OUTRO agente (NÃO TOCAR, não é seu):** no momento estavam modificados
  `crates/ph2d-panel-painter-layers/src/event.rs`, `crates/ph2d-tool-painter/src/tool/paint/selection_gizmo.rs`,
  `.../paint/stroke_multi.rs`, e vários `shells/desktop/src/render_loop/painter_bridge_*overlay*.rs`/`_gizmo.rs`/`_op_badges.rs`.
  Isso é o trabalho de **Selection + Stroke multi-shape** em vôo. **Na máquina nova esse estado
  pode estar (a) commitado, (b) ainda WIP no drive, ou (c) perdido se não migrou.** Reavalie com
  `git status` + `git log` antes de começar (§3).

---

## 3. ⚠️ Pré-requisito / posse: Selection precisa ter landado
A **Wave 1B/1C do Deform edita os MESMOS arquivos** que o sistema de Selection (ADR-0103):
`canvas_pointer.rs` (ladder de roteamento), `paint_mode.rs`, `brush_settings.rs`/`snapshot.rs`,
`paint.rs`, e todo o `ph2d-panel-painter-layers`. Além disso a Wave 1 **reusa** a infra de
Selection para o **Freeze/Protect** (`selection_mask`, `selection_coverage_at`,
`restore_deselected_region` em `selection.rs`).

**Antes de iniciar W1B/W1C, confirme que a Selection está no HEAD:**
```bash
git log --oneline -20 | grep -i -E "selection|mask"      # a Selection landou?
git grep -n "PaintMode::Selection" crates/ph2d-tool-painter/src/tool/paint/paint_mode.rs
git grep -n "fn selection_coverage_at" crates/ph2d-tool-painter/src/tool/paint/selection.rs
```
- Se **presentes e commitados** → posse liberada, siga a Wave 1 normalmente, herdando a infra de Freeze.
- Se **ainda WIP / ausente** → **PARE**: ou espere landar, ou comece só pela **W1A (rail)**, que é
  isolada em `ph2d-editor-core` e não colide com a Selection. Nunca edite arquivo com WIP alheio
  no working tree (memória: `feedback_parallel_agent_collision`).

*(Nota: numa máquina Linux solo, provavelmente não haverá agentes paralelos — mas o WIP pode ter
vindo no drive sem commit. A regra vale igual: `git status` limpo dos arquivos que você vai tocar,
ou eles commitados, antes de editar.)*

---

## 4. Migração macOS → Linux — rebaseline de ambiente (LEIA)
A stack de velocidade (DIRETRIZ §6.6, SKILL_Stack) foi **afinada para o Mac de 8 GiB**. Vários
itens **mudam no Linux** — não copie as configs de Mac cegamente:

| Item | No Mac (antigo) | No Linux (novo) — ação |
|---|---|---|
| **Linker** | lld/ld-prime via `~/.cargo/config.toml` global (`-fuse-ld=/opt/homebrew/bin/ld64.lld`); `mold` **incompatível** | `mold` **É compatível e recomendado** no Linux (ELF). Ajuste o `config.toml` do usuário na máquina nova; o path Homebrew do Mac não existe. **Remova/ajuste** o `-fuse-ld` que aponta pra `/opt/homebrew`. |
| **RAM / concorrência** | 8 GiB → ≤3 cargos simultâneos, rust-analyzer full BLOQUEADO | **Re-meça a RAM do desktop.** Se ≥16-32 GiB, o teto de ≤3 e o bloqueio de rust-analyzer/LSP podem cair — reavalie DIRETRIZ §6.6.B antes de assumir os limites do Mac. |
| **Slots CoW** | APFS clone (`scripts/slot-seed.sh`) | APFS é macOS-only. No Linux o clone-on-write depende do FS (Btrfs/XFS reflink). Reveja `scripts/slot-seed.sh`/`slot-env.sh` — pode precisar de fallback sem reflink. |
| **ISPC asset-cooker** | crasha com cargo concorrente (`feedback_ispc_cross_process`) | idem provável; um cargo por vez no cook. |
| **Path do projeto** | `/Volumes/MAC_EXTERNO/...` (drive externo) | Será outro path no Linux. Nada no plano depende do path absoluto — os file:line são repo-relativos. |
| **CI** | matrix linux+macOS+windows (inalterado) | inalterado; `./scripts/ship.sh` continua a paridade. |

**Sanity check obrigatório na máquina nova (DIRETRIZ §0):**
```bash
git log --oneline -5
git status -sb
cargo check --workspace 2>&1 | tail -5      # baseline compila no Linux?
```
Divergência (build quebrado, toolchain diferente) → resolver **antes** de codar Deform.

---

## 5. Transferência dos docs para o Linux (não perca este plano)
`docs/Deform/` (incl. este handoff) está **untracked**. Para garantir que chega na máquina Linux,
uma das opções:
- **(recomendado) commit escopado + push** — isolado, zero colisão com o WIP alheio:
  ```bash
  git status                                   # confirme que nada alheio está staged
  git add -- docs/Deform/
  git commit -m "docs(deform): design + implementation plan + handoff"
  git push origin main                         # (ou deixe o Coordenador pushar)
  ```
  Isso é seguro: `docs/Deform/` são arquivos novos, disjuntos do WIP de Selection/Stroke.
- **ou** levar o **drive externo** físico junto (o repo inteiro, incl. untracked, vai junto).
- **ou** copiar a pasta `docs/Deform/` manualmente para a máquina nova.

> O Enio pediu **"não commit"** enquanto havia agentes em vôo. Se ainda houver, use o drive/cópia
> manual, OU commite **só** `docs/Deform/` quando o índice estiver livre. O plano em si não muda.

---

## 6. Primeiros passos concretos (quando desbloqueado)
Do [`04_implementation_plan.md`](04_implementation_plan.md), Wave 1, na ordem:

1. **W1A — botão da rail** (`ph2d-editor-core`, Coord-only, **pode começar já**, isolado):
   ids em `ids/chrome/rail_painter.rs`, tupla em `left_rail.rs` `PAINTER_TOOLS` (antes do Mask,
   reusa `IconId::Transform`), string `"deform"` em `rail_painter_tools.rs::push_paint_mode`.
   Gate: `cargo test -p ph2d-editor-core`.
2. **W1B — kernel no painter** (após Selection): `PaintMode::Deform`, módulo novo
   `tool/paint/warp/{mod,field,apply,reconstruct}.rs` (cada ≤600 LOC), branch no
   `canvas_pointer.rs`, undo estrutural, snapshot `is_deform`.
3. **W1C — painel** (após Selection): `paint_deform.rs` mode-exclusivo, cards responsivos
   (variantes `*_adaptive`), populate/event/seam test.

**Regra-mãe (DIRETIVA):** verde-de-compilação **não** é pronto. DoD da W1 = **seam test
comportamental verde** (`ph2d-ui-testkit` dirige o evento real → efeito observável) + **paridade
numérica** do kernel (campo identidade ⇒ byte-idêntico) + **smoke do Enio** (incl. largura estreita
p/ o reflow). Kill-criterion de perf (dab ≤8 ms @4K; two-strikes → GPU) fixado em [`04` §1E](04_implementation_plan.md).

---

## 7. Disciplina de trabalho (invariantes do projeto)
- **Isolamento:** edite só a sua pasta/módulo; precisou de foundational/shell/contrato → PARE e reporte ao Coordenador.
- **UI canônica:** zero hex, zero f32-de-UI, tudo tokens/i18n (HR-15); labels em inglês.
- **Git anti-colisão:** `git add -- <seus paths>`; nunca `-A`/`git add .`/`git stash`; `git status` antes de stage.
- **Inner loop:** só `cargo check -p <crate>`; teste/clippy/auditoria 1× no fechamento do módulo.
- **Referência publicada antes de inventar** (DIRETIVA §1): MLS = Schaefer 2006; inverse-warp = backward-gather; HR-5 sin/cos gated.
- **Você não pusha** (implementador) — reporta commit local ao Coordenador (CLAUDE.md §3). No modo solo Linux, o Enio dispara o ship.

---

## 8. Índice de referência rápida (onde está o quê)
- Plano/waves/sites file:line → [`04_implementation_plan.md`](04_implementation_plan.md)
- Kernel/IA/superioridade → [`02_design_and_architecture.md`](02_design_and_architecture.md)
- Painel/cards/widgets/responsividade/mockup → [`03_ui_ux_panel_spec.md`](03_ui_ux_panel_spec.md)
- Pesquisa Procreate + referências → [`01_research.md`](01_research.md)
- Roteador do projeto / inegociáveis → [`CLAUDE.md`](../../CLAUDE.md) · processo → [`DIRETRIZ.md`](../IntegracaoMultiAgente/DIRETRIZ.md) · por-etapa → [`DIRETIVA_IMPLEMENTACAO.md`](../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md)
- Memória/estado → [`MEMORY.md`](../../../.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md) · posse → [`SESSION_ACTIVE.md`](../SESSION_ACTIVE.md)
