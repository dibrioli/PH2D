# HANDOFF de integração — `line/InputMap`, ONDA 3, 10/11 (tudo o que ficou aberto, de uma vez)

Leitor: o integrador. Branch `line/InputMap`, código até `15b796a68c` (pós-smoke: §2b), fotos `c0fe8a64d9` (+ este handoff): a linha inteira, **188 commits** (`rev-list --count main..HEAD`, +1 com este), 366 files, `+29 705 −3 286` (`git diff --shortstat main...HEAD`), sobre `bc4a6ce930` = merge-base = ponta do `main` (**não andou: sem rebase, `--ff-only` serve**). Nada integrado, nenhum push. **Suíte completa, `nextest-impacted` e `ship.sh` NÃO correram** (são do integrador). Este handoff COMPLETA o da onda 2 (superfície, crates, schema 232, `play_map`, F1 continuam valendo): [onda 2](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA2.md) · [onda 1](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10.md) · [plano §14](../01_plano.md) · [prompt da onda 3](PROMPT_MISSAO_InputMap_ONDA3_2026-11-10.md).

## 1. Ordem do dono (10/11) — cumprida

«Corrigir tudo que for necessário de uma vez» — prompt [PROMPT_MISSAO_InputMap_ONDA3_2026-11-10.md](PROMPT_MISSAO_InputMap_ONDA3_2026-11-10.md): itens 1–8 cumpridos (§2).

## 2. O que entrou, por item

1. **Fila F em meia altura** (`618f07e082`): `FN_ROW_FACTOR` **0,64**, medido (plano §14). Vale também para Home/End/PgUp/PgDn.
2. **Plaquinha no comando e nos chips do «Also:»**: o nome numa plaquinha, as barras por trás (como o adesivo da tecla). Seta do stick da manche também.
3. **Hover da tela «Controls» do jogo** (`cab8aa7779`, `4fe0a5e177`): a linha sob o ponteiro acende no quadro; Reset com hover e premido, **Down arma / Up dispara** (`ControlsScreen::click(hit)` sem `PlayerKeys`; `host::click(desceu, principal)`: só o botão principal). O hover **deriva do último ponto do ponteiro sobre a disposição actual** (recalculado se a disposição muda); `CursorLeft` apaga; perder o foco esquece Alt/Ctrl/Shift (`App::on_focus_lost` em `despacho_metodos_janela_e_vetor.rs`; `ph2d-input` `editor_keys::held_from_flags`).
4. **Alt + tecla nunca liga sob a escuta partilhada** (`141b4917b7`; recusado, a escuta continua). AltGr reportado como Alt também é recusado.
5. **`danger-press`** (`2fac6ea5bc`): token novo `ColorToken::DangerPress`, mais fundo que o pairado nos 4 temas; o × premido o pinta (cura do §10 da onda 2).
6. **Shell** (`ec4d560a95`): o ajuste do Gap Closure pela roda sai para `ph2d_app_flip::gap_live::gap_wheel` (+27 no flip); comentários de história cortados: `input_dispatch.rs` **599 → 547**; shell **188 940 → 188 914**.
7. **Swipe lateral do trackpad.** `DocumentSurface::wheel_sideways` (default `false`). **Regressão da onda 2 curada**: a onda 2 mandava o swipe a TODA aba como Shift+roda, e Sprite/SFX/Planner/List rolavam na vertical. Agora liga só onde há onde rolar: Input Map (só sobre o teclado), Code (só o texto), grelha das camadas da física, Kanban (quadro + Tabela; a roda passa `[dx,dy]` pelas camadas). **Tabela do Kanban: roda invertida curada** (`cd84ebaafb`): a 2.ª conversão foi removida (a vertical também estava invertida); a roda converte-se uma vez.
8. **Revisão independente lado a lado, 3 rodadas + auditoria de 2 lentes (13 achados, curados):** tecla apertada = corpo `REST_FACE` + plaquinha acesa Text1/Bg0 (contraste 7,80–17,16); sem acção a plaquinha não se pinta; setas da manche com a mesma caixa 34×25,2; o contorno da acção aberta não come barras (cruz 2,00 px, glifo do braço 15,2); Alt/Ctrl/Shift esquecidos ao perder o foco.

