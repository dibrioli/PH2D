# Handoff de continuação — Luz 2D, W6d: o disco em zoom (o F5 do plano) (2026-10-18)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `ad5f8fe39c`, por cima da [W6c](HANDOFF_CONTINUACAO_W6c_2026-10-17_O_PISCAR_DO_RENASCIMENTO.md) (HEAD `fd4d1896a3`, não
> integrada). Placa RTX 5060 Ti. Sem rebase (é do integrador).

## 0. O F5 de HOJE (medido antes de qualquer cura — `medida_luz2d_w6d`, `target/w6d/rodada0.log`)

As fotos do dono (`fotos_W5d/dono_disco_zoom_*.png`) são da W5d (o miolo pelo exterior mais próximo: a pizza). Depois
vieram a W5g (o pé + a face) e a W5h (o miolo por DIFUSÃO). A régua dos anéis em 17 zooms contínuos de 0,6 a 4 (três
fases, sete anéis, disco e elipse polígonos + o disco de imagem), contra o gabarito da difusão:

- a luz só muda por OITAVA da grade (0,6–1,09 · 1,22–2,21 · 2,49–4 dão os mesmos números — confirma a W5g);
- o produto: **×0 cruzamentos a mais em todo zoom**; TV ≤ +0,59 (polígonos, 0,97 do raio, a oitava grossa) · ≤ +0,22
  (imagem); erros > 10 %: 5 (0,202, o anel 0,97, uma fase) · imagem 3 (0,105, a oitava do meio);
- o controlo (a lei antiga): 21 926 errados, ×4–6, TV +3,7–4,6 — a régua vê a escada quando ela existe.

**Fotos** (`target/fotos_w6d/`, a luz recebida píxel a píxel): o miolo sai em gradiente liso em 1 · 1,6 · 2,5 · 4. O que
SOBRA: pontos ESCUROS isolados colados à borda do lado aceso (e um grão junto às pontas da passagem). A régua dos anéis
não os via (o anel de fora fica a 2,5 texels; eles estão a < 1).

## 1. O mecanismo (censo da borda, `medida_luz2d_w6d_borda`, `target/w6d/borda0.log`)

O censo: cada texel fino a menos de 1,5 texel da borda do disco, o valor no CENTRO contra o gabarito, longe das
passagens. 3 zooms × 8 fases (polígono) + o disco de imagem; o produto e as peças desligadas, intercalados:

| | escuros (< −0,10) na faixa [−0,5; 0) por fase | o pior escuro |
|---|---|---|
| produto | 1–14 (crescem com o zoom: 0,8 → 1–6 · 3,2 → 8–14) | −0,90 a −0,96 (o texel PRETO), d ≈ −0,05 |
| sem borda exata (o iso-½) | 0–6 | −0,10 a −0,20 (raro −0,84) |
| sem pé · sem face · sem miolo | como o produto | −0,9 |

⇒ os pretos são texels de DENTRO, rentes à borda (d ≈ −0,02 a −0,09 texel), e vêm da **borda exata**. A borda exata é
UMA RETA POR TEXEL da captura (`c3_borda.wgsl`), e o `dentro_exato` (`c2_rc.wgsl`) usa a do texel onde o ponto cai: as
retas de dois texels vizinhos não se encontram ⇒ o predicado «dentro» é DESCONTÍNUO ao longo da borda e não é monótono
num segmento de dentro para fora. A bisseção do ponto de fronteira (`na_fronteira`/`cruzamento`) cai numa FRESTA «fora»
entalada no corpo; de lá a marcha rumo à lâmpada entra no corpo logo ⇒ visibilidade 0 ⇒ o texel preto (e a bilinear do
receptor espalha-o pelo chão vizinho: o ponto escuro da foto).

## 2. A cura candidata e o kill-criterion (escrito ANTES de medir)

**A borda CONTÍNUA** (`Config::borda_continua`, override `BORDA_CONTINUA`): o uso que o próprio AAEDT prevê — o campo
de DISTÂNCIA com sinal interpolado. No `dentro_exato`, os 4 texels da bilinear em volta de `p`; cada um dá a distância
com sinal de `p` à SUA reta (`dot(p − centro, n) − recuo`), e o «dentro» é a mistura bilinear delas ≥ 0. Os texels
cheios/vazios da faixa (com um parcial na 3×3) recebem no `c3_borda` a reta do parcial vizinho mais próximo,
transladada para o centro deles (o sinal conferido: cheio ⇒ ≥ 0 no centro, vazio ⇒ < 0; senão, sem reta); sem reta,
±1. Um fio (parcial sem gradiente) entre os 4 ⇒ o iso-½ de hoje. Contínuo por construção; numa borda RETA todas as
retas coincidem e a mistura é a reta — a mesma resposta exata de hoje. O `normal_de_fora`/`reta_da_borda` não muda (só
lê os parciais).

