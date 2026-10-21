# HANDOFF DE INTEGRAÇÃO — line/CodeEditor, Onda 2 (item 7: de quem é o teclado com a cena a correr) · 20/10/2026

> Leitor: o agente integrador e a próxima LLM da linha. **Não integrado, não enviado.** A linha continua por
> integrar no seu todo desde o [I0–I5](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md) (17/10, último no
> `main`). O [handoff do item 6 (a API)](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-20_ONDA2_I6_A_API.md) **continua
> válido na sua parte** (API de scripts, `ph2d-input`/`ph2d-script`, cena 6, os seus ABERTOS); este cobre o item 7 POR
> CIMA dele. Lei, oráculo e decisões do dono: [06_posse_do_teclado.md](../06_posse_do_teclado.md).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` · HEAD
  `eb371d4963` (+ o commit de docs deste fecho) · merge-base `100d34c8e0` (= `main` do dia) · **99 commits** · 646 ficheiros.
- Commits do item 7: `4b5020cf56` oráculo Godot + plano 06 · `cd7f0585dd` lei + composição + moldura + gate de ordem ·
  `8676c6e840` roteiro conduzido 15/15 · `eaeac6df3c` curas das duas lentes da auditoria + gates do dedo re-ancorados ·
  `eb371d4963` testes dos 2 sobreviventes da mutação + fixtura.

## 2. Superfície de colisão (colada de `scripts/collision-surface.sh`, 20/10 — REFERÊNCIA, não evidência)

```
SUPERFÍCIE DE COLISÃO — line/CodeEditor contra main
  merge-base 100d34c8e0   ·   99 commit(s)   ·   646 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        191   (base: 191)
      └ tripla do gate               (191, 13, 23)   (base: (191, 13, 23))
    VEC_SCENE_SCHEMA                       23   (base: 23)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 20   (base: 20)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs                              108   (base: 108)
    ph2d-render (espelho)                 109   (base: 109)
    ph2d-script (espelho)                 109   (base: 109)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR
    último no disco: 0188   próximo livre: 0189
    esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 3 pacote(s) '+name' novo(s):
      "ph2d-code-explain"
      "ph2d-code-format"
      "ph2d-code-help"
▸ MARCADORES DE CONFLITO
    nenhum nos arquivos da linha
▸ TETOS DE LOC nos arquivos que a linha tocou
  ✗  1059 / 600   shells/desktop/src/main.rs
