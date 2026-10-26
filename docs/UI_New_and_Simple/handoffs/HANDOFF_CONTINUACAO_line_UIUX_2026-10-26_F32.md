# HANDOFF de continuação — line/UIUX, onda F32 (26/10) — smoke do dono OK

Ponta `3484bed7ab` + este doc, sobre o main `c3662cd8e4` (o main não andou; **não integrado**). Plano e mecanismo medido de
cada item: [F32_plano.md](../F32_plano.md). Onda anterior: [F31](HANDOFF_CONTINUACAO_line_UIUX_2026-10-25_F31.md).

## 1 — Commits da F32 (`git log --oneline 8477e1511b..HEAD`, do mais antigo)

| commit | o quê |
|---|---|
| `537626b46d` | plano: os seis itens medidos no código |
| `b3f6c278da` | §6 — antes da 1.ª entrada do artista todo quadro quieto é a base do undo (`UndoLinks::houve_entrada`; cobre TODA cena de smoke, o boot e o load, sem a shell crescer) |
| `79b059c936` | §1 — o binding de item que o ficheiro trouxe e já não existe sai no load (`TimelineState::itens_por_decidir`, `purga_itens::decide`, porta `canal_item_vivo::decidido/nomeia_item`); na sessão a lei de 25/10 fica intacta |
| `bf5ff7442b` | §3/§4a — `TimerState::item` (reconcile por id, o slot sem id adopta), `StateMachineRuntime::item` (o `current` segue o estado), `RemoveState` recua o `initial` |
| `9b8d9bb6ad` | §2/§4b — `Tween::relogio`, `PathFollow::relogio_item` (portas `tween::timer_do`, `path_follow::timer_de`; adopção no passe dos ids, no mesmo quadro); `ObjectPose::filter_ids` + `VecFilter::troca_pares`; `PROJECT_SCHEMA` 196→197 (`item_id_v196`, `item_id::degraus`, âmbito `PoseLayoutV196` na leitura E na escrita); o preset do tween já não apaga o `item`; `InspectorTweenRow::relogio_pos` |
| `3fe26dc7d9` | §5 — `.ph2dmacro` antigo: `Inicial::le`/`Evento::le` passam cada componente por `item_id::migra_blob` (sem versão; a forma de agora tem precedência — colisão MEDIDA: dois `Timers` de agora lêem-se por inteiro na forma 194) |
| `260b82ba0e` | fmt + clippy da onda |
| `3484bed7ab` | os vermelhos da suíte dos impactados (abaixo) |

## 2 — Números medidos

- Mutação: **24** mutações aplicadas pelo `muta.sh` (exit 1 = vermelho), todas vermelhas; uma sobrevivente (a precedência da forma
  de agora no `migra_blob`) ganhou o gate `sem_versao_a_forma_de_agora_tem_precedencia` e passou a sangrar.
- Suíte dos impactados (`PH2D_SUITE=onda nextest-impacted`, load 60–80): 22 323 testes, 6 vermelhos. 4 nossos, curados por nome:
  `the_shell_only_shrinks` (189097 > 189041 — cura por corte: escada 194→197 condensada, auxiliares nos testes do ficheiro
  antigo, braço `195..=197`), `todo_campo_de_um_cartao_tem_nome_ou_motivo` (`relogio_item` = `Hidden::Editor`),
  `the_first_baseline_is_settled_too` e `every_word_this_family_shows_comes_from_the_string_table` (estes dois vermelhos desde
  a F31 `5e94574c23`). 2 de carga: `a_cena_3_conduzida_passo_a_passo` verde sozinho; `o_deposito_de_um_pincel_pesado_…`
  vermelho sozinho a load 23 (FLAKES_DE_CARGA.md:83) — `ph2d-tool-painter` não depende de nenhuma crate da onda.
- Testes de projecto da shell (`project`): 120/120. Clippy `--all-targets -D warnings` nas 12 crates tocadas: limpo.
- Binário smoke compilado depois da última edição (`Finished smoke … 53 s`).

## 3 — Superfície de colisão (DELTA para a integração)

- **Formato:** `PROJECT_SCHEMA` **197** (main 193 ⇒ +4 na linha: F30 +2, F31 +1, F32 +1; tripla `(197, 13, 24)`). Recontar:
  `python3 scripts/schema-recount.py`. Os degraus 195..197 estão num bloco só da escada (`project_schema.rs`).
