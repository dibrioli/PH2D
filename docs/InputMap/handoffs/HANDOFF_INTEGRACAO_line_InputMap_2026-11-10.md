# HANDOFF de integração — `line/InputMap`, 10/11 (Input Map vira aba central: A · List, B · Keyboard, C · Try it + tela «Controls» do jogador)

Leitor: o agente integrador. Branch `line/InputMap`, HEAD `b72e9f9945` (+ 3 acabamentos visuais de uma R12b em curso na hora de escrever; o integrador lê o HEAD real com `git log -1`), **82 commits** sobre `bc4a6ce930` = merge-base = ponta do `main` (conferido 10/11; `--ff-only` serve). `git diff --shortstat main...HEAD`: 253 files, +16 592 −2 755. Nada integrado, nenhum push. **Suíte completa, `nextest-impacted` e `ship.sh` NÃO correram** (regra do dono 23/10): são do integrador. Plano e contratos: [01_plano.md](../01_plano.md) (§10–§12 = o mapa; o código manda). Prompt da missão: [PROMPT_MISSAO](PROMPT_MISSAO_InputMap_2026-11-10.md).

## 1. Ordem do dono (10/11) e as 4 decisões — todas cumpridas

*«Implementaremos as 3 versões sugeridas mantendo a boa aparência das 3 e com um seletor de tipo de input map no topo do quadro central.»* (1) A tela de controles do jogador FUNCIONA no Play (F1) — ver §6 sobre o «exportado». (2) Projeto novo nasce com as 8 ações de fábrica; receitas em «Start from…» (um passo no undo, com confirmação). (3) A aba abre na última versão usada (pref `input_map_view`; 1.ª vez B). (4) Luau lê pela ação: `ph2d.action_{down,pressed,released,strength}`.

## 2. Crates novas e o que mudou nas antigas

| Crate | Papel |
|---|---|
| `ph2d-input-map-editor` | pintura da aba: doc, `View`, `Snapshot`, `InputMapEdit`, `edit::apply`, 3 vistas (`view_list`, `view_keyboard`, `view_try`), `raw_keys` (filtro de teclas que a aba nunca liga) |
| `ph2d-app-input-map` | a família (`FAMILY = "input-map"`), `DocumentSurface` de uma aba, `install/frame/view/surface_raw_key`, `listen::{pad_binding, deliver_pad}`, `gesture_in_progress`. **No tecto de LOC (449)** |
| `ph2d-player-controls` | tela «Controls» do jogador: `paint`/`paint_preview`, `ControlsScreen`, `PlaySession` (`keep_off_disk`, `forget`), `GameId` + `new_game_id`, `player_file_of`, `file::{load,save}`, `host.rs` (`key`/`click`/`Clock`, a porta que a shell só traduz) |
| `ph2d-input-map-scene` | cenas de smoke (folha própria, catraca das `ph2d-app-*`) |
| `ph2d-input` (existente) | ganha `controls`, `recipes`, `readers`, `player_keys`; `strength_of`/`pressed_of` públicas; `is_analog` cobre gatilho como botão e eixo |
| `ph2d-script` | `action_api.rs` (`ph2d.action_*`, `Devices::feed_actions`), `runtime/luau/ph2d.d.luau`, `api_help_actions.rs` |

**Removido:** a janela flutuante do Input Map (−1 211 editor-core, −188 shell, −1 023 testes, −48 i18n, R2a); scrollbar id **842 reformada, NUNCA reutilizada** (`scrollbar_ids.rs`: próximo livre = 842 antes; confira o próximo livre no `main` do dia); `ph2d_app_physics::player_input::resolve_player_input` apagada (sem chamador desde a R3b); `ProjectFile.input_map` (o mapa vive em `ProjectState.input_map`, uma fonte só).

## 3. Foundational/compartilhado tocado — por quê

