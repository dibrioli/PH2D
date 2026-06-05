═══════════════════════════════════════════════════════════════════
HANDOFF → IMPLEMENTADOR Vector · W5 INTEGRAÇÃO — pressão→variable-width (desbloqueado)
Autor: Coordenador (2026-06-04) · foundational ENTREGUE, agora é tua integração
═══════════════════════════════════════════════════════════════════

## §0 — ANTES DE TUDO
**Baseline = HEAD LOCAL** (~59 commits não-pushados). NÃO rebase pra origin. Sanity:
`cargo check -p ph2d-tool-vector-pencil`. **Git SCOPED** (`git add -- <teus paths>` ·
`--no-verify`; NUNCA `-A`/`stash`). ⚠️ **Há outro implementador (Painter) com arquivos
STAGED no índice compartilhado** — commit SEMPRE com pathspec (`git commit ... -- <teus paths>`),
`git status` antes de stage, `M`/`??` alheio → não comite.

## §1 — O QUE EU (COORD) ENTREGUEI (foundational, pronto pra consumir)

| Hook | Onde | Uso |
|---|---|---|
| `draw_variable_width_stroke(scene, &[Vec2] centerline, &[f32] widths, OklchColor, Affine)` | `ph2d_vector` (`79ecd2e`) | **render-time** — largura por amostra (pressão live). Expande a polyline → band preenchida no GPU. |
| `WidthProfile { start, end, bulge }` + `scale_at(t)` | `ph2d_vector_doc` (`8dca426`) | **paramétrico/persistido** — taper start→end + bulge no meio. |
| `StrokeStyle.width_profile: Option<WidthProfile>` | `ph2d_vector_doc` (`8dca426`) | `Some` → o renderer (`draw_vector_network`) **já desenha variable-width automático**. `None` = constante. |

**NÃO toque `ph2d-vector` nem `ph2d-vector-doc`** — é foundational/Coord, já entregue. Se precisar
de mais (ex.: jitter no profile, cap arredondado), **reporta** — eu estendo.

## §2 — TUA INTEGRAÇÃO (3 peças, tudo nos TEUS crates)

### A. Live preview no drag (`shells/desktop/src/render_loop/vector_pencil_bridge.rs` — TEU, §3.A.4)
Hoje (`~L117`) o bridge faz `scene.stroke(&Stroke::new(line_width_world), ...)` — **constante**.
Troque por variable-width usando as amostras em progresso:
```rust
// centerline = posições das StrokeSample; widths = base × sample.pressure
let centerline: Vec<Vec2> = samples.iter().map(|s| s.pos).collect();
let widths: Vec<f32> = samples.iter().map(|s| line_width_world as f32 * s.pressure).collect();
ph2d_vector::draw_variable_width_stroke(scene, &centerline, &widths, stroke_color, world_to_screen);
```
(A pressão JÁ está capturada — `StrokeSample.pressure`, 0..=1. Não precisa capturar nada novo.)

### B. Commit persistido (`ph2d-tool-vector-pencil` — TEU)
Ao commitar o traço, dê a cada SEGMENTO um `StrokeStyle.width_profile` com a pressão dos seus
dois vértices (assim o traço salvo/editável renderiza variable-width sem geometria baked):
```rust
let mut style = StrokeStyle::default();
style.width = DEFAULT_PENCIL_WIDTH;            // base
style.width_profile = Some(WidthProfile {
    start: pressure_at_start, end: pressure_at_end, bulge: 0.0,
});
// SetStrokeStyle { seg, style } por segmento → draw_vector_network desenha variable-width
```
(Per-segmento taper entre as pressões dos endpoints = a pressão por-amostra preservada. O renderer
já consome `width_profile` automaticamente — zero mudança de render do teu lado.)

### C. Riqueza do nó `vector.width-profile` (TEU, opcional)
Estenda os eixos de perfil (taper/contrast) na abordagem de banda geométrica do nó, OU faça o nó
emitir `WidthProfile` no `StrokeStyle` das suas regiões (agora que o tipo existe). Tua escolha.

## §3 — Anti-colisão + caps
- **TEU:** `vector_pencil_bridge.rs` (tool-bridge, exceção §3.A.4) · `ph2d-tool-vector-pencil` ·
  `ph2d-node-vector-width-profile`.
- **NÃO TEU:** `ph2d-vector`/`ph2d-vector-doc` (Coord, entregue) · `render_loop/mod.rs` (CONTENDED —
  se precisar mexer no call-site do bridge, **reporta**, não edita).
- **Caps congelados:** largura é render-time (`draw_variable_width_stroke`) OU via `StrokeStyle`
  (entregue) — **NÃO** inche `Vertex`/`Segment`. WidthProfile NÃO está no contrato congelado.

## §4 — Validação + smoke
- Inner loop `cargo check -p ph2d-tool-vector-pencil` (e `-p ph2d-node-vector-width-profile`).
- Smoke: desenha com o Pencil variando pressão (ou trackpad force) → traço afina/engrossa live;
  solta → persiste variable-width. Me reporta quando landar que eu fecho a lente visual do T5.3.
- A primitiva já tem teste de unidade (`variable_width_band`); o profile idem (`scale_at`).
═══════════════════════════════════════════════════════════════════
