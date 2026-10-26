# HANDOFF de CONTINUAÇÃO — line/UIUX, onda F31 (25/10): a identidade das camadas do Vetor

Base: `main` = `c3662cd8e4` (não andou). Ponta: `783887fa0f`, **48 commits** à frente (F30 + F31). A linha NÃO integrou.
Plano: [F31_plano.md](../F31_plano.md). Smoke do dono: **OK** (25/10, cena 36, depois da auditoria).

## 1 — Commits da F31 (`git log --oneline 36c29f1e5e..HEAD`, do mais antigo)

| commit | o quê |
|---|---|
| `6f8b350c19` | C — apagar/inserir/reordenar um CONTORNO re-chaveia a forma inteira (`canais_topologia::propagar`, casamento por âncoras iguais bit a bit; `mudou()` vê o nº de contornos) |
| `f13baefd2e` | A — `FieldKey { familia, campo, i, item: ItemId, j }`; `FieldKey::do_item`; `grupo()` com item; `DOC_VERSION` 23→24, a <24 lê-se por `canal_chave_v23::with_field_key_layout_v23`; fixture `timeline_doc_v23.bin` |
| `d1eba3b742` | A — migração da chave posicional antiga → id no `upkeep` (`ph2d-timeline-persist/src/migra_vetor.rs`, reconhecida PELA FORMA, sem bandeira); `TimelineDoc::rechaveia`; nome da linha com a posição |
| `727fc9a529` | B2 — `StableIdCounter::aloca_item` público |
| `3881e9c48b` | B2 — `VecFilter` com `ops`/`ids` privados (só `push/insert/remove/move_*`, `troca_ops`); `PROJECT_SCHEMA` 195→196 (`vec_filter_v195.rs`); `canais_filtros` pela chave durável |
| `b5802e44a1` | B1 — `PaintEntry.id`/`FxEntry.id`; `VEC_SCENE_SCHEMA` 23→24 (`pilha_wire.rs`, `PilhaLayoutV23`); `ids_da_pilha::passe` (atribui + reconcilia o contador com a cena + publica `IdsDoVetor`), chamado em `main.rs` antes do `post_frame_undo` e na `fase_autokey`; `canais_campos` `interno`/`duravel`; `marcas::Contexto.ids` |
| `e7c0034f6a` | o plano |
| `38f522cfd5` | cena `PH2D_AUTOKEY_SMOKE=36` (`autokey_smoke_camadas.rs`) + fotos `img_F31/` |
| `57eb76cc4d`, `f7d6d63ff1` | fmt/clippy |
| `5e94574c23` | os 5 vermelhos da suíte, por CORTE: `fase_filter_commands` → `ph2d_app_vec::fx_live_comandos`; `canal_vetor_lista.rs`; `vida_arena_receitas.rs`; `read_project_file` e `fase_vector_overlays` ≤ 200; o teste do roxo compara sem o `item` (ids desde a F30) |
| `09d0903928` | auditoria — a TOCAR, o quadro que repete o instante não vira documento: `ph2d_ecs::canal_conducao::ConducaoDosCanais` (Vetor/Flip/Física), `a_tocar(bool)` dito pelo `fase_timeline_drain` |
| `dec6259691` | auditoria — a camada apagada leva a linha (decisão do dono): `TargetBinding.item_missing` ⇒ missing, escondida, NÃO apagada; revive quando o undo devolve o id. Porta `ph2d_ecs::canal_item_vivo` (lê `IdsDoVetor` e `PosicoesDosItens`, que ganhou o conjunto das entidades publicadas). Vale para a LISTA da F30 |
| `783887fa0f` | auditoria — `UndoLinks::rearmar`/`tira_a_base`: a cena autokey que devolve linha no `fase_object_mode` re-arma a base do undo no próximo quadro quieto; `TimelineHistory` com hora por passo (`ph2d_board_model::undo_tick`, **dep nova**) e o Ctrl+Z da timeline só se `mais_novo_que` o global |

## 2 — Números medidos

- Mutação: **13/13** gates da onda sangraram (tabela no relatório do agente; âncoras: `duravel` Tinta/Efeito, `degrau()`, casamento de contornos, `migra_vetor::duravel`, âmbitos v23 do doc/cena/projecto, reconcile/dedupe da pilha e dos filtros, `canal_do_campo`, `i_antigo`, `grupo`). Auditoria: cada cura vista VERMELHA antes (`canais_tests.rs:150`, `canais_camada_tests.rs:149`, `canais_corpo_tests.rs:220`, `item_apagado_tests.rs:69` ×3, `history.rs:211`).
- Undo a tocar (medido no app, antes): **120 passos em ~7 s**, `neste_quadro == 0` nos quadros de `t` repetido.
- Suíte (nextest, fecho da onda, antes da auditoria): 22 297 testes, 5 vermelhos — curados por nome em `5e94574c23`. `ph2d-timeline` inteira depois da dep nova: **585/585**. Tectos por nome verdes: `the_shell_only_shrinks`, `workspace_src_files_under_loc_cap`, `shell_files_respect_hr18_loc_cap`, `shell_functions_respect_their_ceiling`; `ph2d-host-desktop undo` 74/74.
- Binário smoke compilado depois da última edição (`Finished … 0.25s`).

