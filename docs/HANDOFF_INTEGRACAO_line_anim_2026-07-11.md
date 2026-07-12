# HANDOFF DE INTEGRAÇÃO — linha `line/anim` (Timeline W4 cauda + W5)

> **Para:** o **agente integrador** (DIRETRIZ §1.5.3–1.5.4). **De:** a linha `line/anim`.
> **Data:** 2026-07-11 · **Regime:** Modo L (workstation) · **Ordem do Enio:** integrar ao `main`.
> **Status da linha:** FECHADA, gate de fechamento VERDE, **NÃO integrada, NÃO shipada**.
>
> Detalhe técnico por feature (o "porquê" de cada decisão, provas, gotchas) está em
> [`HANDOFF_line_anim_integracao_2026-07-11.md`](HANDOFF_line_anim_integracao_2026-07-11.md) §1–§17.
> **Este documento é o que o integrador precisa** — identidade, colisões, gates, ordem.

---

## 1. Identidade da linha

| | |
|---|---|
| **Branch** | `line/anim` |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim/` |
| **Base do fork** | `1c7c9a22` |
| **`main` HEAD agora** | `1c7c9a22` — **IDÊNTICO à base** |
| **Relação** | **FAST-FORWARD PURO.** `main` não andou desde o fork: `git log main ^line/anim` = vazio. Sem rebase, sem conflito, sem drift. |
| **Commits à frente** | **33** (17 já existiam no handoff anterior + 16 desta jornada) |
| **Árvore** | limpa (`git status` vazio) |

**Integração esperada:** `bash scripts/foundational-integrate.sh` de dentro do worktree → `--ff-only` trivial.
Se `main` tiver andado entre este handoff e a integração (outra linha entrou antes), veja **§6 — conflitos previsíveis**.

---

## 2. O que a linha entrega (resumo executivo)

Cauda da W4 + a **W5 inteira de autoria**. Todas as features abaixo foram **smokadas e aprovadas pelo Enio**:

| # | Feature | Commits-chave |
|---|---|---|
| §7 | Auto-key inerte durante o Play (bug: 1 key por quadro) | `bd412e01` |
| §8 | **Speed graph** (vista de velocidade + handles editáveis) | `e78c0394`, `60289e0c` |
| §9 | 4 bugs de UX do smoke (pin de pose, heal de bindings, easing multi-coluna, seleção individual) | `f2d8608f` |
| §10 | **Weighted tangents** (`Interp::BezierW`, segmento flat curva) + speed handles AE | `1b68db52`, `4fa4cbeb` |
| §11 | **Time remap** (relógio por objeto, modelo AE) | `9a6ba0ee` |
| §13 | **Fix definitivo do Time remap** (seed do K = `remapped_time`) | `72803d18` |
| §12 | **Roving keys** (rove across time) | `5b8b6e7f` |
| §14 | **Delete Track** por R-click na label | `10cb7771` |
| §15 | Menu fecha no clique fora + diff/pin do autokey no relógio do apply | `2de2f062` |
| §16 | **Performing / Record** (gravar durante o play) | `26a77af4` |
| §17 | **Simplificação de keyframes do record** (fit de curva) | `b09f7003`, `e2df085d`, `7ee4c1fa`, `267ae2a2` |

---

## 3. Foundational tocado (o que exige atenção do integrador)

A linha **tocou foundational** (permitido em Modo L, ADR-0107). Superfície, por crate:

| Crate | Natureza | Risco de colisão |
|---|---|---|
| **`ph2d-anim`** (foundational) | `Interp::BezierW` (variant **apendado por ÚLTIMO**) · `Interp::slope/value/value_slope` · módulos NOVOS `curve_weighted.rs`, `curve_fit.rs`, `rove.rs` · `Track::{roving,resolve_roving,simplify_range,simplify_range_at,range_samples}` · `TrackData.roving` (campo apendado) | **BAIXO** — variantes/campos APENDADOS, módulos irmãos novos. Só colide se outra linha também mexeu em `Interp`/`Track`. |
| **`ph2d-timeline`** (foundational) | `PropKind::TimeRemap = 6` (apendado, FORA do `ALL`) · `TimelineFlags.performing` · `TimelineIntent::{SetPerforming,SetRove,SetSelectedRove,Unbind(já existia)}` · `TimelineViewSnapshot.performing` · módulos novos `speed.rs`, `persist.rs` · `apply_from_doc_except` virou predicado · **`DOC_VERSION` 1→2** | **MÉDIO** — ver §4 (schema). |
| **`ph2d-editor-core`** (foundational) | ids novos (§4) · `TimelineHitKind::Row` · `ContextMenuKind::TimelineTrack` · tabela `TIMELINE_TRACK_MENU` · `pointer_down.rs` (bloco de dismissal de menu) · `pre_populate`/`context_menu_overlay` (append) | **MÉDIO** — `pointer_down.rs` é arquivo quente; ver §6. |
| **`ph2d-ui-testkit`** | `MockPanelHost::set_toggle_on` (método NOVO, append-only) | **NULO** |
| **`ph2d-i18n`** | 3 chaves novas (§4) | **NULO** (arms append-only) |
| **`ph2d-panel-timeline`** | crate da linha — 18 arquivos | **NULO** (ninguém mais mexe) |
| **`shells/desktop`** | 15 arquivos (bridge, autokey, presets, persist) | **MÉDIO** — shell é compartilhado; ver §6. |

**Nenhuma dep externa nova.** Único `Cargo.toml` alterado: `ph2d-panel-timeline` ganhou **dev-dep** `ph2d-core` (Playhead nos seam tests) — path dep interna, sem impacto em machete/deny/audit.

---

## 4. Símbolos NOVOS — grep de colisão de mesmo-símbolo

Se outra linha introduziu um símbolo de mesmo nome, é **colisão de mesmo-símbolo** (DIRETRIZ §1.5.5) → **PARE e reporte ao Enio**.

**ids (`ph2d-editor-core::ids`)** — todos por `hash_node_id`, colisão detectada pelo gate `node_id_collisions` (verde):
`TIMELINE_SPEED` · `TIMELINE_RECORD` · `TIMELINE_ADDPROP_TIME` · `CTX_MENU_TL_ROVE` · `CTX_MENU_TL_DELETE_TRACK` · `TIMELINE_TRACK_MENU` (tabela) · `TIMELINE_SEGMENT_MENU` 6→7 · `timeline_row_id()` (`dynamic_id("timeline.row")`)

**i18n (`ph2d-i18n`)**: `panel.timeline.speed` · `panel.timeline.record` · `panel.timeline.prop.time`

**Tipos/variants:**
`Interp::BezierW` · `PropKind::TimeRemap = 6` · `TimelineHitKind::Row` · `ContextMenuKind::TimelineTrack` · `TimelineIntent::{SetPerforming,SetRove,SetSelectedRove}` · `TimelineFlags.performing` · `TimelineViewSnapshot.performing` · `KeyView.roving` · `Preset::Rove` · `ph2d_anim::{FitKey, fit_fcurve, fit_fcurve_at, smooth_values, RangeSamples}`

### ⚠️ `DOC_VERSION` 1→2 (o único ponto de schema)

`crates/ph2d-timeline/src/doc.rs`: **`DOC_VERSION: u32 = 1` → `2`** (roving flags = vec paralelo persistente, postcard posicional; v1 é **rejeitado** pelo gate de versão, não deslido).

- **Se NENHUMA outra linha bumpou `DOC_VERSION`:** integra como está, nada a fazer.
- **Se outra linha TAMBÉM bumpou:** é colisão de mesmo-símbolo → **PARE e reporte ao Enio** (não renumere sozinho; a ordem dos campos postcard importa).
- Nenhum writer de produção do doc da timeline existia antes desta linha (o save é sidecar novo), então **não há save de usuário a migrar**.

**Contratos CONGELADOS (§6 do CLAUDE.md): NENHUM tocado.** `Tool` / `RasterEditTool` / `CanvasPaintTool` / `PanelEvent` / `NodeOp` / `OpResolver` / `NodeManifest` / vector-doc **intactos** — verificado por diff (zero linhas). **Nenhum ADR necessário.**

---

## 5. Gate de fechamento — o que EU rodei (verde)

Rodado no worktree, sobre o diff acumulado das 33 commits:

| Gate | Resultado |
|---|---|
| `rustup run 1.95 cargo fmt --all -- --check` | ✅ limpo (pin canônico) |
| `cargo nextest run` nas **7 crates tocadas** (`ph2d-anim`, `ph2d-timeline`, `ph2d-panel-timeline`, `ph2d-editor-core`, `ph2d-ui-testkit`, `ph2d-i18n`, `ph2d-host-desktop`) | ✅ **1377/1377** |
| `cargo clippy --all-targets -- -D warnings` (mesmas 7) | ✅ 0 erros |
| `architecture_workspace_file_loc_cap` | ✅ (maior arquivo novo: `track.rs` 614 ≤ 700) |
| `file_loc_caps` (shell, cap 600) | ✅ (autokey_pass 380; testes divididos em 3 arquivos) |
| `architecture_panel_loc_cap` · `architecture_panel_wiring_parity` · `node_id_collisions` | ✅ |
| `no_tofu_glyphs` | ✅ (corrigido: uma seta `→` num assert — commit `f2c69022`) |
| dhat `apply_from_doc_is_zero_alloc_steady_state` | ✅ (dentro do nextest) |

---

## 6. O que SÓ o `ship.sh` pega — e os conflitos previsíveis

> [[project_integrator_ship_catches_latents_budget_iterations]]: o gate per-linha **NÃO** roda fmt-all / clippy-all-workspace / machete / deny / audit / typos. **Orce 2–4 iterações de ship.**

- **`cargo fmt --all`**: rodei o `--check` do workspace inteiro e está limpo. Risco ~nulo.
- **machete / deny / audit**: **zero dep externa nova** → risco ~nulo. (A dev-dep `ph2d-core` no painel é path-dep interna e É usada.) O advisory-db pode ter RUSTSEC novo desde o fork — o ship reconfirma.
- **typos**: prosa em pt-BR nos docs + strings de UI em inglês. Risco baixo, mas é um gate que só o ship roda.
- **clippy `--workspace`**: rodei nas 7 crates tocadas. Uma crate NÃO tocada que *consome* `ph2d-anim`/`ph2d-timeline` poderia acusar (ex.: `ph2d-eval-motion`, `ph2d-panel-motion-*` consomem `ph2d-anim`). **Se a árvore combinada acusar, é aqui.** Nenhuma API foi REMOVIDA (só apendada), então o risco é de lint, não de quebra.

### Conflitos previsíveis se `main` andou (outra linha entrou antes)

| Arquivo | Por quê | Como resolver |
|---|---|---|
| `Cargo.lock` | toda linha mexe | **regenerar** (`cargo check`), não resolver à mão |
| `CLAUDE.md` §5 | edição ADITIVA na entrada **Timeline** | Mergiraf/manual: manter AMBAS as entradas (a minha é um bloco contíguo na entrada Timeline) |
| `crates/ph2d-editor-core/src/ids/menus.rs` e `ids/chrome/timeline.rs` | consts append-only | manter ambos os blocos; o gate `node_id_collisions` prova que não colidem |
| `crates/ph2d-i18n/src/lib.rs` | arms append-only | manter ambos |
| `crates/ph2d-editor-core/src/interaction/dispatch/pointer_down.rs` | **arquivo quente** — adicionei um bloco de *dismissal* de menu no TOPO do `dispatch_down` | Se outra linha mexeu aqui: o meu bloco deve ficar **antes** das capturas de graph/timeline surface (é o ponto do fix §15). Se conflitar com lógica de OUTRA superfície, é decisão de design → **reporte** |
| `crates/ph2d-editor-core/src/interaction/types.rs` | variants apendados em 2 enums | manter ambos os variants |
| `shells/desktop/src/render_loop/mod.rs` | drena intents + chama o autokey pass | append; se outra linha mexeu na mesma região do render loop, resolver por ordem de passes |

---

## 7. Ordem de integração

**Uma única linha, FF puro — não há ordem interna a respeitar.** Os 33 commits são sequenciais e cada um compila (cada feature fechou com gate). Integre o branch inteiro:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim
bash scripts/foundational-integrate.sh        # --ff-only + gate da árvore combinada
```

