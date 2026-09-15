---
name: feedback-an-event-read-as-state-is-right-for-as-long-as-nobody-clears-it
description: Uma UI que pinta um FACTO lendo um canal de EVENTO acerta por acidente — e a leitura passa a depender de quem limpa o canal, não do mundo.
metadata:
  type: feedback
---

O Inspector pintava *«The flight is over»* a partir do canal `projectile_done()`, que anuncia
**quem morreu NESTE dispatch**. Resultado medido (15/09): com o relógio **parado** a etiqueta ficava
de pé (nada limpava o canal) e com o relógio **a andar** sumia no quadro seguinte — a mesma bala,
duas respostas, nenhuma sobre o mundo.

**Why:** um evento e um facto leem-se igual num `Vec<Entity>`, e a diferença só aparece no dia em
que alguém muda *quando* o canal é limpo — ou seja, numa wave que nada tem a ver com a UI. A
etiqueta certa é derivada do estado (`ProjectileState::finished`), que é verdade enquanto for
verdade.

**How to apply:** ao ligar uma superfície a um canal, pergunte *isto é um ACONTECIMENTO ou um
FACTO?* Um acontecimento serve quem age uma vez (o dreno que apaga a entidade); um facto serve quem
PINTA. Se a mesma lista tem de servir os dois, são duas portas com nomes diferentes
(`projectile_done` / `projectiles_finished`), nunca uma. Irmão de
[[reference_topic_control_design_hazards]].
