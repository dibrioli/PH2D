---
name: a-constraint-imposed-in-one-phase-and-not-the-next-is-a-starting-point
description: Impor uma restrição num passe e não na fase seguinte dá saída byte-idêntica ao controlo — e lê-se como «a restrição não faz nada».
metadata:
  type: feedback
---

Uma restrição imposta numa fase e **não** na seguinte não é uma restrição — é um **ponto
de partida**. A fase seguinte relaxa e desfaz.

⛔ **O modo de falha é o caro:** a saída sai **byte-idêntica ao controlo** *com a
restrição a entrar em todos os grupos*. Isso lê-se como «a restrição não faz nada», que é
uma conclusão **falsa** sobre um mecanismo que de facto funcionou e foi apagado a jusante.

**Why:** medido em 2026-08-26 na `line/quadextract`: o 2.º passe do G3 impunha as amarras
dos arcos, e logo a seguir o `round_welded` construía o relaxador da escada gulosa **sem**
elas. Duas corridas de A/B foram gastas antes de o contador dizer que os grupos entravam.

**How to apply:** ao ligar uma restrição a um solver com **mais de uma fase** (contínuo →
arredondamento → endurecimento), ligue-a a **todas**, e conte quantos grupos entraram em
**cada** — nunca só na primeira. E antes de concluir «não fez nada», exija o contador que
separa *«não correu»* de *«correu e foi desfeita»*: sem ele os dois são o mesmo byte.
Irmão de [[a-new-half-can-make-the-old-half-unobservable]] e de
[[counting-the-work-done-is-not-counting-the-work-delivered]].
