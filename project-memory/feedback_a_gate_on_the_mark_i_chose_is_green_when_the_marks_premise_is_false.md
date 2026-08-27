---
name: feedback-a-gate-on-the-mark-i-chose-is-green-when-the-marks-premise-is-false
description: Um gate que afirma «pus a marca X» fica verde quando a premissa de que X produz o efeito é falsa — afirme o FIM, não o mecanismo que você escolheu.
metadata:
  type: feedback
---

Escrevi *«Criar componente esconde a receita»* assim:

```rust
assert!(world.get::<Visibility>(master).is_some_and(|v| v.hidden),
        "a receita ficou visivel — o artista ve' dois objetos empilhados");
```

Verde durante toda a fase. E a receita **continuava a desenhar** sempre que era um
grupo, porque neste motor `Visibility` é **per-entidade e não desce aos
descendentes** — facto escrito, pelo nome, no doc do próprio ficheiro que extrai as
sprites.

**Why:** o gate afirmava a **marca que eu escolhi** (`hidden = true` na raiz), não o
**fim** que a frase promete (*nada da receita aparece na tela*). Entre a marca e o fim
há uma premissa — *«esconder o pai esconde os filhos»* — que eu nunca medi. Um gate
sobre o meio fica verde sobre o defeito que ele existe para apanhar, e ainda **defende**
a premissa falsa: quem lê o gate acredita nela.

**How to apply:** ao gatear um efeito visível, pergunte *«que ENTIDADE tem de sair da
tela?»* e afirme sobre todas elas — a raiz **e** uma peça —, nunca sobre o interruptor
que você acabou de ligar. Se o mecanismo é herdado (visibilidade, trava, camada),
**meça a herança antes de a usar** em vez de a assumir; e prefira uma marca **derivada**
por um passe (aqui: `MasterPiece`, re-carimbada por quadro sobre a raiz e toda a
descendência) a uma marca autorada que você espera que se propague — a derivada não
pode discordar da árvore. Irmã de
[[feedback-counting-the-work-done-is-not-counting-the-work-delivered]] (ali o gate media
o produtor e a afirmação era sobre o consumidor) e de
[[feedback-a-claim-no-mutation-can-kill-is-a-claim-about-nothing]].
