# 11 — Receita: converter um painel para a lista de cartões

Nasceu do bloco B (Physics, 04/11). Exemplo completo e testado: `crates/ph2d-panel-physics/src/cards.rs` (404 linhas,
9 cartões) + `tests/it/os_cartoes.rs`. Só o que foi medido; lei visual em [05](05_ESPECIFICACAO_UI_COMPLETA.md).

## Passos, por ordem

1. **Inventário ANTES de tocar.** Liste os ids que o `populate.rs` do painel regista e o `event.rs` despacha. Escreva-o
   num `fn inventario()` no teste novo, derivado das tabelas do painel (`os_cartoes.rs:147`).
2. **Interruptor** (`cards.rs:31-42`): `cards_on()` lê `PH2D_CARDS` e `force_cards_on_this_thread` força por thread.
   Nos testes antigos, uma linha no ajudante comum: `force_cards_on_this_thread(Some(false))` (`tests/seam.rs:69`).
   O ambiente é partilhado entre testes paralelos: nunca use `set_var`.
3. **Ramo no paint**, depois do `set_panel_rect` e antes da moldura antiga (`paint.rs:45`): `prepare(store)`, `cards()`,
   `paint_card_list(ctx, PANEL, título, &cartoes)`, publique alturas e `return`. Nada de `paint_panel_surface`.
4. **`prepare(store)` com o store mutável** (`cards.rs:66`), todo quadro:
   - `store.mark_card_field(chip, CardFieldSpec { step, scrub, default })`; `default` lido do `Default` do modelo.
   - `store.set_number_value(chip, valor)`, salvo se `store.number_input_drag()` é esse id.
   - Tooltip de cada ícone/célula muda (`tooltip_for(id).is_none()` antes de `set_tooltip`).
   - Fecho: `store.mark_card_list_close(PANEL, ID_DO_X)` (`card_list_ops.rs:197`). O Bones não tem «×», o Physics tem.
   - Secção que nasce dobrada: `store.set_collapsed_if_unchosen(id, true)`. Só aqui, nunca no `populate`: o caminho
     `PH2D_CARDS=0` tem de abrir como antes.
5. **Um `CardDef` por secção, com o MESMO id de secção** (`rows::section_header_ids`): ordem gravada, dobra, pega,
   tema e notas vêm com ele. `populate` e `event` não mudam.
6. **Corpo = tabela do painel**, em pares: `cells.chunks(2).fold(y, |y, par| c.row(par, y))` (`cards.rs:133`).
7. **Rótulo com a unidade**: chave nova `card.<painel>.*` em `crates/ph2d-i18n/src/cartoes.rs` só para as linhas com
   unidade; as outras reusam a chave antiga. Inglês, Plain English.
8. **Testes** (ver abaixo), **foto**, **portões**, commit por caminho.

## Que widget para que controlo

| Controlo de antes | No cartão | Id |
|---|---|---|
| Slider + chip ligado | `Widget::Field { id: chip, glyph }`. A barra deixa de se pintar; o scrub na pega faz o trabalho dela | chip no widget; a barra continua registada e é ela que avisa o painel |
| Dois números de UMA grandeza | 1 rótulo na 1.ª célula, letras X/Y, `.key(&[a, b])` na última (`cards.rs`, `gravidade`) | os mesmos |
| Dois números irmãos | 2 células, cada uma com rótulo e `.key(from_ref(&chip))` | os mesmos |
| Rádio / `paint_choice_row` com ≤ 5 opções | `Widget::Choice(&segs)`, via `c.full(Some(rótulo), …)` | cada `Seg.id` = id da opção |
| Caixa de marcar | `c.check(id, rótulo, on, y)`; o que não anima: `Rotulo::vista(TextKey::new(k))` | o mesmo; emite `Toggled` |
| Botão | `Widget::Button { id, text, primary }` | o mesmo |
| Facto só de leitura | **novo** `Widget::Readout { text }`: sem caixa, sem clique | nenhum |
| Dica que quebra | **novo** `c.hint(texto, y)`: devolve o fundo medido | nenhum |
| Matriz triangular | **novo** `c.toggle_triangle(&ToggleTriangle { … }, y)`: legendas, arrasto que pinta, grupo composto | os das células |
| Lista com muitas opções | `Widget::Dropdown` + `dropdown::declare(id)` em `prepare`; a caixa é `Button` só em cartões (`ph2d-panel-skeleton/src/seam.rs`, `lista`) | os de hoje |

