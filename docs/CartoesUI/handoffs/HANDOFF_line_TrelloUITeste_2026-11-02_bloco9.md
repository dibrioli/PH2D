# Handoff — linha `TrelloUITeste`, bloco 9: T60 reset nos cartões de componente (02/11/2026)

Linha de TESTE (não integrada); o que sobrevive é `docs/CartoesUI`. Worktree `Worktrees/line-TrelloUITeste`, branch `line/TrelloUITeste`. Nunca houve push. Anterior: [bloco 8](HANDOFF_line_TrelloUITeste_2026-11-02_bloco8.md) (os ABERTOS dele continuam, menos os resolvidos abaixo).

**Commits:** `60db04fc7e` (T60) · `359216545f` (docs) · `a14c0acc2d` (re-semeadura, cura do 1.º smoke) · este handoff — base `d80ea556b1`.

**Smoke do dono: OK em 02/11** (cena `PH2D_VEC_CARTOES_SMOKE=6`; o 1.º smoke falhou: «o reset não funcionou» — curado em `a14c0acc2d`).

## Feito (detalhe, medições e contraprovas: bloco 9 de [04](../04_plano_de_construcao.md); decisão T60 em [01](../01_decisoes_do_dono.md))

| item | mecanismo | teste |
|---|---|---|
| T60 ↺ em todo cartão de componente | `cards_componentes::{reset_of, repoe, prepara, reset_click}` → `WidgetStore::request_component_reset` → shell `fase_component_palette` → `component_attach::reset_by_name` (componente principal de `reveal::SECTION_COMPONENT` + linhas `row_of` em cadeia, depois `attach_by_name`) | `component_attach::tests::resetting_the_{body_puts_body_shape_and_zones_back_as_attached, player_leaves_the_body_it_needs}`, `o_bloco9_reset_e_alfinete::every_component_card_has_a_reset_that_asks_for_its_component` |
| 1.º smoke: a caixa ficava com o número antigo | as caixas dos componentes só se semeiam com `entity_changed`; `take_component_resets` arma `CardLists::resync = 2`, o `sync` consome-o (`take_component_resync`) | o mesmo `o_bloco9…` (a caixa «digitada» volta ao da ficha) |

Shell −12 linhas. Foto `maquete/t60_reset_antes_depois.png`.

## ABERTO

1. **T61 — alfinete nos cartões de componente: MEDIDO, não construído** (meio-feito mostraria os números da seleção num cartão «preso»). Desenho em três peças e custo a medir: bloco 9 de [04](../04_plano_de_construcao.md) (shell: retrato por objecto em vez da seleção, ~45 leituras em 5 publicadores; painel: `Retrato` genérico dos ~50 `current_inspector_*` na pintura e no gesto; valores: diff do `sync` por id, generalizar `com_valores` a `InteractiveState`). Mapa em [03](../03_mapa_do_codigo.md) (Bloco 9).
2. **Ctrl+Z de um campo do Player não re-semeia a caixa** (antigo, o mesmo guarda `entity_changed`): não confirmado pelo dono; medir. Família provável: toda secção semeada só na troca (o Joint espelha a cada quadro e não tem o defeito) — a cura deve ser única (espelhar a cada quadro saltando foco/arrasto, como `sync_joint_fields`), nunca por campo.
3. Vermelho ANTIGO novo na lista (igual com o código de HEAD): `ph2d-panel-inspector --test it` `seam_constraints::the_value_cards_follow_dropdown_writes_the_chosen_property`. Os antigos do bloco 8 continuam.
4. T29 2.ª metade e tema/notas das cópias soltas continuam (contagens no handoff dos blocos 3–7).

## Regras aprendidas

- O `fotografa_cena.sh` RECOMPILA a árvore: a foto «antes» tira-se desligando a cura (e repondo), não com o binário antigo.
- Uma escrita no mundo que o Inspector deve mostrar sem trocar de objecto precisa de re-semear as caixas: os testes do painel que só leem o pedido não o veem — estenda o teste até ao número na caixa.
- Semente de solto do Inspector: `loose.<secção>=<INSP_PANEL 6892305470752409708>,x,y,w` (ids = FNV-1a 64 do nome do `hash_node_id`).
