# ESTADO da missão Pincéis (a FILA viva; ≤ 8 KB)

> Toda janela lê isto primeiro e actualiza-o antes de sair. Contrato: [`00_MISSAO.md`](00_MISSAO.md).
> Última escrita: JANELA 1, 2026-10-20.

## Feito (JANELA 1)

| Entrega | O quê | SHAs |
|---|---|---|
| Bateria | `ferramentas/bateria.py` → `bateria.json`: a fonte ÚNICA dos gestos (3 cenas 1000×640: gestos · tamanhos · sobre) | `4eaee9d72c` |
| A — Rebelle | **os 281 presets fotografados** na bateria (766 cenas, 1 532 PNG, 0 falhas): `fixtures/rebelle_bateria/` + README/indice.csv; ferramentas `bateria_rebelle.py`, `fotos_bateria.sh`, `lote_bateria.sh` (retomável) | `d2a6dfb614` + 14 por TOOL (`7c3ee33b75`…`31583f8e32`) |
| B — Catálogo | `01_CATALOGO.md` (112 pincéis, 13 categorias, 68 P1) + `01b_FONTES.md`; licenças: `mypaint-brushes` CC0 (portável), bundle Krita CC-0 declarado | `c0b2f1cf7b` |
| C — Oráculos | `02_ORACULOS.md`. **ArtRage 6 demo** corre por script (`.arscript`, pressão exacta, sem expirar): 35 pincéis × 3 cenas em `fixtures/artrage_bateria/` (404 presets instalados — mais é só acrescentar à `lista_artrage.tsv`). Krita/MyPaint: ver a linha de baixo | `a08399a7f4` |
| C — Krita/MyPaint | **MyPaint** (harness C sobre libmypaint ISC; 196 `.myb` CC0 = PORTÁVEIS) × 3 cenas, determinístico: `fixtures/mypaint_bateria/` · **Krita 6** (144 presets do bundle padrão, caneta com pressão injectada dentro do Krita, Xvfb próprio): `fixtures/krita_bateria/`. Nenhum expira; nenhum mede tilt/secagem (A = B). Pencil-2/Ink-2 do Krita têm acaso ⇒ comparar por estatística | `dc1dd28633` `6cbdbcd3c9` `0bbabaaedd` `609655ca17` |
| D — Bancada | `04_BANCADA.md`, `05_REVISAO_CEGA.md`; `exame.sh` (pinta a bateria em qualquer preset dos 4 meios: `tests/it/exame_bancada.rs`), `exame_regua.py`, `exame_matriz.py`, `exame_prancha.py` (legível + cega), `--custo` | `39967873bd` `72dbc191f8` `e70940fdd4` |
| E — Motores | `03_MOTORES.md`: o que cada motor exprime (pelo código) e 19 lacunas por ordem | `c9d8dcff53` |
| F — Limpeza P53 | Filbert, Flat Oily, Knife Smooth fora da fábrica (31 → **28**); portões verdes por nome | `d9e85a11c4` |

## Decisões de produto (com o porquê)

1. **Os 3 do P53 saem** (veredito do dono, confirmado de novo por ele a 19/10: «inúteis ou muito ruins — modificados
   ou retirados»). Voltam SÓ na JANELA 4, com a lâmina que gira (L3/C38), e só se passarem o EXAME com o Rebelle ao lado.
   A matriz explicou o Filbert: o oráculo mais perto dele era o Round Hard digital do Rebelle (0,507), não o Filbert.
2. **A matriz é TRIAGEM; o veredito é a prancha + a revisão cega.** Lei medida nos oráculos (`04_BANCADA` §4):
   «claramente longe» = **d ≥ 0,596** (o p10 do vizinho-mais-perto entre os presets do Rebelle, fora da família;
   1,8× o ruído de repintura 0,33). **Série graduada** (dureza/densidade) entra como FAMÍLIA: fora dela ≥ 0,596;
   dentro, cada degrau ≥ 0,40 (o menor degrau que a prancha do Rebelle mostra: HB→2B). Escala congelada em
   `matriz/escala.json` (228 presets do Rebelle).
3. **13 dos 28 de fábrica estão «perto»** (Colored 4B~Oil Pastel 0,30 · Dip Pen~Ink Bullet 0,36 · Soft Pastel~Wax
   Scrape 0,36 · Graphite 8B~Oil Pastel 0,39 · Marker Liner~Ink Bullet 0,43 · Graphite HB/2H/2B~Colored 4B · Marker
   Bullet~Liner · Square Hard~Oil Pastel; 2H→HB = 0,37 < 0,40). ⇒ os existentes passam pelo MESMO exame na janela da
   sua categoria (3 = lápis/tinta/marcadores; 4 = pastel/óleo): cura contra o oráculo de origem, ou fundem-se/saem.
   Só 8 de 29 têm a sua origem do Rebelle como oráculo mais perto (mediana: 4.º lugar).