```

Resumo: schemas e registos = base; §6 intocado; nenhum ADR; os 3 pacotes do `Cargo.lock` são internos e vêm de
itens anteriores (regenerar, nunca à mão); o ✗ de LOC é a isenção pré-existente do `main.rs`.

## 3. Foundational / partilhado tocado — e porquê é aditivo

- **`ph2d-app-components`:** `posse_do_jogo` novo — a lei pura (`Cena`, `FocoDoJogo {quadro, tecla, clique, premido,
  tem_o_teclado}`, `Tecla {desceu, acorde, shift, f5, repete}`, `Transporte`, `Destino`, `destino()`), 15 testes
  unitários. `posse_do_jogo_registo` novo — sonda `PH2D_POSSE_LOG=<ficheiro>`, só escreve, modo append. Uma entrada FORA
  em `tests/it/every_word_this_family_shows_comes_from_the_string_table.rs` (linhas da sonda nunca se pintam).
- **`ph2d-editor-core`:** `ViewState::jogo_com_o_teclado: bool` (campo pub, `Default` false; `ViewState` não tem
  literal de struct fora do seu `Default` — aditivo) · `paint_canvas_overlays::paint_game_frame` (`ColorToken::Accent`,
  `StrokeToken::Thick`, dentro de `last_content`, abaixo das barras/painéis flutuantes; `paint::stroke_rect`, não uma
  moldura de controlo).
- **Shell (só composição):**
  - `shells/desktop/src/input_dispatch/posse_do_jogo.rs` novo: `cena_do_jogo` (`a_vista` = nenhuma aba de documento/
    superfície, nenhuma entrada de texto, nenhum menu de contexto/modal Fill, nenhuma paleta de comandos, nenhuma
    pré-visualização do Morph) · `quadro_do_foco_do_jogo` (no início de `run_render_frame`, antes de
    `fase_pointer_subjects` e dos scripts) · `tecla_do_jogo` · `rato_do_jogo` · `menu_aberto` · `clique_so_do_jogo`
    (Down/Up do HUD + laboratório de física: `poke_press` para explosão/atração; `mao_no_jogo` = o agarrável mais ao
    topo por `hover_highlight::pick_objects_at` + `body_grab::take_hold`, e selecciona-o) · `alterna_o_relogio` = a
    ÚNICA porta do Espaço e do F5.
  - `keyboard.rs`: o bloco antigo *«o dedo do jogador observa, nunca consome»* (W3) é SUBSTITUÍDO por
    `self.tecla_do_jogo(code, pressed, repeat)`, depois da captura de bind e da paleta e ANTES do toggle do F9 e de todo
    atalho. Shift+F5 empurra `EditorAction::Transport(TransportCmd::Reset)` = o botão Rebobinar (mesmo escoamento do
    barramento, `runtime_intent` StopScene).
  - `input_dispatch.rs::on_mouse_input`: `rato_do_jogo` passa para depois de `ramo_aperto_solta_teclado` (blur) e
    antes do 1.º `return`; o premir/soltar só-do-jogo → `clique_so_do_jogo` + `return`; `menu_open_before = self.menu_aberto()`.
  - `handlers_teclas_editor.rs`: o braço do Espaço → `alterna_o_relogio`. `despacho_metodos_janela_e_vetor.rs`: o
    `rato_do_jogo` antigo sai; `hero_ref` passa a `pub(super)`.
  - `ComponentsShell` ganha `foco_do_jogo` (a catraca dos campos da `App` fica em 172).
- **Gates mudados:**
  - `shells/desktop/tests/it/the_players_finger_reaches_the_bridge.rs` — os dois testes de texto-fonte do bloco antigo
    (observa-sem-consumir, acorde-no-bloco) re-ancorados à porta POR COMPORTAMENTO (`destino`): sem o jogo a ter o
    teclado a seta não é consumida (nudge do Vector); um acorde nunca chega ao jogador; o soltar passa sempre.
  - `morph_arrow_seam_tests.rs` — a agulha `self.input.apply_event` → `self.tecla_do_jogo(` (a ordem feed < modal < flip mantém-se).
  - **Novo** `shells/desktop/tests/it/a_posse_do_teclado_esta_na_ordem.rs` (4 testes).

## 4. O que um merge pode partir

- Qualquer linha que toque o bloco observador do `keyboard.rs` ou o prólogo de `on_mouse_input`.
- Qualquer linha que ACRESCENTE linhas à shell: a catraca `the_shell_only_shrinks` está a **189 008** (medido depois da
  última cura) contra o tecto **189 041** — 33 linhas de folga, que **soma entre linhas**.
- O campo novo em `ViewState`; um literal de `ViewState` noutra linha deixa de compilar.
- **Semântica:** os `keydown` do `KeyboardState` só chegam ao `InputState` enquanto o jogo tem o teclado, ou na
  pré-visualização do Morph. Outra linha que leia `InputState`/`ActionState` FORA do jogo fica muda. Leitores auditados:
  `player_input` (a física só avança com a cabeça a andar), gatilhos (só a tocar), morph (só na pré-visualização);
  nenhum na UI do Input Map.

## 5. Contratos congelados

**Nenhum.** `node.rs` e `tool.rs` intocados; nenhum `PROJECT_SCHEMA`, registo ou ADR (§2).

## 6. Portões do fecho (20/10, MEDIDOS)

| Portão | Resultado |
|---|---|
| `cargo fmt --check` · clippy `--workspace --all-targets --all-features -D warnings` (43,8 s) · `CARGO_BUILD_WARNINGS=deny check --workspace --all-targets` | ✓ |
| `censos-da-arvore-combinada.sh` | ✓ 121/121 |
| `check-standalone-optional` · `check-workflow-packages` · `cargo machete` (3 crates) | ✓ |
| `file_loc_caps` 4/4 · `arch_safe_clamp_only` · `the_shell_only_shrinks` · `the_app_only_sheds_fields` 3/3 | ✓ |
| `nextest-impacted` (fecho), passagem 1 | 18 559 de 19 344 corridos em 1 666,9 s; 18 536 ok; 2 FAIL reais + 5 TIMEOUT; o prazo de 30 min da fatia deu SIGTERM a 16 e deixou 785 por correr (`loadavg` 45/135/166) |
| — os 2 FAIL | os dois testes do bloco antigo de `the_players_finger_reaches_the_bridge` (re-ancorados, §3); 6/6 ✓ depois |
| — os 5 TIMEOUT | `ph2d-app-flip` ×2 (luz), `ph2d-app-components` `as_duas_barras_do_igual`, `ph2d-app-light2d` ×2: a vez da placa estava com a suíte da `line_luz2d`; sozinhos com `PH2D_GPU=1` 5/5 ✓ (19 s, 1 s, 2 s, 20 s, 2 s) |
| `nextest-impacted`, passagem 2 sobre os 804 em falta | 801 ✓; 3 falharam na corrida 1 a `load 29`, todos de relógio-razão e já listados em [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md), em crates que a linha não toca: `ph2d-physics-ecs` `measure_player_budget::the_cost_of_a_player_is_linear_in_their_number` (sozinho 2/3 ✓, 1 ✗ a `load 15`), `ph2d-timeline` `motion_path_perf::the_cost_of_sampling_a_path_is_flat_in_its_anchors` (3/3 ✓), `ph2d-tool-painter` `mask_gate_tests::the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` (3/3 ✓); corrida 2: 402/402 ✓ |

**Auditoria (2 lentes, só leitura).**

- *Costura:* o autorepeat do F5 alternava o transporte em rajadas → só a 1.ª pressão (`Tecla::repete`); a paleta
  Ctrl+K engolia o KeyUp e deixava uma seta presa no jogo → paleta/menu/diálogo/pré-visualização do Morph TAPAM a cena
  (a posse cai, as teclas largam-se); o soltar de um clique só-do-jogo caía nas ferramentas do editor sem o seu premir
  → o soltar de um premir só-do-jogo é só-do-jogo (com o Up do HUD).
- *Correcção:* o Esc da pré-visualização do Morph era engolido pelo jogo → a pré-visualização tapa a cena; o foco
  sobrevivia a tapar/destapar sem clique → tapar LIMPA o foco (Godot/Unity).
- Tudo curado em `eaeac6df3c`, com testes.

**Prova de mutação: 15 mutantes + controlo.** 13/15 sangraram na 1.ª rodada; sobreviveram M10 (`premido` sem `a_vista`)
e M11 (sem alimentar o `FocoDoJogo::tecla`) → testes `com_a_cena_tapada_um_clique_nao_e_so_do_jogo` e
`a_tecla_do_jogo_entra_no_teclado_dele_e_so_ela` (`eb371d4963`); ambos sangram depois. Controlo verde. O gate de ordem da
shell matou M12–M15.

**Conduzido de ponta a ponta:** [`ferramentas/conduz_posse/corre.sh`](../ferramentas/conduz_posse/corre.sh) (o binário
smoke numa Xvfb, cena 6, teclas e cliques reais por `xdotool`, os passos do Godot) — **15/15 ✓, duas vezes** (antes e
depois das curas da auditoria); fixtura `saida.txt` + fotos `foto_01..05`. A app corre sob Xvfb (caminho por software,
~40 fps) e o XTest chega-lhe — ao contrário da Xwayland do `kwin --virtual`: porta nova para smokes conduzidos. Duas
armadilhas da sonda: dois processos a escrever o mesmo ficheiro de log (a app sem append apagava as marcas do script) e,
sem gestor de janelas, ninguém dá foco de teclado (`xdotool windowfocus`).

**Medições do oráculo:** o editor do Godot 4.7.2 numa Xvfb ([`ferramentas/oraculo_teclado/`](../ferramentas/oraculo_teclado/saida.txt)):
jogo em foco recebe tudo; F8/F9/F10 reencaminhados ao editor, F7 não; clique no editor tira o foco, clique no jogo
devolve-o; modo «2D»: nem o clique nem a tecla chegam ao jogo.

**Régua do agente** (`agent-loop-profile.sh`, 20 sessões): paralelismo 1,18 ✗ (≥ 1,5) · respostas/sessão 142 ✓ ·
teste:check 2,2 ✗ · edições sem conferência 0,3 % ✓ · contexto/passo 312 k ✗ (≤ 250 k) · contexto inicial 64 k ✓.

## 7. Binário do smoke

`rm -rf target/*/incremental` (≈ 40 GB: debug 38 G + smoke 2,1 G) e depois, duas vezes,
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`; a 2.ª:
`Finished \`smoke\` profile [optimized] target(s) in 0.44s`.

## 8. Smoke para o DONO

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=6 cargo run -p ph2d-host-desktop --profile smoke`
2. A cena já começa a correr, com uma moldura azul à volta dela (o jogo tem o teclado).
3. **Espaço** atira estrelas e **não** pausa; as **setas** movem o círculo azul.
4. **F5** pausa e continua; **Shift+F5** rebobina e pára.
5. Clique num painel: a moldura some; agora o Espaço pausa e as setas não mexem o círculo.
6. Clique na cena: a moldura volta. Clique no quadrado cinzento: **não** o selecciona.

**Errado se:** o Espaço pausa com a moldura ligada · a moldura fica depois de clicar num painel · o quadrado fica seleccionado.

## 9. ABERTO

- **Consequências declaradas** (todas decorrem do «clique só do jogo» do dono, ou de a posse ser por clique, como no
  Godot):
  - o botão do meio (pan) e o menu do botão direito do canvas, sobre a cena, são do jogo enquanto ela corre; a roda
    (zoom) continua do editor;
  - as teclas encaminhadas pelo rato sobre o grafo do Motion/linha do tempo vão ao jogo enquanto ele tem o foco (o foco é
    por clique);
  - um quadro de atraso do foco depois do Play (a subida vê-se no início do quadro seguinte);
  - Shift não é acorde: Shift+letra vai ao jogo (o menu Shift+A fica indisponível a jogar);
  - teclas sem nome em `ph2d_input` são engolidas enquanto o jogo tem o teclado;
  - fora de jogo nenhum `keydown` chega ao `InputState` (o jogador não lê as setas ao avançar quadro a quadro com a
    física armada);
  - uma cena que começa a correr tapada precisa de um clique.
- Tudo o que continua aberto no [handoff do item 6, §9](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-20_ONDA2_I6_A_API.md)
  (o item do Espaço que lá estava aberto fecha-se aqui; o resto não se copia).
- **Smoke do dono do item 7: pendente.**
- **Integração:** a entrada do CLAUDE.md §5.1 já aponta para este handoff; no merge, só o link «Último» muda.
