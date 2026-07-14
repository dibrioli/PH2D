---
name: feedback-ask-the-same-question-of-the-other-side
description: Instalou um gate/regra num lado de uma fronteira? Faça a MESMA pergunta ao outro lado — foi assim que 65,9 MB por música apareceram
metadata:
  type: feedback
---

O [ADR-0117] emendou o HR-13: *quem declara budget possui um gate que MEDE*. Escrevi os gates do
**Audio Editor**, todos verdes, e o item estava fechado.

Aí fiz a pergunta óbvia que quase não fiz: **e o outro lado?** O HR-13 fala do mixer em **runtime**
(a linha "Audio buffers", 30 MB no iPad) — o que embarca dentro de um jogo. Escrevi o mesmo gate lá.

Ele nasceu **vermelho**: uma faixa estéreo de 3 min custava **65,9 MB** residente, **2,2× o
orçamento inteiro de áudio do iPad**, antes de um único efeito sonoro. E o corolário: o seletor de
codec que eu tinha acabado de construir (ADR-0113/0116) **não economizava um byte de RAM** — Opus é
6,4% do WAV16 em disco e 100% dele na memória, porque o load expandia tudo de volta. Virou o
[ADR-0118] (vozes por streaming).

**Why:** quando você conserta uma classe de defeito, a classe **quase nunca mora num sítio só**. O
gate que você acabou de escrever é uma *pergunta* — e uma pergunta boa vale em mais de um lugar. O
perigo é o oposto do que parece: não é esquecer o gate, é o gate verde te dar a sensação de que o
assunto acabou. Um lado auditado e o outro intocado parece "feito" e é meio-feito.

## A mesma lição, de novo — a DIREÇÃO que ninguém exercitou (2026-07-13)

O `ph2d_audio_edit::conform` reamostra por interpolação linear. Estava certo havia um ano — porque
o **único** caller era o **paste**, que quase sempre **sobe** de taxa (44,1k → 48k), e subir não
dobra. O alvo **Mobile** do W6 (24 kHz) foi o **primeiro caller que desce**: sem filtro anti-alias,
tudo acima do Nyquist do destino **volta pra dentro da banda** — um shimmer de 15 kHz reaparecendo
como um tom de 9 kHz que nunca foi gravado. Ia shipar isso pra dentro de um jogo.

**Uma rotina correta para o caller pra quem foi escrita não é, por isso, correta.** O outro lado
aqui não era outro subsistema: era **a outra direção da mesma função**.

**How to apply:** ao fechar um gate/regra, liste **as outras instâncias da mesma fronteira** e faça
a mesma pergunta a cada uma, mesmo (sobretudo) quando você espera verde:
- editor ↔ runtime · control thread ↔ RT thread · save ↔ load · encode ↔ decode · CPU ↔ GPU
- **subir ↔ descer · crescer ↔ encolher · avançar ↔ retroceder** — toda função com uma *direção* tem
  um lado que os callers existentes nunca pisaram, e é lá que o bug espera
- o caminho que o TESTE percorre ↔ o caminho que o PRODUTO percorre ([[feedback_frozen_bar_check_the_arithmetic_before_gaming_it]])

Se o gate do outro lado **não existe**, escrevê-lo é barato (é o mesmo gate). Se ele nascer
vermelho, você achou um defeito que ninguém estava procurando — que é o melhor tipo.

Parente de [[feedback_a_rule_that_never_observes_cannot_fire]] (a regra tem de OLHAR) e de
[[feedback_audit_lens_diversity]].
