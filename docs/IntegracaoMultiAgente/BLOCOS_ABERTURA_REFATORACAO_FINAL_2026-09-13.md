# `line/input-dispatch` + `line/render-bodies` + `line/loc-caps` — as três linhas que fecham a refatoração

> **De onde vêm:** depois da integração de 13/09 (enviada em `4b170862b`), a auditoria de
> arquitectura tem 10 dos 11 achados fechados e a A10 medida e adiada de propósito
> ([§6](AUDITORIA_ARQUITETURA_2026-09-12.md)). O que sobra é **tamanho**: as listas numeradas de
> tolerância. O dono pediu *«3 linhas em paralelo para resolver tudo»*.
>
> **Medido no `main` de 13/09 (`4b170862b`):**
>
> | lista | tecto | entradas | a maior |
> |---|---:|---:|---|
> | `shells/desktop/tests/it/fn_loc_caps.rs` `FN_OVERAGE_OK` | 200 L/função | **31** | `input_dispatch.rs::on_mouse_input` 3 102 |
> | `shells/desktop/tests/it/file_loc_caps.rs` `FILE_OVERAGE_OK` | 600 L/ficheiro | **8** | `input_dispatch.rs` 7 117 |
> | `crates/ph2d-editor-core/tests/it/architecture_workspace_file_loc_cap.rs` | 700 L/ficheiro | **16** | `ph2d-render/src/compressed_pipeline.rs` 993 |
> | `crates/ph2d-editor-core/tests/it/architecture_panel_loc_cap.rs` `FN_OVERAGE_OK` | 200 L/função | **6** | `ph2d-panel-hierarchy/src/paint.rs::paint_hierarchy_body` 252 |
>
> ⭐ **O alvo das três é a MESMA frase: as linhas do seu território saem destas listas.** Uma entrada
> que fica exige medição e a razão escrita ao lado dela (uma TABELA que é um `match` de cenas pode
> ser mais legível inteira) — ⛔ nunca por comodidade, e nunca uma entrada nova.
>
> ⛔ **NENHUMA das três muda produto:** nenhum pixel, texto, ordem de quadro ou comportamento. O
> smoke do dono é repetir o que já usa e ver tudo igual.

---

## §1 — Os territórios, e porque não se tocam

| linha | ficheiros DONOS (só ela edita) | entradas que ela zera |
|---|---|---|
| `line/input-dispatch` | `shells/desktop/src/input_dispatch.rs` · `shells/desktop/src/input_dispatch/**` · `shells/desktop/src/input_handlers.rs` | `on_mouse_input` 3 102 · `advance_gizmo_drag` 708 · `key_input` 544 · `on_cursor_moved` 375 · `handle_editor_key` 358 · ficheiros `input_dispatch.rs` 7 117 · `input_dispatch/gizmo_drag.rs` 816 |
| `line/render-bodies` | `shells/desktop/src/render_loop/**` · `shells/desktop/src/hero_intents/**` · `shells/desktop/tests/it/frame_text.rs` | `fase_bus_drain` 2 401 · `snapshots::publish` 1 068 · `run_render_frame` 984 · `sim_extract::run` 491 · `run_present_phase` 484 · `image_edit::dispatch` 483 · `drain_merge_sprites` 439 · `hierarchy::dispatch` 414 · `inspector_commits::dispatch` 383 · `fase_audio_panels` 373 · `bgremoval_preview::dispatch` 333 · `apply_samples` 317 · `probe_push_render_and_look` 314 · `drain_reparent` 297 · `drain_bgremoval` 237 · `fase_snapshots_publish` 216 · `fase_vector_bands` 209 · `drain_equalize_sizes` 204 · ficheiros `fase_bus_drain.rs` 2 654 · `render_loop/mod.rs` 1 604 · `snapshots.rs` 1 398 · `sim_extract.rs` 1 155 |
| `line/loc-caps` | `shells/desktop/src/{init,project_load,build_smoke,build_smoke_router,envelope_smoke,blend_smoke,layout_live,main,app_state}.rs` · os 16 ficheiros do `architecture_workspace_file_loc_cap.rs` · os 6 do `architecture_panel_loc_cap.rs` | `build_initial_state` 499 · `project_load_from` 484 · `route` 441 · `build_smoke` 416 · `App::new` 263 · `envelope_smoke::frame` 247 · `lay_out` 206 · `blend_smoke` 205 · ficheiros `app_state.rs` 1 551 · `main.rs` 1 289 · as 16 + 6 das crates |

**Porque não colidem:** os `hero_intents` são chamados SÓ de `render_loop/` (`image_edit.rs`,
`hierarchy.rs`, `fase_hierarchy_group_merge.rs`, `fase_sprite_precision_emissive.rs`,
`fase_vector_tree_settle.rs`) e dos testes deles — vão com a `render-bodies`. `git branch --no-merged
main` não devolve nenhuma `line/*`: as três nascem sem vizinhos.

## §2 — ⛔⛔ AS CERCAS (LEI nas três)

1. **Ficheiro dono é de UMA linha.** Precisa de mudar o de outra? Não mude: escreva no handoff.
2. **Nenhuma linha acrescenta `mod` ao `main.rs` nem campo à `App`.** Módulos novos nascem
   declarados DENTRO do território (`input_dispatch.rs` · `render_loop/mod.rs` ·
   `hero_intents/mod.rs`). A `loc-caps` corta o `app_state.rs` e o `main.rs` **sem renomear nem
   reagrupar campos da `App`** — as outras duas compilam contra `self.<campo>`.
