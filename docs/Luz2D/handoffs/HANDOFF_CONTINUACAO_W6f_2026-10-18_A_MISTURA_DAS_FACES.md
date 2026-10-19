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

## 3. Rodadas (`target/w6f/caixa_9.log`, `f3_9.log`; 9′: `caixa_9b.log`, `f3_9b.log`)

| variante | cantos da caixa errados | F3 penumbra (teto 0,062) | texel (152, 245) (gab. 0,142) |
|---|---|---|---|
| antes da W6d | 4 | — | — |
| lei W6d (o padrão) | 7 | 0,0531 | 0,140 |
| cunha | 6 | 0,0689 ✗ | 0,173 |
| W6d + 9 | 6 | 0,0523 | 0,128 |
| **cunha + 9** | **3** ✗ (≤ 1) | **0,0539** | **0,1423** |
| cunha + 9a / + 9ac | 3 | 0,0689 | 0,173 |
| cunha + 9b | — | 0,0536 | 0,1423 |
| **9′** (cunha + 9 + os pés das perpendiculares + a direção pela normal do campo com o centro rente) | **7** ✗ | 0,0528 | 0,1423 |

- **A 9 morre no item 1** (3 > 1). A 9a sozinha separa as normais (o aberto, zoom 1 · 3/8, passa a misturar), a 9c não
  muda nenhum canto da caixa, a 9b cura o F3 (a luz do ponto pela face: 0,12 → 0,08 = a lei no ponto).
- Os 3 que ficam: zoom 1 · 4/8 cima-direita (mistura, f2 0,43 contra 0,36, mas a luz da OUTRA face sai 0,01 contra
  1,18, no ponto (+0,25; +0,51) colado ao vértice) · zoom 2,5 · 3/8 baixo-esquerda (o centro a 0,01 texel da fronteira
  do campo, as normais diagonais: o chanfro) e cima-direita (o centro EM CIMA da fronteira, saída 5).
- **A 9′ morre no item 1** (7): com os pontos nos pés das perpendiculares, a luz da outra face sai **0,00** nos cantos
  em que as duas faces estão acesas (2,71 / 2,73 → 1,65 · 1,79 · 2,09): o pé fica a ~0,06–0,4 texel do vértice, DENTRO
  do corpo que o campo dá ali. Two-strikes ⇒ a onda para e prova o modelo (§4). A 9′ está em
  `target/w6f/c3_direta_9linha.wgsl` (fora do git); o código fica com a 9 (desligada).

## 4. O modelo PROVADO — o campo bilinear não tem vértice (`target/w6f/caixa_modelo2.log`)

O iso-0 do campo da borda (a bilinear do 4.º canal, a mesma do `meio`), medido ao longo da normal de cada face
(texels finos; `+` = para fora da face verdadeira) a 0 · ¼ · ½ · 1 · 2 texels do vértice, com a cunha, os 4 vértices × os
8 casos (`arredondado` da `medida_luz2d_w6f`):

| distância ao vértice | 0 | ¼ | ½ | 1 | 2 |
|---|---|---|---|---|---|
| o iso-0 (o típico; os extremos) | −0,2 a −0,6 (ou além de −1) | −0,05 a −0,12 | ±0,06 | ±0,06 | ±0,03 |
| a BOSSA para fora de algumas faces | — | +0,21 | +0,18 · +0,35 | +0,12 · +0,14 | +0,03 |

⇒ junto a um vértice convexo o campo é ~½ texel mais redondo que a geometria, e algumas faces fazem bossa para fora a
½–1 texel dele. Todo ponto de face de um texel de canto fica a < ½ texel do vértice, logo a luz que se marcha dali
(o ponto remapeado sem normal secante; a outra face, sempre pela marcha) lê OUTRO corpo: vaza no F3 (+0,04) e tapa na
caixa (0,00–0,01). A cunha não o cura porque corrige UMA distância por texel da captura, e a bilinear entre centros
continua a arredondar. Os sintomas das W6d–W6f (o chanfro, o «canto do campo», a luz da face) são esta peça.

## ⛔ Recusas MEDIDAS (W6f — não reconstruir sem ler a §3/§4)

| Recusa | Porquê |
|---|---|
| A candidata 9 (normais de um lado + a luz pela face + a fração de dentro) no padrão | 3 cantos errados (o critério é ≤ 1). Passa o F3 com a cunha (0,0539), mas os que ficam são do campo (§4). |
| A 9′ — os pontos das faces nos pés das perpendiculares | 7 cantos: o pé cai no arredondado do vértice, a luz da outra face sai 0,00. |
| A fração na parte de dentro (9c) sozinha | Não muda nenhum canto: a fração da área inteira erra < 0,1 ali. |

## 5. ABERTO (por ordem)

