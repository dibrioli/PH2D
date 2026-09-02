---
name: a-threshold-discriminator-can-exhaust-itself
description: "Quando as duas classes de um limiar se tocam, afinar o número só muda de que lado cai o precipício — a pergunta tem de mudar de GRANDEZA"
metadata:
  type: feedback
---

**Medido na `line/motion-value`, 2026-08-30.**

O `source.lsystem` decidia *«esta gramática refina ou cresce pela ponta?»* por um limiar sobre a
razão de expansão medida (`>= 1,25`). Ao mudar a régua, o modo **GUIADO — o default do nó** —
caiu para `1,2502`: **`0,017 %`** acima do limiar. Um passo do `Length Scale` (`0,89 → 0,90`, e
`0,90` é o default do painel) atravessava-o e saltava o tamanho **`+15,4 %`**.

⚠️ **E não havia limiar melhor.** Varridas **8 100** combinações dos knobs, o guiado chega a
`1,4294` e o refinador mais fraco do corpus (Dragon) está em `1,4791` — **`3,5 %`** de
separação. *Duas classes que se tocam não se separam por um número, e uma banda de mistura
também não cabe.*

**A cura foi mudar a GRANDEZA da pergunta.** A resposta exacta já existia no mesmo ficheiro,
escrita de outra maneira, para a outra metade da lei: quem cresce pela ponta **guarda** os
módulos que desenham das gerações antigas (o `F` é terminal); quem refina reescreve tudo. Zero
limiar, zero fronteira, e as duas leis que respondiam à mesma pergunta de maneiras diferentes
passaram a ter **uma porta**.

**Why:** um limiar é uma afirmação sobre uma FOLGA. Quando a folga desaparece, afinar o número
não é conservador — é escolher em silêncio qual utilizador cai no precipício.

**How to apply:** ao mexer numa medição que alimenta um limiar, **meça a folga dos dois lados**
antes de dar por bom. Se ela for menor que ~10 %, procure a resposta ESTRUTURAL — e procure-a
primeiro no código que já responde à mesma pergunta noutro sítio.
Relacionado: [[feedback_a_declared_fence_chooses_the_shape_of_its_own_cure]] ·
[[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]].
