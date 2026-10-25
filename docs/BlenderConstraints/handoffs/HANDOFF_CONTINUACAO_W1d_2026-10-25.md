# Continuação — `line/BlenderConstraints`, onda W1 (4.ª janela → 5.ª), 25/10/2026

> Anteriores: [`W1c`](HANDOFF_CONTINUACAO_W1c_2026-10-25.md) (ajustes por tipo, unidades, gizmo) ·
> [`W1b`](HANDOFF_CONTINUACAO_W1b_2026-10-25.md) · [`W1`](HANDOFF_CONTINUACAO_W1_2026-10-25.md). Plano:
> [`01_plano.md`](../01_plano.md) · ADR
> [`0192`](../../architecture/decisions/0192-a-pilha-de-constraints-e-um-componente-por-entidade-avaliada-da-pose-autorada.md).

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4` (o `main`
não andou: `git log HEAD..main` = 0). HEAD = o commit deste handoff, árvore limpa. Shell **189 018 / 189 041**
(folga 23). Commits da janela: `8e2873b372` (compensação + cena 1) · `5bb1250b59` (bake + schema 194) ·
`4d7de856f7` (8 sobreviventes ganham gate) · `60ac936003` (fmt/clippy, fecho de onda). Binário
`target/smoke/ph2d-host-desktop` a reconferir antes do smoke (`--profile smoke`).

## Smoke do dono (cena 1) — AINDA POR FAZER

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-BlenderConstraints && PH2D_CONSTRAINT_SMOKE=1 ./target/smoke/ph2d-host-desktop`
2. A cena abre parada no quadro 0 com a Espada escolhida, o cartão Constraints aberto (Child Of → Mão,
   Influence 0) e a timeline com Auto Key ligado. Carregar Play (▷ na timeline).
3. A mão desce, pega a espada no quadro 24 sem salto, leva-a, pousa-a no 48 e vai-se. Arrastar o tempo para trás
   e para a frente dá o mesmo.
4. Errado = a espada salta ao pegar/largar, ou já está na mão antes do 24, ou volta ao chão depois do 48.
5. Depois: carregar «Bake» no cartão. O movimento fica igual e a Influence fica 0 em todo o tempo (as chaves da
   Espada passam a ser de posição).

## Feito

| Peça | Onde | Prova (por nome, verde) |
|---|---|---|
| **Chave de compensação do Auto Key** (pegar/largar). `inspector::liga` regista `Pega` (recurso `Pegas`) quando o item passa de inerte a activo ou o contrário (campo FORCA ou LIGADA). `compensar` devolve `Escrita`s: o campo em f−1 com o valor visto antes (`curve_seen` em f−1, senão o valor antigo) com `Interp::Hold`, e em f com o novo; ao largar um Child Of, `Transform` (5 canais) em f−1 = pose autorada (ledger `ConstraintPose` ou curva) e f = pose vista. `ph2d_timeline::curve_seen` passou a pública | `ph2d-app-constraints/src/compensa.rs`, `inspector.rs`; `ph2d-timeline/src/autokey.rs`, `lib.rs` | `pegar_e_largar`: `picking_and_dropping_with_auto_key_never_jumps_and_keeps_the_past`, `one_gesture_is_one_undo_step`, `without_auto_key_the_gesture_keys_nothing_and_is_forgotten` |
| **Escritor único de chaves**: `gravar` faz `upsert_key` direto num passo de undo (cada `AddKey` abriria um passo; precedente: bake da física); tempo/valor pelas portas do Auto Key (`remapped_time`/`key_home`/`key_value_in_active_clip`, `prime_rooted` por tempo). `tudo=true` (bake) recusa o lote se uma chave não tem casa ou se o histórico já está aberto. `do_quadro(tl, sim, preview, playhead, inst, painel, interp)` é chamado em `fase_autokey.rs` ANTES do `autokey_pass::run` (medido no app: depois dele, f−1 lia a chave nova de f — força 1 desde o quadro 0; com f já escrito o Auto Key lê mundo == curva e não abre 2.º passo) | `chaves.rs`; shell `render_loop/fase_autokey.rs` | `one_gesture_is_one_undo_step` · cena 1 |
| **Cura em `avalia.rs`**: com a pilha inerte, se outra mão (a curva) escreveu o autorado neste quadro (`base_local == agora`) não se devolve o autorado do ledger por cima (arrastar 36→48 punha a espada no chão) | `ph2d-constraints-ecs/src/avalia.rs` | sonda da cena 1 (3,01 m do chão no 36) |
| **Cena 1** `PH2D_CONSTRAINT_SMOKE=1`: `smoke.rs::mao_e_espada` (Chão, Mão, Espada com Child Of→Mão a força 0; constantes `MAO`, `QUADRO_PEGA=24`, `QUADRO_LARGA=48`, `ESPADA_NO_CHAO`) + `smoke_pega.rs::step` conduzido (gancho de 1 linha em `fase_object_mode.rs`): grava a curva da mão, liga Auto Key, pega/larga pela porta `apply_all`, visita 10/23/24/36/48/60, abre o cartão. Sonda no app: salto 0,000000 m ao pegar e ao largar; chão no 10 e 23; 3,01 m do chão no 36; largada no 60; chaves da força em [23,24,47,48]; foto conferida | `smoke.rs`, `smoke_pega.rs` | `every_level_up_to_the_router_ceiling_answers` (era o vermelho aberto) |
| **Bake**: botão «Bake» (`INSP_CONSTRAINT_BAKE`, só com itens; i18n `panel.inspector.constraint.bake`) → `ConstraintFieldEdit::Cozer` → recurso `Cozer` → `do_quadro` coze a pilha INTEIRA do dono em `ph2d_timeline::bake_range` (movido da física para a timeline, a física re-exporta; loop armado, senão extensão do doc, senão 5 s). Avalia sobre a pose AUTORADA (`substitute_authored`) com UM ledger de rascunho no laço (um por quadro partia da pose do quadro anterior — vermelho visto no quadro 31) e devolve o mundo (Transforms, pilhas, `CanaisEscritos`). Escreve chaves do Transform em cada quadro inteiro + as que já lá estão (Linear), a última em Hold; força 0 em t0/t1 (Hold) e chaves internas a 0 (`Escrita::Valor`); bordas de−1/ate+1 com a força e a pose autorada de antes; folga 1e-9 no ceil/floor (loop num quadro exacto) | `bake.rs`, `ph2d-timeline/src/bake_range.rs`, `ph2d-app-physics/src/bake.rs`; painel `sections/constraint.rs`, `event_constraint.rs`, `populate_constraint.rs`; `editor-core/src/constraint_edits.rs` | `the_bake_equals_the_live_stack_at_every_frame` (73 quadros, ao bit) · `baking_a_stretch_with_the_stack_active_keeps_every_frame_and_the_outside` · seam `o_bake_pede_cozer_e_so_existe_com_itens`, `o_bake_com_a_pilha_vazia_nao_pede_nada` · testes do bake da física por nome |
| **`PROJECT_SCHEMA` 193 → 194** (Constraints REGISTADO; precedente Skeleton/MotionInput/Macro: um binário v193 descartaria a pilha). O 193 lê-se com o tipo vivo (`PROJECT_SCHEMA_SEM_CONSTRAINTS`); tripla (194, 13, 23). Pago arquivando verbatim o comentário dos braços 97/98 em `docs/archive/project-schema/bracos_97_e_98_recusados.md` | `shells/desktop/src/project_schema.rs`, `project_schema_tests.rs`, `project_load_read.rs` | testes de schema por nome; censos abaixo |
| **Fecho de onda**: `fmt --all`; `clippy -D warnings` limpo nas 8 crates tocadas; contrato congelado: a prova do plano §3 imprime vazio | — | — |

