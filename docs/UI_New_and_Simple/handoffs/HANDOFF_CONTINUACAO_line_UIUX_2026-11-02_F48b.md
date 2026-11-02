# HANDOFF de continuação — line/UIUX, F48b (02/11): o Offset Path no padrão do Inspector

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, HEAD de código `1887fc21d3` (sobre F48 `3a16487f7c`/`878fa728b1`).
**Nada integrado, nada enviado.** Antes de ler: `cd` + `pwd` + `git branch --show-current`. Anterior:
[F48](HANDOFF_CONTINUACAO_line_UIUX_2026-11-02_F48.md).

## 1 — Pedido

Dono, com foto da secção Expand: *«expand não é intuitivo, auto explicativo e provavelmente não funciona perfeitamente.
Veja o layout da ferramenta em outros apps»*; aprovou a proposta com *«faça tudo como no inspector»* (as três escolhas
recomendadas: resultado novo POR TRÁS, distância em px, perfil de largura fora). Comparação e recusas:
[F48_offset_path_como_no_inspector.md](../F48_offset_path_como_no_inspector.md).

## 2 — O que mudou (mecanismo)

- **Painel** (`paint_expand.rs`, `state_expand.rs`, `event_expand.rs`): secção «Offset Path» (id `VECTOR_SECTION_EXPAND`)
  — `Distance (px)` = `world_number_field(VECTOR_EXPAND_OFFSET)`; `Corners` = `icon_segmented` com os glifos do Join;
  `Miter limit` (`VECTOR_EXPAND_MITER`, 1–500) só com Miter; `Holes · Offset too | Keep` (códigos 2/0; o 1 = só furos fica
  para documentos antigos, o painel não o oferece); `Result · New shape behind | Replace`; `Apply` (Accent) e `Remove` só
  com `offset_live_on()`. Estado panel-local = `OffsetKnobs {join, side, miter, behind}` (`EXPAND_KNOBS_AT_BIRTH`: Miter,
  2, 4.0, behind). Os eventos vivem em `event_expand::expand_event` (o `apply_event` estava no tecto de 200).
  Secção nova **Convert** (`VECTOR_SECTION_CONVERT`, no grupo Advanced) com Outline Stroke. Os quatro W + `Apply Power
  Stroke` foram para o fim do `stroke_settings_body` (`power_stroke_rows`), rótulos «Start width / Middle width / End
  width / Middle at»; a fileira de presets do Expand saiu (a do Stroke settings já existia).
- **Modelo**: `VecOffset {miter (serde default 4 = MITER_AT_BIRTH = MITER_LIMIT, gate), behind (default false)}`;
  canais `OFFSET_MITER 213` (contínuo) e `OFFSET_ATRAS 214` (escolhas `e::ATRAS`); `OFFSET_LADO` mostra Keep/Inner/Offset
  too. Marcas: `VECTOR_EXPAND_MITER`, `RESULT_BEHIND/REPLACE`.
- **Motor**: `ph2d_vec_boolean::offset_path_with(.., miter)` (o `offset_path` delega com `MITER_LIMIT`); `Expand::Offset`
  ganhou `miter`.
- **Vivo** (`offset_live.rs`): `arm(spec: VecOffset)`, `retune/choose_knobs(OffsetKnobs)`, `shown_d`, `chip_d` (substituem
  `shown_frac/chip_frac`). `recook` com behind empurra a fonte de mundo DEPOIS do offset (desenha por cima, no z dela).
  `materialise(.., restack)`: Replace como antes; behind mantém a fonte (id, pose, componentes), insere as peças em `z`,
  desarma a fonte e pede `restack [novas…, fonte]` (a ordem de desenho é a da Hierarchy; a forma nova só tem entidade no
  `sync` seguinte — mesma porta do Blend, `fase_vector_tree_settle`). `vec_convert::to_curves` ganhou o mesmo param.
- **Shell** (`fase_live_offset_and_width.rs`): o campo diferente de `vec.offset_shown` = o artista ⇒ `arm` com
  `DocPx::to_world`; chips ⇒ `choose_knobs` com `chip_d` (¼ da meia-dimensão se o campo está a 0); sem foco/arrasto o campo
  mostra `of_world(shown_d)`; publica `set_offset_live_on`. `Remove` zera o campo no painel e esta borda desarma.
- **Alça** (`offset_handle.rs`, novo): modo Select, forma única, secção aberta OU offset vivo; senta a ¼ da altura da
  borda direita (a ½ é a pega de escala do gizmo) + `d`; press/move/up em `despacho_clique_select`/`despacho_mover`/
  `despacho_metodos_modos_e_alcas`, desenho com `draw_width_handle` em `fase_vector_tool_handles`.
- **Atalho** `Ctrl+Alt+O` = clique em `VECTOR_EXPAND_OUTLINE_STROKE` (tabela `KEYS` 22); `file_chords` deixou de agarrar
  `Ctrl+Alt+…` (antes Ctrl+Alt+O abria projecto).
- `Above Effects`: id escrito no `checkbox_row` — o gate `todo_campo_pintado_com_losango_tem_canal_ou_porque` não lê
  variável (falha herdada da F47, apanhada aqui).

## 3 — Provas

Gates novos: panel `f48_offset_path_inspector.rs` (7), app-vec `offset_live_behind_tests.rs` (4), `offset_handle_tests.rs`
(2), `tool_keys_tests::ctrl_alt_o_outlines_the_stroke`, `vector_mode_scenes_tests::a_cena_51_e_o_offset_path`. Mutação
14/14 sangrou (lista no relatório da sessão: limite só no Miter, Result, Remove, perfil no Stroke settings, Inner fora, fonte
por cima, z, desarme, restack, miter no recook e no motor, sinal da alça, atalho, canal 214). Verificador: panel inteiro
(motivo: registo de painel), app-vec/ecs/vec-boolean/i18n/shell por nome, gates de canais, catracas de LOC (painel, ficheiro,
shell, app-vec), HR-15, english, claude_md — verdes; clippy `PH2D_SUITE=fecho` e fmt limpos nas 6 crates.
Catraca app-vec: 3.º lote de cabeçalhos verbatim (−418) para `docs/Vector Module/50_cabecalhos_do_app_vec.md`.

## 4 — Smoke (cena 51, fotos em `img_F48/cena51.png` e `cena51_apply.png`)

`PH2D_OBJECT_MODE_SMOKE=51` (rectângulo com degradê, Offset Path aberto, offset vivo 15 px Miter atrás);
`PH2D_OBJECT_MODE_GESTO=apply` fotografa depois do Apply (Path 1 acima de Path 0 na Hierarchy = desenhado por baixo).
**A cena 50 e a 49 (2.ª ronda) continuam à espera do smoke do dono.**

## 5 — Aberto (por ordem)

1. **Smoke do dono** das cenas 50 e 51 (PARAR até lá).
2. ABERTO 2 da F48: Inner/Outer + «Above Effects» (a faixa vai por baixo dos filtros) — gate vermelho primeiro.
3. Os quatro W do perfil ainda são `slider_row` (slider + chip), não o campo do Inspector; convertê-los mexe em
   `profile_live` (store dos sliders), `populate` e nos gates `every_chip_is_linked_to_its_slider`/
   `the_width_sliders_author_the_live_profile`.
4. Não há menu de contexto do vetor no app (o Outline Stroke ficou na secção Convert + atalho).
5. Herdados da F48: itens vivos desenham camadas dilatadas sem dilatar; alças de largura só no traço de base; `path_at`
   ignora larguras; fronteira de blend do objecto por peça.
