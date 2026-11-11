# HANDOFF de integração — `line/InputMap`, ONDA 3, 10/11 (tudo o que ficou aberto)

Branch `line/InputMap`, código até `a3eac373a0` (§2b), fotos `c0fe8a64d9` (+ este handoff): a linha inteira, **192 commits** (`rev-list --count`), 366 files, `+29 705 −3 286` (`diff --shortstat`), sobre `bc4a6ce930` = merge-base = ponta do `main` (**não andou: sem rebase, `--ff-only` serve**). Nada integrado nem pushado. Suíte completa, `nextest-impacted` e `ship.sh` NÃO correram (integrador). Completa o da onda 2 (schema 232, `play_map`, F1 valem): [onda 2](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA2.md) · [onda 1](HANDOFF_INTEGRACAO_line_InputMap_2026-11-10.md) · [plano §14](../01_plano.md) · [prompt da onda 3](PROMPT_MISSAO_InputMap_ONDA3_2026-11-10.md).

## 1. Ordem do dono (10/11) — cumprida

«Corrigir tudo que for necessário de uma vez» (prompt linkado acima): itens 1–8 cumpridos (§2).

## 2. O que entrou, por item

1. **Fila F em meia altura** (`618f07e082`): `FN_ROW_FACTOR` **0,64**, medido (plano §14). Vale também para Home/End/PgUp/PgDn.
2. **Plaquinha no comando e nos chips do «Also:»**: o nome numa plaquinha, as barras por trás (como o adesivo da tecla).
3. **Hover da tela «Controls» do jogo** (`cab8aa7779`, `4fe0a5e177`): a linha sob o ponteiro acende no quadro; Reset com hover e premido, **Down arma / Up dispara** (`ControlsScreen::click(hit)` sem `PlayerKeys`; `host::click(desceu, principal)`: só o botão principal). O hover deriva do último ponto do ponteiro; `CursorLeft` apaga; perder o foco esquece Alt/Ctrl/Shift (`App::on_focus_lost` em `despacho_metodos_janela_e_vetor.rs`; `ph2d-input` `editor_keys::held_from_flags`).
4. **Alt + tecla nunca liga sob a escuta partilhada** (`141b4917b7`; recusado, a escuta continua). AltGr reportado como Alt também é recusado.
5. **`danger-press`** (`2fac6ea5bc`): token novo `ColorToken::DangerPress`, mais fundo que o pairado nos 4 temas; o × premido o pinta.
6. **Shell** (`ec4d560a95`): o ajuste do Gap Closure pela roda sai para `ph2d_app_flip::gap_live::gap_wheel` (+27 no flip); comentários de história cortados: `input_dispatch.rs` **599 → 547**; shell **188 940 → 188 914**.
7. **Swipe lateral do trackpad.** `DocumentSurface::wheel_sideways` (default `false`). **Cura da onda 2** (o swipe ia a TODA aba como Shift+roda): agora só onde há onde rolar: Input Map (só sobre o teclado), Code (só o texto), grelha das camadas da física, Kanban (quadro + Tabela; a roda passa `[dx,dy]` pelas camadas). **Tabela do Kanban: roda invertida curada** (`cd84ebaafb`): a 2.ª conversão foi removida (a vertical também estava invertida); a roda converte-se uma vez.
8. **Revisão independente, 3 rodadas + auditoria de 2 lentes (13 achados, curados):** tecla apertada = corpo `REST_FACE` + plaquinha acesa Text1/Bg0 (contraste 7,80–17,16); setas da manche na mesma caixa 34×25,2.

## 2b. Pós-smoke (`972dc4f7c9..a3eac373a0`)

