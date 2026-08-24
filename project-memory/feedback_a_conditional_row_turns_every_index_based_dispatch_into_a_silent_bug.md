---
name: feedback-a-conditional-row-turns-every-index-based-dispatch-into-a-silent-bug
description: Assim que uma fileira de botões deixa de ter tamanho fixo, casar o slot por NÚMERO faz o botão executar o verbo do vizinho — sem erro e sem teste vermelho
metadata:
  type: feedback
---

A fileira de ações do painel do modelador era **três, sempre as mesmas**, e quem drenava a intenção
casava `slot` por número (`0`, `1`, `ISOLATE_SLOT`). A W57 acrescentou dois verbos **condicionais**
(largar/ligar o desenho) — e a colisão passou a ser concreta: **sem vínculo o slot `3` é *ligar*;
com vínculo é *largar***.

**Why:** um índice só é uma identidade enquanto a lista for fixa. Publicar e despachar em sítios
diferentes são **duas cópias da lista**, e a que envelhece é a que o artista clica. Não há erro de
compilação e o tipo (`usize`) não sabe de nada.

**How to apply:** ao tornar qualquer fileira condicional, faça a lista ser **uma função** chamada
pelos dois lados, e resolva o slot em **chave** (`list.get(slot).copied()` → `match` por constante).
⚠️ E o gate tem de empurrar a intenção pelo **dreno de verdade**: um gate que só lê a lista publicada
deixa passar a mutação que devolve o despacho ao array fixo — foi medido, ela sobreviveu
([[feedback-a-rule-only-exists-if-it-is-on-the-path-of-who-executes-it]]).
