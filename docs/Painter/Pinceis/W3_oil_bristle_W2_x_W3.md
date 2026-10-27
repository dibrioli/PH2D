# Oil Bristle — W2 × W3 × Rebelle com o Óleo Real ligado (27/10)

> Pedido: o dono tirou o Oil Bristle da fábrica «a princípio» no smoke de 27/10 (a W2 com ele foi aprovada, «MUito
> bom»). Antes de o tirar: a W3 piorou-o? O preset NÃO mudou entre a W2 e a W3 (o ficheiro de fábrica é o mesmo byte a
> byte em `backup/oleo-real-W2-2026-10-27` e em `1acf37e292`); só o motor mudou.

## Método

- Duas exportações limpas (`git archive`, target próprio cada uma): a base W2 (`backup/oleo-real-W2-2026-10-27`) e o
  HEAD da W3 (`1acf37e292`). Na W2 o `PH2D_EXAME_OLEO=1` é o óleo ligado desde o 1.º traço; na W3 é «na foto»
  (ao byte o mesmo, gate `a_foto_com_oleo_so_na_foto_e_a_de_sempre`).
- Bancada (`exame_bancada::exame_pinta`, escala 1): `gestos`, `tamanhos`, `sobre`, fotos A e B.
- Papel TEXTURIZADO (o do dono): sonda só nas exportações — o Oil Bristle de fábrica sobre o Cold com Relief 0,6, Óleo
  Real ligado, 103 px recto, 40 px recto e inclinado, um vermelho a cruzar; a tela final (`run_full` + a luz do relevo).
- Bisecção LINEAR pelos commits de motor `2e24c8d9b1..1acf37e292` (os 16 que tocam `ph2d-wet-paint`,
  `ph2d-tool-painter/src`, `ph2d-painter-brush`; o `8966e951b2` sozinho não compila) com a sonda do papel.
  ⚠️ Exportar por cima de uma árvore com `tar -x` restaura o mtime do commit e o cargo julga o crate FRESCO (o
  1.º bisect leu «fa11 = W2» com o rlib de outro commit): `tar -x -m`.

## Resultado

| foto | tinta (saturada nos dois): px ≠ · px > 8 · Δ luminância | sombra no papel em volta: px ≠ · Δ luminância |
|---|---|---|
| gestos B | 13,6 % · 0,9 % · +0,29 | 93 % · **+31** |
| tamanhos B | 4,4 % · 0,1 % · +0,05 | 83 % · **+28** |
| sobre B | 4,9 % · 0,1 % · +0,06 | 94 % · **+41** |
| papel Cold + Relief 0,6 | 3,4 % · 0,2 % · +0,07 | 84 % · **+25** |

**A tinta do Oil Bristle é a mesma** (Δ ≤ 0,3 níveis; os fios, o relevo e a cor iguais à vista). O que mudou é a
**sombra cinzenta no papel em volta do traço**: mais clara na W3 — mais perto do Rebelle, que não tem halo nenhum.

| commit (papel texturizado) | sombra no papel: média · p10 | tinta |
|---|---|---|
| W2 (`2e24c8d9b1` … `7677042b2d` ao bit) | 190,9 · 114 | 110,7 |
| **`03048b3e7d`** (K4) | **214,6 · 166** | 110,7 |
| **`1f89fd782a`** (K10) | 215,8 · 169 | 110,7 |
| HEAD `1acf37e292` | 215,8 · 169 | 110,7 |

- **`03048b3e7d` (K4), o mecanismo principal:** a sombra do relevo no papel passou a ser fracção da LUZ linear —
  `sombra_em_u8` guarda `1 − sRGB(1 − s)`; antes `s·255` escurecia o byte sRGB e tirava ~2,2× a luz. O halo cinza
  em volta de todo traço com relevo fica ~metade (21 % dos píxeis da cena mudam, todos fora da tinta).
- **`1f89fd782a` (K10):** o rebote de um salto da tinta vizinha acende o papel junto às paredes (10 % dos píxeis, +1
  nível na média).

**Veredito: a W3 NÃO piorou o Oil Bristle** — a tinta é a da W2 aprovada; a sombra à volta ficou mais leve (na direcção
do Rebelle). Ele sai da fábrica pela decisão do dono, como candidato: ficheiro em `W2_Bristle_candidatos/`, receita
`OIL_BRISTLE` em `EM_ESTUDO_W2` (`familias/oleo_acrilica.rs`), e continua a base das receitas e das medidas que o
usam (leitores com recurso ao candidato). Se o dono o quiser de volta, é mover o ficheiro e a receita para o `WET`.

Folha: [`pranchas/W3_oil_bristle_W2_x_W3.jpg`](pranchas/W3_oil_bristle_W2_x_W3.jpg) (W2 | W3 | Rebelle `Bristle Flat`,
as três cenas B, o recto de 200 px ampliado, o papel texturizado e o seu zoom).