- **(a) Ordem do dono 10/11: LT/RT acima do contorno do comando** (`8df5a2a828`, fotos B3 `104f48ecf1`), com vão de **6,00 px (`SEG_INSET`)** a 1600 e 1280. Teste `os_gatilhos_ficam_acima_do_contorno_com_um_vao` (`shells/game-scenes/tests/it/input_map/onda3_tracos.rs`); o teste dos cantos ficou sem a exceção dos gatilhos.
- **(b) Cenas novas 20 P2 (rato na linha Jump), 21 P3 (Reset premido), 22 P4 (premido arrastado para fora)** (`ea863264cb`): **CENAS 19 → 22**. O ponteiro entra pela porta do host (`ph2d-input-map-scene` `rato_do_p`, `PlaySession::layout()`; `shells/desktop/src/janela_do_jogo.rs` +1/−1, líquido 0). **P1..P4 pedem `PH2D_GAME_SMOKE_DRIVE=editor`.** Teste `as_cenas_p2_a_p4_poem_o_ponteiro_pela_porta_do_host`. Fotos anotadas `docs/InputMap/fotos/smoke_f1_{1_aberta,2_rato_na_linha,3_reset_apertado,4_reset_arrastado}.png` (`c0fe8a64d9`).
- **(c) Token de cromo novo `row-hover`** (`ac7b8292ce`): camada de estado neutra, opacidade 0,135 em `tokens.json`; `derive.rs` lê o alfa do clássico; `skin_body` `Surface(Raised)`. É o hover da linha da tela Controls: pairado ≥ 1,32:1 contra o painel e texto ≥ 4,58:1 nos 12 temas. Testes `o_hover_da_linha_le_se_e_o_texto_le_se_nele` e `o_hover_e_neutro_e_a_escolhida_e_a_do_acento` (`ph2d-player-controls`, `onda3_hover_contraste_tests.rs`). Tecto do painel tokens **2043 → 2074** (medido); `color.rs` **699/700**.
- **(d)** `15b796a68c`: clippy `type_complexity` no `mutacao_onda3` (alias `Ouvido`).
- **(e) Ordem do dono 10/11: botão Controls na barra do Play** (`a3eac373a0`): à direita do Stop, separador vertical (filete da casa), moldura própria. Ícone novo `gamepad` (`docs/design/icons/gamepad.svg`, na ordem de `enum_order_matches_svgs`); id `TOPBAR_PLAYER_CONTROLS` (na lista de `every_topbar_verb_has_a_door`). No Play o clique = a tecla (`host::toggle` → `host::key`), aceso com a tela aberta; fora do Play desligado, com o porquê; a dica cita a tecla atual do mapa; a shell deixa passar o clique sobre o botão com a tela aberta. Ficheiros: `ph2d-editor-core` `transport_bar.rs`, `game_stage.rs`, `action_bus.rs`; `ph2d-player-controls` `host.rs`; shell `input_dispatch/controles_do_jogador.rs`, `fase_bus_chrome.rs` (shell −1 líquido). Testes: `transport_bar` 5, `onda3_barra_tests` 3. Limites: tela aberta não acende ao pairar; em escuta o clique só cancela a escuta; a barra de tela cheia não o tem. Fotos `fotos/barra_controls_*`.

- **(f) Smoke do dono: a dica do botão saía cortada («Try…»)** (`ae9da26216`). Cura no tooltip PARTILHADO (`crates/ph2d-editor-core/src/widget/tooltip.rs`): `mede_a_dica`/`paint_dica` medem no peso/modo da pintura, quebram por palavras a min(320 px `LARGURA_MAXIMA_DA_DICA`, janela). Consumidores: topbar `paint_hover_tooltip` e balão de reticência; `paint_tooltip`/`Tooltip` antigos intocados. Frases novas: `input_map_try.rs`. Teste `a_dica_do_controls_nunca_corta_e_cabe_na_janela`. Varredura por consumidor verde: editor-core tooltip 8, dica 20, hover 38, elid 34, balao 22, widget 669, cromo 14, transport_bar 12; app-code dicas 13, botoes 6; panel-motion-graph tooltip 3; fmt e clippy `--workspace` limpos.

## 3. Foundational/compartilhado tocado

Aditivo salvo o indicado. `ph2d-tokens` (`DangerPress`, `derive.rs`, `skin_body`) · `ph2d-editor-core` (`doc_surface.rs`, `board_view.rs`, `surface_input.rs`, `input_map_ops.rs`, e o botão: §2b e) · `ph2d-input` (`editor_keys::held_from_flags`) · `ph2d-panel-tokens` (família recolhida «Input Map») · `ph2d-panel-registry-init` (tecto de tokens **2012 → 2043** = UMA fileira `danger-press`, medido) · `ph2d-app-flip` (`gap_live::gap_wheel`) · `ph2d-app-code`, `ph2d-app-kanban`, `ph2d-kanban-views`, `ph2d-panel-physics`, `ph2d-app-planner`, `ph2d-app-sfxlab`, `ph2d-app-sprite-anim` (swipe lateral e testes) · `shells/desktop`.

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

1. **Tectos no limite exacto:** `action_bus.rs` (editor-core) 700/700 · `ph2d-app-input-map` 449/449 · `ph2d-app-kanban` 32 175/32 175 · `ph2d-tokens/src/color.rs` 699/700 · `ph2d-app-code` 26 626/26 628 · `ph2d-app-sprite-anim` 5 209/5 209. Linha que acrescente nestas corta antes.
2. **Tecto do painel tokens 2074**: linha que acrescente token de cromo soma 31 px por fileira.
3. **`ColorToken::DangerPress` e `RowHover` na lista da macro** (ordem): linha que acrescente token na mesma lista conflita na ordem.
4. **`DocumentSurface::wheel_sideways`** (trait com default): implementações novas noutras linhas compilam; quem mexer na roda de `board_view.rs`/`surface_input.rs` conflita.
5. **Kanban:** a roda passa `[dx,dy]` pelas camadas (linha que mexa nas camadas de roda conflita); a Tabela sem a 2.ª conversão (quem afirmar o sentido antigo reprova).
6. **`input_dispatch.rs` 547** e o `on_focus_lost` novo em `despacho_metodos_janela_e_vetor.rs`; o Gap Closure vive em `ph2d_app_flip::gap_live::gap_wheel` (quem editar o ajuste na shell conflita).
7. **`ControlsScreen::click(hit)`** sem `PlayerKeys`; `host::click(desceu, principal)`: chamadores novos noutras linhas não compilam.
8. Id `TOPBAR_PLAYER_CONTROLS` e ícone `gamepad` novos (listas de ordem: outra linha que acrescente ícone/verbo conflita). `CENAS` 22 (`ph2d-app-game/src/smoke.rs`) e `.config/testes-lentos.tsv` (gerado). Schema 232 (o da onda 1). Itens 1–11 do §7 da onda 2 valem, salvo o 9 (547) e o 10 (`wheel_sideways`).
9. **A pílula de dica da topbar passou a quebrar linhas** (§2b f): linha que mexa em `paint_hover_tooltip`/`tooltip.rs` conflita; dicas de uma linha podem ficar um pouco mais largas (peso Medium medido).

