═══════════════════════════════════════════════════════════════════
HANDOFF — Sprite Inspector v2 · W2 in progress (Inspector sections + edit pipeline)
Autor: agente solo Coord+Impl · Data: 2026-05-30
Para: o próximo agente (continua a W2 e segue pra W3..W8)
═══════════════════════════════════════════════════════════════════

LEIA PRIMEIRO o mandato + operacional em `docs/HANDOFF_sprite_solo_coord_impl.md`
(§0 padrão-ouro · §1 UI só widget que existe · §6 slot/gates/CI · §7 decisão
autônoma). ESTE doc é o ESTADO + RECEITA + PRÓXIMOS PASSOS específicos da W2.

───────────────────────────────────────────────────────────────────
§1 — ESTADO (tudo LOCAL, NADA PUSHADO; árvore limpa)
───────────────────────────────────────────────────────────────────
W1 fechado + CI verde em `d15fbaa`. **18 commits locais** em cima (todos
com testes verdes); os **2 mais recentes fecham a seção Color & Tint inteira**:

- `b590cdb` **Tint + Self Tint swatches** — par de swatches reusando o
  BlenderColorPicker (OKLCH); helpers `tint_f32_to_u8`/`tint_u8_to_f32`
  (round-trip exato 256 níveis → dispatch converge); dispatch via
  `widget_color`→`sync.rs`. Audit 2-lentes → CRITICAL (swatch não estava
  registrado no store = clique morto, fix `InteractiveState::Plain`) +
  HIGH (guard de troca-de-entidade fora do bloco `Some(sp)`) fechados.
- `ea8e531` **sub-tabs + per-corner gradient grid** — restruturação §3.0
  D11: `[Tint][Self][Corners][Effects]` (~120px vs ~342px flat). Per-corner
  = grade 2×2 + **preview de gradiente bilinear ao vivo** (`corner_bilerp`,
  8×8 células — verificado bit-a-bit contra o `sprite.wgsl` @location(9..12),
  sem swap TL/BL) + **Equalize Corners**. `PerCornerTint` despacha o array
  inteiro com 1 canto trocado. Audit 2-lentes → zero CRITICAL/HIGH.

**Seção Color & Tint (§3.6, a crítica ⭐⭐⭐) está COMPLETA**: Tint · Self Tint ·
Per-corner (4 cantos + gradiente + equalize) · Tint Fill · Opacity — todos
render-ready e auditados. MED aceito: 2 labels de tab apertam ~1px só na
largura mínima 220px (ok no default 304px).

Histórico anterior desta sessão (16 commits) abaixo:

**Skew (foundation + UI + RENDER):**
- `ea5f70c` Transform skew foundational (ADR-0025-amendment-1): `Transform` v2
  (skew_x/y, VERSION 1→2, 28B cap), compose/from_transform R·Sk·S via `libm::tanf`,
  migrator `TransformVersioned`, fixtures + determinism golden. Auditado.
- `90227b5` Skew X/Y editável no Inspector (graus).
- `025bd8c` **fix de render do skew (ADR-0070-amendment-4)** — `RenderInstance.rotation`
  (escalar) → `basis: [f32;4]` (a base 2×2 do mundo). O extract decompunha a matriz
  e jogava o shear fora; agora o shader aplica a base inteira (paralelogramo real).
  ABI 144→156B, field count 12. Picking inverte a base. Gate `sprite_wgsl_valid` (naga).
  Auditado correto + sem regressão (no-skew = R·S, idêntico ao antigo).

**Pipeline de edição de Sprite + seções:**
- `7e256b4` **InspectorSpriteEdit infra (KEYSTONE)** — vide §3 (a receita).
- `9102307` Color & Tint (Opacity + Tint Fill) · `d23e9eb` Opacity virou Slider-com-chip 0..100%.
- `5890469` sprite-sheet sub-UV no extract (render) · `bbea536` Sprite Sheet section
  (HFrames/VFrames/Frame + Flip H/V).
