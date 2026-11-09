# HANDOFF DE CONTINUAÇÃO — line/GameEngineFuncional02, 09/11 — Outline: W0 e W2a feitas, W2b–W6 por fazer

> NÃO é handoff de integração (o dono proibiu fechar a linha). Leitor: a próxima LLM desta linha.
> Anterior: [BLOCOS_1_2](HANDOFF_CONTINUACAO_2026-11-09_BLOCOS_1_2.md). Lei do Outline: o plano
> [f2_contorno_e_formas_plano.md](../componentes/f2_contorno_e_formas_plano.md) — decisões do dono no topo, a REVISÃO opção B,
> e no FIM «## W2 — desenho fechado (09/11)» + «### Endereços conferidos (09/11)». Não re-decida nada disso.

## 1. Commits desde `ce390095bc` (todos na branch, nada empurrado)

| hash | quem | o quê |
|---|---|---|
| `ba83afa14c` | GEFuncional | **W0**: `measure_outline_pieces` (motor real) + `MAX_PIECES = 256`, `MAX_POINTS = 512`, `trace_fitted`; peças 14× mais rápidas, byte-idênticas |
| `225684b374`, `5b4d8e2e8d` | GEFuncional | plano: desenho fechado da W2 e endereços para W2b–W6 |
| `dcb56671a7` | GEFuncional | blocos 1 e 2 provados pelo caminho REAL (linha da Hierarquia, Inspector pintado, desfazer por quadros) |
| `7b0adbfe1c` | GEFuncional | **W2a**: `ShapeDesc::Outline { half_x, half_y, key }` + `PhysicsWorld::register_outline` (motor) |
| `4df0d9d9ca`, `9f1644d7f2`, `aad7129625`, `ecca0421ac` | GameEngineFuncional02 | corpo sem forma fica no solver; nav ignora `shape_off`; aba «Differ from this layer»; alerta na Nav Region; foto |

⚠️ A janela GameEngineFuncional02 estava a curar o tecto de LOC de `ph2d-app-physics` (55 579 > 55 515,
`crates/ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs:221`) — confira `git log` antes de tocar nessa crate.

## 2. Blocos 1 e 2 — provas fechadas (`dcb56671a7`)

- Olho: `shells/desktop/src/render_loop/fase_bus_hierarchy_tests.rs:59` — a linha sai de `build_hierarchy_snapshot` →
  `EntityNodeMap` («Collision Shape (7)» sob «Player»), `entity_for` dá a FORMA (hipótese (a) refutada pela cadeia real); o
  Inspector é PINTADO (`inspector_da_fisica`, `:226`). Verde: y 0,450 → −10,350, cartão pintado. Mutações: `continue` na ponte
  (y fica 0,450), `main_shape` no Inspector («Add Shape» no lugar do cartão), linha resolvida ao pai (o olho fecha o Player) — vermelhas.
- Multisseleção: `fase_bus_inspector_preso_tests.rs:326` chama o `drain_section_edits` real (agora `pub(super)`), modelo de
  QUADROS (o `post_frame_undo` real exige `gfx`). Mutação «2.ª forma um quadro depois» → 2 passos, Ctrl+Z deixa uma → vermelha.
  Misto + «Use layer default»: `:445` (misto `0b1`; o default tira a exceção das duas pelo dreno real).
- Smoke do dono dos blocos 1–2: PENDENTE (passos no handoff BLOCOS_1_2 §5; o SELECT do Player é `PH2D_PHYSICS_SMOKE_SELECT=Player`).

## 3. W0 — o custo e os tectos (`ba83afa14c`)

Tabela em `crates/ph2d-outline-trace/README.md` §Ceilings. ~1,2 µs por peça em contacto por tique (0,30 ms a 257 peças);
retraço O(pontos²): 3,56 ms a 516 pontos. `trace_fitted` sobe o Detail ao MENOR que cabe (dobra + bissecção); só com o Detail do
tamanho da imagem sobe o Ignore specks under; nunca corta a arte (`tests/tecto.rs`, 3 mutações). As peças eram CÚBICAS
(Hertel–Mehlhorn varria todos os polígonos): índice aresta→dono + fusão retomada = mesma sequência (16 107 peças, um sha256).

## 4. W2a — o motor (`7b0adbfe1c`)

