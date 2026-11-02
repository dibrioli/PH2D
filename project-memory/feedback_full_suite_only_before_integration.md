---
name: feedback-full-suite-only-before-integration
description: "Dono 23/10 — a suíte completa de testes só corre ANTES DA INTEGRAÇÃO; na linha, só testes por nome"
metadata:
  node_type: memory
  type: feedback
  originSessionId: face16e2-7148-4e0c-af58-9eb75df1d4bd
  modified: 2026-10-23T21:50:39.592Z
---

⛔ **A suíte completa (`nextest-impacted.sh`, ship, `PH2D_SUITE=fecho`, crate inteira com `motivo:`) só corre ANTES DA INTEGRAÇÃO** — quem a corre é o integrador. Dentro da linha (meio do trabalho, fim de onda, fecho da linha): só os testes do item **por nome** e as provas de mutação.

**Why:** dono, 23/10 (line/PainterWatercolor): *«a partir de agora só rode os testes completos antes da integração. Perdemos muito tempo.»* Na mesma janela correram 3 suítes de ~20–37 min cada (19 235 testes) para uma cura de Smudge e uma mudança de valores de fábrica; as falhas que elas acharam eram testes que liam o pincel de fábrica — achá-las é trabalho do integrador.

⛔⛔ **Reiterado 25/10** (dono: *«não corremos a bateria completa de testes a cada commit. Desse modo nunca
acabaremos»*): a «bateria POR CRATE» (todas as crates tocadas, lib + it, com `motivo: formato gravado`) É a
suíte para esta regra — numa janela correram 3 dessas (~22 min cada) por causa de um prompt da janela anterior
que a pedia. Um prompt/handoff que peça bateria NÃO a autoriza; clippy+fmt das crates tocadas no commit são
baratos e ficam.

**How to apply:** na linha, depois de mudar algo partilhado, corra por NOME os portões que o tocam (descubra-os por grep/explorador) e diga no handoff que a suíte fica para a integração. Não use `PH2D_SUITE="motivo: …"` para contornar. Relaciona-se com [[reference-topic-ship-ci-integration-lessons]] (a suíte 1× por linha — agora 1× na integração).
