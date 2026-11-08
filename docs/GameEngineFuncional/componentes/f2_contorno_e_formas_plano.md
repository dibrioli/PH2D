# Plano: Shape «Outline» pelo contorno, Collision Shape pela Hierarchy, várias formas num corpo (`line/GameEngineFuncional02`)

Leitor: a próxima LLM desta linha. Aprovado pelo dono em 08/11/2026 («faça como você sugere»). Oráculo do contorno:
[`../oraculo_contorno/README.md`](../oraculo_contorno/README.md) (Godot 4.7.2, MIT; fixtures + `cobertura.txt`).

## Decisões do dono (08/11)

- Animated Sprite: **um contorno só** = união (OR) do alfa dos quadros usados pelas tags (todos, se não houver tags).
- Reentrância = **sólida** (peças convexas). **Buraco fechado continua buraco.**
- Controlos: **Detail** (px, 2), **Alpha cutoff** (0.5), **Ignore specks under** (px), pontos editáveis no canvas
  (arrastar · duplo clique na aresta acrescenta · Alt+clique tira); «Follows the picture» × «Custom size» + «Fit to picture».
  Auto Key grava também Detail e Alpha cutoff (lei de 19/10; retraçar custa 80–310 µs, medido no oráculo).
- **Add ▸ Collision Shape:** sem seleção → objeto próprio com `RigidBody{Static}` invisível; nunca entra noutro corpo.
  Com objeto selecionado → **objeto FILHO** (`ChildOf`, linha nova na Hierarchy), nunca componente do selecionado. Pai
  (ou antepassado) com corpo → mais uma forma desse corpo (W-Compound, «Part of: X»). Pai sem corpo → o filho nasce com
  o seu próprio Static (Inspector: «Own body (Static) — make the parent a body to join it»).
- **Várias formas num corpo: UM jeito só — os filhos** (ADR-0110, `ph2d-physics-ecs/src/bridge/parts.rs`). O cartão Shape
  ganha a lista **«Shapes»**, que é VISTA dos filhos (linha 0 = a forma do corpo; depois cada parte na ordem da árvore;
  tipo, tamanho, offset, rotação; clicar seleciona o filho; «+ Add shape» = o spawn acima; «Remove shape» = apagar o filho,
  um passo de undo). Canvas: «Edit shape on canvas» mostra as alças de TODAS as formas do corpo.
  ⛔ Recusado: `Collider.shapes: Vec<…>` (segunda verdade ao lado dos filhos, perde `Copy`, degrau de schema).

## Desenho técnico (padrão-ouro)

- `ColliderShape::Outline { half_x, half_y }` (variante ANEXADA, `Copy`, caixa de limites em m) + componente registado
  novo `ColliderOutline { rings: Arc<[Ring]>, detail_q4, alpha_cutoff, min_island_px, follows, source, cell_px }`.
  Anéis em inteiros Q4 (1/16 px), externo CCW, buracos CW. Peças convexas DERIVADAS (função pura), nunca gravadas.
  ⛔ Recusado: `Arc`/`Vec` dentro do enum (tira `Copy` de `Collider`/`ShapeDesc`/`BodyDesc`, 351 ficheiros).
- `PROJECT_SCHEMA` 219 → **220** só pelo componente novo (escada + tripla + espelhos `ph2d-render`/`ph2d-script`).
- Crate pura nova **`ph2d-outline-trace`**: máscara → marching squares inteiro (ordem fixa) → Douglas–Peucker UNILATERAL
  (nunca corta a arte; predicados i64; cruzamento devolve o vértice) → ponte dos buracos (Eberly) → ear clipping exato →
  Hertel–Mehlhorn → fusão de vizinhas dentro do Detail sem cobrir buraco → ilhas abaixo do mínimo saem. Oráculo
  interno: `parry2d::transformation::hertel_mehlhorn_idx` (peças antes da fusão iguais). ⛔ VHACD (aproximado, float).
- Cola `ph2d-physics-outline`: recurso `PictureAlpha` publicado por `sheet::refresh` (sprite-anim); `physics_apply` e
  `follow_auto_collider` chamam `trace()` → `queue_set` (um passo de undo). Rapier: `ColliderBuilder::compound` de
  `ConvexPolygon`. ⚠️ Espelho: escala COM sinal e inverter o sentido quando o determinante < 0 (`scale.rs:75` usa `abs()`).
- Tecto de peças/pontos: MEDIDO antes (W0, `measure_outline_pieces`, `--profile smoke`); acima dele o Detail sobe e o
  cartão diz («Too detailed — Detail raised to {x} to stay under {n} pieces»); nunca corta a arte.

## Ondas

