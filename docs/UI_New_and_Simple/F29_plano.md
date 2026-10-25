# F29 — plano (line/UIUX, 25/10)

Fila: [handoff F28 §8](handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-24_F28.md) (reports 1–3, fotos em
`handoffs/img_F28_bucket_weld/`) + report 4 do dono (25/10, a meio da onda): *«A tool Cut não exibe suas funções nem no
painel nem na barra flutuante, logo, não consigo cortar nada com ela. Vamos colocar na barra flutuante.»*

## Bloco A — o objecto composto (weld + balde) tem UM z (reports 1 e 2)

### Mecanismo medido (código, antes de qualquer edição)

1. **A tinta vai para o fundo da CENA.** `bucket::arm_new_fills` (`crates/ph2d-app-vec/src/bucket.rs:303`) faz
   `zorder::reorder(…, ZOrder::ToBack)` — a tinta é uma RAIZ com o z mais baixo da cena, e as linhas do composto
   ficam onde estavam. Um terceiro objecto entre os dois (o quadrado da foto 2) desenha por cima da tinta e por baixo
   das linhas: *«partes por baixo e outras por cima»*.
2. **A rede do balde vê os OUTROS objectos como paredes.** `bucket::upkeep` constrói a rede com `contornos_mundo` sobre
   **todos** os caminhos visíveis da cena (menos tintas). Arrastar o composto sobre o quadrado põe os arcos do quadrado
   na rede; as faces partem-se pela borda dele e a re-cozedura por âncoras devolve à tinta só os pedaços que as âncoras
   cercam: *«o quadrado corta a tinta amarela»* (foto 3).
3. As âncoras casam um arco pela **fracção do próprio contorno** (`FillAnchor { path, contorno, frac, frente }`,
   `crates/ph2d-ecs/src/vec_bucket_fill.rs:80`) — tirar da rede os caminhos de outros objectos não muda o casamento.
4. A pilha do Vetor (`zorder::keyed_stack`, `crates/ph2d-vec-entities/src/zorder.rs:313`) ordena por
   `(Z efectivo, DFS)`; o filho desenha **sobre** o pai e o `ShowBehindParent` (Godot `show_behind_parent`,
   `crates/ph2d-ecs/src/sorting.rs:170`) **não é lido** — só a chave de sprites (`sort_key.rs:185`) o honra.
5. O Z é relativo ao pai por omissão (`ZAsRelative`, `sort_key.rs:362`). Apagar um caminho despawna a entidade no `sync`
   (`entities.rs` passo 2), e o `despawn` leva os filhos; o passo 1 do `sync` seguinte tira o caminho deles.
6. **A solda escreve a rede NO LUGAR do participante mais ao fundo e apaga os outros**
   (`crates/ph2d-app-vec/src/weld.rs` `apply_vec_weld`). Uma tinta filha de um traço consumido morreria com ele.

### Cura

- **A tinta nasce FILHA do caminho-base, desenhada atrás dele.** `arm_new_fills` escolhe o pai entre os caminhos das
  âncoras (o da FRENTE na pilha — atrás dele a tinta fica por cima dos outros que a cercam e por baixo do traço do pai),
  pendura-a com `reparent_keeping_world`, tira o `RootOrder` (como o `group_entities`) e põe `ShowBehindParent`.
  ⇒ z = o do pai, move, esconde e apaga com ele.
- **A pilha do Vetor honra `ShowBehindParent`** com a mesma emissão da chave de sprites: `[filhos atrás] [eu]
  [filhos normais]`. O desempate da pilha passa a ser essa ordem de emissão; o «os meus descendentes» do Arrange
  continua a ler o DFS da árvore (campo separado), senão o *To Back* mira num filho atrás de mim.
- **A rede de cada tinta é a do OBJECTO dela:** paredes = o pai + os caminhos das âncoras que existem, unidos por
  componente (union-find) entre tintas que partilham caminhos — duas tintas do mesmo composto continuam a disputar faces
  como hoje. A rede global (todos os visíveis) fica só para o realce/clique do balde na mão: o que o artista vê aceso é
  o que nasce.
- **A solda passa as filhas dos traços consumidos para o anfitrião** (`apply_vec_weld` devolve o anfitrião e os
  consumidos; a shell re-pendura os filhos mantendo o mundo antes do `sync`). Vale para qualquer filho, não só tinta.
