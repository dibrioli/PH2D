---
name: feedback-what-the-undo-does-not-photograph-the-undo-does-not-restore
description: Separar CONFIG (registado) de VIVO (não registado) é o desenho certo e tem uma consequência que ninguém escreve — nada repõe o vivo ao rebobinar, e um relógio/uma semente/um contador sobrevivem ao Reset.
metadata:
  type: feedback
---

O molde do PH2D separa o componente **registado** (config, viaja no ficheiro, o undo fotografa) do
componente **vivo** (sem `Serialize` — a cerca é o TIPO). Isso impede que cada tique vire um passo
de `Ctrl+Z`. Mas em 15/09 mediu-se o outro lado: **o `ph2d-ecs` tinha `advance`/`reconcile`/`start`/
`stop` e NENHUMA porta de reposição** — ninguém *podia* repor o vivo ao rebobinar.

Resultado: um `Timer` corrido continuava corrido depois de um Reset; uma `Factory` com `Max Total`
gasto **recusava-se a produzir na 2.ª corrida**; e com o `rng` a continuar, uma fábrica **aleatória
dava outra corrida a cada rebobinar**.

**Why:** a ausência é estrutural e silenciosa — nenhum gate pergunta *«quem repõe isto?»*, e o
sintoma (*«comportamento diferente a cada rewind»*) chega por report do dono, não por suíte.

**How to apply:** ao criar um estado vivo não-registado, escreva no MESMO commit (a) o «nascer»
dele, (b) o chamador dentro do **invariante** do rebobinar (relógio no início e parado — nunca um
gancho num botão), e (c) o **censo** que exige que todo vivo passe pela porta, com piso de
população. ⚠️ O «nascer» **não é `Default`**: um `autostart` nasce a correr, e o vivo de uma câmera
nasce da pose AUTORADA (ali a cura é APAGAR o componente e deixar quem sabe recriá-lo).
Irmão de [[feedback_two_hand_written_loops_that_must_agree_need_one_door]].
