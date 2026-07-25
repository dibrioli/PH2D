---
name: feedback-a-mutation-that-does-not-bleed-may-indict-the-oracle-not-the-finding
description: "Mutação que não sangra pode acusar o ORÁCULO, não o achado — troque o oráculo pela propriedade que a mudança É, antes de descartar"
metadata:
  node_type: memory
  type: feedback
---

Quando a mutação que deveria provar um achado **não sangra**, há duas leituras: o achado é falso, ou o
**oráculo é cego a ele**. A regra "sobrevivente = gate faltando" ([[reference_topic_mutation_proofs]])
vale nos dois sentidos — antes de descartar o achado, pergunte *qual propriedade essa mudança É*, e meça
ELA.

**Caso real (Painter, cobertura da máscara, 2026-07-25):** a rota de Per-Layer Color escapava da porta da
lei nova. Dois oráculos reprovaram por CONSTRUÇÃO do fixture: o **feather** não podia responder (armar
camadas instala uma silhueta de Shape, cuja borda é dura por desenho ⇒ ~0,8 px sob qualquer lei) e a
**cor** não podia responder (a rota resolve para cinza ali ⇒ o gate passava COM o bug: verde por
vacuidade). O terceiro — **esfregar** (a propriedade que a lei governa) — mediu **16 níveis com a rota
aberta contra 0 com ela fechada**, e o guard ficou justificado.

**Why:** um oráculo escolhido pela conveniência do fixture, e não pela propriedade sob teste, produz
tanto falso-verde (o gate não pode falhar) quanto falso-descarte (o achado parece imaginário). O segundo
é pior: você remove a correção E escreve no doc que o defeito não existe.

**How to apply:** mutação não sangrou ⇒ (1) escreva em uma frase a propriedade que a mudança afirma;
(2) confira se o fixture consegue exibi-la (se ele fixa uma variável que a esconde, o oráculo está
morto — [[reference_topic_fixture_discipline]]); (3) só declare o achado falso depois de um oráculo que
mede a propriedade DIRETO. E registre os oráculos reprovados no gate: o próximo agente tentaria os
mesmos.

**Segunda instância, no mesmo dia e no mesmo módulo (a variante sem mutação):** a lei nova da cobertura da
máscara deixava contas por-dab visíveis; eu medi a modulação **no EIXO** do traço (6 níveis de 255) e
escrevi *"invisível"*. O eixo **satura em qualquer lei** — as contas vivem no **OMBRO**. A foto do Enio
disse o contrário do meu número, e o número não estava errado: estava no lugar errado. Antes de concluir
"desprezível" a partir de uma amostra, pergunte **onde a grandeza é MÁXIMA** e amostre lá (uma varredura de
coluna custa dez linhas de sonda).