| Onda | O quê | Formato salvo? | Estado |
|---|---|---|---|
| W3 | Add ▸ Collision Shape (3 casos) + lista «Shapes» no cartão | não | **agora** (antes da integração da noite) |
| W1 | `ph2d-outline-trace` contra as fixtures do Godot (0 % de arte fora; anel com buraco; disco em 1 peça; L em 2) | não | **agora** |
| W0 | medir o custo das peças; escrever o tecto | não | depois da integração |
| W2 | motor e formato (schema 220), rapier, rewind, espelho | **sim** | depois da integração |
| W4 | Outline no cartão + Auto Key | sim | depois |
| W5 | canvas: alças de todas as formas, pontos do Outline, seguir a imagem | sim | depois |
| W6 | cena 120 (contorno, argola, corpo de 3 formas) + fotos + smoke do dono | — | depois |

Riscos: LOC sem folga em `ph2d-app-physics` e na shell (cura = cortar/MOVER); `InspectorPhysicsInfo` sem `Default`
(outras linhas precisam dos campos novos); Static filho de pai Dynamic é corpo separado; nav usa a caixa de limites.

## ⛔ REVISÃO 08/11 (smoke do dono): opção B — toda forma é um objeto filho

Dono: *«tirar collision shapes dos componentes e deixar apenas como Objeto aparecendo como filhos na hierarchy como
objetos independentes (invisíveis). Sai dos componentes e suas configurações não aparecem no inspector de outros
objetos.»* A lista «Shapes» (W3-B, `986c158873`) é REVOGADA. ADR novo previsto: **0199 «A forma de colisão é um objeto
filho do corpo»** (confirma e estende o ADR-0110; não o revoga).

- **Motor, UMA lei:** as formas de um corpo = o seu `Collider` (se tiver) e depois as formas descendentes em ordem de
  árvore, parando num corpo filho e atravessando grupos (`ph2d_physics_ecs::shapes_in_order`, `parts.rs:150`). A 1.ª é a
  **forma principal**, viaja no `BodyDesc` como hoje (Crouch, linha d'água, força de zona, `rest`, nav e gatilhos leem-na);
  as outras são partes (`bridge/parts.rs`). `BodyDesc.shape_pose: [f32;3]` anexado (zero = hoje). O lado-forma do catálogo
  (`row_of("ph2d::physics::Collider")`: material, one-way, superfície, zonas, sinais, e o futuro `ColliderOutline`) é lido
  da entidade da FORMA. Mesmo hash de 300 ticks construída das duas maneiras (gate de equivalência).
- **Editor, UMA lei:** nenhuma porta do editor põe `Collider` num corpo (gate `nenhuma_porta_do_editor_poe_a_forma_no_corpo`).
  «Collision Shape» sai do Add Component (fica registada). Corpo nasce com a filha (`nasce_com_forma`, physics-ecs).
- ⛔ Recusados: corpos sem collider próprio com todas as formas como partes (parte os 5 leitores da forma principal e a
  ordem dos handles) · motor que recusa `Collider` no corpo (234 ficheiros de teste sem ganho) · sistema por quadro que
  converte (muta o documento e o undo) · `Collider.shapes: Vec` · lista «Shapes» no Inspector do corpo.
- **Migração** `migra_formas_para_filhos(world)` no load, `ver <= 219` → **schema 220** (o Outline passa a **221**): por
  `StableId`, filha «Collision Shape» primeira entre irmãs, Transform identidade, leva o `Collider` e o lado-forma; mestre
  só (instâncias materializam); trilhas da Timeline do lado-forma re-alvejadas para a filha; macros resolvem
  `main_shape(body)` ao tocar.

| Onda | O quê | Est. |
|---|---|---|
| B0 | tirar a lista W3-B (fica `shapes_in_order`, os cortes de LOC e a rota `set_command_pick(COLLISION_SHAPE)` = «Add Shape») | 1 h |
| B1 | motor: `BodyQuery` sem `&Collider`, forma principal resolvida, lado-forma da forma, `shape_pose`, ordem das partes | 1 d |
| B2 | migração + schema 220 (+ instâncias, Timeline) | 1–1,5 d |
| B3 | ações: catálogo, seed do `RigidBody`, Player/Character/Static nascem com filha, `pendura_forma`, `follow_auto_collider` pelas formas da imagem, Make Independent Body | 1 d |
| B4 | Inspector: corpo só Body; filha com o cartão Shape inteiro + Collision Layers + «Shape of: {body}» | 1 d |
| B5 | canvas: corpo selecionado mostra as alças da forma filha | 0,5–1 d |
| B6 | força de zona em formas extra (efetor por collider) | 0,5 d |
| B7 | cenas (120, jogo, macros 8) por `nasce_com_forma`; fotos; smoke | 0,5 d |

Outline (W0–W6) passa para a FILHA: `ColliderOutline` na forma, `source` = antepassado com imagem mais próximo até ao
corpo. **Último ponto seguro para integrar:** o commit da B0 (sem a lista revogada, sem mudança de formato).