`crates/ph2d-physics/src/world/outline.rs`: `OutlineKey` (blake3 do conteúdo, u64), guarda das peças que o hull recusaria,
`builder` (compound de `convex_hull`; chave ausente/peças inválidas → `cuboid(half_*)`), `register_outline` (só insere).
`ShapeDesc::Outline` em `world/shape.rs:52`; `shape_builder`/`build_collider` recebem a tabela (spawn, `retune_body`,
`attach_part`, `reshape_body`). Nav/overlay/snap leem o Outline como `Cuboid half_*`; `top_shrink_room` = 0 (agachar não
deforma a arte). Testes `crates/ph2d-physics/tests/it/outline_shape.rs` (U côncavo, pouso, massa, rewind, chave ausente,
chaves) + 4 mutações vermelhas. `measure_outline_pieces` usa o caminho do produto; tiques iguais aos da W0.

## 5. O que falta — nesta ordem

1. **W2b — ECS + ponte + formato.** `ColliderShape::Outline { half_x, half_y }` (anexada) e `ColliderOutline { rings:
   Vec<Vec<[i32;2]>>, picture_px, detail_q4, alpha_cutoff, min_island_px }` registado, na FILHA. Ponte (`scale.rs::body_desc`,
   `bridge/bodies.rs`, `bridge/parts.rs`): peças = `convex_pieces(rings, detail)` com CACHE por `(hash dos anéis, detail)`,
   metros com a escala COM SINAL (fórmula no plano), `register_outline`, chave no `ShapeDesc`. `body_law.rs`
   (`shape_fitted_to`/`follows_picture` para o Outline como o `Cuboid`), todos os `match` de `ColliderShape`. Overlay desenha as
   peças/anéis, não a caixa. `PROJECT_SCHEMA` 225 → 226 (escada + tripla + espelhos; o integrador reconta com `schema-recount.py`).
   Testes red-first: U pela ponte ECS (`PhysicsBridge::dispatch`), L espelhado (`scale.x = −1`) cai do outro lado, escala,
   ida-e-volta save/load, rewind do ECS (`bridge/rewind.rs` re-spawna por `BodyDesc`).
2. **W2c — o produtor.** Imagem-fonte = o próprio objeto ou o antepassado com imagem mais próximo até ao corpo
   (`SpritePixels`/atlas → `AssetDb::image_rgba8`; Animated: OR das células de `tag_cells`, todas sem tags) → `Mask::from_rgba8`
   (`alpha_cutoff`) → `trace_fitted` → `queue_set` (um passo de undo). Código fora de `ph2d-app-*` (crate de cola nova).
3. **W4 — cartão + Auto Key.** «Outline» no seletor de forma; Detail, Alpha cutoff, Ignore specks under; «Too detailed — Detail
   raised to {x} to stay under {n} pieces» quando `Fitted::raised`; Follows the picture × Custom size + Fit to picture; canais
   ANEXADOS em `canal_fisica.rs` para Detail e Alpha cutoff (retraço no Play). App só em inglês, zero hex/f32/string solta.
4. **W5 — canvas.** Pontos do Outline só com a FORMA selecionada (resposta 3 do dono): arrastar, duplo clique na aresta
   acrescenta, Alt+clique tira (o vetor não tem esses gestos — implementar em `ph2d-shape-handles`), um passo de undo; editar
   pontos passa a «Custom»; Fit to picture retraça.
5. **W6 — cena 120:** um contorno (Stretched Ball ou Crate com imagem), argola com buraco, corpo de 3 formas; fotos com
   `docs/Components/ferramentas/fotografa_cena.sh` ANTES do smoke; smoke ao dono em passos numerados, um por bloco (1, 2 e 3).
6. Binário smoke compilado na worktree (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2×).

## 6. Aberto (não esquecer)

- O(pontos²) que resta (busca de orelhas, conflito de cordas): baixar a ordem move `MAX_POINTS` (re-medir com `measure_outline_pieces`).
- `trace()` corre ANTES do tecto: imagem enorme com Detail mínimo paga a escadaria inteira antes de o `trace_fitted` subir o Detail
  (não medido acima de 3 076 pontos).
- Tecto de LOC de `ph2d-app-physics` (cura da outra janela; W4 acrescenta lá — mover para crate não-`app`).
