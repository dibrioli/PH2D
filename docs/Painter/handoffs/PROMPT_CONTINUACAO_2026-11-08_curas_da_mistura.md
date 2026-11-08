/pd-linha-assumir PainterWatercolor

Estado: linha `line/PainterWatercolor`, último commit da janela de 08/11 depois de 65eccb563f (Fixed dab size no Jitter
com dica). Smoke do dono OK em 08/11 para: mistura visível nos 12 pincéis Wet blend, Pointillism (segue o traço,
Hardness 0,5, Fixed dab size), Size (px) em todo meio, Fixed dab size em todo pincel (no Jitter, com dica), Grain
«Same as Paper» com Paper None = sem grão e a caixa esmaecida. Nada integrado nem enviado.

LEIA ANTES DE TOCAR EM CÓDIGO:
- `docs/Painter/AUDITORIA_MISTURA_2026-11-08.md` — os defeitos #73–#80 com medição, mecanismo (ficheiro:linha de 08/11,
  confira-os), cura e decisão do dono; e o «Conferido e bom» (não refazer).
- `project-memory/project_color_mixing_tuned_for_our_engine_not_csp_2026_11_08.md` — as decisões do dono:
  (a) esbatedores: APROVADA a fórmula que pesa pela quantidade; (b) Composite: cada parte mistura só a tela de ANTES do
  traço; (c) SIM à opção «Sample: This layer · All visible layers», «All» de fábrica no Gouache Blender e no Finger Tip.
- Sondas da auditoria (ignoradas, à mão): `crates/ph2d-tool-painter/tests/it/sonda_auditoria_papel_grao.rs` e
  `sonda_auditoria_composite_camadas.rs` — são a régua de antes/depois de cada cura.

Próximo item: CURAR num bloco só (CLAUDE.md §0 item 10) os 8 defeitos #73–#80. Raiz comum de #73/#75/#77: a rota da
mistura sai cedo em `stamp_route.rs:520` e não lê o que as rotas seguintes leem (blend, Strength, rampas) — cure-os
juntos e varra TUDO o que essa porta salta (ponta de pixel incluída). Ordem sugerida: #73 + #75 + #77 (a porta) →
#74 (estado por posição da pilha; prova: 60 × 120 eventos = mesmo byte com Composite + mistura) → #76 (a fórmula,
muda a cor a meio da carga em todos os pincéis com mistura: alarmes de pincel vermelhos ESPERADOS, relatório
`target/relatorio-pinceis/index.html` vai ao smoke; regere só os afetados com PH2D_REGERA_SO a partir da janela
principal) → #78 (amostrar a 3 % sem carimbar a 3 % nos que espalham) → #79 (a opção nova, cartão do Wet blend, inglês,
com dica de hover; o preset guarda-a) → #80 (regra própria da Depth, Aquarela incluída).
Cada cura: teste por nome que reprova sem ela (prove com a mutação) e passa com ela; as sondas da auditoria antes ×
depois numa folha.

Outros abertos (depois dos 8, ou diga ao dono se não couberem): `o_pincel_guardado_pinta_o_mesmo_traco` (lib
ph2d-tool-painter) já reprovava em de47355fc3 — ache a causa; Wet Wash Mix (grão em imagem não aparece e fica 4× mais
lento — motor, `dab::grain_at`); Pastel fraco (tecto do spray com grão) e sem referência do ponto fixo (decisão do
dono); Splash espalha muito no tamanho de fábrica (o dono olha); filme do Óleo Real ainda com papel de reserva sem
Paper (só com ordem); tecto `SPRAY_FIXO_MAX` medido sem Wet blend.

Inegociáveis: Watercolor Basic idêntico ao byte (`o_watercolor_basic_nao_mudou_com_o_pigment_mix`); mistura Off pinta
como antes; pincéis sem mistura não mudam; os 2 alarmes de pincel por nome; teste no app (Xvfb próprio,
`PH2D_WETPAINT_SMOKE=1`, perfil smoke, `import -window`, nunca spectacle) com foto antes do smoke; agentes ajudantes
45 min cada, sem `fila-pesada.sh correr`. Não integre nem faça push sem ordem do dono.