3. **As quatro listas numeradas são partilhadas:** cada linha BAIXA ou APAGA só as linhas do
   próprio território, sem reordenar. O conflito textual é do integrador (o valor certo é o medido
   na árvore combinada).
4. **Um gate que leia os territórios de duas linhas** (ex.: `the_highlight_has_one_source.rs` lê o
   quadro E o `input_dispatch.rs`): cada linha edita só as agulhas do SEU território.
5. **`TETO_LOC` = 196 990 não se toca.** A shell mede **193 205** (folga 3 785) e a folga é
   repartida — cada linha mede o SEU delta sobre a base e fica dentro da quota:

   | linha | quota de linhas NOVAS na `shells/desktop` |
   |---|---:|
   | `input-dispatch` | 1 200 |
   | `render-bodies` | 1 500 |
   | `loc-caps` | 800 |
   | margem do integrador | 285 |

   ⚠️ Referência medida: a divisão do quadro custou ~24 linhas por fase (assinaturas + gates
   re-apontados). Quota a acabar = um corpo é de FAMÍLIA e sai para a crate dela (HOWTO §4). ⛔ Nunca
   suba o número.
6. **API pública das crates que a `loc-caps` corta NÃO muda de caminho** — a `render-bodies` usa o
   `ph2d-render`, a `input-dispatch` usa painéis. Primeiro saem testes (`#[cfg(test)] mod tests` →
   `#[path = "<x>_tests.rs"]`, o padrão da casa) e ajudantes privados; um item público que tenha de
   mudar de ficheiro mantém o caminho (`x.rs` → `x/mod.rs` + irmãos; um `pub use` DENTRO da mesma
   crate para preservar o caminho é aceite, entre crates nunca).

---

## §3 — BLOCO `line/input-dispatch` (cole numa janela nova)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA — Modo L · REFATORAÇÃO FINAL: CLIQUES E TECLADO
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/input-dispatch.  Alvo: o que decide o que
cada clique, arrasto, roda e tecla fazem deixa de ser meia dúzia de
funções gigantes. As entradas do SEU território saem das listas
numeradas de tamanho (tecto 200 L por função, 600 por ficheiro):
  input_dispatch.rs::on_mouse_input      3 102
  input_dispatch/gizmo_drag.rs::advance_gizmo_drag   708
  input_dispatch/keyboard.rs::key_input   544
  input_dispatch.rs::on_cursor_moved      375
  input_handlers.rs::handle_editor_key    358
  ficheiros: input_dispatch.rs 7 117 · input_dispatch/gizmo_drag.rs 816
⛔ ESTA LINHA NÃO MUDA PRODUTO: nenhum pixel, texto ou comportamento, e a
   ORDEM em que os consumidores reclamam um evento é o contrato. Uma cura
   que peça mudança visível ou de ordem → PARE e reporte ao Enio.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. bash scripts/hw-profile.sh          # tem de dizer `workstation`
2. cd /home/enio/Documentos/Projetos/PH2D && git status -sb
      → RAIZ, em main. M/?? em project-memory/ são ALHEIOS: não toque.
3. mkdir -p Worktrees
   git worktree add -b line/input-dispatch Worktrees/line-input-dispatch main
4. cd Worktrees/line-input-dispatch
   pwd && git branch --show-current    # DEVE dizer line/input-dispatch
5. cargo check -p ph2d-host-desktop    # warm-up; o 1º build é frio
6. bash scripts/mergiraf-setup.sh      # idempotente; ✗ não é bloqueio
7. A BASE DA PROVA, antes de editar uma linha:
   mkdir -p target/prova
   cargo nextest list --workspace --cargo-profile ci-test > target/prova/antes.txt
   wc -l target/prova/antes.txt        # ⚠️ vazio = a prova mede nada
   bash -c 'find shells/desktop -name "*.rs" -not -path "*/target/*" -exec cat {} + | wc -l' > target/prova/shell_antes.txt
8. OS INSTRUMENTOS DA PROVA DE MOVIMENTO (não versionados):
   mkdir -p .cauda-input-dispatch
   cp /home/enio/Documentos/Projetos/PH2D/Worktrees/line-render-loop/.cauda-render-loop/{verbatim.py,extract_phase.py,mutlib.sh,prescan_gates.py} .cauda-input-dispatch/
      → ⛔ NUNCA os commite (git add só por caminho). Faltou algum? Leia
        o §4 do handoff da line/render-loop e escreva o seu equivalente.
9. LEIA INTEIRO (dentro da worktree):
   a) docs/IntegracaoMultiAgente/BLOCOS_ABERTURA_REFATORACAO_FINAL_2026-09-13.md
      — §1 territórios e §2 CERCAS (LEI).
   b) docs/IntegracaoMultiAgente/HANDOFF_INTEGRACAO_line_render_loop_2026-09-13.md
      — §4 (o MÉTODO: fases pela mesma ordem, prova verbatim, gates
        re-apontados ANTES de extrair) e §10–§12.
   c) docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md §2
      (as ⛔ são MUDAS) e §3 (a prova).
   d) docs/IntegracaoMultiAgente/DIRETRIZ.md §0, §1.5, §6.7
   e) docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md — tudo, e
      releia a cada passo.
