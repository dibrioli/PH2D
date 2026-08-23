# HANDOFF — `line/Sprite` · auditoria do transporte da §11 Animation (2026-08-23)

> **Pedido do Enio:** *«às vezes preciso clicar mais de uma vez para checar Playing. Corrija isso e
> faça auditoria completa do sistema de animação da sprite.»*
>
> **Continuação de:** [`HANDOFF_INTEGRACAO_line_Sprite_MOUNT_2026-08-23.md`](HANDOFF_INTEGRACAO_line_Sprite_MOUNT_2026-08-23.md)
> (mesma linha, mesma jornada). A §11 tinha um dia de vida.
>
> **A auditoria completa, com o mecanismo de cada achado e as recusas medidas, está em
> [`docs/Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md`](../21_auditoria_da_animacao_2026-08-23.md).**
> Este handoff é o registro de integração: o que mudou, onde, e o que o integrador tem de saber.

## §1 — O achado que resume a wave

O report era a ponta de uma família. **Todos os quatro defeitos vivem no mesmo estado** — depois de
uma animação de uma volta chegar ao fim —, e nenhum aparece numa sprite que só o painel tocou:

| # | Defeito | Mecanismo |
|---|---|---|
| F1 | a caixa «Playing» precisava de 2 cliques | **dupla fonte de verdade**: pintava do snapshot, decidia do `WidgetStore`; o motor escreve `playing` sozinho e a semente do sync só corre em aresta de entidade/linha |
| F2 | ligar uma animação terminada era um gesto **morto** | `playing = true` com o contador cheio e a imagem na ponta ⇒ o 1.º passo de `advance` re-fecha o ciclo |
| F3 | «Rewind» não movia a imagem | `rewind` zera contadores que ninguém vê; `advance` só reposiciona um frame **fora** do intervalo |
| F4/F5 | escolher outra animação começava-a a meio, ou deixava a sprite muda | os intervalos **partilham o pool** (a tese do modelo), e `SetCurrent` não tocava no `playing` |

⇒ **Uma lei nova, statable numa linha:** *a reprodução que se ESGOTOU volta ao princípio quando
alguém lhe toca — e escolher outra animação é tocar-lhe. Uma pausa explícita não é tocada.*

## §2 — O que mudou, por ficheiro

| Ficheiro | Mudança |
|---|---|
| `crates/ph2d-ecs/src/sprite_anim.rs` | **+2 API**: `SpriteAnimator::is_finished(tag)` e `entry_frame(animator, tag, cells)`. Nada removido, nada renomeado |
| `crates/ph2d-ecs/src/lib.rs` | re-export de `entry_frame` |
| `crates/ph2d-panel-inspector/src/event_anim.rs` | as duas caixas derivam do **snapshot** (`!info.playing`), não do store |
| `crates/ph2d-panel-inspector/src/sync_sections.rs` | as duas caixas passam a **espelho por quadro** (a exceção à lei das irmãs, documentada no sítio) |
| `crates/ph2d-panel-inspector/src/sections/anim.rs` | a linha de aviso de **seleção múltipla**, antes de qualquer controlo |
| `crates/ph2d-editor-core/src/screens/hero/inspector_model_anim.rs` | **−2 campos** de `InspectorAnimInfo`: `library_present` e `mixed` (calculados, nunca lidos) |
| `shells/desktop/src/render_loop/inspector_anim.rs` | `rewind_to_start` (o rebobinar completo, com o `Sprite::frame`) · `current_tag` · os braços `SetCurrent`/`Playing`/`Rewind` · `build_anim_info` perde o parâmetro `selected` e o clone por quadro |
| `shells/desktop/src/render_loop/snapshots.rs` | a chamada acompanha a assinatura |
| `shells/desktop/src/render_loop/mod.rs` | um comentário que **afirmava o contrário do código** (o `tick` de passo grande) |
| `shells/desktop/src/anim_smoke.rs` | dois passos novos no roteiro (Rewind · re-tocar o `attack`) |

### ⚠️ Superfície tocada FORA do módulo

- **`ph2d-ecs`** — só **acrescenta** duas funções públicas ao `sprite_anim`, que nasceu nesta linha.
  Nenhum contrato congelado (§6) é tocado; nenhum registo de componente mudou (segue em **69**),
  e o `PROJECT_SCHEMA` **não se move** (nenhum layout de componente mudou).
- **`ph2d-editor-core`** — **remove dois campos** de `InspectorAnimInfo`, struct que nasceu nesta
  linha ontem. ⚠️ **Risco de merge:** uma linha concorrente que tenha acrescentado um campo a essa
  struct funde limpo (adições disjuntas), mas uma que tenha passado a **ler** `mixed`/
  `library_present` quebraria a compilação — `git grep` no `main` de 2026-08-23 dá zero
  consumidores fora deste módulo.

