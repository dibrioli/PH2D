# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-11 — F11: OS GIZMOS DAS FORMAS COMO O ILLUSTRATOR, A CAIXA COM O TRAÇO, A ④ E O CARTÃO MAIS LONGE

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Jornada anterior: [F10, 11/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-11_F10_PARAMETROS.md) (que cita
> a da F9) — também NÃO integrada: esta linha leva F9 + F10 + F11 (F0–F8 já estão no `main`).
> Lei, oráculo (o Illustrator pelas fotos), decisões e recusas:
> [`17_gizmos_do_illustrator.md`](../vetor_nova_ui/17_gizmos_do_illustrator.md). Prompt:
> [`16_prompt_F11_…`](../vetor_nova_ui/16_prompt_F11_gizmos_do_illustrator.md).

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `8f20c60de` (o `main` não andou — sem rebase).
- Commits da F11: `4d3e4f076..HEAD`, em duas janelas (a 1.ª fechou pela R1:
  [continuação](HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F11.md)).
- **O pedido (dono, 11/10):** *«Ainda não gostei dos nossos gizmos. Se for necessário simplifique.
  Veja como faz o Illustrator.»* · *«O gizmo de transformação… não engloba o stroke. Faça englobar.»* ·
  *«O modal das formas e a barra superior abrem muito próximos das formas. Afaste um pouco.»*

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

UI/UX: «Último:» → este ficheiro; «as alças das formas no modo Forma» → «no idioma do Illustrator»;
smoke `6..16` → `6..17`; «rolagem única, » saiu para caber nos 700 bytes.

## §1 — O que mudou

| commit | o quê |
|---|---|
| `82a97f1e4` | a caixa de transformação = a caixa do que a forma PINTA (`ph2d_vec_render::visual_bounds`: o traço expandido pela kurbo sob a caneta redonda `w·√|det|`, Outer/Inner, a pilha, as pontas de seta); a MESMA porta para a caixa, a ④, o cartão, o enquadrar e o arrasto de escala; a ④ e o cartão a `selection_gap()` = alça + `Spacing::Lg` (24 px) |
| `21226cb08` | as caras do Illustrator (`shape_handles_face::glyph_of`: ⊙ canto · ⇕ contagem · ⊙ na haste dos ângulos · ○ raio); menos pontos no canvas (envergadura, furo, raio interno da espiral → cartão); durante o arrasto: a caixa só contorno (`outline_only`, sem hits), a ficha do número (`handle_readout`), o círculo tracejado do raio |
| `4d466ed93` | o ⊙ do rectângulo DENTRO do canto, no centro do arco (`HandleDesc::in_corner`, `spots::corner_centre`); com raio zero `park_at` em cada eixo; o ponto verdadeiro fica no início do arco — a lei `Nearest` não muda |
| `3e63b35dc` | a cena `=17` (`vector_mode_stroke`, estágios 90–93): rectângulo arredondado de traço 20 px pelo campo *Width*, o ⊙ na diagonal; fotos novas da 15, 16, 17 (doc 17 §6) |
| `381496192` | o desvio do arrasto de escala com o traço na caixa, MEDIDO (doc 17 §2) + o gate da lei exacta no proporcional |

## §2 — Medido e decidido (técnico, delegado)

- **Oráculo:** o Illustrator pelas fotos (helpx/Learn/Tuts+, fora do repo em `/var/tmp/ph2d_F11_illustrator/`)
  — tabela no doc 17 §1. O Affinity (o produto do dono) NÃO engloba o traço; o Illustrator só com
  *Use Preview Bounds*. Segue-se o pedido do dono.
- **O arrasto de escala com o traço na caixa:** ⛔ a premissa «o *Resize Box* fixa o traço» era FALSA
  (a largura escala por `√|det|` em `path_bake_xform`). Sonda, ×2: proporcional EXACTO (6e-6); só em X
  as duas bordas desviam `m·(sx − √(sx·sy))` = 0,0586 com traço 0,2. Não se cura nesta linha: é a lei
  da caneta redonda, em dois caminhos (a moldura na shell e a escala livre no `compute_gizmo_transform`
  partilhado com sprite/Flip). Pergunta de produto ao dono no §7.
- **A cena 17 ensinava o contrário** na 1.ª foto: o traço escrito por baixo do painel mostrava *Width 3*
  sobre 20 px desenhados. Agora vai pelo evento do campo e pelo valor que o campo guarda.

## §3 — Foundational tocado e contratos

Nenhum contrato congelado (§6 do `CLAUDE.md`). Apenso, sem quebrar chamadores:
`ph2d-vec-scene` (`VecPath::all_contours_closed`), `ph2d-vec-render` (`visual_bounds`:
`path_visual_bounds`, `path_world_visual_bbox`, `path_intrinsic_visual_bbox`), `ph2d-editor-core`
(`GizmoStateGroup::{outline_only, handle_readout}`, `float_bar::strip_rects_at`,
`tool_bars_card::selection_gap`), `ph2d-panel-vector` (`nomes_do_motor::campo` passou a `pub`),
`ph2d-tool-vector` (`HandleDesc::in_corner` — campo novo: quem construir um `HandleDesc` literal fora
da tabela tem de o pôr; hoje só `shape_handles_table.rs`). Shell: `fase_vector_tool_handles.rs` −2;
`vec_frame_resize_tests.rs` +1 gate (~28 linhas — a catraca `the_shell_only_shrinks` conta testes).

