# Handoff de INTEGRAÇÃO — `line/GameEngineFuncional` (02/11)

Leitor: o agente integrador (DIRETRIZ §1.5.3 / §1.5.9). Cobre as ondas **GEF 20 → 21b** (câmera intuitiva, moedas,
som como asset do projeto, Add ▸ Text) desde o handoff de 01/11 (já no main: o merge-base é o main de hoje).
Integração e ship só por ordem do Enio.

**Smoke do dono 02/11:** onda 20 e 20b OK · metade 1 das moedas OK · a queda do som no campo Sound CONFIRMADA no
app (depois de `2f2e2af52c`); o jogo toca o som da moeda. O placar «Coins: N» montado pelo dono ficou **ADIADO por
decisão dele** («serão necessários muitos ajustes em HUD, Label, Text») — vem DEPOIS da integração (ABERTO 1).

## 1. Identidade

| | |
|---|---|
| Branch | `line/GameEngineFuncional` |
| HEAD | o commit deste handoff (código: `8b9cba1cda`) |
| merge-base com o main | `3df5894489` — **o main não andou ⇒ `--ff-only` possível hoje** |
| Commits / ficheiros | 20 antes deste handoff · 131 ficheiros, +4 657 / −842 (`git diff --shortstat 3df5894489..8b9cba1cda`) |
| Crate NOVA | `ph2d-app-text` (folha de família: a entrada Add ▸ Text) |

Não há vermelho herdado conhecido. A suíte inteira e o `nextest-impacted` NÃO correram na linha (ordem do dono
23/10): o integrador corre-os e confronta com o main em `3df5894489`.

## 2. Superfície de colisão (`scripts/collision-surface.sh`, caminho absoluto, VERBATIM)

```
SUPERFÍCIE DE COLISÃO — line/GameEngineFuncional contra main
  merge-base 3df5894489   ·   20 commit(s)   ·   131 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        214   (base: 213)
  ⚠   └ tripla do gate               (214, 13, 27)   (base: (213, 13, 27))
    VEC_SCENE_SCHEMA                       27   (base: 27)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 25   (base: 25)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              116   (base: 116)
    ph2d-render (espelho)                 117   (base: 117)
    ph2d-script (espelho)                 117   (base: 117)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0196   próximo livre: 0197
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 1 pacote(s) '+name' novo(s):
      "ph2d-app-text"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
  ✗   919 / 600   shells/desktop/src/app_state.rs
  ✗  1024 / 600   shells/desktop/src/main.rs
───────────────────────────────────────────────────────────────────────────────
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.
```

Leitura (DELTA, não valor):

- **`PROJECT_SCHEMA` 213 → 214** (o `ProjectFile.sounds`, onda 21b); tripla `(214, 13, 27)`, base `(213, 13, 27)`.
  Reconta-se contra o main do dia: `python3 scripts/schema-recount.py <valor>`. `VEC_SCENE_SCHEMA` 27,
  `FLIP_SCHEMA` 13, `DOC_VERSION` 25 inalterados; registo de componentes 116/117/117 inalterado.
- §6 (`node.rs`, `tool.rs`) INTOCADOS; nenhum ADR; `Cargo.lock`: só o pacote INTERNO `ph2d-app-text` (mais a
  dev-dep interna `ph2d-tags` em `shells/game-scenes`, da onda 21).
- `app_state.rs` 919/600 e `main.rs` 1024/600: ambos na lista `FILE_OVERAGE_OK` do `file_loc_caps`, que passa.
- Tectos que SOMAM entre linhas (reconferir na árvore combinada): shell 189 011 / 189 041 (regra do gate) ·
  `ph2d-app-game` 2 728 / 2 728 (**no tecto**) · `ph2d-app-components` 80 602 / 80 869 · `ph2d-app-vec` 93 637 / 93 637
  (intocado nesta onda; no tecto) · `ph2d-app-text` 323 / 423 (medido 323 + folga 100) ·
  `ASSET_KIND_FILTERS` 6→7 · `CARGA_DE_COMANDOS` inspector 100→101 (21b).

## 3. O que mudou por onda (resumo; o mecanismo vive nos docs linkados)

