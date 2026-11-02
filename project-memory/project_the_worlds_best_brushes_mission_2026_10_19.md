---
name: project_the_worlds_best_brushes_mission_2026_10_19
description: "Missão do dono (19/10): os melhores pincéis do mundo no Painter — os mais usados do Procreate/Krita/Photoshop/Rebelle/MyPaint reproduzidos com QUALQUER dos 4 motores; 5 janelas sequenciais autónomas, sem pedir permissão"
metadata:
  type: project
---

Depois do smoke do P53, o dono (19/10): *«você vai assumir um trabalho gigantesco que vai completar sozinho sem
minha ajuda e sem solicitar nenhuma permissão»* — definir e implementar uma GRANDE quantidade dos pincéis mais
amados e usados (Procreate, Krita, Photoshop, Rebelle e outros), com **qualquer um dos 4 motores** (Digital,
Watercolor, Impasto, Wet Paint), instalando e estudando apps se preciso. 5 janelas do Claude, uma por vez, cada uma
passa a vez à seguinte por sinal; agentes em paralelo dentro de cada janela para não esgotar o contexto.

**Why:** supera a decisão de 07/10 ([[project_wet_paint_is_the_only_painter_mode_to_evolve_2026_10_07]]) no USO dos
motores; a cerca byte-idêntica dos 3 meios antigos continua para o comportamento que já existe (capacidade nova
nesses motores entra aditiva, desligada por omissão). O critério de qualidade é o de
[[feedback_a_brush_that_passes_the_ruler_can_still_be_useless]].

**How to apply:** o plano e o contrato das janelas vivem em `docs/Painter/Pinceis/00_MISSAO.md` (line/PainterWatercolor);
cada janela tem o seu `JANELA_<n>.md` e escreve `ESTADO.md` + o sinal à seguinte. Oráculo é um programa que se CORRE
(CLAUDE.md §0.9): Krita é GPL (só correr), libmypaint ISC e os `mypaint-brushes` (licença por ficheiro) são porta
aberta; ⛔ GIMP não é referência (dono: «fraco, estude os fortes»); os fortes (Clip Studio, ArtRage, Corel Painter, Photoshop/Fresco) tentam-se sob Wine como o Rebelle; Procreate/Photoshop que não correm — os pincéis deles reproduzem-se pelo comportamento documentado e por
equivalentes que CORREM, nunca copiando os ficheiros deles. Integração e push continuam só por ordem explícita.

**Encerrada (J5, 22/10), à espera do smoke do dono:** 18 pincéis aprovados no exame lado a lado; fábrica 28 → 45; o
alvo 80–120 NÃO foi alcançado. Os outros itens do catálogo estão travados por MOTOR, não por valores: depois da
2.ª ronda o rendimento é zero, e a ordem de custo do dono (21/10: ≤ 2 rondas por pincel, uma lacuna de motor por
agente, revisor `sonnet`) fechou a fila. Missão futura começa pela fila de motor medida em `01_CATALOGO.md` §3d. O fecho da linha está
verde; o handoff é `docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-22_PINCEIS.md`, e o
tutorial do smoke está em `docs/Painter/Pinceis/tutorial/01_pinceis.pdf` (decisões 5 e 8 perguntadas lá).