## 3 — Superfície de colisão (DELTA para a integração)

- **Formato:** `PROJECT_SCHEMA` **196** (main 193 ⇒ +3 na linha: F30 +2, F31 +1; tripla `(196, 13, 24)`); `DOC_VERSION` 23→**24**; `VEC_SCENE_SCHEMA` 23→**24**. Reconte com `python3 scripts/schema-recount.py`.
- **`FieldKey` ganhou um campo** (`item`): literais `FieldKey { .. }` de outras linhas partem em compilação; `CanaisLidos::esquecer` usa MIN/MAX completos. `PaintEntry`/`FxEntry` ganharam `id` (literais partem). `VecFilter.ops` ficou PRIVADO (`.ops` de outras linhas parte: use `ops()`/`ops_mut()`/`push`).
- **Dep nova:** `ph2d-timeline → ph2d-board-model` (o relógio `undo_tick`).
- `PosicoesDosItens` é agora `(BTreeMap, BTreeSet<u64>)`.
- Ficheiros quentes: `ph2d-ecs/src/{canal.rs, canal_vetor*.rs, canal_chave_v23.rs, canal_conducao.rs, canal_item_vivo.rs, vec_filter*.rs, item_id.rs, stable_id.rs}` · `ph2d-timeline/src/{doc.rs, doc_rechave.rs, history.rs, apply.rs, binding.rs, persist.rs}` · `ph2d-timeline-persist/src/{lib.rs, migra_vetor.rs}` · `ph2d-vec-scene/src/{paint_stack.rs, effect.rs, pilha_wire.rs, schema.rs, lib.rs}` · `ph2d-app-vec/src/{canais*.rs, ids_da_pilha.rs, marcas.rs, fx_live_comandos.rs, autokey_smoke_camadas.rs}` · shell: `main.rs`, `project_load_read.rs`, `project_schema*.rs`, `undo_app.rs`, `undo_app_macros.rs`, `keyboard_timeline.rs`, `fase_object_mode.rs`, `fase_timeline_drain.rs`, `fase_filter_commands.rs` (encolheu).
- `line/GameEngineFuncional` e `line/Vector` mexem em `ph2d-app-vec`/`ph2d-ecs` (ver F30 §3).
- Contadores: `PH2D_AUTOKEY_SMOKE` +1 (`..36`).

## 4 — Smoke do dono (feito, OK)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=36 ./target/smoke/ph2d-host-desktop` — apagar o Fill (a linha some, o Stroke continua), Ctrl+Z (Fill e animação voltam), apagar o Glow e Ctrl+Z, apagar o contorno de cima do Path 1 em Edit Mode (o de baixo continua).

## 5 — Decisões (não reabrir)

- Dono 25/10: **apagar uma camada animada leva a linha da timeline; o Desfazer devolve as duas** (o After Effects). Mecanismo: a lei do objecto apagado (missing, guardado), não uma purga no undo próprio da timeline.
- Técnicas: `ItemId` u32 na `FieldKey` (opção c da F30 §A2); leitura de formatos antigos por âmbito thread-local (precedente `with_stroke_layout_v22`); ids no MESMO quadro da edição, antes da captura do undo.

## 6 — ABERTO, por ordem

1. O binding de item apagado **nunca se purga** (fica escondido no ficheiro depois de fechar/reabrir, quando já nenhum undo o pode devolver). Cura natural: purgar no load os `item_missing` depois de o espelho estar publicado.
2. A pose dos estados de UI (`ObjectPose.filters`) continua POR POSIÇÃO: um filtro só de um estado nasce com id novo a cada entrada; reordenar entre estados troca ids (`ui_state_edit.rs` usa `troca_ops`).
3. `.ph2dmacro` gravado antes com `VecFilter`/componentes de lista não migra (o `MacroDoc` tem versão própria) — junta-se ao F30 §6.4.
4. O re-arme da base do undo cobre as cenas que passam pelo `fase_object_mode` (as `PH2D_AUTOKEY_SMOKE`); as `PH2D_OBJECT_MODE_SMOKE`/outras famílias de cena que montam documento sem entrada não foram varridas. A prova ponta-a-ponta do re-arme é só o smoke (o App sem janela não tem VecScene).
5. Seleção múltipla: o losango da pilha mostra só a forma que o painel mostra (com a chave por id, a entrada `k` de duas formas são canais diferentes).
6. Da F30 §6, ainda abertos: 2 (`TimerRuntime` por comprimento, tween↔Timers por posição — código da `line/GameEngineFuncional`), 3 (setas da StateMachine por índice), 5 (`View::Toggle` invertido — decisão do dono), 6 (sem losango: faixas da mesa, painel do esqueleto, matriz de camadas da física), 7 (o scrub re-simula sem reler a curva por passo), 8 (mundo e mesa fora do undo), 9 (cor crua nos overlays vetoriais).
7. `CLAUDE.md` §5 conta-se no fecho da linha/integração: `PH2D_OBJECT_MODE_SMOKE=1|2|4|6..39`, `PH2D_AUTOKEY_SMOKE=1..36`; trocar o link «Último» da UI/UX pelo handoff de integração.
8. A suíte inteira antes da integração (do integrador). `ph2d-panel-painter-layers` NÃO é desta linha.
9. **NÃO integrar nem fazer push sem ordem do Enio.**
