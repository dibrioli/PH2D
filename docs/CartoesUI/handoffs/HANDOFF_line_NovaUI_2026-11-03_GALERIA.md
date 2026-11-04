# Handoff — linha `NovaUI`: fundação dos cartões + Widget Gallery em cartões (03/11/2026)

Worktree `Worktrees/line-NovaUI`, branch `line/NovaUI`, HEAD do código `60aaaa938d` + o commit deste handoff, merge-base com `main` = `3a3896a7ab`
(35 commits à frente, contando o deste handoff). **Não integrada, sem push.** Leitor: a próxima LLM / o integrador.
Plano-mãe: [06](../06_COMO_PORTAR_PARA_O_MAIN.md) §3 · desenho da galeria: [08](../08_galeria_em_cartoes.md).
O diff de `60aaaa938d` contra o merge-base: 344 ficheiros, +25 238 −1 836 (inclui docs e fotos).

## 1 — O que a linha entrega

1. **Fundação (famílias 2 e 4 de 06 §3)**, portada de `line/TrelloUITeste` `45908badd2` (`049989b94a`):
   `ph2d-card-stack`, `ph2d-card-rope`, tokens `Card*`/`Kanban*` + `derive::card_tone`, `ph2d-i18n/src/cartoes.rs`
   (111 chaves), `verlet-rope::corda`, e o núcleo `ph2d-editor-core` (`widget/card_ink`, `card_field`, `panel/cards*`,
   `interaction/state/card_*_ops`, despacho, `property_row/{cartao,escolha,marca}`). `panel/cards.rs` e mais quatro
   acima do tecto foram partidos por responsabilidade (`5ee70929c1`, movimento puro).
2. **Widget Gallery em cartões**, no encaixe da direita como um Inspector (doc 08 §1): **17 cartões de ferramentas
   à Figma** (Align, Boolean, Transform, Layout, Corners, Constraints, Fill, Stroke, Effects, Text, Export…),
   **14 casos reais** (Image, Physics Body, Collider, Health, Platform Player, Camera, Animation, Brush, Bones,
   Constraint, Light 2D, Audio, Pixel Art, Script), **5 amostras do Rive** (Interpolation, State machine inputs,
   Trim path, Solo, Gradient; URLs em 08 §1.4) e **11 cartões de sempre** com título de ferramenta.
   - `PH2D_CARDS=0` = a janela flutuante antiga.
   - `PH2D_GALLERY_SMOKE=1` abre a galeria já visível; `PH2D_GALLERY_SMOKE_SCROLL=<px>` segura a rolagem;
     `PH2D_GALLERY_SMOKE_SIDE=1` abre a linha do efeito. Fotos `maquete/g1..g7b`.
3. **A fundação continua um DAG** — curas de `7439627882` (o porte fechava três ciclos de módulos de topo e subia
   a catraca widget→interaction 43→44):
   - interaction→panel: a lista publica as linhas (`WidgetStore::set_color_style_rows`), o despacho só lê o store;
     o teste do ícone do menu foi para `panel/cards_menu.rs`.
   - widget→panel e property_row→panel: a tinta do corpo desce para `widget::card_ink`; o chip do dropdown para
     `widget/dropdown/card_chip.rs`; as três linhas do corpo (fields/choices/check) viram TABELA injectada
     (`property_row::cartao::CardRows`, publicada por `panel::cards_corpo::LINHAS`).
   - `CardInk::of_card` recebe o tema da secção; `CardFieldSpec` desce para `widget::card_field`.
   - `cards_estilos`: ids por literal (`hash_node_id`), o censo cobre-os.
   - `A11Y_OPT_OUT`: `card_ink.rs` e `dropdown/card_chip.rs`.
   - ⚠️ **PARA QUEM PORTAR O INSPECTOR (família 6):** `WidgetStore::publish_object_mode/object_mode` foi REMOVIDO.
     Só o «+» do topo da D25b do Inspector o lia, e ele fechava o ciclo interaction↔object_mode. O Inspector
     redesenha-o: p.ex. o `mode_drive` publica um bool simples (`at_top`) cujo campo é definido em `interaction`.
     Não repor o par antigo.
4. **Noto Sans de fábrica (D11)** veio com o porte. Curas do censo sob a Noto: `widget/showcase/encaixe.rs`;
   `property_row` (`que_quebra_o_nome`, `com_os_menus`: o nome que não cabe QUEBRA em duas linhas, o menu pede o
   chip da opção mais larga); `painter-layers` (secções); tokens.

## 2 — Foundational tocado e porquê