## §3 — Gates novos (todos com mutação a sangrar)

**`crates/ph2d-panel-inspector/tests/seam_anim.rs` — 10 gates, ficheiro NOVO.** A §11 tinha 20
gates da lei pura e 13 do commit, e **zero** que carregassem num pixel — o defeito reportado vive
exatamente entre os dois. Irmão do `seam_player.rs`, com `click_at` real.

- `the_playing_box_asks_the_scene_not_its_own_memory` ⭐ **corrido RED-FIRST** (falhou com
  `Playing(false)` onde tinha de mandar `Playing(true)`), verde depois da cura
- `the_autoplay_box_asks_the_scene_not_its_own_memory` (o irmão latente)
- `every_player_control_reaches_the_bus` · `every_library_control_reaches_the_bus`
- `clicking_a_row_picks_what_plays_and_moves_the_editor_with_it`
- `every_edit_the_model_declares_is_reachable_by_a_gesture` — as **18** variantes de
  `AnimFieldEdit`, com `match` exaustivo (uma variante nova **não compila** até ser amostrada)
- `the_empty_face_offers_only_the_gesture_that_creates_the_player`
- `the_library_fields_show_what_was_authored_not_the_seed`
- `a_multiple_selection_says_so_before_offering_any_control`

**`crates/ph2d-ecs/src/sprite_anim_tests.rs` — +2** (`a_finished_animation_is_told_apart_from_a_paused_one`,
`the_entry_cell_is_the_end_the_effective_direction_starts_from`). Total 22.

**`shells/desktop/src/render_loop/inspector_anim_transport_tests.rs` — ficheiro NOVO** (irmão por
HR-18: o pai chegou a **617/600** depois dos gates novos, e o corte é por LEI — transporte aqui,
autoria lá). 4 gates.

**`shells/desktop/src/render_loop/inspector_anim_tests.rs` — +1**
(`the_panel_and_the_engine_agree_on_a_dangling_playback`).

### As 11 mutações

| # | Mutação | Sangrou em |
|---|---|---|
| M1 | `rewind_to_start` volta a ser `p.rewind(tag)` | 3 gates do transporte |
| M2 | tirar o `is_finished` do braço `Playing` | `turning_playing_back_on_replays…` |
| M3 | `Playing(true)` rebobina **sempre** | `turning_playing_back_on_replays…` (a metade da pausa) |
| M4 | o despacho volta a ler `store().checkbox(id)` | `the_playing_box_asks_the_scene…` (**este foi o red-first**) |
| M5 | `is_finished` usa `>` em vez de `>=` | `a_finished_animation_is_told_apart…` |
| M6 | `entry_frame` ignora a direção | `the_entry_cell_is_the_end…` |
| M7 | `SetCurrent` nunca retoma | `choosing_another_animation_resumes…` |
| M8 | `SetCurrent` retoma **sempre** | `choosing_another_animation_resumes…` (a metade da pausa) |
| M9 | o aviso de seleção múltipla desaparece | `a_multiple_selection_says_so…` |
| M10 | o aviso aparece **sempre** | `a_multiple_selection_says_so…` |
| M11 | `current_dangling` esquece o `fits` | `the_panel_and_the_engine_agree…` |

## §4 — ⚠️ O que foi MEDIDO e NÃO curado (decisão do Enio)

**Um quadro de reprodução mexe num componente registado.** O `SpriteAnimator` guarda o relógio
(`elapsed_ticks`/`repeat_count`/`pingpong_reverse`) e está no `ComponentRegistry` ⇒ entra no
`ProjectState`, que é a unidade do undo. Com a animação a tocar, **um quadro que tenha input regista
um passo cujo conteúdo é só o relógio**.

⚠️ **Não é um defeito da §11 — é uma propriedade do undo do app.** A ponte de física escreve o
`Transform` (também registado) a cada passo enquanto o mundo simula. As três saídas possíveis e por
que nenhuma se aplicou daqui estão na [auditoria §4](../21_auditoria_da_animacao_2026-08-23.md).

## §5 — Estado da linha

- **Nada foi integrado, nada foi pushado.** A linha fecha e PARA (§0.7).
- `PROJECT_SCHEMA` **inalterado** por esta wave (segue no valor que o handoff MOUNT registou);
  registos **69 / 70 / 70**; nenhum contrato congelado tocado.
- Recusas medidas desta wave (⛔ não reconstruir sem ler): alcance nos campos da biblioteca ·
  alcance em `from`/`to` · clicar na lista tocar sempre · limpar o `current` ao apagar uma
  animação · dicas de hover. Todas na [auditoria §5](../21_auditoria_da_animacao_2026-08-23.md).
