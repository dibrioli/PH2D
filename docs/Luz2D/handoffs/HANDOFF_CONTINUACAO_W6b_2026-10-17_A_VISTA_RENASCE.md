# Handoff de continuação — Luz 2D, W6b: a vista renasce quando a placa a degrada (2026-10-17)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` reaberta sobre o
> `main` `ad5f8fe39c` (W5x–W6a integradas em 17/10). Item 1 do §3 da [W6a](HANDOFF_CONTINUACAO_W6a_2026-10-17_O_EVENTO_DO_DRIVER.md):
> a cura C1, com o kill-criterion escrito lá ANTES de medir. Placa RTX 5060 Ti, `wgpu` 29.0.4. Commits:
> `fe49f1a384` (C1) · `c249604f7c` (C1b, variante) · o deste handoff (C1b é a única; a C1 sai). **C1b PASSA.**

## 1. A cura C1 (`fe49f1a384`)

- `quadro_renasce.rs` (irmão do `quadro.rs`): o `Vigia` lê a `c2_total()` que o governador já lê a cada quadro
  (o bloco `tempos_sem_esperar` do `gravar_com`, só com `preferencia` — o editor tem sempre). Assinatura: uma
  leitura ≥ 2× o piso da geração e, nas 30 seguintes, o mínimo ≥ piso + max(0,3 ms, 3 %) ⇒ `resolvido = None`
  (o mesmo caminho do `RENASCE`); `nasceu()` a cada montagem; no máximo 2× por processo; linha
  `[luz2d] a placa degradou a vista …` no terminal.
- `PH2D_LIGHT2D_RENASCE_SE_DEGRADA=0` desliga, lida em execução (a régua ON × OFF no mesmo binário).
- Portões: 6 testes do vigia (por leitura: o evento, pico sem degradação, degradação sem pico, a margem maior das
  duas, o teto de 2, o piso por nascimento) · `a_vista_renascida_da_os_mesmos_bits_que_a_nova` (placa: High,
  holográfica em textura, cena 5, a renascida = a nova ao bit após K = 3 quadros; controlo: K+1 ≠ K).

## 2. A régua da C1 (2560×1080 High, cena 5, 6 000 quadros, `CUSTO=1 SERIE=1`, sem `RENASCE`)

20 processos, intercalados ABBA sobre os limpos; **limpo** = nenhum processo na tabela INTEIRA do `nvidia-smi`
(C e G) além de kwin, Xwayland, plasmashell, portal, VSCode e o próprio app, durante todo o processo, e 60 s de
placa livre antes. 8 sujos (testes de placa da Macros e da PainterWatercolor, apps da UIUX), descartados.

| | c2 do último terço (ms), os 6 limpos |
|---|---|
| **cura ON** | 11,802 · 11,638 (sem evento) · 11,863 · 11,819 · 11,886 · 11,799 |
| **cura OFF** | 12,511 · 12,873 · 12,475 · 12,928 · 12,527 · 12,459 |

- O evento aparece em 11/12 limpos, entre 2,7 e 55 s depois do nascimento (a W6a dizia 0,3–5 s); a assinatura
  é a da W6a (≈ 30 e ≈ 54 ms). Cada um dos 5 ON com evento renasce **1×**, sempre depois de um pico.
- **Veredito pelo critério da W6a §3: NÃO PASSA** (1/6 ≤ 11,65 — o único sem evento) **e NÃO MORRE** (nenhum ON
  ≥ 12,2; ≤ 2 renascimentos; os bits iguais). A cura tira ~0,7 dos ~0,9 ms; fica ligada por omissão (estritamente
  melhor: o pior ON 11,886 < o melhor OFF 12,459).
- **O resto (+0,15–0,25 ms):** a cada renascimento o alocador do app passa de **12 para 16–17 blocos** com os
  mesmos 2 230 MiB, e a memória da placa +1 GB que nunca volta; sem renascimento, 12 blocos o processo inteiro.
  A vista nova monta-se no quadro seguinte, com a velha ainda viva no driver (os envios em voo): nasce noutra
  memória — o regime de alocação da W5x.
- Sob contenção (Chrome, outras linhas) o vigia dispara falsos positivos (até 2×, o teto): renascer custa uma
  montagem; ficou dentro do critério.
