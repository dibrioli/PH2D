# Handoff de continuação — `line/ScriptEditor`, plano 14 Ondas Q3 + Q4 (09/11)

> Plano: [14_plano_quadros.md](../14_plano_quadros.md) · maquete
> [14_quadros_do_depurador.html](../maquetes/14_quadros_do_depurador.html) · antes:
> [Q1+Q2](HANDOFF_line_ScriptEditor_2026-11-09_Q1_Q2.md) (smoke aprovado 09/11). **Não integrado, sem push.**
> Q3: commit `4960f77ba5`, smoke do dono **pendente** (cena 14, passos 7–10). Q4: **por fazer** (§4).

## 1. Q3 — o «Live values» em tabela (feito)

- **Motor** (`ph2d-script-debug/src/ffi.rs::live_vars`): os valores ao vivo de uma linha levam as locais E os
  campos de cima do `self` (`self.t`), lidos com `locals_at(l, 0, 1)` e achatados (nenhum desce; tabelas ficam
  `{N fields}`). Antes só havia locais (a cena mostrava só `swing`). O texto do fim da linha também ganhou o `self.t`.
- **Anel** (`ph2d-ui-table/src/trail.rs`): `Trails::new(cap)`, `sample(iter (chave, texto, número))` — o número
  entra no anel (só os `cap` últimos), um texto sem número limpa o traço, quem não veio sai; `old(chave)` = o texto
  da amostra anterior se mudou; `Span::of` = mín · máx · actual. O traço (`paint_spark`) pinta no tom dos números.
- **Modelo** (`ph2d-code-debugcard/src/live.rs`): `LIVE_WINDOW_S = 4`, `LiveRow { line, var: VarRow, trail }`
  (chave `linha:nome`), `live_rows(live, &mut Trails)`, `DebugCard::live`, `live_card(who, rows)`. `GameStop` mudou-se
  da `ph2d-app-code` para `model.rs` (re-exportado em `crate::debug`). Re-exporta `LineLive`, `ValueKind`, `Var`.
- **Amostra** (`ph2d-panel-code/src/live_sampler.rs`): `LiveSampler { ghosts, rows }` — a MESMA leitura de 2,5 Hz
  (`ghosts_due`, `Duration::Slow`) dá os fantasmas e as linhas; anel de `window_samples()` = 10. `ghost_text` veio para cá.
  `rate_text()` = «2.5×/s».
- **Pintura** (`ph2d-panel-code/src/debug_card/live.rs`, 600 linhas — no tecto do ficheiro):
  - Sheet: `live_columns()` Line · Name · Value · Last 4 s (só o traço esconde-se); tabela a toda a largura, sem
    coluna do lado (logo sem separador ↔); o puxador de cima, o tecto de ⅓, a rolagem e a barra ↕ são os da Q2.
    Line leva à linha (`CardClick::Jump`), Value copia.
  - Cards: um mostrador por variável (nome + `ln N`, valor numa caixa com moldura âmbar se mudou, traço), em filas
    (`gauge_grid`) que rolam inteiras pelo `cards_scroll[0]` e a barra da casa (↕ pelo `look` de sempre).
  - Strip: «nome · ln N» pequeno, valor e traço ao lado.
  - Traços: `painted.sparks: Vec<(chave, Rect)>`. Pairar → `PointerLook::Hand` e a dica mín · máx · actual
    (`spark_hover`, vale um quadro, como o `split_hover`). Clicar → `DebugCardView::graph` (abre/fecha). O gráfico
    grande: na Sheet são `GRAPH_ROWS = 4` linhas falsas (`chave#graph0..3`) inseridas por baixo da linha, pintadas
    por cima (abrir rola até caberem: `reveal_graph`); nos Cards/Strip vai ao pé com o que sobra de uma fila (≥ 2
    linhas, senão não pinta). Conteúdo: máx/mín à esquerda com as linhas, o nome, o traço, «−4 s … now».
  - Faixa (`head.rs`): ● redondo no Live (■ parado) e «2.5×/s» à direita.
- **Decisão de acabamento (diferente da maquete):** no Live quase tudo muda a cada amostra — a faixa âmbar e o
  antigo riscado em todas as linhas eram ruído (foto). O âmbar fica SÓ no valor (e na moldura do mostrador / no rótulo
  da Strip); o passado é o traço. Logo, numa cena a mexer, os valores aparecem âmbar e não azuis como na maquete.
- **Cola** (`ph2d-app-code`): `DocDebug::live: LiveSampler` troca `ghosts` + `sampled`; `ghosts()` saiu; o
  sincronismo dos pontos de parada compactado (`mine` uma vez; o mesmo efeito, `depurador` 13/13). Catraca
  **26 627 / 26 628**.
