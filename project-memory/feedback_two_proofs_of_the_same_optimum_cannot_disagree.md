---
name: feedback-two-proofs-of-the-same-optimum-cannot-disagree
description: Uma busca que se declara "provada" e devolve custos diferentes conforme a configuração não é uma prova — e o gate certo é sobre a INVARIANTE, não sobre o resultado
metadata:
  type: feedback
---

Quando um solver devolve **um ótimo e uma afirmação de que ele é O ótimo**, a única leitura válida de
dois valores diferentes é: *uma das duas "provas" não era prova*. Não arredonde, não escolha a menor,
não atribua a ruído numérico — **procure a incompletude**.

**Why:** medido em 2026-08-20 no `ph2d-quantize` (F4 do quad remesh). O ramifica-e-limita partia a faixa
de uma variável inteira nos dois **PONTOS** `x = piso` e `x = piso+1`, em vez das duas **meias-retas**
`x ≤ piso` e `x ≥ piso+1`. Parece equivalente — e não é: pontos descartam em silêncio todo inteiro fora
deles. A busca continuava a esgotar a fila e a declarar-se **provada**, sobre um ótimo que não era o
ótimo. O mesmo layout deu **29,86**, **29,92** e — depois da cura — **29,81**, os três com `prova = sim`.

⚠️ **E o gate sobre o RESULTADO não apanhava.** Força bruta num tetraedro, num octaedro, num layout com
junção em T e num prisma pequeno: a mutação sobreviveu a **todos**. Instâncias pequenas raramente
precisam que uma variável se afaste dois passos do valor fracionário, então o atalho acerta por acaso.
O que matou a mutação foi extrair a ramificação para uma função de três inteiros e gatear a
**propriedade que a define**: *os dois ramos particionam a faixa* — disjuntos e cobrindo tudo. Nove
linhas, sem malha, sem solver, e vermelho imediato.

**How to apply:** (1) trate discordância entre duas execuções "provadas" como **bug de completude**, não
de precisão; (2) quando um gate de resultado não derruba a mutação, **suba um nível**: qual é a
invariante de que o resultado depende? Gate-a sozinha, com aritmética pura; (3) desconfie de qualquer
busca que enumere **candidatos** onde a matemática pede **regiões**. Irmã de
[[feedback_a_mutation_that_survives_may_mean_a_missing_gate]] e de
[[reference_topic_gate_discipline]] (verde por acidente). O caso de um número de qualidade sem limite
certificado ao lado é [[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]] pelo outro lado:
aqui havia limite, e foi ele que denunciou o furo.
