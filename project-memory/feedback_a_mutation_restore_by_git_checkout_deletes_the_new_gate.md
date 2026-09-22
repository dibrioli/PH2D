---
name: a-mutation-restore-by-git-checkout-deletes-the-new-gate
description: Desfazer uma mutação com `git checkout -- <ficheiro>` apaga o gate acabado de escrever e ainda não commitado — e a corrida seguinte fica VERDE por já não haver régua.
metadata:
  type: feedback
---

⛔⛔ **A restauração de uma mutação usa o `.bak` que o próprio arnês guardou, NUNCA o git.** Medido
em 2026-09-22 (`line/PainterWatercolor`, auditoria do watercolor): o arnês mutava um ficheiro de
teste que carregava um censo **acabado de escrever e ainda não commitado**; o `git checkout --` de
restauro devolveu-o ao `HEAD` e **levou o censo junto**.

**Why:** o modo de falha é o caro — a corrida seguinte fecha **VERDE**, e o verde não diz *«o produto
está certo»*, diz *«já não há gate nenhum a medi-lo»*. É a mesma família do
[[a-mutation-restore-by-mv-leaves-cargo-with-the-mutated-build]] (lá o mtime engana o cargo, aqui o
git engana o autor) e do [[python-replace-silent-noop-after-fmt]]: *um instrumento que falha em
silêncio lê-se exactamente como um produto limpo*.

**How to apply:** (1) o que é NOVO commita-se **antes** de se correr a primeira mutação; (2) o
restauro é `cp <bak> <ficheiro>` mais um `touch`; (3) depois de uma ronda de mutação, confirme que o
gate ainda existe (`grep -c <nome_do_teste>`) antes de acreditar no verde. Ver também
[[reference-topic-mutation-proofs]].