| Onda | Commits | O quê | Doc |
|---|---|---|---|
| **20** câmera intuitiva | `2fe470dab5` `bf5828f316` `c268291fc6` `500354c781` | dicas em todo campo da câmera · alvo por arrastar/Eyedropper · zona morta no meio da moldura · gizmo à Vector (Esc restaura, duplo clique repõe, Shift encaixa) · limites nascem no contorno do nível · `camera-frame/lib.rs` cortado (gestos para `gestos.rs`) | [handoff](HANDOFF_CONTINUACAO_2026-11-02_20_CAMERA_INTUITIVA.md) · [plano](../20_camera_intuitiva_plano.md) |
| **20b** indicador de queda | `046a10bbe4` `795bbe4896` `1e23193cf4` | todo campo que aceita o payload ganha contorno verde; o campo sob o cursor, contorno grosso (verde/vermelho); porta única `asset_drop::{field_target, verdict, field_marks}` | idem |
| **21** moedas (metade 1) | `f08629069a` `6494f86cd2` `97ce78ad18` `b17ca59fcb` | moedas do nível 2 montadas com peças (Trigger + SignalOnHit + AudioSource2D + SignalActions, zero Luau) · HUD com Counter `coins` · som de disparo sobrevive ao dono numa corrida · `smoke::Portas.camera` virou `componentes` | [handoff](HANDOFF_CONTINUACAO_2026-11-02_21_MOEDAS.md) · [plano](../21_moedas_plano.md) |
| **21b** som asset (metade 2) | `b13f56c6ab` `1da2f0054d` `3fdb3fb3a0` `e8f38eae8e` `ba68966b1d` | `ph2d_ecs::ProjectSounds` (blob `PH2DSONS`), `AudioSource2D.sound` = NOME · `AssetKind::Sound`, chip «Sound», 4.º campo receptor de queda · `PROJECT_SCHEMA` 213→214 (o load importa caminhos antigos) · rebobinar só conta o que muda · queda solta o teclado do campo | [handoff](HANDOFF_CONTINUACAO_2026-11-02_21b_SOM_ASSET.md) |
| **21c** (esta janela) | `b3ac769b15` `2f2e2af52c` `8b9cba1cda` | ver §3.1 | — |

### 3.1 Commits novos desde o 21b

- **`b3ac769b15` Add ▸ Text** (decisão do dono 02/11, «como no Figma»): crate NOVA `ph2d-app-text` (folha; **não**
  depende de `ph2d-app-vec` — o gate `architecture_no_dependency_climbs_a_layer` recusou a 1.ª versão; a shell passa o
  «armar» como fecho e o «armada?» como bool). Entrada de menu «Text» (i18n `object_add.vector.text`, placeholder
  `object_add.vector.text.placeholder` = "Text"). Arma a ferramenta do vetor; a shell abre a sessão de texto já
  escrita «Text» em Draw ▸ Text (`shells/desktop/src/vec_text.rs`: `vec_text_add_open`; `vec_text_start` partido do
  `vec_text_click`); a 1.ª tecla troca a palavra; com o HUD (ou filho dele) seleccionado, nasce no centro do HUD e
  filho dele (`reparent_keeping_world`). Estado `TextAdd` no `App` (`app_state.rs`/`main.rs`); ready+upkeep em
  `render_loop/fase_vector_live_recooks.rs`; Add em `render_loop/fase_object_add.rs` (`FAMILIES` ganha
  `ph2d_app_text::ENTRIES`); `undo_app.rs` repõe. `hero.rs` 709→688: `blur_focus` e `solta_o_teclado` mudaram para o
  irmão `crates/ph2d-editor-core/src/screens/hero/foco.rs` (o 709 vinha do `e8f38eae8e`). `App::new` 209→202 (os três
  `Playhead` em pausa por um fecho `paused()`).
  Foto: cena vazia + `PH2D_OBJECT_ADD_PICK=object_add.vector.text` — «Text» no centro, «Path 0» seleccionado,
  Draw ▸ Text, cursor no fim. ⚠️ Na cena `PH2D_GAME_SMOKE=4` o pick automático corre no 1.º quadro, antes de a vista
  se enquadrar, e o texto nasce em (0,0) fora do enquadramento — artefacto do atalho de smoke, não do clique do dono.
- **`2f2e2af52c`**: o som largado no Sound não aparecia até re-seleccionar — `sync_audio_fields`
  (`crates/ph2d-panel-inspector/src/sync_sections.rs`) só semeava com `entity_changed`; agora `assinatura_audio` +
  `InspectorState.last_audio_sig` (molde da câmera, plano 24 W7). Gate
  `a_sound_changed_outside_the_panel_shows_without_reselecting`; mutação (tirar o `|| mudou`) sangra.
