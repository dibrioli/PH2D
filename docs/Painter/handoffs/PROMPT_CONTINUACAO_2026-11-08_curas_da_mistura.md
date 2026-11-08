/pd-linha-assumir PainterWatercolor

Estado: commits de 08/11 na linha (b5d5d5794f mistura visível · de47355fc3 Pointillism Hardness 0,5 · fd5b28f4a5
Fixed dab size · b2693186b8 sondas da auditoria + Pointillism regerado). Leia: a memória
`project-memory/project_color_mixing_tuned_for_our_engine_not_csp_2026_11_08.md` (decisões do dono), e os dois
relatórios da auditoria nas pastas `docs/Painter/handoffs/material_2026-11-08_auditoria_papel_grao/` e
`material_2026-11-08_auditoria_composite_camadas/` (sondas `tests/it/sonda_auditoria_*.rs`).

Próximo item: CURAR num bloco só (CLAUDE.md §0 item 10) os defeitos que a auditoria MEDIU da mistura Wet blend
(`mistura_opaca`) — todos de MOTOR:
1. Composite: a camada Erase PINTA com Wet blend (`stamp_route.rs:520` só olha `stroke_erases()`, não
   `brush.blend == EraseAlpha`).
2. Composite: as camadas Brush partilham UM reservatório/película por traço → o byte depende da cadência dos eventos
   (60 × 120 eventos difere) e a camada de cima apanha a tinta da de baixo. Dono: cada parte mistura SÓ a tela de
   ANTES do traço → um estado de mistura por posição da pilha, trocado em `acumula_camada` como `composite_mask`/
   `rng_camada`. Prova: 60 × 120 eventos = mesmo byte com Composite + mistura.
3. Composite (receita do dono Blur/Smear/Brush): a Strength da camada Brush não age com Wet blend
   (`mistura_opaca.rs:336-470` não lê `spec.strength`). Confirme por que a pilha de 2 Brush a respeita.
4. Esbatedores (Amount 0) pintam a cor do pincel no rasto sobre o vazio. Dono APROVOU: pesar pela quantidade,
   `P = (res·c·(1−a) + pincel·a) / (c·(1−a) + a)`; muda a cor a meio da carga em todos os pincéis com mistura →
   alarmes de pincel + relatório ao smoke.
5. Controlos mortos com Wet blend: Grain Colors, Shape Color, Per-Layer Color (e ponta de pixel): a rota da mistura
   sai em `stamp_route.rs:520` antes de `color_ramp_owner` (`:592-626`). Cura: a cor por píxel da rampa/camada
   entra como a cor do pincel na mistura.
6. Pincéis que espalham ficam cheios com a mistura (Pointillism, Pastel, Dry Gouache): `spec/queries.rs:395`
   (`PASSO_DA_MISTURA` 3 %) multiplica os pontos espalhados. Cura: amostrar a 3 % sem carimbar a 3 % nos pincéis que
   espalham (veja junto com o Fixed dab size novo).
7. NOVO (dono SIM): opção da mistura «Sample: This layer · All visible layers» (inglês, no cartão do Wet blend);
   «All visible layers» de fábrica no Gouache Blender e no Finger Tip.

Abertos à parte: `o_pincel_guardado_pinta_o_mesmo_traco` (lib `ph2d-tool-painter`) já reprovava em de47355fc3 —
ache a causa; Wet Wash Mix (grão em imagem não aparece no traço — motor, `dab::grain_at`); Pastel fraco (tecto do
spray com grão); Pastel com ponto fixo sem referência (decisão do dono); Splash espalha muito no tamanho de fábrica
(o dono olha no smoke).

Inegociáveis: Watercolor Basic idêntico ao byte; mistura Off pinta como antes; pincéis sem mistura não mudam; os 2
alarmes de pincel por nome; teste no app com foto antes do smoke. Não integre nem faça push sem ordem.
