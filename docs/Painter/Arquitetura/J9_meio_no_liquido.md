# J9 — o meio anda com o líquido, a «subida» era a pegada, a composição sem Real Oil (janela 9, 01/11)

Ordem: o §3 do [handoff da janela 8](HANDOFF_01_11_janela8.md). Base `803ba5cf6a`. Commits: `73ce4379d6` (§1) ·
`2d7e44b483` (§2) · `364e5e1457` (§3).

## 1. O meio anda com o líquido (`73ce4379d6`)

**Lei** (`wp/solver/meio_transporte.rs`, chamado de `advect_jacobi_rows` depois do `transport_rows`): o gather Jacobi
move o volume molhado `susp + K_W·film` com os coeficientes `k` do passe; o meio vai na mesma combinação:

`m' = Σ k·molhado·m ÷ Σ k·molhado` (a própria célula com `molhado·(1 − leave)`; as quatro fontes do retro-traço).

- A célula SECA não tem voto (molhado 0): a água que chega a óleo seco é água; o óleo molhado leva `m = 1` ao papel.
- Sem meio na faixa (`precisa`) o passe não corre: a água ao bit (`flow_bit`, `advect_jacobi_`, `oleo_sobre_oleo`,
  `assenta`, `bit`, `meio` — 64 verdes). A rota fundida (≥ 4 M células) cede ao passe quando há meio.
- Por linhas na rota do passe (`par::walk_rows`); salta a célula cujas fontes têm todas o mesmo meio.
- Interruptor de medição `solver::MEIO_PARADO` (a lei de antes).

**Medido** (`queda_na_ladeira_tests::diag_o_que_trava_a_queda`, mesmo processo): queda do Wet Flat cheio na crista
4 862 → 80 046. Com o meio SEM efeito (ETA 1, sem endurecer, sem Bingham) o meio que anda e o parado dão o mesmo ao bit
(26 342 · 80 038) — o passe só muda o que o meio lê.

**A régua da queda** (`a_queda_cresce_com_a_diluicao`): o ×0,15 do `agua_vezes` da J8 é Dilution **0,51** — uma
aguada, não um pincel quase seco. Curva medida (Wet Flat, mesmo pigmento):

| água × | 0,03 | 0,06 | 0,1 | 0,15 | 0,3 | 0,5 | 1 |
|---|---|---|---|---|---|---|---|
| Dilution | 0,174 | 0,296 | 0,412 | 0,512 | 0,677 | 0,778 | 0,875 |
| queda | 1 331 | 6 582 | 16 256 | 26 915 | 34 757 | 46 997 | 80 046 |

Os pontos passam a Dilution 0,2 · 0,6 · a do preset (`escala_da_diluicao`): **1 484 · 33 241 · 80 046**, verde.

**Testes novos** (wp): `a_agua_que_chega_ao_oleo_seco_e_agua` · `o_oleo_molhado_leva_o_seu_meio_ao_papel` ·
`o_meio_que_anda_e_o_mesmo_bit_nas_rotas` (Serial = Parallel = Medida). Mutação (o peso da própria célula +1; o passe
desligado): os dois primeiros sangram.

**Óleo diluído continua mais lento que a água** (`oleo_diluido_tests::diag_o_oleo_diluido_escorre`, abaixo aos 15 s,
d 0,9): Wet Round óleo 0,034 × água 0,116 · Filbert 0,100 × 0,147 · Dabber 0,049 × 0,099 · Thick Impasto 0,085 × 0,128.

**Custo** (`desempenho_oleo_tests`, A/B intercalado com variantes « parado»): advecção do Filbert Oil d0,9 0,262 →
0,351 ms/passo (a 1.ª versão, em série, dava 0,599). O óleo molhado fica acordado mais tempo (Wet Drip Oil: 6 030
células vivas aos 120 s contra 1 982 parado) — é CERTO: antes o óleo que escorria para o papel virava água (`m = 0` da
célula virgem) e evaporava depressa.

## 2. A «água que sobe a ladeira» era a pegada do pincel (`2d7e44b483`)

Sonda nova `sobe_a_ladeira_porta_tests`:

- `diag_por_que_porta_a_agua_sobe` — a massa no miolo do óleo entra TODA durante o traço (k10 9 228 → k40 95 492 →
  ao soltar 95 891) e depois só DESCE (aos +600 tiques 56 012); até soltar, com e sem a ladeira, ao bit. Metade é `sett`.
- `diag_a_pegada_do_wet_flat` — no papel limpo, o Wet Flat de **Size 20 pousa de −18 a +19 linhas** (Size 12: −10 a
  +11): o Size é o RAIO (`brush.radius_px`). A cena punha o centro a `topo − 10 − folga`: o pincel pousava 6 px DENTRO
  do óleo.

Cura da régua (`agua_sobre_oleo_tests`, `RAIO = 10`, o pé a `folga` px): com a ladeira × sem ela (controlo
`LADEIRA_ROTA = 2`):

| cena | com | sem |
|---|---|---|
| encostado | 1 552 | 258 349 |
| a 4 px | 0 | 112 637 |
| de lado, encostado | 0 | 35 994 |
| de lado, a 4 px | 0 | 0 (a água não chega — por isso o eixo `x` prova-se encostado) |

Tectos = 10 % do controlo. `a_agua_nao_sobe_a_ladeira_do_oleo` verde.

### O escorrer por cima do óleo seco (a observação 1 do dono) — continua ABERTO

`escorre_contorna_tests::diag_o_escorrer_contorna_o_relevo`, Dabber seco + Wet Drip Water por cima (fotos: J8 com
ladeira · J9 com ladeira · J8 sem · J9 sem — [`fotos/j9/contorna_j8_j9.png`](fotos/j9/contorna_j8_j9.png)):