Linhas novas (`hint`, `check`, `toggle_triangle`) moram em `panel/cards_linhas_extra.rs`, porque a linha de `Widget` tem
altura fixa de campo. Geometria nova: `ph2d_card_stack::metrics`; cor: `c.ink.color(token)`.

Marca de animação: só onde o id está numa tabela de canais (Physics: `ph2d-app-physics/src/marcas_mundo.rs`). Ajuste de
ferramenta ou de sessão não leva `.key` (D17).

## Armadilhas reais

- **Chip que o painel engole.** O Physics ignora `ValueChanged(chip)` e só ouve a barra. O Esc a meio do scrub não avisava
  a barra: a caixa voltava e o mundo ficava com o valor arrastado. Curado em `dispatch/card_field.rs:escape_scrub`, para
  todo painel. Teste o Esc mesmo assim.
- **O scrub tem limiar:** um único `Move` pode só cruzá-lo, sem mudar o valor. Nos testes, dois `Move`.
- **Mock não integra a rolagem nem a mola.** Pinte ~60 quadros em `aquecimento::com_passo_fixo` antes de ler rects. A roda
  conta em px: `wheel_at(x, y, -3.0)` rola 3 px.
- **`hit_at` no título devolve a PEGA** (`grip_de`), não a secção. Prove o clique pelo efeito (`is_collapsed`).
- **`i::NOME` não conta para o censo de canais:** `architecture_every_editable_field_is_a_channel` só lê `ids::NOME`. Em
  `c.check`, escreva `crate::ids::NOME`, senão aparece «id numa variável».
- **`type_keys_then_enter` não selecciona:** em caixa com `0` escrever `3` dá `30`. Use `type_into_number`.
- **O corpo do cartão corre mais de uma vez por quadro** (voo, solto), noutro `HitIndex`: não ponha efeito de estado nele.
- **Valor do preso:** um painel sem sujeito (Physics é o mundo) não precisa de `cards_presos`; o Bones precisa.
- **Agente em paralelo:** um `cargo test` em `registry-init` pode não compilar pelo WIP do outro. Espere e repita; não
  conserte o que não é seu. Em `zsh`, `$P` não divide palavras: use `bash -c`.

## Testes a escrever (modelo: `os_cartoes.rs`)

Por gesto real, nos rects pintados: inventário (registado e pintado em ALGUM estado alcançável; barras nunca pintadas) ·
scrub de cada número, escreve só o seu campo, e Esc devolve · escrever número · cada opção, caixa, verbo, célula · dobra ·
arrastar reordena (`section_order`) · soltar fora da coluna e o campo responder lá · rolar · botão direito abre o menu ·
tooltips (`dicas::icon_only_ids`) · `cards_off` pinta o corpo antigo. Contraprova: tire a linha e veja vermelho.

## Portões que apanharam o Physics (nomes exactos)

Em `ph2d-panel-registry-init --test it`:

- `quantas_entradas_tem_cada_painel::a_carga_de_comandos_de_um_painel_so_encolhe`: acusou `14 contra 2`. Era a escolha que
  não se declarava composta; `CardCtx::choice` agora declara. O que resta sobe **+1 por painel: o «…»** (Physics `2 → 3`,
  medido e comentado na tabela). Não é licença: qualquer outro acréscimo tem de ser explicado.
- `quantas_entradas_tem_cada_painel::a_altura_de_abertura_de_um_painel_so_encolhe`: acusou `1370 contra 1317`; o cartão
  custa mais que a linha antiga. Cura: nascer dobrado (Debug), e escrever o número novo, que é exigido se descer (`1236`).
