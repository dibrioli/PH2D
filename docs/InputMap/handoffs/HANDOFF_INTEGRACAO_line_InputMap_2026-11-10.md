# HANDOFF de integração — `line/InputMap`, 10/11 (Input Map vira aba central: A · List, B · Keyboard, C · Try it + tela «Controls» do jogador)

Leitor: o agente integrador. Branch `line/InputMap`, código até `d019c09ff5` (+ este handoff), **86 commits** sobre `bc4a6ce930` = merge-base = ponta do `main` (fecho 10/11: o `main` não andou, sem rebase; `--ff-only` serve). `git diff --shortstat main...HEAD`: 256 files, +17 072 −2 755. Nada integrado, nenhum push. **Suíte completa, `nextest-impacted` e `ship.sh` NÃO correram** (regra do dono 23/10): são do integrador. Plano e contratos: [01_plano.md](../01_plano.md) (§10–§12 = o mapa; o código manda). Prompt da missão: [PROMPT_MISSAO](PROMPT_MISSAO_InputMap_2026-11-10.md).

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

**Removido:** a janela flutuante do Input Map (−1 211 editor-core, −188 shell, −1 023 testes, −48 i18n); scrollbar id **842 reformada, NUNCA reutilizada** (`scrollbar_ids.rs`; confira o próximo livre no `main` do dia); `resolve_player_input` (`ph2d-app-physics`, sem chamador); `ProjectFile.input_map` (o mapa vive só em `ProjectState.input_map`).

## 3. Foundational/compartilhado tocado — por quê

| Onde | O que | Natureza |
|---|---|---|
| `ph2d-editor-core` | `DocumentSurface::raw_key` (`doc_surface.rs:309`, default append-only); menu → `request_input_map_tab` (`input_map_ops.rs:12`); janela removida (`chrome/input_map*`); slot de escuta partilhado agora `Option<Binding>`: `capture_binding`/`take_captured_binding` (`:68/:77`) | aditivo + **remoção** (linha que use a janela/ids da janela não compila) |
| `ph2d-editor-core` icons | `ArrowUp/Down/Left/Right` | aditivo |
| `ph2d-tokens` | `Categorical01..12` + `CATEGORICAL`/`categorical(i)`; `preview-sky`, `preview-ground`, `input-map-highlight`, `input-map-warning`, `input-map-warning-bg` (+ painel Tokens: família «Category Colors») | aditivo, ver §4 |
| `shells/desktop` | `prefs.input_map_view`; `ProjectState.input_map` (`undo.rs:100`, `undo_app.rs`), gesto em curso em `motivo_da_supressao`; `ProjectFile.game_id` (`project.rs`); `input_dispatch/controles_do_jogador.rs` (`play_map`); porta tecla/clique via `ph2d_player_controls::host`; `undo_jogo.rs`; leitores em `render_loop/fase_*` | composição |
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
  merge-base bc4a6ce930   ·   86 commit(s)   ·   256 arquivo(s)
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
**Schema como DELTA:** `PROJECT_SCHEMA` **+1** (231 → 232: `InputMap.controls`, `ProjectFile.game_id`, mapa no undo) nos três sítios (escada, `project_schema.rs`, tripla em `project_schema_tests.rs`); `recontar.py --apenas schema` ✓. **Reconte pelo `main` do dia** (`schema-recount.py`): se outra linha subiu, esta vira `main + 1`. Sem migração (§0.11). Componentes: DELTA 0 (`ph2d-ecs` 121). §6 intocado.

## 6. Decisões técnicas e o porquê

