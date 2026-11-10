# HANDOFF de integração — `line/InputMap`, ONDA 2, 10/11 (teclado de notebook inteiro como na foto do dono + camada de polimento pro)

Leitor: o integrador. Branch `line/InputMap`, código até `2a58c4da03`, fotos `c0af7311c8` (+ este handoff): onda 2 = `8f5c9d81b3..HEAD` (inclui as ordens pós-smoke dos §2b e §2c); a linha inteira, **149 commits** (`git rev-list --count main..HEAD`, antes deste commit), 309 files, `+26 687 −3 081` (`git diff --shortstat main...HEAD`), sobre `bc4a6ce930` = merge-base = ponta do `main` (**não andou: sem rebase, `--ff-only` serve**). Nada integrado, nenhum push. **Suíte completa, `nextest-impacted` e `ship.sh` NÃO correram** (são do integrador). Este handoff COMPLETA o da onda 1 (superfície, crates, schema 232, `play_map`, F1 continuam valendo): [onda 1](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10.md) · [plano §13](../01_plano.md) · [prompt da onda 2](PROMPT_MISSAO_InputMap_ONDA2_2026-11-10.md).

## 1. Ordem do dono (10/11) — cumprida

Teclado B/C COMPLETO como na foto do notebook dele (`docs/InputMap/maquetes/06_teclado_notebook_dono.png`) + polimento pro (hover, mouse down, foco nos campos, acabamentos). Verbatim: prompt §2.

## 2. O que cada onda entregou

**W1 — códigos e nomes** (plano §13.1). `NAMED` 21 → **50** teclas nomeadas; numérico com códigos próprios `0xF740–0xF74F` (**Numpad Enter `0xF74F` ≠ Enter**); tabela à parte das teclas do sistema (Windows ×2, Fn, PrtSc: rótulo, sem nome de script, `Key::system_owned()` → porquê i18n `input_map_keyboard.system.{fn,windows,print_screen}`). API nova em `Key`: `cap`, `cap_short`, `cap_lines`, `is_enter_like`, `ENTER`, `NUMPAD_ENTER`. **Medido:** Fn ausente do evdev; KDE liga Print→Spectacle e Meta→lançador; winit 0.30.13 sem estado de Num Lock ⇒ **sem luz**. **Crate folha nova `crates/ph2d-input-winit`** (a tradução winit→`Key` saiu da shell: **189 034 → 188 940**; Super/Fn/PrintScreen ficam `None`). Luau `KeyName` com **101 nomes** (`kp7`; `api_table` + `ph2d.d.luau`; gate `os_nomes_de_tecla_sao_os_do_teclado`). `editor_keys.rs` em `ph2d-input` = o que nunca liga (F5, acordes; `raw_keys` reexporta). A escuta partilhada de `ph2d-editor-core` (`input_map_ops.rs`) recusa teclas do sistema e F5/Ctrl; **a recusada segue** (F5 sob a escuta ainda comanda o Play). O ficheiro do jogador lê `key:kp.`/`kp-`/`kp+`.

**W2 — o teclado desenhado** (plano §13.2). Tabela de dados de **101 teclas**; arranjo e legenda por função pura medida (`keys_fit.rs`; o `Narrow` desta onda foi APAGADO no §2c); legenda por **GRUPO** (fila F, borda do principal, setas, Home/End/PgUp/PgDn, numérico): palavra a 11 → palavra a 10 (`TypeToken::Xxs`) → curta → símbolo ISO/IEC 9995-7. Ícones novos `kbd-{end,enter,home,page-down,page-up,print-screen,num-lock}` (**Num Lock = ⇭, nunca cadeado**); PrtSc em duas linhas «Prt»/«Sc» a 1600 e **o ícone a 1280** (duas linhas não cabem em 22 px). **B a 1280 empilha**: teclado na largura inteira em cima, lista + cartão rolam como UMA região por baixo. Numérico em colunas de 1 u (a foto tem ~0,82 u: **desvio consciente**). Teclas do sistema esmaecidas, sem ligar, porquê no hover. Cena **19 «B5 · numpad»**; `CENAS` 18 → 19.

