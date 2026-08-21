---
name: feedback-mutation-undo-with-cp-never-git-checkout
description: Desfazer uma mutação de gate com `git checkout -- <arquivo>` apaga o trabalho não-commitado DAQUELE arquivo — use backup por `cp`
metadata:
  type: feedback
---

Ao provar um gate vermelho por mutação, **desfaça a mutação com `cp` de um backup**, nunca com
`git checkout -- <arquivo>` / `git restore`.

```bash
SP=<scratchpad>
cp crates/.../alvo.rs $SP/alvo.bak      # ANTES de mutar
# … muta, roda o gate, confirma o VERMELHO …
cp $SP/alvo.bak crates/.../alvo.rs      # desfaz
```

**Why:** o arquivo que você acabou de mutar quase sempre é o mesmo que carrega a implementação
**não-commitada** que o gate está testando. `git checkout` o devolve ao **HEAD** — apagando a mutação
*e a feature*. O sintoma é traiçoeiro: o gate volta a passar (porque a feature sumiu junto), então
você lê "restaurado, verde" e segue. Aconteceu **3× na linha do Painter** (2× no Push, 1× no filme do
Impasto); na terceira, o `git checkout` do `stamp.rs` também fez uma mutação parecer não-derrubável,
porque o teste rodou numa árvore que já não tinha o código.

**⚠️ E o nome do BACKUP também precisa ser único** (Sculpt W1, 2026-07-13). Um script de mutação guardava
os alvos por `basename` — e a linha tinha **dois** arquivos chamados `sculpt.rs` (um em
`ph2d-painter-brush/src/`, outro em `ph2d-tool-painter/src/tool/paint/`). O segundo `cp` sobrescreveu o
backup do primeiro, o `restore()` escreveu o conteúdo de um **por cima do outro**, e as 11 mutações
voltaram todas como *"não compila"* — um resultado que parece problema das mutações e é problema do
backup. Guarde por caminho, ou prefixe pelo crate (`tool_sculpt.rs` / `brush_sculpt.rs`).

**⚠️ E uma mutação em BACKGROUND que é morta deixa a árvore MUTADA** (Sculpt/quantize, 2026-08-20). O
ciclo `cp backup → muta → testa → cp restaura` só é atômico se ele **terminar**: um `pkill` do cargo (ou
o timeout da ferramenta) mata o script entre o "muta" e o "restaura". O estrago só aparece um passo
depois — o `cp` de backup da *próxima* mutação captura o arquivo **já mutado**, e a partir daí
"restaurar" repõe a mutação. Nesta sessão o sintoma foi um gate que passava sobre código mutado e um
`assert old in s` que falhava sem explicação.
⇒ **Rode a mutação em PRIMEIRO PLANO** (ela dura segundos) e, se ela for parar em background,
**confirme o estado ANTES de tirar o próximo backup**, não depois. Um `grep -c <linha-original>` que
devolve `0` é a assinatura exata.

**How to apply:** antes de qualquer mutação, `cp` o alvo pro scratchpad, **com nome único**. Depois de
restaurar, **confirme o estado** (`grep -c <símbolo-novo> <arquivo>` ou `git status --short`) — árvore
limpa num arquivo que devia estar modificado é a assinatura do estrago; e `head -1` num arquivo que
começa com o doc-comment do *outro* módulo é a assinatura desta variante. Vale para toda a família
destrutiva do git ([[feedback_destructive_git_outside_pasta]],
[[feedback_destructive_reset_collision_2026_05_28]]), mas esta variante é auto-infligida: o path é *seu*,
e é justamente por isso que ninguém te avisa. Irmã de [[feedback_pipe_masks_script_exit_code]] — nos dois
casos o comando "deu certo" e o ESTADO é que estava errado. E quando uma mutação sobrevive de verdade,
[[feedback_a_mutation_that_survives_may_mean_a_missing_gate]].
