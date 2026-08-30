---
name: feedback_a_smoke_scene_that_teaches_the_opposite_is_worse_than_no_scene
description: Quando um comportamento muda, o smoke que o demonstra é o último sítio a ser lembrado e o primeiro que o dono do produto lê — e uma cena que ensina o contrário não é acreditada só depois de o ter enganado
metadata:
  type: feedback
---

⛔⛔ **Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente** — a
ausente não é acreditada; a errada é.

**Medido (2026-08-30).** A cena `PH2D_PHYSICS_SMOKE=15` imprimia *«a bola laranja atravessa a
parede limpa e sai do ecrã»*, para demonstrar o que a bandeira de colisão contínua compra. Sonda que
reproduz a cena exacta (160 m/s, r=0,15, parede 0,05×1,0): **as duas bolas param em `x = −0,2000`,
idêntico**, com e sem gravidade. A `rapier2d` 0.35 passou a varrer todo corpo rápido contra
colliders **fixos de graça**, e a parede da cena era estática.

⭐⭐ **O doc da biblioteca já estava corrigido** — com dois gates a prová-lo — e a **CENA** é que não
foi. *Quando um comportamento muda, o smoke que o demonstra é o **último** sítio a ser lembrado e o
**primeiro** que o dono do produto lê.*

A cura não foi reescrever a frase: foi fazer a cena demonstrar o que a bandeira compra **hoje** — a
parede passou a ser **cinemática**, que é a classe de alvo que a varredura por omissão não alcança,
exactamente o que o gate irmão já media.

**Why:** o §0.8 do `CLAUDE.md` diz que o dono do produto **não conhece as ferramentas** — o smoke é
onde ele as **aprende**. Uma cena errada não gasta só um teste: ensina-lhe uma regra falsa que ele
vai usar para julgar tudo o resto.

**How to apply:** ao mudar um comportamento que uma cena de smoke demonstra, **corra a cena** e leia
a mensagem dela contra o que acontece. Se a mensagem promete um contraste, **meça os dois lados** —
e se o contraste desapareceu, mude a **cena**, não a frase. Ver
[[feedback_a_dead_knob_has_two_species_no_probe_catches]] e
[[feedback_ready_to_smoke_example]].
