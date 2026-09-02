---
name: feedback-a-bar-calibrated-without-the-approved-side-measures-our-own-defects
description: "Uma barra calibrada só com a NOSSA saída mede a distância entre os nossos próprios defeitos; e o PISO de quem entra no censo decide o que a régua vê — corra a régua no lado que o dono APROVOU antes de a fixar (quadextract, 2026-09-02)"
metadata:
  type: feedback
---

Quad remesh, 2026-09-01/02. A jornada de 01/09 entregou três curas cujas réguas diziam
*«melhorou muito»* (desvio `0,47 → 0,22`, dobras `17 → 6`) e o Enio disse *«absolutamente nenhuma
melhoria»* — quatro vezes num dia, com foto. Nenhuma régua da linha tinha alguma vez sido corrida
sobre a retopologia que ele **aprovou** (`Sculpt_Blender.obj`). Corridas em 02/09, os defeitos
eram de **calibração**:

- **A barra da grade (`1,5`) saía do vazio entre as NOSSAS pontas boas (`0,38–0,52`) e as NOSSAS
  más (`1,43–3,85`).** A aprovada entrega `≤ 0,79` em todas as pontas; a barra deixava passar a
  `1,10–1,40` exactamente o que ele via. Barra certa: `1,0`, no vazio `0,88…1,10`.
- **O piso do censo (`0,55` do raio, corte em `12`) escondia as pontas da foto** (`0,43–0,51` do
  raio): *a régua do produto media 4 pontas e a da foto não era nenhuma delas*. Todas as tabelas
  de «pior ponta» de dois dias eram sobre a população errada.
- **A mediana da vizinhança afogava o ponto que define a ponta**: a agulha reprovada lia `p50 0,84`
  (verde) com o ápice a `1,11` da superfície.
- **A malha aprovada também tem coisas grossas** (bossas a `1,0–1,47`, um botão de 5 células a
  `1,35`) — logo o filtro de quem é «ponta» é *load-bearing*: baixar o piso sem filtro de forma
  acusaria a peça aprovada.

**Why:** uma barra só tem dois lados quando os dois foram medidos. Sem o lado aprovado, o vazio
que se vê é entre os nossos defeitos, e uma cura que atravessa esse vazio lê-se como progresso
sem mover o que o olho vê. E o censo — quem entra na régua — é uma barra escondida: um piso
que não deixa a ponta da foto entrar torna toda medição «pior ponta» uma afirmação sobre outra
população.

**How to apply:** antes de fixar qualquer barra, corra a régua sobre o artefacto que o dono
APROVOU e sobre o que REPROVOU, e escreva os dois lados na tabela ao lado da constante; se não
há aprovado, diga-o. Quando uma régua acusa «0 de N», pergunte *quem são os N* e se a ponta da
foto está lá. Quando a saída de outra ferramenta é a referência, confira de que ENTRADA ela
veio (caixa, contagem) — a aprovada era de outra escultura. E olhe para o artefacto: um arame de
cada ponta (renderizador de 80 linhas) mostrou o que nenhuma coluna mostrava. Portão que fica:
`crates/ph2d-quadfill/tests/pontas_do_dono.rs`. Ligado a [[feedback-the-measured-refusal-you-need-is-in-the-neighbouring-knob]] e
[[feedback-a-missing-knob-cell-can-hide-a-defect-measure-before-pricing]].
