# ESTADO da missão Pincéis (a FILA viva; ≤ 8 KB)

> Toda janela lê isto primeiro e actualiza-o antes de sair. Contrato: [`00_MISSAO.md`](00_MISSAO.md).
> Última escrita: JANELA 3, 2026-10-21. O detalhe de cada janela: `HANDOFF_JANELA_<n>.md`.

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

- **JANELA 3** ([handoff](HANDOFF_JANELA_3.md), HEAD `c9087dc183`): a fábrica por família nos 4 meios
  (`tests/it/familias/`, `04_BANCADA` §9), `BRIEFING_FAMILIA.md`, as vezes (`vez_pesada.sh`, `vez_do_app.sh`), o
  tecto de tamanho por preset. As 7 famílias A–E, J examinadas: **6 aprovados novos** (Hard Round, Soft Round,
  Pixel 1px, Glaze Round, Flat Block-in, Mapping Pen — Digital); os 12 pré-existentes curados; 20 reprovados em
  estudo (`EM_ESTUDO`, sem ficheiro); 7 fundidos. Fábrica 34 (Wet 27, Digital 7). Relatórios `exames/<família>.md`.

- **JANELA 4** (a meio, 21/10, base `823c6e1b2b`): uma família por agente — 1.ª onda `pastel_carvao`,
  `oleo_acrilica`, `aguarela`, `mistura`, `texturas`; 2.ª onda `guache`, `efeitos`. Dono de cada recurso de motor
  por família (para dois agentes não o fazerem de duas maneiras): troca molhada × molhada e entrada clara → óleo ·
  fibras/orla/backrun/escorridos → aguarela · gota do pó, grão no tecto → pastel · arrastar/levantar/reabrir →
  mistura · dispersão, carimbo, cor por carimbo → texturas (que reexamina também J-2, J-3, J-5).

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
8. **J3: o TECTO DE TAMANHO por preset (o Max Size do Rebelle) ENTRA** — contra a recusa da J1. Pedido a 200 px, o
   HB do Rebelle pinta ~41 px; o nosso pintava uma nuvem de bolhas de 200 px que nenhum lápis pinta
   (`pranchas/grafite/graphite_hb.png`, tamanhos). O produto de referência do dono decide a lei.
9. **J3: a rampa de pressão fica a do DONO, não a do Rebelle.** O Rebelle entra mais tarde e enche mais cedo; o piso
   que o imitava foi revogado no smoke de 18/10 (a mão dele vai de 0,24 a 0,86). ⇒ a diferença «a pressão entra mais
   tarde/enche mais cedo» numa revisão cega é divergência DECIDIDA, não defeito; qualquer outra diferença de pressão
   continua a reprovar.
10. **J3: o 2H é o HB com menos pigmento** (0,75 do tom do HB). O 2H do Rebelle só difere pela cor de omissão do
    preset (cinza 149 × 115), que os nossos presets não levam; um 2H real é mais claro à mesma mão.
11. **J3: o Round Pressure (A-3) funde-se no Hard Round.** Com a opacidade por dab do Krita feita, fica a 0,286 do
    Hard Round (degrau < 0,40) e o revisor cego não os separa nos tamanhos nem no «sobre»; a diferença (a opacidade
    também pela pressão) é uma linha do cartão Pen no Hard Round, não outro pincel.
12. **J3: «diferença VISÍVEL»** = confiança média/alta, ou repetida por DOIS revisores novos. A de confiança baixa
    que revisores seguidos CONTRADIZEM (Mapping Pen, 6 rondas) não reprova — senão o exame não fecha.
13. **J3: a matriz manda nas fusões das canetas duras.** Studio Pen (0,273 do Hard Round) e Technical Pen (0,33 do
    Pixel 1px) são esses pincéis com outra curva/tamanho — não entram; a curva em S fica como linha do cartão Pen.
14. **J3: os grafites NÃO saem da regra do Spacing 0,1 + Continuous** (ordem do dono, 18/10). As bossas da borda
    do 8B do Rebelle que a regra alisa são divergência DECIDIDA. Régua MEDIDA na foto × revisor: vale a foto.
15. **J4: a regra da rampa de pressão NÃO reprova a entrada tardia do oráculo.** O Flat Bristle dá 0,202 na barra
    0,18 só porque o Rebelle não pinta abaixo da pressão 0,2 — copiá-lo é o piso que a decisão 9 proíbe. A régua da
    rampa compara-se acima do início do oráculo (a mudar na lei, aditiva, quando um pincel a pedir).
16. **J4: no conflito entre uma foto nova do Rebelle e um número antigo gravado num portão, vale a foto** (Square
    Hard: o cruzamento mais escuro que a revisão pede × o 0,58 do portão das mídias). O número re-mede-se na foto.
17. **J4: o Dabber fica como está** (curado, não aprovado); fiel ao Rebelle cairia a 0,55 do nosso 8B. A J5 decide
    com a matriz final. Acrylic Round (→ Hard Round) e Rigger (→ Hard Round) fundidos; Wet Blend Round não entra
    (o Soft Round já pinta o Wet Paint do Krita, 366/454).
7. **Fórmulas dos modos = W3C em sRGB** (provado contra o Rebelle/Krita); o Saturation/Linear Burn/Dodge do Rebelle a
   10× não seguem fórmula nenhuma — não se perseguem.

## A FILA (por ordem)

**Fila de MOTOR da J3** (o que destrava cada reprovado de A–E, J; [HANDOFF_JANELA_3](HANDOFF_JANELA_3.md) §fila):
a J4 faz os que os seus pincéis também pedem (o Wet: tinta molhada × molhada, entrada clara, fibras, pó); a J5 os
restantes e reexamina os `EM_ESTUDO`.
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

- A prancha cega do Oil Bristle ainda sem revisão cega (J4). A do 8B: feita na J3 (`exames/grafite.md`).
- Para o smoke do dono: a cauda do taper (decisão 5); o tecto de tamanho por preset (decisão 8); os 4 grafites e
  as 3 tintas pré-existentes ficaram «curados, não aprovados» (decisão 3) — o que falta está no relatório.
- A régua dos ruídos da pressão e da mistura: ver `04_BANCADA.md` §3.1 (o que ficou provisório e porquê).
