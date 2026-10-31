# R2 — pincéis do Rebelle que faltam (janela reserva 02, 31/10)

Ramo `line/PainterWatercolor-r2` (base `6ba5d06493`). Três agentes em paralelo por família, método REGRAS_DE_CUSTO
(simular antes de construir, ≤ 2 rondas, teste por nome). Detalhe: [A pontas](R2_A_pontas.md) ·
[B óleo](R2_B_oleo_grao.md) · [C spray/marcador](R2_C_spray_marker.md).

## Resultado

| pincel | estado | o que mudou | prancha |
|---|---|---|---|
| Charcoal | melhorado, na fábrica | fim CARREGA (esbate negativo): fim do recto 200 0,21·0,07·0,03 → 0,32·0,15·0,08 (Rebelle 0,31·0,18·0,06); miolo igual | [r2_charcoal](pranchas/r2_charcoal.png) |
| Wax Scrape | melhorado, na fábrica | fim 0,64·0,28·0,09 → 0,82·0,55·0,32 (Rebelle 0,84·0,63·0,48); miolo igual | [r2_wax_scrape](pranchas/r2_wax_scrape.png) |
| Square Hard | melhorado, na fábrica | esquina da entrada recta pelo eixo: altura no canto ÷ corpo 0,59 → 0,95 (40 px), 0,70 → 0,95 (200 px) | [r2_square_hard](pranchas/r2_square_hard.png) |
| Marker Chisel (candidato) | muda de passagem | a mesma esquina recta (lei das pontas rectas) | — |
| Knife Stain | NÃO curado | mecanismo medido: `sem_entalhes` (`oleo_borda.rs`) sobe cobertura à mediana e pinta 13,8 % do papel entre grãos; com o grão salvo sobram 2–3 pontinhos de sombra na tinta (`a_orla_rasgada_nao_tem_pontinhos_escuros`) | [r2_knife_stain](pranchas/r2_knife_stain.png) |
| Knife Grainy | NÃO curado | o grão morre no DEPÓSITO (sem óleo já 16,7 × Rebelle 85,6); 7 variantes de receita, nenhuma dá as lascas claras/escuras | [r2_knife_grainy](pranchas/r2_knife_grainy.png) |
| Grunge | não começado | sem receita em `oleo_acrilica.rs` | — |
| Spray Can | NÃO promovido | lei `VEU_OPACIDADE_MAX = 4` (`c4564d3294`, neutra ≤ 1, teste `o_veu_acima_de_um_e_a_populacao_escura`) não escurece: a cobertura do dab já satura em 1 (opac 1 × pressão; o fluxo só entra na composição) | [r2_spray_can](pranchas/r2_spray_can.png) |
| Dry Marker | NÃO curado | Rebelle: riscas de 8 px ±0,018; no Wet grão de riscas e espaçamento não mudam nada; a receita Digital em estudo está mais perto (candidata a cega/dono) | [r2_dry_marker](pranchas/r2_dry_marker.png) |
| Splatter, Rake Sharp, Blender Grainy | não começados | prazo / rendimento | — |

Leis novas (neutras ao bit no ponto antigo): `pontas_esbate` aceita −4..8 (abaixo de 0 as pontas carregam e o resto
do rasto pousa ao soltar) · entrada das pontas rectas medida no eixo · `veu_opacidade` até 4.
Testes novos verdes: `as_pontas_carregam_com_o_esbate_negativo`, `a_entrada_das_pontas_rectas_tem_a_esquina_cheia`,
`o_veu_acima_de_um_e_a_populacao_escura`. Ignorado com o número: `o_grao_ralo_nao_engorda_no_oleo` (+0,85).

## Fábrica

Charcoal, Wax Scrape e Square Hard regerados (preset + miniatura). `cada_preset_de_fabrica_e_a_sua_receita` e
`the_factory_presets_are_counted` verdes. ⚠️ `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` VERMELHO em 23 presets
fora da R2 (Blend ×4, Smear ×5, Oil ×10, Ink Brush, Dip Pen, Wet Liner, Wet Wash) — o «miniaturas (24 mudam)» da J11
no ESTADO, anterior à R2; nenhuma lei da R2 corre neles. Fica para a integração. Idem `a_borda_da_mancha_molhada_esbate`
(Wet Liner, `pontas_esbate` 0), visto uma vez pelo R2-A sob edições dos irmãos.

## Precisa de ficheiro proibido / decisão

- `spec.rs`: doc de `pontas_esbate` passa a −4..8; o chão devia morar ao lado de `PONTAS_ESBATE_MAX` (hoje em `ids`).
- Knife Stain: o pontinho é a sombra da tinta à volta do furo novo (por eliminação) — curar na luz do óleo; depois
  regerar Oil/* pela fila pesada.
- Spray Can: tinta na FORÇA (opac 0,4 · fluxo 1) ou fluxo próprio do véu — muda a acumulação, medir antes.
- Dry Marker: glaze por traço (CAP-11) com carimbo visível; por que o Wet ignora o grão (`wetpaint/pousa_o_dab.rs`).
- Square Hard: grão fino e denso pelo diâmetro (lei de carga/grão por tamanho).
- Charcoal: o Rebelle tem estrias horizontais no miolo; o nosso é granulado.

## Rendimento

~1 h 40 de agentes (3 × ~30 min + 3 retomas): 3 pincéis melhorados, 0 aprovados por cega (não corrida), 3 leis de
motor neutras. Abaixo de 1/h ⇒ reavaliar com o dono antes de outra ronda.
