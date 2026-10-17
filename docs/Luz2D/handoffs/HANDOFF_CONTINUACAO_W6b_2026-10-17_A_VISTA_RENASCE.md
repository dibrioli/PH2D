# Handoff de continuação — Luz 2D, W6b: a vista renasce quando a placa a degrada (2026-10-17)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` reaberta sobre o
> `main` `ad5f8fe39c` (W5x–W6a integradas em 17/10). Item 1 do §3 da [W6a](HANDOFF_CONTINUACAO_W6a_2026-10-17_O_EVENTO_DO_DRIVER.md):
> a cura C1, com o kill-criterion escrito lá ANTES de medir. Placa RTX 5060 Ti, `wgpu` 29.0.4. **EM CURSO.**

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
