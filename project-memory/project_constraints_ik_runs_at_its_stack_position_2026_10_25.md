---
name: project_constraints_ik_runs_at_its_stack_position_2026_10_25
description: Constraints (line/BlenderConstraints) — a IK corre na POSIÇÃO da pilha, não por último como no Blender; decisão delegada pelo dono em 25/10
metadata:
  type: project
---

Na pilha de constraints do PH2D a IK é um item como os outros e corre na posição em que está na lista.
O Blender corre-a sempre por último (medido no 5.2.3: um Copy Rotation abaixo da IK some sem aviso);
Spine, Rive e Unity 2D IK seguem a ordem da lista.

**Why:** o dono delegou a escolha em 25/10 («descubra o melhor e faça»). A regra «a ordem da lista é a
ordem do cálculo, sem exceções escondidas» é o contrato de UX da pilha (plano §0 item 5).

**How to apply:** não perguntar de novo na W2. A razão técnica do Blender (a IK reescreve os ancestrais
da cadeia) cura-se na `ordem()` com a aresta «descendentes da cadeia fora do caminho dependem do dono da
IK» — detalhe em `docs/BlenderConstraints/01_plano.md` §2.2. Gate `ik_is_an_ordinary_item_in_the_stack`.
