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

## 9. Rodada 4 — campo + fora só no rente: a QUALIDADE passa, o custo não (`decide4.log`, `load` 2)

(1a) 139 → **1** (o pior −0,176) ✓ · (1b) 40 → 37 ✓ · anéis sem regressão ✓ · bits de fora **0** em 24 ✓ · o produto
idêntico à rodada 2 · custo C3 cena 5 3,394 → 3,711 (+9,3 %), cena 6 2,211 → 2,457 (+11 %) ✗ — a fina até desce
(2,42 → 2,39 · 1,36 → 1,26). Estudo do `c3_borda` (cena 5): produto 0,374 · só as 8 leituras dos vizinhos 0,512 · sem
vizinhos 0,403 · o campo inteiro 0,715 ⇒ as leituras +0,14 ms, a RETA de cada vizinho (o Sobel dele, 5×5) +0,20 ms.

## 10. A candidata 5 (só custo) e o seu kill-criterion (escrito ANTES de medir)

A reta emprestada sem o Sobel do vizinho: o texel da faixa tira a NORMAL do seu próprio 3×3 (as leituras que a busca do
vizinho já faz) e, de cada parcial vizinho, só a cobertura (o recuo pela área com essa normal, transladado). Itens e
limiares: os do §6 (0 bits de fora; (1a) ≤ 10 % e o pior ≥ −0,25; o custo ≤ +5 %).

## 11. Rodada 5 — a reta emprestada barata: a qualidade igual, o custo ainda acima (`decide5.log`, `load` 1,5)

(1a) 139 → 1 (−0,176) · (1b) 36 · anéis sem regressão · bits 0 · o produto idêntico — a mesma qualidade da rodada 4. Custo
C3 cena 5 3,391 → 3,605 (+6,3 %), cena 6 2,189 → 2,382 (+8,8 %) ✗; a borda 0,376 → 0,637. **Candidata 5b** (só custo, o
mesmo critério do §10, escrito antes de medir): o `c3_borda_quad` — 2×2 texels por invocação, o bloco 4×4 lido uma vez,
o bloco sem parcial grava ±1 sem conta; o produto continua no `c3_borda` (uma função por texel partilhada, `linha_de`).

## 12. Rodadas 6–8 — o custo da borda (só custo: as linhas do censo e dos anéis IDÊNTICAS à rodada 5 em todas)

| `c3_borda` da contínua (cena 5, 1080p, o mínimo de 7, `load` 1,5–2) | borda | C3 |
|---|---|---|
| produto (a reta só nos parciais) | 0,374 | 3,389 |
| rodada 5: 9 leituras soltas por texel | 0,637 | 3,605 (+6,3 %) |
| ⛔ rodada 6: 2×2 texels por invocação (o bloco 4×4) | 0,748 | 3,706 |
| ⛔ rodada 7: a 3×3 por 4 `textureGather` | 0,888 | 3,869 |
| ✅ **rodada 8: 4 `textureGather` e, sem parcial na 3×3, ±1 SEM listas** | **0,498** | **3,471 (+2,4 %)** — cena 6 2,193 → 2,248 (+2,5 %) |

⇒ o custo não era a leitura: era a `array<f32, 9>` indexada em laço (memória local) para TODO texel. O caminho rápido
vetorial deixa as listas só para a faixa. **A candidata 5 (o campo + o fora só no rente + a reta emprestada barata + o
caminho rápido) PASSA os itens 1, 2, 4 e 5 do §6**; o item 3 (os portões) com ela ligada por omissão.

## 13. Item 3 do critério — os portões com a cura ligada (`739fb6cfb1`), e o ESTADO desta janela

Agente `verificador`, por nome, na placa, `load` 1,6–2,5: 37 de 40 filtros verdes (app 24/25 filtros · gpu `it` 10/10 ·
gpu `--lib` 6/6). O portão NOVO `a_borda_nao_tem_pontos_pretos` (`a_borda_sem_pontos_pretos.rs`) verde com o controlo a
reprovar: `pretos rentes · polígono +0.000 (0) · imagem -0.176 (0) — antes -0.949 (12) · -0.958 (8)`. Os três vermelhos:

1. `o_rebote_do_desenho::os_outros_devolvem_a_luz_como_a_imagem` (Disco pequeno 0,0031 > `TOL_REBOTE_OUTROS` 0,003) —
   **vermelho TAMBÉM sem a cura** (a cura desligada no padrão, o mesmo 0,0031): anterior à W6d, não causado por ela.
   Por medir quando entrou (a W6c só correu os portões do renascimento).
2. `o_miolo_das_formas` — o controlo (a W5f) herdava a cura e deixava de ver os degraus. ✅ Curado: o controlo desliga
   também `borda_continua`/`borda_fora` (a W5f inteira); verde.
