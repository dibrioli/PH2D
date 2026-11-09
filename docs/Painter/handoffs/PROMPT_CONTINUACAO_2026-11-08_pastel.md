/pd-linha-assumir PainterWatercolor

Estado: linha `line/PainterWatercolor`, último commit = o do fecho da onda «relevo do papel no Digital + preview do
Smear/Blur» (08/11, depois de `e479b31c49`). Smoke do dono OK nas três coisas; nada integrado nem enviado.

LEIA ANTES DE TOCAR EM CÓDIGO:
- `docs/Painter/handoffs/HANDOFF_ONDA_line_PainterWatercolor_2026-11-08_RELEVO_DIGITAL_E_PREVIEW.md` — o que entrou,
  os vereditos verbatim, o que fica conhecido e a FILA.
- Memórias `project_mixing_smoke_verdicts_and_queue_2026_11_08.md` (o Pastel «spray ruim»),
  `feedback_brush_method_owner_triage_and_simulate_before_building.md`,
  `feedback_the_owners_reference_product_decides_the_law.md`, `feedback_brush_oracle_must_include_blank_canvas.md`,
  `project_digital_paper_relief_must_show_on_paint_2026_11_08.md` (a rampa do papel na tinta é decisão do dono).

Próximo item: **o Pastel parecer pastel, não spray**. A queixa é ANTERIOR à revogação da #78 (que devolveu
Pastel/Chalk/Pointillism à miniatura aprovada): PRIMEIRO fotografe o Pastel de hoje no app (e a miniatura) e pergunte
ao dono se ainda é «spray ruim» e qual o produto de referência dele para pastel. Só então: oráculo CORRIDO (licença
por artefacto, entradas nossas, 3 fundos: branco, blocos, tela vazia), simular o conserto antes de construir,
antes/depois lado a lado, o dono faz a triagem. Depois, em bloco: (2) abertos herdados · (3) papel de imagem ·
(4) memória do 1.º traço (o `noite` do main ficou success em 08/11 — confira de novo).

Inegociáveis: Watercolor Basic idêntico ao byte (`o_watercolor_basic_nao_mudou_com_o_pigment_mix`); Paper desligado e
pincel sem Grain = mesmo byte; os 2 alarmes de pincel por nome (`a_largura_de_fabrica_e_a_aprovada`,
`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`) — regravar só com aprovação do dono (`PH2D_PINCEIS_APROVA=1`);
Modo L: foundational append-only permitido, PARE e reporte ao Enio só em contrato congelado §6 ou rebase a conflitar
fora dos seus arquivos; fotos no app por `ph2d-run.sh` em Xvfb próprio com o smoke RECOMPILADO, `import -window`, nunca
`spectacle`, foto antes do smoke; texto de UI em inglês; agentes ajudantes 45 min cada; nada de `fila-pesada.sh
correr`; não integre nem faça push sem ordem do dono.
