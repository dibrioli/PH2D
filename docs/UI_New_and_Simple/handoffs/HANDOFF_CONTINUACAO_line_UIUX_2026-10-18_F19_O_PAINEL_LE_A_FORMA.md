# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 18/10 — F19 (o painel do vetor lê a forma) feita

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, merge-base = `main` `ad5f8fe39c`. Anterior:
  [F18](HANDOFF_CONTINUACAO_line_UIUX_2026-10-17_F18_O_DESENHO_E_O_CORPO.md).
- Fecha o **ABERTO 2** do F18: «a amostra *Fill* mostra a cor da FERRAMENTA e só relê da forma na troca de seleção».
- Plano e pesquisa: [`30_plano_F19_o_painel_le_a_forma.md`](../vetor_nova_ui/30_plano_F19_o_painel_le_a_forma.md).
  Pesquisa: painel do Illustrator/Figma = a seleção; Blender com notificador por escritor RECUSADO (mudança por Python não
  redesenha até `tag_redraw`: o 11.º escritor esquece); Blender «campo laranja» / Figma modo protótipo = a edição que vai
  ser descartada é sinalizada ou bloqueada. **Decisão do dono (18/10): opção (A) do §5.1 — durante o Preview o painel
  mostra e não edita.**

## 1 — Commits

| commit | o quê |
|---|---|
| `bd93475249` | docs: plano 30 |
| `0c36db9b16` | fix: `seed_style_from_selection` comparava só o id (`STYLE_TARGET`); agora `StyleSeen {id, fill, stroke}` (dado do documento, unidades do mundo — o zoom não dispara) comparado por CONTEÚDO; `note_style_seen(pen, scene)` grava DEPOIS do passo 4 (a escrita do próprio painel) para que escrita própria nunca volte como alheia; `reseed_open_picker` re-semeia o BlenderPicker aberto (`set_blender_value`) para Fill (sólido ou o slot do gradiente via `selected_grad_color`) e Stroke. Parâmetro novo `grad_handle` em `seed_style_from_selection`. Gate `vector_bridge_alheia_tests.rs` (`dispatch` real; escrita alheia por `ui_state_edit::install` = porta Show/sinal/Preview; e troca do `VecScene` = `undo::restore`) |
| `f759163119` | feat: `ph2d_app_vec::ui_preview_lock::recusa(store, painel, alvo)` — com Preview ligado, um Down sobre `VECTOR_PANEL` / `INSP_PANEL` / `INSP_BLENDER_PICKER` é recusado (toast i18n `panel.vector.states.preview.locked` = «Preview is on: press Esc to edit again»), salvo vista: `VECTOR_STATE_PREVIEW`, `VECTOR_TAB_DRAWING/INTERFACE`, abas de slot (`slot_tabs::panel_for_tab`), cabeçalhos de seção recolhível. Porta da shell: `shells/desktop/src/input_dispatch/despacho_clique_flip.rs` (logo após o ramo Preview do canvas); teclado `input_dispatch/keyboard.rs` limpa o foco (`store.set_focus(None)`) com Preview ligado, antes de `forward_key_to_hero`. Gate de arquitetura `shells/desktop/tests/it/the_preview_refuses_panel_edits_at_the_pointer_door.rs`. Também `reseed_style_sliders` escreve `VECTOR_WIDTH_NUM` (`set_number_value`, respeita foco/arrasto): o chip Width da fila e o «W» da barra pintam do NumberInput, não do slider — a foto da 29 mostrou «3» sobre traço de 6 px (preexistente: mentia também na troca de seleção). Cenas 29/30: `crates/ph2d-app-vec/src/vector_mode_le_a_forma.rs`, estágios `250..=253`, `handles` 15/16 → `vector_mode_interface::FIRST`, fim roteado em `vector_mode.rs`; tabela `vector_mode_scenes.rs` 9..=30 |
| `94ae0b3b89` | chore: fmt do registo dos testes da shell; plano 30 regista a escolha (A) |

## 2 — Superfície de colisão e decisões que um merge pode partir

- **A única porta de detecção é `StyleSeen`** em `vector_bridge_style.rs`: canal de estilo novo em `StrokeSpec`/`Paint`
  entra de graça (comparação por conteúdo). NÃO reintroduzir notificação por escritor nem contador de revisão no
  `VecScene` (recusados; `morph_live.rs:53`).
- **`note_style_seen` fica DEPOIS do passo 4 de `vector_bridge::dispatch`** (as mutações M3/M4 sangram
  `o_painel_nao_briga_com_a_propria_escrita`).
- **A lista de painéis travados e de ids de vista do Preview mora na família** (`ui_preview_lock.rs`); a shell só pergunta.
- Shell cresceu ~25 linhas (duas guardas); `the_shell_only_shrinks` e `file_loc_caps` verdes.
- `PROJECT_SCHEMA` 191 intocado; nenhum contrato congelado tocado (grep = 0 em `tool.rs`, nos dois gates de superfície, `primitives.rs`).
- Cenas de smoke: tabela `vector_mode_scenes.rs` vai a 30; conferir o roteador antes de abrir a 31.

## 3 — Smoke (por fazer pelo dono)

