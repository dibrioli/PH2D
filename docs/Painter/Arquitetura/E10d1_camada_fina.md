# E10d1 — a camada FINA prende o matiz à cor escolhida (lei `Pigmento`, 01/11)

Cura de [E9 §7.3 #1](E9_portao_mae.md) pela decisão do dono [F3 §8 Q8](F3_controlos.md) («o que der menos artefactos»).
Só a lei `Pigmento`; a `Velha` não passa por aqui (o ecrã da tinta só corre com `LeiDaTinta::Pigmento`, `render.rs`).

## 1. O defeito (medido)

Com `𝒮` uniforme e o `𝒦` por banda tirado do masstone, uma banda escura (o vermelho do azul, `R∞ = 0,021`) fecha
numa película que as outras ainda deixam passar: a borda do azul `40,90,220` sai CIANO, a do amarelo `245,205,25` sai
LIMÃO. Medido na faixa de 48 volumes (`0,002 µm · 1,25ᵏ`) sobre branco, em 8 bits: azul até **49,2°** de desvio de
matiz Oklab (`186,241,255` a `v = 0,002 µm`), amarelo até **15,1°** (`255,254,35`).

## 2. A lei

Sobre o papel o resultado POR BANDA dá a claridade `L` e a croma; o MATIZ é o da cor escolhida. Sobre tinta curada
(vidrado) continua por banda (amarelo sobre azul curado = verde). O meio-termo é contínuo, pesado pelo papel que
ainda se vê através do fundo curado.

- `escolhida` = Oklab de `filme ⊕ aberto` (`Soma::cor`; com o lado espectral, `cor_pigmento` — a cor do Pigment mix).
- `R_p = camada(o, 1)` (a película sozinha sobre o branco), `(L_p, a_p, b_p) = Oklab(R_p)`, `C_p = |(a_p, b_p)|`.
- `preso = Oklab⁻¹(L_p, C·ĥ)` com `ĥ = (a, b)/|(a, b)|` da escolhida e `C ≤ C_p` a maior croma que cabe no sRGB nesse
  `(L_p, ĥ)` (bissecção, 20 passos). Uma película fina é mais CLARA e menos saturada — nunca de outro matiz.
- `g = média_b(R_g − R_g0)` ∈ `[0,1]`: `1` sem fundo curado (`R_g = 1`, `R_g0 = 0`), `→ 0` sob tinta curada opaca
  (azul de 6 µm: `R_g − R_g0 ≈ 1e-7`). Sem limiar: é a fracção do branco que o fundo ainda devolve.
- `R|branco = R_w + g·(preso − R_p)` (por banda, `[0,1]`), `R|preto = R_b + g·(preso − R_p)` (preso a `[0,1]`).
- O ecrã decompõe `R|branco` com a cobertura `média(1 − (R|branco − R|preto))` ([E5](E5_ecra.md), `decompoe`).
- Cinzento (`|ab| < 1e-4`) ou sem pigmento: a pilha por banda de hoje, ao bit.

O acréscimo também sobre o preto é para que o FECHO continue invisível: quem cura grava `(R|branco, R|preto)` como
`(r_g, r_g0)` e, depois, o ecrã sem tinta aberta lê exactamente a mesma cor e a mesma cobertura.

Código: `crates/ph2d-wet-paint/src/render/tinta_matiz.rs` — `cor_escolhida` :14, `croma_na_gama` :30,
`prende_o_matiz` :50, `pilha_presa` :82; ligado em `render/tinta.rs` `ecra_sobre_o_fundo` :24–31.

## 3. Antes × depois

Sonda `sonda_borda_fina` (`render/tinta_matiz_tests.rs`, `#[ignore]`; mesmo processo, antes = a pilha por banda,
depois = `ecra_da_celula`): `PH2D_SONDA_OUT=<dir> bash scripts/ph2d-run.sh cargo test -p ph2d-wet-paint --profile smoke
--lib sonda_borda_fina -- --ignored --nocapture`. Desvio de matiz Oklab da cor sobre branco (8 bits) à escolhida,
nos 48 volumes com tinta visível:

| tinta | lei | p50 | p99 | máx (v, cor da borda) |
|---|---|---|---|---|
| azul `40,90,220` | antes | 0,2° | **49,2°** | `0,0020 µm`, `186,241,255` (ciano) |
| azul | depois | 0,0° | **0,6°** | `0,0025 µm`, `211,226,255` (azul claro) |
| amarelo `245,205,25` | antes | 1,9° | **15,1°** | `0,0061 µm`, `255,254,35` (limão) |
| amarelo | depois | 0,1° | **0,9°** | `0,0020 µm`, `255,247,216` (creme dourado) |

Em `f32` (sem o arredondamento de 8 bits) o desvio depois é `0,00°` em água (`m = 0`) e óleo (`m = 1`).
Foto: [`fotos/e9/borda_fina_depois.png`](fotos/e9/borda_fina_depois.png) — da esquerda (quase nada) à direita
(masstone): linha 1 azul antes, 2 azul depois, 3 amarelo antes, 4 amarelo depois. A de antes no traço real:
[`fotos/e9/borda_fina.png`](fotos/e9/borda_fina.png).

## 4. Testes

| teste | HEAD `e8f35b5710` | depois |
|---|---|---|
| `a_camada_fina_tem_o_matiz_da_cor_escolhida` (azul e amarelo, `m` 0 e 1, ≤ 2°) | **vermelho** (azul 50,04°) | verde (0,00°) |
| `o_vidrado_sobre_tinta_curada_continua_por_banda` (amarelo T 0,9 sobre azul curado 6 µm = `84,110,25`, ≤ 1e-6) | verde | verde (igual ao bit nos 3 canais impressos) |
| `fechar_a_camada_fina_guarda_o_matiz` (fechar com `pilha_presa` não muda o ecrã, ≤ 0,01/255) | — | verde |
| `render::tinta::tests::*` (a régua `esperado` passou a ler `pilha_presa`) | verde | verde |
| `advect_jacobi_*`, `o_invariante_soma_massa*`, filtros `tinta`/`ecra`/`decompoe`/`assenta`/`pigmento` (76 testes) | — | 75 verdes, **1 vermelho**: `drying::tinta::tests::fechar_e_o_que_se_via` (§5.1) |
| `portao_mae_f0_lei_pigmento_minimo` (com a árvore da D2 ao lado) | — | verde, 11 s: Thick Impasto e Wet Round na Pigmento `mix_p99 3,0` · `mix_max 4,1` · ferrugem/furos/degraus 0 |

A `Velha` não muda: `ecra_da_celula` só corre com a lei `Pigmento`; `advect_jacobi_*` e `o_invariante_soma_massa*`
verdes ao bit.

## 5. Aberto

1. **O fecho tem de gravar a pilha presa** (`drying/tinta.rs fecha_celula`, ficheiro da linha D2): hoje grava a pilha por
   banda, e a camada fina que cura volta a ciano (`fechar_e_o_que_se_via` vermelho, célula 3 `0,05 µm` verde). A cura
   é trocar as duas linhas `c.r_g0 = camada(…r_g0)` / `c.r_g = camada(…r_g)` por
   `(c.r_g, c.r_g0) = crate::render::tinta::matiz::pilha_presa(&o, c, so_aberto.as_ref());` (antes de zerar o
   aberto) — provado sem tocar no ficheiro por `fechar_a_camada_fina_guarda_o_matiz`.
2. **A croma do amarelo fino é a da gama**: a `L ≈ 0,97` o dourado só cabe pálido no sRGB, e a croma sobe depressa
   quando a película escurece (`~0,05–0,2 µm`: a rampa é contínua mas íngreme, visível na linha 4 da foto).
   Se o olho do dono a achar um degrau, a alternativa medida a fazer é a recta Oklab branco → masstone (`C = s·C_esc`).
3. A foto do traço real (`sonda_varredura_oleo`, coluna `orla_p99_plano`) não foi regravada nesta etapa; a foto
   depois é a da faixa (§3). A grelha F0 inteira fica para a fila.
4. Molhado: o matiz preso é o da cor escolhida SECA; o escurecimento molhado (`s(m, molhado)`) só muda `L` e a croma.