3. ⚠️ **`o_interior_do_opaco_nao_depende_da_posicao_na_grade` — CAUSADO PELO CAMPO** (sozinho: 2 errados; o `fora`
   sozinho: 0; a cura desligada: 0): na LEI ANTIGA (`miolo: None`; a difusão do produto continua 0 errados), zoom 1,
   fase 3/8, 2 texels PRETOS (placa 0,012 · gabarito 0,65) em (1,820; 0,883) e (1,836; 0,898) — a ~3 cm do canto de cima
   à direita da caixa (x ∈ [0,75; 1,85], y ∈ [−1,3; 0,9], a cena 3). Hipótese (NÃO medida): a bilinear da distância
   arredonda o canto CONVEXO (no canto os 4 vizinhos dão +, −, −, − ⇒ a mistura < 0) e o texel do eixo medial junto ao
   canto mapeia num ponto de fronteira cuja luz sai tapada. ⛔ Não subir o teto (1): um preto novo é a família que
   esta onda cura.

**O próximo passo (W6d continua numa janela nova):** medir o mecanismo do item 3 — o ponto de fronteira e a
visibilidade do texel (1,820; 0,883) com e sem o campo (uma sonda de cor no `c3_fina`, fotografada e retirada, como na
W5g §1), e curar no campo (candidatos a medir: preservar o canto — na faixa, o cheio/vazio cuja 3×3 tem parciais de
DUAS faces leva a combinação de conjunto das retas (convexo: o mínimo das distâncias; côncavo: o máximo) em vez da
reta mais perto; ou o canto pela reta do PRÓPRIO parcial do canto). Depois: o item 3 inteiro de novo (o lote do
verificador acima), a prova de mutação (agente `mutacao`: o ramo do campo no `meio`, o «rente» no `c3_direta`, o
sinal da reta emprestada, o caminho rápido do `c3_borda`), as fotos no app (`fotografa_cena.sh`, cenas 5 e 6) e o
fecho da onda (`/pd-onda-fechar`).

Fotos sem janela (a luz recebida píxel a píxel, `medida_luz2d_w6d_fotos`): [`fotos_W6d/`](fotos_W6d/) — antes ×
depois na borda acesa (zoom 4, o polígono) e no disco de imagem (zoom 2,5), e o disco de imagem em 4 zooms.

## 14. O canto da caixa — o mecanismo medido (`medida_luz2d_w6d_canto`, `target/w6d/canto_s0..5.log`)

A sonda temporária do `c3_fina` (`PH2D_LUZ2D_SONDA`: o ponto remapeado, a outra face e a fração, a normal secante, a
distância do campo, a luz de cada face; retirada antes do commit) numa janela 9×9 em volta dos pretos (zoom 1, fase
3/8, lei antiga; face direita x = 1,8559, topo y = 0,9176; o topo aceso 1,2, a direita no ambiente 0,012):

| texel (fino) | produto: o ponto remapeado (texels finos), fração | sem o campo | luz produto / sem o campo / gabarito |
|---|---|---|---|
| (1,8203; 0,8828) — no eixo, 2,2 da borda | (+1,76; +1,76) na DIAGONAL, f2 = 0, normal 0 | topo (+0,45; +2,27) + direita, f2 = ½ | 0,012 / 0,606 / 0,656 |
| (1,8359; 0,8984) — no eixo, 1,2 | (+0,75; +0,75) diagonal, f2 = 0 | topo + direita, f2 = ½ | 0,012 / 0,604 / 0,650 |
| (1,8516; 0,9141) — o texel do canto (fora do portão) | — | — | 0,012 / 0,575 / — |

⇒ **o campo CHANFRA o canto convexo**: a distância lida no centro do texel do canto (dentro por ~0,4 texel da captura)
é −0,05 (fora). O parcial do canto tem a reta DIAGONAL (o Sobel de duas faces) e os cheios em volta emprestam retas com
a normal diagonal do próprio Sobel ⇒ o iso-0 corta o canto ~1,3 texel da captura ao longo da diagonal (a reta por texel
cortava-o só dentro do texel do canto). O texel do eixo medial acha o ponto de fronteira mais perto NO CHANFRO (a 45°
das duas faces: o `outra_face`, limiar de 60°, não vê outra face ⇒ f2 = 0), o ponto sem normal marcha a partir de
dentro do corpo verdadeiro ⇒ tapado ⇒ o ambiente. A hipótese do §13 CONFIRMADA (o canto arredondado), com a peça
que ela não dizia: perde-se também a média das duas faces.

## 15. A candidata 6 — a UNIÃO com a reta por texel — e o seu kill-criterion (escrito ANTES de medir)

