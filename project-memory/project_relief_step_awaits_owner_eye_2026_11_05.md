---
name: project_relief_step_awaits_owner_eye_2026_11_05
description: "Bug #71 (tinta parece afundada em papel de relevo alto): cura pronta mas desligada em PH2D_PAPEL_SEM_DEGRAU; só o olho do dono decide ligar"
metadata:
  type: project
---

Bug #71 medido em 05/11: `height_at` soma `dente×papel_a_vista`, a tinta tapa o dente e a altura média cai dentro do traço (degrau para baixo). A cura (`papel + meio_do_dente×(1−v)`) existe atrás de `PH2D_PAPEL_SEM_DEGRAU=1` e NÃO é o padrão.

**Why:** a cura muda a cara de pincéis que o dono aprovou (Soft Pastel e Graphite 8B perdem grão em relevo, até Δ78/Δ50), e ele disse que adora os pincéis como estão. Só ele julga o ganho (S médio +0,048 → +0,005) contra a perda.

**How to apply:** não ligue a chave por omissão nem remova-a sem o dono ver a [prancha](../docs/Painter/handoffs/material_2026-11-05/relevo_antes_depois.png) e pedir. Se aprovar: tirar a chave, portão com a métrica S, varrer papéis (só Cold foi medido). O controlo Paper Press (0×1) fica: [[project_paper_press_is_a_brush_control_2026_10_28]]. Referência dos pincéis: [[project_brushes_as_they_are_are_the_reference_2026_11_05]]. Detalhe: `docs/Painter/BUGS_painter.md` #71.
