# HANDOFF → Coordenador — Painter W5 Brush Studio (ship me)

**De:** Implementador Painter · **Para:** Coordenador (ship + push + babysit CI)
**Status:** ✅ completo e verde localmente · **NÃO commitado** (aguarda teu ship 1×/jornada)

---

## §0 — TL;DR

Fechei o **Brush Studio** (painel `ph2d-panel-brush-studio`, fecha a W5) **+** implementei/consertei os
parâmetros de brush que estavam mortos ou faltando. Tudo no meu escopo (painéis + `ph2d-painter-brush`
+ `ph2d-tool-painter` + `painter_bridge`); toquei **editor-core** só com **adições** (ids, scrollbar,
scroll-mapping) — reportadas em §4. Árvore limpa de WIP alheio (o P4 API-key já foi commitado por ti).

Gate local (slot-1, `cargo test/check/clippy -p`): **tudo verde** — ver §3.

---

## §1 — O que shippa (agrupado por commit lógico)

Sugiro **um commit** (ou separar painel × motor se preferires granularidade):

### Painel novo `ph2d-panel-brush-studio` (drop-crate, ADR-0029)
- 6 seções colapsáveis, espelhando inspector (seções+scroll) + widget-gallery (widgets canônicos):
  **Stroke Path** · **Shape** · **Rendering** · **Color Dynamics** · **Dynamics**.
- Snapshot próprio `BrushStudioSnapshot` (uncapped) — separado do `PainterUiSnapshot` (cap 18 intacto).
- Edits via **1 variante capeada** `PainterUiEdit::SetBrushParam(BrushParam, f32)` (`BrushParam` é
  uncapped) → mantém `PainterUiEdit` em **21/24** (gate `architecture_painter_contract_surface` verde).
- Aberto pelo botão **"Brush Studio"** no sidebar; X fecha; ocupa o mesmo slot do right-dock
  (3º estado ao lado de sidebar/layers). Bridge: visibilidade + z-bump + publish do snapshot.
- Registrado via `ph2d-panel-sync` (regenerou registry-init markers + Cargo) + `EXPECTED_TYPED`
  hand-count atualizado + feature `panel-brush-studio` no `shells/desktop/Cargo.toml` (default).

### Motor (`ph2d-painter-brush`) — features T1.7 implementadas (eram no-op/ausentes)
- **Input smoothing:** `streamline` (lazy-mouse EMA) + `stabilization` (média móvel, ring fixo
  zero-alloc) no `stamp_scheduler`. Determinístico, no-op em 0, reset nos boundaries.
- **Falloff:** taper de opacidade por distância (modelo depleção-de-tinta **Procreate-faithful** —
  `falloff` controla a TAXA, sempre chega a zero; corrigido após 2 iterações + pesquisa do Handbook).
  Aplicado em `push_one_stamp` **e** no caminho **wash** (`apply_one_stamp_wash` ignorava `stamp.opacity`
  — era o bug "falloff não funciona").
- **Dynamics:** `jitter_size` + `jitter_opacity` por-stamp (det_random axes `0xD1/0xD2`).
- **Shape Scatter (bug duplo consertado):** era (a) lido como GRAUS enquanto o painel manda 0..1
  (→ ~1°, morto) e (b) só rotação (invisível em pincel redondo). Agora é **posicional** (espalha o dab
  ao redor do path — Procreate Handbook) + rotação, escala 0..1. Faz `shape_count`/`count_jitter`
  finalmente visíveis. Axes `0xD3/0xD4`.
- **Color Dynamics:** já era wirado no scheduler (`apply_stamp_color_jitter`), só faltava UI — exposto.
  **+ wash colour accumulation** (`wash_color` buffer no tool + `cpu_render`): a wash misturava só a cor
  do ÚLTIMO dab contra o backdrop → dabs jitterados viravam discos discretos ("queda de resolução" do
  Enio). Agora acumula a média ponderada-por-cobertura das cores → dabs parciais sobrepostos se fundem.
  Byte-idêntico p/ pincel de cor única (usa a cor exata do stamp quando coincide). Pigment: re-prepara
  só quando a cor variou (fast-path preservado).
- **Wet/Burnt Edges: REMOVIDOS** (ambas as tentativas — per-dab e coverage-feather — produziram um
  contorno duro artificial de baixa-res, foto do Enio). Dormentes. **Reimplementação correta (física
  de transporte de pigmento) é mandato do novo implementador** →
  [`HANDOFF_painter_brush_overhaul_impl.md`](HANDOFF_painter_brush_overhaul_impl.md).
- **Split de arquivo:** `ph2d-panel-brush-studio/src/paint.rs` passou de 600 LOC (rustfmt expandiu os
  row-builders multi-linha) → dividido em `paint.rs` (orquestrador) + `sections.rs` (seções + helpers).

### Label fix (Stroke Path)
- "Jitter" do painel agora = `jitter_lateral` (jitter **posicional**, = "Jitter" do Procreate, visível);
  o antigo (variação de espaçamento) virou "Spacing Jit". (Resolve "Jitter não funciona".)

---

## §2 — Commit escopado (árvore limpa — pode usar `git add` por path)

```sh
git add -- \
  crates/ph2d-panel-brush-studio \
  crates/ph2d-tool-painter \
  crates/ph2d-panel-painter-sidebar \
  crates/ph2d-panel-registry-init \
  crates/ph2d-painter-brush/src/cpu_render \
  crates/ph2d-painter-brush/src/stamp_scheduler \
  crates/ph2d-editor-core/src/ids/chrome.rs \
  crates/ph2d-editor-core/src/widget/scrollbar.rs \
  crates/ph2d-editor-core/src/widget/mod.rs \
  crates/ph2d-editor-core/src/interaction/dispatch/scroll.rs \
  shells/desktop/Cargo.toml \
  shells/desktop/src/forwarding.rs \
  shells/desktop/src/render_loop/painter_bridge.rs \
  Cargo.lock
git commit --no-verify -m "feat(painter): W5 Brush Studio panel + T1.7 brush dynamics (smoothing/falloff/jitter/scatter/edges)" -- <mesmos paths>
```

