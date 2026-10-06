---
name: feedback-a-brake-applied-outside-a-warm-started-solver-makes-the-pile-shake
description: "Travão aplicado por FORA de um solver com aquecimento (cortar ω depois do passo, binário, amortecimento) faz a pilha tremer/rastejar; só uma restrição DENTRO do solver segura o repouso"
metadata:
  type: feedback
---

Medido em 06/10 (doc 121 §9.20, o `Rolling` sobre o `rapier2d`, que não tem resistência ao rolamento): seis leis.
Cortar o giro depois do passo, um binário no passo seguinte (ganho 1 e ½), o amortecimento angular do rapier, o
travão relativo do Box2D v3 e o contacto «achatado» pelo gancho `modify_solver_contacts` — TODAS deixaram a pilha
mais trémula com o botão do que sem ele (caixas: tremor `0,048` contra `0,004` °/tique), ou a rastejar, ou aceleraram a
bola (os pontos do gancho ficam presos ao corpo e giram com ele). Só a lei em DUAS FASES ficou limpa: parada ⇒ a
rotação TRANCADA no solver (`lock_rotations`) até o binário que os contactos pedem passar a capacidade
`Σ μr·λn·|braço|`; a rolar ⇒ um binário constante da capacidade.

**Why:** o solver aquece cada contacto com o impulso da vez anterior; um ω mexido por fora deixa esse impulso a
corrigir uma velocidade que já não existe, e um binário constante dentro de um passo em que o atrito vira o giro
passa do zero (bang-bang).

**How to apply:** numa lei que falta ao motor, procure a forma de a pôr DENTRO do solver (um eixo trancado, uma
restrição) antes de a aplicar por fora; e meça-a numa pilha PARADA (onde o tremor aparece), não só numa bola a rolar.
Família: [[reference-topic-code-pattern-gotchas]].
