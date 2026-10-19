# 51 — Spacing 0,1 + Continuous em todos os pincéis: o que se MEDIU (19/10)

> Prompt [51](51_PROMPT_spacing_continuous_em_todos_os_pinceis.md). Dados crus em [`51_regua/`](51_regua/):
> [bissecção](51_regua/bisseccao_2026-10-19.txt) · [pontas do Airbrush](51_regua/pontas_do_airbrush_2026-10-19.txt) ·
> [custo final](51_regua/custo_final_2026-10-19.txt) · [custo, sub-passo para baixo](51_regua/custo_sub_passo_para_baixo_2026-10-19.txt) ·
> [métricas antes × fábrica](51_regua/regua_metricas_antes_x_fabrica.txt) · [custo, traço rápido, 20 pincéis](51_regua/custo_traco_rapido_20_pinceis.txt) ·
> [custo a 40 px, regra 1,25](51_regua/custo_40px_regra_125.txt) · [Oil Bristle, sub-passo 2 × 3 px](51_regua/oil_bristle_sub_passo_2_x_3px.txt) ·
> [falhas da bateria](51_regua/bateria_2026-10-19_falhas.txt). [Prancha](51_prancha_antes_agora.png): os 20 presets,
> «antes» = o Spacing de antes da regra sem Continuous, «agora» = a fábrica; mesmo processo,
> `sonda_spacing_continuo::sonda_a_regua_do_spacing` + `PH2D_SONDA_OUT`. Handoff do WIP (a lei até a 4.ª rodada; o seu
> §2.4 está REVERTIDO abaixo): [P51 WIP](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-19_P51_WIP.md).
> Todas as variantes correram no MESMO processo, por uma máscara de bits thread-local temporária (já removida).

## §1 — A lei final

1. **Pigmento por COMPRIMENTO** (`pigmentSpacing`, como no WIP). Referência = o raio do PINCEL
   (`brush.passo_ao_longo_px`): a variante pelo raio do dab deu Oil Bristle idêntico e Graphite com tinta a 103 px +1,2 %.
2. **Sub-dabs a um número INTEIRO de células, arredondado para BAIXO** (`capsula::sub_passo_sem_batimento` = floor,
   mínimo 1 célula): o traço nunca fica mais ralo que o sub-passo natural. A lei antiga (até 1,25 → 1, depois o mais
   perto, ≥ 2) sobrevive em `capsula::sub_passo_mais_perto`, escolhida por preset pelo knob oculto
   `capsuleRoundNearest` (KNOB_COUNT 89): **Graphite 8B, Charcoal, Soft Pastel, Oil Pastel** (custo) e **Airbrush** (o pó).
3. **Cada sub-dab o seu sorteio** (`sorteio_do_sub_dab`, como no WIP) **e agora a sua POSIÇÃO**:
   `capsula::Salto {desvio, alcance, por_raio}` viaja em `Ante.salto` (preenchido em `stroke/dab_build.rs` e no
   preenchimento de velocidade `stroke/speed.rs`); `sub_dabs` tira o desvio do dab da frente e sorteia outro no disco
   (splitmix semeado pelo desvio da frente e pelo múltiplo do sub-passo). Antes os sub-dabs eram o dab da frente
   transladado pelo traço (o MESMO desvio em todos).
4. **A janela do rasto pela CORDA entre centros de novo** (`direct_segment`) — REVERTE o WIP §2.4 (sub-passo natural).
5. **Pente do relevo por comprimento na tinta, mantido**: não era o suspeito do WIP §4.1 (Oil Bristle 103 px, escada
   3,047 com e 3,047 sem).
6. **Airbrush:** `pressureSizeCurve` 0,7 (ordem do dono 19/10, «afinar para o Rebelle»).
7. **A régua:** para mídia com pó, `midias_de_fabrica::cena_seca` toma a MEDIANA de 7 tiragens
   (`PainterTool::set_semente_dos_tracos`, porta de teste pública) em pontas, rampa, rampa_cob.

## §2 — Escada do Oil Bristle a 103 px (8° · 25° · curva; tecto Rebelle + 0,3 = 2,78 · 2,60 · 2,50)

| variante | 8° | 25° | curva |
|---|---|---|---|
| WIP 2 (janela pelo sub-passo natural) | 3,05 | 2,94 | 2,87 ✗ |
| **corda entre centros** (a final) | **2,23** | **2,39** | **2,22** ✓ |
| janela corda × natural/sub | 2,23 | 2,39 | 2,22 |
| antes da regra | 1,90 | 1,42 | 2,03 |