10. Reporte "linha input-dispatch pronta" e SIGA (não pare).

A OBRA — MEDIDO em 13/09 (RE-MEÇA na base; a 1.ª tabela do handoff é a sua):
  on_mouse_input 3386–6487, 106 `return;`, secções EM ORDEM:
    3387 soltura (física, pose, FK, âncora) · 3417 arrasto da biblioteca ·
    3461 aperto no canvas larga o foco do painel · 3472 sculpt3d/field3d ·
    3506 alça do gizmo de âncora · 3513 locais kind/mapped_button/evt ·
    3545 editor de áudio · 3618 menu_open_before/eyedropper/on_canvas ·
    3671 preview de UI · 3683–3935 Flip + costura do dock + guias +
    gizmos de warp/field · 3939 modais do Select vetorial · 4017 esqueleto ·
    4059 física (picks, poke, joint draw) · 4138 alças de textpath/
    patternpath/motion path · 4246–5195 FERRAMENTA VETORIAL fora do Select
    (950 L; um `match` de 4 braços, o de canvas com 517 L em `if` por modo)
    · 5203 forward_to_hero · 5220 fill/Input Map/onion · 5244 `match
    (button, kind)` dos reclamantes ordenados · 5375–6399 BLOCO DO GIZMO E
    DO PICK (1 025 L) · 6405 pan do botão do meio · 6416 barra lateral,
    paleta legada, dispatch_panel_pointer.
  ⚠️ OS EMPRÉSTIMOS que decidem o corte:
    · 5375–6399: UM `if let Some(gfx)=self.gfx.as_mut() && let Some(hero)=…`
      atravessa o bloco inteiro, com ZERO `self.metodo(` dentro (o comentário
      6190–6194 diz porquê). Mova-o INTEIRO para um método; corte o interior
      depois, passando gfx/hero e os campos disjuntos como PARÂMETROS.
    · os locais kind, mapped_button, evt, on_canvas, menu_open_before,
      eyedropper_armed_before são lidos até ~5338: uma secção extraída
      recebe-os como parâmetros e devolve «consumido?» no lugar do `return;`.
    · a soltura (3392) e o blur (3461) TÊM de ficar antes do 1.º return.
  on_cursor_moved 2912–3286 (19 passos, 39 returns, sem empréstimo longo) ·
  on_mouse_wheel 3288–3384 · keyboard.rs::key_input · gizmo_drag.rs ·
  input_handlers.rs::handle_editor_key — mesma cura.
  O MOLDE: métodos `impl App` em ficheiros IRMÃOS sob input_dispatch/,
    chamados PELA MESMA ORDEM; on_mouse_input fica o ÍNDICE. ⛔ Nada de
    `Vec<Box<dyn Handler>>`, trait de reclamante ou registo de callbacks.

⛔⛔ OS GATES QUE LEEM O TEXTO — re-aponte-os ANTES de extrair, um a um,
  e prove que cada um AINDA reprova (mutação):
  · 15 testes fazem include_str!("../../src/input_dispatch.rs"): o
    ficheiro TEM de continuar a existir (fica o índice).
  · O INSTRUMENTO: shells/desktop/tests/it/input_text.rs, gémeo do
    frame_text.rs — o corpo de on_mouse_input com cada chamada
    `self.<prefixo>_*(` substituída pelo corpo dela, recursivo, marcada
    ⟦nome⟧ ⇒ posição no texto = ordem de execução. Com os auto-testes do
    frame_text: o texto chega ao fim · nenhuma peça órfã · peça chamada e
    não achada reprova alto (should_panic) · chamada partida pelo rustfmt.
  · ENDEREÇO + ORDEM sobre o corpo inteiro (partem mesmo com os nomes
    vivos): the_grab_is_wired_to_the_pointer (o on_mouse_input_body vai
    da assinatura ao FIM DO FICHEIRO) · the_pose_is_wired_to_the_pointer ·
    um_aperto_no_canvas_larga_o_teclado_do_painel · the_node_ops_are_wired
    (diz que on_mouse_input é o ÚLTIMO fn do impl) ·
    the_preview_owns_the_pointer_and_the_undo · joint_draw_gesture (agulha
    num COMENTÁRIO) · src/layout_scroll_gesture_tests.rs.
  · ORDEM por posição (33 no total): the_bone_pickers_are_modal (3) ·
    the_pencil_owns_its_whole_gesture (3) · the_width_tool_owns_its_gesture
    (3) · the_motion_path_anchor_is_drawn_and_dragged (2) ·
    the_patternpath_handles_… · the_textpath_handle_… · the_warp_gizmo_… ·
    the_input_map_window_can_be_moved · the_node_selection_scale_is_wired ·
    the_node_press_freezes_a_live_shape_recipe · the_bucket_tool_owns_… ·
    the_trim_tool_owns_… · the_skeleton_panel_only_opens_… · pulley_wheel_
    handles · the_sculpt_delete_says_why_it_refused.
  · PRESENÇA: ~20 (the_highlight_has_one_source conta por ficheiro com
    ("src/input_dispatch.rs",2) — edite SÓ essa agulha, §2.4 do bloco).
  · LEI (independentes do sítio, só confirme): arch_no_absolute_drag_pattern
    · the_key_blocks_ask_whether_the_keys_are_live (varre keyboard*.rs por
    prefixo — ⚠️ um ficheiro novo do teclado com outro prefixo SAI dela) ·
    the_camera_pan_has_exactly_one_door · no_downcast_to_concrete_tool_…
  · ⚠️ Gates de FAMÍLIA que leem a shell pelo caminho moram FORA dela:
    bash -c 'git grep -n "shells/desktop/src" -- crates tools'  (três
    reprovaram na ponta da line/render-loop sem ninguém ver).
  · ⚠️ keyboard.rs tem #[path] para keyboard_tail.rs; keyboard_hierarchy.rs
    para keyboard_hierarchy_tests.rs — um `mod` apagado re-liga o
    #[cfg(test)] ao vizinho, em silêncio (HOWTO §2).

