# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-13 — F14: O «CORNER ROUND» EM TODA FORMA QUE TEM QUINA

> Anterior: [F13](HANDOFF_INTEGRACAO_line_UIUX_2026-10-12_F13_SELECIONAR.md) (no `main` desde 12/10).
> Pedido do dono e escopo A–E: [`23_prompt…`](../vetor_nova_ui/23_prompt_F14_arredondar_as_quinas.md).
> Lei, medidas, a lista de fora e as recusas: [doc 24](../vetor_nova_ui/24_arredondar_as_quinas.md).

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, sobre o `main` `757afebd58` (0 atrás).
- **`PROJECT_SCHEMA` NÃO mudou** (`VecShape::Param.values` é `[f64; 8]`; o índice novo de cada forma
  era um lugar a zero). Nenhum contrato congelado tocado (`ph2d-vec-*` não é congelado).

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

UI/UX: «… o modo Forma que selecciona e o «Corner round» em toda forma com quina (F14)», smoke
`…6..21`, «Último:» → este handoff (662/700 bytes).

## §1 — O que mudou

| commit | o quê |
|---|---|
| `cfab20e549` | **A** `ph2d_vec_scene::round_shape_corners` (`corner_round.rs`): a porta única, por cima do `cook` (`cook` = `cook_sharp` + o campo de `ShapeKind::corner_round_field`). Carimba o raio em TODO vértice (a geometria decide), principal + sub-contornos fechados; as linhas abertas da face seguem o contorno pelo corte mais perto da recta da ponta. Sai o `round_slice_corners` (a pizza passa pela porta). **Motor** (`corner_live`): colinear pelo SENO (`straight_or_folded`, também no `corners.rs`) em vez do `acos`; contornos fechados de 2 vértices (o coração). **B** os censos `every_live_corner_has_a_rounder` · `every_shape_is_in_exactly_one_class` · `the_corner_round_field_is_the_last_and_neutral`; o `no_cooked_shape_loops_or_hooks` varre o campo novo sozinho (0 → 500 px, e aos pares). **C** o campo nas 37 formas; ⊙ `corner_round(field, corner, next)` em 24, ⇕ em 10, a pizza/elipse/segmento no ⊙ da pizza; as alças antigas dessas formas leem o raio a zero (`on_sharp(…, &[F])`) — gate `the_corner_round_moves_no_other_handle`. **D** o Motion chama a mesma porta; o `corner` passa a mexer o coração (ParamGate + nota no gate de bytes, como a engrenagem). **E** a cena 21 (`vector_mode_quinas`) |
| `3e22e37b76` | gates `a_closed_hole_rounds_too` e `the_corner_is_the_vector_corner_round` (Motion); a grelha da cena cabe entre a paleta e o Inspector |
| `d8c673d585` | o gate das linhas da face exige o corte MAIS PERTO (a mutação `min→max` sobrevivia) |
| `171a337e9f` | o tecto de LOC (o portão apanhou `kind.rs` 710, `corner_live.rs` 704, `vector_mode.rs` 703): a tabela do campo vive no `corner_round.rs`, comentários curtos |

## §2 — Medido e decidido (técnico, delegado)

- **O tipo do vértice mente**: a engrenagem marca os 32 cantos dos dentes `Smooth`, o documento 2 ⇒ a
  lei é a do Motion (carimbar tudo), não a da pizza da F12 (só `Corner`).
- **O `acos` mentia**: o ruído dos lisos da elipse esticada lia-se `2,1e-8` rad pelo `acos` e `0`
  exacto pelo produto vetorial; a menor quina real do catálogo é `0,18` rad. O gate da F12
  `only_the_slice_corners_round_the_smooth_arc_vertices_stay` atribuía-o às alças.
- **O coração (2 vértices)** tinha duas quinas reais (`~43°`, `~107°`) invisíveis ao motor e ao censo,
  que lia pelo mesmo predicado ⇒ gate com régua independente `the_engine_sees_every_corner_the_geometry_has`.
- **O rectângulo**: o «Corner round» dele = o *Radius* do rectângulo arredondado a `1e-9` (o resto é
  o `KAPPA` de 8 casas) e o Affinity dá-lhe raio ⇒ recebe o campo.
- **Visível**: o censo conta a quina com recuo máximo `> 1e-6` da caixa (a espiral com *Expansion*
  ≥ 4,17 tem troços de `1e-9` no centro).
- **A cena** anda pelo TRILHO da alça (o `track` da vista com o cursor por cima): «para o centro»
  não subia o raio no raio (Bolt) e na cruz.

## §3 — Foundational tocado

