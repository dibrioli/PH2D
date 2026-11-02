# HANDOFF de continuação — line/UIUX, F48c+F48d (02/11): «In front» e o Offset Path como FERRAMENTA

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, HEAD de código `28b3ac6255` (F48d) sobre `fa1283eec8` (F48c).
**Nada integrado, nada enviado.** Anterior: [F48b](HANDOFF_CONTINUACAO_line_UIUX_2026-11-02_F48b.md) (o mecanismo do
offset vivo, px, Miter limit, Result, alça e Convert está lá). Decisão do dono em memória:
`project-memory/project_offset_path_is_a_toolbar_tool_2026_11_02.md`.

## 1 — Pedido

Smoke da cena 51 (F48b) **OK**; o dono pediu: *«melhor transformar numa tools na barra de tools»* e *«acrescentar a opção
de desenhar pela frente, não só por trás»*.

## 2 — Mecanismo

- **F48c:** `VecOffset.behind:bool` → `result:u8` (`RESULT_REPLACE 0 · RESULT_BEHIND 1 · RESULT_FRONT 2`, serde default 0;
  o campo nunca saiu da linha). `recook`: à frente a fonte entra PRIMEIRO na `LiveGeometry` (por baixo); `materialise`
  insere em `z+1` e pede `restack [fonte, novas…]`. Canal 214 com três escolhas (`e::ATRAS`).
- **F48d:** `DrawMode::Offset` (`VECTOR_MODE_OFFSET`, `vector-offset.svg`, KEYLESS, depois do Width). Fila ② =
  `bars_row::offset_chips(store)`: `Distance (px)`, `Corners` (ícones), `Miter limit` (só Miter), e duas LISTAS
  `VECTOR_BAR_HOLES`/`VECTOR_BAR_RESULT` (`bars_control.rs`: `HOLES`/`RESULT`, `offset_lists`, popovers em
  `paint_bar_popovers`, dono do fecho em `close_list_on_pick`) cujas linhas são os ids de sempre ⇒ `event_expand`
  inalterado. Apply/Remove na ④ (`bars_actions.rs`) só com offset vivo; Offset é `PRECISE`. Gesto:
  `offset_handle::press_tool` (alça ou qualquer forma sob o cursor, selecciona) + `drag` `d = d0 + (x − x_press)`, ganchos
  à maneira do Width em `despacho_clique_vetor_premido/solto`, desenho em `fase_vector_tool_handles`; Select sem ganchos.
  `overlay.edit` desligado no Offset. Secção do Inspector removida (`VECTOR_SECTIONS` 32). A secção **Convert** fica.
- **editor-core:** `size_row_pulldowns` soma `key_mark_room(id)` (o losango comia a face: `PROFILE Un…`).

## 3 — Provas

Gates: `f48_offset_path_tool.rs` (panel, barras), `offset_handle_tests` (press em qualquer ponto + lei do delta, alça só na
ferramenta), `in_front_draws_the_offset_over_the_source_and_applies_above_it`, `um_pulldown_com_canal_mede_a_face_e_o_losango`
(vermelho com a cura tirada, verde reposta), palette gate com as duas listas. Verificador: tool-vector, panel-vector (lib+it),
app-vec, editor-core (lib + gates de canais/LOC/HR-15/english/contrato/claude_md), registry-init, shell — verdes; fmt/clippy
fecho limpos. ⚠️ Mutação em lote da F48d NÃO corrida (só a do `key_mark_room`); a da F48b foi 14/14.

## 4 — Smoke

`PH2D_OBJECT_MODE_SMOKE=51` (ferramenta na mão, 15 px vivo atrás); `GESTO=apply` / `GESTO=result` (lista aberta). Fotos
`img_F48/cena51_tool.png`, `cena51_tool_apply.png`, `cena51_tool_result.png`. O arrasto real no app não foi exercitado (o
script de foto não arrasta) — é o primeiro passo do smoke do dono.

## 5 — Aberto

1. Smoke do dono da F48c/F48d (PARAR). Depois: prova de mutação da F48d.
2. ABERTO 2 da F48: Inner/Outer + «Above Effects».
3. Os quatro W do perfil ainda são `slider_row`.
4. `cargo fmt --all --check` acusa `ph2d-panel-pixel-preview/src/paint.rs:95` e `ph2d-vec-entities/src/connector_handles.rs:142`
   — não são desta linha (para o integrador).