| Onde | O que | Natureza |
|---|---|---|
| `ph2d-editor-core` | `DocumentSurface::raw_key` (`doc_surface.rs:309`, default append-only); menu → `request_input_map_tab` (`input_map_ops.rs:12`); janela removida (`chrome/input_map*`); slot de escuta partilhado agora `Option<Binding>`: `capture_binding`/`take_captured_binding` (`:68/:77`) | aditivo + **remoção** (linha que use a janela/ids da janela não compila) |
| `ph2d-editor-core` icons | `ArrowUp/Down/Left/Right` | aditivo |
| `ph2d-tokens` | `Categorical01..12` + `CATEGORICAL`/`categorical(i)`; `preview-sky`, `preview-ground`, `input-map-highlight`, `input-map-warning`, `input-map-warning-bg` (+ painel Tokens: família «Category Colors») | aditivo, ver §4 |
| `shells/desktop` | `prefs.input_map_view` (+ `forwarding_persist.rs:42`); `undo.rs:100` `ProjectState.input_map`, `do_hero/ao_hero` (`undo_app.rs`), gesto em curso em `undo_app_macros.rs::motivo_da_supressao`; `ProjectFile.game_id`, `App::game_id()` (`project.rs`); `input_dispatch/controles_do_jogador.rs` (`play_map`), porta da tecla/clique (`keyboard.rs`, `input_dispatch.rs`, via `ph2d_player_controls::host`); `undo_jogo.rs` (`jogador_entra/sai`); leitores em `render_loop/fase_*` | composição |
| `ph2d-app-registry-init`, `ph2d-app-sync`, `ph2d-app-game`, `ph2d-app-physics`, `ph2d-app-components` (`script_input.rs`) | registo da família, `CENAS`, `resolve_player_input` fora, mapa ao script | composição |
| `ph2d-i18n` | `input_map_tab/_list/_keyboard/_try.rs`, `player_controls.rs` (um ficheiro por dono); `input_map.rs` encolhe | aditivo |
| `Cargo.lock` | 4 `+name` internos, nenhum externo | — |

**Mesmo-símbolo a grepar numa fusão:** as 5 chaves de token em `tokens.json` (nos 4 temas), a tripla do schema, `CENAS` em `ph2d-app-game/src/smoke.rs:23`, `scrollbar_ids.rs`, os ficheiros i18n acima e o `mod` de `ph2d-i18n/src/lib.rs`.

## 4. Valores literais que colidem (de `docs/design/tokens.json`)

