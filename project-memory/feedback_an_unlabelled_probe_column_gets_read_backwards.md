---
name: feedback-an-unlabelled-probe-column-gets-read-backwards
description: Seis números lado a lado com rótulo em UM só — eu li "17 buracos" onde a linha dizia `0 bordo` e `17 dobradas`, e diagnostiquei duas fases erradas
metadata:
  type: feedback
---

Uma sonda que imprime N grandezas na mesma linha tem de pôr **um rótulo colado a
cada número**. Sem isso ela é lida ao contrário — e o custo não é a leitura: é o
**plano** que sai dela.

**Why:** medido em 2026-08-21 (quad remesher). A linha era

```text
GLOBAL 260 quads (100% quads) 19   irreg 0    bordo 17   dobradas (6.5 %)
```

Os rótulos vinham **depois** de cada valor, e os dois do meio ficaram alinhados de
forma a parecer `19 irreg` · `0` · `bordo 17`. Reportei ao dono do produto
**«a casca volta aberta, buracos de verdade»**, abri o plano com *"os buracos"* em
primeiro lugar, e fui diagnosticar arcos órfãos e patches não-disco. Medido depois:
o censo dava `{2: 50}` — **todo arco partilhado por exactamente dois patches** — e
`boundary_edges = 0` nas quatro densidades. *Nunca houve buraco nenhum.* O defeito
era a coluna vizinha, e a fase dele é outra.

⚠️ **A correcção seguinte quase foi outro erro pelo mesmo caminho:** ia escrever que
a régua radial *"mentiu"*. Medida contra a régua nova: `11·17·19·22` contra
`12·17·19·23`. *Ela estava certa* — o que falhou foi a minha leitura, e o commit
teria enterrado a culpa num instrumento inocente.

**How to apply:**

1. **Rótulo antes do valor, ou par `nome=valor`.** `bordo 17` e `17 dobradas` na
   mesma linha são ambíguos por construção; `bordo=0 dobradas=17` não é.
2. ⚠️ **Antes de reportar um número de uma sonda, faça-o dizer o nome dele.** Um
   `assert` ou um segundo `println` com uma grandeza só custa segundos e é o que
   separa *"a fase X está partida"* de *"eu li a coluna errada"*.
3. ⭐ **A sonda mais barata é a que responde UMA pergunta.** A que respondeu esta
   foi o censo `uso {2: 50}` — uma linha, um invariante (*um arco pertence a
   exactamente dois patches*), sem margem para leitura torta.
4. **Quando corrigir, corrija só o que a medição condena.** Culpar o instrumento é
   tentador porque encerra o assunto; verifique-o antes.

Irmã de [[feedback_a_suite_of_topological_assertions_is_blind_to_geometry]] e de
[[feedback_a_defect_count_without_provenance_names_the_wrong_phase]].