## 8. Prova de fecho (feita na linha)

- `cargo fmt`, `clippy --workspace --all-targets -D warnings`, machete, `check-standalone-optional.sh`, `check-workflow-packages.sh`: ✓.
- Censos da árvore combinada **132/133**: o vermelho (`ph2d-app-code` 26 639 > 26 628, teste da mutação) curou-se em `e6e8658d39`.
- Testes por nome (a re-corrida abaixo é a vigente): `ph2d-player-controls` (47/11/5/7/4/3/13 nos ficheiros `tests`, `onda3_hover`, `_r2`, `r10a`, `r11a`, `f3`, `play`); `ph2d-panel-tokens` `familia`/`os_cartoes`/`seam_tokens`; `ph2d-app-flip` `gap` 11; `swipe_lateral` ×6.
- **Mutação: 54 mutações, 48 sangraram na 1.ª; 6 sobreviventes mortas** por testes novos (`0199210908`: `mutacao_onda3.rs`, `completar.rs`, `onda3_hover_r2_tests.rs`); a onda 2 e as 4 leis da onda 1 continuam a sangrar. **NÃO mutada:** a ordem `dock_seam_move` antes da aba em `input_dispatch.rs:173` (lei da shell; só W2a/W2b na aba a protegem) — o integrador pode mutá-la com um teste de App.

- **Re-corrida pós-smoke, verde (botão: `enum_order`, `hr12`, `no_magic_numeric`, `file_loc_cap`, `the_shell_only_shrinks` ×2, `a_tabela_inglesa`, `topbar`, `menu_bar`, `chrome::transport`, `the_app_frame_is_reachable`; fmt e clippy `--workspace` limpos):** `cargo fmt`; `clippy --workspace --all-targets --keep-going -D warnings`; `input_map::` **234**; `onda3_hover` 11; `ph2d-tokens` 196; `ph2d-tokens-dtcg` 26; `a_altura_de_abertura`, `familia`, `file_loc_cap`, `the_shell_only_shrinks` 2, `the_shell_only_shrinks_and_every_app_crate_too`, `no_magic_numeric` 3, `a_tabela_inglesa_fala_ingles` 4.

## 9. Smoke (do dono) — **OK para toda a onda 3 (10/11), tela Controls incluída; o botão novo da barra do Play (§2b e) está PENDENTE de smoke**

Comando: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap && PH2D_GAME_SMOKE=18 PH2D_GAME_SMOKE_DRIVE=editor bash scripts/ph2d-run.sh cargo run -p ph2d-host-desktop --profile smoke` (cenas 6..22; P1..P4 = 19..22 pedem `PH2D_GAME_SMOKE_DRIVE=editor`). Fotos em `docs/InputMap/fotos/` (`ae3cde492c`). Gamepad físico não smokado.

## 10. ABERTO e fora da linha

- O fundo de repouso da tecla é `KanbanCardRaised` (token de dado de outra família, desde a onda 1, decisão aprovada): nenhum neutro de cromo passa o contraste das barras (plano §14.2).
- Kanban Calendar/Dashboard e drawer da Timeline: sem swipe lateral.
- **Por confirmar no app:** o Planner manda a roda crua ao Calendário que espera a convertida.
- Fora por ordem: legendas do numérico, R7b, ◆ em Dead/Press, Inspector vazio.

## 11. Proposta para o `CLAUDE.md §5.1` (só na integração; a linha NÃO a escreve)

Sem entrada própria. Proposta (≤ 700 B):

> - **Input Map** — aba central com três vistas (List · Keyboard · Try it), teclado de notebook e comando, tela «Controls» no jogo (F1), ligação física, Luau pela ação (`ph2d-input`, `ph2d-input-map-editor`, `ph2d-app-input-map`, `ph2d-player-controls`, `ph2d-input-winit`). Smoke `PH2D_GAME_SMOKE=6..22`. Último: [onda 3, 10/11](docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_2026-11-10_ONDA3.md) · [plano](docs/InputMap/01_plano.md)