**W3–W5 — polimento** (plano §13.3). `interact.rs`: **registo único de controlos** (`doc.control`: alvo + premido nativo — Down captura, Up dentro dispara, fora cancela — + Tab/Shift+Tab/Enter/Espaço/Esc + nós a11y + cursor `look_at`). **Hover INSTANTÂNEO por decisão D32** (`docs/CartoesUI/01_decisoes_do_dono.md`; o «fade do hover» do prompt está SUPERADO). Curado: o `WidgetStore` da aba nunca alimentava `hover_live` (botões do cartão sempre em hover). Polegares `Role::Fade`; pulso da escuta `Role::Decoration`; caret por `UiMotion::elapsed()`. `field.rs` = campo numérico partilhado (A e B): ↑/↓ 0,01, Shift ×10, Alt ×0,1, Enter = UM passo de undo, Esc devolve, Tab aplica e segue, clamp com frase (Dead `MAX_DEAD_ZONE` 0,99). Diálogo da receita: foco preso, nasce no Cancel. **Esc cancela a escuta do «Opens with»** (lei mudada; F5 segue recusado). Tela do jogador: linha escolhida, Reset botão, Remove/Add desligados com o porquê.

**W6 — acabamento.** Fotos e revisões visuais triadas. **NÃO feito, por decisão:** frase do hover da B substitui o cabeçalho (R7b da onda 1); sem ◆ em Dead/Press; fila F em meia altura; lupa de busca do app; Inspector. 4 defeitos curados vermelho→verde (Tab em linha recortada; Numpad Enter na busca vazia; aviso de clamp 1,00 com 0,99 guardado; foco morto).

## 2b. Depois do smoke do dono (10/11) — duas ordens

**Smoke do dono OK 10/11** para a onda 2 e para estas duas ordens.

1. **Legenda num adesivo** (`3de41af070`, `2c0c36ce04` memória, `8ab6cf4070`). A legenda fica PARADA no centro da tecla (`legend_box`) numa plaquinha opaca (`keys_fit::plate_of`); as barras das acções passam POR TRÁS (`keys_fit::bar_stack`, `BAR_PEEK`). Saem `reserved_bars`/`bars_top`. Testes `adesivo` e `a_legenda_fica_parada_na_plaquinha_com_qualquer_numero_de_barras`. Close-up `docs/InputMap/fotos/adesivo_b1_1600.png`.
2. **Hover/premido da List** (`b223f0ee6a`, `74d1bb71cd`; `view_list/looks.rs`, censo aditivo `census.looks`). Defeito curado: o hover lia os rects da pintura EM CURSO (um quadro tarde); agora `begin_hot` lê a anterior. ×, «+», lixo com alvo de 24 px. **Clicar na linha (ou Enter) só FOCA**; escuta só no «+» e no «No key». Testes: `shells/game-scenes/tests/it/input_map/polimento_w3_lista.rs`.

## 2c. Pós-smoke, rodada 2 (ordens do dono 10/11; smoke OK)

1. **Passo mínimo da tecla** (`65613ee429`, plano `ab68448af7`). `PITCH_FLOOR = 31,5 px`, MEDIDO (30,5 Default / 31,5 CrispHeavyPlus): a legenda cabe em todo grupo, ≥ 2 px de barra de cada lado do adesivo; o teclado (603 px) cabe nos 630 px da aba real a 1280. **`Arrangement::Narrow` APAGADO** (um arranjo só; o numérico nunca desce). Abaixo do piso o desenho é recortado e rola na horizontal (polegar, arrasto, Shift+roda; Tab revela; tooltip contido na aba). Testes: `teclado_piso.rs`. **Efeito fora da linha:** `ph2d-editor-core` `board_view.rs` e `surface_input.rs` mandam o swipe horizontal do trackpad a TODA aba de documento como Shift+roda.
2. **Painéis redimensionáveis com a aba aberta** (`14b7cc16b4`). Causa: `dock_seam_move` corria DEPOIS de a aba consumir todo movimento (`view_keyboard/input.rs` devolvia `true` fora da sua área). Cura: a shell chama `dock_seam_move` ANTES em `pointer_moved` (**código movido**; `shells/desktop/src/input_dispatch.rs` agora **599/600** — avise o integrador: uma linha a mais reprova); a aba só reclama movimento dentro do retângulo pintado ou durante o próprio premir. Testes: `paineis.rs`; `ph2d-app-input-map` **449/449**.
3. **× do chip = emblema redondo de 16 px** (`2a58c4da03`; censo `badges`): repouso BorderStrong + ícone Text1; hover Danger + AccentFg; premido DangerSoft (não existe token de perigo mais escuro: no tema claro o premido pode ler MAIS claro). Fotos `c0af7311c8` (`b1_900.png`, `c2_900.png`).

## 3. Foundational/compartilhado tocado nesta onda