## 2b. Pós-smoke (`972dc4f7c9..15b796a68c`, 6 commits)

- **(a) Ordem do dono 10/11: LT/RT acima do contorno do comando** (`8df5a2a828`, fotos B3 `104f48ecf1`), com vão de **6,00 px (`SEG_INSET`)** a 1600 e 1280. Teste `os_gatilhos_ficam_acima_do_contorno_com_um_vao` (`shells/game-scenes/tests/it/input_map/onda3_tracos.rs`); o teste dos cantos ficou sem a exceção dos gatilhos.
- **(b) Cenas novas 20 P2 (rato na linha Jump), 21 P3 (Reset premido), 22 P4 (premido arrastado para fora)** (`ea863264cb`): **CENAS 19 → 22**. O ponteiro entra pela porta do host (`ph2d-input-map-scene` `rato_do_p`, `PlaySession::layout()`; `shells/desktop/src/janela_do_jogo.rs` +1/−1, líquido 0). **P1..P4 pedem `PH2D_GAME_SMOKE_DRIVE=editor`.** Teste `as_cenas_p2_a_p4_poem_o_ponteiro_pela_porta_do_host`. Fotos anotadas `docs/InputMap/fotos/smoke_f1_{1_aberta,2_rato_na_linha,3_reset_apertado,4_reset_arrastado}.png` (refeitas em `c0fe8a64d9` com o hover novo).
- **(c) Token de cromo novo `row-hover`** (`ac7b8292ce`; `ColorToken::RowHover`): camada de estado neutra, opacidade 0,135 em `tokens.json`; `derive.rs` lê o alfa do clássico; `skin_body` `Surface(Raised)`. É o hover da linha da tela Controls: pairado ≥ 1,32:1 contra o painel e texto ≥ 4,58:1 nos 12 temas. Testes `o_hover_da_linha_le_se_e_o_texto_le_se_nele` e `o_hover_e_neutro_e_a_escolhida_e_a_do_acento` (`ph2d-player-controls`, `onda3_hover_contraste_tests.rs`). Tecto do painel tokens **2043 → 2074** (uma fileira de cromo `row-hover`, medido); `color.rs` **699/700**.
- **(d)** `15b796a68c`: clippy `type_complexity` no `mutacao_onda3` (alias `Ouvido`).

## 3. Foundational/compartilhado tocado

Aditivo salvo o indicado. `ph2d-tokens` (`DangerPress`; `tokens.json` 4 temas; `derive.rs` família moderna; `skin_body`) · `ph2d-editor-core` (`doc_surface.rs` `wheel_sideways`; `board_view.rs`/`surface_input.rs` rota dx/dy; `input_map_ops.rs`) · `ph2d-input` (`editor_keys::held_from_flags`) · `ph2d-panel-tokens` (família recolhida «Input Map» com `input-map-*`/`preview-*`; i18n `panel.tokens.family.input_map`) · `ph2d-panel-registry-init` (tecto de tokens **2012 → 2043** = UMA fileira de cromo `danger-press`, medido; comentário no ficheiro) · `ph2d-app-flip` (`gap_live::gap_wheel`) · `ph2d-app-code`, `ph2d-app-kanban`, `ph2d-kanban-views`, `ph2d-panel-physics`, `ph2d-app-planner`, `ph2d-app-sfxlab`, `ph2d-app-sprite-anim` (swipe lateral e testes) · `shells/desktop`.

## 4. Superfície de colisão (colada de `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap/scripts/collision-surface.sh`, 10/11, linha inteira)

```

SUPERFÍCIE DE COLISÃO — line/InputMap contra main
  merge-base bc4a6ce930   ·   180 commit(s)   ·   357 arquivo(s)
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
  ✗  1023 / 600   shells/desktop/src/main.rs
───────────────────────────────────────────────────────────────────────────────
  (SONDA CEGA ph2d-render/ph2d-script e «isto é o MAPA, não o gate»: cortados; ver o texto abaixo)
```

