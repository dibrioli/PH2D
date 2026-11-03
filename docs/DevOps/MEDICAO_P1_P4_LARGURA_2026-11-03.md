# Medição P1–P4 da largura do pincel — 03/11

> Leitor: a próxima LLM / o integrador. Plano: [PLANO_ACELERACAO_2026-11.md](PLANO_ACELERACAO_2026-11.md).
> Os brutos (`m1/m2/m3.txt`) estavam no scratchpad da janela antiga e já não existem; **as tabelas abaixo são o registo**.

## A. P1 — reta contra as tabelas gravadas (sem simulação)

Erro = `W_cheia(m_B(w)) − w` sobre `w` 4..min(600, `size_max_px`); a tabela é invertida como `raio_do_motor_da_tabela`.

| Medida | Valor |
|---|---|
| reta + 5 pontos passa | **34 / 55** presets |
| falha em | ferramentas Blend, Smear, Water; aguadas Fan, Wet Round/Drip/Dry, Express Oil, Dabber, Gouache |
| joelho até | 990 (Water Brush) |
| pontos que bastam à tolerância do portão | mediana 3, máx 25 |
| a ½ da tolerância | mediana 6, máx 32 |
| área pintada: parar na cobertura | 33,5 % |
| área pintada: amostragem gulosa | 8,4–10,9 % da tabela cheia |
| miniatura sozinha | ≤ 0,41 s por preset |

Custo CPU por raio (s): Water Brush 3,5→5 · 13→16 · 55→64 · 120→214 · 300→708; Blender Soft 300→498; prova da semente 134–365.

## B. D — a meia fase do misturador

Commit `500cc8ce70`: `cy = ⌊h/2⌋`. A meia fase movia a largura até 0,66 px (Water Brush motor 7,2, 7,3 %) e 0,29 px nos Blender (13,2).

## C. Prova de mutação do alarme (B ⊇ A)

Commit `949a9cc17f`. A = os 8 irmãos `a_largura_e_a_prometida_*` (retirados em `a76326af2b`); B = o alarme + miniatura. **B apanha um superconjunto de A.**

| Mutação | A | B |
|---|---|---|
| M1 `+1` no `raio_do_motor` | 7 (óleo) | 55 |
| M2 `×0,9` | 28 | 55 |
| M3 dab `+0,37` célula y | 0 | 20 |

Base verde; relógio dos 3 testes juntos 85–99 s. Alarme: 55 presets, raios 2·7·26, 26,6 s, determinístico 3/3.

## D. A×B do byte de umidade (pedido do dono: manter a cura `62e7fc8505` e devolver o molhado)

Sonda temporária: 10 leis no MESMO processo, 45 presets de pintura (contados no TSV da sonda; a mensagem do commit 5bdaef3fa3 diz 40 por engano), raios 7 e 26; cena = traço 800 px sobre branco, relógio fixo, 480 quadros.
**A água posta é igual em todas as leis (+0,0 %): a hipótese de perda de água caiu.**

Miolo = byte ÷ cheio na linha central, depois do traço, antes do tempo. Cada célula: r26 / r7. Tinta/umidade = variação contra a lei antiga.

| Lei | miolo Round | miolo Drip | tinta Round | tinta Drip | umidade Round | umidade Drip |
|---|---|---|---|---|---|---|
| antiga (byte inteiro) | 0,98 / 0,98 | 0,98 / 0,98 | ref. (px 41 993 / 11 589) | ref. (px 42 519 / 11 527) | — | — |
| cobertura sobrescrevendo (`62e7fc8505`) | 0,44 / 0,27 | 0,33 / 0,23 | −25,4 / −13,0 % | −22,7 / −13,2 % | −74,4 / −81,9 % | −84,4 / −86,8 % |
| cobertura + orla só molha | 0,93 / 0,94 | 0,80 / 0,94 | −25,8 / −13,2 % | −24,4 / −12,9 % | — | — |
| **rampa 1 célula + orla só molha (ESCOLHIDA)** | 0,98 / 0,96 | 0,98 / 0,96 | −1,6 / −11,4 % | −2,9 / −10,9 % | −1,8 / −7,5 % | −2,7 / −10,5 % |
| rampa 2 (orla só molha) | — | — | −3,4 / −13,4 % | −4,7 / −12,8 % | — | — |
| rampa 4 (orla só molha) | — | — | −6,9 / −13,5 % | −7,6 / −12,8 % | — | — |
| rampa 1 sobrescrevendo | 0,97 / 0,61 | 0,89 / 0,51 | −5,3 / −13,0 % | −10,4 / −12,7 % | — | — |
| rampa 2 sobrescrevendo | — | — | −9,6 / −13,3 % | −15,2 / −13,2 % | — | — |
| rampa 4 sobrescrevendo | — | — | −15,1 / −13,2 % | −16,6 / −13,2 % | — | — |
| rampa ½ + orla só molha | 0,98 / 0,97 | 0,98 / 0,97 | −0,9 / −6,9 % | −1,5 / −6,2 % | — | — |

**Lei escolhida:** `cobertura = depósito ÷ min(cheio − portão, cheio ÷ raio)`; `byte = max(round(cheio·cobertura), min(atual, cheio))`.
Porquê rampa 1 e não ½: a rampa de meia célula satura antes e traz de volta mais da orla que depende da fase (coluna da orla no motor 4,75: 16,1 contra 13,9; margem 0,08 contra 0).

**Aberto:** Wet Round/Drip no raio 7 ainda têm ~11 % menos área que a lei antiga (~1,7 px de 14,5). O dono julga a olho.

### ⛔ Recusas MEDIDAS