`Config::borda_uniao` (override `BORDA_UNIAO`): no `meio`, «dentro» ⟺ o campo ≥ 0 **OU** a reta por texel (o
`dentro_exato` da W5d, com o recuo lido como −distância no canal do campo) — esta só onde algum dos 4 vizinhos do campo
está na faixa (> −1). As frestas «fora» eram da reta por texel e o campo cobre-as (a cura do §1 mantém-se); o chanfro
é do campo e a reta por texel cobre-o (o canto volta ao da W5d: cortado só dentro do texel do canto). Um único
predicado ⇒ a busca e a marcha continuam a concordar (o 2.º mecanismo do §5).

**Régua** (um processo, intercalada): o censo da borda + os anéis (§2: 3 zooms × 8 fases + o disco de imagem) e os
bits de fora, variantes **antes da W6d** (a reta por texel, sem o fora) × **produto** (`739fb6cfb1`) × **produto +
união**; o custo da C3 (cenas 5 e 6, mínimo de 7) das três; a sonda do canto; e o portão `o_interior_do_opaco*`.

**PASSA** (a união): (1) o `o_interior_do_opaco_nao_depende_da_posicao_na_grade` verde (a lei antiga ≤ 1 errado em todo
caso) e o texel do canto (1,8516; 0,9141) a ±0,1 do «sem o campo»; (2) família (a) ≤ 14 (10 % dos 139 de antes) e o pior
dela ≥ −0,25; (1b) ≤ antes + 2; (3) anéis sem regressão contra o produto (0 errados a mais, ×0, TV ≤ +0,05); (4) bits
fora dos corpos IGUAIS aos de antes (0 diferentes); (5) o custo da C3 ≤ +5 % sobre o de antes (cenas 5 e 6); (6) os
portões do item 3 do §2 verdes. **MORRE**: um preto (< −0,25) novo na família (a), bits de fora diferentes, o portão do
interior ainda vermelho. Entre os dois (custo de 5 a 20 %, (a) de 15 a 70): não entra.

## 16. Rodada 9 — a união: a QUALIDADE passa toda, o custo não (`decide9.log`, `custo9.log`, `canto9_s1.log`)

(1) `o_interior_do_opaco` VERDE (zoom 1 · 3/8: lei antiga 0 errados) e o canto = o «sem o campo» (o texel do canto
0,575; os dois do eixo 0,604 · 0,606, o remapeado de volta ao topo + direita com f2 = ½) · (2) família (a) 139 → **0**
(sem a união: 1, −0,176) · (b) 40 → 30 (sem a união 36) · (3) anéis sem regressão · (4) bits de fora 0 em 24 ·
(5) ✗ custo C3, `load` 1,3, o mínimo de 7 repetido 3× NO MESMO processo (estável a ±0,02 ms):

| C3 (ms) | antes da W6d | sem a união (`739fb6cfb1`) | produto + união |
|---|---|---|---|
| cena 5 | 3,37 (fina 2,40) | 3,05 (fina 1,95) | 3,55 (fina 2,45) — **+5,3 %** |
| cena 6 | 2,18 (fina 1,32) | 2,10 (fina 1,12) | 2,33 (fina 1,35) — **+7,1 %** |

⇒ o campo por UM `textureGather` é MAIS BARATO na fina que a reta por texel (−0,45 ms na cena 5; a rodada 8, com
outro `load`, lia +2,4 % — o número desta tabela é o estável); a união devolve esse custo porque chama o
`dentro_exato` em todo ponto de fora da faixa.

**Candidata 6b (só custo) e o seu kill-criterion (escrito ANTES de medir):** o `dentro_exato` só onde o campo DOBRA —
a torção da bilinear `|s.x + s.z − s.y − s.w|` > τ (numa borda reta as distâncias são uma função linear e a torção é
0; no canto não). τ ∈ {0,05 · 0,15 · 0,30} (`Config::torcao_da_uniao`, override `TORCAO_DA_UNIAO`; 0 = a união em
toda a faixa), as variantes intercaladas no mesmo processo. **PASSA** o menor custo cujas leituras dos itens (1)–(4)
sejam as da rodada 9 (o portão do interior verde, o canto a ±0,1, (a) 0, (b) ≤ 32, anéis iguais, bits 0) E o custo
C3 ≤ +5 % sobre antes nas duas cenas. Nenhuma passa ⇒ não entra; a próxima é marcar o canto no `c3_borda`.

## 17. Rodada 10 — a torção: ⛔ recusada (`decide10.log`, `custo10.log`)

