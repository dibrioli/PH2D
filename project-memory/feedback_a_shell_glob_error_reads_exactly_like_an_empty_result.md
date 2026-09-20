---
name: a-shell-glob-error-reads-exactly-like-an-empty-result
description: Um erro de glob do shell («no matches found») lê-se como «não existe» — e foi assim que eu reconstruí e escrevi por cima de uma feature inteira já shipada
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 5717f360-1d4d-4e37-a7d1-f0af91986b16
  modified: 2026-09-20T01:20:46.413Z
---

⛔⛔⛔ **Um ERRO do shell lê-se exactamente como um RESULTADO VAZIO, e o preço é reconstruir o que
já existe.**

Medido em 2026-09-19, `line/components`. Ordem do dono: *«siga»*. Eu li o TOP-20, concluí que o
**#19 `SequencePlayer`** era o último item aberto, e a PRIMEIRA medição que corri foi:

```
grep -rn "SequencePlayer" crates/ shells/ --include=*.rs
→ (eval):1: no matches found: --include=*.rs
```

O shell desta máquina é **fish**: ele expande `*.rs` antes do `grep`, não encontra ficheiro nenhum
com esse nome no directório actual, e **aborta o comando inteiro**. Nada foi procurado. Eu li a
linha como *«zero ocorrências»* e escrevi, por baixo dela, **«`SequencePlayer` não existe em
código»**.

**O que se seguiu:** uma sonda do §5.0 inteira, um plano de wave, uma lei nova de 227 linhas com 12
gates — e um `Write` que disse **«updated»** (não «created») sobre
`crates/ph2d-ecs/src/sequence.rs`, **apagando 140 linhas de desenho medido**. Quem me apanhou foi o
compilador: `SequencePlayer must be defined only once in the type namespace`.

⭐ **E o que estava lá era MELHOR do que o que eu escrevi**, o que torna o erro caro duas vezes:

- ele **não tem relógio**, por medição de 2026-09-17: o relógio de corrida já existe inteiro no
  `Timer` (duração · repetir · `autostart` · sinal por disparo · `progress()` derivado), logo um
  `SequenceRuntime` seria um **segundo relógio** e um verbo `PlaySequence` **uma segunda maneira de
  dizer «começa»** — exactamente as duas coisas que eu ia construir;
- a sequência chama-se por **NOME** e nunca por índice (apagar um container renumera os de baixo ⇒
  um índice guardado tocaria a cutscene do vizinho, em silêncio);
- e a wave estava **inteira**: lei + ponte (`fase_sequences.rs`) + inspector + cena de smoke.

## As três leis

1. ⛔ **Um comando que ERRA não é uma medição.** Antes de escrever *«X não existe»*, confira que o
   comando **correu**: `echo $?`, ou uma contagem (`| wc -l`), ou um **controlo positivo** (procurar
   algo que de certeza existe com o mesmo comando). *Um `0` e um `erro` imprimem os dois «nada».*
2. ⛔⛔ **Neste repo o shell é `fish`: cite todo glob** (`--include='*.rs'`) ou não use glob nenhum.
   A mesma linha corre em bash e aborta aqui — e aborta **em voz baixa**, no meio de saída
   legítima.
3. ⛔⛔⛔ **A sonda do §5.0 pergunta *«a composição já exprime isto?»* e NÃO pergunta *«isto já
   está construído?»***. São perguntas diferentes, e a segunda é mais barata e vem primeiro:
   `git log --oneline -S "<NomeDoTipo>"`, ou o `grep` com o controlo positivo. ⚠️ A minha sonda
   chegou a produzir uma tabela bonita que **argumentava a favor** de construir o segundo relógio
   que a casa tinha recusado por escrito — *uma medição pode ser internamente coerente e estar a
   medir um mundo que não existe*.

⭐ **O `Write` diz qual é:** ele responde **«created»** ou **«updated»**. *Um «updated» num ficheiro
que eu julgava novo é um aviso de que a premissa da wave está errada* — e ele apareceu duas vezes
(`sequence.rs` e `sequence_tests.rs`) antes de o compilador falar.

Ver também [[feedback-the-line-ends-at-the-handoff-never-ask-the-owner-an-integrators-question]] e
a família de [[reference-topic-measurement-discipline]].