| Recusado | Porquê (medido) |
|---|---|
| sobrescrever o byte | traço listrado (foto); miolo 0,27–0,61 no r7 |
| rampa pelo raio todo | mancha −25 % de tinta |
| rampa de ½ célula | satura cedo, traz a orla de fase (16,1 × 13,9; margem 0,08) |

## E. A régua do Smear

Colunas da cena (lei nova), `x:valor`:

| motor | cx | colunas |
|---|---|---|
| 4,5 | 89 | 89:25,4 · 90:45,5 · 91:22,4 · 92:12,7 · 93:11,5 · 94:10,4 |
| 5 | 90 | 90:32,6 · 91:54,1 · 92:33,8 · 93:14,1 · 94:12,0 · 95:11,7 |
| 13 | 106 | 106:52,4 · 107:69,4 · 108:35,5 · 109:16,0 · 110:14,9 |
| 26 | 132 | 132:70,1 · 133:88,4 · 134:47,4 · 135:34,7 · 136:34,0 |

As 3 colunas depois da orla são água a correr pela orla da base, não rasto. Fora delas: pior fase 1,67 · pior vizinho 1,50 (lei nova); 1,64 · 1,39 (lei de antes da cobertura).

⚠️ **Decisão a rever:** a tolerância de `o_rasto_do_smear_wet_e_continuo_e_nao_depende_da_fase` subiu de 1,0 para **1,75 px**; o 1,0 só passava com o miolo seco. O que a cobertura curou fica guardado no salto COM a orla: `SALTO_DA_ORLA_PX` 6 (byte inteiro 10,1 · nova 4,7).
Recusado por medição: estender a janela ao fim do traço (`x1 = cx + 2,5m`) — já tinha recusa de 29/10 no doc-comment e não curava sozinho (1,27).

## F. O Blend

A tabela lia a largura equivalente (~metade da pegada). Com a tabela velha, círculo → visível a 10 %: 52 → 78–82 · 104 → 160–176 · 200 → 310–339.

Lei crua Blender Soft (motor → equivalente / 5 % / 10 %): 20 → 19,8/21/20 · 28 → 29,5/51/29 · 40 → 40,0/63/44 · 56 → 65,5/101/100 · 80 → 89,8/142/141 · 160 → 175,3/286/284 · 224 → 249,2/404/400.

Tabela nova = 14 raios (2..224) pela mudança a 5 % (~1 min CPU por preset, sem refino). Círculo → visível depois:

| Preset | 14 | 26 | 52 | 104 | 200 |
|---|---|---|---|---|---|
| Soft | 14 | 24 | 53 | 104 | 200 |
| Hard | 13 | 28 | 54 | 105 | 199 |
| Dots | 14 | 25 | 52 | 105 | 199 |
| Textured | 13 | 26 | 64 | 109 | 203 |

Guarda: `a_area_do_blend_cabe_no_circulo` (folga 1,25× + 2 px; 13·26·52). Nos 4 presets só `width_table` e miniatura mudam (conferido por campo no iTXt).

## G. Provas de mutação da lei (commit `5bdaef3fa3`; base 3 verdes, ~25 s)

| Mutação | Reprova |
|---|---|
| sobrescrever | `o_traco_molhado_fica_molhado_ate_a_orla` + `a_area_do_blend` |
| rampa pelo raio | `o_traco_molhado` |
| byte inteiro | `o_rasto` (salto da orla) |
| Blender Soft com a tabela velha | `a_area_do_blend` |

`o_rasto` não reprova nas duas primeiras: é a umidade que as guarda.

## H. Estado

Vermelhos DE PROPÓSITO até o dono aprovar: `a_largura_de_fabrica_e_a_aprovada`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` (24), `sem_glaze` (13). Aprovar: `PH2D_PINCEIS_APROVA=1`.
Fan, Smear Shattered e Smear Wet voltaram ao ficheiro de `6cbcce804c` porque as tabelas regeradas em `0dc6532492` mediam a lei que saiu.

## I. A família de carga, sozinha (03/11, depois de 5bdaef3fa3)

Um filtro de cada vez, perfil `smoke`. ⚠️ Load 10–14 de outras linhas: o relógio NÃO vale como número de calma (só serve de teto). Os 9 filtros passam mesmo assim; nenhum foi promovido a `FLAKES_DE_CARGA`.

| Filtro | Crate | Testes | Correr (s) | Load antes→depois |
|---|---|---|---|---|
| `cada_interruptor_que_neutraliza` | `ph2d-panel-painter-layers` | 1 | 35,4 | 3,64→10,56 |
| `pular_a_pincelada` | `ph2d-tool-painter` | 1 | 44,3 | 10,56→12,38 |
| `cada_knob_do_material` | `ph2d-tool-painter` | 1 | 12,1 | 12,38→13,68 |
| `portao_mae_f0` | `ph2d-tool-painter` | 4 (+1 de placa, ignorado) | 29,6 | 13,68→14,01 |
| `o_oleo_pouco_diluido` | `ph2d-tool-painter` | 1 | 20,2 | 14,01→13,68 |
| `o_pen_down` | `ph2d-tool-painter` | 1 | 1,6 | 13,68→13,63 |
| `o_deposito` | `ph2d-tool-painter` · `ph2d-wet-paint` | 1 · 5 | 4,8 · 0,2 | 13,63→12,07 |
| `o_splash` | `ph2d-tool-painter` | 3 | 0,5 | 12,07→11,26 |
| `a_pincelada_nova_a_cruzar_as_gravadas_nao_deixa_resquicio` | `ph2d-app-painter` | 1 | 12,4 | 11,26→9,52 |