## §4 — O que só a árvore combinada pega

- `HandleDesc` ganhou um campo: uma linha paralela que acrescente alças por literal não compila até o pôr.
- A caixa visual mudou o número que a ④/cartão/enquadrar leem para TODO caminho vetorial com traço —
  gates de outras linhas que fixem a posição da ④ em px sobre uma forma com traço deslocam ½w×zoom.

## §5 — Provas de mutação

14 mutações (agente `mutacao`, controlos verdes e > 0 testes em cada filtro, âncora 1× antes de
mutar, `touch` depois de restaurar): **14/14 sangram** — 13 à 1.ª, 1 curada com teste.

| id | cláusula | mutação | quem sangra |
|---|---|---|---|
| M1 | `corner_centre`: cada perna ≥ `park_at` | sem estacionar | `a_corner_handle_sits_inside_the_corner_at_the_arc_centre` · `a_parked_handle_grabbed_and_held_still_changes_nothing` |
| M2 | `corner_centre`: as DUAS pernas | só uma | os mesmos dois |
| M3 | `handle_grip`: `in_corner` | o bloco apagado (⊙ volta à aresta) | os mesmos dois |
| M4 | a haste por fora do aro | `(r + park)/r` → para dentro | `an_angle_handle_lives_outside_the_rim_off_the_box_handles` |
| M5 | `glyph_of` | canto ⇒ cara de raio | `each_handle_wears_the_face_of_what_it_does` |
| M6a | a caneta redonda `√|det|` | factor 1 | **sobrevivia** no vec-render (o teste usava `(2, 0,5)`, `|det| = 1`) → curada em `5db01b782` (`(2, 1)`); já sangrava em `the_gizmo_box_wraps_the_stroke_…` |
| M6b | idem | factor = eixo máximo | `under_unequal_scale_the_pen_stays_round` + o do app-vec |
| M7 | Outer = a faixa `2w` | Outer como Centre | `outer_reaches_the_whole_width_and_inner_stays_inside` |
| M8 | Inner fica dentro | Inner pinta fora | o mesmo |
| M9 | a pilha inteira | só a 1.ª marca | 4 testes do `visual_bounds` |
| M10 | `outline_only` sem hits | as alças voltam a registar | `an_outline_only_box_registers_nothing_and_the_handle_readout_paints` |
| M11 | `selection_gap` = alça + `Lg` | sem o `Lg` | `the_action_bar_and_the_card_float_one_handle_plus_lg_off_the_selection` + 1 |
| M12 | a caixa (e o pivô) VISUAL | `anchor_half` pela caixa da curva | `the_gizmo_box_wraps_the_stroke_…` · `a_proportional_resize_with_the_stroke_in_the_box_keeps_both_visual_edges` |

## §6 — Smoke (do dono)

O relatório ao dono (passos numerados) está na resposta final da janela; as cenas `=15`, `=16`, `=17`
e as fotos no doc 17 §6.

## §7 — O que fica ABERTO

1. **Pergunta ao dono (produto):** a moldura (*Resize Box*) deve manter a ESPESSURA do traço ao
   redimensionar (o *Scale Strokes & Effects* do Illustrator, desligado por omissão)? Hoje a espessura
   escala. Com ela fixa o desvio numa direcção vira `m·(r − 1)` e cura-se só na moldura (doc 17 §2).
2. O balão rectangular (`SPEECH_RECT`) ficou com o canto na aresta (o canto roda com o bico; sem foto).
3. Herdado da F10: as perguntas da posição do cartão, *Inverter/Fechar* da pizza, o canto POR canto.

## §8 — Portão, binário e `agent-loop-profile.sh`

- **Portão batched** (agente `verificador`, 1× sobre `8f20c60de..HEAD`, loadavg 10–40):
  `nextest-impacted` **19 581 / 19 584** — os 3 vermelhos eram da F11 e foram curados em `50eb54f1d`
  (re-corridos por nome, verdes): `no_tofu_glyphs` (uma seta «→» num teste da shell), e duas agulhas
  de texto da shell que a 1.ª janela envelheceu — `the_render_loop_draws_the_shape_handles_by_the_mode_policy`
  (o `if overlay.shape_handles {` virou o argumento `shown` do `shape_handles_live::draw`) e
  `the_paint_pass_draws_the_point_gizmo_last` (o rustfmt partiu `paint_sprite_gizmo_with(scene`).
  Clippy do `ship.sh` (`--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings`): 3 avisos
  curados, depois **exit 0**. `fmt --all --check` ✓. `the_shell_only_shrinks` ✓ ·
  `the_object_box_condition_names_every_authoring_tool` ✓ · `workspace_src_files_under_loc_cap` ✓ ·
  `architecture_claude_md_cabe_no_orcamento` 3/3 ✓ (a entrada UI/UX: 691/700 bytes).
- `bash scripts/doc-index.sh` ✓ · `rm -rf target/*/incremental` (13 G + 3,7 G).
- **Binário de smoke** (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`),
  2.ª corrida depois da limpeza, HEAD `50eb54f1d`:
  `Finished \`smoke\` profile [optimized] target(s) in 0.22s` (zero `Compiling`).
- `agent-loop-profile.sh` (20 sessões): paralelismo 1,16 ✗ · respostas/sessão 144 ✓ · test:check
  2,6 ✗ · Edit 50 % ✗ · contexto/passo 348 mil ✗ · início 63 mil ✓.