| Crate | Porquê |
|---|---|
| `ph2d-editor-core` (145 ficheiros) | núcleo dos cartões (porte), partição de `cards.rs`, curas de DAG, `showcase/` |
| `ph2d-tokens`, `ph2d-panel-tokens`, `docs/design/tokens.json`, `tools/ph2d-widget-sync` | tokens `Card*`/`Kanban*`, `derive::card_tone`, bloco gerado do `widget/` |
| `ph2d-i18n` | `cartoes.rs`, `galeria.rs` (180 chaves), `galeria_reais.rs` (91), `galeria_rive.rs` (36), `galeria_avancado.rs` (46); só inglês |
| `ph2d-node-motion-verlet-rope` | `corda.rs` (+72 linhas, para o `card-rope`) |
| `ph2d-panel-painter-layers`, `ph2d-panel-registry-init` | só testes e a secção que quebra o nome (ver §4 e §4.1) |
| `ph2d-app-kanban` | a cura do corte de 1 ulp nos chips da frente (ver §4.2); o aspecto aprovado não muda |
| `Cargo.lock` | crates novas (regenerar, nunca resolver à mão) |

Contratos congelados de CLAUDE.md §6 (`NodeOp`/`OpResolver`/`NodeManifest`, `Tool`, `Vector`): **nenhum tocado**
(`git diff 3a3896a7ab HEAD --name-only` não lista `ph2d-nodegraph`, `ph2d-tool-*`, `ph2d-vec-*` nem os gates
`*_contract_surface`).

## 3 — Ids, chaves e ícones novos

- `crates/ph2d-panel-widget-gallery/src/`: `ids.rs` 145 · `ids_reais.rs` 156 · `ids_rive.rs` 33 · `ids_avancado.rs` 22 constantes `pub`.
- `ph2d-i18n`: 180 + 91 + 36 + 46 (`galeria_avancado.rs`) chaves `panel.widget_gallery.*` (+ `cartoes.rs` 111, do porte).
- `docs/design/icons/card-*.svg`: **40** (28 no porte, 12 acrescentados depois, p.ex. interp-linear/cubic/hold e capsule). Nenhum glifo emprestado: o campo mostra a letra da grandeza (08 §1.3).

## 4 — Listas e catracas BAIXADAS (medidas, nunca subidas)

- `cada_nome_deste_painel_cabe_na_coluna_da_seccao` (painter-layers): coluna 220: 12 → 1; coluna 245: 1 → 0;
  a régua passou a perguntar à secção do PRODUTO (`seccoes::seccao`).
- Degrau estreito (`1f9360db07`): `inspector` 86 → 84 cortes (letras 84 → 82); `tokens` 2 → 1; `color_equalization` 2 → 1.
- Nestas curas nenhuma lista cresceu; as subidas estão na §4.1. Os testes de `a_docked_panel_never_reaches_the_drawing_area` e `a_panel_paints_where_its_tab_says`
  passaram a ler a declaração D24 (lista que FLUTUA) com controlo (`listas >= 1`).

### 4.1 — Catracas SUBIDAS, para o integrador julgar

⚠️ A §4 fala das listas que BAIXARAM. Estas duas subiram, em `e0572d8336`
(`crates/ph2d-panel-registry-init/tests/it/quantas_entradas_tem_cada_painel.rs`, `a_altura_de_abertura_de_um_painel_so_encolhe`).
Nenhuma é secção a nascer aberta; a galeria não está nessa tabela e não é a causa.

| Painel | De → para | Porquê | Grau de prova |
|---|---|---|---|
| `inspector` | 1488 → 1507 | a Noto de fábrica (D11) quebra uma legenda em mais uma linha | **medido**: sonda das duas fontes na mesma corrida, Inter 1488 × Noto 1507 |
| `painter_layers` | 1593 → 1618 | o nome que não cabe quebra em vez de cortar (`1787b8c3ba`), um passo (`1 × ROW_H_PX`) | **atribuído por leitura do commit**, sem medição separada da causa |

- A cura alternativa do `inspector` seria encurtar a legenda; a do `painter_layers` devolveria o corte. Julgue-as.
- `tokens` voltou a 2302 sem mexer no número: os 3 `card-*` dobram com a família Kanban (`+75 = 3 × 25`).
- **Lista que cresceu:** `PAINEIS_COM_ID_DERIVADO` ganhou `widget_gallery`. A forma foi olhada antes: os dois botões sem
  nome são `cards_estilos::botao_de(CARD_FILL)` e `botao_de(CARD_STROKE)`, o ⠿ dos estilos de cor (os dois ids conferem
  pelo FNV). Vieram com o porte e só a corrida inteira da `ph2d-panel-registry-init` os acusou.