> `git status` antes: a árvore deve conter SÓ estes paths (+ docs). Se aparecer `M`/`??` alheio
> nos meus paths, **não comita** — me chama.

---

## §3 — Gate local (slot-1) — tudo verde

| Gate | Resultado |
|---|---|
| `cargo test -p ph2d-painter-brush --lib` | **299** passed |
| `cargo test -p ph2d-tool-painter --lib` | **184** passed (1 ignored pré-existente) |
| `cargo test -p ph2d-panel-brush-studio` | **7** passed |
| `cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface` | **81** passed (PainterUiEdit 21/24) |
| `cargo test -p ph2d-panel-registry-init` | staleness + count verdes |
| `cargo test -p ph2d-editor-core --test architecture_panel_loc_cap` | **2** passed |
| `cargo test -p ph2d-painter-brush --lib` → `det_random_axis_tags_match_registry` | verde (axes 0xD1–0xD4 registrados) |
| `cargo check -p ph2d-host-desktop` | compila |
| `cargo clippy -p {brush, panel, tool, editor-core, host}` | limpo* |

\* único warning é `paint_vector_prompt_dialog` **never used** em `editor-core/.../context_menu_overlay.rs`
— é **WIP alheio do P4** (não meu); só não esquecer no `ship.sh -D warnings`.

**NÃO rodei** `./scripts/ship.sh` completo (paridade CI) nem smoke `./play.command` — é teu passo
(o Enio confere visual). Sugiro smoke ANTES do push (abrir Brush Studio, mexer cada seção).

---

## §4 — Tocou foundational (editor-core) — só adições, sem quebrar contrato

Reportando per CLAUDE.md §0.2. **Nenhum gate de contrato afetado** (são adições puras):
- `ids/chrome.rs`: ~50 const NodeId novos (`PAINTER_STUDIO_*`, `PAINTER_SIDEBAR_BRUSH_STUDIO`). Sem gate de contagem.
- `widget/scrollbar.rs`: `PAINTER_BRUSH_STUDIO_SCROLLBAR_ID = NodeId(830)` + re-export em `widget/mod.rs`.
- `interaction/dispatch/scroll.rs`: 1 arm em `scrollbar_panel_for_id` (studio scrollbar → panel).
- `ph2d-painter-brush`: 4 axes PRNG novos (`0xD1`–`0xD4`) registrados na tabela `det_random` **e** no
  gate `det_random_axis_tags_match_registry` (ambos atualizados juntos).
- **Stamp ABI 96B intacto** — nenhum campo/flag novo (a tentativa de wet/burnt foi revertida).

---

## §5 — Bloqueado (precisa de TI / ADR) + deferidos

1. **Roundness** + **Alpha Floor** — exigem **campo novo no Stamp ABI 96B congelado**
   (`architecture_painter_contract_surface`) = **Coord + ADR**. Plumbing dormente já existe
   no painel/tool (BrushParam/ids/routing) pronto pra religar quando o campo nascer.
2. **Paridade WGSL** do scatter posicional / dynamics / wet-burnt / wash-colour-accumulation: o
   `stamp.wgsl` + o wash GPU NÃO espelham ainda. Caminho **ao vivo é CPU** (handoff §3 gotcha 1), então
   o Enio VÊ o efeito; paridade GPU é follow-up. Sem gate automático CPU↔GPU.
3. **Wet/Burnt edge band é fino em pincéis pequenos** — a silhueta (feather de cobertura) é ~0.15×raio,
   então o rim escala com o tamanho do pincel (visível em médio/grande, sutil em pequeno). Um rim mais
   largo exigiria distância-à-borda (distance transform sobre a máscara) — refino futuro, não bug.

---

## §6 — Aproximações / knobs de ajuste (estética — decisão do Enio)

- **Falloff:** `FALLOFF_LENGTH_DIAMETERS = 8` (em `stamp_scheduler/mod.rs`) — comprimento até zerar
  com falloff=1. Modelo é depleção (distância fixa), NÃO normalizado ao comprimento total (impossível
  ao vivo — não sabemos o fim do stroke).
- **Scatter:** raio posicional = `scatter01 × diameter`; rotação = `scatter01 × ±180°`.

> **Doc desatualizada:** `docs/Painter_projeto/01_brush_engine.md` linha 74 descreve `falloff` como
> "desvanece até o fim" (normalizado ao comprimento). A implementação real (e o Procreate) é
> distância/taxa. Vale corrigir a spec quando puder (não-bloqueante).

---

## §7 — Smoke sugerido (`./play.command`)

Abrir Painter → **Brush Studio** (botão no sidebar). Por seção:
- **Stroke Path:** Spacing alto → vê dabs separados; Jitter → borda encrespada; Falloff 50% → traço
  longo desvanece (mais devagar que 100%); Streamline/Stabilize → traço liso/atrasado.
- **Shape:** Count 8 + Scatter 50% → spray de dabs espalhados; Count Jit varia a densidade.
- **Rendering:** Mode cicla 6; Pigment/Accumulate; Grain. (Wet/Burnt Edges NÃO expostos — ver §5.)
- **Color Dynamics:** Hue/Sat 40% → cada dab cor diferente.
- **Dynamics:** Size/Opacity Jit → textura "dry brush".
- Scroll do painel, colapsar seções, clicar no painel NÃO pinta atrás (oclusão).
