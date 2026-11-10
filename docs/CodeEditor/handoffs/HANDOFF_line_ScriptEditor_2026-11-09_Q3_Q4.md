# Handoff de continuação — `line/ScriptEditor`, plano 14 Ondas Q3 + Q4 (09/11)

> Plano: [14_plano_quadros.md](../14_plano_quadros.md) · maquete
> [14_quadros_do_depurador.html](../maquetes/14_quadros_do_depurador.html) · antes:
> [Q1+Q2](HANDOFF_line_ScriptEditor_2026-11-09_Q1_Q2.md) (smoke aprovado 09/11). **Não integrado, sem push.**
> Q3: commit `4960f77ba5`, smoke do dono **pendente** (cena 14, passos 7–10). Q4: feita (commits `a4852bd071`,
> `0a31976fab`, `a3259bc58d`, `3d62984e3d`, §6), smoke do dono **pendente** (cena 13 + Script options + Problems).

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

## 5. Smoke do dono 09/11 (Q3)

Três pedidos, todos no `ph2d-panel-code` (zero linhas na `ph2d-app-code`, 26 627/26 628; `code-smoke` 614/614):

- **Dicas de rato na tabela ao vivo** (`debug_card/live_marks.rs`, novo): `DebugCardView::look` devolve
  `PointerLook::Hand` sobre as células `line` e `value` da Sheet do Live (`cell_at` → `Layout::hit` →
  `Hit::Cell`; só em tabelas com coluna `line`) e lembra-a em `cell_hover` (vale um quadro, como o `spark_hover`);
  `press` guarda-a em `cell_pressed` até ao `release`. Pintura depois de `ph2d_ui_table::paint`: pairar = moldura
  `BorderStrong`/`Thin`, premida = `Accent`/`Thick`, ambas recuadas `Xxs` e com `frame_radius(Sm)`.
- **Nada mexe quando os números vivos mudam de largura**: `live_marks::Steady` guarda a maior largura natural por
  coluna (Sheet) e a maior largura do valor por chave (Strip); só crescem e zeram quando o quadro sai do Live
  (`DebugCardView::paint`: parado ou sem linhas). Na Strip o valor encosta à direita da largura estável e o traço
  fica a seguir a ela. Os Cards já tinham a caixa fixa da grelha.
- **Moldura nos números vivos do código** (`debug_marks.rs::paint_ghosts`): `Ghost.min_chars` (o maior número de
  caracteres visto por linha, `LiveSampler::widest`, zera com o sampler) dá a largura da moldura (o texto com os
  caracteres que faltam como «0»), `Border`/`Thin`/`frame_radius(Sm)`, `Sm` dos lados, a linha menos `Xxs`. O texto
  passou para a linha de base do código ao lado (antes centrava a caixa da letra e ficava ~2 px abaixo; foto).
- `live.rs` estava no tecto (600): `short_number`, `span_text` (re-exportados) e `tip` mudaram-se para
  `live_marks.rs`; `live.rs` 556.

| Prova (`ph2d-panel-code` `valores_ao_vivo`) | Vermelho (mutação) → verde |
|---|---|
| `as_celulas_line_e_value_pedem_a_mao` | sem o `cell_at` no `look`: reprova ✓ |
| `os_numeros_vivos_mudam_de_largura_e_nada_mexe` — Sheet: «1.2» → «12345.678» → «1.2», `lay.cols` da 3.ª = da 2.ª | `Steady::cols` sem o máx: reprova ✓ |
| idem — Strip: a caixa do traço não se desloca | `Steady::strip` sem o máx: reprova ✓ |
| idem — `Ghost.min_chars` não desce | `widest` sem o máx: reprova ✓ |

⚠️ A régua da largura precisa da mono MEDIDA: com um `CodeView::new()` cru o `advance` é 0 e todas as colunas ficam
no `min_col` (a mutação sobrevivia). O teste usa `pinta_medido` (uma pintura de aquecimento do código antes).

Verdes também: `quadro_do_depurador` 5; `ph2d-app-code` `depurador` 13, `barra_da_aba` 6; `ph2d-ui-table` `o_`/`a_` 12;
`ph2d-editor-core` `shell_only_shrinks`, `workspace_file_loc_cap`, `panel_loc_cap`, `no_magic_numeric`,
`no_literal_color`, `hr15`. Fotos: `ferramentas/smoke/cena14/passo_7.png` (molduras no código e a tabela),
`passo_7_pairar_line.png` (`RATO="568 533"`: moldura à volta do «17»), `passo_10.png` (Strip).

## 6. Q4 — a mesma família na tabela da casa (feito, smoke do dono pendente)

Quatro commits sobre `32b7bf216e`. Maquete: «A mesma família nos outros quadros» (linhas 261–316).

1. **`a4852bd071` — a tabela** (`ph2d-ui-table`): `Key = Cow<'static, str>` em `Column/ColSpan/Hit/TableState.widths/hidden`
   (a grelha tem uma coluna por `ph2d.property`); `TableState.selected` (vence changed e hover); `lit_col` (AccentSoft +
   título Text1) e `soft_col`; `Cell.changed` (banda `changed_tint` recuada `Xxs`, texto `Ink::Changed`) e `Cell.swatch`;
   `Metrics.header_h = 0` = sem cabeçalho (nem pintura nem clique); `paint_cell_frames`/`Layout::cell_frame` movidos de
   `live_marks`; `StableWidths` (por chave, só crescem) substitui `Steady::cols`.