- **Segunda porta do nome curada:** a lista da galeria lê `WidgetGalleryPanel::TITLE.tr()`, não `tr("panel.widget_gallery.title")`.
- **Excepção retirada:** `physics` saiu de `COLUNAS_DECLARADAS_POR_PAINEL` em `onde_comeca_o_valor.rs`
  (`dentro_de_um_troco`): mede 1 coluna (`x = 26`, a grelha), a lista é uma igualdade e menos é acertar.
  `widget_gallery` fica em 3.

### 4.2 — O vermelho do Kanban: `d14_field_names_are_never_cut_while_they_fit`

- **Medido:** no merge-base verde (árvore exportada, alvo próprio). No HEAD, Inter × Noto no mesmo processo:
  Inter `Priority: Option 3` inteiro (orçamento `97,56445` = texto `97,56445`); Noto `Priority: Option…` (orçamento
  `98,12399` × texto `98,124`, **1 ulp**). ⇒ os edits de `card_ink`/`date_picker`/`paint_card` de `d1c9126d41` não
  são a causa: a Noto (D11) só destapou o defeito.
- **Mecanismo:** o chip da frente mede-se (`texto + respiro`) e o pintor recalculava o orçamento por subtracção
  (`r.x + r.w − respiro − tx`), que perde 1 ulp conforme onde a medida cai. A elisão compara `<=` e corta. Era latente
  em qualquer fonte: com a cura desligada a varredura acusa as 10 primeiras opções na Noto e 7 na Atkinson.
- **Cura, na lógica de encaixe do próprio Kanban:** `FieldAt::room` leva ao pintor a largura MEDIDA; o chip de rótulo
  ganhou `front::chip_room` (a mesma família, mesma subtracção). Geometria e aspecto iguais; o nome inteiro é o que a
  Noto já pedia.
- **Gates novos:** `d14_a_chip_never_cuts_the_text_that_sized_it_in_any_font` (varre `UiFont::ALL` × 30 opções, porque
  o ulp depende da medida) e `a_label_chip_never_budgets_less_than_the_name_that_sized_it` (com controlo).
- **Provas de mutação (4/4 sangraram):** `room` volta à subtracção (`d14_field_names` + a varredura) · `chip_room` volta
  à subtracção · `hugging_rects` sem o `trim_end` · sem o `paint_demo_rows`.

## 5 — Lacunas conhecidas

- Paragens do Gradient: clicáveis, **não arrastáveis** (a posição muda-se no `Position (%)`).
- Camera: o alvo por arrastar da Hierarchy é só uma **dica** na galeria.
- Números de Brush, Image, Audio e Script são **fictícios** (os do código real estão no 08 §1.3).
- `CAN_FLOAT = false` faz o `PH2D_CARDS=0` mostrar também uma aba.
- **Metadata do Advanced sem entradas aninhadas:** nenhuma entrada de exemplo pede filhos, e o nesting não entrou.
- **Coluna estreita:** Metadata e Custom properties põem o valor na linha de baixo, em vez de cortar. É a decisão, não
  uma falha, mas a coluna estreita fica mais alta.
- **As barras da amostra do Comments não são interactivas:** no cartão mostram a rolagem `0` sobre o tamanho real das
  linhas (`demo_rows_content`); arrastá-las não mexe nas linhas. Rolar de verdade é o widget nas listas reais.
- **Mola do Kanban NÃO extraída** (D19): `ph2d-card-stack` existe, mas o `ph2d-app-kanban` ainda tem a sua
  própria — família 3 de 06 §3 pendente.
- `card-rope` portado **sem consumidor** até ao Inspector/Bones.
- Ainda por portar (linhas seguintes): famílias 5 (Bones) e 6–10 (Inspector, estilos de cor do Vector, cola da shell,
  cenas de smoke, testes `o_*`/`os_*`) — prompts em [`../PROMPT_janela_Bones.md`](../PROMPT_janela_Bones.md) e
  [`../PROMPT_janela_InspectorVector.md`](../PROMPT_janela_InspectorVector.md).

## 5.1 — Smoke do dono 03/11

Palavras dele: *«Dates parece fora do novo padrão. Card advanced tb parece fora do novo padrão. Slider Horizontal
diferente do vertical que é o padrão. Em cardback falta melhor coerência do design. Card Render poderia melhorar muito
seu layout e widgets. Na dúvida ver como Figma e Rive fazem. Restante me agradou muito e o smoke do restante está OK»*