A PROVA DE QUE NADA MUDOU:
  · UM commit por peça; cada um compila e passa — bissectável.
  · o corpo MOVE-SE (verbatim.py; `git diff --color-moved=zebra
    --color-moved-ws=allow-indentation-change`): só assinaturas, parâmetros
    e o «consumido?» são linhas novas.
  · zero alocações ou clones novos por evento de ponteiro.
  · as entradas da lista descem NO MESMO commit (a metade «ficou para
    trás» obriga) e a quota de linhas da §2.5 fica respeitada.

O FIM DA LINHA:
  · cargo nextest list --workspace --cargo-profile ci-test > target/prova/depois.txt
    python3 scripts/nextest-list-diff.py target/prova/antes.txt target/prova/depois.txt
    → ONLY-A = 0; ONLY-B só os gates novos, nomeados.
  · cargo test -p ph2d-host-desktop --test it   À PARTE
  · cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings
  · cargo check --workspace --all-targets sem aviso
  · bash scripts/check-standalone-optional.sh · bash scripts/check-workflow-packages.sh
  · cargo machete · typos --force-exclude <ficheiros tocados>
  · o delta de linhas da shell contra target/prova/shell_antes.txt ≤ 1 200
  · auditoria ≥ 2 lentes sobre o diff acumulado (/pd-auditoria)

REGRAS PERMANENTES:
A. Tudo DENTRO de Worktrees/line-input-dispatch/. `pwd` na dúvida.
B. Sondas em bash (`bash -c '…'`): o shell das ferramentas é zsh.
C. Edite pela ferramenta Edit; script só para mover/renomear em N sítios,
   SEMPRE com assert de contagem. Renomear por NOME destrói prosa.
D. `git commit --no-verify` frequente; NUNCA push, --force, `git add -A`.
E. ⛔ TETO_LOC não se toca; as SUAS entradas das listas descem no mesmo
   commit; as dos outros territórios nunca.
F. Fechar = PARE. Você NÃO integra nem roda ship.sh.
G. PARE e reporte ao Enio SÓ se: contrato congelado (CLAUDE.md §6), rebase
   a conflitar fora do seu território, ou cura que muda produto/ordem.
H. HANDOFF em docs/IntegracaoMultiAgente/HANDOFF_INTEGRACAO_line_input_dispatch_<data>.md
   — `bash scripts/collision-surface.sh` colado; a tabela peça → ficheiro
   → linhas; os gates re-apontados com a mutação de cada um; o que foi
   construído, medido e REVERTIDO; as premissas DESTE bloco que a medição
   derrubou; o delta da shell; e os gestos do smoke: em CADA ferramenta
   clicar, arrastar, rodar a roda e as teclas — vetor (todos os modos),
   Flip, esqueleto, física, 3D, áudio, pintura, Hierarquia, painéis.