Binário `smoke` já compilado nesta worktree. 2.ª corrida: `Finished \`smoke\` profile [optimized] target(s) in 0.23s` (zero «Compiling»).

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=29 ./target/smoke/ph2d-host-desktop`
   — botão laranja com contorno amarelo tracejado e grosso; painel da direita: Fill laranja, Stroke amarelo, Width 6,
   Cap Round, Join Round; barra de controle: FILL laranja, LINE amarelo, W ≈ 5,9. Errado = azul ou branco no painel ou na barra.
2. Idem `=30` — igual, contorno mais grosso; W = 9; continua laranja/amarelo.
3. Idem `=28` — o Fill do painel agora é laranja (na foto do F18 era azul).
4. Manual, na 29: Ctrl+Z / Ctrl+Shift+Z — o painel acompanha cada passo.
5. Manual: abra o seletor de cor do Fill e, na aba Interface, aperte *Show* em Default / Hover — o seletor troca de cor e a forma não volta atrás.
6. Manual: ligue o *Preview* (aba Interface › States) e passe o rato no botão — o painel fica laranja em cima, azul ao sair;
   clique em Width ou Fill no painel: nada muda e aparece «Preview is on: press Esc to edit again»; Esc sai; o botão Preview em si ainda funciona.

Fotos: [28](../vetor_nova_ui/fotos/F19_cena28_hover_painel.png) · [29](../vetor_nova_ui/fotos/F19_cena29_show_le_a_forma.png) ·
[30](../vetor_nova_ui/fotos/F19_cena30_width_nao_apaga.png). Os `.log` irmãos ficam não rastreados.

Medido nas fotos (pixel): cena 28 no F18 (antes) — canvas `[240,140,40]`, Fill do painel `[40,120,240]`, FILL da barra `[40,120,240]`.
F19 cena 28 — Fill do painel e FILL da barra `[240,140,40]`, LINE `[240,240,245]`, W 2,952. Cena 29 — Fill (painel+barra)
`[240,140,40]`, Stroke (painel)+LINE (barra) `[250,210,60]`, Width do painel 6, W 5,904, Cap/Join Round acesos; banda do contorno no canvas
4 px contra 2 px na 28. Cena 30 — W 9, banda 8 px, fill/stroke mantidos. Tempo de quadro: **não medido**.

## 4 — Verificação

- **Vermelho primeiro:** 4 dos 6 testes `alheia` ficaram VERMELHOS antes da cura, exatamente os previstos (Fill com a cor velha;
  Width apagava o laranja do Hover; seletor aberto reescrevia a cor velha; undo invisível); 2 controlos verdes antes e depois
  (`o_painel_nao_briga_com_a_propria_escrita`, `o_zoom_nao_re_semeia`). A asserção do NÚMERO (adicionada depois) vista VERMELHA (3,0 vs 6,0) antes do conserto.
- **Verdes por nome:** `ph2d-app-vec` `vector_bridge` (26+), `vector_mode` (58 com outros), `ui_preview_lock` (3); shell
  `the_preview_refuses` (2) + `file_loc_caps`; editor-core `the_shell_only_shrinks`, `workspace_file_loc_cap`.
- clippy `-D warnings` (`PH2D_SUITE=onda`) em `ph2d-app-vec`, `ph2d-i18n`, `ph2d-host-desktop`: verde. `fmt --all` limpo.
- **Prova de mutação 10/10 sangrou:** M1 comparar só o id → 4 VERMELHOS · M2 sem re-semear o seletor → `o_seletor_aberto` ·
  M3 `note_style_seen` vazio e M4 chamada removida → `o_painel_nao_briga` · M5 sem `set_number_value` → `o_painel_reve` (NÚMERO) ·
  M6 `recusa` sempre false → `os_controlos…recusados` · M7 sem isenção do Preview e M8 sem isenção de seção → `a_saida_e_a_vista_passam` ·
  M9 sem `set_focus(None)` e M10 Down→Up → gate de arquitetura da shell.
- Suíte `nextest-impacted` (`PH2D_SUITE=onda`) em `94ae0b3b89`: **19 051 corridos, 19 051 verdes**, 0 falhas,
  10 849 fora do impacto, 836 s (28 SLOW, o pior `ph2d-tool-painter … pular_a_pincelada…` 423 s).

## 5 — ABERTO

1. Smoke do dono (§3).
2. A isenção das abas de slot (`panel_for_tab`) não tem gate: o registo de painéis é do processo e não é instalado no binário de teste do `app-vec`.
3. Width está em px de TELA (lei antiga da ferramenta, `params.rs`): o número agora diz a verdade ao zoom atual — 2,952 a 98 % para um traço de 3 px desenhado noutro zoom. O dono julga se o painel deve falar em unidades do documento.
4. Seleção múltipla mostra o primário em vez de «misto» (o «?» do Illustrator).
5. Edições feitas no painel DURANTE um *Show* em curso (tween) são sobrescritas pelos quadros do tween (raro, curto).
6. O bloqueio do Preview cobre painel do vetor, Inspector e seletor de cor; edições na Hierarquia/Timeline persistem ao sair (não são canais de pose), por isso ficam abertas.
7. Opacidade dirigida (timeline) e cor por token são valores de VISTA (`VecDrivenStyle`, `vec_view.bound`), não `VecPath`: o painel mostra o valor autorado (o token tem hachura).
8. Herdados do F18: ABERTO 3 (aparência de checkbox/toggle, BUGS #26), 4 (Preview com rato real não fotografado), 5 (tracejado na placa só percorre 1 par, profiler das sondas do Motion, grupos visuais), 6 (sinais chamam `request`).
9. `CLAUDE.md` §5.1 UI/UX: **não** alterado nesta janela; ao fechar a linha, trocar o link do último handoff e o intervalo do smoke (`…26` → `…30`).

> ✅ **Smoke do dono: OK (18/10)** — cenas 28, 29 e 30 e os passos manuais (desfazer, seletor aberto, Preview).
