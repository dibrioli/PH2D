---
name: feedback-a-gate-red-on-your-correct-code-may-predate-you
description: "Gate novo nasce VERMELHO no seu código correto? Rode-o contra o kernel SHIPADO antes de assumir que a culpa é sua — e antes de 'consertar' o que não quebrou"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: b294ecd6-99c8-41cf-ac4b-c6001c30b1c7
---

Escrevi um gate novo (*"tocar um knob no meio do traço não move a figura"*) e ele nasceu **VERMELHO no
meu próprio código correto**: 2520 bytes de desacordo. O reflexo é: *eu quebrei alguma coisa*.

Rodei o gate contra o kernel **SHIPADO** (`git show HEAD:<path> > /tmp/orig.rs`, `cp` por cima, rodar,
`cp` o backup de volta — nunca `git checkout`, [[feedback_mutation_undo_with_cp_never_git_checkout]]). O
gate ficou vermelho **lá também**: **6 bytes**. Ou seja:

* o defeito é **PRÉ-EXISTENTE** — não é meu;
* mas o meu fix o **AMPLIFICAVA 400×** — e isso É meu.

Essa separação mudou a ação inteira. Sem ela eu teria (a) caçado um bug meu que não existia, ou (b)
declarado o gate "estrito demais" e afrouxado até passar — enterrando o amplificador junto. Com ela:
consertei o amplificador (o composite passou a ler o plano CONGELADO, 2520 → **6**, exatamente o nível
herdado), **nomeei** o resíduo com números (2 texels a 4/255, e a causa: a advecção não sabe DES-pintar),
capei-o em consts explícitas (`GHOST_TEXELS`/`GHOST_DEPTH` = *o orçamento do defeito herdado, nunca
licença para aumentá-lo*) e mandei o fecho de verdade pro handoff, porque custa uma decisão de PERF
(escrever a janela inteira por pointer-move) que não se toma de carona num fix de borda.

**Why:** "meu gate está vermelho" e "eu causei isto" são afirmações diferentes, e a segunda não se deduz
da primeira. A medição contra o shipado é barata (minutos) e decide entre três ações opostas: consertar,
afrouxar, ou herdar-com-orçamento. É o mesmo raciocínio de [[project_painter_t19_latent_red_macos_2026_05_28]]
(*builde o commit claimed-green ANTES*), aplicado ao seu próprio diff.

**How to apply:** gate novo vermelho no código que você acredita correto ⇒ **rode-o contra o `HEAD`
shipado antes de tocar em qualquer coisa**. Verde lá = é seu, conserte. Vermelho lá = é herdado: sua
obrigação é **não amplificar** (meça os dois números e prove que empatou), nomear o resíduo com causa +
magnitude, e passar o fecho adiante. Um gate capado num defeito herdado só é honesto se o cap tiver nome,
número e a frase *"orçamento, não licença"*.
