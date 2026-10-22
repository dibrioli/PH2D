# HANDOFF — JANELA 5 (fecho da missão, 2026-10-22)

> Leitor: a próxima LLM / o integrador. Integração: [HANDOFF_INTEGRACAO ...PINCEIS](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-22_PINCEIS.md)
> (fecho, colisão, números). Estado: [ESTADO.md](ESTADO.md). Catálogo: [01_CATALOGO §3d](01_CATALOGO.md).
> Árvore `Worktrees/line-PainterWatercolor`, `line/PainterWatercolor`, base da J5 `41b958338b`, HEAD `b74b608d0e`.

## O que a J5 fez (`git log --oneline 41b958338b..HEAD`)

- **Exames** (regras de custo: ≤ 2 rondas por pincel, um dono por ficheiro de motor):
  - **Aprovado:** Pastel Side (`c6c99afab7`; pó solto medido, Flat Soft 60 grãos/100 px × nosso 59; revisão cega r4: o
    NOSSO tomado por original em 4/6; portão `o_pastel_side_e_o_lado_do_bastao_com_po`, mutações sem lâmina e sem pó sangram).
  - **Curados, não aprovados:** Square Hard (`2e1212e7ec`, decisão 16: grão salpicado, pontas do 200 esmaecem) · Water
    Brush r2, Dry/Lift, Blender Grainy (`db170ae238`) · Wet Liner, Wet Wash (`4c0a2b7457`) · Gouache Wash (`953eb21e32`).
  - **Reprovados:** Mop (a mancha do molhado é «carimbo»), Gouache Round.
  - **Decisões:** Gouache Flat fica (0,763 do Wet Flat; a cura partia 3 leis) · Wet Dry Brush não se funde (vizinho
    Airbrush 1,236). Relatórios de família cortados a ≤ 10 KB (`29534edaaf`).
- **Motor:** `Knob::WetOpen`/`WetOpenPush` (`73b8e11688`, tecto da cauda `eb04c3440d`) em `ph2d-wet-paint`
  `trail/molhado.rs`; molhado × molhado, desligado por omissão; portão `o_azul_abre_uma_mancha_redonda_no_vermelho_molhado`,
  3 mutações sangram. Ligado no Wet Liner e Wet Wash — o ganho não bastou para aprovar (resta a borda esbatida, §3d).
- **Biblioteca** (`2d6116b4c3`, [exames/biblioteca.md](exames/biblioteca.md)): matriz dos 45 (nenhum par fora de família
  < 0,596), custo calmo a 5/40/200 px (p95 ≤ 12,5 ms; Ink Brush 12,54 ms, 11/475 quadros > 16,7 — a vigiar; RSS ≤ 105
  MiB; load 1–3,4), arrumação; série de lápis por grau (`002c75267b`).
- **Tutorial** do dono (`a054d6508f`): `tutorial/01_pinceis.pdf`, 12 páginas, com as perguntas das decisões 5 e 8.
  Catálogo §3d (`9b8ab81ae0`): 18 aprovados, fábrica 45, o porquê do alvo falhado e a fila de motor.
- **Fecho** (1×, `PH2D_SUITE=fecho`): 19 576 corridos, 0 falhas, 2 TIMEOUT de carga promovidos à família de flakes
  (`b74b608d0e`). Curas do fecho: `d9d763b06d` `b515ce2549` `b12e750c11` `ddb6a9608e` `f6e2242bac` `27179b7792`
  `9851c5e93b` `48be9e05c0`; estilo `1543fb9f5b` `0f3c03a695` `348fb0644c` `cde29e86ed`. Detalhe no handoff de integração.

## Lições

- O defeito que travou famílias inteiras foi o mesmo (molhado × molhado / borda da mancha): um recurso de motor vale
  mais que rondas de valores — e mesmo assim um knob só destrava pincéis que dependem SÓ dele.
- O fecho achou 17 falhas reais em código que ninguém tinha testado junto (portas de cor, ficheiros > 700 LOC,
  controlos mortos, altura do painel): o gate batched do fecho paga-se; não o antecipe por rondas a meio.
- Alvo de contagem («80–120») sem lista de motor que o suporte é meta enviesada: o catálogo agora diz qual motor
  destrava quantos.

## Aberto

Fila de motor (§3d) · brecha do gate `arch_color_space_typed` (`set_brush_shape_image_rgba`, `&[u8]` em linha à parte) ·
Ink Brush a vigiar · decisões 5 e 8 do dono (smoke pendente).