Tudo aditivo salvo o movimento indicado: `ph2d-input` (`Key` 50 + numérico + sistema, `cap*`, `is_enter_like`, `system_owned`, `editor_keys.rs`, `MAX_DEAD_ZONE` pub) · **crate folha NOVA `ph2d-input-winit`** (`Cargo.lock` +1 interno; shell −94) · `ph2d-editor-core` (escuta partilhada recusa sistema/F5/Ctrl em `input_map_ops.rs`; caret aditivo em `WidgetStore`/`TextInput`; `UiMotion::elapsed()`; swipe→Shift+roda (§2c); 7 ícones `kbd-*`; **MOVIMENTO:** `paint_text_elided*` de `paint_text.rs` para `paint_text_elided.rs`, reexportado do mesmo caminho, 701 → 526) · `ph2d-script` (`KeyName` 101 nomes) · `ph2d-i18n` (frases `input_map_*`) · `ph2d-app-game` (`CENAS` 18 → 19).

**Mesmo-símbolo a grepar numa fusão:** a lista do §7.

## 4. Superfície de colisão (colada de `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap/scripts/collision-surface.sh`, 10/11, linha inteira, em `c0af7311c8`)

```
SUPERFÍCIE DE COLISÃO — line/InputMap contra main
  merge-base bc4a6ce930   ·   149 commit(s)   ·   309 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        232   (base: 231)
  ⚠   └ tripla do gate               (232, 13, 29)   (base: (231, 13, 29))
    VEC_SCENE_SCHEMA                       29   (base: 29)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 25   (base: 25)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              121   (base: 121)
  ✗ ph2d-render (espelho = ecs + K)       —   (base: —)
  ✗ ph2d-script (espelho = ecs + K)       —   (base: —)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0200   próximo livre: 0201
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 5 pacote(s) '+name' novo(s):
      "ph2d-app-input-map"
      "ph2d-input-map-editor"
      "ph2d-input-map-scene"
      "ph2d-input-winit"
      "ph2d-player-controls"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
  ✗  1030 / 600   shells/desktop/src/main.rs
───────────────────────────────────────────────────────────────────────────────
  ✗ SONDA CEGA em: ph2d-render (espelho = ecs + K), ph2d-script (espelho = ecs + K) — a const mudou de ficheiro e nenhum candidato a tem.
    Corrija o caminho NESTE script: um «—» dos dois lados lia-se «ninguém mexeu» (13/09).
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.
```

**Contadores como DELTA:** `PROJECT_SCHEMA` — a onda 2 NÃO o mexeu (231 → 232 é da onda 1; `python3 scripts/schema-recount.py`). Componentes DELTA 0 (sonda cega: a linha não toca o registo). §6 intocado. ADR nenhum. `Cargo.lock` **+1** interno nesta onda, nenhum externo. `CENAS` **+1** (18 → 19). Shell **−94** (189 034 → 188 940; tecto da catraca 189 041 da onda 1: **re-meça** contra o `main` do dia). `main.rs` 1 030/600 é pré-existente (a linha não o fez crescer).

## 5. Valores literais que colidem

Nenhum token novo nesta onda (os 5 + `categorical-*` são da onda 1). Smoke: `PH2D_GAME_SMOKE` **6..19** (19 = B5 · numpad; 18 = P1 com `PH2D_GAME_SMOKE_DRIVE=editor`). **Se outra linha tomou cenas após 5, renumere pelo `main` do dia** (conte, não escolha).

## 7. O que um merge pode partir

1. **Schema/tripla** (§4) — conte contra o `main` do dia.
2. **Catraca da shell** (`the_shell_only_shrinks`): linha que cresça a shell soma sobre os −94; mover, nunca subir. Re-meça.
3. **`paint_text.rs` (526) / `paint_text_elided.rs`**: linha que edite `paint_text_elided*` em `paint_text.rs` conflita (o código mudou de ficheiro).
4. **`ph2d-input`**: `NAMED`/`script_names` e o gate `os_nomes_de_tecla_sao_os_do_teclado` (101 nomes) — linha que acrescente tecla soma na lista; `Key` e o ficheiro do jogador.
5. **`ph2d-app-input-map` está em 449/449 exatos**: nenhuma folga; linha que acrescente aí corta antes.
6. **Ícones**: ordem de `enum_order_matches_svgs` (7 `kbd-*` novos) e a lista de ícones de `ph2d-editor-core`.
7. **i18n**: `lib.rs` e as chaves `input_map_*`; `a_tabela_inglesa_fala_ingles` (app só em inglês).
8. `CENAS` 19 e `.config/testes-lentos.tsv` (gerado).
9. **`shells/desktop/src/input_dispatch.rs` 599/600** e a ordem em `pointer_moved` (`dock_seam_move` ANTES da aba): linha que mexa ali conflita ou reprova o tecto.
10. **`board_view.rs` / `surface_input.rs`** (`ph2d-editor-core`): swipe horizontal vira Shift+roda em toda aba de documento; linha que mexa na roda deles conflita no comportamento.
11. `Arrangement::Narrow` não existe mais: quem o referir não compila (`PITCH_FLOOR` em `keys_fit.rs`).