- `da9ac83` **fix dos 7 achados da auditoria** (vide §5).
- `ca7f7ee`, `8f7360a`, `6f537ff`, `382a4c7` docs/handoff.

**Inspector tem 6 live sections:** Name · Visibility · Transform(+skew) · Render
Source · Color & Tint(opacity slider + tint_fill) · Sprite Sheet(hframes/vframes/
frame + flip). Todas com chrome funcional (collapse + color-dot).

**Auditoria da W2: FECHADA A ERRO-ZERO.** 3 lentes (data-flow, UX/a11y/dispatch,
render-readiness) + re-auditoria do fix commit. Render-readiness PASS (cada campo
ligado muda pixels). Gates: editor-core 568 · ph2d-render 23+2 (naga) · ph2d-ecs
skew/determinism · node_id_collisions · clippy zero-warning.

───────────────────────────────────────────────────────────────────
§2 — ⚠️ A ARMADILHA (NÃO LIGUE UI QUE O RENDER NÃO APLICA)
───────────────────────────────────────────────────────────────────
O incidente desta sessão: skew era autorável mas o renderer decompunha a matriz →
"misturou scale+rot". REGRA: só ligue UI de um campo se o `sim_extract` realmente o
aplica. Matriz de render-readiness HOJE:

| Campo | Render-ready? | Onde aplica |
|---|---|---|
| flip_x/flip_y | ✅ | flip_uv bit0/1 → shader espelha UV |
| tint, self_tint | ✅ | collapsed_tint → RenderInstance.tint |
| per_corner_tint | ✅ | @location(9..12) shader |
| opacity | ✅ | RenderInstance.opacity → shader |
| tint_fill | ✅ | flip_uv bit2 → silhueta |
| hframes/vframes/frame | ✅ | `sprite_sheet_subrect` no extract (5890469) |
| region_enabled/region_rect/filter_clip | ✅ | `region_subrect` no extract (`20ba954`); UI `bdc82bf` |
| offset / centered | ✅ | `Sprite::resolve_anchor` → extract+gizmo (`e85e0e5`); UI `f86e70e` |

Antes de fazer a UI de region (T2.4) ou offset, faça primeiro o sub-UV/offset no
`shells/desktop/src/render_loop/sim_extract.rs` (padrão `sprite_sheet_subrect`).

───────────────────────────────────────────────────────────────────
§3 — RECEITA: adicionar um campo editável de Sprite (infra pronta, KEYSTONE 7e256b4)
───────────────────────────────────────────────────────────────────
O caminho geral Inspector→Sprite já existe. `SpriteFieldEdit` (enum em
`crates/ph2d-editor-core/src/screens/hero.rs`) JÁ declara o set completo de
variantes (FlipX/Y, Centered, Offset, Hframes/Vframes/Frame, RegionEnabled/Rect/
FilterClip, Tint, SelfTint, TintFill, Opacity, PerCornerTint). Pra ligar um campo:

1. **InspectorSpriteInfo** (hero.rs) += campo de display + **snapshots.rs** producer
   lê do `Sprite` + os 2 sites de teste em `screens/hero/tests.rs`.
2. **ids.rs** — id do widget (e se for seção nova: id da seção + id do color-dot).
3. **populate.rs** (`crates/ph2d-panel-inspector/src/`) — registra o widget state.
4. **sections.rs** — pinta + `hit_index.register(id, rect)` (as 3 pernas: populate +
   paint/hit + event; faltar uma = controle MORTO em silêncio).
5. **event.rs** — `WidgetEvent::Toggled/ValueChanged(id)` → `bus.push(
   EditorAction::InspectorSpriteEdit { entity_bits, edit: SpriteFieldEdit::X(v) })`.
6. **sync.rs** — reflete o snapshot no widget. **Checkbox: SÓ no `entity_changed`**
   (senão flicker 1-frame — o bus já foi drenado quando o sync roda). Number/slider:
   todo frame, pulando o focado/dragging.
