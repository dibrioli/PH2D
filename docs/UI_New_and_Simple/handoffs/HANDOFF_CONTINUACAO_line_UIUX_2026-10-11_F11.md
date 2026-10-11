# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 11/10 — a F11 a meio (janela 1 fechada pela R1)

As regras R1–R9 são as do [handoff da F10](HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F10.md) (topo) —
não copiadas aqui. O prompt da F11: [`16_prompt_F11_…`](../vetor_nova_ui/16_prompt_F11_gizmos_do_illustrator.md).
A lei, o oráculo e as recusas da F11: [`17_gizmos_do_illustrator.md`](../vetor_nova_ui/17_gizmos_do_illustrator.md).

## Onde a linha está

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, merge-base `8f20c60de` (o `main` não andou em
  11/10 até esta janela). F9 + F10 fechadas e NÃO integradas; a F11 em curso, 3 commits:
  `82a97f1e4` (a caixa com o traço + a ④/cartão a 24 px) · `21226cb08` (as caras do Illustrator, menos
  pontos, o contorno e a ficha durante o arrasto) · o deste handoff (doc 17).
- As imagens do Illustrator (lidas a olho, quadros extraídos): `/var/tmp/ph2d_F11_illustrator/` —
  fora do repo. A tabela delas está no doc 17 §1; NÃO repita a pesquisa.

## O que JÁ está feito e verde (testes dirigidos, não a suíte)

- Item 0 (oráculo) e item 2 (a caixa engloba o traço), item 3 (afastar) e o grosso do item 1 —
  doc 17 §1–§4. Corridas por nome: vec-render `visual_bounds` 6/6 · app-vec 310 (caixas) + 50 (alças,
  cenas, cartão) · editor-core 72 (barras/cartão) + 161 (overlays/gizmo) · tool-vector 36 (alças) ·
  vec-render + panel-vector 72. **Nenhum** gate batched ainda; **nenhuma** mutação ainda.
- Foundational tocado: `ph2d-vec-scene` (`VecPath::all_contours_closed`, saiu da booleana),
  `ph2d-editor-core` (`GizmoStateGroup::{outline_only, handle_readout}`, `float_bar::strip_rects_at`,
  `tool_bars_card::selection_gap`), `ph2d-panel-vector` (`nomes_do_motor::campo` passou a `pub`).
  Shell: `fase_vector_tool_handles.rs` −2 linhas (a `draw` recebe o `hero` e o plano).

## O que está ABERTO (por ordem — a próxima janela começa no 1)

1. **O ⊙ do rectângulo no CENTRO do arco** (doc 17 §4 «ABERTO»): hoje fica na aresta, estacionado. O
   Illustrator põe-no dentro do canto (centro do arco; ~10 px na diagonal com raio 0). Desenhar-se ali
   (`handle_grip`, num IRMÃO — `shape_handles.rs` ~680/700 LOC); a pega mede-se ao ponto verdadeiro
   (`press` já o faz), então a lei `Nearest` não muda. Rever o gate
   `a_parked_handle_grabbed_and_held_still_changes_nothing` e a cena 15.
2. **As cenas** (item 4): refazer a foto da `=15` (a pizza com as hastes) e da `=16` (a estrela com
   ⇕ ⊙ ○ ⊙, o cartão agora com a *Curve*); **nova `=17`** (`vector_mode_stroke.rs`, estágios 90+,
   `HANDLES = 3` no roteador `vector_mode.rs` — contar lá): um rectângulo arredondado com traço GROSSO
   seleccionado no modo Forma — a caixa POR FORA do traço, os 4 ⊙, a ④ e o cartão à folga nova.
   Fotografar TODAS (`docs/Components/ferramentas/fotografa_cena.sh PH2D_OBJECT_MODE_SMOKE=17 1930 1040
   …`) e comparar com `/var/tmp/ph2d_F11_illustrator/` ANTES do relatório; guardar em
   `vetor_nova_ui/fotos/f11_*.png` e linkar no doc 17 §6.
3. **O desvio do arrasto de escala** com o traço na caixa (doc 17 §2 «ABERTO»): decidir se a cura
   exacta (o rácio que desconta a margem, pivô na geometria) entra nesta linha — é foundational
   partilhado com o sprite e o Flip. Medir primeiro com um teste do `advance_gizmo_drag` (traço 0,2,
   *Resize Box* ligado, ×2): o canto oposto anda `m·(r−1)`.
4. Fecho (DIRETRIZ §1.5.9): `mutacao` sobre as cláusulas novas (a caneta redonda em `√|det|`, o Outer
   ×2, o Inner, a pilha, `glyph_of`, a haste por fora, o `outline_only` sem hits, a folga da ④) ·
   `verificador` 1× (nextest-impacted + clippy `--all-targets` + `-D warnings` + fmt + shell gates
   `the_shell_only_shrinks`/`the_object_box_condition_names_every_authoring_tool`) · handoff de
   INTEGRAÇÃO novo (cita o da F10) · CLAUDE.md §5.1 «Último:» (a entrada UI/UX tem 699/700 bytes:
   corte texto) · `bash scripts/doc-index.sh` · `rm -rf target/*/incremental` · binário smoke 2× ·
   smoke ao dono em passos numerados.

## Premissas do prompt que a medição mudou

- «O Affinity engloba o traço?» — **não** (doc 17 §1); o Illustrator só com *Use Preview Bounds*.
  Segue-se o pedido do dono.
- «Confira se já existe uma bbox com traço» — existia (`path_bounds_under`), mas CONSERVADORA; a caixa
  pede a exacta (doc 17 §5).
- A posição da ④ do Illustrator (a *Contextual Task Bar*): o produto não a documenta; a folga nova é
  proposta nossa (alça + `Lg`), não medida no oráculo.

## Para a próxima janela

`cd Worktrees/line-UIUX && pwd && git branch --show-current` antes de ler. Leia este handoff e o doc
17 — não o histórico.
