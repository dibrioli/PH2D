---
name: a-ratchet-over-a-non-invariant-quantity-blocks-its-own-cure
description: "Uma catraca que conta uma grandeza NÃO-INVARIANTE bloqueia exactamente a cura que a mensagem dela prescreve"
metadata:
  type: feedback
---

⛔⛔⛔ **Uma catraca que conta uma grandeza que NÃO É INVARIANTE bloqueia a cura que ela própria
prescreve — e o modo de falha é ela ter razão sobre o problema e estar errada sobre a régua.**

Medido 2026-09-22 (`line/UIUX`, os selectores de cor). O gate
`as_formas_de_um_selector_de_cor_so_encolhem` contava as **larguras distintas** dos `109` selectores
de cor do app (`[18, 32, 59, 120, 268]`) e exigia que o conjunto só encolhesse. Ele estava **certo
sobre o problema** (trazia o report do dono, com desenho) e a mensagem de erro dele até dizia a
cura: *«um selector de cor novo passa pela porta `paint_color_row`»*.

⭐ **E a largura de um selector que «enche a coluna» é função da largura do PAINEL** — `inspector`
dá `120`, `vector` dá `112`, a vitrina dá outra coisa. ⇒ **converter um painel pela porta que a
catraca prescrevia fazia SEMPRE nascer uma largura nova, e ela reprovava.** As duas tentativas
mediram `134` (coluna derivada no painel) e `112` (pela porta): as duas «formas novas».

⚠️ *A condição de sucesso dela — o conjunto encolher até um — era **inalcançável por construção**
enquanto os painéis tivessem larguras diferentes.*

**Como se reconhece:** a catraca nomeia a cura, você aplica **exactamente** essa cura, e ela
reprova. Nesse instante a pergunta não é «como passo?», é **«que grandeza é que ela está a
contar?»**.

**A cura:** trocar a grandeza pela FORMA que o nome dela já prometia — aqui, *«a swatch ocupa a
caixa que a porta dá à fileira dela»*, comparada **por painel** contra a própria porta
(`property_row::caixa_do_controlo`), sem um número escolhido. ⛔ Nunca afrouxar a lista, nunca
acrescentar a largura nova: as duas «passam» e as duas desfazem a padronização.

⚠️ **E a substituição escreve-se com a morte da premissa no lugar dela** — a catraca velha sai com
o parágrafo que diz porque a grandeza estava errada, senão alguém a reconstrói.

Ver [[feedback_a_gate_that_compares_two_constructions_is_blind_to_a_shared_mutation]] ·
[[reference_topic_gate_discipline]] · [[reference_topic_measurement_discipline]]

---

⛔⛔ **IRMÃO, medido no mesmo dia: uma lista de PRIMITIVOS que não acompanha as PORTAS que a casa
cria acusa precisamente quem as adopta.**

O `hr12_widgets_a11y` reconhece a11y por *«o ficheiro fia a11y» OU «chama um primitivo canónico»*,
com a lista a dizer *«keep in sync with `src/widget/`»*. Ao converter quatro linhas de cor para a
porta `property_row::paint_color_row`, **dois ficheiros ficaram vermelhos sobre código MELHOR**: eles
deixaram de nomear `paint_color_swatch` porque passaram a chamar a porta **que o chama**, e a cadeia
ganhou um salto que a lista não conhecia.

⭐ *O sinal é inconfundível: o gate reprova ficheiros que o diff melhorou.*

⚠️ E a cura tem **duas** metades, com a segunda a tornar a primeira honesta: o marcador novo
(`paint_color_row`) é o que faz a verificação da porta de crate casar pela **delegação real** em vez
de casar por acidente com outro `paint_*` do mesmo ficheiro — o defeito de subcadeia que aquele
mesmo gate já regista.