I. No fim: rm -rf target/*/incremental ; DEIXE O SMOKE COMPILADO, 2
   corridas, a 2ª no handoff:  cargo build -p ph2d-host-desktop --profile smoke
═══════════════════════════════════════════════════════════════════
```

---

## §4 — BLOCO `line/render-bodies` (cole noutra janela nova)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA — Modo L · REFATORAÇÃO FINAL: OS CORPOS DO QUADRO
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/render-bodies.  Alvo: o quadro já é um
índice de 125 fases, mas as fases e os drenos que elas chamam ainda têm
corpos gigantes. As entradas do SEU território saem das listas numeradas
(tecto 200 L por função, 600 por ficheiro):
  render_loop/fase_bus_drain.rs::fase_bus_drain   2 401
  render_loop/snapshots.rs::publish                1 068
  render_loop/mod.rs::run_render_frame               984
  render_loop/sim_extract.rs::run                    491
  render_loop/present.rs::run_present_phase          484
  render_loop/image_edit.rs::dispatch                483
  hero_intents/sprite_merge.rs::drain_merge_sprites  439
  render_loop/hierarchy.rs::dispatch                 414
  render_loop/inspector_commits.rs::dispatch         383
  render_loop/fase_audio_panels.rs::fase_audio_panels 373
  render_loop/bgremoval_preview.rs::dispatch         333
  render_loop/autokey_pass.rs::apply_samples         317
  render_loop/push_look_probe.rs::probe_push_render_and_look 314
  hero_intents/hierarchy.rs::drain_reparent          297
  hero_intents/image_edit/bgremoval.rs::drain_bgremoval 237
  render_loop/fase_snapshots_publish.rs::fase_snapshots_publish 216
  render_loop/fase_vector_bands.rs::fase_vector_bands 209
  hero_intents/image_edit/equalize_sizes.rs::drain_equalize_sizes 204
  ficheiros: fase_bus_drain.rs 2 654 · render_loop/mod.rs 1 604 ·
             snapshots.rs 1 398 · sim_extract.rs 1 155
⛔ ESTA LINHA NÃO MUDA PRODUTO: nenhum pixel, texto ou comportamento, e a
   ORDEM do quadro é o contrato. Cura que peça mudança visível ou de ordem
   → PARE e reporte ao Enio.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. bash scripts/hw-profile.sh          # tem de dizer `workstation`
2. cd /home/enio/Documentos/Projetos/PH2D && git status -sb
      → RAIZ, em main. M/?? em project-memory/ são ALHEIOS: não toque.
3. mkdir -p Worktrees
   git worktree add -b line/render-bodies Worktrees/line-render-bodies main
4. cd Worktrees/line-render-bodies
   pwd && git branch --show-current    # DEVE dizer line/render-bodies
5. cargo check -p ph2d-host-desktop    # warm-up; o 1º build é frio
6. bash scripts/mergiraf-setup.sh      # idempotente; ✗ não é bloqueio
7. A BASE DA PROVA, antes de editar uma linha:
   mkdir -p target/prova
   cargo nextest list --workspace --cargo-profile ci-test > target/prova/antes.txt
   wc -l target/prova/antes.txt        # ⚠️ vazio = a prova mede nada
   bash -c 'find shells/desktop -name "*.rs" -not -path "*/target/*" -exec cat {} + | wc -l' > target/prova/shell_antes.txt
8. OS INSTRUMENTOS DA PROVA DE MOVIMENTO (não versionados):
   mkdir -p .cauda-render-bodies
   cp /home/enio/Documentos/Projetos/PH2D/Worktrees/line-render-loop/.cauda-render-loop/{verbatim.py,extract_phase.py,mutlib.sh,prescan_gates.py,run_phases2.sh} .cauda-render-bodies/
      → ⛔ NUNCA os commite. Faltou algum? §4 do handoff da line/render-loop.
9. LEIA INTEIRO (dentro da worktree):
   a) docs/IntegracaoMultiAgente/BLOCOS_ABERTURA_REFATORACAO_FINAL_2026-09-13.md
      — §1 territórios e §2 CERCAS (LEI).
   b) docs/IntegracaoMultiAgente/HANDOFF_INTEGRACAO_line_render_loop_2026-09-13.md
      — INTEIRO: é a linha cujo trabalho você continua (§4 o método,
        §10–§12 o que ficou).
   c) shells/desktop/tests/it/frame_text.rs — o instrumento de que 63
      ficheiros de teste dependem.
   d) docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md §2, §3, §4
   e) docs/IntegracaoMultiAgente/DIRETRIZ.md §0, §1.5, §6.7
   f) docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md — tudo, e
      releia a cada passo.
10. Reporte "linha render-bodies pronta" e SIGA (não pare).

A OBRA:
  O MOLDE é o da line/render-loop, um nível abaixo: um corpo grande parte-se
  em sub-fases `impl crate::App { pub(super) fn fase_<x>(&mut self, …) }` em
  ficheiros irmãos, chamadas PELA MESMA ORDEM; locais entram por
  parâmetro e voltam por valor ou contexto nomeado; o que é de FAMÍLIA sai
  para a crate dela (HOWTO §4). ⛔ Nunca ganchos genéricos, trait de fase
  ou registo de callbacks.
  ⛔⛔ A ARMADILHA MUDA DESTA LINHA: o `frame_text::render_frame()` só
     emenda chamadas `self.fase_*(` (recursivo). Um bloco que sai de uma
     fase para uma função LIVRE, ou para um método com outro prefixo,
     DESAPARECE do texto do quadro — e os gates de ORDEM e PRESENÇA que o
     liam ficam VERDES a ler menos. Duas saídas, escolha medindo:
       (a) o bloco extraído é `self.fase_<x>(` e o render_frame o vê;
       (b) o frame_text aprende a nova forma, com auto-teste e mutação.
     Antes de extrair um corpo, `grep` os gates que citam texto dele.
  · run_render_frame (984) é um índice de 125 chamadas: o corte é por
    ASSUNTO em sub-índices `fase_*` — ⚠️ o auto-teste
    the_frame_text_is_the_whole_frame_and_every_phase_is_called tem de
    continuar a chegar ao run_present_phase e a não achar órfãs.
  · fase_bus_drain (2 401): o `for action in hero.bus.drain()` — cada braço
    é um ASSUNTO; ⚠️ a ordem dos braços de um `match` não é ordem de
    execução, a dos `if` sequenciais é.
  · snapshots::publish (1 068): publica os instantâneos que o Inspector, a
    Hierarquia e o gizmo leem NO MESMO quadro (P7c) — ⚠️ o sintoma de uma
    ordem partida é um valor que só aparece ao 2.º clique.
  · os drain_* de hero_intents são chamados de image_edit.rs, hierarchy.rs,
    fase_hierarchy_group_merge.rs, fase_sprite_precision_emissive.rs,
    fase_vector_tree_settle.rs e de testes (hierarchy_order_tests.rs,
    vec_zorder_fixpoint_tests.rs): corte o interior, a assinatura fica.
  · ⚠️ ORDEM com gate (não a quebre): o pick de hover no topo · os sinais
    entre produtores e dreno · as máquinas de UI e o undo · o dreno da
    hierarquia antes da projecção de z antes da captura do undo
    (a_hierarchy_drag_leaves_the_capture_a_fixed_point).
  · ⚠️ Gates de FAMÍLIA que leem a shell pelo caminho moram FORA dela:
    bash -c 'git grep -n "shells/desktop/src" -- crates tools'
  · ⚠️ Gate lido também pela line/input-dispatch (ex.:
    the_highlight_has_one_source): edite SÓ as agulhas do quadro.

A PROVA DE QUE NADA MUDOU:
  · UM commit por peça; cada um compila e passa — bissectável.
  · o corpo MOVE-SE (verbatim.py; `git diff --color-moved=zebra
    --color-moved-ws=allow-indentation-change`).
  · zero alocações ou clones novos por quadro (é o laço quente).
  · as entradas da lista descem NO MESMO commit; quota da §2.5 respeitada.
  ⚠️ Sem janela, o quadro devolve no 1.º `let Some(gfx)` — NENHUM teste o
    corre. O smoke do dono é a prova de comportamento.

O FIM DA LINHA:
  · cargo nextest list --workspace --cargo-profile ci-test > target/prova/depois.txt
    python3 scripts/nextest-list-diff.py target/prova/antes.txt target/prova/depois.txt
    → ONLY-A = 0; ONLY-B só os gates novos, nomeados.
  · cargo test -p ph2d-host-desktop --test it   À PARTE
  · cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings
  · cargo check --workspace --all-targets sem aviso
  · bash scripts/check-standalone-optional.sh · bash scripts/check-workflow-packages.sh
  · cargo machete · typos --force-exclude <ficheiros tocados>
  · o delta de linhas da shell contra target/prova/shell_antes.txt ≤ 1 500
  · auditoria ≥ 2 lentes sobre o diff acumulado (/pd-auditoria)

REGRAS PERMANENTES:
A. Tudo DENTRO de Worktrees/line-render-bodies/. `pwd` na dúvida.
B. Sondas em bash (`bash -c '…'`): o shell das ferramentas é zsh.
C. Edite pela ferramenta Edit; script só para mover/renomear em N sítios,
   SEMPRE com assert de contagem. Renomear por NOME destrói prosa.
D. `git commit --no-verify` frequente; NUNCA push, --force, `git add -A`.
E. ⛔ TETO_LOC não se toca; as SUAS entradas das listas descem no mesmo
   commit; as dos outros territórios nunca.
F. Fechar = PARE. Você NÃO integra nem roda ship.sh.
G. PARE e reporte ao Enio SÓ se: contrato congelado (CLAUDE.md §6), rebase
   a conflitar fora do seu território, ou cura que muda produto/ordem.
H. HANDOFF em docs/IntegracaoMultiAgente/HANDOFF_INTEGRACAO_line_render_bodies_<data>.md
   — `bash scripts/collision-surface.sh` colado; a tabela corpo → peças →
   linhas; o que o frame_text passou a ver (ou não); o que foi construído,
   medido e REVERTIDO; as premissas DESTE bloco que a medição derrubou; o
   delta da shell; e os gestos do smoke: abrir cada módulo, mexer nos
   painéis (Inspector com DOIS objectos, Hierarquia: renomear, olho,
   cadeado, apagar, duplicar, arrastar + Ctrl+Z), ferramentas de imagem
   (Remoção de fundo, Equalize Sizes, Color Equalization, fundir sprites),
   painéis de áudio, e seleccionar objectos e ver Inspector/Hierarquia/
   gizmo acompanharem NO MESMO quadro.
I. No fim: rm -rf target/*/incremental ; DEIXE O SMOKE COMPILADO, 2
   corridas, a 2ª no handoff:  cargo build -p ph2d-host-desktop --profile smoke