| Item | O que foi feito | Commit |
|---|---|---|
| Dates | num cartão (`card_body`): rótulos `LABEL_*`, caixas `FIELD_H` que medem a data e nunca a cortam (a hora fica com o resto ou desce), marca `CHECK_BOX` única (`paint_card_check`), chip do cartão, Save/Remove lado a lado, superfície `KanbanCardRaised`; Calendar por `card_ink::tinta` | `d1c9126d41` |
| Card Advanced | `Bounds`, `Rotation`, grelha das 32 camadas nos tons do cartão, `Slice mode`, `Metadata` à Figma e `Custom properties` vivas; na coluna estreita o valor desce (`e0572d8336`) | `00b53b114f`, `e0572d8336` |
| Slider Horizontal | as duas barras saem de UMA lei (`AxisBar::of_theme`): calha da `track_rect`, `Bg3` + `thumb_color`, setas e raio iguais, eixos trocados | `d1c9126d41` |
| Card back | `RichEditor` com `Regua::Cartao`: uma coluna de marcadores, títulos Md/Base/Sm, código sobre `CardField`, traço fino da citação, barra Bare com os doze botões. Depois: linhas de amostra na caixa das barras (`paint_demo_rows`) e o fundo do código inline justo a cada linha visual (`hugging_rects`) | `d1c9126d41` + o fecho |
| Card Render | à exportação do Figma/Rive: `Format` + `Quality`, estado com `%` e barra fina, medidores L/R deitados com escala em dB, etiquetas que dizem o que são, corrida fictícia por quadro, `Cancel` só quando há o que cancelar | `00b53b114f` |

Fora de um cartão nada muda (testes de controlo e os do verso do Kanban). Fotos: `maquete/g6`, `g7`, `g7b`. **A rodada
destas curas ainda não passou pelo olho do dono:** o «OK» dele vale para o restante.

## 6 — O que só a integração apanha

**Impactados e suíte completa ficam para a integração.** Nada disto foi corrido na linha além dos testes por nome
e dos portões tocados. O schema não mudou nesta linha (nada a recontar).

## 7 — Saída dos portões

Rodada de 03/11 sobre a árvore de `60aaaa938d`.

- **Verdes:** `rustfmt --check --edition 2024 --config skip_children=true` nos 223 `.rs` do diff; `ph2d-panel-widget-gallery` inteiro (58/58); `ph2d-panel-registry-init` inteiro (158 passaram, 33 ignorados); `ph2d-editor-core` por nome (`card`, `icon`, `architecture_widget`, os três de LOC, `the_foundation_modules_form_a_dag`, `every_non_literal_hash_is_named`, `hit_indexed_ids_are_registered`, `no_hex_color_literals_in_widget_or_screens`, `no_magic_numeric_in_widget_or_screens`, `every_widget_file_wires_a11y`, `every_label_this_crate_paints_comes_from_the_string_table`, `no_panel_surface_wears_the_section_card_tone`, `every_registered_panel_is_reachable_by_the_z_order_walk`, `the_shell_only_shrinks`, `calendar`, `date_picker`, `rich_editor`, `scroll`: 163 + 59 passaram); `ph2d-app-kanban` por nome (`d14`, `calendar`, `card_back`, `a_label_chip`: 5 passaram). A corrida da `ph2d-panel-registry-init` foi antes de uma última edição só de forma em `demo_bars` (mesma conta, mudou a linha do `LITERAL-PX-OK`); depois dela correram `no_magic_numeric_in_widget_or_screens`, `the_horizontal_bar_is_the_vertical…` e `the_scroll_sample…`.
- **Vermelhos que NÃO são desta linha:** `clippy --workspace --all-targets --keep-going -D warnings` acusa só `ph2d-panel-wet-tuning/tests/it/seam.rs`, `ph2d-paint-tinta/tests/it/meio.rs` e `ph2d-app-motion/src/motion_shape_placa_quadro.rs` (fora do diff contra o merge-base); zero nos ficheiros da linha. `cargo machete` da rodada anterior não foi repetido.
- **Shell:** `shells/desktop` fora do diff contra o merge-base.
- **Impactados e suíte completa ficam para a integração.**
- `agent-loop-profile`: paralelismo 1,15/passo (alvo ≥ 1,5) · respostas por sessão 143 · `test:check` 2,6x (alvo ≤ 1) · edições sem conferir 1,3 % · contexto relido 224 mil · contexto no início 61 mil.
- `target/*/incremental` **não** foi apagado: faria o próximo smoke recompilar. Apague-o ao arrumar a linha.
- **Smoke:** `cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida:

```
▸ linha line_novaui · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.20s
```

## 8 — Smoke para o dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-NovaUI && PH2D_GALLERY_SMOKE=1 cargo run --profile smoke -p ph2d-host-desktop`.
2. A galeria abre na coluna da direita, como o Inspector. Role: Align, Transform, Fill, Effects…, depois os casos
   reais, as amostras do Rive e os widgets de sempre.
3. Deve acontecer: clicar «+»/«−» do Fill, arrastar a pega da curva em Interpolation, escolher uma paragem do
   Gradient. Tudo responde.
4. Deu errado se: nomes cortados com «...», uma letra emprestada num campo, ou um cartão que não responde ao clique.
