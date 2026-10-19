# Handoff de continuação — Luz 2D, W6f: o texel do canto, a mistura das faces (2026-10-18)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `ad5f8fe39c`, por cima da [W6e](HANDOFF_CONTINUACAO_W6e_2026-10-18_O_CANTO.md); sem rebase (é do integrador).

## 1. A sonda de cor do `c3_fina` (o aberto 1 da W6e) — `target/w6f/caixa_sonda.log`, `f3_sonda2.log`

A sonda da W6d §14 tinha sido retirada antes do commit. Agora é um instrumento que fica: `Config::sonda` (`0` = a
luz; o override `SONDA` elimina o ramo na criação da pipeline), o `sonda_cor` do `c3_direta.wgsl`: 1 = o ponto
remapeado − o centro e a fração `f2` · 2 = por onde o `onde_calcular` saiu (`+10` = foi às sementes; 0 não
remapeado · 1 sem candidatos · 3 sem outra face · 4/5 a face 1/2 não se mediu · 6 as normais «iguais» · 7/8 misturou)
e as distâncias das duas faces · 3 = a normal da face 1 e o cosseno entre as faces · 4 = a luz (verde) do ponto, a da
outra face e `f2` · 5 = a normal secante no ponto · 6 = o ponto da outra face. Réguas: `medida_luz2d_w6f` (a caixa: os 4
cantos × os 8 casos do B, lei antiga, o gabarito DECOMPOSTO por face) e `medida_luz2d_w6f_f3` (o F3, 3×3 em volta do
(152, 245), com a lei exata no ponto da placa).

A régua reproduz a W6e: cantos errados antes da W6d **4** · lei W6d **7** · cunha **6**.

| mecanismo (sonda 2) | erros (17 = 7 casos × 3 variantes) | o que a sonda lê |
|---|---|---|
| **6 — as normais «iguais»** | 13 (zoom 1 · 3/8 W6d; 1 · 4/8 e 1 · 6/8 e 2,5 · 4/8 nas 3; 2,5 · 0/8 W6d e cunha; 2,5 · 3/8 cunha) | as duas faces achadas (direções a > 60°), mas o `pe_na_face` mede cosseno **0,63–0,97** entre as normais (a verdade: 0); o teste «mesma face» corta a mistura e o texel fica com UMA face. As luzes das faces estão certas (sonda 4: 1,19 / 0,012 contra o gabarito por face 1,182 / 0,012). |
| 0 — não remapeado | 2 (2,5 · 3/8: antes da W6d cima-direita, W6d baixo-esquerda) | o centro cai fora pelo campo (o chanfro): é do campo; com a cunha o mesmo texel passa à saída 6 |
| 5 — a face 2 não se mede | 2 (2,5 · 3/8 cima-direita, W6d e cunha) | o `pe_na_face` da outra face perde os cruzamentos (a vizinha a ±w já cruza a outra face) |

O mecanismo do 6: o `pe_na_face` mede a tangente por duas paralelas a `P0 → b` a ±w (`w = min(h, d/2)`); no texel do
canto o vértice fica dentro dessa largura e uma das paralelas cruza a OUTRA face.

**O F3 é outro mecanismo** (texel (152, 245), saída 8: misturou). Gabarito 0,142 = aresta 1 0,302 × 0,29 + aresta 2
0,079 × 0,71. Placa: fração 0,27–0,28 · a outra face 0,30 (lei no ponto 0,3029) · o ponto remapeado na aresta 2
(deslocamento (−0,31; −0,62), o mesmo nas duas): **0,08 sem a cunha · 0,12 com ela, e a lei exata NESSE ponto é
0,0798**. A normal secante ali é 0 (o «canto»), por isso a luz sai da marcha cheia do ponto, e o campo bilinear
arredonda o vértice convexo para a marcha: os raios passam por trás da face. Com o campo da W6d, a bojo no vértice
(+0,57 contra +0,50) tapava-os por acaso. A normal da face, porém, está medida certa (sonda 3 (−0,26; −0,96); a aresta 2
tem (−0,262; −0,965)).

## 2. A candidata 9 — a FACE no canto — e o seu kill-criterion (escrito ANTES de curar)

Três peças da mesma família, num bloco:
- **9a — as normais de UM lado:** quando a outra face foi achada pela direção mas o `pe_na_face` centrado as dá
  «iguais» (cosseno ≥ ½) ou não as mede, medir de novo cada face com as duas paralelas do lado OPOSTO à outra face (o
  ponto `P0` e `P0 + h` afastado dela). O texel que continua «igual» é da mesma face, como antes.
- **9b — a luz do ponto pela face:** no ponto remapeado em que a normal secante desiste (0, o canto), a visibilidade
  pela face com a normal medida no `onde_calcular` (a do `pe_na_face`), em vez da marcha cheia.
- **9c — a fração na parte de DENTRO:** no texel que uma das faces corta, a fração `f2` conta só a parte do texel
  dentro das duas faces (8×8 subamostras pelas duas retas medidas), como o gabarito; nos outros, a área de sempre.

`Config::canto` (os três interruptores; overrides `CANTO_*`), desligado no padrão até passar.

**Régua** (um processo, intercalada): `medida_luz2d_w6f` e `medida_luz2d_w6f_f3` com as variantes lei W6d · cunha ·
W6d + 9 · cunha + 9 (e as peças sozinhas na lista); depois, com a candidata no padrão, os 5 itens do §1 da W6e
(`medida_luz2d_w6d_borda`: (a), (b), anéis, bits, custo) e os portões (B, F3, o campo, o interior na difusão).

**PASSA** (todos, a candidata que entra no padrão):
1. cantos da caixa (lei antiga, 4 × 8 casos) **≤ 1** errado (de 7; antes da W6d 4) e o aberto (zoom 1 · 3/8
   cima-direita) a ≤ 0,1 do gabarito 0,662;
2. o F3 verde: penumbra ≤ 0,062, o texel (152, 245) a ≤ 0,01 do gabarito 0,142, o vértice girado a ≤ 0,01 de 0,207;
   o portão B e o do campo verdes; o interior na difusão sem regressão (`TETO_ERRADOS_DIFUSAO` = 2);
3. (a) ≤ 14 com o pior ≥ −0,25 · (b) ≤ 42 · anéis sem regressão sobre a lei atual em todo zoom;
4. bits de fora dos corpos IGUAIS aos de antes (0 em 24);
5. C3 ≤ **+5 % sobre a lei atual** nas cenas 5 e 6 (o mínimo de 7 intercaladas, no mesmo processo).

**MORRE**: qualquer um dos cinco. Bater na 2.ª reconstrução sem passar ⇒ PARE e prove o modelo (two-strikes).
