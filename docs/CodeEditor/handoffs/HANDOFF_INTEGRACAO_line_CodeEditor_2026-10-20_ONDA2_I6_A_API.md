# HANDOFF DE INTEGRAÇÃO — line/CodeEditor, Onda 2 (I6 → item 6: a página de ajuda e a API que funciona) · 20/10/2026

> Leitor: o agente integrador e a próxima LLM da linha. **Não integrado, não enviado.** O último handoff de
> integração no `main` é o do [I0–I5](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md) (17/10). Tudo
> desde lá está na linha, contado nos de continuação:
> [I13](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I13.md) ·
> [Acabamento](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-19_ONDA2_ACABAMENTO.md) ·
> [Ajuda](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-20_ONDA2_AJUDA.md) — e, para o item 6, este documento e o
> plano [05_api_que_funciona.md](../05_api_que_funciona.md) (§1 oráculo, §2 medições, §3 leis, §6–§7 o que mudou
> no caminho e no fecho).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` · HEAD
  `9f7d084ea2` · merge-base `100d34c8e0` (= `main` do dia) · **92 commits** · 617 ficheiros.
- Commits do item 6 + fecho (sobre `0f7de73019`): `c7164bccdd` oráculo+plano · `cad77678e0` a API ·
  `27f74b40fc` cena 6 + override · `d67f106ee7` docs · `7296b88377` `table.insert` · `622b41a8c0` cenas 3–5 →
  shell · `50dbc8968e` curas das auditorias e portões · `826ac56826` portões das mutações · `402acd3751`
  `set_mode` sai · `293a66e6d9` corte de LOC · `9f7d084ea2` fotos da cena 6.

## 2. Superfície de colisão (colada de `scripts/collision-surface.sh`, 20/10 — REFERÊNCIA, não evidência)

```
merge-base 100d34c8e0 · 92 commit(s) · 617 arquivo(s)
SCHEMAS            PROJECT_SCHEMA 191 (base 191) · tripla (191, 13, 23) · VEC_SCENE 23 · FLIP 13 · DOC_VERSION 20
REGISTRO           ph2d-ecs 108 · ph2d-render (espelho) 109 · ph2d-script (espelho) 109 — todos = base
CONTRATO §6        crates/ph2d-nodegraph/src/node.rs intocado · crates/ph2d-editor-core/src/tool.rs intocado
ADR                nenhum novo (último 0188)
Cargo.lock         pacotes novos: ph2d-code-explain, ph2d-code-format, ph2d-code-help (internos; regenerar, nunca à mão)
MARCADORES         nenhum
TETOS DE LOC       ✗ 1059/600 shells/desktop/src/main.rs (isenção pré-existente; o gate da shell passa; main tem 1060)
```

## 3. Foundational / partilhado tocado — e porquê

- **`ph2d-input`** (folha, aditivo + uma mudança de semântica medida):
  - `Key`: tabela ÚNICA (`NAMED`) para o rótulo do artista e o nome de script; `script_name`, `named_in_script`,
    `script_names`, `all_named`, `SCRIPT_ALIASES`. `label()` dá os mesmos textos (gate).
  - `KeyboardState`: as bordas passam a EVENTOS (`went_down`/`went_up`; `was_down` saiu) — um toque dentro de
    um quadro já não se perde. `pressed`/`released` só têm leitores no `ph2d-script` (a auditoria confirmou: o
    `resolve` do Input Map lê `held`, o `player_input` lê o `ActionState`). Novos `iter_pressed/iter_released`.
  - `MouseState` + `MouseButton` novos; `Event::MouseButtonDown/Up` novos (2 braços no `shells/desktop/src/input_log.rs`);
    `InputState.mouse` novo. O `FocusLost` larga também o rato.
- **`ph2d-script`** (API pública mudou — quem a chamar fora da linha não compila):
  - **Saíram:** `ReadSnapshot`, `SpawnQueue`, `SpawnCommand`, `SpawnQueueFull`, `NameSnapshot`, `Scheduler`,
    `TickReport`, `ScriptRuntime::spawn`, `ScriptHost::{provide_read, clear_reads, spawn_named, drain_spawns,
    provide_name, clear_names, spawn_queue, name_snapshot}`, e `ph2d.spawn_named` (Luau).
  - **Mudaram:** `ScriptHost::scene_tick/scene_hear` recebem `&mut SimWorld` (era `&mut World`); novos
    `scene_tick_with/scene_hear_with(…, &mut dyn SceneDoors)`; `SceneReport` ganha `despawned`, `wrote_before`;
    `PodValue::Table` é `BTreeMap<PodKey, PodValue>` (`PodKey` novo — não é formato gravado: ninguém fora da crate
    serializa `PodValue`); `ConsoleKind::Warning` novo (braço no `ph2d-app-code/src/console.rs`).
  - **Novos:** `lane.rs` (`SceneDoors`, `NoDoors`), `scene_wait.rs`, `scene_self.rs`, `stdlib.rs`, `Devices`,
    `Edge`; dependência nova `ph2d-input`.
- **`ph2d-app-components`:** `motores::correm` ganha `portas: &mut ScriptDoors` e DEVOLVE `Vec<Death>`;
  `script_bridge::frame` ganha `doors`; `ScriptDoors` (a porta da `Factory` para os scripts); `script_input` novo (o
  empurrão dos dispositivos saiu da shell); `code_smoke` nível 6. Dev-deps `ph2d-app-code`/`ph2d-panel-code` SAÍRAM
  (curava `no_dependency_climbs_a_layer` — o §9(a) do handoff da Ajuda).
- **Shell (composição):** `fase_signal_outbox.rs` (monta `ScriptDoors`, `deaths.extend(motores::correm(…))`) ·
  `app_state_devices.rs` (`pump_gamepad` limpa SÓ o comando; `push_input_to_script(&mut self)` →
  `script_input::frame`) · `input_dispatch.rs` (uma linha: `rato_do_jogo`) · `despacho_metodos_janela_e_vetor.rs`
  (`pointer_on_canvas`, `rato_do_jogo`) · `App::cursor_shown` → `AppGfx::cursor_shown` (+ `init.rs`,
  `frame_gfx.rs`): a `App` volta a **172** campos e `main.rs::new` a **211** linhas (vermelhos herdados do Acabamento).
  Testes conduzidos `a_cena_{3,4,5}_do_editor_conduzida.rs` mudaram-se para `shells/desktop/tests/it/`; dev-dep
  `ph2d-panel-code` na shell.
- **`ph2d-editor-core` (só gates):** `the_intent_drain_reaches_every_variant.rs` — saem as 4 linhas `SpawnCommand`
  do `UNCONSUMED_PENDING` e `MIN_QUEUES` 9 → **8** (a fila de spawn deixou de existir); `hr12_widgets_a11y.rs` —
  `ph2d-panel-code/src/paint.rs` em `PANEL_A11Y_DELEGATE_OK` (medido: zero `NodeId`/`hit_index`/`register(`).
- **`ph2d-i18n`:** `code.console.warning`, `code.console.warning_noline`,
  `app.components.script_bridge.it_is_not_a_master`, `app.components.script_bridge.the_copy_would_land_inside_its_own_master`;
  textos de `code_help/coroutine.rs` e `globals.rs` (`loadstring`).
- **Gerados:** `runtime/luau/ph2d.d.luau` (bindgen `--write`; tipos `KeyName`, `MouseButtonName`).
- **`tests/spike`:** `c3_coro_timing` mede as ESPERAS DA CENA (fixtura `scene_waits.luau`, dep `ph2d-ecs`);
  `c14_memory` sem o `Scheduler`.

## 4. O que um merge pode partir

- Qualquer linha que chame `ScriptHost::scene_tick/scene_hear(world.world_mut(), …)` → agora `&mut SimWorld`.
- `CENAS` 5 → **6** nos DOIS sítios (`ph2d-app-components/src/code_smoke.rs`, `ph2d-app-code/src/smoke.rs`).
- `motores::correm` (assinatura) e `fase_signal_outbox.rs` (o texto emendado do quadro foi conferido:
  `o_cerebro_fala_antes_de_a_tabela_ouvir` verde).
- `KeyboardState::pressed/released` mudaram de semântica (toque no mesmo quadro conta).
- A catraca `the_app_only_sheds_fields` (172) e `fn_loc_caps` (`main.rs::new` 211, `on_mouse_input` ≤ 200),
  `input_dispatch.rs` 586/600: outra linha que some campos/linhas na shell reprova lá.
- `Cargo.lock`: regenerar.

## 5. Contratos congelados

**Nenhum.** `node.rs` e `tool.rs` intocados (colisão acima); nenhum `PROJECT_SCHEMA`, registo ou ADR.

## 6. Portões do fecho (20/10, sobre o diff acumulado — números MEDIDOS)

| Portão | Resultado |
|---|---|
| `cargo fmt --all -- --check` | ✓ |
| clippy `--workspace --all-targets --all-features -D warnings` | ✓ |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ |
| `censos-da-arvore-combinada.sh` | ✓ 121/121 |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` · `cargo machete` (9 crates) | ✓ |
| `nextest-impacted.sh` (fecho) | 19 329 corridos · 19 326 verdes · 2 FAIL + 1 TIMEOUT, todos resolvidos abaixo |
| `architecture_workspace_file_loc_cap` (vermelho na suíte: `scene_api_tests.rs` 705) | curado em `293a66e6d9` (628) e verde por nome |
| `ph2d-app-board` `opening_the_room_says_how_to_open_the_firewall` | flake de carga: `loadavg` 251 na suíte; sozinho a 41–43 ✓ |
| `ph2d-tool-painter` `o_glaze_do_pincel_escurece_o_cruzamento` (TIMEOUT) | flake de carga: sozinho a 42 ✓ (e os outros 3 do pintor da 1.ª corrida, sozinhos a 74, ✓) |
| `tests/spike` C3 (1000 esperas, 60 Hz) | p50 11,667 ms · p95 16,157 · p99 16,557 · máx 16,657 (tecto 16,667) · 0 vazadas — PASS |

**Auditoria (2 lentes, só leitura):**

- **Costura.** Achou um ⛔⛔: as bordas do teclado e do rato eram apagadas antes da entrega ao script, e nenhum gate o via. Curado e com gate.
- **Correcção.** Achou sete defeitos:
  - curados: corrotinas reaproveitadas alcançáveis por outro objecto, troca de script a meio do quadro, `init` de quem morre, filhos de uma cópia, nível do erro do `lrotate`;
  - o furo do bytecode no `loadstring` foi **refutado por medição** (o modo seguro do mlua já o recusa);
  - o resto ficou declarado ([05 §7](../05_api_que_funciona.md)).

**Prova de mutação: 22 leis.**

- 18 da lista do fecho: 15 sangraram e 3 sobreviveram.
- As 3 sobreviventes e um ramo do `resolve` ganharam portão (`826ac56826`) e sangraram depois.
- A excepção é o `set_mode(Text)`: sobreviveu porque era defesa morta, e **saiu**.
- O controlo (um comentário) ficou verde.

**Medições do item 6** (plano 05 §2):

- **Custo por gancho:** `call` 49 ns · corrotina nova 198 ns, a usada. A reaproveitada (107 ns) foi recusada pela auditoria.
- **Godot 4.7.2 corrido:** fixtura em `ferramentas/oraculo_api/saida.txt`.
- **Família do inteiro:** 17 funções saturam e o `table.insert` passa a dar erro; a classificação foi MEDIDA no gate.

## 7. Binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, duas vezes depois de
`rm -rf target/*/incremental` (95 GB recuperados); a 2.ª: `Finished \`smoke\` profile [optimized] target(s) in 0.47s`.

## 8. Smokes para o DONO

- **Cena 5 (a página de ajuda): FEITA pelo dono, OK (20/10).**
- **Cena 6 (o jogo): PENDENTE.** Comando:
  `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=6 cargo run -p ph2d-host-desktop --profile smoke`
  — a cena já corre. Setas movem o círculo azul; Espaço atira estrelas (sobem e somem em 1,5 s); clique na cena
  leva o quadrado cinzento ao rato; Rebobinar devolve tudo. Fotos: `docs/CodeEditor/ferramentas/smoke/cena6/jogo{,_02,_03,_04}.png`
  (t = 2,0 · 3,3 · 4,4 · 5,5 s; piloto `PH2D_CODE_SMOKE_DEMO=1` pelo `InputState` real). **Errado se:** o Espaço ou
  o clique não fazem nada; uma estrela fica parada; o quadrado não volta ao rebobinar.
- Cenas 1–4: `PH2D_CODE_SMOKE=1..4` (como no I5).

## 9. ABERTO

- **Declarados** ([05 §6–§7](../05_api_que_funciona.md)):
  - a roda do rato e o IME não chegam ao jogo;
  - o teclado do jogo também chega aos atalhos do editor;
  - nomes iguais na cena são permitidos;
  - `ph2d.wait` numa corrotina do artista devolve a marca;
  - a mensagem dos erros que passam pelas funções embrulhadas leva o prefixo `ph2d:NN:`;
  - `shift` dá `key_pressed` quando o 2.º lado desce;
  - o `on_signal` não vê bordas num quadro com tique;
  - chaves inteiras do estado acima de 2^53 perdem precisão;
  - um clique da alça da âncora ou da onda do áudio chega ao jogo;
  - `math.random(1, 2^40)` dá erro com mensagem enganadora.
- **Do handoff da Ajuda §9 que continuam:**
  - (c) as páginas não estão na lista de scripts;
  - (d) a página re-disposta guarda o scroll em pixels;
  - (e) o `fotografa_cena.sh` devia guardar a 2.ª captura;
  - (f) o prefixo do chunk do `eval`;
  - (g) os do I13 §9.
  - O (a) foi curado neste fecho; o (b) é este item 6.
- **Integração:** a entrada do CLAUDE.md §5.1 já diz `PH2D_CODE_SMOKE=1..6`; o link do «Último» troca para este
  handoff quando integrar.