Se o script pedir rebase (main andou), veja §6. **Não faça squash** — os commits contam a história das 3 iterações do Time remap e das 3 do fit (valor forense; a memória do projeto referencia SHAs).

---

## 8. Estado pós-integração (o que o integrador reporta ao Enio)

- **`main` verde local** = `./scripts/ship.sh` verde (fmt, clippy-all, machete, deny, audit, nextest, typos).
- **Ship/push:** só por **ordem EXPLÍCITA do Enio** (§0.7). O integrador **PARA** no main verde.

### Smokes JÁ APROVADOS pelo Enio (não precisam repetir)
Time remap · Delete Track · Performing/Record · Roving keys · simplificação do record (fit + colunas alinhadas).

### Aberto (documentado, NÃO bloqueia a integração)
- **W4.T4/T7** — docar timeline no `motion_timeline_slot` · relógio único `MotionTransport`←`Playhead` (coordenam com a linha **Motion**).
- **W4.T6/B5** — save unificado cena+timeline (deferido, cross-cutting).
- **Refinamentos do fit** (§17, deferidos por escolha): corner pre-pass (cusps = tangentes BROKEN) · overshoot clamp p/ canais limitados · rotation unwrap p/ spins multi-volta.
- **W5 restante**: NLA / multi-clip UI · markers→signals · MCP/Luau · bake · export.
- `vec_history` morto (limpeza de OUTRA linha).

---

## 9. Se algo quebrar na árvore combinada

1. **Compilação** de uma crate NÃO tocada que consome `ph2d-anim`/`ph2d-timeline`: nenhuma API foi removida (só apendada) — provável que seja um `match` não-exaustivo sobre `Interp` ou `PropKind` numa crate consumidora. **Adicione o arm** (`BezierW` / `TimeRemap`) seguindo o padrão do arm vizinho; se a semântica não for óbvia, **reporte**.
2. **`DOC_VERSION` duplo-bump**: §4 → **PARE e reporte**.
3. **Contrato congelado**: não deveria acontecer (zero linhas de diff) — se acontecer, **PARE, exige ADR**.