- **`8b9cba1cda` fecho (auditoria de 2 lentes: correcção + arquitetura/regras).** A lente de correcção achou 3
  defeitos da mesma família no Add ▸ Text, curados juntos prendendo o pedido à ORIGEM da sessão: (1) a troca da
  1.ª tecla passava a outra sessão (duplo clique noutro texto) e apagava-o — `fresh: Option<origem>`,
  `before_key(origem,…)`; (2) Backspace inicial matava a entidade e a seguinte nascia fora do HUD — o pedido vive
  até a sessão acabar, `upkeep(sessão)` reparenta toda entidade dela; (3) pedido consumido sem a ferramenta na mão
  ficava pendurado — `ready` larga-o. 4 mutantes, 4 sangram. A lente de arquitetura achou só o `fn_loc_caps`
  (`App::new` 208→202 na catraca). Portões da shell que o `workspace_file_loc_cap` não vê e estavam **VERMELHOS
  latentes** das ondas anteriores, agora curados por corte verbatim: `project_schema.rs` 607→593 (o degrau 166→167
  subiu para `project_schema_history_v160.rs`, agora v160..v167, com nota datada) ·
  `input_dispatch/despacho_metodos_picks_e_arrastos.rs` 603→581 (`cursor_over_motion_graph`/`cursor_over_timeline`
  para o irmão novo `input_dispatch/despacho_cursor_sobre.rs`).

## 4. Foundational / partilhado tocado e porque é aditivo

Tudo aditivo: variantes novas (`AssetKind::Sound`, `AssetRef::Sound`, `DragPayload::Sound`, `DropTarget::InspectorSound`,
`DropAction::SetSound`), campo novo no fim de `ProjectFile.sounds` (degrau 213→214, o load aceita ficheiros ≤ 213
importando os caminhos antigos), crate folha nova, portas novas (`asset_drop::field_target`/`field_marks`,
`add_by_component`). Movimentos verbatim (sem mudar assinatura): `inspector_audio.rs` → `ph2d-app-components`
`audio_inspector_card.rs`; `blur_focus`/`solta_o_teclado` → `hero/foco.rs`; gestos das alças →
`ph2d-camera-frame/src/gestos.rs`.

## 5. O que um merge pode partir (some aos itens dos três handoffs de continuação, que ainda valem)

- `FAMILIES` do menu Add ganhou uma linha (`ph2d_app_text::ENTRIES`): quem testa a lista do menu reconfere.
- `App` ganhou o campo `text_add`: quem constrói `App` à mão colide.
- `HeroScreen::blur_focus`/`solta_o_teclado` mudaram de ficheiro (mesma assinatura).
- A escada do schema perdeu o degrau 166→167 para o arquivo v160: outra linha que também cortou a escada colide
  aqui; o degrau 214 reconta-se contra o main (`python3 scripts/schema-recount.py <valor>`).
- `despacho_cursor_sobre.rs` é novo em `input_dispatch.rs`.
- Da 21/21b: `smoke::Portas` (`camera` → `componentes`; `importa_som`), `SceneAudio::play` 5.º arg `sons`,
  `forget_absent` 3.º arg, `match` exaustivo sobre `DragPayload`/`AssetRef`/`AssetKind`/`DropTarget`/`DropAction`,
  `ASSET_KIND_FILTERS` 6→7, `rewind_runtime_state` conta só mudanças.
- Da 20: `drop_target_at`, `asset_drag_wire.rs`, `camera_no_canvas.rs`, `ph2d-camera-frame/src/{lib,tinta}.rs`; campo receptor novo entra por `asset_drop::field_target`; `stroke_frame` sobre `Feel::Active` some na família moderna.

## 6. Prova de fecho

Sobre o HEAD de código `8b9cba1cda`. `clippy -D warnings --all-targets` em `ph2d-app-text`, `ph2d-editor-core`,
`ph2d-panel-inspector`, `ph2d-i18n`, `ph2d-host-desktop`: verde. `fmt` verde. Testes por nome verdes:
`ph2d-app-text` inteira (6, crate nova) · host-desktop `add_text_opens_written_with_geometry…`, `fase_object_add` (6),
`loc_caps` (6: `file_loc_caps` + `fn_loc_caps`), `architecture_` (3), `the_hovered_area_owns_the_clipboard_chord` (3),
`the_vector_tool_keys_are_wired` (2), `ladder` (5), `escada` (1), `schema` (1) · editor-core `architecture`
(119 + 1 ignorado), `arch_safe_clamp_only` (2), `the_shell_only_shrinks` (2) · panel-inspector
`the_audio_section_is_alive` (5), `seam_px_m` (4). Binário smoke compilado.

