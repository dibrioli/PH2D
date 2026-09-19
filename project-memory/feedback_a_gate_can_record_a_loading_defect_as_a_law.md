---
name: a-gate-can-record-a-loading-defect-as-a-law
description: Um portão pode ter um DEFEITO escrito dentro dele como se fosse uma propriedade do mundo — e fica verde sobre a população truncada que o defeito criou
metadata:
  type: feedback
---

Um portão que varre «todos os X» está a afirmar duas coisas, e só uma delas se lê: a LEI, e que a
lista que ele varreu é a lista inteira. Quando um defeito trunca a lista, o portão fica verde e **o
defeito passa a estar documentado dentro dele como se fosse uma lei do mundo**.

**Medido (Teste Cascadeur, 2026-09-19).** O `index.html` carregava `amostras_cascadeur.js` **depois**
de `exemplos.js`, e o `exemplos.js` lê as amostras **no momento em que corre** ⇒ o ficheiro entrava
na página e **não entrava no menu**: o app mostrava 6 exemplos e os instrumentos em node viam 14.

⛔⛔ E o portão que varria o menu tinha isto escrito no cabeçalho dele, em linguagem de LEI:

> «o único caso do corpus onde a barra de facto decide é a `amostra_16_07` — e ela **não é alcançável
> neste mundo**: as amostras são carregadas DEPOIS do ficheiro que monta a lista.»

Aquilo era **verdade** e não era uma lei: era um defeito de carregamento de três linhas de HTML.
Corrigida a ordem, o portão *«prender os pés nunca piora o escorregão nem o tranco, em exemplo
NENHUM do menu»* **reprovou de imediato**, sobre produto que já shipava — porque a afirmação era
verdadeira sobre 6 exemplos e falsa sobre 14.

⭐ **E a lei que ficou é mais forte do que a que substituiu**, porque a medição sobre a população
inteira tem duas metades onde a antiga tinha uma: *o escorregão nunca piora* (sem excepção) e *quando
o tranco piora, é uma TROCA e ganha-se mais do que se perde* (5,45→1,86 cm contra 0,81→1,09). Sem a
segunda metade, «piorou» e «trocou bem» leem-se igual num placar.

**Como aplicar:** quando escrever num portão que algo *«não é alcançável»*, *«não existe neste
mundo»* ou *«a lista não o tem»*, pergunte **porquê** antes de o aceitar como cerca. Se a resposta
for uma ordem de carregamento, um filtro, um caminho ou um nome, isso é um DEFEITO — e a cura dele
vai fazer o portão reprovar, que é o portão a funcionar. ⚠️ E todo portão de varredura devia imprimir
**o tamanho da população** ao lado do veredito: `14 exemplos no menu · 7 melhoram · 0 pioram` diz uma
coisa que `TUDO OK` nunca diz. É a mesma família de [[feedback_a_mutation_proof_needs_a_control_on_its_own_filter]]
— um filtro que casa menos do que devia lê-se igual a um produto correcto.

Ver também [[reference_topic_gate_discipline]] e [[reference_topic_measurement_discipline]].
