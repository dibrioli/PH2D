---
name: reference_topic_code_pattern_gotchas
description: Padrões de código — os gotchas silenciosos dobrados do índice, verbatim, uma linha por memória
metadata:
  type: reference
---

# Padrões de código (gotchas silenciosos) — lições dobradas do índice (2026-09-10)

> Cada linha é uma memória com o gancho ORIGINAL do índice; abra o ficheiro para o mecanismo.
> Dobradas aqui porque o `MEMORY.md` passou o teto suportado (32 KB > 24 KB): ficam a dois saltos.

- [UI = gallery + inspector](feedback_ui_source_of_truth_gallery_inspector.md) · [UI em inglês](feedback_app_ui_english_only.md)
- [Nada de → em string literal (asserts também)](feedback_no_tofu_arrows_in_string_literals.md)
- [Pré-multiplicada inverte o Multiply](feedback_a_premultiplied_source_breaks_the_blend_whose_identity_is_one.md) · [clone + id de ponteiro = CoW; use versão](feedback_a_held_clone_plus_pointer_identity_change_detection_forces_copy_on_write.md)
- [Atributo separado do item por doc muda de dono](feedback_an_attribute_separated_from_its_item_by_a_doc_comment_changes_owner.md)
- ⭐ [A nota ao lado de uma CONTA descreve a população que ela tinha quando foi escrita — item novo herda-a sem a reconferir](feedback_the_note_beside_a_count_describes_the_population_it_had_when_it_was_written.md)
- ⭐ [Chave cujo dono nunca MORRE não tem sepultador: nem viva nem enterrada = invisível](feedback_a_key_whose_owner_never_dies_has_no_gravedigger.md) · [emparelhar por NOME solto parte a árvore — a chave é o CAMINHO](feedback_matching_by_a_loose_name_breaks_the_tree_the_key_is_a_path.md)
- ⛔⛔ **RE-DERIVAR por subtracção uma grandeza que já foi MEDIDA não a devolve, em `f32`.** Medido
  2026-09-14 (`line/UIUX`, report do dono *«3 pontos mesmo com folga»*, foto do cartão JUMP): um
  rótulo alinhado à direita começa em `recuo = x + (col_w − largura)` e o pintor recebia o orçamento
  `x + col_w − recuo`, *pretendendo* dizer `largura`. O cancelamento cai um ULP abaixo em **310 de
  3 600** células (9 rótulos × 400 posições de `x`, coluna `140 px`, o mais largo `100,3`), e o
  pintor **volta a cortar** um texto que já cabia — pondo reticências num nome com 40–77 px de
  folga. ⚠️ **A assinatura na foto é que o defeito NÃO ordena pelo comprimento**: `Takeoff Gravity`
  (`94,97`) saía inteiro e `Air Jumps` (`62,92`) cortado, na MESMA coluna — *quando o defeito não
  ordena pela grandeza que a lei usa, a causa não é a lei*, e isso mata a hipótese óbvia antes de se
  abrir um ficheiro. ⇒ **quem mede uma grandeza DEVOLVE-A** (a porta passou a `(texto, x, largura)`;
  ela já a calculava e deitava fora). ⛔ E a lei já estava escrita **um nível abaixo**, no próprio
  pintor que foi enganado: *«`INFINITY`, not `max_width`: it fits, and passing the budget back would
  let a sub-pixel measurement disagreement re-introduce the wrap»* — *o comentário descrevia
  exactamente o que o chamador fazia.*