## 8. Prova de fecho (feita na linha)

- Bateria verde: `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings`, `cargo machete`, `check-standalone-optional.sh`, `check-workflow-packages.sh`; crate nova `ph2d-input-winit` inteira; testes por nome: `input_map::` **209 passed** (pós-§2c, 10/11; fmt e clippy `--workspace --all-targets -D warnings` limpos em `8ab6cf4070`, **não refeitos** após o §2c: refaça no fecho da integração), `ph2d-player-controls` 39, `ph2d-input` `keyboard` 13, gates do `ph2d-editor-core` (`file_loc_cap`, `the_shell_only_shrinks`, `enum_order_matches_svgs`, `hr12_widgets_a11y`, `no_magic_numeric`), `ph2d-i18n` `a_tabela_inglesa_fala_ingles`, `ph2d-host-desktop` `the_two_normalizers_never_disagree`. **Censos combinados 133/133 e 12 de 12.** Nenhum `#[cfg(target_os` escrito ou movido.
- **Provas de mutação: 40 mutações, 37 sangraram na 1.ª rodada; as sobreviventes mortas** (`mutacao_w2.rs`, `doc_tests.rs`); as **4 leis da onda 1 continuam a sangrar**; o **buraco do F5** fechado (F5 em «press one» nunca liga; sob a escuta do ladrilho Listen o F5 liga-se/segue ao Play). **L4d equivalente por desenho**; **L6f inalcançável pelas portas públicas** — ambas assumidas. ⚠️ `mut.py` restaura com `git checkout HEAD -- <file>`: comite antes de mutar. Mutação NÃO refeita para o §2c.

## 9. Smoke (do dono) — **feito, OK 10/11** (onda 2 + §2b + §2c)

Fotos em `docs/InputMap/fotos/` (1600×1000 e 1280×720; montagens, incl. `montagem_teclado_dono`, close-up `adesivo_b1_1600.png`). Comando: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap && PH2D_GAME_SMOKE=19 bash scripts/ph2d-run.sh cargo run -p ph2d-host-desktop --profile smoke` (cenas 6..19; a 18 junta `PH2D_GAME_SMOKE_DRIVE=editor`). Gamepad físico não smokado.

## 10. ABERTO e achados FORA da linha

- **Fila F em altura inteira** (as barras precisam dela; não meia altura como na foto).
- **Comando (pad) e chips do «Also:»** mantêm as barras POR BAIXO (o adesivo é só da tecla).
- **Acordes com Alt ligam sob a escuta partilhada** (o store não guarda estado de Alt).
- Numérico sem segundas legendas (End/PgDn/setas sob 1/3/…): o binding é físico.
- Tela «Controls» do jogo **sem hover** (sem caminho de ponteiro em `host.rs`).
- **× premido no tema claro** pode ler mais claro (sem token de perigo mais escuro).
- Campos numéricos da C (se vierem) arrastam ao premir.
- Fora da linha (herdado): aba de região/Animated Sprite, Planner e física sem `chrome_over`; `CrispHeavyPlus` perde pingos de i/j a ~13 px negrito.

## 11. Proposta para o `CLAUDE.md §5.1` (só na integração — DIRETRIZ §1.5.9 item 8; a linha NÃO a escreve)

Trocar o link do «Último» e o intervalo do smoke (≤ 700 B):

> - **Input Map** — aba central com três vistas (List · Keyboard · Try it), teclado de notebook inteiro (101 teclas), mapa no undo do projeto, tela «Controls» do jogador (F1 no Play), Luau pela ação (`ph2d-input-map-editor`, `ph2d-app-input-map`, `ph2d-player-controls`). Smoke `PH2D_GAME_SMOKE=6..19`. Último: [onda 2, 10/11](docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA2.md) · [plano](docs/InputMap/01_plano.md)

**Memória:** `project_inputmap_three_versions_2026_11_10.md` ganhou o parágrafo «Onda 2 entregue 10/11»; a linha do `MEMORY.md` não muda.