- **`hidden_in_game`** (não `shown_in_game`): o default «todas as ações aparecem» não precisa de entrada; o projeto só guarda o que o autor escondeu.
- **O mapa entra no undo do projeto por diff**; cada `InputMapEdit` = UM passo, Ctrl+Z devolve o mapa bit a bit (postcard). O arrasto de zona é um passo (gate `motivo_da_supressao`). «Forget them» NÃO é passo (é do computador do jogador).
- **Play restaura o documento inteiro no Stop**, mapa incluído, + **faixa de aviso na aba** durante o Play.
- **Não existe jogo exportado no app**; a tela «Controls» mora no runtime do Play (F1) e um export herda: o «Play e exportado» está cumprido no que existe.
- **`game_id`** sorteado na 1.ª chamada (relógio + pid + contador, `DefaultHasher` — HR-5; nunca 0). Ficheiro do jogador `~/.ph2d/players/<16 hex>.txt`, **id 0 = sem ficheiro** (`player_file_of`); smokes fora do disco (`keep_off_disk`).
- **F1** abre a tela, só com «Players can change keys»; recusa Esc/F5 como tecla de abrir. Aberta, é dona do teclado e do clique e pausa o relógio. Com a opção desligada o Play ignora o ficheiro (ele fica intacto).
- **Tecla repetida do jogador = AVISO, nunca troca** (guia Xbox 107, TLOU2, demo do Godot).
- **Um mapa efetivo (`play_map`) para os CINCO leitores do Play** (resolve do jogador, Luau, morph/vector, sinais, `acoes_premidas` da física); o gate de ordem recusa `|h| &h.input_map`.
- **Frase do hover da B SUBSTITUI a legenda** (R7b; supera o «sempre reservada» do §12 do plano).
- LB/RB/Back/Start/cruz no comando desenhado (ligáveis). **Sem ◆ em Dead/Press** (o ◆ é a marca do Auto Key; aqui seria controlo morto).
Censo de textos (0 cortados, 0 números mortos) em `Default` e `CrispHeavyPlus` com o registo de elisão do próprio pintor, a 1600×1000 e 1280×720.

## 7. O que um merge pode partir

1. **Schema/tripla** (§5) — conte contra o `main` do dia.
2. **`ph2d-editor-core`**: linha que ainda use a janela do Input Map, o slot de escuta como `Binding` solto, ou o id 842.
3. **`ph2d-i18n/src/lib.rs`** e `tokens.json`/`ph2d-tokens` (listas ordenadas de chaves; `color.rs` está no tecto 700, `fase_snapshots_publish` no 200).
4. **Catraca da shell** (`the_shell_only_shrinks`): **189 034 / tecto 189 041** no fim (R13; nenhum número subiu — a cura da R10a foi MOVER para `ph2d_player_controls::host`). Folga de 7 linhas: se outra linha cresceu a shell, a soma passa — mover, nunca subir. **Re-meça.**
5. **`ph2d-app-input-map` no tecto 449**; `project_input_map_audit_det_tests.rs` foi fundido em `project_input_map_tests.rs`.
6. **HR-15 da soma**: verde na linha (§8); um literal novo de outra linha acorda-se só na soma.
7. Cenas de smoke 6..18 (§4) e `.config/testes-lentos.tsv` (gerado).

## 8. Prova de fecho (feita na linha)

- Bateria de fecho **verde**: `cargo clippy --workspace --all-targets -D warnings`, `cargo fmt`, `cargo machete` (saiu `tracing` do app-input-map), `check-standalone-optional.sh`, `check-workflow-packages.sh`. **Nenhum `#[cfg(target_os` escrito ou movido** (sem cruzamento macOS).
- **Auditoria** por duas lentes independentes (correção e wiring · determinismo e persistência) + três revisões visuais lado a lado com a maquete; 9 defeitos curados com teste VERMELHO→verde (escuta armada após trocar de vista, Ctrl/F5 engolidos, Shift na busca, Forget no Play, número atravessando a troca de mapa, ordem dos leitores, 5.º leitor, smoke no `~/.ph2d` real, `game_id` 0).
- **Censos da árvore combinada** (`d019c09ff5`): **133/133 verdes**, 12 de 12 censos (HR-15, `file_loc_caps`, `the_shell_only_shrinks*`).
- **Impactados e suíte completa ficam para a integração.**

### Provas de mutação

Rodada 1: **28 mutações** nas quatro leis → **20 sangraram, 8 sobreviveram**:

| Lei | Sobrevivente da rodada 1 |
|---|---|
| 1 · Dead/Press só com manche | `is_analog` ignorando gatilho (botão/eixo) na UI das três vistas |
| 2 · tecla repetida e «Nobody reads» | apresentação do aviso: pé da List, cartão da Keyboard, «nobody» do hover |
| 3 · resolução Pad no Play | `PadButton`→0 fora de `ph2d-input` (dedo do jogador, mapa efectivo) |
| 4 · mapa do jogador por cima do projeto | o corpo de `play_map` |