- **§7 (tinta órfã) à luz disto:** apagar o pai leva a tinta pela cascata. A regra «sem face E com âncora num caminho
  que já não existe ⇒ sai» continua necessária para a tinta antiga (raiz, documentos anteriores) e para a âncora num
  segundo caminho que foi apagado. O gate `apagar_a_forma_pintada_leva_a_tinta_do_balde` fica, e ganha o irmão da
  cascata.
- Tinta de documentos anteriores (raiz, sem pai): **não** é migrada por um sistema por quadro (escritor por quadro =
  passo de undo espúrio); fica raiz, com paredes = caminhos das âncoras.

### Gates (cada um visto VERMELHO por mutação)

- `a_tinta_do_balde_nasce_filha_do_caminho_e_atras_dele` — pai, `ShowBehindParent`, e na pilha imediatamente antes do pai.
- `a_pilha_desenha_o_filho_atras_do_pai` (`ph2d-vec-entities`) — mutação: ignorar o marcador.
- `outro_objecto_sobre_o_composto_nao_corta_a_tinta` — quadrado alheio sobre a tinta: geometria intacta.
- `a_solda_leva_as_filhas_do_traco_consumido` — mutação: não re-pendurar.
- `apagar_o_pai_leva_a_tinta_filha` (cascata pelo `sync`).

## Bloco B — booleana com um operando transparente (report 3)

### Mecanismo medido

`ph2d_vec_boolean::apply_chain_checked` (`crates/ph2d-vec-boolean/src/lib.rs:172`) veste o resultado com o estilo
do operando de CIMA (`folds.last()`) em TODA operação, e `compound_from` copia `fill`/`stroke` crus. O cortador de um
Subtrair é tipicamente o de cima e transparente (`fill: None` ou `Solid` alfa 0) ⇒ o resultado sai sem preenchimento;
pintar o operando muda a entrada do memo da booleana viva e o resultado «conserta». O Illustrator: *Minus Front* veste o
de TRÁS. A reproduzir num teste antes da cura (alfa 0 e `None`, Union e Subtract, viva e destrutiva).

### Cura

Uma regra de doador, no motor (uma porta para a destrutiva e a viva): ordem de preferência = o de cima… o de trás
(Subtract: o de trás primeiro); `fill` = o primeiro VISÍVEL nessa ordem, `stroke` idem; sem nenhum visível, o do
preferido. Visível = não `None` e não totalmente transparente.

Gate: `um_operando_transparente_nao_apaga_o_preenchimento` — mutação: voltar a `folds.last()`.

## Bloco C — a ferramenta Corte sem funções (report 4)

### Mecanismo medido (código; o XTest não chega à Xwayland da foto, logo o clique não se conduz no app)

1. **A barra flutuante (④) some com o Corte na mão**: `DrawMode::Cut` está em `PRECISE`
   (`crates/ph2d-panel-vector/src/bars_actions.rs:14`). Cortar/Descartar viviam só na fila ② do topo, e só com uma
   lâmina (`bars_row.rs`, `cut_line_exists()`); o painel lateral não tem secção do Corte.
2. **O clique do Corte quase nunca começa a lâmina onde o artista clica**: a shell chamava o `PenTool::on_press` comum
   (`despacho_clique_vetor_premido.rs`), que antes de começar um caminho REABRE uma ponta, AGARRA um nó (10 px) ou
   INSERE um nó no segmento da forma seleccionada (`crates/ph2d-vec-edit/src/lib.rs` `on_press`). Clicar sobre a forma
   a cortar editava-a; sem lâmina, nenhum botão aparece.

### Cura

- `PenTool::on_press_blade`: o mesmo `press` com `lamina = true` — sem reabrir, agarrar, inserir nem emendar; a shell
  chama-o com o Corte na mão.
- A ④ do Corte: com lâmina E seleção (é só então que o `apply_cut` corta), mostra [Cortar] [Descartar]; os dois saem
  da ② (uma casa só).

Gates: `o_clique_do_corte_comeca_a_lamina_mesmo_sobre_a_forma` (`ph2d-vec-edit`, com o controlo da caneta comum) e
`the_cut_chips_need_the_cut_tool_and_a_blade` (`ph2d-panel-vector`, reescrito para a ④).

## Cena de smoke

`PH2D_OBJECT_MODE_SMOKE=38` — o roteador do Vetor (`vector_mode_scenes.rs`) vai até 37 e a cena 7 já solda; o
`PH2D_AUTOKEY_SMOKE` é do Auto Key (a `=34` é a do Blend), não deste assunto. A cena: o composto (traços soldados + três
tintas) sobre um quadrado com z único, a booleana com um transparente correcta, e o Corte com a barra. Fotografada antes
de mandar.