1. **O campo que guarda o vértice** (candidata 10, com o kill-criterion do §2 reaproveitado): o `meio` decide
   «dentro» por uma representação que não arredonda o canto, não pela bilinear de UMA distância. Candidatas a medir no
   mesmo processo: (a) a distância MULTICANAL (o `msdf`: três distâncias e a mediana, a lei clássica para o canto de uma
   fonte); (b) no texel da captura que tem vértice, a cunha das duas retas lida exata no `meio` (o predicado, não a
   distância). A régua é o perfil da §4 (alvo: |iso-0| ≤ 0,06 a ≥ ¼ texel do vértice) e depois a `medida_luz2d_w6f`.
   Só então a 9 (desligada no código) volta a medir-se como variante: com o campo certo, a 9b talvez deixe de ser
   precisa.
2. A família (b) (37, a penumbra dos tangentes): é da W5g, fica fora.

## 6. Prova, superfície de colisão e commits

- Portões por nome com o padrão (`target/w6f/portoes_*.log`, `load` 16 das outras linhas): gpu `c3_grade_fina` 6/6 (o
  F3, o bit-idêntico da grade fina, a região) · app 6/6 (`o_interior_do_opaco` ×2, `o_campo_da_borda`,
  `a_borda_sem_pontos_pretos`, `g6_ponto_neutro` ×2, byte a byte). O padrão não muda: `Canto::DESLIGADO` e `sonda: 0`;
  os overrides apagam os ramos na criação da pipeline.
- Tocam `ph2d-light2d-gpu` (`config.rs` — `Config::sonda`, `Config::canto`, `Canto` —, `lib.rs`,
  `solver_c3_constantes.rs` — `comuns` passa a `[_; 22]` —, `wgsl/c2_rc.wgsl` (`SONDA`), `wgsl/c3_direta.wgsl`,
  `tests/it/c3_grade_fina.rs`) e `ph2d-app-light2d` (`tests/it/medida_luz2d_w6f.rs` novo, `main.rs`,
  `o_interior_do_opaco.rs` — `LADO` passa a `pub(crate)`). Sem contrato congelado, sem schema, sem shell. Quem constrói
  `Config` sem `..` parte com os dois campos novos (nenhum no repo).

## 7. Smoke do Enio

Nada mudou na tela: a cura ficou desligada. O smoke da W6d continua valendo
(`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=6 cargo run -p ph2d-host-desktop --profile smoke`).
O ponto escuro no canto de uma caixa continua lá: é o aberto 1, agora com a causa medida.

## 8. A candidata 10 — o campo que GUARDA o vértice — e o seu kill-criterion (escrito ANTES de medir)

Mecanismo (§4): o `meio` decide «dentro» pela bilinear da distância de 4 texels que, junto a um vértice, carregam retas
DIFERENTES; a bilinear de retas diferentes é uma hipérbole (com o vértice no meio da célula, o iso-0 recua 0,29 da
célula na diagonal). Cura: a decisão pelas duas retas da cunha combinadas DEPOIS (convexo: o mínimo; côncavo: o
máximo), sem interpolar, nos texels a ≤ 2 de um parcial de vértice; no resto, a bilinear de sempre.
- **(A)** o `c3_borda` grava em `V` (rgba16f, a resolução da captura) as duas retas da cunha de cada parcial de vértice
  aceite (a mesma deteção da W6e: `explica_a_3x3` e a área da cunha), sentinela nos outros;
- **(B)** o passe novo `c3_cantos` dá a cada texel a cunha do parcial de vértice mais perto a Chebyshev ≤ 2 (o
  desempate pela ordem da varredura: determinístico), no referencial do texel (`V2`);
- **(C)** no `meio` (só com a borda contínua e `borda_aqui`), quando a bilinear dá |d| < 1, um `textureLoad` de `V2`:
  com cunha, decide por ela.

`Config::borda_cantos` (override `BORDA_CANTOS`; exige `borda_vertice`, de onde tira os limiares). Desligado, `V` e
`V2` não se alocam.

**Régua** (um processo, intercalada): o perfil do iso-0 da §4 (a réplica na CPU lê `V2` também) · a `medida_luz2d_w6f`
e a `medida_luz2d_w6f_f3` com as variantes lei W6d · cunha · cunha + 10 · cunha + 10 + 9 · depois, a candidata que
entrar no padrão, os 5 itens do §1 da W6e e os portões.

**PASSA** (todos): o perfil — |iso-0| ≤ 0,06 a ≥ ¼ texel fino do vértice em todo canto da caixa; e os cinco do §2
(cantos ≤ 1 · F3 verde com o texel (152, 245) a ≤ 0,01 · (a) ≤ 14 / (b) ≤ 42 / anéis sem regressão · bits de fora iguais
· C3 ≤ +5 %). **MORRE**: qualquer um; two-strikes como sempre.

