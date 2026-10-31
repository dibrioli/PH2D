# HANDOFF — auditoria do óleo sobre óleo (31/10, tarde)

Prompt: [PROMPT_AUDITORIA_OLEO_SOBRE_OLEO.md](PROMPT_AUDITORIA_OLEO_SOBRE_OLEO.md). Base `e1315189e9`. Ramo
`line/PainterWatercolor`. Queixa: «qualquer tipo de óleo sobre óleo faz encaroçar. o branco ocorre no óleo sobre óleo»
(`smoke_31_10_oleo/dono_3..5.png`). Nada na NVIDIA: o app correu na Radeon (`VK_ICD_FILENAMES` radeon).

## 1. Mecanismos (ficheiro:linha) e curas

| defeito | mecanismo | cura | commit |
|---|---|---|---|
| BRANCO/papel no óleo sobre óleo (dono_5) | Com o Real Oil na mão a sessão põe `Ecra::Cobre.oleo = PINCEL_DE_OLEO` (`sim.rs:349`); `com_oleo` não o trocava e `na_caixa_das_tintas` (`assenta.rs`) fazia do tapar extra TRANSPARÊNCIA: o papel aparecia por óleo grosso (cobertura 1,0 → 0,61/0,49/0,55) e o assentar guardava bege (211,205,179); o `a_f` varia por célula com os fios ⇒ pontilhado. | Com o `PINCEL_DE_OLEO`, célula com óleo mistura como a OPACA (`com_opaca` pela rampa `OLEO_PLENO`; `com_opaca` passa a `max`): o extra fica (a cobertura, `a4c8c80e3d`) e pinta-se pela mistura K-M; `na_caixa_das_tintas` não faz transparência de tinta opaca. | `6032edee62` |
| CAROÇOS (dono_3/4) | A ladeira do óleo com o óleo na mão lê a altura FOTOGRAFADA no início do traço (`solver/ladeira_do_oleo.rs`); o termo que SEGURA (`o·dfilm < 0`) ia inteiro e, na tinta grossa (`film ≈ 0`), o sinal de `dfilm` é ruído: cada passada cavava e empilhava pelo relevo da anterior. | Rota 7 (o produto): com a fotografia, a que segura prende-se a `\|dfilm\|` — pára a água, nunca a inverte. Sem o óleo na mão (aguada ao pé do óleo seco) fica inteira. Recusada: o excesso pesado pela água (p99 54,7). | `ce913b3909` |
| ORLA FERRUGEM com pressão baixa (smoke do dono a meio da janela, amarelo fino sobre azul fino) | Trazida pela linha de cima e já latente na guache opaca: `Ponto::canal` (`assenta/ponto.rs`) misturava em K-M o assentado JÁ diluído em branco com a tinta pura e re-exprimia pela cobertura — o canal que o amarelo absorve ia a 0 (cor da camada (181,48,0) a `a_s = a_f = 0,02`); a concentração lia só `a_e`. | Mistura entre as tintas PURAS (`Ponto::sc`) e depois a cobertura; concentração = parte da tinta de cima no pigmento, `a_e ÷ (a_e + a_s·(1−a_e))` (= `a_e` com o assentado cheio: a guache calibrada não muda). | `c87ef602f9` |
| FUROS até ao papel (dono_3) | Não reproduzidos em massa no motor (`sonda_oleo_furos`: 0 em todos os pincéis, N ≤ 20). No app (agente A, com a cura da cor já dentro) 0–5 no Thick Impasto, 20–50 de 1–3 px no Bristle/Knife/Dabber; depois das duas curas o Thick Impasto 0 e o Dabber visualmente liso (a régua de furos conta os salpicos da orla do Dabber). Leitura: os furos cor de papel do dono eram o papel pelo tapar extra (linha 1) nos poços da linha 2. | as duas acima | — |

Lentes: B (massa) achou também o coeficiente de espessura por célula (`grid/espessura.rs:47-55`) e o relevo dos fios em
fase (`bristles_relevo.rs:47`) — NÃO curados: a medição depois da ladeira já dá o relevo igual ao sem-ladeira; ficam
como hipóteses se o smoke ainda mostrar estrias somadas. C achou a mistura RGB da aguada sobre o óleo (`ponto.rs:110`,
`oleo = 1`) — fica: é a lei «do main» que o dono escolheu em 30/10 (`776a9ff173`).

## 2. Medidas

Motor (`sonda_oleo_furos`, raio 32, vaivém deslocado, pausa 2 s; p99 da altura fina, picos 5×5):