A 41 px: 2,18 · 1,91 · 2,23 em TODAS as variantes (antes 1,90 · 1,42 · 2,03). ⇒ o culpado do WIP §4.1 era a janela
pelo sub-passo natural, não o pente.

## §3 — Camadas (medianas das camadas 1..4; regra: cada uma escurece ≥ 0,002 e a 4.ª não clareia; p95 < 1,04 × 0,1396 = 0,14518)

| pincel | variante | camadas | p95 | |
|---|---|---|---|---|
| Marker Bullet | janela natural | 0,1396 · 0,0937 · 0,0794 · 0,0721 | 0,1441 | ✓ |
| | corda + mais perto | … · 0,0730 | 0,1452 | ✗ |
| | **corda + baixo** | 0,1396 · 0,0930 · 0,0794 · 0,0721 | 0,1441 | ✓ |
| | antes | | 0,1430 | |
| Dip Pen | janela natural | 0,1396 · 0,0989 · 0,0971 · 0,0950 | | ✗ (0,0971 ≥ 0,0969) |
| | corda × natural/sub | igual | | ✗ |
| | corda + mais perto | 0,1396 · 0,0996 · 0,0971 · 0,0960 | | ✓ (à custa de CLAREAR a camada 2) |
| | **corda + baixo** | 0,1396 · 0,0989 · 0,0960 · 0,0960 | | ✓ |
| | antes | 0,1396 · 0,0989 · 0,0960 · 0,0950 | | |

⇒ a raiz das duas: o sub-passo arredondado PARA CIMA de 1,5 para 2 células.

## §4 — Textura dos secos de grão (Rebelle vale: Graphite > 8,40 [R 9,093] · Oil Pastel > 12,53 [R 17,354])

Sem o salto de posição a fábrica igualava o «antes» (Graphite 7,06 contra 7,05; Oil Pastel 9,72 contra 9,80) ⇒ o
MECANISMO é o salto por sub-dab.

| | desvio rígido (o mesmo em todos) | **salto por sub-dab** | antes |
|---|---|---|---|
| Graphite | 7,74 | **8,69** | 8,77 |
| Oil Pastel | 11,02 | **12,84** | 13,12 |
| Charcoal | 7,32 | **7,59** | 7,61 |

O sorteio repetido por sub-dab e o pigmento por comprimento desligado não mudaram nada no Graphite.

## §5 — Pontas do Airbrush (barra da W6: Rebelle 0,8 ⇒ ≥ 0,6)

É RUÍDO DE SEMENTE. 21 tiragens: fábrica 0,34–0,85, mediana **0,580**; antes 0,39–0,72, mediana 0,558; o portão
antigo lia só a semente 0 (0,596 / 0,635). Com o pó desligado, fábrica = antes (0,714). Varrimento
`pressureSize` 0,4–0,8 × `pressureSizeCurve` 0,5–2 (30 pontos × 21 tiragens): as pontas ficam sempre ≈ rampa − 0,06
(Rebelle: + 0,17) — nenhuma recta acerta as duas. Escolhido **0,5 × 0,7**: pontas 0,680, rampa 0,704 (R 0,63, vale
0,52–0,74), rampa_cob 0,378 (R 0,34). Pó a 31 px (`o_po_e_do_pixel`, Rebelle 23,45): floor no Airbrush **16,38 ✗**;
mais perto **22,3 ✓**.

## §6 — Custo (release, mínimo de 7 rodadas intercaladas, 1024², 25 px/evento, 3 eventos/quadro; ms/quadro; orçamento 16,7)

Lei final, todos ≤ 11,8.

| | antes | final |
|---|---|---|
| Oil Bristle 103 px | 14,5 | **7,3** |
| Express Oil 103 px | 18,9 | **6,5** |
| secos de grão 40 px | 11,7–13,0 | ~8–10 |
| Wet Round 40 px | 7,1 | 11,8 |
| Wet Wash 40 px | 7,4 | 11,1 |
| marcadores/tintas 103 px | ~4,6 | 6,8–7,4 |

Mais lento que antes em alguns, dentro do orçamento. A variante «floor para todos»: Oil Pastel 40 px 10,3 → **20,2**,
Charcoal 18,0, Soft Pastel 17,4 ✗ (recusada; ver §8). Tinta a 103 px da lei final contra antes da regra: os 20 a
+0,6…+3,7 % (coluna «antes» da rodada r6 da bissecção: antes/fábrica 1,005–1,037).

## §7 — Portões que mordem (mutação medida nestas rodadas) e repinados