- `categorical-01..12`: valores em `docs/design/tokens.json` (escuro forge/workshop `oklch(0.8019 0.1119 231.99)` … `(0.7792 0.0401 359.821)`; claro sunstone/blueprint mesmo matiz/croma, L 0.5598–0.5966, ≥ 3,2:1 sobre bg-1..3).
- `preview-sky` `oklch(0.94 0.025 205)` e `preview-ground` `oklch(0.76 0.13 143)` — iguais em todo tema.
- `input-map-highlight`: escuro `oklch(0.8741 0.152 88.1114)` (#FFCF4D) · claro `oklch(0.84 0.16 85)`. `input-map-warning`: `oklch(0.8238 0.1408 70.1707)` · claro `oklch(0.58 0.15 62)`. `input-map-warning-bg`: `oklch(0.3654 0.0071 3.7044)` · claro `oklch(0.935 0.035 75)`. Todos papel `Graphic`; emprestados nos derivados (`derive.rs`).
- Smoke: `PH2D_GAME_SMOKE` **6..18** (`CENAS = 18`): 6 A1 · 7 A2 · 8 A3 · 9 A4 · 10 B1 · 11 B2 · 12 B3 · 13 B4 · 14 C1 · 15 C2 · 16 C3 · 17 C4 · 18 P1 (P1 pede `PH2D_GAME_SMOKE_DRIVE=editor`). **Se outra linha tomou cenas após 5, renumere pelo `main` do dia** (conte, não escolha).

## 5. Superfície de colisão (colada de `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap/scripts/collision-surface.sh`, 10/11)

```
SUPERFÍCIE DE COLISÃO — line/InputMap contra main
  merge-base bc4a6ce930   ·   82 commit(s)   ·   253 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        232   (base: 231)
  ⚠   └ tripla do gate               (232, 13, 29)   (base: (231, 13, 29))
    VEC_SCENE_SCHEMA 29=29 · FLIP_SCHEMA 13=13 · DOC_VERSION (timeline) 25=25
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS
▸ REGISTRO DE COMPONENTES
    ph2d-ecs 121 (base: 121)
  ✗ ph2d-render / ph2d-script (espelho = ecs + K)   —   (SONDA CEGA, confira à mão; a linha não toca o registo)
▸ CONTRATO CONGELADO (§6)
    node.rs  intocado   ·   tool.rs  intocado
▸ ADR   último 0200, próximo livre 0201 — esta linha não cria ADR
▸ Cargo.lock — 4 '+name' novos, todos internos:
    ph2d-app-input-map · ph2d-input-map-editor · ph2d-input-map-scene · ph2d-player-controls
▸ TETOS DE LOC
  ✗  1030 / 600   shells/desktop/src/main.rs   (já estava assim: a linha NÃO o fez crescer; a catraca é a `the_shell_only_shrinks`)
```
**Schema como DELTA:** `PROJECT_SCHEMA` **+1** (231 → 232: `InputMap.controls`, `ProjectFile.game_id`, mapa na captura do undo), nos três sítios (escada, const em `shells/desktop/src/project_schema.rs`, tripla `(232, 13, 29)` em `project_schema_tests.rs`). `python3 scripts/recontar.py --apenas schema` → `✓ PROJECT_SCHEMA = 232 (tripla (232, 13, 29))`, `✓ recontar: tudo conta`. **Reconte pelo `main` do dia** (`git show main:shells/desktop/src/project_schema.rs`; `python3 scripts/schema-recount.py`): se outra linha subiu, esta vira `main + 1`. **Sem migração** (CLAUDE.md §0.11). Registo de componentes: DELTA 0 (o `ph2d-ecs` fica em 121). §6 intocado: nenhum `NodeOp/OpResolver/NodeManifest/Tool/VectorOp` tocado.

## 6. Decisões técnicas e o porquê

- **`hidden_in_game`** (não `shown_in_game`): o default «todas as ações aparecem» não precisa de entrada; o projeto só guarda o que o autor escondeu.
- **O mapa entra no undo do projeto por diff**; cada `InputMapEdit` = UM passo, Ctrl+Z devolve o mapa bit a bit (postcard). O arrasto de zona é um passo (gate `motivo_da_supressao`). «Forget them» NÃO é passo (é do computador do jogador).
- **Play restaura o documento inteiro no Stop**, mapa incluído (mesma lei de toda edição) + **faixa de aviso na aba** durante o Play (`InputMapWarningBg/Warning`).
- **Não existe jogo exportado no app**; a tela «Controls» mora no runtime do Play (F1) e um export herda. Por isso o «Play e exportado» do dono está cumprido no que existe.
- **`game_id`** sorteado na 1.ª chamada (não há comando «New project»); hash de relógio + pid + contador do processo (`DefaultHasher`; HR-5 proíbe `RandomState`); nunca 0. **Ficheiro do jogador `~/.ph2d/players/<game_id 16 hex>.txt`; id 0 = sem ficheiro** (`player_file_of`). Smokes não tocam o disco (`PlaySession::keep_off_disk`).
- **F1** abre a tela, só com «Players can change keys»; recusa Esc/F5 como tecla de abrir. Aberta, é dona do teclado e do clique e pausa o relógio. Com a opção desligada o Play ignora o ficheiro (ele fica intacto).
- **Tecla repetida do jogador = AVISO, nunca troca** (guia Xbox 107, TLOU2, demo do Godot).
- **Um mapa efetivo (`play_map`, `controles_do_jogador.rs`) para CINCO leitores do Play**: resolve do jogador (`fase_pointer_subjects.rs`), Luau (`app_state_devices.rs`), morph/vector (`fase_vector_view_and_drives.rs`), sinais (`fase_signal_outbox.rs`) e `acoes_premidas` no passo da física. Gate de ordem na crate e na Mesa recusa `|h| &h.input_map`.
- **Frase do hover da B SUBSTITUI a legenda** (R7b; o §12 do plano diz «sempre reservada» — está desatualizado: sem salto de layout, sem linha de repouso).
- LB/RB/Back/Start/cruz no comando desenhado (ligáveis). **Sem ◆ em Dead/Press** (o ◆ é a marca do Auto Key; aqui seria controlo morto).
Censo de textos (0 cortados, 0 números mortos) em `Default` e `CrispHeavyPlus` com o registo de elisão do próprio pintor, a 1600×1000 e 1280×720.

## 7. O que um merge pode partir

1. **Schema/tripla** (§5) — conte contra o `main` do dia.
2. **`ph2d-editor-core`**: linha que ainda use a janela do Input Map, o slot de escuta como `Binding` solto, ou o id 842.
3. **`ph2d-i18n/src/lib.rs`** e `tokens.json`/`ph2d-tokens` (listas ordenadas de chaves; `color.rs` está no tecto 700, `fase_snapshots_publish` no 200).
4. **Catraca da shell** (`the_shell_only_shrinks`): fechou em **189 018 contra o tecto 189 041** na R10a (sem subir número; a cura foi MOVER para `ph2d_player_controls::host`). Se outra linha cresceu a shell, a soma pode passar — a cura é mover, nunca subir. **Re-meça:** `the_shell_only_shrinks`.
5. **`ph2d-app-input-map` no tecto 449**; `project_input_map_audit_det_tests.rs` foi fundido em `project_input_map_tests.rs`.
6. **Censo de texto (HR-15) da árvore combinada**: `bash scripts/censos-da-arvore-combinada.sh` — a linha introduziu muitas chaves i18n; um literal novo de outra linha acorda-se só na soma.
7. Cenas de smoke 6..18 (§4) e `.config/testes-lentos.tsv` (gerado).

## 8. Prova de fecho (feita na linha)

- Bateria de fecho **verde**: `cargo clippy --workspace --all-targets -D warnings`, `cargo fmt`, `cargo machete` (saiu `tracing` do app-input-map), `check-standalone-optional.sh`, `check-workflow-packages.sh`. **Nenhum `#[cfg(target_os` escrito ou movido** (sem cruzamento macOS).
- **Auditoria por duas lentes independentes** (correção e wiring · determinismo e persistência) **+ três revisões visuais independentes** (lado a lado com a maquete). Defeitos confirmados e curados com teste visto VERMELHO→verde: escuta armada após trocar de vista/fechar a aba; Keyboard engolindo Ctrl/F5; busca da List apagando texto com Shift; Forget durante o Play; número digitado atravessando a troca de mapa; ordem dos leitores por arquétipo; quinto leitor do Play; smoke lendo o `~/.ph2d` real; `game_id` 0 partilhando ficheiro.
- **Impactados e suíte completa ficam para a integração.**

### Provas de mutação

> TODO (coordenador): quatro leis — Dead/Press só com manche · aviso de tecla repetida e «Nobody reads» · resolução Pad no Play · mapa do jogador por cima do projeto. Cada mutação deve sangrar SÓ o gate certo. Colar a tabela aqui.

### Binário do smoke

> TODO (coordenador): depois do último commit e de `rm -rf …/Worktrees/line-InputMap/target/*/incremental`, `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` 2×; colar a 2.ª saída («Finished», zero «Compiling»).

### Perfil do loop do agente (`bash scripts/agent-loop-profile.sh`, 20 sessões)

```
✗ paralelismo de ferramenta        1.20/passo   alvo >= 1,5  (11% dos passos com 2+ chamadas)
✗ cargo test : cargo check         219 : 72     razao 3.0x (alvo <= 1,0; baseline 4,3x)
✓ edições conferidas 1.2% · ✓ contexto/passo 228 mil · ✓ início 67 mil · ✓ respostas/sessão 116
```

## 9. Smoke (do dono; o que NÃO foi smokado: tudo que é clique real e o gamepad físico)

Comando (a worktree, **mesmo perfil e árvore do build do §8**): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap && PH2D_GAME_SMOKE=<6..18> bash scripts/ph2d-run.sh cargo run -p ph2d-host-desktop --profile smoke`. Cena 18 junta `PH2D_GAME_SMOKE_DRIVE=editor`. Uma variável de ambiente NÃO é um build novo. Menu: abrir o Input Map pelo menu (abre a aba no meio, seletor List · Keyboard · Try it no topo). Fotos e 13 montagens nossa|maquete em `docs/InputMap/fotos/` (40 ficheiros; 1600×1000 e 1280×720).
Não smokado por humano: troca de tecla pelo jogador no Play com F1 (provado por gates e Mesa), comando real (gilrs → `deliver_pad`), `Forget them` com disco real.

## 10. ABERTO e achados FORA da linha (para outras linhas)

- Sem jogo exportado no app: a tela herda no dia em que o export existir.
- **Fora da linha, não tocado:** o editor do Animated Sprite e a aba de região, o Planner e a aba da física NÃO implementam `chrome_over` (o rodapé pinta sob o HUD de baixo); o modo de texto de fábrica `CrispHeavyPlus` faz sumir os pingos de i/j a ~13 px em negrito (app inteiro, ex. «33 entitles»); a Hierarquia trunca «Collision Shap…».
- Marcadores de conflito: nenhum. `main.rs` 1 030/600 é pré-existente.
- `01_plano.md` §12 («hover sempre reservado») foi superado pela R7b (§6).

## 11. Proposta para o `CLAUDE.md §5.1` (só na integração — DIRETRIZ §1.5.9 item 8; a linha NÃO a escreve)

Frase (≤ 300 caracteres) e entrada ≤ 700 bytes, a trocar o link ao integrar:

> - **Input Map** — aba central com três vistas (List · Keyboard · Try it), mapa no undo do projeto, tela «Controls» do jogador (F1 no Play), Luau pela ação (`ph2d-input-map-editor`, `ph2d-app-input-map`, `ph2d-player-controls`). Smoke `PH2D_GAME_SMOKE=6..18`. Último: [aba e tela do jogador, 10/11](docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_2026-11-10.md) · [plano](docs/InputMap/01_plano.md)

(`CLAUDE.md` 33 878 B de 40 KB.) **Memória:** `project_inputmap_three_versions_2026_11_10.md` ganhou o parágrafo «Entregue 10/11»; a linha do `MEMORY.md` não muda.
