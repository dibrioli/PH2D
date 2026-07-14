---
name: feedback-a-lateral-effect-needs-a-nonlocal-operator
description: Se o efeito que o artista PEDE é lateral (engordar, espalhar, empurrar pra fora), nenhuma fórmula por-texel produz isso — e o sintoma é que ela vira uma CONSTANTE no dado real
metadata:
  type: feedback
---

O `Inflate` do Sculpt shipou como `h = pre + Depth·n_z` ("sobe pela normal"). O Enio olhou por 1 traço:
*"parece fazer a mesma coisa de Layer"*. **Fazia — ao bit.**

Dois erros, e só o segundo é interessante:

1. **A normal ia invertida.** O offset verdadeiro de um campo de altura sobe pela **secante**
   (`Depth·S`, `S = √(1+|∇h|²) = 1/n_z`), não pelo cosseno. Íngreme move **MAIS** — é assim que uma parede
   anda de lado e a forma engorda. `·n_z` movia menos, o que *arredonda a crista*: era um Smooth pior.

2. **Consertar o sinal NÃO consertaria a ferramenta.** `h + d·S` é **UM passo de Euler** da PDE de offset
   (`∂h/∂t = √(1+|∇h|²)`), e um passo de PDE hiperbólica **não move matéria de lado**. "De lado" é a palavra
   inteira de *inflar*. O operador certo é **não-local**: dilatação/erosão morfológica por uma **BOLA**.

**Why:** o sintoma que denuncia isso é específico e mensurável — **a fórmula colapsa numa constante sobre o
dado que o produto de fato produz**. Medi `n_z` sobre o relevo do depósito real: **`p50 = 1.000`** (o miolo
de um traço é chapado; o settle borra o que resta). Então `Depth·n_z`, `Depth/n_z` e `Depth` são o MESMO
número em todo texel que o artista olha.

**How to apply:**
- Se o nome do efeito é **espacial** (engordar / espalhar / empurrar / fechar vinco / comer), pergunte
  *"que texel LONGE daqui esta conta lê?"*. Se a resposta é "nenhum", a conta não pode fazer o que promete.
- **Meça a entrada real antes de acreditar num gate sobre ela.** Uma sonda de 20 linhas sobre o caminho do
  produto (não sobre um fixture sintético) responde em 1 minuto. Ver
  [[feedback_a_gate_only_proves_what_its_fixture_contains]].
- Cuidado com o parente: **o gate estava VERDE e mutation-proven** — chamava-se *"inflate arredonda a
  crista"*, e arredondar a crista **É** o bug. Um gate prova que o código faz o que você **disse**; nada
  nele avisa que o que você disse está errado. Só o smoke pega isso.
- Corolário do preço: o operador não-local custa `O(ρ²)`. Vale medir e otimizar (layout contíguo + quebrar a
  cadeia serial do `max`: 15,9 → 5,7 ms/move), não capar a faixa artística.