Rodada 2 (`d019c09ff5`): `shells/game-scenes/tests/it/input_map/mutacao_r13.rs` (5 provas), `ph2d-physics-ecs/src/player_input_tests.rs::a_pad_button_bound_to_jump_arms_the_jump`, `ph2d-player-controls/src/play_tests.rs::the_effective_map_still_resolves_a_pad_button`, `shells/desktop/src/project_input_map_tests.rs::o_play_le_o_mapa_do_jogador_e_fora_dele_o_do_projecto` ⇒ **as 8 sangram, e os 3 controlos também**; o censo passou a gravar fundo e faixa de cada aviso. ⚠️ **Armadilha:** `mut.py` restaura com `git checkout HEAD -- <file>` — comite antes de mutar.

### Binário do smoke

Após apagar `target/{debug,smoke}/incremental`: 1.ª corrida 129 `Compiling`, 50,5 s; **2.ª** (zero `Compiling`, código `d019c09ff5`):

```
▸ linha line_inputmap · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.26s
```

### Perfil do loop do agente (`bash scripts/agent-loop-profile.sh`, 20 sessões)

```
✗ paralelismo de ferramenta        1.20/passo   alvo >= 1,5  (11% dos passos com 2+ chamadas)
✗ cargo test : cargo check         219 : 72     razao 3.0x (alvo <= 1,0; baseline 4,3x)
✓ edições conferidas 1.2% · ✓ contexto/passo 228 mil · ✓ início 67 mil · ✓ respostas/sessão 116
```

## 9. Smoke (do dono; o que NÃO foi smokado: tudo que é clique real e o gamepad físico)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap && PH2D_GAME_SMOKE=<6..18> bash scripts/ph2d-run.sh cargo run -p ph2d-host-desktop --profile smoke` (o build do §8; cena 18 junta `PH2D_GAME_SMOKE_DRIVE=editor`). O menu abre o Input Map como aba no meio, seletor List · Keyboard · Try it no topo. Fotos e 13 montagens nossa|maquete em `docs/InputMap/fotos/`. Não smokado por humano: F1 no Play (provado por gates e Mesa), comando real (gilrs → `deliver_pad`), `Forget them` com disco real.

## 10. ABERTO e achados FORA da linha (para outras linhas)

- **Acabamento fechado (R13):** a prévia das 11 ações lê-se na aba REAL (≥ 3/4, abas das fotos `c1_1600`/`c1_1280`); B3: LT/RT na aresta do topo, LB/RB dentro do corpo; P1 com o mesmo espaço em cima e em baixo. Nada visual em aberto.
- Sem jogo exportado no app (§6): a tela herda quando o export existir.
- **Fora da linha, não tocado:** o editor do Animated Sprite e a aba de região, o Planner e a aba da física NÃO implementam `chrome_over` (o rodapé pinta sob o HUD de baixo); o modo de texto de fábrica `CrispHeavyPlus` faz sumir os pingos de i/j a ~13 px em negrito (app inteiro, ex. «33 entitles»); a Hierarquia trunca «Collision Shap…».
- Marcadores de conflito: nenhum. `main.rs` 1 030/600 é pré-existente.

## 11. Proposta para o `CLAUDE.md §5.1` (só na integração — DIRETRIZ §1.5.9 item 8; a linha NÃO a escreve)

Frase (≤ 300 caracteres) e entrada ≤ 700 bytes, a trocar o link ao integrar:

> - **Input Map** — aba central com três vistas (List · Keyboard · Try it), mapa no undo do projeto, tela «Controls» do jogador (F1 no Play), Luau pela ação (`ph2d-input-map-editor`, `ph2d-app-input-map`, `ph2d-player-controls`). Smoke `PH2D_GAME_SMOKE=6..18`. Último: [aba e tela do jogador, 10/11](docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_2026-11-10.md) · [plano](docs/InputMap/01_plano.md)

(`CLAUDE.md` 33 878 B de 40 KB.) **Memória:** `project_inputmap_three_versions_2026_11_10.md` ganhou o parágrafo «Entregue 10/11»; a linha do `MEMORY.md` não muda.
