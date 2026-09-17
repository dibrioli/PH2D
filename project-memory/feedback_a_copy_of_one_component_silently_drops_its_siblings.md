---
name: feedback-a-copy-of-one-component-silently-drops-its-siblings
description: Copiar UM componente de uma entidade deixa para trás os irmãos que descrevem a mesma coisa — cinco consumidores desenhavam/apontavam a pose ERRADA
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 288048cc-7a6f-40b2-8ea6-37cc673393d2
  modified: 2026-09-13T23:43:07.569Z
---

Medido em 2026-09-13 (W3 do plano `docs/Skeleton/03`): quando uma sprite passou a poder ser desenhada
como MALHA (um `SpriteMesh` **ao lado** da `RenderInstance`, na mesma entidade de presente), **cinco**
consumidores ficaram errados de uma vez — e nenhum deixou de compilar:

- os que **COPIAM** a instância para a redesenhar isolada: o vidro do prefab, o emissivo, os
  fantasmas do onion ⇒ desenhavam o quad de REPOUSO;
- os que **LÊEM o quad** para responder onde a coisa está: o picking, as caixas do gizmo/laço e o
  *View All*.

**Why:** a cópia de um `Copy` compila sempre e parece completa. O que a torna incompleta é uma
capacidade NOVA que passou a viver noutro componente da mesma entidade — e o sintoma é visual
(a arte deformada na cena, em repouso no halo), nunca um erro.

**How to apply:** ao acrescentar um componente que MUDA o que uma entidade desenha, faça o censo de
quem copia o componente antigo (`git grep 'query::<.*&RenderInstance'` e afins) ANTES de fechar a
wave; e dê a cada família uma PORTA que leve os dois juntos (aqui: `LiftedInstances::collect_from`),
para que um consumidor novo não possa esquecer o irmão. A pergunta *«isto desenha-se como malha?»*
mora numa função só, e o desenho e o picking chamam-na — senão o cursor apanha onde nada é desenhado.
