---
name: feedback_one_ruler_measures_one_clock
description: Dois dados na mesma régua/eixo com bases de tempo diferentes = a mesma coluna significa dois instantes; a queixa de UX era um bug de modelo
metadata:
  type: feedback
---

Duas metades de um painel podem compartilhar um EIXO e não compartilhar a BASE dele.
Na timeline, uma key é carimbada no tempo do **clip** e um strip senta no tempo da
**timeline**: desenhados contra a mesma régua, a mesma coluna de pixels quer dizer
dois instantes — e nada na tela admite. Playhead em 4.0 = relógio do clip em 2.0.

**Why:** o Enio pediu "um modo isolado para lanes/strips" por achar **confuso**. Investigar
a queixa em vez de só atendê-la achou a causa: não era poluição visual, era **duas bases de
tempo numa régua**. O ADR-0115 R8 tinha decidido "sem modo" e estava certo sobre o *modo* —
errou o corolário (concluiu que as metades podiam coabitar uma VISTA sem notar que
coabitavam uma RÉGUA). Passou despercebido porque **sem pilha os dois relógios são o mesmo**:
a feature que os separa era a que ninguém tinha usado ainda.

**How to apply:** quando o usuário disser que uma tela é confusa, procure a **unidade** antes
do layout — dois dados no mesmo eixo, a mesma cor com dois sentidos, o mesmo número com duas
escalas. Se as bases divergem, separe (o padrão-ouro separa: Unity nem deixa editar key na
janela do Timeline; Blender usa editores distintos; quem mistura — Unreal — tem usuário
pedindo socorro). E o degenerado ("sem pilha é um relógio só") é o que esconde o bug **e** o
que torna a correção barata: nesse caso nada muda. Ver [[feedback_ergonomics_verdict_is_a_design_bug]],
[[feedback_documented_decision_chesterton_fence]], [[feedback_two_doors_to_the_same_question_diverge]].
