---
name: a-new-view-cannot-replace-the-one-consumers-read
description: Uma representação nova entra ao LADO da que os consumidores já lêem, nunca no lugar dela
metadata:
  type: feedback
---

Ao dar **arcos** ao `ph2d_field::Profile` (2026-09-16), a 1.ª tentativa pôs os *bulges* **paralelos**
à polilinha: a mesma lista de pontos, com uma curvatura por aresta. Com isso a polilinha deixou de
descrever a figura — ela passou a ser a lista de **cordas**.

**Dois gates que já existiam apanharam-no na primeira corrida**, e nenhum deles era sobre arcos:
a tolerância do achatamento passou a medir a corda, e um furo mediu `0,2828` em vez de `0,4`.
*Vinte e quatro leitores tratavam `contours()` como a figura, e estavam certos.*

**Why:** uma representação nova é sempre mais informativa que a antiga, e isso torna tentador
substituí-la. Mas os consumidores não foram contados: cada um deles leu a antiga com um contrato
implícito (*«isto É a forma»*) que a nova quebra em silêncio para quem não for reescrito.

**How to apply:** a vista nova entra **ao lado**, com a antiga intocada e byte-idêntica, e um gate a
atar as duas (*descrevem a mesma curva a menos da tolerância*). Só quem paga o preço que a nova
resolve — aqui a fita da marcha, que custa por primitiva — muda de leitor. ⭐ E o custo é pequeno:
memória, não correcção.

⚠️ **E a porta recebe as duas JUNTAS** (`with_arcs(Vec<(polilinha, decomposição)>)`) — é isso, e não
disciplina, que as impede de descrever figuras diferentes. Um construtor por vista seria o vector
paralelo com outro nome.

Ver [[a-per-command-ceiling-does-not-compose-only-a-slice-does]] (a mesma forma noutro domínio: a
resposta certa é *onde* a coisa vive, não *o que* ela guarda).