7. **inspector_commits.rs** (shell) `apply_sprite_field` — já trata TODAS as variantes
   com clamps (hframes/vframes≥1, frame<cells, opacity[0,1]). Se a variante é nova,
   adicione o match + clamp. **PRESERVE `premultiplied`** (vide §5 F1).

**Seção NOVA (6→7+):** além do acima, em `paint.rs` (`crates/ph2d-panel-inspector`):
bump `notes_per_section: [Vec<_>; N]`, em `ids.rs` bump `LIVE_SECTION_IDS: [_; N]`,
adicione `live_section!(SECTION_ID, idx, ...)` + separator, e registre a seção em
`pre_populate.rs` em DOIS lugares: o loop `Plain` (color-dot) E `mark_collapsible_section`
(chevron) — senão chrome inerte (§5 E-1). Atualize os comentários "N live sections".

───────────────────────────────────────────────────────────────────
§4 — PRÓXIMOS PASSOS (ordem de menor risco; tudo padrão-ouro + auditar)
───────────────────────────────────────────────────────────────────
1. ~~**Tint / Self Tint + Per-corner colors (T2.7/T2.8)**~~ ✅ **FEITO** (`b590cdb`+
   `ea8e531`). Seção Color & Tint inteira em sub-tabs, auditada a erro-zero. O
   **padrão swatch→picker→sync-dispatch** está provado — reuse-o para qualquer campo
   de cor futuro (`crate::state::tint_f32_to_u8`/`tint_u8_to_f32` + loop em `sync.rs`).
2. ~~**Region (T2.4)**~~ ✅ **FEITO** (render `20ba954` + UI `bdc82bf`). `region_subrect`
   no extract (atlas.region_px + individual.dims; filter_clip = inset meio-texel CPU,
   sem shader/ABI); UI = toggle + 4 NumberInput (px) + filter_clip na seção Render Source
   (spec §3.3, NÃO é Rect2Editor — eram 4 NumberInputs). Funciona p/ Atlas E Individual
   (fix de source_pixels via renderer.individual().dims). Auditado 2-lentes.
3. ~~**offset / centered**~~ ✅ **FEITO** (render `e85e0e5` + UI `f86e70e`).
   `Sprite::resolve_anchor(ppm)` resolve centered/offset no campo `anchor` existente
   (sem shader/ABI): centered=false → +[w/2,-h/2] (origem top-left), offset px→m via ppm
   (Godot +y-down → -y local), aditivo sobre o anchor de tool. Aplicado em DOIS sites
   (extract + gizmo box) p/ a caixa de seleção acompanhar o quad; picking auto-segue
   (lê `ri.anchor`). 6 testes. **2 carry-overs documentados:** (a) gizmo box decompõe
   o affine → sob skew+anchor≠0 a caixa desloca (o mesmo skew-F1; GizmoView precisa
   carregar a basis inteira — task do gizmo); (b) centered/offset usam `size` cheio, não
   a célula do sheet (consistente c/ o modelo PH2D quad=size, mas decisão de spec-fidelity
   Godot p/ o Enio).
4. ~~**BulkSelect (T2.0)**~~ ✅ **FEITO** (`90d965c`). Editar com N sprites selecionados
   aplica a TODOS (fan-out no drain); campos divergentes mostram "Mixed" (checkbox
   Indeterminate · NumberInput em branco · swatch com traço). `compute_sprite_mixed`
   compara 19 campos. Audit 2-lentes pegou 2 stomps silenciosos (D-1 paired-axis →
   variantes per-axis OffsetX/Y+RegionX/Y/W/H; F5 swatch sem indicador → traço) +
   D-3/D-5 — todos fechados.
