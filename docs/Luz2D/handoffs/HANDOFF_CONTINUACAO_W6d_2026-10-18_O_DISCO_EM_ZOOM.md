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