**Régua** (um processo, as variantes intercaladas): o censo da borda (§1) e os sete anéis (§0) em 0,8 · 1,6 · 3,2 × 8
fases (polígono) + o disco de imagem; produto × contínua × sem borda exata (a referência).

**PASSA** (a contínua, contra o produto da MESMA corrida):
1. escuros < −0,10 nas faixas [−0,5; 0) e [0; 0,5): **≤ 10 % dos do produto**, somados; e o pior escuro ≥ −0,25 em toda
   configuração;
2. os anéis sem regressão: 0 errados a mais, ×0, TV ≤ a do produto + 0,05 em cada oitava;
3. verdes: `as_formas_redondas_nao_saem_em_facetas`, `o_disco_de_imagem_nao_sai_em_facetas`,
   `o_interior_com_o_rebote_nao_sai_em_riscos`, `junto_a_um_canto_a_face_le_a_sua_luz`, `o_interior_do_opaco*`,
   `a_face_acesa_da_parede`, `o_miolo_das_formas*`;
4. FORA dos corpos (os texels a ≥ 1 texel fino de todo corpo da cena 5, sem rebote) os bits IGUAIS aos do produto;
5. o custo da C3 (preenche + visibilidade + fina + borda, o mínimo de 7, cena 5, 1080p) ≤ +5 %.

**MORRE**: escuros > 50 % dos do produto · algum portão do item 3 vermelho · os bits de fora mudam. **Nem passa nem
morre** (entre 10 e 50 %, ou o custo acima de 5 %): não entra; vira recusa medida e a próxima candidata é a busca
robusta (a bisseção que confere a saída pela normal).

**Controlo**: o produto de hoje tem de REPROVAR o item 1 na mesma corrida (já reprova: 1–14 pretos por fase).

## 3. Rodada 1 — a contínua por 4 leituras: NÃO ENTRA (`target/w6d/decide1.log`, `lista.log`)

Pela letra do §2: escuros nas faixas rentes 179 → 56 (31 %, o item 1 pede ≤ 10 %), o pior −0,97; os bits fora dos corpos
IGUAIS (0 em ~3,7 M por configuração, 24 configurações); o custo da C3 3,41 → 8,21 ms (cena 5) e 2,21 → 5,35 (cena 6),
intercalado, mínimo de 7, mas com `load` 54–73. ⇒ **nem passa nem morre: não entra** (e o custo sozinho reprovaria).

A lista dos pretos (zoom 1,6, 8 fases) separa DUAS famílias:

| | (a) rentes no lado ACESO (< 86° da lâmpada), a placa ≈ só o ambiente (0,03 contra ~2,3) | (b) nas PONTAS (≥ 86°: a penumbra dos tangentes) |
|---|---|---|
| produto | 43 | 16 |
| contínua (4 leituras) | **2** | 16 |
| sem borda exata | 2 | 27 |

⇒ o mecanismo do §1 CONFIRMADO para a família (a) — os pontos escuros das fotos — e a (b) é OUTRO (a penumbra dos
tangentes, a W5h já a mediu: o pior texel do anel −27 %, «é a W5g»), que nenhuma das três toca.

## 4. A candidata 2 e o seu kill-criterion (escrito ANTES de medir)

**O campo de distância** (o mesmo `BORDA_CONTINUA`, outra forma): o `c3_borda` grava no 4.º canal a DISTÂNCIA COM SINAL
no centro do texel (o parcial: −recuo; o fio: cobertura − ½; o cheio/vazio da faixa: a do parcial vizinho, a reta
emprestada; o resto: ±1) e o `meio` decide «dentro» pela bilinear dela ≥ 0 — UM `textureGather` (o mesmo preço da
cobertura de hoje), contínua por construção, exata numa borda reta (a bilinear reproduz uma função linear).

**PASSA** (contra o produto da MESMA corrida, 3 zooms × 8 fases + o disco de imagem): (1a) os escuros da família (a)
≤ 10 % dos do produto e o pior deles ≥ −0,25; (1b) os da família (b) ≤ os do produto + 2; (2)–(4) como no §2; (5) o
custo da C3 ≤ +5 % (intercalado; com `load` > 5 o número vai com o `load` ao lado e repete-se se passar de +5 %).
**MORRE**: (a) > 50 % · um portão vermelho · os bits de fora mudam · o custo > +20 %. Entre os dois: não entra.

