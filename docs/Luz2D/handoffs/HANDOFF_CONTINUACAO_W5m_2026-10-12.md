# Handoff de continuação — Luz 2D, W5m: a sonda de DENTRO não é o campo de FORA (2026-10-12)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre `main`
> `9ea782138` (o main não andou desde a W5l); W5d–W5m por integrar. Esta janela: `c7bde6134` (a cura, os portões, as
> sondas) · o deste handoff. A tarefa: a [W5l](HANDOFF_CONTINUACAO_W5l_2026-10-12.md) §4 (0–3). O porquê para o
> dono: [BUGS #5](../BUGS_luz2d.md), o último item.

## 1. A lei

**Uma sonda de DENTRO de um corpo (τ < 1, translúcido também) é o campo de dentro — nunca amostra do campo de
fora.** A W5k pesava-a por τ (contínuo em α); era meia-cura. Peso **0** nos três sítios onde o exterior lê sondas:

| sítio | antes (W5k) | agora |
|---|---|---|
| `peso_de_fora` (`c3_direta.wgsl`, a `c0_em` vista de fora: a face, o anel do miolo, a fonte do rebote) | τ | 0 (o opaco já saía pelo alfa da c0) |
| o «+1» (`hrc_soma`, `c2_hrc.wgsl`) | `mix(u, u+1, τ_adiante/τ_aqui)` | `u+1` só se `τ_adiante ≥ τ_aqui`; para dentro de um corpo, o próprio `u` |
| o borrão em cruz (`hrc_saida`) | peso `min τ / max τ` | só vizinhos do MESMO τ |

Para o opaco as três dão o bit de antes (τ ∈ {0, 1}). O caminho de DENTRO (a transmitida, `c0_tau < 1`) continua a
ler todas as sondas.

## 2. Medido (RTX 5060 Ti, `dev`)

**§4.0 — o g3 «céu e vidros sobrepostos».** `sonda_o_g3_ceu_e_vidro_por_regiao` (o RMSE partido; base = os `src/`
das quatro crates em `dde505de25`, os testes de agora): fora dos vidros nada mudou (parcela 0,00612 × 0,00614); a
regressão era TODA dentro dos vidros — o fundo com viés −9,3 % (base −1,1 %). Com os vidros a 0,001 a placa bate
(−1,1 %) e a 1 também (+2,7 %): só a mistura a meio falhava. O miolo a ½ do gabarito ≈ 0,32 (o exterior junto ao
vidro, que vê também a luz que o atravessa por trás); a placa alimentava o anel com a bilinear das sondas vizinhas,
as de dentro do vidro com peso ½ — conta de guardanapo −10 %.

| g3 céu e vidro (Alto · Médio · Baixo) | base (W5c) | W5k | **W5m** |
|---|---|---|---|
| RMSE relativo, fora dos opacos | 0,0074 · 0,0087 · 0,0117 | 0,0100 · 0,0102 · 0,0122 | **0,0065 · 0,0074 · 0,0098** (teto apertado: 0,0072 · 0,0081 · 0,0108) |
| fundo dos vidros, viés | −1,1 % | −9,3 % | −0,8 % |

**§4.1 — o rebote translúcido na sala do g9.** `sonda_a_forma_do_residuo_na_sala`: sem rebote, zero mudança em
tudo; com rebote a holográfica caía −0,11 · −1,06 · −3,0 % a 0,999 · 0,99 · 0,9 (o gabarito +0,01 · +0,08 · +0,9 %:
os albedos 1 batem no teto 0,9, `min(α·a, 0,9)` não muda até 0,9) — um DECLIVE, não um salto.
`sonda_o_transporte_do_rebote_na_sala` (converge opaca, UM quadro a 0,99): o transporte não muda (+0,0000); a
perda entra pela FONTE desse quadro (−0,25 % por iteração; × ~4,8 da série ≈ −1,1 %). `sonda_a_fonte_do_rebote_na_sala`:
a face da SALA cai −1,3 % (a soma inteira sobe porque a face de fora das paredes passa a receber a luz que vaza).
Mecanismo: a fonte lê a c0 junto à face, onde o «+1» misturava a sonda de dentro da parede — que não conta a
entrada, logo não vê o brilho da parede — e o borrão puxava o vizinho de dentro.

| sala a 0,99 × opaca, holográfica, 4 pontos | mudança |
|---|---|
| W5k | 4,6 · 3,9 · 4,4 · 3,5 % |
| só a C3 (peso 0) | 1,25 · 1,10 · 1,25 · 1,02 % |
| C3 + «+1» | 0,50 · 0,41 · 0,42 · 0,37 % |
| C3 + borrão | 0,79 · 0,65 · 0,75 · 0,65 % |
| **as três** | **0,08 · 0,04 · 0,08 · 0,08 %** |
| gabarito | 0,14 · 0,07 · 0,14 · 0,08 % |

⇒ `o_translucido_reflete_a_sua_parte_na_placa` VERDE, teto apertado de 2 % para `TOL_SALA_A_099 = 0,003` (cada cura
sozinha reprova).

**Controlos:** `cmp` da fluência da cena 7 a α = 1 (`sonda_grava_a_fluencia_a_alfa_1`) contra `alfa1_base.bin`:
**idêntica**, depois de cada mudança. Salto em α → 0 (`sonda_o_salto_em_alfa_zero`, o disco a 0,001 × a cena sem
ele): **0,012 %** do máximo, nas duas cascatas. `a_luz_e_continua_na_opacidade_do_oclusor`,
`o_translucido_da_placa_e_o_do_gabarito`, g3 penumbra · anel · faísca · precisão, `a_luz_junto_a_uma_face_e_a_do_gabarito`:
verdes e iguais.

**§4.3 — dois corpos sobrepostos.** Portão novo `dois_translucidos_sobrepostos_multiplicam_o_tau` (discos a ½ e
0,6, céu + lâmpada, as duas cascatas): a SOMBRA da lente (fora dos dois, a reta à lâmpada atravessa os dois: × 0,2,
sem miolo no caminho) rms 0,0098 · 0,0099, pior 0,091 · 0,098; o controlo contra o gabarito de UM disco rms 0,116
(11×); a LENTE por dentro rms 0,041 · 0,045. Tetos = medido + 10 %.

## ⛔ Recusas MEDIDAS (W5m)

| tentativa | medido | porquê não |
|---|---|---|
| clássica: a sonda translúcida dos níveis > 0 marcada como «dentro» (alfa 0) para a `FRONTEIRA_NA_FACE` contar a entrada | sala a 0,99 7,4 % → **21 %**; g3 céu e vidro 0,0065 → 0,0078 | a regra da face foi desenhada para τ = 0 (a transmissão zera o que vem depois) |
| idem, contando a entrada só quando a sonda de cima começa já dentro (alfa ½ como marca) | **idêntico** à anterior (21 %, 0,0078) | o estrago vem de marcar a sonda e não da contagem — a fusão bilinear da clássica não tem onde pôr o translúcido sem reescrever a contabilidade das entradas |

## 3. Portões e sondas

Novo: `dois_translucidos_sobrepostos_multiplicam_o_tau`. Apertados: `TOL_CEU_E_VIDRO`, `TOL_SALA_A_099`. Sondas
(`--ignored --nocapture`): `sonda_o_g3_ceu_e_vidro_por_regiao` (+ `PH2D_G3_DEGRAU=0|1|2`), `sonda_a_forma_do_residuo_na_sala`,
`sonda_a_fonte_do_rebote_na_sala`, `sonda_o_transporte_do_rebote_na_sala`, `sonda_o_perfil_da_lente`,
`sonda_o_salto_em_alfa_zero`.

## 4. ⚠️ ABERTO, na ordem

1. **A clássica (não-produto) continua a 7,4 % na sala a 0,99** (§4.2 da W5l): o «+1» e o borrão não existem nela;
   a C3 é partilhada e já tem a cura; a regra da face dela é do opaco e as duas extensões medidas pioram (Recusas).
   Só os portões de comparação a usam — fica com o porquê.
2. **O miolo da pirâmide no lado ESCURO** (W5h, não desta onda): na cena da lente, o 2.º disco SOZINHO a 0,6 sai
   +5 % no fundo do lado da sombra; a união opaca dos dois discos +22 % com céu e até +84 % relativo sem céu (valores
   pequenos). `o_miolo_da_placa_e_o_do_gabarito` normaliza pelo MÁXIMO da luz e não o vê. Suspeito: a convergência
   (`relaxa` 4 → 32; o W5h mediu 0,039 → 0,025 convergido) numa união alongada. Régua: `sonda_o_perfil_da_lente`.
3. Herdados: os da [W5j §3](HANDOFF_CONTINUACAO_W5j_2026-10-11.md) (itens 6–7, o Unlit do vetor e das formas do
   Motion, o vão da cena 8), o censo dos cartões (22 na dívida), e os vermelhos ANTERIORES (§5).

## 5. Gate batched e binário do smoke

`fmt --check` e clippy `--all-targets -D warnings` nas quatro crates: limpos. `nextest-impacted` (base = merge-base,
`PH2D_GPU=1`, load até 53): **17 542 corridos em 1 458 s**, 17 526 verdes, 12 vermelhos + 4 timeouts — todos
ANTERIORES ou de carga:
- anteriores, iguais na base (W5l §5): `so_a_regiao_se_calcula_e_por_dentro_e_o_mesmo_bit`,
  `o_interior_de_um_opaco_e_o_exterior_mais_proximo`, `g5_o_que_a_placa_alocou…`; na app `as_formas_redondas` ×3,
  `o_miolo_das_formas…`, `o_rebote_nao_depende_da_oitava`, a posição na grade, cenas 5/6/7 «ensina», e
  `o_traco_do_flip_recebe_a_luz_como_o_sprite`;
- carga: `g4_o_custo_da_holografica_nao_depende_do_numero_de_luzes` (novo na família; passou 3× sozinho, load ~5),
  `a_sombra_nitida_nao_tem_escada` (sozinho 98 s, verde);
- ⚠️ `o_oclusor_fino_faz_a_sombra_do_gabarito` NÃO é de carga: sozinho, a load 2–4, passa em **380 s** (≈ tudo CPU
  de uma thread, o gabarito) — acima dos 180 s do nextest. Cena só de opacos (bit a bit igual à W5l). Para a
  família de [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md) não serve: é LENTO — a cura é a faixa `slow` dele
  no nextest ou um gabarito mais barato, não um censo de carga.

## Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=7 cargo run -p ph2d-host-desktop --profile smoke`
2. Clique em «Disco grande (imagem)» (a bola azul de baixo); no cartão «Light Occluder», leve «Opacity» de 1 a 0,99,
   depois 0,5 e 0,1.
3. Tem de acontecer: a 0,99 nada muda à vista (nem a bola, nem o brilho azulado à volta dela); a 0,5 a bola fica
   meio transparente e o miolo dela tem a claridade da beira de fora — a passagem de fora para dentro é suave; a 0,1
   ela quase some.
4. Errado: a 0,99 o brilho azulado em volta escurece; ou a 0,5 o interior fica visivelmente mais escuro que a beira
   de fora, com um degrau na borda.
