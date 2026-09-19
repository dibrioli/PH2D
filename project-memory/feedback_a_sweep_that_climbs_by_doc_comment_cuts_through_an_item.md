---
name: a-sweep-that-climbs-by-doc-comment-cuts-through-an-item
description: Mover um corpo de função subindo por `///` e não por `#[` deixa o doc e os atributos colados ao VIZINHO — e rouba-lhe os dele
metadata:
  type: feedback
---

Ao mover uma função para um ficheiro irmão (corte por tecto de LOC), a varredura que acha o
início do bloco **tem de subir por `///` E por `#[`**. Subir só pelo doc-comment pára no primeiro
atributo e corta **dentro** do item.

**Medido duas vezes na mesma sessão** (2026-09-19, `ph2d-eval-motion`), e a segunda foi pior:

1. `last_cooked_tick` — o `#[must_use]` entre o doc e a `fn` fez o corte levar o doc do VIZINHO.
   Falhou a compilar (`expected item after attributes`) ⇒ barato.
2. `scrub_to_scoped` — o `#[allow(clippy::too_many_arguments)]` ficou no ficheiro de origem
   **com o doc inteiro**, colados ao `boundary_streams`, que **perdeu o doc próprio e o
   `#[must_use]`**. Compilou. O clippy só acusou porque a função movida deixou de ter a isenção
   — *a metade que rouba o doc do vizinho não acusa nada.*

**Por que é a assinatura:** as duas metades aparecem em sítios diferentes. A que falha alto é o
`allow` órfão (clippy na função movida); a que fica **muda** é o vizinho a herdar prosa e
atributos de outra função. Quem curar só a primeira deixa a segunda no `main`.

**How to apply:** ao cortar, suba por `("///", "#[")`, e depois do corte varra os ficheiros dos
DOIS lados por doc/atributo seguido de linha em branco ou `}`. E compare o `git show <base>:<f>`
do vizinho — *um doc que descreve outra função é invisível a todo gate deste repo*.

Irmão de [[feedback-an-exemption-is-a-property-of-the-code-not-of-its-address]]: uma isenção de
clippy é propriedade do CÓDIGO e viaja com ele, exactamente como a do censo de texto.