- Medições descartadas: 2560×1440 (a régua é 2560×1080) e as que tinham o Chrome na placa — o 1.º detector só via
  a lista de COMPUTAÇÃO do `nvidia-smi` (o Chrome é só G).

## 3. C1b — o renascimento solta a vista velha antes de montar a nova (kill-criterion escrito ANTES de medir)

A vista larga-se, a placa espera (`device.poll(wait)`) que o driver a destrua, e a nova monta-se depois, no mesmo
sítio. Régua: a da §2 (C1b × OFF, 6 + 6 limpos, intercalados). **Passa** se C1b 6/6 ≤ 11,65, os blocos do alocador
depois do renascimento = antes (12), e sem ela ≥ 1 processo com o evento (≥ 12,2). **Morre** se algum C1b ≥ 12,2,
renasce > 2× ou a imagem muda. Se nem passa nem morre: fica a melhor das duas pela mediana dos 6.

**Medido** (12 processos, 12 limpos, `PH2D_LIGHT2D_RENASCE_SE_DEGRADA=2` no binário de `c249604f7c`):

| | c2 do último terço (ms) | blocos antes → depois |
|---|---|---|
| **C1b** | 11,522 · 11,497 · 11,533 · 11,486 · 11,483 · 11,485 | 12 → 12 (6/6) |
| **OFF** | 12,567 · 12,909 · 12,496 · 12,617 · 12,615 · 12,972 | 12 → 12 |

Evento em 12/12 (2,8–89 s depois do nascimento); cada C1b renasce 1×, depois do pico. **PASSA**: 6/6 ≤ 11,65, os
blocos iguais, OFF 6/6 ≥ 12,2, ≤ 2 renascimentos, os bits iguais (o portão corre o mesmo `renascer`). A C1b ficou o
único caminho (`device.poll(wait)` depois de largar a vista, no fim do quadro); a C1 sai do código.

## 4. ABERTO, por ordem

1. Itens 2–3 do §4 da W5z: o berço e o nascer 2× só servem o caminho em buffer (retirar é decisão de produto) ·
   «bloco inteiro × páginas» (W5x §5.2, só noutra placa) · W5w §8 itens 2–4.
2. O piso do vigia inclui a 1.ª leitura de cada nascimento, sistematicamente mais baixa (11,24 contra ~11,7 depois;
   §2): a assinatura dispara mais cedo sob contenção. Medido sem consequência na régua (≤ 1 renascimento nos limpos);
   não mexido — a assinatura é a do kill-criterion.
3. Os outros solvers (o entorno) não renascem: o evento mede-se na `c2` da vista (W6a §2).

## 5. ⛔ Recusas MEDIDAS (somam às da W6a §4, W5z §5, W5y §5 e W5x §8)

| Recusa | Medida |
|---|---|
| C1: renascer sem soltar a vista velha (a nova monta-se no quadro seguinte) | Não passa: 11,80–11,89 com evento (1/6 ≤ 11,65); o alocador 12 → 16–17 blocos e +1 GB que não volta (§2). |
| Um detector de «alheios» só pela lista de COMPUTAÇÃO do `nvidia-smi` | Cego ao Chrome (só G): a contenção dele leu-se como c2 de 15–18 (§2). |

## 6. Para o integrador

Sem contrato congelado, sem schema. API nova `doc(hidden)`: `LuzDoQuadro::renascer_a_vista` (o portão). Ficheiros
novos: `src/quadro_renasce.rs`, `tests/it/a_vista_renasce_com_os_mesmos_bits.rs`. Variável nova:
`PH2D_LIGHT2D_RENASCE_SE_DEGRADA=0` (desliga). LOC: `quadro.rs` 697/700 (só a fiação: o campo, a montagem, a leitura
e o fim do quadro) · `quadro_renasce.rs` ~200. CLAUDE.md §5.1 Luz 2D: «Último:» para este handoff.
Roteiros da régua (fora do repo): laço ABBA sobre `mede_custo_no_app.sh` com o detector da tabela INTEIRA do
`nvidia-smi` (C e G, com lista branca da área de trabalho) e 60 s de placa livre antes de cada processo.
