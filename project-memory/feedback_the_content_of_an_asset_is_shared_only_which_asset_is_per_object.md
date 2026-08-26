---
name: feedback-the-content-of-an-asset-is-shared-only-which-asset-is-per-object
description: Num modelo de instância/override, o CONTEÚDO de um asset (os pixels) é partilhado; o que é per-objeto é QUAL asset ele usa.
metadata:
  type: feedback
---

Num sistema de instâncias (prefab/componente), a pergunta *«esta edição é uma excepção
desta cópia ou uma mudança da receita?»* **não se responde por igual para tudo**. A
fronteira medida é entre **os pixels e os botões**:

- **os botões** (tint, pose, máscara, qualquer knob) são propriedades ⇒ editar numa
  cópia é um **override**;
- **o conteúdo de um asset** (a imagem que a sprite mostra) é **partilhado** ⇒ pintar
  numa cópia tem de subir até à receita e chegar a todas. O que é per-objeto é *qual*
  imagem ele usa, não o que está dentro dela.

**Why:** PH2D, 2026-08-26 — o Enio pintou uma cópia e as irmãs não mudaram. O modelo
estava a funcionar (o sync capturou um override de `Sprite` + `SpritePixels`) e o
resultado era errado. Dois factos decidiram: (a) em todo motor 2D pintar a textura muda
quem a usa; (b) a **receita está escondida** de propósito, logo pintá-la não era
alcançável por gesto nenhum — os pixels de um componente eram a única coisa do app sem
forma de ser editada.

**How to apply:** antes de classificar uma edição como override, pergunte se o que
mudou é um **valor do objeto** ou o **conteúdo de uma coisa que vários objetos
partilham**. Se for a segunda, a edição sobe pelo funil que já existe (nunca num sítio
de chamada), **não vira override** — e por construção: ao escrever na receita, o passe
seguinte lê *«o mestre mexeu-se»*, a cópia editada já tem os bytes (`want == have`) e o
ponto fixo do sync fica intacto. Nomeie a fronteira com um gesto: *Detach* primeiro,
para editar uma cópia sozinha. Veja [[feedback-decide-dont-ask-gold-standard]] e
[[feedback-measure-the-defects-structure-before-designing-its-cure]].