## 9. A candidata 10 — medida (`target/w6f/caixa_10*.log`, `f3_10*.log`, `rodada10.log`)

Reconstrução 1 = (A)(B)(C) do §8. Reconstrução 2 = + (i) a cunha aceita como reta a aresta cheio|vazio do anel (a face
alinhada à grade, sem parciais) · (ii) a luz da OUTRA face também pela face no texel que uma face corta
(`face_da_outra`; a marcha saltava a lasca junto ao vértice: 0,72 contra 0,001) · (iii) o lado da paralela de um lado
pela normal da outra face quando o ponto dela está rente (`longe_de`).

| variante | cantos (≤ 1) | perfil (≤ 0,06 a ≥ ¼) | F3 (≤ 0,062) | (152, 245) (0,142 ± 0,01) | (a) / (b) / anéis / bits | C3 cena 5 / 6 (≤ +5 %) |
|---|---|---|---|---|---|---|
| lei W6d (o padrão) | 7 | ✗ (−0,2…−0,6, bossa +0,35) | 0,0531 | 0,140 | 1 (−0,176) / 37 / 0 / 0 | 3,491 / 2,229 |
| 10, rec. 1 | 3 ✗ | ✗ nos vértices sem cunha | 0,0417 | — | — | — |
| 10 + 9, rec. 1 | 2 ✗ | ✗ | 0,0371 | — | — | — |
| **10 + 9, rec. 2** | **1** ✓ | **✓** (só −0,23 no próprio vértice) | **0,0371** ✓ | **0,124** ✗ | **1 (−0,176) / 38 / 0 / 0** ✓ | **4,296 / 2,933 (+23 % / +32 %)** ✗ |

**MORRE na 2.ª reconstrução** (two-strikes) por dois itens, e os dois têm o mecanismo medido:
- **o texel (152, 245):** a luz da face do ponto, pela face, sai 0,04 contra 0,079 (a média da lei na parte dele);
  a normal medida pelo `pe_na_face` CENTRADO é (−0,22; −0,97) contra (−0,262; −0,965), 2,4° de erro (sem a cunha 0,7°):
  a paralela centrada cruza a outra face, o mesmo mecanismo da 9a, mas pouco o bastante para o teste do cosseno não o
  apanhar. Na penumbra a fração da lâmpada acima da face é sensível a isso;
- **o custo:** é todo do `preenche + borda` (0,53 → 1,22 ms; a fina +0,08 / 0): a varredura da captura INTEIRA no
  `c3_cantos` e as duas escritas de textura inteira (`V`, `V2`). O `meio` com o `textureLoad` a mais não pesa.

Estado: `Config::borda_cantos = false` e `Config::canto = Canto::DESLIGADO` no padrão; o código das duas fica (overrides
eliminam os ramos), `V`/`V2` só se alocam com os cantos, e a lista de passes declara a textura vazia do binding 15 sem
eles (o G5 apanhou-a fora da lista). Portões com o padrão (`target/w6f/portoes3_*.log`, `load` 16–43 das outras
linhas): gpu 9/9 (`c3_grade_fina`, G5, G10) · app 15/15 (`o_interior_do_opaco`, `o_campo_da_borda`,
`a_borda_sem_pontos_pretos`, `g6_ponto_neutro`, `a_vista_renasce`, `os_tetos_da_placa`). O custo do padrão não muda
(lei W6d 3,491 / 2,229 contra 3,499 / 2,245 na W6e).

## ⛔ Recusas MEDIDAS (a 10)

| Recusa | Porquê |
|---|---|
| A 10 + 9 (rec. 2) no padrão como está | +23 % / +32 % na C3, todo nos passes da borda; e o (152, 245) a 0,018 do gabarito (a normal centrada a 2,4°). |
| A 10 sem a 9 | 3 cantos: sem a mistura pelas faces o texel do canto fica com uma face, mesmo com o campo certo. |

## 10. ABERTO (por ordem — substitui o §5)

1. **A 10 + 9 com o código leve e a normal de um lado** (é a candidata a entrar; o kill-criterion do §8 vale tal e
   qual): (a) o `c3_cantos` só onde há vértice (o grupo 8×8 sem parcial de vértice no bloco salta; a `V` escrita só nos
   parciais, a `V2` lida pelo `meio` só onde |d| < 1, que o passe cobre) — o alvo é a borda ≤ +0,17 ms; (b) no texel que
   uma face corta, as normais SEMPRE pelo `pe_de_um_lado` (não só quando a centrada as dá «iguais»). A régua: a
   `medida_luz2d_w6f`, a `_f3`, o `arredondado` e a `medida_luz2d_w6d_borda` (variante 3 = «cunha + 10 + 9»);
   `python3 -I docs/Luz2D/ferramentas/resumo_borda.py <log>` resume-a por variante (reproduz a tabela da W6e).
2. A família (b) (37–38): da W5g, fora.
