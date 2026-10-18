# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 17/10 — F18 (o desenho é o corpo do widget) feita

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, merge-base = `main` `ad5f8fe39c`. Anterior:
  [F17](HANDOFF_CONTINUACAO_line_UIUX_2026-10-16_F17_ABAS.md); integração anterior:
  [F16–F17](HANDOFF_INTEGRACAO_line_UIUX_2026-10-17_F16_F17.md).
- Fecha o **ABERTO 4** daquele handoff: «a pele Button desenha o botão do tema por cima da cor autorada».
- Plano, pesquisa e recusas: [`29_plano_F18_o_desenho_e_o_corpo_do_widget.md`](../vetor_nova_ui/29_plano_F18_o_desenho_e_o_corpo_do_widget.md).
  Pesquisa (§2): Figma/Rive/Flash = o desenho É o widget; Unity *Color Tint* recusado; mecanismo = Godot
  *local theme overrides* + propriedades CSS com escopo; tinta = CSS `contrast-color` + M3 *on-colour*/*state layers*.
  **Decisão do dono (17/10): (A) o painel gerado também veste a cor do desenho.**

## 1 — Commits desta janela

| commit | o quê |
|---|---|
| `3fe6cbef0c` | núcleo: `ph2d-tokens::skin_body` (escopo `Body::{Drawing,Ground,Flat}`), consultado em `ColorToken::resolve` e `Widgets::of`/`Chrome::of`; `ColorToken::role()` exaustivo; `readable_on`; `SkinParam.body`; `WidgetSkin{scene, over_drawing}`; painel gerado e `ui-codegen` levam `body`; `VectorScene::fill_shape` + `paint_rounded` gravam; cenas 27/28 + sonda headless |
| `da7c99e2fa` | o que a foto da 27 mostrou: `WidgetKind::drawing()`, Toggle+Checkbox = `Ground`, a pele recorta-se pelo contorno PREENCHIDO, gate de conteúdo, `paint_icon` grava |
| `e0e4ab2487` | o *Show* do Default parte da CENA: `Machine::settle_on` + `live_ids`; `ui_state_bridge::request_seen` |
| `4001e8a98c` | gate `o_rotulo_le_se_sobre_o_token_ligado_ao_fill` (fecha o sobrevivente da mutação) |
| `8017227b7b` | fecho: fmt, clippy, `generated/panel.rs` regenerado, `skin.rs` 502→499 LOC, fotos F18 |

## 2 — Superfície de colisão e decisões que um merge pode partir

- **Escopo, não parâmetro por chamada.** `skin_body` é um escopo (sem resíduo nem num pânico) com três
  corpos: `Drawing` (a cor do artista é a superfície), `Ground` (o desenho é o chão; as partes do widget
  ficam, guardadas ≥ 3:1 contra ele), `Flat` (o painel gerado; pinta a cor sem desenho por baixo).
  Quem lê cor de widget fora de `ColorToken::resolve`/`Widgets::of`/`Chrome::of` NÃO vê o escopo — é a porta a vigiar num merge.
- **`ColorToken::role()` é exaustivo** (Surface Rest/Raised/Pressed · Line · Ink Primary/Tertiary/Disabled/OnAccent ·
  Graphic): token novo sem papel não compila. Tinta = `readable_on` = o melhor dos polos do tema (`Text1`/`Bg0`),
  com recuo para preto/branco puro ⇒ sempre ≥ 4,5:1. *State layers* M3 8 %/10 %; tinta desativada 38 %.
- **`WidgetKind::drawing()` → `Drawing::{Body,Ground,Consumed}`**: Toggle e Checkbox = `Ground` (a foto da 27 mostrou
  o toggle sem pino e o checkbox sem caixa quando eram `Body`).
- **Despacho**: desenha a forma e depois a pele (`WidgetSkin.over_drawing`); a pele é recortada pelo contorno
  PREENCHIDO da forma (a moldura inclui o excesso do traço — a barra do slider vazava nas quinas).
- **`widget_live` passa o `Body` a partir do fill PINTADO** (o token ligado conta), nunca de `path.fill`
  (a mutação #8 era exatamente essa troca).
- **Painel gerado**: `ui_panel_spec`/`ui-codegen`/`panel-authored` levam `body` IRMÃO de `rgba` (nunca dentro dele);
  o painel pinta com `Body::Flat`. `generated/panel.rs` regenerado: `body Some([64,84,128,255])` nas linhas que vestem o desenho.
- **`ph2d-vector`**: `VectorScene::fill_shape` e `paint_rounded` passam a GRAVAR (a régua precisa de ver o fill);
  `paint_icon` grava por `stroke_path` (a régua não via o Spinner).
- **Show do Default**: `Machine::settle_on` + `live_ids`; `ui_state_bridge::request_seen` lê a cena por
  `ui_state_edit::capture` ANTES de pedir; o *Show* da shell usa-o. Causa medida no app real com sonda: na cena 25 a
  cópia ficava LARANJA porque a máquina nascia em Default e o atalho de `go_to` (`ph2d-ui-state` `machine.rs`) dizia «já estou lá».
- **Cenas 27 e 28** novas (sonda `vector_mode_vestidos`); conferir o número da próxima cena no roteador da família antes de abrir outra.

## 3 — Smoke (por fazer pelo dono)

Binário `smoke` JÁ compilado nesta worktree: `cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida
`Finished \`smoke\` profile [optimized] target(s) in 0.24s` (zero «Compiling»).

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=25 ./target/smoke/ph2d-host-desktop`
   — o botão à esquerda da estrela amarela é AZUL, com o nome «Path 0 (1)» em letras escuras. Errado = botão cinza/escuro do tema ou laranja.
2. Idem com `=27` — sob ele, uma fila: interruptor verde, slider roxo (barra branca dentro), caixa de seleção clara
   (o quadrado escuro é a caixa desmarcada), campo de texto escuro com letra clara. Errado = qualquer caixa cinza cobrindo uma cor, ou forma sem a sua peça.
3. Idem com `=28` — o botão é LARANJA (o Hover gravado, mostrado) e as letras continuam legíveis.
4. Na 27: clique no botão, abra a aba Interface, em States aperte *Show* em Hover e depois *Show* em Default —
   o botão fica laranja e volta a azul (é a cura do `e0e4ab2487`). Errado = nada acontece.
5. Na 27, ligue o *Preview* (seção States) e passe o rato sobre o botão — laranja enquanto está em cima, azul ao sair.
6. Na Hierarquia renomeie «Path 0 (1)» para «Play» — o texto do botão muda.
7. Ainda pendentes de antes: cenas 23 (barra de controle) e 26 (aba Interface, palco tingido).

Fotos (smoke): [23](../vetor_nova_ui/fotos/F18_cena23_controle.png) · [25](../vetor_nova_ui/fotos/F18_cena25_botao_azul.png) ·
[26](../vetor_nova_ui/fotos/F18_cena26_interface.png) · [27](../vetor_nova_ui/fotos/F18_cena27_fila_vestida.png) ·
[28](../vetor_nova_ui/fotos/F18_cena28_hover.png). Os `.log` irmãos (e os de F16/F17) ficam não rastreados.

Medido nas fotos: 25 e 27, corpo do botão `[40,120,240]`, rótulo mais escuro `[0,0,0]` = 5,04:1 (média do glifo 4,10:1 por
serrilhado); 28, corpo `[240,140,40]`, rótulo `[20,24,33]` = 7,18:1; 26, corpo `[58,130,242]` (tinto do palco), 4,69:1.
Fila da 27: toggle `[52,168,83]` com marca escura pequena à direita; slider `[124,77,255]` com a barra branca recortada
dentro das quinas; checkbox `[236,236,228]` com a caixa h×h escura (desmarcada) à direita — lê-se como botão partido
(lei antiga do canvas, BUGS_vector #26: a caixa preenche a altura do quadro); campo `[32,36,44]` com texto claro; a 23 não mudou.
Tempo de quadro: **não medido** (os logs não o trazem).

## 4 — Verificação

- Verdes em `8017227b7b`: `ph2d-tokens` `skin_body` (8 testes, incl. `a_tinta_escolhida_le_se_sobre_qualquer_corpo`
  com o pior cinza L≈0,179 COLOCADO, e `o_escopo_nao_deixa_residuo_nem_num_panico`); editor-core `widget::skin::tests::body`
  (censo 20 tipos × 12 temas × 5 corpos: `nenhuma_superficie_do_tema_cobre_o_desenho` com controlo de modo-tema,
  `o_conteudo_le_se_sobre_o_desenho`, `cada_pele_vestida_guarda_o_seu_conteudo`); vec-render
  `the_widget_skin_goes_over_the_drawing_or_replaces_it`; app-vec `o_corpo_da_pele_vestida_e_o_desenho`,
  `o_fill_do_estado_chega_ao_corpo_e_ao_rotulo`, `o_icone_e_a_amostra_nao_desenham_duas_vezes`,
  `o_rotulo_le_se_sobre_o_token_ligado_ao_fill`, `o_show_do_default_parte_da_cena_e_devolve_a_cor` (porta antiga como controlo),
  sonda `vector_mode_vestidos`; panel-authored `seam_authored_body` (`o_painel_gerado_veste_a_cor_do_desenho` + controlo);
  ui-codegen `the_body_comes_out_beside_the_colour_never_inside_it`.
- Censos verdes: `the_shell_only_shrinks`, tectos de LOC, `no_orphan_source_file`, `no_hex_color_literals_in_widget_or_screens`,
  `architecture_panel_wiring_parity`, `the_generated_panel_is_not_stale`.
- `nextest-impacted` (`PH2D_SUITE=onda`): 18 012 testes, 18 010 verdes + 2 falhas (tecto de LOC; painel gerado velho),
  curadas e re-corridas POR NOME, verdes. clippy `-D warnings` em 9 crates verde. fmt limpo.
- **Prova de mutação: 9/9.** 8 sangraram de início; a #8 (`body_of` com `path.fill` em vez de `painted(bound).fill`)
  SOBREVIVEU ⇒ gate novo (`4001e8a98c`), visto VERMELHO sob essa mutação.

## 5 — ABERTO

1. Smoke do dono (§3); cenas 23 e 26 sem confirmação explícita.
2. **Controlo que mente (adjacente, não curado):** a amostra *Fill* do painel do vetor mostra a cor da FERRAMENTA e só
   relê da forma na troca de seleção (`vector_bridge.rs:414`, `vector_bridge_style.rs:376` `seed_style_from_selection`).
   Depois de um *Show* de estado que muda o fill o painel mente até reselecionar.
3. Aparência de checkbox/toggle no canvas (caixa = altura do quadro; marca do toggle pequena): lei do BUGS #26 — o dono julga no smoke.
4. O *Preview* com rato real não está fotografado (a ferramenta de foto não move o rato).
5. Herdados do F16–F17: tracejado na placa só percorre 1 par; achado do profiler das sondas do Motion; grupos visuais além do Advanced.
6. Os sinais ainda chamam `request` (só correm dentro do Preview, onde a cena é a da própria máquina).
7. `CLAUDE.md` §5.1 UI/UX: **não** alterado nesta janela; ao fechar a linha, trocar o link do último handoff e o intervalo do smoke (`…26` → `…28`).

> ✅ **Smoke do dono: OK (18/10)** — cenas 25, 27 e 28 (e as pendentes 23 e 26) aprovadas.
