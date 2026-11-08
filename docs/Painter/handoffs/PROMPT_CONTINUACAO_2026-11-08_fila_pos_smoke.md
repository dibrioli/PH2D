/pd-linha-assumir PainterWatercolor

Estado: linha `line/PainterWatercolor`, último commit = o do fecho da onda «curas da mistura» (08/11, depois de
`24c82ab7c3`). Curas #73–#80 feitas; smoke do dono OK nos pontos 2–6; Pointillism/Pastel e dois pedidos novos
abertos. Nada integrado nem enviado.

LEIA ANTES DE TOCAR EM CÓDIGO:
- `docs/Painter/handoffs/HANDOFF_ONDA_line_PainterWatercolor_2026-11-08_CURAS_DA_MISTURA.md` — o que entrou, o
  veredito do dono verbatim, a FILA (a)–(g)+memória do 1.º traço, o que está aberto.
- `docs/Painter/BUGS_painter.md` § «Bugs #73–#80 (CURADOS)» e `docs/Painter/AUDITORIA_MISTURA_2026-11-08.md` (a
  medição antes da cura; «Conferido e bom» = não refazer).
- `project-memory/project_mixing_smoke_verdicts_and_queue_2026_11_08.md` e
  `project_color_mixing_tuned_for_our_engine_not_csp_2026_11_08.md` (decisões do dono).

Próximo item: a fila do handoff, em bloco (CLAUDE.md §0 item 10: junte as famílias equivalentes — plano único, um
lote de binários, UMA rodada de medição, um gate batched; série só se o resultado de um passo muda o seguinte, e
diga-o). Ordem proposta, reordene medindo: (a) Composite: sizes em px nas camadas · (b) Grain/Grain Colors com
mistura não matam o Paper (MEDIR primeiro) · (c) Pointillism: mostrar ao dono antes/depois/Off e devolver o look
aprovado · (d) Pastel parecer pastel, não spray (oráculo do produto do dono antes de construir; simular antes; o
dono faz a triagem) · (e) campo de busca no dropdown de pincéis · (f) prévia especial dos esbatedores (Finger Tip,
Gouache Blender e todo pincel que só mexe tinta): efeito sobre traços de tinta pré-pintados, na miniatura e onde a
prévia aparece · (g) abertos herdados (Wet Wash Mix, Splash, SPRAY_FIXO_MAX, painel antigo, rótulos). Memória do 1.º
traço (dono 07/11) só depois do CI da integração — confira o estado.

Inegociáveis: Watercolor Basic idêntico ao byte (`o_watercolor_basic_nao_mudou_com_o_pigment_mix`); mistura Off pinta
como antes; pincéis sem mistura não mudam; os 2 alarmes de pincel por nome
(`a_largura_de_fabrica_e_a_aprovada`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`) — as 3 miniaturas
vermelhas (Pastel, Pointillism, Chalk) NÃO se regravam sem o dono (`PH2D_PINCEIS_APROVA=1`); teste no app em Xvfb
próprio, `PH2D_WETPAINT_SMOKE=1`, perfil smoke, `import -window`, nunca `spectacle`, foto antes do smoke; UI nova:
maquete/foto lado a lado antes do smoke, textos em inglês; agentes ajudantes 45 min cada; nada de
`fila-pesada.sh correr`; não integre nem faça push sem ordem do dono.