═══════════════════════════════════════════════════════════════════
```

---

## §5 — BLOCO `line/loc-caps` (cole noutra janela nova)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA — Modo L · REFATORAÇÃO FINAL: O RESTO DAS LISTAS
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/loc-caps.  Alvo: as entradas do SEU
território saem das listas numeradas de tamanho. Duas frentes:
 FRENTE 1 — shell, fora do laço (tecto 200 L/função, 600 L/ficheiro):
  init.rs::build_initial_state 499 · project_load.rs::project_load_from 484
  build_smoke_router.rs::route 441 · build_smoke.rs::build_smoke 416
  main.rs::App::new 263 · envelope_smoke.rs::frame 247
  layout_live.rs::lay_out 206 · blend_smoke.rs::blend_smoke 205
  ficheiros: app_state.rs 1 551 · main.rs 1 289
 FRENTE 2 — crates:
  architecture_workspace_file_loc_cap.rs (tecto 700 L/ficheiro), 16:
   ph2d-render/src/compressed_pipeline.rs 993 · ph2d-render/src/renderer.rs 932
   ph2d-tool-bgremoval/src/algorithm/compose.rs 931
   ph2d-tool-color-equalization/src/params.rs 888
   ph2d-render/src/layer_compositor/mod.rs 882
   ph2d-painter-effects/src/adjustments/mod.rs 863 · …/adjustments/spatial.rs 856
   ph2d-editor-core/src/grid_snap/state.rs 796 · ph2d-vector/src/vector_network.rs 780
   ph2d-tool-equalize-sizes/src/algorithm.rs 755
   ph2d-tool-color-equalization/src/gpu/auto_wb.rs 748 · …/gpu/tonal_batch.rs 744
   ph2d-imageio-ph2d-native/src/schema.rs 746 · ph2d-tool-rasterize/src/algorithm.rs 734
   ph2d-render/src/individual.rs 708 · ph2d-tool-bgremoval/src/algorithm/chroma/mod.rs 704
  architecture_panel_loc_cap.rs FN_OVERAGE_OK (tecto 200 L/função), 6:
   ph2d-panel-hierarchy/src/paint.rs::paint_hierarchy_body 252
   ph2d-panel-equalize-sizes/src/paint.rs::paint_body_sections 237
   ph2d-panel-inspector/src/event.rs::apply_event_impl 230
   ph2d-panel-hierarchy/src/row.rs::paint_hierarchy_row 226
   ph2d-panel-color-equalization/src/populate.rs::populate 203
   ph2d-panel-audio-mixer/src/paint.rs::paint 203
⛔ ESTA LINHA NÃO MUDA PRODUTO: nenhum pixel, texto, formato de ficheiro
   ou comportamento. Cura que peça mudança visível → PARE e reporte.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. bash scripts/hw-profile.sh          # tem de dizer `workstation`
2. cd /home/enio/Documentos/Projetos/PH2D && git status -sb
      → RAIZ, em main. M/?? em project-memory/ são ALHEIOS: não toque.
3. mkdir -p Worktrees
   git worktree add -b line/loc-caps Worktrees/line-loc-caps main
4. cd Worktrees/line-loc-caps
   pwd && git branch --show-current    # DEVE dizer line/loc-caps
5. cargo check -p ph2d-host-desktop    # warm-up; o 1º build é frio
6. bash scripts/mergiraf-setup.sh      # idempotente; ✗ não é bloqueio
7. A BASE DA PROVA, antes de editar uma linha:
   mkdir -p target/prova
   cargo nextest list --workspace --cargo-profile ci-test > target/prova/antes.txt
   wc -l target/prova/antes.txt        # ⚠️ vazio = a prova mede nada
   bash -c 'find shells/desktop -name "*.rs" -not -path "*/target/*" -exec cat {} + | wc -l' > target/prova/shell_antes.txt
8. LEIA INTEIRO (dentro da worktree):
   a) docs/IntegracaoMultiAgente/BLOCOS_ABERTURA_REFATORACAO_FINAL_2026-09-13.md
      — §1 territórios e §2 CERCAS (LEI; a §2.2 e a §2.6 são SUAS).
   b) docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md §2
      (as ⛔ são MUDAS) e §3.
   c) os cabeçalhos das quatro listas (fn_loc_caps.rs, file_loc_caps.rs,
      architecture_workspace_file_loc_cap.rs, architecture_panel_loc_cap.rs)
      — o censo de obsolescência e a FOLGA_MAXIMA de cada uma.
   d) docs/IntegracaoMultiAgente/DIRETRIZ.md §0, §1.5, §6.7
   e) docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md — tudo, e
      releia a cada passo.
9. Reporte "linha loc-caps pronta" e SIGA (não pare).

A OBRA:
  O CORTE é por ASSUNTO, e começa pelo mais barato:
   1. testes embutidos saem para irmãos: `#[cfg(test)] #[path =
      "<x>_tests.rs"] mod tests;` (o padrão da casa). ⚠️ um `mod` apagado
      re-liga o #[cfg(test)] ao item VIZINHO, em silêncio — o gate
      architecture_no_orphan_source_file apanha o órfão; confira a
      contagem de testes (ONLY-A = 0).
   2. ajudantes privados saem para ficheiros irmãos do mesmo módulo.
   3. só então itens públicos — ⛔ o CAMINHO PÚBLICO NÃO MUDA (a
      line/render-bodies usa o ph2d-render; outras crates usam o resto):
      `x.rs` → `x/mod.rs` + irmãos, com o item acessível pelo mesmo
      caminho. Um `pub use` DENTRO da crate para preservar o caminho é
      aceite; entre crates, nunca.
  · App::new e app_state.rs: ⛔ NÃO renomeie nem reagrupe campos da App —
    as outras duas linhas compilam contra `self.<campo>`. Mova impl, tipos
    auxiliares e docs por assunto. Se o ficheiro não chega a 600 sem
    reagrupar campos, BAIXE a entrada ao medido, escreva o porquê, e
    ponha a proposta de agrupamento no handoff (é da próxima rodada).
  · build_smoke_router::route e envelope_smoke/blend_smoke são roteadores
    de CENAS: ⚠️ os números das cenas contam-se no `match`
    (no_two_*_scenes_claim_the_same_level e os censos de roteador por
    família) — um `match` partido em sub-roteadores por FAIXA tem de
    manter cada número a chegar à mesma cena. Se partir tornar a tabela
    ilegível, a entrada fica com a razão escrita.
  · project_load_from: é a escada de migração do formato — ⛔ nenhum degrau
    muda; a tripla do PROJECT_SCHEMA (project_schema.rs +
    project_schema_tests.rs) fica intacta.
  · ph2d-imageio-ph2d-native/src/schema.rs é FORMATO de ficheiro: mover
    código, nunca campos nem ordem de serialização.
  · ⚠️ FEATURES NÃO VIAJAM (HOWTO §2.4; mordeu a 13/09 com 75 erros
    invisíveis): um `#[cfg(feature = …)]` que muda de ficheiro continua a
    precisar da feature declarada na MESMA crate. Prova: `cargo check -p
    <crate> --all-targets` SOZINHA para cada crate tocada, além do portão
    bash scripts/check-standalone-optional.sh.