As ondas 20/21/21b trazem o próprio fecho nos handoffs delas (fmt/clippy, mutação 20: 14/14, 20b: 3/3, 21b: lote de
14 com os 2 sobreviventes reforçados, gate de placa da 20 verde). ⛔ **Suíte completa e `nextest-impacted` NÃO
correram** (só na integração).

`agent-loop-profile`: paralelismo 1,08/passo ✗ (alvo 1,5) · test:check 2,4× ✗ (alvo 1) · resto ✓.

## 7. Para o integrador

**CLAUDE.md §5.1, «Game Engine»:** o link do «Último:» já aponta para este handoff (feito na worktree). Memória
por comitar **no checkout principal** (não na worktree): `project-memory/project_add_text_entry_owner_decision_2026_11_02.md`
e mais seis `project_*_2026_11_02.md` + `MEMORY.md` modificado (ver `git -C /home/enio/Documentos/Projetos/PH2D status --short`);
o integrador recolhe-os. O `MEMORY.md` está acima do alvo de 17 KB.

Reconferir na árvore combinada: `python3 scripts/schema-recount.py` · tectos SOMADOS (§2) · censos `ph2d-app-` · suíte completa 1×.

## 8. Smokes (binário `smoke`)

1. **Moedas e som** — `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=4 ./target/smoke/ph2d-host-desktop`.
   Hierarchy → **Coin 1** (Inspector: Physics Body Trigger, Signal On Hit `coin`, Audio Source 2D, Signal Actions).
   **Run ▸ Play in a Window**, setas + espaço até uma moeda: faz o som e some; Stop: voltam as 5. Assets mostra
   `moeda.ogg` (ícone de alto-falante, chip «Sound»). Largar um som no campo Sound de uma moeda: aparece logo.
   Errado se: sem som · um toque remove várias · o som largado só aparece ao re-seleccionar.
2. **Add ▸ Text** — a mesma cena: seleccionar o HUD, «+» da Hierarchy ▸ **Text**: «Text» nasce no centro do HUD,
   filho dele; a 1.ª tecla troca a palavra. Errado se: nasce fora do HUD ou a 1.ª tecla apaga outro texto.
3. **Foto do Add ▸ Text** — `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && bash docs/Components/ferramentas/fotografa_cena.sh "PH2D_OBJECT_ADD_PICK=object_add.vector.text" 1930 1040 <saida.png> 12`.
4. **Câmera (onda 20/20b)** — `PH2D_GAME_SMOKE=4` como acima; Hierarchy → **Camera**; arrastar **Player** até **Target**:
   contorno grosso verde, escreve o nome. Gizmo: Esc restaura · duplo clique repõe · Shift encaixa. Foto:
   `PH2D_GAME_SMOKE_FOTO=camera_larga` (mesma pasta/binário).

## 9. ABERTO (próxima onda, em ordem)

1. **Placar «Coins: N»** e os ajustes de HUD/Label/Text que o dono vai pedir (smoke dele adiado). O texto nasce com o
   nome «Path 0» na Hierarchy — perguntar ao dono se prefere «Text».
2. **Som sem gaveta nem lápide:** arrumar um som numa gaveta dos Assets (pede a família no `SavedAssignment`, outro
   degrau) e tirar um som do projeto (lápide no `ProjectState`, como a imagem) — hoje recusam com voz.
3. Herdadas da GEF 20: o conta-gotas só apanha sprites (limite conhecido; a lista/dropdown de objectos foi RECUSADA
   pelo dono — não reabrir); perf do `LayerCompositor` (ver o [handoff 19/4](HANDOFF_CONTINUACAO_2026-11-01_19_ORDEM_E_CLIQUE_4.md)).
4. **J6 inimigos:** `SignalTagFilter` existe e ainda não é usado (para que inimigos não apanhem moedas).
5. Catraca: `ph2d-app-game` e `ph2d-app-vec` estão NO tecto — a próxima onda que os tocar MOVE antes de crescer.