- **Literais que partem noutras linhas:** `TimerState` (+`item`), `StateMachineRuntime` (+`item`), `Tween` (+`relogio`),
  `PathFollow` (+`relogio_item`), `ObjectPose` (+`filter_ids`), `InspectorTweenRow` (+`relogio_pos`). `VecFilter::troca_ops`
  saiu (→ `troca_pares`). `item_id::Passe` ganhou `atual`.
- **Dep nova:** `ph2d-ui-state → ph2d-item-id`.
- **`line/GameEngineFuncional`:** toca `fase_sequences_tests.rs` e `inspector_timer_tests.rs` (aqui +1 linha cada: `item`) e move
  `timer_tick.rs` para `ph2d-app-components`; não toca `timer.rs`/`tween.rs`/`state_machine.rs`/`path_follow.rs`.
- Ficheiros quentes: `ph2d-ecs/src/{item_id.rs, item_id_v196.rs, timer.rs, tween.rs, path_follow.rs, state_machine.rs,
  vec_filter.rs, canal_item_vivo.rs}` · `ph2d-ui-state/src/{pose.rs, pose_wire.rs, transition.rs}` ·
  `ph2d-timeline-persist/src/{lib.rs (600/600), purga_itens.rs}` · `ph2d-editor-core/src/undo_link.rs` ·
  `ph2d-app-macros/src/mundo/formato.rs` · `ph2d-panel-inspector/src/{event_tween.rs, sections/tween_editor.rs}` · shell:
  `project_load_read.rs`, `project_schema*.rs`, `project_itens_tests.rs`, `project_opacidade_tests.rs`.
- Contadores de cena: nenhum (o smoke usou `PH2D_TWEEN_SMOKE=1`, que já existia).

## 4 — Smoke do dono (26/10, OK)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_TWEEN_SMOKE=1 ./target/smoke/ph2d-host-desktop` — o
«Cresce», `x Remove Timer` no 1.º timer: a altura continua no ritmo dela, a largura pára; o Ctrl+Z devolve os dois.

## 5 — Decisões (não reabrir)

- Técnicas: o vivo (timer, máquina) reconcilia-se pela IDENTIDADE do item; o tween/seguidor fixa o timer pelo id no passe dos ids
  (a posição só é a escolha inicial / o painel); um tween cujo timer saiu fica INERTE (não cai noutro relógio); a base do undo é
  o documento até à 1.ª entrada; a purga do item apagado só depois de um LOAD (na sessão o undo global pode devolvê-lo).
- Recusado, com razão: guardar ids nas CHAVES de `from`/`to` de uma seta (o graph editor mostraria ids como números) — fica
  aberto em §6.3. O `SequencePlayer` toma «o primeiro timer»: é um PAPEL, não um item — fica.

## 6 — ABERTO, por ordem

1. ⭐ **Achado do dono no smoke (26/10): filtros × efeitos.** Uma forma com o filtro **Bevel** (+ Drop Shadow) a que se aplica o
   efeito **Flag** ganha artefactos no Bevel — quebras/vincos nas reentrâncias e na borda, o relevo pior resolvido. Fotos:
   [sem o Flag](img_F32/bevel_sem_flag.png) · [com o Flag](img_F32/bevel_com_flag.png). O dono pediu **uma AUDITORIA da
   interacção entre filtros (raster, `VecFilter`/`ph2d-fx-op`) e efeitos (de caminho, `PathEffect`)**, não só a cura deste par:
   medir o mecanismo (o Flag subdivide/curva o caminho e o Bevel lê a distância/normal de uma geometria achatada? resolução da
   máscara do filtro depois da deformação?), varrer todos os pares filtro × efeito com uma régua por par, e curar a família.
2. A pose dos estados de UI guarda os ids dos filtros, mas o MEIO da transição alinha por ÍNDICE (lei de 21/08); reordenar entre
   estados faz um degrau trocar de tipo a meio (entra do neutro) — comportamento antigo, não regressão.
3. Uma chave sobre `from`/`to` de uma seta guarda o número do estado; apagar um estado anterior não a reescreve.
4. Da F31 §6 ainda abertos: 5 (losango da pilha só da forma mostrada), 6.5–6.9 da F30 (`View::Toggle` — decisão do dono;
   faixas sem losango; scrub sem reler a curva; mundo/mesa fora do undo; cor crua nos overlays vetoriais).
5. `CLAUDE.md` §5 conta-se no fecho da linha/integração: `PH2D_OBJECT_MODE_SMOKE=1|2|4|6..39`, `PH2D_AUTOKEY_SMOKE=1..36`.
6. A suíte inteira antes da integração (do integrador). **NÃO integrar nem fazer push sem ordem do Enio.**
