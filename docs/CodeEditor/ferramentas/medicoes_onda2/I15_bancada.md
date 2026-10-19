# I15 — a bancada tecla→pixel com a célula completa (19/10)

Sonda `bancada::tecla_pixel` (`crates/ph2d-app-code/tests/it/bancada.rs`, `#[ignore]`). Saída inteira das três
corridas: [I15_bancada_bruto.md](I15_bancada_bruto.md). Re-correr:
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-code --test it bancada -- --ignored --nocapture`
(precisa de `bash scripts/luau-lsp-fetch.sh`).

## A célula nova (`Lista::Real`)

O que faltava desde o I4/I5/I11/I9: uma célula com **tudo aberto ao mesmo tempo**, contra o assistente de
verdade (`IntelServer::Pinned`, `luau-lsp` 1.70.1) e não contra o servidor falso.

- A linha-alvo vira `math.max(0, dt )` com o local `dtxwmzk`, para a lista continuar viva a cada letra.
- Ajuda de assinatura aberta, cores semânticas e dicas de tipo LIGADAS (51 linhas: 5 dicas; 7 500 linhas:
  297 dicas e 293 diagnósticos do assistente).
- Por tecla a sonda **afirma** que a lista E a assinatura estão pintadas (célula que perde um dos dois pára, não mede
  outra coisa).
- O balão de hover/problema fecha na tecla por desenho (I5/I8b): o balão que vive durante a escrita é o da
  assinatura.

## Resultado

3 corridas (`loadavg` 9–40, perfil `smoke` otimizado, RTX 5060 Ti Vulkan), n = 1 000 por célula, células
intercaladas. Tecla→pixel, ms:

| Célula | p50 (3 corridas) | p99.9 |
|---|---|---|
| **completa**, 51 linhas, vista ~43 | **1,41–1,80** | ≤ 7,1 |
| **completa**, 7 500 linhas, vista ~43 | **1,66–2,08** | ≤ 9,7 (pior corrida = `load` 40) |
| lista falsa («lista aberta»), 51 / 7 500 | 1,20–1,52 / 1,18–1,57 | ≤ 6,3 / ≤ 7,4 |

Tudo dentro de UM quadro a 60 Hz (16,7 ms), com folga de ~8× no p50 e de 1,7× no pior p99.9.

Fases em p50 (pintura): 51 linhas **0,65–0,81 ms**, 7 500 linhas **0,82–1,01 ms**. O servidor ao vivo custa
~+0,15–0,3 ms de pintura sobre a célula falsa (cores + dicas + a assinatura); com 7 500 linhas há **+0,17 ms**
a mais, que cresce com a CONTAGEM dos diagnósticos/dicas (293/297), não com as linhas visíveis (43 em ambos).

Inalterados face ao [I1](I1_tecla_fora_do_reparse.md): Ctrl+S → selo (51 linhas 0,05–0,07 ms; 7 500 linhas
2,7–3,3 ms) e as tabelas da gramática (reparse, `folds`, realce de 100 linhas).

**Regra de Fatin** (7 500 cabe no ruído do par de réplicas das 51 linhas): a cena PARADA passa em p50 nas
três corridas; os quantis de cauda (p99/p99.9) da cena A TOCAR reprovam na corrida 1 — como já no I1 — e as caudas
trocam de veredito de corrida para corrida (corrida 2: só o p99; corrida 3: o p99.9 da parada e o p50 da a tocar,
`+0,024`). O ruído do par é 0,6–1,0 ms nas caudas. Não é efeito do tamanho do ficheiro: o excesso muda de sítio.

## O que as curas fizeram (commit `aca1bc0ed1`)

Três varreduras por linha/diagnóstico foram curadas antes da corrida 3:

1. ponto da calha por `partition_point` sobre `problems` ordenado por linha;
2. globais desconhecidos extraídos UMA vez por quadro e cortados pela busca;
3. tokens semânticos ORDENADOS na chegada (o `partition_point` de `spans` assumia a ordem do servidor).

**Não moveram o +0,17 ms:** corrida 2 (antes) 0,822 ms × corrida 3 (depois) 0,847 ms, a 7 500 linhas.
Causa suspeita, **não medida**: `CodeDoc::problems()` reconstruído e ordenado a cada quadro (`doc.rs` ~359), com a
fusão em `problems.rs` ~217–262 em O(D×R). Fica aberto no [handoff](../../handoffs/HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-19_ONDA2_I15.md) §9.

## ⛔ Recusas MEDIDAS

| Ideia | Porque não |
|---|---|
| Curar as varreduras por linha (calha, globais desconhecidos, cores) para tirar o +0,17 ms a 7 500 linhas | medido sem efeito (0,822 → 0,847 ms): o custo não está nelas; a suspeita é o `problems()` por quadro, ainda por medir |
| Esperar calma para fechar a regra de Fatin nas caudas | as caudas trocam de veredito entre corridas com o ruído do par em 0,6–1,0 ms; o p50 e o limite do quadro (16,7 ms) são a régua que decide |
| Medir a lista com o servidor falso e chamar a isto «a bancada completa» | o falso não pinta cores, dicas nem diagnósticos: a célula real custa +0,15–0,3 ms de pintura (e +0,2–0,5 ms de tecla→pixel) |