**Contadores como DELTA:** `PROJECT_SCHEMA` — a onda 3 NÃO o mexeu (231 → 232 é da onda 1; `python3 scripts/schema-recount.py`). Componentes DELTA 0 (a SONDA CEGA `ph2d-render`/`ph2d-script` é a mesma das ondas 1–2: a linha não toca o registo). §6 intocado. ADR nenhum. `Cargo.lock` sem pacote novo nesta onda. `CENAS` **19 → 22** (pós-smoke). Shell **−26** (188 940 → 188 914; `janela_do_jogo.rs` líquido 0; re-meça contra o `main` do dia). `main.rs` 1 023/600 é pré-existente. Tecto de tokens do painel +62 (2012 → 2074: `danger-press` +31, `row-hover` +31). A tabela da §4 foi colada antes do pós-smoke (180 commits, 357 arquivos): releia-a no dia.

## 5. Valores literais que colidem

Dois tokens novos: `danger-press` (`ColorToken::DangerPress`, 4 temas) e `row-hover` (`ColorToken::RowHover`, 12 temas). Smoke `PH2D_GAME_SMOKE` **6..22** (P2–P4 = 20..22). Se outra linha tomou cenas, renumere pelo `main` do dia (conte, não escolha).

## 7. O que um merge pode partir

1. **Tectos no limite exacto:** `ph2d-app-input-map` 449/449 · `ph2d-app-kanban` 32 175/32 175 · `ph2d-tokens/src/color.rs` 699/700 · `ph2d-app-code` 26 626/26 628 · `ph2d-app-sprite-anim` 5 209/5 209. Linha que acrescente nestas corta antes.
2. **Tecto do painel tokens 2074**: linha que acrescente token de cromo soma 31 px por fileira.
3. **`ColorToken::DangerPress` e `RowHover` na lista da macro** (ordem): linha que acrescente token na mesma lista conflita na ordem.
4. **`DocumentSurface::wheel_sideways`** (trait com default): implementações novas noutras linhas compilam; quem mexer na roda de `board_view.rs`/`surface_input.rs` conflita.
5. **Kanban:** a roda passa `[dx,dy]` pelas camadas (linha que mexa nas camadas de roda conflita); a Tabela sem a 2.ª conversão (quem afirmar o sentido antigo reprova).
6. **`input_dispatch.rs` 547** e o `on_focus_lost` novo em `despacho_metodos_janela_e_vetor.rs`; o Gap Closure vive em `ph2d_app_flip::gap_live::gap_wheel` (quem editar o ajuste na shell conflita).
7. **`ControlsScreen::click(hit)`** sem `PlayerKeys`; `host::click(desceu, principal)`: chamadores novos noutras linhas não compilam.
8. `CENAS` 22 (`ph2d-app-game/src/smoke.rs`) e `.config/testes-lentos.tsv` (gerado). Schema 232 (o da onda 1). Os itens 1–11 do §7 da onda 2 continuam valendo, salvo o 9 (agora 547) e o 10 (agora via `wheel_sideways`).

## 8. Prova de fecho (feita na linha)

- `cargo fmt` ✓; `cargo clippy --workspace --all-targets --keep-going -D warnings` ✓; machete ✓; `check-standalone-optional.sh` ✓; `check-workflow-packages.sh` ✓.
- Censos da árvore combinada **132/133**: o único vermelho (`ph2d-app-code` 26 639 > 26 628, vindo do teste da mutação) foi curado em `e6e8658d39` e o gate re-corrido verde.
- Testes por nome: `input_map::` **227**; `ph2d-player-controls` tests 47 / `onda3_hover` 8 / `onda3_hover_r2` 5 / `r10a` 7 / `r11a` 4 / `f3` 3 / `play` 13; `ph2d-input` `keyboard` 13 + `alt_em_baixo` 1; `ph2d-editor-core` `file_loc_cap` 2, `the_shell_only_shrinks` 2, `the_shell_only_shrinks_and_every_app_crate_too` 1, `enum_order_matches_svgs` 1, `hr12_widgets_a11y` 3, `no_magic_numeric` 3, `board_view` 7; `ph2d-tokens` 196; `ph2d-tokens-dtcg` 26; `ph2d-panel-tokens` `familia` 1 / `os_cartoes` 11 / `seam_tokens` 30; `a_altura_de_abertura` 1; `a_tabela_inglesa_fala_ingles` 4; `ph2d-app-flip` `gap` 11; `ph2d-host-desktop` `the_two_normalizers` 1 + `perder_o_foco` 1; `swipe_lateral`: code 1 (após fusão), physics 1, kanban 1, planner 1, sfxlab 1, sprite-anim 1; `ph2d-kanban-views` `table` 10 / `wheel` 1; kanban `canvas_entrada` 6 / `d16` 5.
- **Mutação: 54 mutações, 48 sangraram na 1.ª; 6 sobreviventes mortas** por testes novos (`0199210908`: `mutacao_onda3.rs`, `completar.rs`, `onda3_hover_r2_tests.rs`); a onda 2 e as 4 leis da onda 1 continuam a sangrar. **NÃO mutada:** a ordem `dock_seam_move` antes da aba em `input_dispatch.rs:173` (lei da shell; só W2a/W2b na aba a protegem) — o integrador pode mutá-la com um teste de App. ⚠️ `mut.py` restaura com `git checkout HEAD -- <file>`: comite antes de mutar.

