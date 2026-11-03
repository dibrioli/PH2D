# Handoff — linha `TrelloUITeste`, bloco 10: T61 alfinete em todo cartão de componente + T62 Ctrl+Z (02/11/2026)

Linha de TESTE (não integrada); o que sobrevive é `docs/CartoesUI`. Worktree `Worktrees/line-TrelloUITeste`, branch `line/TrelloUITeste`. Nunca houve push. Anterior: [bloco 9](HANDOFF_line_TrelloUITeste_2026-11-02_bloco9.md) (os ABERTOS dele continuam, menos os resolvidos abaixo).

**Commits:** `b8cd05e105` (código T61 + Ctrl+Z, cena 6/7, foto) · `0489fdd4c4` (docs 01/03/04) · este handoff — base `399115dda5`.

**Smoke do dono: OK em 02/11** (cena `PH2D_VEC_CARTOES_SMOKE=6`: prender o Platform Player ao Player A, escolher o B, editar o preso, Ctrl+Z).

## Feito (detalhe, medições e contraprovas: bloco 10 de [04](../04_plano_de_construcao.md); decisões T61/T62 em [01](../01_decisoes_do_dono.md); mapa em [03](../03_mapa_do_codigo.md))

| item | mecanismo | teste |
|---|---|---|
| T61 alfinete em todo cartão de componente | retrato por objecto (`ph2d_panel_inspector::retrato`; `com_cada_preso` na shell em 3 fases: `snapshots::publish`, `fase_snapshots_publish`, `fase_macros`; `snapshots_inspector::Alvo`, `publish_global`) · ensaio das caixas (`cards_presos::ensaia` + `WidgetStore::abre_ensaio/fecha_ensaio`, diff por id de `InteractiveState` e cor, troca à volta da pintura do solto em `cards_solto::pinta_soltos`) · gesto com o retrato do preso (`event::apply_event` → `com_retrato`) · cache pela chave (retrato do preso, da seleção, unidade) | `o_bloco9_reset_e_alfinete::a_pinned_component_card_shows_and_edits_its_own_object` |
| T62 Ctrl+Z re-semeia a caixa | `sync::semeia_tudo` a cada quadro fora da troca, saltando as vivas (foco, arrasto, botão) e com pedido no barramento; sai o `resync` do T60 | `an_undo_on_a_component_field_reseeds_its_box`, `every_component_card_has_a_reset_that_asks_for_its_component` |

Custo: por preso ≈ 90 µs/quadro (cena 7, smoke); sem a cache o ensaio e a montagem custavam 997 e 607 µs. Shell −24 linhas, ph2d-app-vec −5. Foto `maquete/t61_alfinete_antes_depois.png`.

## ABERTO

1. **Observação do dono no smoke (02/11), palavra dele:** «se num player coloque o theme da seção Physic Player no canvas em vermelho, ao selecionar outro e arrastar a mesma seção para o canvas, o card muda de cor sozinho.» Família: tema/notas das cópias soltas (T54 — tema e notas são por id de SECÇÃO, partilhados entre a cópia 1 e a 2; ver `ph2d-editor-core` `section_prefs_ops`/`cards_estilos`/tema por secção e `loose_instance_of`). Medir primeiro: qual dos dois cartões muda (o preso ou o novo) e porquê; a cura é uma lei para tema E notas das cópias (por instância), nunca só a cor. Se houver escolha de PRODUTO (a cópia nova nasce com o tema da original ou o de fábrica?), perguntar ao dono numa linha.
2. T29 2.ª metade (caixas de marcar, listas suspensas, `property_label_row`, textos à mão) — contagens no handoff dos blocos 3–7.
3. Vermelhos ANTIGOS (não desta linha): `ph2d-panel-inspector --test it` seam_* (134 nesta rodada, faixa 134–139), `node_id_collisions::every_non_literal_hash_is_named`.
4. Antigo, não tratado: um solto colado ao topo do canvas tem o cabeçalho sob a barra de modos («Rail Backdrop» regista-se depois) — o alfinete não se clica ali.

## Regras aprendidas

- O registo dos painéis é do PROCESSO (`PANEL_REGISTRY`, Mutex): o `InspectorState` é partilhado entre testes paralelos; um teste que dependa de «o objecto não trocou» (foco) só julga os quadros em que o foco sobreviveu.
- Fixture com sprite desliga o Inspector em cartões (`cards::active` exige `current_inspector_sprite() == None`): `arma_tudo` arma sprite — desarme-a.
- Uma foto «antes» só vale com a cura INTEIRA desligada (pintura + ensaio): metade desligada mostrou os números do preso pelo overlay e a foto mentia.
- Meça o custo por quadro com instrumentação temporária marcada (`// T61-MEDE`, env) e tire-a antes do commit; com o retrato comparável (PartialEq) a cache é barata.
