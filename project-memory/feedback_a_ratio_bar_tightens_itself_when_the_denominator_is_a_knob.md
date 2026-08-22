---
name: feedback-a-ratio-bar-tightens-itself-when-the-denominator-is-a-knob
description: A barra dizia «nada atravessa a peça» e media `máx / alvo` — subir a resolução reprovava saída CORRETA sem que nada tivesse piorado
metadata:
  type: feedback
---

Uma barra escrita como **razão** a um número que o utilizador controla **aperta-se
sozinha** quando ele mexe nesse número. ⇒ Antes de aceitar uma barra relativa,
pergunte **de que grandeza a asserção fala** — e se ela for absoluta, meça-a
absoluta.

**Why:** medido em 2026-08-21 (quad remesher). O gate dizia, com todas as letras,
*"alguma coisa na malha atravessa a peça"* — e comparava `aresta_máxima / alvo`,
onde o **alvo é o slider do artista**. Ao subir a resolução, na **mesma malha e sem
defeito nenhum**:

| alvo | quads | `máx / alvo` | **`máx / diagonal`** |
|---|---|---|---|
| grosso | 1 336 | 2,71× | **7,2 %** |
| médio | 20 039 | 7,71× | **5,1 %** |
| fino | 38 315 | ⛔ 9,48× | **4,5 %** |

⭐ **A razão triplica e a fração MELHORA.** Nada piorou: encolheu o denominador. A
barra de `4,0×` teria bloqueado um aumento de resolução de **15×** que a medição
mostra ser melhor em todas as outras colunas.

⚠️ **E o defeito que a barra existe para apanhar é absoluto:** a catástrofe real de
uma semana antes media `2,01` numa peça de diagonal `3,46` — **58 %**. Contra os
4,5 a 7,2 % do caminho correcto, uma barra de **20 % da diagonal** tem onze vezes
de margem e **não se move com o slider**.

⛔ **Trocar a unidade de uma barra NÃO é afrouxá-la**, e a diferença é verificável:
a barra nova continua a reprovar o defeito histórico com margem maior. *Afrouxar é
subir o número mantendo a grandeza; corrigir é medir a grandeza que a frase afirma.*

**How to apply:**

1. **Leia a mensagem do `assert` e pergunte que grandeza ela nomeia.** *"Atravessa a
   peça"* é fração da peça. *"Tem o passo pedido"* é razão ao alvo — essa fica
   relativa, e ficou.
2. ⚠️ **Sinal de alarme: a barra reprova quando você melhora outra coisa.** Se a
   mediana está em `1,03×`, as dobras em `0,03 %` e só a razão-máxima subiu, a
   suspeita nº 1 é a régua, não o produto.
3. **Prove a troca contra o defeito HISTÓRICO**, com o número dos dois lados. Sem
   isso, a mudança é indistinguível de conveniência.
4. ⭐ **Guarde as duas no relatório e imprima as duas no log.** Uma decide, a outra
   continua a ser legível; imprimir só uma deixa o leitor a comparar números que não
   são comparáveis entre duas corridas do slider.

Irmã de [[feedback_a_conserved_invariant_cannot_grade_quality]] e de
[[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]].
