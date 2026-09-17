---
name: feedback-a-wait-for-calm-loop-fires-when-your-own-run-starts
description: Um laço «espera por load < 5 e mede» dispara no segundo em que a NOSSA corrida arranca — a média de 1 min ainda não a viu; exija leituras calmas seguidas e nada seu ao lado
metadata:
  type: feedback
---

Medido em 2026-09-13 (`line/3DModeling`, W148): um relógio A/B em fundo esperava `until load1 < 5`, e eu
lancei a suíte das três crates do campo ao mesmo tempo. O laço disparou a **`4,63`** no mesmo segundo em
que a suíte arrancou (a média de 1 minuto é uma média — ainda não tinha visto a carga que eu próprio
acabava de criar), e a medição inteira correu a **`37`–`60`**. As colunas de ms saíram inúteis; as de
contagem valeram.

**Why:** a média de carga atrasa ~1 minuto em relação ao que se lança. Uma condição de uma leitura só
mede *o passado*, e o disparo coincide exactamente com o arranque de qualquer corrida nossa.

**How to apply:** um laço que espera pela máquina calma exige **duas leituras calmas seguidas** com
60 s entre elas (1 min **e** 5 min abaixo da barra), imprime o `loadavg` antes e depois, e **nada meu
corre ao lado** enquanto espera ou mede — se for preciso correr gates, eles vão antes de armar o laço
ou depois de ele acabar. Ligado a [[reference_topic_measurement_discipline]] e à regra do `CLAUDE.md
§5.0` (nenhum relógio vale acima de `load ~5`).