5. ~~**GlobalTint cascade**~~ ✅ **FEITO** (`bb4e5ea`, fechou a smoke do Enio "tint não
   atua nos filhos"). `cascade_tint_with_ancestors` no extract folda a cadeia de modulate:
   render = `self_tint × tint × Π(ancestor.tint)` — cada ancestral contribui o `tint`
   (cascateia), não o `self_tint` (local). Walk ChildOf O(depth), sem alloc (HR-3 verde),
   raiz bit-idêntica ao antigo. 2 testes provam tint cascateia + self_tint não.
6. **PRÓXIMO: refactor `sections.rs`** (~1400 linhas; gate `inspector_section_loc_cap`
   `#[ignore]`) → **audit W2 final** (lentes rotacionadas sobre o diff acumulado) →
   smoke → **ship** (Coordenador faz push 1× no fechamento da W2).
4. **BulkSelect (T2.0)** — Checkbox `Indeterminate` já existe; multi-select aplica o
   edit a N sprites (o drain de `sprite_edits` já é Vec).
5. **GlobalTint cascade** (handoff §5 carry-over) — pass `Π(ancestors.tint)` análogo a
   propagate_transforms (não existe; smoke `smoke_w2_color_tint.scene`).
6. **T2.1 refactor** sections.rs (já ~900 LOC) → módulos + remover `#[ignore]` do gate
   `inspector_section_loc_cap`.
7. **Audit W2 final** (lentes rotacionadas) → **smoke do Enio** → ship.
8. **W3..W8** (sorting/visibility/sampling · material/animation · named anchors ·
   widgets foundation · polish/i18n) — vide `docs/Sprite_projeto/15_plano_de_implementacao.md`.

───────────────────────────────────────────────────────────────────
§5 — GOTCHAS APRENDIDOS (não repita)
───────────────────────────────────────────────────────────────────
- **F1 (CRÍTICO):** `Sprite.premultiplied` é `#[serde(skip)]`. O `SetComponent`
  round-trip zera (default false) → fringe do BG-Removal volta. `apply_sprite_field`
  já preserva (captura antes, re-aplica via get_mut depois). Qualquer NOVO path que
  serializa Sprite precisa fazer o mesmo.
- **F2:** o `sync` roda DEPOIS do bus ser drenado → guard de "pending edit" no sync é
  morto. Checkbox: seed só no `entity_changed`.
- **E-1:** seção nova precisa de `mark_collapsible_section` + registro Plain do color-dot
  + roteamento em event.rs — senão chevron/dot inertes.
- **E-2:** shader — no branch premultiplicado, rgb escala por `extra_alpha`
  (opacity·tint.a·corner.a), NÃO pelo α do texel (senão α²). Spec §4.4 amendada.
- **Dispatch 3-pernas:** populate + paint/hit_index + event. Faltar uma = clique
  dropado em silêncio ([[feedback-panel-populate-register]]).
- **rustfmt CI:** rode `rustup update` antes de confiar no fmt local (CI é mais novo).

───────────────────────────────────────────────────────────────────
§6 — OPERACIONAL (resumo; detalhe no handoff principal §6)
───────────────────────────────────────────────────────────────────
- Slot warm: `bash scripts/slot-seed.sh impl-sprite` → prefixe TODO cargo com o
  `CARGO_TARGET_DIR=.../target-slots/slot-impl-sprite` impresso.
- Inner loop = `cargo check -p <crate>`. Gate no fim: nextest + clippy --all-targets + fmt.
- Gates Sprite: `architecture_sprite_inspector_surface` (Sprite==20 / RenderInstance==12
  / **size==156**) · `sprite_wgsl_valid` (naga) · `vertex_attr_offsets_match_struct` ·
  `transform_determinism`/`transform_versioned_postcard` (skew golden).
- Você NÃO pusha. Reporta commit local; o Coordenador faz ship + CI no fim da W2.
- Memória: `MEMORY.md` índice; [[project-sprite-w2-transform-skew-2026-05-30]] tem o detalhe.

**Confiança:** W2 ~60% (6 das ~9 seções; skew completo+auditado; pipeline de edição
provado e auditado a erro-zero). Daqui é colar campos render-ready na infra pronta
(§3), fazer o render de region/offset antes da UI, e fechar com audit + smoke. Sem medo.
═══════════════════════════════════════════════════════════════════