A PROVA DE QUE NADA MUDOU:
  · UM commit por ficheiro/função; cada um compila e passa.
  · o código MOVE-SE (`git diff --color-moved=zebra
    --color-moved-ws=allow-indentation-change`).
  · as entradas descem NO MESMO commit; a quota da §2.5 (800 linhas na
    shell) fica respeitada — as crates não contam para ela.

O FIM DA LINHA:
  · cargo nextest list --workspace --cargo-profile ci-test > target/prova/depois.txt
    python3 scripts/nextest-list-diff.py target/prova/antes.txt target/prova/depois.txt
    → ONLY-A = 0; ONLY-B só os gates novos, nomeados.
  · cargo test -p ph2d-host-desktop --test it   À PARTE
  · cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings
  · cargo check --workspace --all-targets sem aviso
  · bash scripts/check-standalone-optional.sh · bash scripts/check-workflow-packages.sh
  · cargo check -p <crate> --all-targets para CADA crate tocada, sozinha
  · cargo machete · typos --force-exclude <ficheiros tocados>
  · o delta de linhas da shell contra target/prova/shell_antes.txt ≤ 800
  · auditoria ≥ 2 lentes sobre o diff acumulado (/pd-auditoria)

REGRAS PERMANENTES:
A. Tudo DENTRO de Worktrees/line-loc-caps/. `pwd` na dúvida.
B. Sondas em bash (`bash -c '…'`): o shell das ferramentas é zsh.
C. Edite pela ferramenta Edit; script só para mover/renomear em N sítios,
   SEMPRE com assert de contagem. Renomear por NOME destrói prosa.