- `ph2d-vec-scene`: `pub fn round_shape_corners`, `ShapeKind::corner_round_field`; saiu o `pub fn
  round_slice_corners` (zero chamadores fora da crate); `corner_live::corner_at` e
  `round_authored_corners*` aceitam 2 vértices; o predicado de colinear mudou (também no `corners.rs`).
- `ph2d-tool-vector`: o catálogo (`shapes.rs`), a tabela das alças; `stroke_census_tests::sweep` é
  `pub(crate)`. `ph2d-node-motion-shape`: o ParamGate do `corner` + `Heart`.

## §4 — O que só a árvore combinada pega

- Qualquer linha que leia `corner_at`/`round_authored_corners` sobre um contorno de 2 vértices ou que
  tenha um gate a contar com o `acos` (a `line/Vector`: as Live Corners, o Fillet/Chamfer do nó).
- Quem chamar `round_slice_corners` (saiu) ou montar `ShapeKind::Rectangle` como «forma sem campo».

## §5 — Provas de mutação (agente `mutacao` + 1 à mão)

12 cláusulas, controlos limpos (âncoras 1×, população > 0); 11 vermelhas à 1.ª, a 5 sobreviveu e foi
curada com um gate (`d8c673d585`, vermelha confirmada à mão: «PredefinedProcess … saltou o contorno»).

| cláusula | mutação | vermelho em |
|---|---|---|
| as linhas da face seguem | desligado | `the_lines_on_the_face_keep_touching_the_outline` |
| sub-contornos fechados | nenhum | `a_closed_hole_rounds_too` |
| carimbo em todo vértice | só `Corner` | `every_corner_rounds` |
| guarda do negativo | sem `r <= 0` | `zero_is_the_shape` |
| o corte mais perto | `min_by → max_by` | (sobrevivia) → `the_lines_on_the_face…` |
| colinear pelo seno | desligado | `a_straight_vertex_never_moves` + o da F12 |
| 2 vértices no cozimento / no `corner_at` | `n < 3` | `every_corner_rounds` / `the_engine_sees_every_corner…` |
| o `cook` aplica o campo | ignorado | `the_rectangle_…_round_rect_radius` · `every_live_corner_has_a_rounder` |
| o coração na tabela | `None` | `the_corner_round_field_is_the_last_and_neutral` (+2) |
| alça lida com raio zero | sem `on_sharp` | `the_corner_round_moves_no_other_handle` |
| o Motion na mesma porta | só o contorno principal | `the_corner_is_the_vector_corner_round` |

## §6 — Smoke

Fotos: `/var/tmp/ph2d_F14_cena21/1_quinas.png` (as 36, o cubo seleccionado com o ⊙) e
`2_retangulo.png` (`PH2D_CENA21_FICA=0`: o ⊙ dentro do canto do rectângulo).

## §7 — O que fica ABERTO

1. Herdados da F13 §7: as perguntas ao dono sobre o *Cutout* do polígono, a seta dobrada com *Tail* >
   *Head width*, o degrau do ✓; a fila de ⇕ com traço grosso.
2. A integração (por ordem do dono).

## §8 — Portão, binário e `agent-loop-profile.sh`

- **Portão batched** (agente `verificador`, 1× sobre `757afebd58..d8c673d585`, loadavg 48–146):
  `nextest-impacted` **18 895 / 18 896**, 9 967 pulados — o único vermelho era
  `workspace_src_files_under_loc_cap` (os três ficheiros acima) → curado em `171a337e9f` e re-corrido
  verde, com `ph2d-vec-scene` 558/558, os censos do `ph2d-tool-vector`, a cena 21, clippy das duas
  crates e `fmt --all --check` de novo · clippy do `ship.sh` (`--workspace --all-targets --features
  ph2d-spike/bevy_ecs -D warnings`) **exit 0** · `fmt --all --check` ✓ · `check --workspace
  --all-targets` ✓ · `the_shell_only_shrinks` ✓ · `architecture_claude_md_cabe_no_orcamento` ✓ ·
  `the_live_process_folder_holds_no_dated_record` ✓ · `every_door_the_names_enter_by_translates_them` ✓.
  Nenhum vermelho da Luz 2D nesta corrida.
- `bash scripts/doc-index.sh` ✓ · `rm -rf target/*/incremental` (22 G + 4 G).
- **Binário de smoke** (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`),
  2.ª corrida, HEAD `171a337e9f`: `Finished \`smoke\` profile [optimized] target(s) in 0.27s` (zero `Compiling`).
- `agent-loop-profile.sh` (20 sessões): paralelismo 1,13 ✗ · respostas/sessão 178 ✓ · test:check
  4,5 ✗ · Edit 46 % ✗ · contexto/passo 362 mil ✗ · início 63 mil ✓.