- `a_marca_tem_a_altura_da_linha::toda_marca_booleana_tem_a_altura_de_uma_linha`: agora sabe que numa lista de cartões a
  caixa mede `FIELD_H`.

Em `ph2d-editor-core --test it`: `architecture_every_editable_field_is_a_channel::todo_campo_pintado_com_losango_tem_canal_ou_porque`
(ver `ids::` acima). Passaram sem mudança, corra-os: `no_magic_numeric`, `hr12_widgets_a11y`, `the_shell_only_shrinks`,
`architecture_widget_showcase_coverage`, `toda_dica_dos_cartoes`, `nenhum_nome_por_cima_do_controlo`, `onde_comeca_o_valor`.

Crate inteira do painel, uma vez, com `PH2D_SUITE="motivo: …"` (registo de painel, Cargo.toml).

## Painel com verbos Apply/Cancel (bloco D3, ferramentas de imagem)

`panel/cards_ferramenta.rs` (`panel::cards::ferramenta`) e `ph2d_ui_testkit::mesa::Mesa<P>` são a forma comum: `cards_on`,
`prepara_numeros`, `campo`, `caixa`, `opcoes`, `cartao`, `pinta_lista(ctx, panel, título, cartões, &Verbos)`. Modelo curto:
`ph2d-panel-padding/src/cards.rs` + `tests/it/os_cartoes.rs`.

- **Os verbos não são um cartão:** `Reset` e `Cancel | Apply` moram numa faixa presa abaixo da lista. Num cartão, o `Apply`
  dobrava-se, reordenava-se e saía para o canvas. A faixa entra no rect flutuante do painel, senão o clique nela vai ao canvas.
- **Liga/desliga pintado como botão vira caixa com o MESMO id:** `caixa()`. O id continua `Button` no store e emite `Click`:
  `populate`, `event` e `PanelEvent` não mudam.
- **Lista suspensa:** no `populate`, `Button` só em cartões (`cards_on()`), e `dropdown::declare` em `prepare`. Com a faixa, a
  lista aberta pinta-se depois dela (`adia_a_lista_suspensa`); medido: sem isto o `Reset` ficava por cima da opção.
- **Sem cabeçalho de secção no painel de antes:** o id novo vai para `ids` da ferramenta (`UPS_SEC_SCALE`).
- **Campo que mostra outra coisa que o pedido** (o EPX arredonda): `card_field::com_valores` no corpo, e a semente em `prepare`.
- **Abrir sem cena:** `PH2D_IMAGE_TOOL_SMOKE=padding|upscale|equalize_sizes|bgremoval|color_equalization`. Não há imagem: o
  `image_edit.mode_on` é o espelho do Edit de uma imagem, e o `mirror` em `ph2d-app-painter` é que respeita a variável.
- **Corpos antigos nos testes:** quem pinta o painel e testa o corpo de antes pede `force_cards_on_this_thread(Some(false))`.

## Fotografar

`bash docs/Components/ferramentas/fotografa_cena.sh PH2D_X_SMOKE=n 1930 1040 saida.png`, à frente `PH2D_PRAZO=<s>` (o guarda
pede). A variável de ambiente passa: `PH2D_CARDS=0 PH2D_PRAZO=200 bash … ` fotografa o painel antigo. Procure uma cena com
`ctx.want.panels.push("<painel>")`.

- **Ver a lista inteira:** janela mais alta (`1930 1800`). A `1930 1040` o que passa da dobra não se vê (05 §11b).
- **Ver estados que a cena não semeia** (rádio noutra opção, célula desligada, cartão dobrado aberto): patch temporário em
  `cards()`, foto, restauro por script com `try/finally`, sem commit.
- **Compare** com `maquete/bones.png` e `PH2D_VEC_BONE_SMOKE=8` lado a lado.
- A `1200` de largura o HUD «EDIT · fps» passou por cima da lista (`maquete/b_physics_debug_dobrado_hud_por_cima.png`);
  a `1930` não. Não é do painel; medido só no Physics.
