═══════════════════════════════════════════════════════════════════
HANDOFF → COORDENADOR · Vector W3 · T3.1 `ph2d-panel-vector-graph` (Coord-B scaffold)
Autor: Implementador Vector (sessão W3, T3.2) · 2026-06-03
═══════════════════════════════════════════════════════════════════

## §0 — Pedido (1 linha)
**Scaffolda o panel docado `ph2d-panel-vector-graph` (T3.1, Coord-B) que coloca/wira o
nó `vector.source` e renderiza o `VectorNetwork` cozido — é o que falta pra fechar o
smoke Day 8.** O nó backend já existe, testado e verde (T3.2 abaixo).

## §1 — O que JÁ está pronto (T3.2 + fundação, esta sessão — commit local)
- **Substrato (ADR-0058-amendment-1):** nós vetoriais emitem/consomem `VectorNetwork`
  pelo canal opaco do cook `CookValue::Opaque(Arc<dyn Any+Send+Sync>)`. `ph2d-nodegraph`
  continua **zero-deps**; caps congelados `NodeOp=2/OpResolver=1/NodeManifest=8` **intactos**
  (gate `architecture_contract_surface` 3/3). `Domain::Vector` + `EvalCtx::{input_any,emit_any}`.
- **Glue `ph2d-vector-graph`:** `VectorEvalExt::{emit_network, input_network}` + `VECTOR_PORT`
  (`Domain::Vector`/`Clock::Static`). É a borda tipada — ninguém toca `Arc<dyn Any>` à mão.
- **Nó `ph2d-node-vector-source` (T3.2):** 5 variantes (rect/ellipse/polygon/star/spiral),
  snap Q16.16 (cross-OS bit-identical), registrado via `ph2d-node-sync`. **7/7 testes**,
  incluindo caminho completo via cook real.
- Consumidores motion/eval-motion/debug/registry migrados (`as_stream()`); shell compila.

## §2 — Contrato do nó `vector.source` (o que o panel dirige)
- **Node id / type:** `"vector.source"` (`NodeTypeId::of("vector.source")`); já em
  `register_all_nodes` (registry-init).
- **Output:** porta 0, `VECTOR_PORT`. Valor cozido = `CookValue::Opaque(Arc<VectorNetwork>)`.
- **Params (8, todos `f32` — sliders/dropdown do panel setam via `graph.set_param`):**
  | param | default | nota |
  |---|---|---|
  | `kind` | 0.0 | **discriminante**: 0=Rect 1=Ellipse 2=Polygon 3=Star 4=Spiral (dropdown → f32) |
  | `width` / `height` | 100.0 | bounding box (radiais usam `width` como diâmetro) |
  | `sides` | 6.0 | polygon/star (contagem) |
  | `inner_ratio` | 0.4 | star: raio interno / externo |
  | `turns` | 3.0 | spiral |
  | `samples_per_turn` | 24.0 | spiral |
  | `rotation` | 0.0 | radianos |

## §3 — O que o panel precisa fazer (Coord-B, plano §6 T3.1)
1. **Crate `ph2d-panel-vector-graph`** docado; registrar em `register_all_panels`; chrome
   dispatch novo se preciso (plano §3.B). Abre quando vector layer selecionado.
2. **Node placement + edge drag + param sliders** (critério T3.1). Para o smoke basta:
   1 nó `vector.source` colocável + sliders dos params acima.
3. **Cozinhar + renderizar (a peça que NÃO existe — é o consumidor):** o W2 renderiza
   **tool-direct** (`committed_vector_*_paths` → `vector_scene::reconcile` → ECS →
   `draw_vector_network`). O graph é um produtor **novo**. O panel/scene precisa:
   - manter um `Cook` + `Graph` (cook incremental por frame; memoiza — reusar o mesmo `Cook`);
   - `let out = cook.cook(&g, registry, target, playhead)?;`
   - extrair o network: `out[0].as_any().and_then(|a| a.downcast_ref::<VectorNetwork>())`
     (ou, mais limpo, um helper sobre `VectorEvalExt`);
   - alimentar esse `VectorNetwork` no caminho de render existente (`draw_vector_network`).
   - **Landmine ADR-0076 (do handoff W2 §4.3):** se espelhar em entidades ECS via reconcile,
     ele re-pareia por **COUNT** — troca wholesale da lista exige `despawn_all_vector_entities()`.
     Pro smoke, render direto do network cozido (sem ECS-reconcile) é o caminho mais curto.

## §4 — Smoke Day 8 (alvo do plano §2)
"Coloca um nó `vector.source` no Geometry Graph panel, vê o primitivo renderizar live;
mexe um slider (ex.: `sides` 3→8, ou `kind` Rect→Star) e vê atualizar real-time."
(O cook memoiza por revisão de input + hash de params — editar um param invalida só aquele
nó + downstream, então o re-cook do slider é barato.)

## §5 — Desbloqueios a jusante
- **T3.3 (boolean)** e **T3.4 (offset)** já têm o substrato pra emitir/consumir
  `VectorNetwork` (`input_network`/`emit_network`). T3.3 ainda depende deste panel (T3.1) +
  ADR-0065 SDF. Quando o panel existir, o Impl segue pra T3.3.

## §6 — Referência
- **[ADR-0058-amendment-1](architecture/decisions/0058-amendment-1.md)** — carrier opaco (normativo).
- Spec [`02_geometry_graph.md`](Vector%20Module/02_geometry_graph.md) §2.1.2/§2.2.1 (atualizados p/ a API real).
- Crates: `ph2d-node-vector-source`, `ph2d-vector-graph`, `ph2d-nodegraph` (`value.rs`/`cook.rs`).
- Padrão de panel docado existente: ver `register_all_panels` + panels do Painter/Sprite.
═══════════════════════════════════════════════════════════════════