D. `git commit --no-verify` frequente; NUNCA push, --force, `git add -A`.
E. ⛔ TETO_LOC não se toca; as SUAS entradas das listas descem no mesmo
   commit; as dos outros territórios nunca.
F. Fechar = PARE. Você NÃO integra nem roda ship.sh.
G. PARE e reporte ao Enio SÓ se: contrato congelado (CLAUDE.md §6), rebase
   a conflitar fora do seu território, ou cura que muda produto/formato.
H. HANDOFF em docs/IntegracaoMultiAgente/HANDOFF_INTEGRACAO_line_loc_caps_<data>.md
   — `bash scripts/collision-surface.sh` colado; a tabela ficheiro/função →
   cortes → linhas; as entradas que FICARAM, com a razão medida; o que foi
   construído, medido e REVERTIDO; as premissas DESTE bloco que a medição
   derrubou; o delta da shell; e os gestos do smoke: abrir o app, abrir e
   gravar um projecto (Ctrl+O/Ctrl+S), as cenas de build/envelope/blend,
   auto layout, Hierarquia e Inspector, Grid Snap, as ferramentas de imagem
   (Remoção de fundo, Color Equalization, Equalize Sizes, Rasterize), o
   mixer de áudio, e abrir/gravar uma imagem .ph2d.
I. No fim: rm -rf target/*/incremental ; DEIXE O SMOKE COMPILADO, 2
   corridas, a 2ª no handoff:  cargo build -p ph2d-host-desktop --profile smoke
═══════════════════════════════════════════════════════════════════
```

---

## §6 — Para o integrador, no dia da fusão

1. **Ordem:** meça com `bash scripts/collision-surface.sh` em cada worktree; a colisão esperada é só
   TEXTUAL nas quatro listas numeradas e nos gates lidos por duas linhas (§2.3–§2.4). O valor certo
   de cada entrada é o medido na árvore combinada.
2. **`TETO_LOC`:** some os três deltas contra a base `193 205`; o medido na árvore combinada, depois
   do `cargo fmt --all`, tem de ficar ≤ 196 990. ⛔ Não sobe.
3. **Os instrumentos da prova de movimento** (`verbatim.py` e irmãos) vivem fora do repo em
   `Worktrees/line-render-loop/.cauda-render-loop/` — duas linhas copiam-nos para pastas não
   versionadas. Versione-os em `scripts/` depois das fusões, se o método ficar.
4. **`the_highlight_has_one_source`** e qualquer gate que as duas linhas do laço editaram: confira
   que a versão fundida lê os DOIS endereços novos e ainda reprova (mutação).
5. **Os portões novos do `ship.sh`** (`check-standalone-optional.sh`, `check-workflow-packages.sh`)
   correm na árvore combinada antes do envio.