| pincel | N | rota 5 (antes) | rota 7 (cura) | rota 2 (sem ladeira) |
|---|---|---|---|---|
| Thick Impasto | 1 / 8 / 20 | 19,7 / 106,5 / 101,8 | 19,7 / 31,1 / 31,9 | 19,7 / 30,0 / 31,5 |
| Filbert (picos) | 8 / 20 | 47 / 39 | 0 / 0 | 0 / 0 |
| Knife Oily | 8 / 20 | 36,0 / 40,1 | 16,2 / 16,8 | 17,4 / 16,7 |

App real (800², raio 32, relógio real; `mede.py`, rugosidade HP do documento composto, 1 passada → azul ×N → +amarelo):
Thick Impasto 12,3 → 19,3 → 23,0 (antes) · 11,8 → 14,3 → 15,4 (depois), furos 0. Dabber 14,1 → 19,5 → 23,4 · 14,1 →
14,8 → 17,0. Montagem: [app_antes_depois_thick_impasto.png](smoke_31_10_oleo/app_antes_depois_thick_impasto.png).
⚠️ O «antes» do app já levava a cura da cor (binário 13:48); o antes puro da cor é a captura do dono. Flat Bristle não
acaba no prazo de 300 s do driver (lento no app — por medir).

## 3. Seam: porque os portões estavam verdes

- `sonda_oleo_grumos`: pausa de 10 ticks entre passadas (a tinta não assenta, o peso `assente` ≈ 0 desliga a ladeira) e
  mede altura, nunca massa mínima. ⇒ `o_oleo_sobre_o_oleo_assentado_nao_encaroca` (pausa 120, raio 32; vermelho visto).
- `borda_branca_tests`/`a_tinta_opaca_mistura_como_pigmento`: só guache, sobre tela seca e papel BRANCO (onde a
  transparência sai igual ao papel); critério `orla` por canal cego a complementares. ⇒ `assenta::tests::oleo_sobre_oleo_tests`
  (papel creme, pincel de óleo, cobertura e distância à curva K-M; 2 vermelhos vistos) + `a_orla_fina_do_oleo_sobre_o_oleo_nao_enferruja`
  (a = 0,02…0,7; o diagnóstico deu (181,48,0) antes).
- `pintar_tudo_com_o_real_oil_e_a_imagem_de_antes_ao_byte` gravava o defeito como lei: reescrito
  `_salvo_o_oleo_sobre_o_oleo` (ao byte fora das linhas pintadas; nenhum píxel EMBRANQUECE — os três canais a subir;
  medido −34,6…+9,4 de luminância, o +9,4 é a orla (128,92,209) → (116,107,226), mais azul).

## 4. Miniaturas e fecho

Regeradas 16 (13 Oil/* + Fan Brush + Gouache Flat/Opaque), conferidas à vista, `c4d63789dc`; os pontinhos
escuros/rosados da Gouache Flat de antes eram a orla ferrugem. App com pressão 0,2 (Thick Impasto, Filbert): orla
amarelo/azul verde, borda no papel amarelo puro diluído —
[app_pressao_baixa_thick_impasto.png](smoke_31_10_oleo/app_pressao_baixa_thick_impasto.png). Clippy/fmt das
duas crates limpos (inclui avisos anteriores em testes/sondas). Vermelhos ANTERIORES, os mesmos números sem estas curas:
`a_agua_nao_sobe_a_ladeira_do_oleo` («a 4 px» 54 411 > 34 000), `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada`,
`o_traco_seguinte_usa_o_depth_o_gloss_e_a_caixa_novos`, `a_aguada_re_molha_o_oleo_soluvel`.

## 5. Aberto

1. Smoke do dono deste build (óleo sobre óleo, várias passadas, amarelo sobre azul, papel creme).
2. `a_agua_nao_sobe_a_ladeira_do_oleo` a 4 px — vermelho anterior a esta janela (bisect por fazer).
3. Flat Bristle lento no app (driver não acaba em 300 s); Tilted Shade e Metallic Flat sem medição no app.
4. Driver do app em [app_oleo/](app_oleo/): copiar `oleo_app_smoke_v2.rs.txt` para
   `shells/desktop/src/render_loop/oleo_app_smoke.rs` (`PH2D_OLEO_APP_PRESSAO` = pressão), `git apply mod_rs_v2.patch`, build smoke, `S=<saida>
   bash scripts/ph2d-run.sh bash app_oleo/lote_depois.sh`, `python3 -I app_oleo/mede.py <saida>/md`; reverter o
   driver no fim (não se comita na shell).
