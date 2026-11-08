# Continuação: opção B (toda forma é um objeto filho «Collision Shape») (`line/GameEngineFuncional02`), 08/11/2026

Leitor: o próximo agente da linha. Plano: [`../componentes/f2_contorno_e_formas_plano.md`](../componentes/f2_contorno_e_formas_plano.md) (seções «Decisões do dono», «REVISÃO 08/11 … opção B», «Respostas do dono à B»). Integração: [`HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-08_FISICA.md`](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-08_FISICA.md).

## Ritual de abertura

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02` · `pwd` · `git branch --show-current` (tem de dar `line/GameEngineFuncional02`) · `git status --short` · `git log --oneline 652b506b58..HEAD | head -20`. Depois este handoff, depois o plano f2.

## ⛔ Duas coisas que não se misturam

1. **Integração desta noite (ordem do dono) = branch `line/GameEngineFuncional02-ponto`** (`175d7debe2` = `b24dc55400` + cherry-pick de `bac19f4e23`). É a física em cartões, sem a opção B. Não mexa neste branch.
2. **Tudo o que veio depois de `b24dc55400` na linha é opção B (schema 220) e NÃO está integrado.** Só entra numa integração futura, depois do smoke do dono.

## Decisões do dono 08/11 (resumo; a fonte é o plano f2)

Opção B:
- Toda forma de colisão é um **objeto filho «Collision Shape»**, invisível.
- O corpo mostra só o cartão **Body**; as definições da forma vivem só no Inspector do filho.
- **Add ▸ Collision Shape:** nada selecionado → cria «Static Body» + filho (o filho fica selecionado); imagem sem corpo selecionada → a imagem vira Static + filho; corpo selecionado → filho dele.
- **Alças da forma só com a PRÓPRIA forma selecionada;** com o corpo selecionado o canvas mostra só o contorno.

Outline polygon (contorno): um contorno por Animated Sprite (união dos quadros); côncavo sólido; buracos ficam buracos; padrões **Alpha cutoff 0,5** e **Detail 2 px**; o Auto Key grava Detail e Alpha. (Memória do dono: `project-memory/project_outline_polygon_owner_decisions_2026_11_08.md` no checkout PRIMÁRIO `/home/enio/Documentos/Projetos/PH2D`, ainda por comitar lá; não está no worktree.)

Smokes do dono 08/11: fecho e emendas 1–4 OK; fecho 2 (aviso corpo sem forma, lista da aba como o editor de animações, nome inteiro ao passar a mão) OK; W3-A Add Collision Shape passos 2–3 OK; **W3-B lista «Shapes» REVOGADA** (removida em `c40927c0dd`).

## Feito na opção B (`git log --oneline 652b506b58..HEAD`)

- **W1** crate `ph2d-outline-trace` (`b83680070d`); oráculo Godot em `docs/GameEngineFuncional/oraculo_contorno/` (`cb0065c8ac`).
- **B1 motor:** `af721f6960` `main_shape` (a forma principal é a 1.ª pela ordem da árvore); `1722b33775` forma principal em filha + gate de equivalência `crates/ph2d-physics-ecs/tests/it/forma_filha.rs`; `4446ad504b` `shape_pose` por libm.
- **B2 migração, schema 219 → 220:** `ea4476e22a` `migra_formas_para_filhos` / `LADO_FORMA` em `crates/ph2d-physics-ecs/src/migra_formas.rs`; `018698763c` trilhas do timeline seguem a filha; `cbf8aba897` schema + load; `4aaa7cdcf9` teste movido para `crates/ph2d-physics-scene/src/ficha_migra_tests.rs` e notas da escada arquivadas em `docs/archive/project-schema/escada_v168_a_v181.md`; `f9e1756738` fmt.
- **B3a nascimentos:** `9b07186ea0` `nasce_com_forma`; `ef834629e2` sinal do sensor filho grita pelo corpo dono; `af68cecfcc` sai `own_body_note`; `9b24d488df` spawns + gate `nenhuma_porta_do_editor_poe_a_forma_no_corpo` (`shells/game-scenes/tests/it/jogo_object_add.rs`); `222ccdae65` cortes de LOC.
- `bac19f4e23` dois vermelhos da linha corrigidos (já no `-ponto`). `a93cc31c62` `torna_corpo` (Add Body / Make Independent sem forma no corpo; **ainda sem chamadores**).

## EM CURSO agora (outro agente): B3b

Os canais da forma passam a escrever em `main_shape`; patch de trabalho em `docs/GameEngineFuncional/handoffs/b3b_wip_add_rewire.patch` (Add→`torna_corpo`, Make Independent, extensão do gate); reescrita dos testes da lei antiga; teste de undo do Make Independent; macros resolvem `main_shape`. **Confira o `git log` e `git status`; se a B3b não fechou, retome por aqui** (não duplique: veja se há árvore suja antes de editar).

## Ondas que faltam

- **B4 Inspector:** corpo = só o cartão Body; filho = cartão Shape completo + Collision Layers + «Shape of: {corpo}»; o aviso corpo-sem-forma fica; botão «Move shape to a child» para forma carregada pelo próprio corpo (feita por script).
- **B5 canvas:** alças só com a forma selecionada; corpo selecionado mostra só o contorno.
- **B6 zona/rotação:** força de zona e rotação em formas filhas (`effector.rs` usa a rotação do corpo; âncoras de nav/snap usam só a translação); eliminar a alocação por quadro de `ordered_children` em `main_shape`.
- **B7 cenas de smoke:** `ph2d-physics-scene` `ficha.rs` (cena 120), `ph2d-app-game` `smoke*.rs`, `factory_smoke.rs`, `nav_smoke_lava.rs`, smoke 8 dos macros — todas via `nasce_com_forma`; fotografar antes de mandar ao dono; smoke dele.
- **ADR-0199** (texto por escrever; último existente: 0198 — confirme em `docs/architecture/decisions/README.md` antes de numerar).
- **Outline:** W0 (medir o custo por peça) → W2 (`ColliderOutline` no FILHO, schema 221) → W4 (cartão) → W5 (pontos no canvas) → W6.

## Processo e armadilhas

- Subagente tem 45 min IMPOSTOS: briefs estreitos. Tectos de LOC com folga ~0 em `ph2d-app-physics`, `ph2d-app-game` e shell: ponha a lógica em `ph2d-physics-ecs` e pague por cortes.
- Nunca partilhe `CARGO_TARGET_DIR` entre worktrees. Inner loop `bash scripts/cargo-check-narrow.sh <crate>`; teste por nome com `cargo-test-narrow.sh`; suíte completa só na integração.
- Flake de carga conhecido: `measure_player_budget::the_cost_of_a_player_is_linear_in_their_number`.
- Árvore: o `.log` em `fotos_fisica/fecho2/` está por rastrear e é alheio a este handoff.
- O smoke é do dono, em passos numerados e sem jargão; cena com a ferramenta de fotos antes de enviar.

## Prompt de continuação (cole numa janela nova)

```
Continue a linha line/GameEngineFuncional02 (opção B: toda forma de colisão é um objeto filho «Collision Shape»).
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && pwd && git branch --show-current
2. Leia docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_2026-11-08_OPCAO_B.md e depois docs/GameEngineFuncional/componentes/f2_contorno_e_formas_plano.md.
3. git log --oneline 652b506b58..HEAD e git status: a B3b (formas escrevem em main_shape, Add/Make Independent via torna_corpo) estava em curso por outro agente; se não fechou, retome por ela.
4. Depois B4 (Inspector), B5 (canvas), B6 (zona/rotação), B7 (cenas de smoke + fotos), ADR-0199, e as ondas do Outline.
5. A integração desta noite usa o branch line/GameEngineFuncional02-ponto: não o toque. Não faça push nem integre.
6. Subagentes com brief estreito (45 min); teste por nome; suíte completa só na integração.
```

## Atualização (fim da janela, 08/11): onde a B3b parou

- **Feito, `dcbf5fcc45` (B3c):** funil único — `ph2d_physics_ecs::dono_do_campo`/`corpo_da_forma`; `PhysicsFieldEdit::e_da_forma()` (28 variantes); `apply_physics_edit` redireciona campo de forma no corpo para `main_shape`; `build_physics_info` lê o `Collider` e o lado-forma da forma principal (⚠️ o corpo com filha mostra hoje a face CHEIA — a B4 refaz); `canais_corpo` publica campos de forma sob os bits da filha; shell `timeline_bridge_keys.rs::chave_da_k` (+1). Teste `a_chave_da_forma_no_corpo_vai_para_a_filha`. Verdes por nome: `canais_` 33, `alcas`, `inspector`, `physics_tests`.
- **Por fazer, nesta ordem:** (1) `git apply` do patch `docs/GameEngineFuncional/handoffs/b3b_wip_add_rewire.patch` (aplica limpo no HEAD; Add→`torna_corpo`, Make Independent, extensão do gate) e reescrever os **20** testes de lei velha do `ph2d-app-physics` (lista: canais_corpo `donos::escrever_pela_fisica…`, `a_peca_publica_so_a_face_dela`, `a_forma_decide_as_medidas`, `cada_campo_mostrado_le_o_que_escreveu`; joint_rig `the_collider_of_a_generated_body_matches_the_sprite_box`; alcas `a_caixa_puxada_passa_a_custom_e_o_fit_volta`, `forma_e_offset_chegam_juntos…`, `o_valor_do_down_devolve_forma_e_offset_juntos`; physics_tests `combine_rule_read_modify_writes…`, `editing_one_field_preserves_the_rest`, `switching_shape_keeps_the_footprint`, `every_presence_marker_attaches_detaches…`, `the_area_buoyancy…`, `the_force_zone…`, `the_area_drag…`, `the_added_collider_is_boxed_to_the_sprite`; f1 `density_fica_guardada…`, `fit_to_picture_mantem…`, `fit_to_picture_volta…`, `undoing_remove_brings_body_and_shape_back`) para lerem de `main_shape(body)` — nunca apagar cobertura; depois os testes de gesto da shell e `jogo_object_add`. ⚠️ Se o patch se perder do scratchpad: o conteúdo é `Add`→`torna_corpo` em `physics_apply.rs`, o teste de Make Independent em `shells/desktop/src/physics/physics_gesture_tests.rs` e o gate estendido a Add Body / Make Independent / Rig. (2) teste do undo do Make Independent. (3) macros resolvem `main_shape` ao tocar. (4) Auto Key com só o corpo selecionado não compara os campos de forma publicados pela filha (só o K/losango redireciona). (5) mutações; gates de fecho; fmt + clippy.
