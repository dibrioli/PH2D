# ESTADO da missão Pincéis (a FILA viva; ≤ 8 KB)

> Toda janela lê isto primeiro e actualiza-o antes de sair. Contrato: [`00_MISSAO.md`](00_MISSAO.md).
> Última escrita: JANELA 2, 2026-10-20. O detalhe de cada janela: `HANDOFF_JANELA_<n>.md`.

**Quem é quem (20/10):** JANELA 2 = `ph2d-d2` · JANELA 3 = `ph2d-94` · JANELA 4 = `ph2d-d1` · JANELA 5 = `ph2d-fa`.
O nome muda se a janela reiniciar (confirme por `ListAgents` antes do sinal).

## Feito

- **JANELA 1** ([handoff](HANDOFF_JANELA_1.md)): a bateria (`bateria.json`, fonte única), os oráculos fotografados
  (Rebelle 281, ArtRage 35, MyPaint 196 CC0, Krita 144), o catálogo (112 pincéis, `01_CATALOGO.md`), a bancada
  (`exame.sh`, régua, matriz, pranchas legível/cega, custo), `03_MOTORES.md`, a fábrica 31 → 28.
- **JANELA 2** ([handoff](HANDOFF_JANELA_2.md)): TODAS as lacunas de motor da fila — L1 L2 L4 L5 L6 L7 L8 L9 L11
  L12 L13 L18 L19 (L3 refutada como motor) — e as 4 leis sistémicas (pressão → carga, traço fino, ponta recta e
  desfiada + relevo que não cresce com o tamanho, a lente sobre a molhada), cada uma com portão red-first, mutação,
  custo e revisão por um agente que não a fez. As leis medidas: [`06_LEIS_DAS_LACUNAS.md`](06_LEIS_DAS_LACUNAS.md);
  o que cada motor faz agora: `03_MOTORES.md` §4 (linhas ✅) e §7/§7.1.

## Decisões de produto (com o porquê)

1. **Os 3 do P53 saem** (veredito do dono). Voltam SÓ na JANELA 4 e só se passarem o EXAME com o Rebelle ao lado.
   ⭐ J2: a lâmina do Rebelle NÃO gira — é FIXA na tela no eixo do preset (Flat Oily α 86°, 39×8 px; 06 §1); o P53
   errou o EIXO (0°). Refazê-las é `dab_angle_deg` + a ponta recta e desfiada das cerdas (§7.1), não motor novo.
2. **A matriz é TRIAGEM; o veredito é a prancha + a revisão cega.** «Claramente longe» = **d ≥ 0,596**; série graduada
   = FAMÍLIA (fora ≥ 0,596; cada degrau dentro ≥ 0,40). Escala congelada em `matriz/escala.json`.
3. **Os «perto» da fábrica** (J1: 13 de 28) passam pelo MESMO exame na janela da sua categoria (3 = lápis/tinta/
   marcadores; 4 = pastel/óleo): cura contra a origem, ou fundem-se/saem. A lista nova está em `matriz/vizinhos.md`
   (refeita na J2 com as fotos certas e a fábrica de hoje).
4. **O oráculo que manda** é o Rebelle quando existe; ArtRage/MyPaint/Krita para o que ele não tem. GIMP fora (dono).
5. **A cauda do taper VOLTA, só no Digital** (J2, `ed926fa0eb`), contra a ordem de 10/08: os três preços que a
   tiraram já não existem (o traço pinta num PLANO próprio, UM desfazer, só onde age) e as canetas do catálogo
   (C-4 C-6 D-1 D-2) afilam as duas pontas. ⚠️ **Perguntar no smoke do dono.**
6. **Dual brush** só no Digital e no Wet (J2); na Watercolor e no Impasto esmaece (nenhum pincel do catálogo o pede).
7. **Fórmulas dos modos = W3C em sRGB** (provado contra o Rebelle/Krita); o Saturation/Linear Burn/Dodge do Rebelle a
   10× não seguem fórmula nenhuma — não se perseguem.

## A FILA (por ordem)

**JANELA 3** — desenho, tinta, aerógrafo (catálogo A, B, C, D, E, J): os novos + o exame dos existentes destas
categorias (decisão 3). Ordem P1 do catálogo §0. Os motores pedidos por estas categorias EXISTEM todos (03 §4 ✅):
presets Digital/Watercolor/Impasto em ficheiro (L1), cartão Pen (L2), modos + «On empty» (L7), tecto por traço e
orlas (L8), cauda e fall-off (L5, só Digital), velocidade (L12), ponta de píxel (L18), grão com modos (L9).
**JANELA 4** — pintura, textura, efeitos (F, G, H, I, K, L, M) + refazer Filbert/Flat Oily/Knife (decisão 1); dual
(L6), N imagens (L19), jitter completo (L4), cor secundária (L11) estão prontos. Abertos de pincel (não de motor)
que a J2 mediu: a rampa 0→1 do Oil Bristle esbate em fiapos (cobertura 0,69 × 0,83 do Rebelle); o azul sobre o
vermelho molhado não puxa o roxo do Rebelle (K–M do óleo).
**JANELA 5** — o resto, a biblioteca inteira (matriz final), o fecho batched (suíte + clippy 1×), handoff, smoke/
tutorial PDF.
Lateral (sem bloquear): Clip Studio 1.13 sob Wine (falta matar o pop-up web do arranque — `02_ORACULOS.md`); mais
presets do ArtRage pelo `lista_artrage.tsv`; o dual da Watercolor se um pincel novo o pedir.

## Bancada: o essencial para quem chega

- `bash docs/Painter/Pinceis/ferramentas/exame.sh "<preset>" docs/Painter/Pinceis/fixtures/rebelle_bateria/<TOOL>/<prefixo>`
  · `--matriz` · `--custo "<preset>" 5,40,200`. Preset = nome de fábrica ou caminho `.ph2d-brush.png` de QUALQUER
  meio (L1); `meio:<X>` continua a valer. Exemplos por meio: `fixtures/presets_exemplo/`; de teste: `presets_teste/`.
- ⭐ A régua usa agora o RUÍDO MEDIDO de cada métrica (`04_BANCADA.md` §3.1, J2) — o ✓/✗ do `compara` vale.
- Fotos novas por lacuna (cantos, zigue-zague, velocidade, efeitos sobre cinza…): `fixtures/*_lacunas/`, gestos em
  `ferramentas/bateria_lacunas.json`. Uma foto do Rebelle só vale estável (`foto_estavel.py`).
- Custo (smoke): p95 ≤ ~9 ms/quadro a 200 px nos pincéis medidos (Oil Bristle 8,6; Dabber 16,0 a load 18 —
  re-medir calmo); RSS ≤ ~100 MiB; o Dabber a 1000 px 1,2 GiB (era 4,3).
- ⚠️ A fatia da linha (24 G) conta o `/tmp` (tmpfs): apague as grelhas de variantes quando acabar.
- Em 651/766 cenas do Rebelle A = B (só aguarela, guache, sumi-e, spatter, Water/Dry/Smudge/Blend mudam ao secar).

## Aberto (com dono)

- As pranchas cegas de 6 linhas do 8B e do Oil Bristle ainda sem revisão cega (J3 faz a do 8B, J4 a do Oil Bristle).
- A régua dos ruídos da pressão e da mistura: ver `04_BANCADA.md` §3.1 (o que ficou provisório e porquê).