| portão | mutação | lê |
|---|---|---|
| `o_relevo_dos_fios::o_relevo_nao_tem_escada` | janela pelo natural | 3,05 |
| `a_lei_da_tinta::a_terceira_camada_da_mesma_tinta_nao_clareia` | mais perto em tudo | Marker 0,1452 |
| `midias_de_fabrica` Graphite / Oil Pastel | desvio rígido | 7,74 / 11,02 |
| `o_po_do_aerografo::o_po_e_do_pixel` | floor no Airbrush | 16,38 |
| `midias_de_fabrica` Airbrush pontas | recta (curva 1) | 0,580 (mediana) |

**Repinados** (a lei fechada): `a_lei_da_tinta` (22 cenas) · `opcoes_tests` (6) · `shape_color_tests` (6; as cenas 1 e 2
agora COINCIDEM — com o Continuous o Spacing já não muda o traço) · `o_po_do_aerografo::sem_po` (1) · os 20 presets
regenerados · `LARGURA_DO_WET_ROUND` (8 linhas). Censo da regra: `presets_de_fabrica::a_regra_do_spacing_da_fabrica`.

## ⛔ Recusas MEDIDAS

| recusado | porquê (medido) |
|---|---|
| sub-passo FRACCIONÁRIO | bate com a grelha num período `1/(1 − 1/passo)` (Oil Bristle a 41 px 4,2 px; Express 8,1 px); a 103 px escada 3,06 contra 2,24 |
| o peso (`pigmentSpacing`) no Oil Bristle | satura (tinta 0,985 sem ele); só lhe dava escada |
| `capsuleSpacing` 0,025 na Acrylic Round | a escada dobra, 6,2 → 12,5 (as cerdas andam com o dab) |
| tecto de 3 px no Oil Bristle | escada 4,71 (os fios têm largura em px; fica 2 px) |
| a corda entre centros PARA O ARCO DO PIGMENTO | 1,82× de tinta no Charcoal (espalha). Distinta da corda da JANELA, que entrou |
| o «para cima» 4,01 → 5 | o sub-passo sobe de 1,5 para 2 e rala o traço (§3) |
| janela do rasto pelo sub-passo natural | escada 3,05 · 2,94 · 2,87 (§2) |
| janela corda × natural/sub | Dip Pen ✗ (§3) |
| floor para TODOS | grão a 20 ms/quadro (§6) |
| `capsuleSpacing` 0,0375 / 0,05 nos secos de grão | era o desvio rígido; tinta −1…−3 % |
| sorteio repetido por sub-dab | o período do Spacing; Airbrush dif 0,27 |
| `pressureSize` recto no Airbrush (pontas) | pontas ≈ rampa − 0,06 em todo o varrimento (§5) |
| UMA tiragem do pó como régua | 0,34–0,85 entre sementes; só a mediana de 7 vale |
| pente por comprimento como culpado da escada | 3,047 com e sem (§1.5) |
| raio do dab (em vez do do pincel) como referência | Oil Bristle idêntico; Graphite tinta +1,2 %; a pressão e a caneta deitada o movem |

## §8 — Aberto

> ⏸ **ADIADO pelo dono (19/10)**, depois do smoke OK do P51: os quatro abertos do handoff P51 §6 (o relógio por
> parte em calma · o carimbo fora do orçamento do quadro · o pó por comprimento · o `Salto` em `Dab::capsula`). Não
> os abra sem ordem dele; o plano de medição num lote está pronto (sonda `sonda_traco_rapido` + cronómetros por fase).

- Os três da 50 §4 (a cor K–M por célula, o pente, o carimbo fora do orçamento do quadro) com o custo novo: o traço
  rápido do Oil Bristle a 103 px 13,7 → **7,3 ms/quadro** (load ~4, `custo_final`) — com 2,3× de folga, nenhum dos
  três está no caminho crítico. A divisão por partes ([`traco_rapido_por_partes_load22`](51_regua/traco_rapido_por_partes_load22_2026-10-19.txt))
  saiu a load ~22: só as razões valem (sem relevo −19 %, sem a cor dos fios −16 %); o relógio por parte, em calma,
  fica por medir. O carimbo continua no handler do ponteiro, sem orçamento de quadro (aberto, não mexido).
  Wet Round / Wet Wash a 40 px subiram (7,1 → 11,8; 7,4 → 11,1), dentro do orçamento.
- Flakes de carga vistos na bateria (passam sozinhos): `offthread_tests::the_water_runs_at_a_usable_rate…`,
  `parallel_rows::the_parallel_walk_is_actually_faster`.