4. **O oráculo que manda** é o do dono (Rebelle) quando existe; ArtRage/MyPaint/Krita servem as famílias que o
   Rebelle não tem (caneta técnica, monoline, smudge de dedo, rolo, elementos). GIMP fora (dono). Procreate: só doc.

## A FILA (por ordem)

**JANELA 2 — motores** (lista e custos em `03_MOTORES.md` §5; o que a bancada mediu sobe para o topo):
0. **Leis sistémicas medidas na bancada — afectam quase todo pincel, antes das lacunas:**
   (a) **pressão**: o Rebelle desvanece a CARGA a largura cheia; nós estreitamos (8B: rampa 26→17 px; Oil Bristle
   37→22). Medir a lei por ferramenta na rampa_0_1 dos 281 e exprimi-la por preset (⚠️ sem piso de pressão:
   `feedback_pen_pressure_floor_must_keep_the_owners_hand_range`). (b) **o traço fino** (5 px) sai claro e
   pontilhado onde o do Rebelle é sólido (8B cobertura 0,32 × 0,80). (c) **as pontas**: cápsula nossa × corte
   recto/desfiado das cerdas; o «brilho de tubo» do Oil Bristle. (d) **sobre a faixa molhada** o nosso incha numa
   lente clara.
1. L1 presets dos 4 meios + navegador + miniatura (CAP-22, ≈45 pincéis; a decisão mora em
   `paint_brush_sections.rs:116`) · 2. L2 a mão (pressão/tilt) nos 3 meios · 3. L3 lâmina que gira com inércia e
   memória (C38) · 4. L7 modos de mistura nos presets (Effect do Rebelle: 7 fotografados) · 5. L4 jitter completo ·
   6. L11 dinâmica de cor · 7. L6 dual brush · 8. L5+L12 fim do traço e velocidade (corte do `stroke.rs`, 696/700) ·
   9. L8 tecto por traço / wet-burnt edges · 10. L19 ponta RGBA · 11. o resto do `03_MOTORES` §5.
   Lateral (sem bloquear): Clip Studio 1.13 sob Wine (entra em «trial limited» sem conta; falta matar o pop-up web
   do arranque — `02_ORACULOS.md`); mais presets do ArtRage pelo `lista_artrage.tsv`.

**JANELA 3** — desenho, tinta, aerógrafo (catálogo A, B, C, D, E, J): os novos + o exame dos existentes destas
categorias (decisão 3). Ordem P1 do catálogo §0.
**JANELA 4** — pintura, textura, efeitos (F, G, H, I, K, L, M) + refazer Filbert/Palette Knife com L3.
**JANELA 5** — o resto, a biblioteca inteira (matriz final), o fecho batched, handoff, smoke/tutorial PDF.

## Bancada: o essencial para quem chega

- `bash docs/Painter/Pinceis/ferramentas/exame.sh "<preset>" docs/Painter/Pinceis/fixtures/rebelle_bateria/<TOOL>/<prefixo>`
  · `exame.sh --matriz` · `exame.sh --custo "<preset>" 5,40,200`. Preset = nome de fábrica, caminho
  `.ph2d-brush.png`, ou `meio:Digital|Watercolor|Impasto|WetPaint` (só Wet Paint tem ficheiro hoje).
- Custo medido (smoke, carga 4,5–7,4): p95 ≤ 6,1 ms/quadro a 200 px, pico 96 MiB (8B, Oil Bristle).
- Ferramentas sem tinta do Rebelle (Blend, Smudge, Eraser, Clone, Water, Dry, Effect) só têm a cena `sobre`.
- Em 651/766 cenas do Rebelle A = B (só aguarela, guache, sumi-e, spatter, Water/Dry/Smudge/Blend mudam ao secar).

## Aberto da JANELA 1 (com dono)

- Tolerâncias da régua (`exame_regua.py`) ainda provisórias → trocar pela tabela de ruído medida (J2, antes do 1.º
  pincel da J3). Lista das famílias graduadas escrita à mão.
- As pranchas cegas de 6 linhas do 8B e do Oil Bristle ainda sem revisão cega (a de 2 linhas reprovou o Oil Bristle).