| (3 zooms × 8 fases + imagem; custo `load` 0,7–1,3, mínimo de 7, 2–3 repetições) | (a) | pior | (b) | canto | C3 cena 5 / 6 |
|---|---|---|---|---|---|
| antes da W6d | 139 | −0,964 | 40 | — | 3,36 / 2,18 |
| sem a união | 1 | −0,176 | 36 | preto | 3,04 / 2,10 |
| união τ 0 | **0** | — | **30** | 0,575 | 3,51–3,54 / 2,30–2,31 |
| união τ 0,05 · 0,15 | 1 | −0,176 | 38 · 36 | 0,575 | 3,50–3,51 / 2,30 |
| união τ 0,30 | 1 | −0,176 | 36 | **preto** | 3,52 / 2,31 |
| união τ ∞ (o código lá, nunca corre) | — | — | — | — | **3,50–3,51 / 2,28–2,29** |

⇒ o custo da união NÃO é chamá-la: é o código dela embutido no `meio`, que o compilador põe em cada passo da marcha
(a variante que nunca corre custa o mesmo; o «antes», que tem o `dentro_exato` no mesmo sítio, também). A torção
só tira qualidade (o pior volta a −0,176 e, a τ 0,30, o canto preto) — o knob saiu do código.

## 18. A candidata 6c — a união só na GEOMETRIA — e o seu kill-criterion (escrito com a rodada a correr, antes de a ler)

`meio` = a marcha (o campo, sem a união: `meio_de(pc, false)`, o ramo morre na compilação); `meio_geo` = o
`num_corpo` e o τ do PRÓPRIO texel (`c3_direta` ×5, `c3_anel`) com a `BORDA_UNIAO`. A busca do ponto de fronteira, o
pé e a classificação do texel veem o canto da W5d; a marcha vê o campo. Concordam no que importa: o que a união põe
FORA o campo também (o campo ⊂ a união) ⇒ o ponto de fronteira e o texel de fora estão fora para a marcha (a marcha
nunca nasce dentro — o 2.º mecanismo do §5). Diferença admitida: a luz da marcha passa pelo chanfro do campo (≤ 1,3
texel da captura no canto).

Itens, régua e limiares: os do §15 (o interior verde, o canto a ±0,1, (a) ≤ 14 e o pior ≥ −0,25, (b) ≤ antes + 2,
anéis sem regressão, bits fora 0, custo C3 ≤ +5 % sobre antes nas cenas 5 e 6) e os portões do item 3 do §2 na mesma
leva. **MORRE**: um preto (< −0,25) novo, bits de fora, o portão do interior vermelho.

## 19. Rodada 11 — a 6c ENTRA (`decide11.log`, `portoes11.log`, `pos_sonda.log`, `load` 1,4–2,7)

Contra o §18, na mesma corrida: (1) o interior VERDE (zoom 1 · 3/8: 0 errados; o pior da lei antiga 0,066) e o canto
= o «sem o campo» (0,575 · 0,604 · 0,606) · (2) família (a) 139 → **0**, (b) 40 → **30** · (3) anéis iguais ao
«sem a união» em todo zoom (polígono 0,8: 10 errados → 0) · (4) bits de fora **0** em 24 · (5) C3 cena 5 3,367 →
**3,510 (+4,2 %)**, cena 6 2,181 → **2,282 (+4,6 %)** (o «sem a união» 3,04 / 2,08: a margem que a união come é
pequena; a fina continua 2,41 — o custo está nas perguntas de geometria, não na marcha) · (6) os portões do item 3:
9/10 verdes; `a_borda_nao_tem_pontos_pretos` agora `polígono +0.000 (0) · imagem +0.000 (0)` (era −0,176 na imagem).

### ⚠️ Correção do §13 item 1: o G7 dos outros É da W6d

`o_rebote_do_desenho::os_outros_devolvem_a_luz_como_a_imagem` usa o `LuzDoQuadro`, que liga a borda pelos
interruptores do app (`quadro_ambiente.rs`: `ligado("PH2D_LIGHT2D_BORDA")`) — desligar a cura no `Config::padrao`
(como o §13 fez) NÃO a desligava neste portão. Medido pelos interruptores (determinístico, 40 quadros):

| o pior canal (`TOL_REBOTE_OUTROS` 0,003 = o −0,27 % da W5e + 10 %) | Disco grande | Disco pequeno |
|---|---|---|
| antes da W6d (`PH2D_LIGHT2D_BORDA=0`) | 0,27 % | 0,27 % ✓ |
| o campo sem a união (`PH2D_LIGHT2D_BORDA_UNIAO=0`) | 0,25 % | **0,31 %** ✗ |
| o produto (+ a união) | **0,33 %** ✗ | 0,29 % |

Os dois discos (a mesma forma pela camada dos outros e pela imagem) diferem só no antisserrilhado da borda; a lei
da borda redistribui ±0,06 % entre eles. A tolerância é um máximo AMOSTRADO sob a lei antiga — por remedir com o
seu controlo: a M12 da W5e (a camada lida como não premultiplicada, −0,39 % sob a lei antiga) sob a lei nova, no lote
da prova de mutação; a tolerância nova só entra se a M12 ficar claramente acima dela.
