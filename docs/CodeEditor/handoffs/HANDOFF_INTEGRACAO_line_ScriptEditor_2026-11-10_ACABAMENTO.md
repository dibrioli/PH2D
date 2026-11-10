# HANDOFF DE INTEGRAÇÃO — line/ScriptEditor: acabamento 10/11 (tabelas da casa, caixa de marcar, molduras, rodada do dono)

> Leitor: o agente integrador. Antecessor: [Onda B + plano 14 Q1–Q4](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-09_PLANO14_Q1_A_Q4.md)
> (**se ainda não estiver no main, integre-o primeiro**; esta linha assenta nele). Mecanismo e diário:
> [handoff de continuação do acabamento](HANDOFF_line_ScriptEditor_2026-11-10_ACABAMENTO.md) (§A e o smoke de lá estão
> **superados** por este: a pílula saiu). Decisões do dono: [00_decisoes_do_dono.md](../00_decisoes_do_dono.md).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor` · branch `line/ScriptEditor` · HEAD de código `c6059ec8af`
  (o commit deste handoff vem a seguir: só docs).
- Merge-base com o `main` = `bc4a6ce930` (`git merge-base HEAD main`). **14 commits** (`git log --oneline bc4a6ce930..HEAD`),
  **62 ficheiros, +1018 −218**.
- **Smoke do dono 10/11: APROVADO («Smoke OK»).**

## 2. Superfície de colisão

Ficheiros tocados (`git diff --name-only bc4a6ce930..HEAD`), por crate:

| Crate / pasta | Ficheiros |
|---|---|
| `ph2d-panel-code` | `src/`: `bancada/{grid,mod,outline,phrase,popover}.rs`, `cards.rs`, `chrome.rs`, `console.rs`, `console/problems.rs`, `debug_card/{boxes,head,live,live_marks}.rs`, `debug_marks.rs`, `metrics.rs`, `options_card.rs`, `script_list.rs`, `status.rs` · `tests/it/`: `bancada.rs`, `bancada/{acabamento,grelha,puras}.rs`, `main.rs`, `medicoes_do_fecho.rs`, `opcoes_do_script.rs`, `problemas.rs`, `quadro_do_depurador.rs`, `valores_ao_vivo.rs` |
| `ph2d-app-code` | `src/{doc,surface}.rs`, `tests/it/onda_b.rs` |
| `ph2d-code-bancada` | `src/{act,lib}.rs`, `tests/it/acto.rs` |
| `ph2d-editor-core` (foundational) | `src/widget/{mod,toggle_classic}.rs` |
| `ph2d-tokens` (foundational) | `src/{lib,state_layer}.rs`, `tests/it/{main,the_hover_layer_lightens_a_dark_surface}.rs` |
| `ph2d-ui-table` (foundational) | `src/{lib,paint}.rs`, `tests/it/{main,reserva_e_fios}.rs` |
| Hook / scripts | `.claude/hooks/prazo-do-agente.sh`, `scripts/processo.prova.d/I.sh`, `docs/Components/ferramentas/fotografa_cena.sh` |
| Docs / fotos | `docs/CodeEditor/handoffs/HANDOFF_line_ScriptEditor_2026-11-10_ACABAMENTO.md`; fotos em `docs/CodeEditor/ferramentas/smoke/` (`ac_*`, `ad_options`, `ad_botoes`, `cena13/ac_passo_{1,2,3}`, `cena13/ad_passo_1`) e `cena14/fotos_q3.sh` |

**Provas de contrato (corridas ao escrever):**

- Contratos §6 (Nodes, Tools, Vector): `git diff --stat bc4a6ce930..HEAD -- crates/ph2d-nodegraph crates/ph2d-tool-api crates/ph2d-vector` = **vazio**.
- `PROJECT_SCHEMA`: `git diff bc4a6ce930..HEAD -- . ':!*.md' | grep -c PROJECT_SCHEMA` = **0**. (Sem o filtro dá 1 linha: a frase
  «`PROJECT_SCHEMA`: intacto» do handoff de continuação.) Nenhum registo, nenhum ADR, nenhuma crate nova, `Cargo.lock` não tocado.

**Foundational tocado — ADITIVO (grep em `crates/` e `shells/` = 0 consumidores dos removidos):**

- `ph2d-tokens`: `state_layer.rs` novo com `HOVER_LAYER_ALPHA` (Material 3, 8 %), usado pelo hover da linha de opções, que agora
  **clareia** o cartão. `switch_knob` entrou e saiu dentro desta linha: saldo líquido = **uma const nova**.
- `ph2d-editor-core`: `paint_switch` e `switch_inks` (`widget/toggle_classic.rs` + re-export em `widget/mod.rs`) **REMOVIDOS**; vieram da
  integração anterior do ScriptEditor e não têm outro consumidor. Outra linha que os use parte a compilação aqui.
- `ph2d-ui-table`: `Metrics::twist`, `rules_bottom`, re-export. Consumidores da crate: só `ph2d-panel-code` e `ph2d-code-debugcard`
  (auditado: a Sheet do depurador mantém o ▸ porque os grupos expandem; as tabelas planas passam a alinhar corpo e cabeçalho).

**O que um merge pode partir:** quem importe `paint_switch`/`switch_inks`/`switch_knob`; o intervalo `PH2D_CODE_SMOKE=1..14`
(`CENAS`, inalterado) e `STEPS_13`/`STEPS_14`; os tectos de LOC (secção 5); as assinaturas `paint_bench`/`paint_popover`/
`ConsoleView::paint`, que agora levam `&mut CodeView`; o nome do ficheiro de teste de tokens (renomeado).
⚠️ **Hook `.claude/hooks/prazo-do-agente.sh`:** o que corre é o do checkout PRIMÁRIO, não o da worktree — a cura (item C abaixo) só
vale depois da integração; nesta linha os subagentes foram recusados com `-p ph2d-code-bancada` e com `git diff` que continha a palavra.

## 3. O que a linha entrega

| Item | O quê |
|---|---|
| A | Cartão «Script options»: o liga/desliga é a **caixa de marcar do cartão** (`paint_card_check`/`card_check_rect`), não a pílula (dono 10/11: «o padrão do app é checkbox e não toggle»). Hover da linha clareia o cartão. Pílula e disco saem do repo (ver §2). |
| B | Tabelas da casa (`ph2d-ui-table`): números na mono do código (grelha «Who has what», balão, Problems, via `CodeCells`); fios das colunas param na última linha inteira (`rules_bottom`); a reserva do ▸ só existe se alguma linha expande (`Metrics::twist`). |
| C | Restos da Onda B: Vec2 da grelha aceita a vírgula decimal (`parse_vec2` em `ph2d-code-bancada/src/act.rs`: `;` separa; senão a única vírgula que toca um espaço; senão dois tokens por espaço; senão `,`; número nunca começa/acaba em vírgula). `FOTO_TECLAS` em `fotografa_cena.sh` recusa alto (exit 3) e aponta `fotos_q3.sh TECLAS=` (XTEST na Xvfb funciona; o Xwayland do `kwin --virtual` ignora). Hook: recusa de subagente = «comando pesado com `--ignored`/`--include-ignored`/`--run-ignored`» (com ou sem «bancada»); `cargo` depois de aspa (`bash -c`) e o alias `cargo t` contam como pesado; `fila-pesada.sh correr` é do coordenador; a palavra solta (grep, caminho, teste por nome, `git log`) passa. 12 casos de prova em `scripts/processo.prova.d/I.sh` (corrigido o caso «janela principal» que colava dois argumentos e passava sempre). |
| D | Molduras de valor de volta nos temas modernos: a integração `411a89fa5c` passara o valor ao vivo no código, o campo de valor do depurador, a dica do traço e o gráfico grande por `stroke_frame(.., Feel::Rest)`, que a pele plana apaga em repouso. Agora `ph2d_panel_code::cards::value_outline` (sempre traça; `FRAME-RAW-OK` com razão). Os botões da cabeça do depurador ficam na porta do tema (são controlos). |
| E | Rodada do dono 10/11: faixa da frase do pé = `ROW_H_PX` (25 → 22 px), texto pela `folga_do_texto`; «Continue F8»/«Next line F10»: rótulo centrado pela `folga_do_texto`, tecla na linha de base do rótulo (`metrics::top_on_baseline`; também linha da cabeça, ritmo 2.5×/s, chip «quem», chip «Waiting», barra de estado); `chrome::label_in` passa a `folga_do_texto` (toda a aba do script); entradas do «In this script» em `outline_entry_h()` = Sm × `LineHeight::Loose` (22 → 19 px), títulos de secção mantêm `ROW_H_PX`. |
| F | Auditoria do fecho (`c6059ec8af`): `parse_vec2` com vírgula à cabeça («1 ,2» dava [1, 0.2]); contornos do hook (`nextest --run-ignored`, `bash -c "cargo …"`, `cargo t`, `--ignored` sem a palavra); lei única de centrar na mesma linha (nome na lista de scripts, título da grelha, título do balão, texto do campo de busca pela `folga_do_texto`, como o `label_in` ao lado; medido no cabeçalho da grelha: desvio 3,64 px antes, 0,64 px de arredondamento depois; tolerância do teste 1 px). |

## 4. Prova de fecho (1×; sem suíte nem nextest — regra do dono 23/10, são do integrador)

- **Testes por nome, verdes:** `ph2d-panel-code`: `opcoes_do_script` (7), `valores_ao_vivo` (7), `problemas` (3), `quadro_do_depurador` (6),
  `bancada::` (32, com `a_faixa_da_frase_e_uma_linha_sem_o_vao_e_o_texto_fica_no_meio`, `as_entradas_do_in_this_script_ficam_mais_juntas_que_os_titulos`,
  `os_textos_da_mesma_linha_centram_se_pela_mesma_lei`) · `ph2d-ui-table` `reserva_e_fios` (2) · `ph2d-code-bancada`
  `o_par_aceita_a_virgula_decimal_como_o_numero` · `ph2d-app-code` `o_passo_1_da_cena_13_abre_o_balao_de_speed_com_os_dois_herois` ·
  `ph2d-tokens` `the_hover_layer_lightens_a_dark_surface` · `bash scripts/processo.prova.sh` todo verde.
- **Mutações:** 18 nas frentes A–C (todas vermelhas; uma sem teste que a apanhe: o x da primeira coluna `x += twist` — a foto prova;
  as da pílula/disco referem código entretanto removido) · D: `value_outline` → `stroke_frame` = vermelho «Dark: a moldura do valor sumiu» ·
  E: 6 mutações vermelhas (caixa no tamanho da pílula, altura da frase, lei do texto da frase, passo das entradas, lei da tecla, lei do rótulo) ·
  F: vírgula separadora do Vec2 desligada = vermelho; lei antiga do título da grelha = vermelho (3,64 px); lei antiga do título do balão = vermelho (`[639.7, 636.7]`).
- **Portões por nome, verdes:** `file_loc_caps`, `fn_loc_caps` (`ph2d-host-desktop`), `arch_safe_clamp_only`,
  `architecture_the_shell_only_shrinks`, `architecture_workspace_file_loc_cap`, `every_frame_goes_through_the_theme_door`, `no_magic_numeric`,
  `no_literal_color`, `hr15_no_hardcoded_ui_strings`, `architecture_widget_showcase_coverage`, `architecture_claude_md_cabe_no_orcamento`,
  `architecture_panel_loc_cap`, `architecture_widget_loc_cap`. `fmt --all --check` ✓ · `clippy --workspace --all-targets -D warnings` ✓
  (`PH2D_SUITE=fecho`) e de novo em `ph2d-panel-code` + `ph2d-code-bancada` depois das curas da auditoria.
- **Binário do smoke**, construído duas vezes depois de `rm -rf target/*/incremental` com
  `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` (1.ª: 20,21 s). Saída da 2.ª:

```
▸ linha line_scripteditor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Blocking waiting for file lock on package cache
    Finished `smoke` profile [optimized] target(s) in 0.34s
```

- **Auditoria, 2 lentes independentes.** Regras: sem violação de código; handoff de continuação desactualizado (curado com a nota de «superado» no topo dele).
  Correcção: 3 defeitos confirmados, todos curados em `c6059ec8af` (item F).

## 5. Folgas medidas (recontar na integração)

| Ficheiro / crate | Medido / tecto |
|---|---|
| `ph2d-ui-table/src/lib.rs` | **692 / 700 — folga 8; ⚠️ uma linha a mais reprova o tecto do workspace** |
| `ph2d-app-code/src/doc.rs` | 687 / 700 |
| `ph2d-app-code/src/surface.rs` | 600 / 700 |
| `ph2d-panel-code/src/debug_card/live.rs` | 550 / 600 |
| `ph2d-panel-code/src/bancada/mod.rs` | 485 / 600 |
| crate `ph2d-app-code` | 26 579 (medido no fecho) × 26 628 (início da linha); tecto 26 628: reconte com `find crates/ph2d-app-code -name '*.rs' \| xargs cat \| wc -l` |

## 6. Smokes (comando exacto; o do dono foi feito e aprovado, fica para o próximo testador)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=11 cargo run --profile smoke -p ph2d-host-desktop` →
   engrenagem ⚙ à direita de «line 21» → «Script options»: **três caixas de marcar** à direita, «Format on save» marcada; o hover de uma linha **clareia**.
   Errado se: aparecer pílula/disco, ou o hover escurecer. Foto: [ad_options.png](../ferramentas/smoke/ad_options.png).
2. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=12 cargo run --profile smoke -p ph2d-host-desktop` →
   parado na linha 18: «Continue F8» / «Next line F10» com rótulo e tecla na mesma linha de base; faixa da frase mais baixa, texto no meio
   ([ad_botoes.png](../ferramentas/smoke/ad_botoes.png)). Deixe continuar: valores ao vivo **emoldurados** no fim das linhas 17–18
   ([ac_molduras_valores_codigo.png](../ferramentas/smoke/ac_molduras_valores_codigo.png)). Cena 14 passo 9 (Cards):
   [ac_molduras_valores_cartoes.png](../ferramentas/smoke/ac_molduras_valores_cartoes.png). Errado se: valor sem moldura.
3. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=13 cargo run --profile smoke -p ph2d-host-desktop` →
   passar o rato no ◆ da linha 1: balão com Hero 220 (omissão) / Hero 2 180 (mudado), números em mono, pontos encostados à esquerda; fios
   da grelha acabam em Hero 2; entradas de «In this script» mais juntas. Foto: [ad_passo_1.png](../ferramentas/smoke/cena13/ad_passo_1.png).
   Numa célula Vec2 escrever «1,5; 2» ou «1 ,2» dá o par (1,5 ; 2) e (1 ; 2).

## 7. Aberto

**Decisão do dono:** apagar `backup/line-ScriptEditor-2026-11-08` — **aguarda o sim dele; NÃO apague.**

**Memória** (primário, `project-memory/`): `feedback_the_app_standard_is_a_checkbox_not_a_toggle.md` + ponteiro no `MEMORY.md`,
**não comitados no primário** — o integrador funde-os com a rodada.

**Plausíveis, não provados (deixados em aberto pela auditoria):**

1. `cards::value_outline` é `pub` e esconde os chamadores do portão de molduras (5 chamadores, todos contornos de valor).
2. A tinta da caixa de marcar das opções é a do cartão em volta (`card_body()`), não a do popover.
3. O hover usa `resolve(Text1)`; o pressionado usa a tinta do cartão.
4. `status.rs` passa um px de fonte já escalado à `folga_do_texto` (só importa se o tamanho do texto ≠ Normal).
5. `surface.rs` pinta bancada/popover só se algum doc casa o id (hit rects velhos possíveis se nenhum casar; não se mostrou alcançável).
6. O limiar de `left_split` ainda assume `ROW_H` para as 3 linhas do outline.
7. `col_band` em `ph2d-ui-table` (`marks.rs`) não tem chamador e ainda cobre o corpo inteiro.

Os abertos de desenho da Onda B/Q3_Q4 que esta linha **não** tocou continuam valendo nos handoffs respectivos.