- **Re-corrida pós-smoke, verde em `15b796a68c`:** `cargo fmt`; `clippy --workspace --all-targets --keep-going -D warnings`; `input_map::` **234**; `onda3_hover` 11; `ph2d-tokens` 196; `ph2d-tokens-dtcg` 26; `a_altura_de_abertura` 1; `familia` 1; `file_loc_cap` 1; `the_shell_only_shrinks` 2; `the_shell_only_shrinks_and_every_app_crate_too` 1; `no_magic_numeric` 3; `a_tabela_inglesa_fala_ingles` 4.

## 9. Smoke (do dono) — **OK para a onda 3 (10/11), salvo o passo da tela Controls: PENDENTE de confirmação**

O passo da tela Controls (hover da linha, Reset premido/arrastado) foi refeito com fotos e reenviado ao dono; a confirmação não chegou. Comando: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap && PH2D_GAME_SMOKE=18 PH2D_GAME_SMOKE_DRIVE=editor bash scripts/ph2d-run.sh cargo run -p ph2d-host-desktop --profile smoke` (cenas 6..22; P1..P4 = 19..22 pedem `PH2D_GAME_SMOKE_DRIVE=editor`). Fotos em `docs/InputMap/fotos/` (`ae3cde492c`: B1, B3, B5, C2, C4, A1 a 1600/1280; B1 e C2 a 900). Gamepad físico não smokado.

## 10. ABERTO e fora da linha

- O fundo de repouso da tecla é `KanbanCardRaised` (token de dado de outra família, desde a onda 1, decisão aprovada): nenhum neutro de cromo passa a lei do contraste das barras (tabela no plano §14.2).
- Kanban Calendar/Dashboard e drawer da Timeline sem swipe lateral (só verticais).
- Sprite: o teste «nada muda» corre sem aba aberta (abrir exige folha e a crate não tem folga). Pixel não é aba de documento.
- Cursor de mão na tela Controls do jogo (a porta não tem cursor).
- **Fora da linha, por confirmar no app:** o Planner manda a roda crua ao Calendário que espera a convertida (pode rolar ao contrário do Kanban).
- Decisões que ficaram de fora por ordem: legendas secundárias do numérico, R7b, ◆ em Dead/Press, Inspector vazio.
- Itens da revisão visual recusados por serem decisões aprovadas: legenda que vira ícone/abreviação, tecla do sistema esmaecida, 1280 empilhado, T invertido das setas, legenda de baixo dos chips.

## 11. Proposta para o `CLAUDE.md §5.1` (só na integração; a linha NÃO a escreve)

O InputMap não tem entrada própria. Proposta (≤ 700 B):

> - **Input Map** — aba central com três vistas (List · Keyboard · Try it), teclado de notebook e comando, tela «Controls» no jogo (F1), ligação física, Luau pela ação (`ph2d-input`, `ph2d-input-map-editor`, `ph2d-app-input-map`, `ph2d-player-controls`, `ph2d-input-winit`). Smoke `PH2D_GAME_SMOKE=6..22`. Último: [onda 3, 10/11](docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA3.md) · [plano](docs/InputMap/01_plano.md)
