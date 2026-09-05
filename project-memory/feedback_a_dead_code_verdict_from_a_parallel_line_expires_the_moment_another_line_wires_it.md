---
name: a-dead-code-verdict-from-a-parallel-line-expires-the-moment-another-line-wires-it
description: Uma linha declara um símbolo MORTO e retira-lhe a porta; outra, no mesmo dia, dá-lhe consumidor. As duas compilam, o merge é limpo, e o produto fica com uma feature viva e inalcançável
metadata:
  type: feedback
---

O pill `Assets` era, em 2026-08-30, um id pintado sem consumidor nenhum. A `line/UIUX` retirou os 29
pills a pedido do dono, realojou os verbos no menu *Window*, e pôs este id numa lista de excepções
com a razão **medida e verdadeira**: *«MORTO PRE-EXISTENTE … SEM consumidor nenhum no repo inteiro»*.

Horas depois, na worktree ao lado, a `line/components` fez desse pill a **única porta** do navegador
de assets que estava a construir.

As duas linhas compilavam. O merge não teve um conflito. E o app shipou um painel **vivo, registado,
despachado e inalcançável** — até o dono reportar *«vc não colocou nenhum meio de abrir a janela»*.

**Why:** uma classificação de «morto» é uma afirmação sobre **a árvore de quem varreu**, e num
repo com linhas paralelas essa árvore não é a árvore que shipa. É o §0.0 com um MOTIVO no lugar do
número: *quem torna alcançável o que uma nota declarou morto tem de reconferir a nota* — e aqui quem
a escreveu não tinha como saber, porque o consumidor ainda não existia do lado dela.

⚠️ **E a catraca com censo de obsolescência não salvou.** Ela existia, estava certa, e só disparou
quando eu **já ia** acrescentar a porta — validou a cura em vez de acusar a janela. *Um censo de
obsolescência apanha o resíduo; ele não apanha o intervalo entre a classificação e a cura.*

**How to apply:**
1. Ao **retirar uma categoria inteira de porta** (uma barra, um menu, um trilho), o inventário do que
   realojar tem de incluir **as worktrees vivas**, não só a sua — `git worktree list` responde em
   segundos, e a lista de painéis/ids de outra linha é um `grep` nela.
2. Ao **dar consumidor** a um símbolo que estava morto, procure quem o declarou morto: uma lista de
   excepções, um `#[allow(dead_code)]`, uma nota de auditoria.
3. ⛔ Uma excepção justificada por **prosa** (*«sem consumidor»*) é uma medição congelada. Derivá-la é
   o certo; se derivar for um censo textual frágil, **escreva que ela é prosa** em vez de a deixar
   parecer verificada.

Relacionado: [[two-lines-can-refactor-the-same-code-differently-and-both-survive-the-merge]],
[[a-ratchet-without-a-staleness-census-only-ratchets-up]],
[[a-door-the-neighbour-does-not-call-is-not-a-door-yet]].
