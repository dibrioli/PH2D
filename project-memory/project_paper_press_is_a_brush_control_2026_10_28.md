---
name: project_paper_press_is_a_brush_control_2026_10_28
description: "Dono 28/10 (smoke J6): a tinta que AMASSA/afunda o relevo do papel é um efeito a GUARDAR — vira controlo do pincel (0 = segue o papel, 1 = amassa); nunca o apagar"
metadata:
  type: project
---

No smoke de 28/10 o dono viu que, com o Real Oil e o Relief alto no cartão Paper, quase todos os pincéis «apertam» o
papel: dentro do traço o relevo some e a tinta fina fica abaixo dos picos (Knife Stain), enquanto a tinta fina sobre
tinta grossa segue o relevo. Causa medida (`oleo_real.rs`): o dente do papel entra só no detalhe ×cobertura e nunca
na altura base nem na sombra — tinta sobre tinta SOMA, tinta sobre papel SUBSTITUI. Decisão do dono: «ter a
possibilidade de afundar o papel é muito interessante e deve estar disponível no painel … descubra como evitar, como
preservar e como melhorar. Mas não exclua esse efeito.»

**Why:** é um efeito expressivo (papel prensado) que nasceu de uma assimetria; o dono quer escolhê-lo, não perdê-lo.

**How to apply:** a lei coerente é superfície = substrato (papel com dente, ou a tinta de baixo) + espessura nova, e o
«afundar» é um controlo do PINCEL (guardado por célula, só vale para os traços seguintes — gates
`mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada`/`um_controlo_do_pincel_nao_recompoe_a_tela`) que leva do «segue o
papel» (0) ao «amassa» de hoje (1). O valor de omissão escolhe-o o dono por foto (0/½/1 + Rebelle). Nunca curar a
assimetria apagando o efeito. Ver [[project_oleo_real_mission_w2_2026_10_27]].
