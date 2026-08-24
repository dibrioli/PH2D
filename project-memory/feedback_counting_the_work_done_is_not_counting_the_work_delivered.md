---
name: feedback_counting_the_work_done_is_not_counting_the_work_delivered
description: "Um gate que conta o trabalho FEITO (cozeduras, chamadas, alocações) passa enquanto o consumidor recebe ZERO — conte o que o consumidor VIU, senão a máquina trabalha e ninguém lhe pega"
metadata:
  type: feedback
---

Construí o leque de tempo (ADR-0163) e um gate para ele:

```rust
assert_eq!(calls, 3, "o driver tinha de cozer nos TRES instantes do leque");
```

Verde. E o `motion.emitter` **ignorava 529 amostras da própria história em
silêncio**, porque `fan_len()` contava as fatias da **PORTA 0** — e um nó SEM
portas (que é toda FONTE) lia zero.

**Why:** o gate media o lado do PRODUTOR (as cozeduras aconteceram) e a
afirmação era sobre o CONSUMIDOR (o nó recebe as fatias). Entre os dois há uma
travessia, e é lá que o defeito mora. O sintoma foi o pior possível: a cena
desenhava, os três modos ficavam **idênticos**, e nada falhava.

**How to apply:**
1. Depois de `assert` sobre trabalho feito, acrescente **um `assert` sobre o
   trabalho RECEBIDO** — a sonda tem de estar do lado de dentro do consumidor
   (aqui: um nó de teste que guarda `ctx.fan_len()` e o gate compara).
   [[reference_topic_ui_seam_discipline]]
2. ⚠️ **Um caminho «sem X» a devolver vazio é uma decisão, não uma ausência.**
   O código empurrava uma entrada *só quando havia aresta*; a lei certa é **uma
   entrada por fatia, sempre**, com o conteúdo vazio quando não há porta. *O que
   falta é o conteúdo, não a fatia.*
3. O gate que afirmava a versão errada (*"um nó sem porta não ganha leque
   nenhum"*) **passava e defendia o bug**. Ao curar, reescreva-o — a premissa
   dele dissolveu. [[feedback_a_claim_no_mutation_can_kill_is_a_claim_about_nothing]]
