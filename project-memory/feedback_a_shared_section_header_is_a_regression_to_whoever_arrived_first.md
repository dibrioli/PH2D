---
name: a-shared-section-header-is-a-regression-to-whoever-arrived-first
description: "Pendurar feature nova na seção de outra é regressão para quem chegou primeiro — o ADR-0166 diz o que MOSTRAR, nunca ONDE"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 50fdf69a-7948-4dc9-ba54-57fb458b4011
  modified: 2026-08-26T01:06:13.271Z
---

Enio, 2026-08-25 (Vector, máquina de estados do Morph): *"vc contaminou ou até mesmo
estragou a feature states previamente implementada? Os states de morph deveriam ter
sessão exclusiva. Restaure ao original o painel e funcionamento da seção states."*

A wave tinha posto as transições do Morph **dentro** da seção `States` (poses de UI +
Smart Animate), com dois argumentos verdadeiros escritos no código: o ADR-0166 (*o
Inspector mostra o que o objecto **TEM***) e *"um objecto raramente é as duas coisas"*.

**Why:** os dois argumentos eram verdadeiros e **nenhum deles era a pergunta**. *A lei diz
o que MOSTRAR, nunca ONDE.* O efeito medível é que o cabeçalho de uma feature **já
entregue e smokada** passou a aparecer por causa de outra — e quem chegou primeiro é quem
paga a regressão, sem ter mexido numa linha.

⛔ E a causa de fundo repetiu a da auditoria do Input Map: **nenhum dos 12 gates da wave
olhava para o que era PINTADO** (todos mediam o mapa e o estado publicado), então doze
verdes conviveram com um cabeçalho alheio na tela.

**How to apply:**
- Feature nova ganha **seção própria**. Partilhar cabeçalho só com o dono da seção
  existente a pedir — e nunca por economia de espaço ou por uma lei de conteúdo.
- Ao encostar em superfície de feature entregue, escreva o gate que mede a **ausência nos
  DOIS sentidos** (A não faz B aparecer, B não faz A aparecer) e **contra o que foi
  pintado** (`MockPanelHost::painted_rect`), não contra o estado publicado.
- A restauração é literal: `git checkout main -- <ficheiro>`, e o `git diff main` dele tem
  de ficar **vazio** — dizer "restaurei" sem esse diff é uma afirmação por verificar.
- Prove a cura mutando de volta a **forma exacta** da contaminação; se ela não compilar,
  a mutação está errada, não a cura.

Ver [[feedback-counting-the-work-done-is-not-counting-the-work-delivered]] ·
[[feedback-a-gate-verde-pode-pinar-um-defeito-de-produto]] ·
[[reference-topic-ui-seam-discipline]]