Censos/gates verdes por nome: `seam_constraints`, `architecture_panel_wiring_parity`, `node_id_collisions`,
`every_word_*`, `the_shell_only_shrinks`, `the_shell_links_exactly_the_registered_families`,
`no_two_families_claim_the_same_router`, fase `the_order_is_topological_not_by_bits_either_way`, `o_cartao`
`a_max_written_below_the_min_rises_to_the_min`, `o_gizmo` `dragging_an_arc_edge_across_180_takes_the_short_way`.

## Mutação: 19/19 sangram

A–R da lista do W1c + as novas. 1.ª ronda: 8 sobreviveram, e cada uma ganhou gate provado vermelho (`4d7de856f7`):
**B** degrau de f−1 (lê o instante duas vezes) · **D** `substitute` · **E** força 0 · **F** Hold final ·
**J1** guarda do evento · **M1** ordem por bits crescentes (a premissa do teste antigo não valia; corrigido em
`fase.rs`) · **Q2** Max<Min (`o_cartao`) · **R** arco através de ±180° (`o_gizmo`).

## Para a integração

- O degrau do schema CONTA-SE na integração (`python3 scripts/schema-recount.py`), nunca se herda o 194.
- Superfície de colisão: schema (3 ficheiros), `ph2d-timeline` (`curve_seen` pública, `bake_range` novo),
  `ph2d-app-physics/src/bake.rs` (agora re-exporta), `fase_autokey.rs` e `fase_object_mode.rs` (1 linha), ids do
  painel `inspector_constraint.rs`. Um merge pode partir: a ordem `do_quadro` antes de `autokey_pass::run`; o
  re-export do `bake_range`; a folga da shell (23).
- Suíte COMPLETA ainda não corrida (só por nome, regra do dono 23/10): é do integrador.

## Falta / aberto

1. O smoke do dono desta onda (acima).
2. Feedback do Bake ao artista: a física mostra toast, aqui nenhum; o Bake não pede confirmação.
3. O desvio do Child Of não é animável: pegar de novo mais tarde re-acerta o offset e muda o 1.º trecho (o Blender
   tem o mesmo; solução seria um item por pega).
4. O canal da força chega um quadro depois de um salto de tempo (pré-existente, visto ao medir).
5. O bake avalia só curva→canais→pilha (física/motion não correm no bake).
6. `bind_on_create` com PreviewDrive vazio no meio da pilha (herdado do W1b).
7. O cartão da Luz 2D fixa metros (`sections/luz.rs`) — anotar para essa linha.
8. Próxima onda: W2 ossos (plano §7) — ou fechar a LINHA (`/pd-linha-fechar`) se o dono quiser integrar a W1.

## Armadilhas

- Ordem no quadro: commit do cartão → `fase_autokey`; a pilha só corre no quadro seguinte. Por isso `do_quadro`
  vai ANTES do autokey.
- A régua de textos isenta só ficheiros `smoke*`/`probe*` (`gate::CENAS`): é o nome `smoke_pega.rs` que isenta as
  frases da sonda.
- A mutação D só morre com a pilha a meia força e SEM curva de pose (com curva, o apply escreve a pose e o
  `substitute` é irrelevante).
- Um ledger por quadro no bake parte da pose do quadro anterior: UM ledger de rascunho no laço.
- O guarda recusa `sed -i` e `cargo test` fora de `ph2d-run.sh`; `git mv` dentro de um comando recusado não corre.
- O fish não parte `$files`; o guarda R5 lê `for t in …` como crate inteira (um teste por chamada).