2. **`0a31976fab` — a grelha «Who has what»** (`bancada/grid.rs`) passa a `ph2d_ui_table::layout/paint`: coluna `@name`
   (`Cell.dot = ColorToken::layer`) + uma por propriedade (números `end`+mono); hover = linha sob o rato/espiada/em edição;
   `lit_col` = `Aim::Column/Cell/edição`; `text_field` da edição sobre `lay.cell_frame`; `grid_hits` saem da MESMA `Layout`.
   Tecto ⅓ da coluna em linhas inteiras, roda, barra ↕ arrastável, puxadores de coluna. `splitter.rs`: a lei do separador da
   Sheet passa a comum (`Splitter` serve grelha|mapa; `debug_card/split.rs` delega). `BancadaView::{release, look}`.
   `fotos_q3.sh` ganhou `CENA`.
3. **`a3259bc58d` — «Script options»**: pintura e estado (`OptionsCard`, `OptionsPainted`) da `ph2d-app-code` para
   `ph2d-panel-code/src/options_card.rs`; a app fica com a cola. Por opção: título, `Toggle` à direita, frase (Text3/Xs); a
   linha inteira é o alvo (mão; moldura de hover BorderStrong/Thin e de clique Accent/Thick, recuadas `Xxs`). Frases
   `code.options.{strict,format,hints}_says`.
4. **`3d62984e3d` — balão ◆ e Problems**: balão (`bancada/popover.rs`) = faixa «**speed** · line 1 … default 220» («heard by»
   no sinal ouvido) + tabela sem cabeçalho (ponto+nome · valor end/mono · estado Muted); `Row.changed` âmbar; `pop_hits` de
   `lay.rows`. Problems (`console/problems.rs`) = Problem (flex) · On (Muted) · Line (end, mono); F8 = `TableState.selected`
   e rola em linhas inteiras; barra ↕; `PaintedProblem`/a11y de `lay.rows`. `Layout::row_frame` + `paint_row_frames`.
   `Problem.on` (`ph2d-code-viewmodel`): o merge dá «Circle, Star +1»; a mensagem guarda «· ×N» / «— in Play» / «— ×N in Play».

### Provas (cada lei com vermelho por mutação)

| Teste | Mutação → resultado |
|---|---|
| `ph2d-ui-table`: 7 testes novos (chaves dinâmicas, coluna acesa, célula mudada, selecção, cor, sem cabeçalho, molduras, larguras estáveis) | 15 mutações, todas vermelhas ✓ |
| `ph2d-panel-code` `bancada::grelha` (5) | 8 mutações, todas vermelhas ✓ |
| costura da app na grelha (o Up larga o arrasto; `tab_pointer_look` pergunta `bench.look`): teste TEMPORÁRIO em `onda_b`, retirado (sem folga na catraca) | 2 mutações vermelhas ✓ |
| balão: sonda `popover_table()`; Problems: sonda `problems_table()` | verdes (mensagem do commit C) |

### Folgas finais (medidas 09/11, `wc -l` e a mesma contagem da catraca)

- `ph2d-app-code` **26 451 / 26 628** (folga 177; D: 26 628 → 26 455, C: −4).
- `ph2d-code-viewmodel` **304 / 304 — CHEIO** (qualquer linha nova aqui exige corte).
- `ph2d-panel-code` (cap 600 por ficheiro): `popover.rs` 220 · `problems.rs` 330 · `options_card.rs` 343 · `grid.rs` 366 ·
  `splitter.rs` 128 · maior da crate `help/layout.rs` 591 (não tocado) · `debug_card/live.rs` 557.
- `ph2d-ui-table/src/lib.rs` 685 / 700 (cap do workspace); `paint.rs` 435.

### Fotos e lado a lado

`ferramentas/smoke/cena13/q4_balao.png`, `q4_passo_1..3.png`, `q4_separador_rato.png` ·
`ferramentas/smoke/q4_options.png`, `q4_options_hover.png`, `q4_problems.png`, `q4_problems_hover.png` ·
**lado a lado** `ferramentas/smoke/maquete_lado_a_lado_q4.png` (Firefox headless `--screenshot --window-size=1500,2600` da
maquete + recortes: balão, Script options, Problems, grelha). Na maquete o Problems sai cortado na coluna Line (a grelha do
HTML estreita a coluna) e a grelha «Who has what» não existe lá: o par é o quadro A (planilha).

### Desvios conhecidos (para o dono ver no smoke)

- O interruptor sai como a **marca quadrada da casa** (`PH2D_UI_NEW`), não a pílula da maquete.
- «Strict types» mantém o nome da paleta e a frase **verdadeira** («Adds --!strict on line 1, so every type is checked»);
  a da maquete («Warn about names the script never declared») era falsa.
- **«On» só nos erros de corrida** («Circle, Star +1»); «—» nos do assistente (medido: os diagnósticos não sabem o objecto).
- A grelha não usa a mono (`UiText`); a linha acesa da grelha é a linha inteira; os fios das colunas descem pelo vazio
  abaixo da última linha; o ponto da gravidade/do balão fica deslocado pela reserva do ▸ na 1.ª coluna.
- **Nenhum teste corre a cena 13 passo 1** (só a foto).
- `fotos_q3.sh` ganhou `CENA` (display `:60+CENA+passo`): cenas com a mesma soma podem colidir.

### Aberto

Smoke do dono da Q4 (cena 13 passos 1–3 + cartão Script options + painel Problems). Sem integração nem push.
