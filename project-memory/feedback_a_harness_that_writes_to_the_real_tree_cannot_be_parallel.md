---
name: a-harness-that-writes-to-the-real-tree-cannot-be-parallel
description: Um arnês de mutação que escreve na árvore de verdade paga TRÊS preços de uma vez — o relógio, a árvore bloqueada e o mutante medido por engano; a cura é uma pasta por mutação
metadata:
  type: feedback
---

Um arnês de prova de mutação que **muta o ficheiro real e o repõe no fim** não paga só o relógio.
Medido no testbed do Cascadeur (2026-09-19), depois de o dono perguntar *«por que qualquer mínima
mudança no app precisamos de muito tempo de espera? Isso não pode acontecer»*:

| | antes | depois |
|---|---|---|
| 95 mutações | **22 min**, em série | **138 s**, 8 ao mesmo tempo |
| 13 testes de rato com mutações | ~15–20 min, em série | **246 s**, 4 ao mesmo tempo |

Os três preços, e só um é o relógio:

1. **o relógio** — nada mais pode correr ao lado, logo tudo é série numa máquina de 32 núcleos;
2. **a árvore bloqueada** — matá-lo a meio deixa uma mutação no ficheiro (aconteceu), ninguém pode
   editar nada enquanto ele corre, e uma corrida da suíte ao lado mede o **MUTANTE** e lê-se como um
   defeito do produto (também aconteceu: «2 FALHARAM» sobre produto correcto);
3. **um portão só para isso** — «a árvore voltou ao lugar», que existe apenas porque ela foi tocada.

**Why:** a cura é uma **pasta por mutação** (symlink de tudo o que não muda, cópia do que muda) — e
ela mata os três de uma vez, porque a árvore real deixa de ser tocada. Os testes de gesto do mesmo
repo já o faziam para mutar o `app.js`; era o arnês de motor que não.

⛔⛔ **E a cura tem uma armadilha que se lê como a lei ser fraca:** o `node` resolve o `__dirname` do
módulo PRINCIPAL pelo caminho **real**, logo um runner que seja um **symlink** carrega os ficheiros da
pasta verdadeira — as mutações não chegam à suíte e o relatório escreve *«a linha mutada não é
observável»*, que é a frase de uma lei sem portão. *Um arnês que mede a árvore errada acusa a lei.*
⇒ o runner é **copiado**, e há um **CANÁRIO**: uma mutação que parte o motor de propósito, corrida
antes de todas; se a suíte sobreviver a ela, a corrida pára e diz que nenhum veredito vale.

**How to apply:** antes de aceitar que um arnês é lento, pergunte **o que o impede de correr N de
cada vez** — quase sempre é um ficheiro partilhado, e tirá-lo de lá cura mais do que o relógio. E ao
paralelizar, ponha um canário que prove que o trabalho chega ao sítio: [[a-mutation-that-does-not-parse-reads-as-a-weak-law]]
é a mesma família — *o silêncio de um instrumento e o silêncio de uma lei sem portão são o mesmo byte*.