- ⛔⛔ **Um piso que NOMEIA o recurso ainda pode estar errado sobre ele — e o recurso costuma já ter
  dono.** Medido 2026-09-14 (`line/UIUX`): escrevi *«o piso do controlo é `ICON_BTN_SIZE_PX` (36) —
  a largura da coluna do stepper de um `NumberInput` — mais um dígito»* ⇒ `48`. A coluna do stepper
  é `clamp(0,6 × altura, 16, 22)`, **nunca 36**: a justificação era falsa e o número não tinha dono.
  O campo **já declarava** o dele (`NUMBER_INPUT_MIN_W_PX = 72`) com uma ordem do dono ao lado desde
  2026-05-24 (*«não permita que a caixa seja redimensionada para menor que isso»*), e metade do
  curso do dock pintava o campo a `48`–`59`. ⇒ **antes de escrever um piso, procure quem já é dono
  do recurso** — e ⚠️ o gate que o defendia **re-derivava a mesma expressão do produto**, logo não
  podia acusar nada (*um gate que refaz a conta do produto mede a conta, não o produto* — 3.ª
  ocorrência em dois dias).

## Descidas do índice em 2026-09-17 — a rodada de seis linhas pôs o `MEMORY.md` a **224 linhas / 36,5 KB** contra o tecto dele (140 / 17 KB), e o carregador cortou **66 linhas em silêncio**. Estas entradas descem VERBATIM; o ponteiro para esta família continua no índice.

- ⛔⛔ [Um painel pode ter UMA coluna de nome por FAMÍLIA de linha (caixa · chip · número), as três no mesmo cartão — «este painel já foi convertido?» é a pergunta errada](feedback_a_panel_can_hold_one_name_column_per_family_of_row.md)
- ⛔ [Medir um texto num peso e pintá-lo noutro faz o pintor CORTÁ-LO (`0....`, e ao afastar some) — a porta é `title_elided_width`, ao lado do pintor](feedback_measuring_a_text_at_one_weight_and_painting_it_at_another_elides_the_text.md)
- ⛔⛔ [`try_query` com um `Option<&T>` devolve NONE se o MUNDO não conhece o tipo — a porta responde «ninguém» e nada o diz (um sinal por tag não alcançava nada)](feedback_a_try_query_with_an_optional_component_answers_nobody.md)

---

## «Vazio quer dizer o valor de omissão» perde em silêncio todo valor que não seja ele

⛔⛔ **Medido 2026-09-22 (`ph2d-gpu-cook::tex_runs`).** A partição de texturas do cozimento deixa a
lista VAZIA quando não há nada a dizer, e o desenho lê isso como *«o átlas, no modo de sempre»* —
uma convenção honesta e byte-idêntica enquanto o único eixo era a textura. No dia em que o run
passou a carregar também a **mistura**, o ramo vazio ficou sem onde a levar: *uma cena de Motion
comum é toda átlas*, logo o caso em que a lista fica vazia é exactamente o caso NORMAL de um
artista que escolhe outro modo.

⭐ **A cura não é encher a lista sempre** (isso mudaria o desenho de toda cena que hoje lá cai): é
o produtor emitir um item EXPLÍCITO só quando o valor não é o de omissão, por uma **saída única**
que os dois ramos de «não há nada a dizer» partilham. Escrita duas vezes, um deles a esquecer o
campo novo seria a saída a mudar conforme um detalhe do grafo — a forma que ninguém liga a uma
causa.

**Why:** uma lista vazia codifica um TUPLO de omissões, e acrescentar um eixo ao item aumenta esse
tuplo sem que nada no código o diga.

**How to apply:** ao acrescentar um campo a um item de uma lista cujo VAZIO tem significado,
pergunte o que o vazio passa a afirmar sobre o campo novo. Se ele afirma o valor de omissão, o
produtor tem de deixar de poder ficar vazio quando o valor não é esse — e o gate leva o CONTROLO
(no valor de omissão a lista continua vazia, byte a byte).
- ⛔⛔ [Canal de desfazer com CERCA: quando ela recusa, **LARGUE** — carregar a janela para a fila oposta faz o refazer **desfazer outra vez**, e só no dia em que a cerca voltar a aceitar](feedback_an_undo_channel_that_cannot_apply_must_be_dropped_not_carried.md)
