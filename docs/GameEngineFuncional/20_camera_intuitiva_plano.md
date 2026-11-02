# 20 — A câmera intuitiva (plano, 02/11)

Leitor: a próxima LLM. Pedido do dono, 02/11: *«ver se a câmera ficou o mais perfeitamente intuitiva antes de seguir»*
→ auditoria por fotos (`PH2D_GAME_SMOKE=4` + `PH2D_GAME_SMOKE_FOTO=camera|camera_limites|camera_segue|camera_olhar`)
→ 7 achados → *«Vamos corrigir tudo»*. Decisões do dono:
[memória](../../project-memory/project_camera_intuitive_owner_decisions_2026_11_02.md).

## Decisões do dono (02/11)

- **Alvo do Follow:** escrever o nome (já existe) · **arrastar da Hierarchy** para o campo · **Eyedropper** (botão ao lado
  do campo, depois clicar no objecto no canvas). ⛔ **Lista/dropdown de objectos: recusada** (cenas com muitíssimos
  objectos).
- **Zona morta no MEIO da moldura** (idioma Godot/Unity): a caixa é da tela da câmera e o alvo anda dentro dela.
  Revoga `centro_da_zona = mira` (cura 17b).
- **Gizmo rico como o das shapes do Vector** (`ph2d-tool-vector/src/shape_handles*.rs`, `ph2d-vec-render/src/shape_nodes.rs`).

## Itens

| # | Achado | Cura | Gate (vermelho se voltar) |
|---|---|---|---|
| C1 | Campos da câmera sem dica (o cartão Player tem) | `set_tooltip` em toda fileira de Game Camera, Follow, Limits e Game Format, Plain English | censo: todo id registado em `populate_camera.rs` tem dica não-vazia |
| C2 | Alvo escrito à mão | `DropTarget::CameraTarget` (Hierarchy → nome) no molde do `ScriptObject`; botão Eyedropper no molde do `AlvoArmado` das constraints, Esc cancela, a própria câmera não se escolhe | seam: largar um `Object` no campo escreve o nome; armar + clique no canvas escreve o nome do de cima; Esc desarma |
| C3 | Zona morta em volta do alvo | `centro_da_zona` = centro da moldura sem o offset da vista | presa a um limite, o centro da caixa = centro da moldura |
| C4 | Três rectângulos de quadradinhos iguais | caras por função (zoom = quadrado; limites = cantos em L + barras nas bordas + cruz de mover; zona morta = círculos), halo sob o ponteiro, etiqueta do nome+valor no ponteiro (hover e arrasto), etiquetas «Limits»/«Dead Zone» nas caixas, Esc cancela o arrasto, duplo-clique repõe, Shift encaixa | por alça: cara ≠ das outras famílias; readout por alça; Esc devolve os valores do `Down`; duplo-clique escreve o padrão |
| C5 | Rótulo cortado («nera · …») e «follows Player» sob o boneco | rótulos dentro da área útil do canvas (fora das réguas); «follows X» no meio do fio, com fundo | rótulo dentro do canvas útil em `Look Through` |
| C6 | Limits nascem ±10 m | nascem no contorno do nível (união das caixas dos objectos de mundo, sem fundo em parallax); duplo-clique numa alça dos limites repõe esse contorno | limites novos = contorno do nível 2 |
| C7 | Cena 4 abre com o jogador fora da vista | a vista do editor abre enquadrando o jogador | foto |

Ordem: C3 · C5 · C7 (pequenos) → C1 → C6 → C2 → C4. Fotos de cada estado antes do smoke do dono.

## Estado

| # | Estado |
|---|---|
| C1 | feito — `populate_camera::dicas` (23 grupos) · gate `toda_fileira_da_camera_tem_dica` (varre o PINTADO) |
| C2 | feito — `DropTarget::CameraTarget`/`DropAction::SetCameraTarget` (nome) · Eyedropper `INSP_CAMERA_TARGET_PICK` → `CameraFieldEdit::PickTarget` → `CameraCanvasView::escolhe_alvo`; o clique no canvas entra pelo guarda do `joint_body_pick` (sprites sob o cursor, `ph2d_camera_frame::escolhe_alvo`); Esc desarma; cursor Crosshair · gates `o_conta_gotas_do_alvo_arma_e_desarma`, `o_conta_gotas_escolhe_o_de_cima_que_nao_e_a_camera`, `largar_um_objecto_no_alvo_da_camera`. ⚠️ Só sprites são escolhíveis pelo conta-gotas (o idioma das juntas/constraints); vetor/Flip pelo arrastar ou pelo nome |
| C3 | feito — `centro_da_zona = centro − offset` · gate `a_zona_morta_pinta_se_no_meio_da_moldura_e_a_borda_escreve_a_fraccao` |
| C4 | feito — `Cara` por família (quadrado · círculo · L · barra · losango) com contorno escuro, halo sob o ponteiro, ficha `etiqueta` no ponteiro (não no miolo sem arrasto), fichas «Limits»/«Dead Zone»; `Arrasto::restaura` (Esc), `padrao` + `UltimoPress` (duplo-clique), `encaixa` (Shift = passo do Inspector) · gates `o_esc_o_duplo_clique_e_o_shift_das_alcas`, `o_duplo_clique_conta_se_por_alca`, `cada_alca_tem_a_sua_etiqueta` |
| C5 | feito — `tinta::canvas_util` (réguas fora) · «follows X» numa ficha no meio do fio · gate `o_canvas_util_tira_as_reguas` |
| C6 | feito — `contorno_do_nivel` (imagens visíveis com `ScrollFactor` 1) + `semeia_limites` na tabela de sementes da paleta de componentes; duplo-clique numa alça dos limites repõe-no · gate `os_limites_nascem_no_contorno_do_nivel` (nível 2 = `[-13, -1]..[17, 7]`) |
| C7 | feito — `SetViewFocus::Selected` na montagem da cena 2/4 (foto) |
