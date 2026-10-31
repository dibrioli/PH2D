# Medida 31/10, rodada 9 — a projecção com o zero e a divergência numa passada (ao bit), base `5b356e4ff4`

Régua a das rodadas 7–8: wash 4096×2560 (montagem do `sonda_fluxo.rs` = `nucleos_wash`, `Lei::Cobre`), perfil
`smoke`, `PH2D_RELEASE="mede velocidade"` pela fatia da linha (CPU ≤ 1600 % de 32), `RAYON_NUM_THREADS=16`. Sonda
temporária `examples/sonda_rodada9_tmp.rs` (apagada, não comitada): wash pintado uma vez, `layers`+`sim` restaurados
antes de CADA corrida; quadros 1–78 por corrida (tempo dos 7–78), hash FNV de `film`/`susp`/`susp_rgb`/`vel_x` nos
quadros 6 · 42 · 78 (e conferido igual entre rondas); **8 rondas × 2 variantes intercaladas com a ordem rodada**,
override EM EXECUÇÃO `solver::PROJECT_FUNDE`; resumo = o MÍNIMO. loadavg 1 min 0,6 → 7,9 (só a sonda).

## O corte (caminho A, fases pequenas)

`project_rows` fazia quatro passadas: zerar `div`/`prs` (faixa viva + margem 1), divergência, pressão, correcção.
A primeira só escreve; a segunda escreve `div` outra vez. Agora as duas são UMA passada por linha: zera a linha de
`div`/`prs` e escreve a divergência das células activas da mesma linha. Ao bit por construção: cada linha só escreve
a sua linha de `div`/`prs`, e a divergência lê `vel` (que nada nesta passada escreve). A expressão da divergência é
a mesma closure (`div_at`) nas duas rotas. Poupa uma varredura de `div` (~8 B/célula, 1 passo em 3).

| variante (16 fios, ms/passo, mínimo de 8 rondas) | ms/passo | hash 6 / 42 / 78 |
|---|---|---|
| projecção em 4 passadas (`PROJECT_FUNDE = 0`) | 50,92 | `62643337cc9a0ebd` `2ba4433e9bc2449b` `78d0d1346b5a9ae5` |
| **zero + divergência fundidos (produto, `= 1`)** | **50,58** | idêntico |

⇒ **−0,34 ms/passo, ao bit** (a conta prevê ~0,3: 84 MB por chamada a ~47 GB/s, 1 em 3). Os hashes desta sonda
não se comparam com os das rodadas 7–8: a função de hash desta sonda foi reescrita (a das rodadas 7–8 foi apagada com a sonda);
o que vale é a igualdade entre as variantes e entre rondas.

## Gates (por nome, `smoke`)

`scripted_session_fingerprint_is_stable` (impressão gravada da sessão) · `the_parallel_projection_is_the_serial_projection_to_the_byte`
· `sim_resumable` (2) — verdes. Nenhum hash gravado muda; nenhuma miniatura de fábrica muda.

## O que NÃO se fez nesta rodada (prazo de 45 min) — e porquê

- **Caminho B (planos `f16`/`u16`)**: não começado. É a única alavanca grande que resta (advecção ~29,5 ms na banda),
  mas exige trocar o tipo de `vel_x/y`, `susp`, `susp_rgb` em todo o crate e a régua de tolerância de ≥ 300 quadros em
  três cenas (wash 4K, Thick Impasto, aguarela em camadas) com max ≤ 1 e p99,9 ≤ 0,5 níveis 8-bit — uma janela própria.
- **Fundir divergência + pressão + correcção por blocos** (como a advecção): a correcção escreve `vel` no próprio
  índice e o halo do bloco vizinho lê esse `vel` ⇒ precisa de `vel` de saída noutro plano (ou de `div` em rascunho por
  fio); o `div` global (= `flow_x`) fica como rascunho nas células que o fluxo seguinte não reescreve — se pode deixar de ser
  escrito não foi verificado. Ganho previsto ≤ ~1 ms/passo.
- **Escrita condicional no `smooth_velocity_rows`** (não escrever `flow_x/y` igual): `flow_x/y` é o rascunho
  `div`/`prs` da projecção 1 passo em 3 ⇒ quase nunca igual; não medido, recusado pela conta.
- **Menos iterações onde o resíduo está abaixo de um bit**: a projecção já é UMA relaxação de Jacobi; não há
  iterações a cortar.

## ⛔ Recusas MEDIDAS / por conta

| proposta | medida | veredito |
|---|---|---|
| escrever `flow_x/y` só quando muda (smooth) | `flow_x/y` é reescrito pela projecção 1 em 3 | ⛔ pela conta: quase nunca igual |
| cortar iterações da projecção | é 1 relaxação | ⛔ nada a cortar |

## Onde estamos

4K, 16 fios: **50,9 → 50,6 ms/passo** (ao bit). Rebelle 37,3. O que falta (~13 ms) é tráfego do estado na advecção:
o próximo corte é o caminho B (bytes, com a tolerância invisível provada) numa janela própria.