| árvore | miolo | abaixo (atravessou) | acima |
|---|---|---|---|
| J8, com ladeira | 701 273 | 0 | 277 517 |
| J8, sem ladeira | 910 183 | 0 | 191 154 |
| J9, com ladeira | 98 902 | 344 982 | 279 582 |
| J9, sem ladeira | 144 996 | 349 076 | 175 818 |

Na J8 o «muro» era a COLA do meio (a água ficava viscosa em cima do óleo seco e parava lá). Com o meio no líquido a
água passa por cima e pinga por baixo; a ladeira quase não pesa (−1 %). Mecanismo provável (por medir): a ladeira entra
no NIVELAMENTO (`flow_jacobi.rs`, `termo`, clampado a `LevelClamp` por passo), enquanto a gravidade da tela acumula na
velocidade persistente — a parede empurra no máximo `LevelClamp` contra uma força que cresce.

### ⛔ Recusa MEDIDA — o dique no destino

A advecção PUXA (cada célula recolhe de `i − uv`), por isso o dique da J8 (cortar na FONTE) não podia morder. Medido o
dique no DESTINO (a célula com chão acima da superfície da fonte não puxa dela; chão assente ou inteiro — iguais):
contorna: abaixo 344 982 → 259 790 (−25 %), miolo 98 902 → 217 803 (empoça em cima do óleo); queda do Wet Flat na
crista 80 046 → 23 527 (−71 %); parede 1 579 · 0 · 0 (igual). Código removido. A cura candidata é a do mecanismo acima:
o declive do chão como FORÇA no momento (`−g∇(h + chão)` das águas rasas), não como nivelamento clampado.

## 3. A composição sem Real Oil (`364e5e1457`)

Cronómetros dentro da composição (`desempenho_oleo_tests`, réplica 0, 120 s; `composite_contagem`):

| variante | campo | pigmento | sombra | resto | total | células motor → renderizadas |
|---|---|---|---|---|---|---|
| Wet Drip Oil d0,9 (sem Real Oil), antes | 5 361 | 390 | 1 278 | 295 | 7 324 | 15 195 → 29 651 |
| idem, depois | 0 | 385 | 1 | 114 | **500** | 15 195 → 15 195 |
| Filbert Oil d0,9 (Real Oil) | 5 768 | 10 164 | 4 392 | 397 | 20 722 | 17 989 → 26 193 |

Causa: a óptica do Óleo Real corria quando o campo `Grid::oleo` existia; desde a E10 ele existe só pelo meio, a
solubilidade ou o corpo. Lei: `Grid::oleo_real` (alguma célula com `oleo > 0` ou `pincel_de_oleo`), ligada em
`oleo_celula::pousa`/`veste_virgens`, recontada em `restaura_janela`, instantâneo (`snapshot.rs`, duas portas) e
limpar (`canvas_ops`). Pode ficar LIGADA a mais (o caminho do óleo, a mesma tela) — nunca desligada a menos (medido
com um `assert` temporário no portão da composição sobre o portão-mãe).

Prova `oleo_sem_real_oil_tests::a_tinta_sem_real_oil_nao_paga_a_optica_do_oleo`: sem Real Oil o caminho do óleo e o
sem óleo dão a MESMA tela ao byte (Velha e Pigmento, Wet Drip e Thick Impasto — `diag_os_dois_caminhos_por_lei`);
controlo com Real Oil: 4 465 bytes mudam. Mutação (o portão de volta a `!oleo.is_empty()`): sangra.

A régua `render_plano` (`sonda_varredura_regua.rs`) arranca o campo à mão — passa a guardar e devolver a marca (sem
isso `portao_mae_f0_lei_pigmento_minimo` ficava vermelho: o caminho do óleo sobre um campo vazio).

## 4. Vermelhos ANTIGOS achados (A/B: iguais com o meio parado e sem a cura da composição)

`medida_oleo_real_fenda`: `o_papel_nos_vaos_entre_lascas_nao_fica_preto` · `o_dente_do_papel_nao_atravessa_a_tinta_grossa`
· `a_fenda_forte_nao_volta_aos_pontos_pretos_na_cena_do_dono` · `transparencia`: `a_transmissao_tinge_o_papel_creme_do_documento`
· `o_modo_blender_casa_com_o_cycles_e_o_clear_glass_e_o_de_antes` · `a_transparencia_deixa_ver_o_grao_e_a_cor_do_papel`
· `a_transmissao_e_o_relevo_do_papel_mostram_o_grao` · `sonda_oleo_furos::o_oleo_sobre_o_oleo_assentado_nao_encaroca` ·
`sonda_oleo_grumos::o_oleo_sobre_oleo_molhado_satura_o_relevo` (p99 1,49 → 10,95). Não estavam na lista da J8; por
bissectar.

## 5. Aberto

1. **O escorrer contorna o relevo** (§2): o declive do chão como força no momento; réguas: `diag_o_escorrer_contorna_o_relevo`
   (abaixo → 0 com o Dabber seco), `a_queda_cresce_com_a_diluicao`, `a_agua_nao_sobe_a_ladeira_do_oleo`, `sonda_oleo_derrete`.
2. **A composição do Filbert com Real Oil** (20,7 s em 120 s): o campo de altura e a sombra refeitos a cada tique
   sobre 26 000 células enquanto o filme só assenta.
3. Os 9 vermelhos antigos do §4 (bissectar, árvore inteira por commit).
4. O resto do §4 da janela 7 (E9 ⚑ furos, chip de Dilution, ficheiros em 700 linhas, E10c, E11 GPU com o meio — a GPU
   também tem de levar o meio com o líquido —, E12, E13, outros 3 modos, ajuda ilustrada, crateras).
