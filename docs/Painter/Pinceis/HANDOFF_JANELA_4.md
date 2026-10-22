# HANDOFF — JANELA 4 → JANELA 5 (2026-10-21)

> Leitor: a JANELA 5. Leia antes: `ESTADO.md` (a FILA nova e as decisões 15–17), `REGRAS_DE_CUSTO.md` (ordem do dono:
> «muito gasto e pouco resultado» — valem para si e para todo agente), `JANELA_5.md`. Árvore
> `Worktrees/line-PainterWatercolor`, `line/PainterWatercolor`, base da J4 `823c6e1b2b`.

## O que a J4 fez
Categorias F, G, H, I, K, L, M, uma família por agente (`BRIEFING_FAMILIA.md` + um comum por janela), com rondas de
motor por família. **11 aprovados** (fábrica 34 → 44: Wet 28, Digital 16):

| família | aprovados | curados, não aprovados | em estudo / fundidos |
|---|---|---|---|
| oleo_acrilica | Filbert (refeito, vizinho Dabber 0,89) | Oil Bristle, Express Oil, Dabber, Flat Bristle (estudo) | Acrylic Round → Hard Round (saiu), Rigger → Hard Round; Palette Knife, Rake, Thick Impasto, Dry Brush, Fan, Wet Blend, Nikko Rull |
| mistura | Hard Eraser, Kneaded Eraser, Blender Soft | Water Brush (estudo) | Smudge Finger, Blur, Blender Grainy, Dry/Lift; Smudge Spiky → Smudge Finger |
| texturas | Halftone, Stars, Bokeh | Foliage, Grass, Noise Grit, Spray Can, Splatter, Stipple | Clouds, Hatching sem oráculo comparável |
| efeitos | Dodge, Burn, Color Glaze, Ballpoint (C-7 da J3) | Glow | Sketchy Web, Metallic, Particles (motor grande), Glitter (sem foto); Hue/Overlay → Color Glaze |
| aguarela | — | Wet Round, Wash, Liner, Flat, Dry Brush, Drip (pré-existentes) | Mop, Bristle WC, Granulation, Fringe Bloom, Spatter WC, Optical Wash, Sumi-e |
| pastel_carvao | — | Charcoal, Soft Pastel (com pó), Square Hard, Oil Pastel, Wax Scrape | Compressed, Powder, Pastel Side, Wax Crayon; Charcoal Pencil → Soft Pastel |
| guache | — | Gouache Flat | Gouache Round, Dry, Wash |

## Motor novo (tudo aditivo, desligado por omissão, portão red-first + mutação; detalhe nos relatórios)
Wet: `Knob::StrokeFilm` (miolo liso: máximo por célula em vez da soma; a ladrilha das cerdas causava as listras) ·
`bristleStreak*` (o veio dos fios) · `thinStrokeFade` · `Knob::PressureGrain` · `Knob::DustDrop` · o pó solto
(`graos_do_po`, `pousa_grao_solto`, `po_lado`) · `Knob::WaterPush` · grão no Blend · `Knob::DryerEdge`.
Digital: `particle_px`/`particle_max_px`/`particle_min_px` · `CorDaMao::vies` · `GrainLaw::area` · `passo_pela_pressao`
(+ velocidade) · `veu` · `parado_s` · `speed_lag_s` · `wire_sorteio`/`wire_cordas` · `pigment_espectral` (lei do
MyPaint 2, libmypaint ISC) · linhas Pen «Speed softness»/«Pressure hardness» (cartão 21 → 23 linhas).
Leis: a rampa da pressão (decisão 15). Fábrica: `ReceitaFerramenta`, `ReceitaMeio::texturas`, base das ferramentas,
matriz «ferramenta».

## Lições (para a J5)
- O rendimento cai a pique depois da 2.ª ronda num pincel: 6 rondas no Halftone/Filbert, 0 mudança de veredito. ⇒
  `REGRAS_DE_CUSTO.md`. O que travou quase tudo no Wet foi o MESMO defeito em 4 famílias (o molhado × molhado):
  um recurso de motor bem escolhido vale mais que 10 rondas de valores.
- Agentes em paralelo no mesmo motor colidem (knobs na mesma tabela, `a_lei_da_tinta` vermelho por fnv alheio a meio).
  ⇒ ≤ 3 em paralelo, um dono por ficheiro de motor.
- A bancada mentia para ferramentas (foto da camada sem compor, sem base) — conserto da família mistura.

## Fecho da J4
Portões da fábrica, leis e famílias corridos por nome no fim — o resultado está na última linha deste ficheiro.

**Resultado (21/10):** 95/96 por nome; o vermelho era a marca gravada do Charcoal e do Soft Pastel, que mudaram de propósito (o pó solto, `9965b60b59`) — regravada, verde por nome. `cada_pasta_de_fabrica_tem_rotulo`, `ph2d-painter-brush` (19) e `ph2d-wet-paint` fingerprint (3) verdes. Árvore limpa.
