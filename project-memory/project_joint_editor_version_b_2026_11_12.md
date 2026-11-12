---
name: project-joint-editor-version-b-2026-11-12
description: Dono 12/11 — Joint Editor = versão B (gabarito por tipo de junta, arrastar peças para os papéis); o Inspector NÃO perde as juntas, melhora; sem monociclo; objetos prontos sim, mágicos não
metadata:
  type: project
---

Decisões do dono em 12/11/2026 sobre a seção «Joint Editor» da crítica (`/home/enio/Documentos/GameEngineSamples/critica_ph2d.html#juntas`, fonte `fonte_critica/onda3c_juntas.py`):

1. **Versão B**: aba central de juntas (molde Input Map / editor de animações); em cima o tipo de junta (Wheel, Pin, Rope…) e o gabarito (Car, Motorbike, Custom…); no meio o desenho com uma forma tracejada por papel; arrastar um objeto do canvas/Hierarchy para dentro do papel cria a junta com os números daquele gabarito. Depois de criada, a junta aparece no Inspector de cada peça.
2. **«Não tire do Inspector. Melhore o Inspector.»** O cartão Joint e os gestos de hoje (Join Selected Bodies, Draw Joint on Canvas, Chain N, Rig N Parts) FICAM; o cartão ganha nomes por papel, unidades, editar as duas rodas de uma vez. Revoga o «no Inspector fica só Create joints» do pedido inicial.
3. **Sem monociclo** — era só exemplo.
4. **Objetos comuns prontos são permitidos**, plenamente configuráveis e com tudo exposto na UI. **Proibido: objeto mágico que parece surgir do nada** (tudo o que o gabarito cria é objeto real, nomeado, na Hierarchy, anunciado).
5. «Faça melhor que no doc de Críticas.»

**Why:** o dono quer montar máquinas físicas (carro, porta, ponte) sem ajustar 17 campos por junta, sem perder o controle fino.
**How to apply:** linha `line/JointEditor` (`Worktrees/line-JointEditor`); plano em `docs/JointEditor/01_plano.md`. Relaciona-se com [[project-physics-critique-answers-2026-11-09]] e com a regra de 02/11 «nada de objeto pronto no Add» (aqui flexibilizada pelo item 4 para gabaritos de junta).