## 5. Rodada 2 — o campo: NÃO ENTRA sozinho (`decide2.log`); o 2.º mecanismo (`lista2–4.log`)

Contra o §4: (1a) 139 → 12 (8,6 %) ✓ mas o pior dos 12 ainda −0,97 ✗ · (1b) 40 → 38 ✓ · anéis iguais ou melhores (zoom
0,8: 10 errados → 0) ✓ · bits de fora 0 ✓ · custo C3 cena 5 3,37 → 3,68 ms (+9 %; a borda 0,37 → 0,71, a fina igual;
`load` 10–49) ✗ ⇒ não entra.

Os 12 que sobram (zoom 3,2, d −0,06 a −0,10, a placa = o ambiente): nenhuma etapa os move — sem face, sem face pelo
ponto, sem pé, sem miolo, sem média no eixo: os MESMOS texels. O que os move: **a marcha da luz direta dos texels de
FORA pela borda exata também** (`Config::borda_fora`, `BORDA_FORA`). O campo (como a reta por texel) erra ~0,1 texel; um
texel rente que ele põe FORA marcha pelo iso-½ (W5d: «a C3 desliga-a na luz direta dos texels de FORA»), e para o iso-½ ele
está DENTRO: a marcha nasce dentro do corpo e sai tapada. Lista, zoom 3,2, 8 fases + imagem:

| | (a) polígono | (b) polígono | (a) imagem | (b) imagem |
|---|---|---|---|---|
| produto | 69 | 14 | 8 | 11 |
| campo | 5 | 10 | 3 | 12 |
| **campo + fora** | **0** | **5** | **0** | 12 |
| exata (reta por texel) + fora | 7 | 14 | 0 | 12 |
| sem borda exata | 4 | 17 | 0 | 11 |

⇒ os DOIS mecanismos: a fresta da reta por texel (cura: o campo) e os dois predicados (cura: o mesmo na marcha de fora).
A M4 da W5d («equivalente: escolha de custo, a mesma foto») NÃO era equivalente — a foto não via um texel rente.

## 6. A candidata 3 — o campo + fora — e o seu kill-criterion (escrito ANTES de medir)

O campo (§4) + `borda_fora`, e o `c3_borda` com a vizinhança 12×12 do grupo em memória partilhada (a mesma conta, menos
leituras). Régua, variantes e itens (2)–(4) como no §4. **PASSA**: (1a) a família (a) ≤ 10 % do produto E o pior dela
≥ −0,25 · (1b) ≤ produto + 2 · (5) o custo da C3 ≤ +5 % (intercalado, o mínimo de 7, cenas 5 e 6; com `load` > 5 o
número vai com o `load` e repete-se se passar de +5 %). **MORRE**: (a) > 50 % · portão vermelho · bits de fora · custo
> +20 %. Entre: não entra.

## 7. Rodada 3 — o campo + fora: MORRE pelo item 4 (`decide3.log`, `load` 3–20)

(1a) 139 → **0** ✓ · (1b) 40 → 27 ✓ · anéis iguais ou melhores ✓ · o produto com as MESMAS linhas da rodada 2 (o censo
e os anéis idênticos) · **bits de fora: 254 396 diferentes em 24 configurações** (~0,3 %: a marcha de TODO texel de fora
pela borda exata desloca as arestas das sombras que passam rente a qualquer corpo) ✗ ⇒ MORRE. Custo C3 cena 5 3,49 →
3,81 (+9 %), cena 6 2,29 → 2,55 (+11 %) — só a borda (0,47 → 0,81; a fina igual: o `fora` não custa). ⛔ O bloco 12×12 em
memória partilhada no `c3_borda` é MAIS LENTO: o produto 0,371 → 0,467 ms, o campo 0,712 → 0,805 (recusado; volta a
leitura direta).

## 8. A candidata 4 e o seu kill-criterion (escrito ANTES de medir)

O campo + **fora só no texel RENTE**: o texel que a borda exata põe fora marcha pela borda exata só se o iso-½ o põe
DENTRO (a fresta entre os dois predicados — o 2.º mecanismo); todos os outros como hoje ⇒ os bits de fora iguais por
construção. Na mesma corrida, um estudo do custo do `c3_borda` (instrumento, sai depois): o campo inteiro × só as 8
leituras dos vizinhos (sem a reta deles) × sem vizinhos (±1). Itens e limiares: os do §6 (o item 4 inteiro: 0
diferentes). Se só o custo falhar, a próxima volta é o custo, com o estudo na mão.