- **Cena 14** (`ph2d-code-smoke/src/steps.rs`, **614 / 614**): 7 = Esc (fecha o menu das colunas do passo 6), tira o
  ponto, Continue → Live; 8 = abre o gráfico do `17:self.t` (`ClickDebugRow`: `chevron_of` devolve também o traço de
  uma linha do Live); 9 = Cards; 10 = Strip.

## 2. Provas

| Prova | Resultado |
|---|---|
| `ph2d-ui-table` `o_anel_guarda_as_ultimas_amostras_e_o_que_mudou` (+ os 11 de antes) | 12/12 |
| `ph2d-panel-code` `valores_ao_vivo` (2: tabela, mão, Line/Value, gráfico abre/fecha e empurra a linha seguinte; Cards e Strip) + `quadro_do_depurador` 5 | verdes |
| `ph2d-app-code` `ao_vivo_a_tabela_tem_o_traco_e_o_clique_abre_o_grafico` (cliques reais, relógio a 2,5 Hz) + `depurador` 13 · `barra_da_aba` 6 · `botao_direito` 6 · `onda1` 6 | verdes |
| Vermelho: sem o registo dos traços na Sheet, os dois testes novos reprovam | ✓ |
| `ph2d-script` `depurador` 24 · `ph2d-code-debugcard` 2 · `ph2d-app-components` `os_scripts_da_cena`, `a_cena` | verdes |
| `ph2d-editor-core` `shell_only_shrinks` `workspace_file_loc_cap` `panel_loc_cap` `no_magic_numeric` `no_literal_color` `hr15` `no_dependency_climbs` `every_member_inherits` | verdes |
| Fotos `ferramentas/smoke/cena14/passo_7..10.png`, `passo_7_pairar.png` (rato real), `maquete_lado_a_lado_q3.png` | conferidas |

Fotos: `ferramentas/smoke/cena14/fotos_q3.sh <passo> <saída> [espera]` (Xvfb `:60+passo`, uma tela por passo; com
`RATO="x y"` o xdotool pára o rato ali). ⚠️ Duas corridas na mesma tela sobrepõem-se (aconteceu com o sorteio).

## 3. Folgas medidas (09/11, depois da Q3)

`ph2d-app-code` 26 627/26 628 · `code-smoke` 614/614 · `code-scenes` 665/666 · `code-debugcard` 659/663 ·
`code-viewmodel` 302/304 · `code-assist` 3 692/3 925. `debug_card/live.rs` 600/600 linhas. Um passo novo na cena
14 exige cortar na `code-smoke` (comentários) ou uma crate Folha de cenas.

## 4. Q4 — o que falta e o que já se sabe

- **Balão ◆** (`ph2d-panel-code/src/bancada/popover.rs`, 171): hoje linhas à mão (●, nome, «default 220» /
  «changed 180»); maquete = tabela objecto · valor · default/changed, a linha mudada em âmbar, faixa «speed · line 1 …
  default 220». Os `pop_hits` (`Aim::Holder`, `Aim::Select`, `Aim::Mark`) têm de continuar a sair da MESMA geometria
  (cena 13 e `onda_b`).
- **Problems** (`ph2d-panel-code/src/console/problems.rs`, 201): maquete = colunas fixas gravidade · Problem · On ·
  Line (à direita). ⚠️ O «On» (quem é afectado) NÃO está no `ProblemRow` — os erros de corrida sabem o objecto, os
  do assistente não: medir de onde vem antes de construir (a cola da app-code tem 1 linha livre).
  `PaintedProblem` e o a11y (`problems_a11y`) leem a geometria.
- **Script options** (`ph2d-app-code/src/options_card.rs`, 244): interruptor + frase por opção. A pintura tem de
  MUDAR-SE para o `ph2d-panel-code` (paga as linhas da cola).
- **Grelha da Bancada** (`ph2d-panel-code/src/bancada/grid.rs`, 325) → `ph2d-ui-table`: realce de linha/coluna,
  célula alterada, edição de célula (a MESMA do Inspector), `Aim::Cell`, espiar pelo nome. Testes que a leem:
  `ph2d-panel-code` `bancada` (+ `tests/it/bancada/puras.rs`), `ph2d-app-code` `bancada`, `onda_b`; cena 13.
- O mesmo acabamento em todos: tecto, rolagem em linhas inteiras, barra ↕ com calha, separador ↔ só onde há coluna
  do lado, âmbar = mudou (`ph2d_tokens::changed`).
