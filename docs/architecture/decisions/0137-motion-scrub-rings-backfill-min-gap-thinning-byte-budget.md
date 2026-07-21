# ADR-0137 — os rings de scrub aprendem: backfill + thinning por gap-mínimo com janela recente protegida + orçamento em BYTES

- **Status:** aceito (implementado nesta linha, `line/gpu-nodes`)
- **Data:** 2026-07-21
- **Contexto:** C1 da fila §E da auditoria
  (`docs/HANDOFF_line_gpu_nodes_auditoria_RESULTADO_2026-07-20.md` §A2 + §B2);
  irmão de ADR-0127 D5 (o ring GPU) e do M2.N2 (o ring CPU).

## O problema (medido, §A2)

Os dois rings de scrub — `ph2d-eval-motion::CheckpointRing` (CPU, denso) e
`ph2d-gpu-cook::GpuCheckpointRing` (GPU, stride 8) — só gravavam tick
**estritamente à frente** do fundo do ring, e evictavam **o mais velho**. Num
LOOP, a composição é uma armadilha permanente: tocado além do fim, a janela
senta na cauda; o wrap para `lo` ancora no seed 0 e re-simula a história
inteira; o re-sim **não grava nada** (todo tick ≤ fundo) — e a próxima volta
repete, para sempre. Medição comitada
(`scrub_tests::loop_wrap_resims_the_whole_history_every_wrap`): loop
`[100, 400]`, volta 1 = 101 evals, volta 2 = **101 de novo**. O custo cresce
com a POSIÇÃO do loop, não com o tamanho; na neve CPU (22,3 ms/tick a 262k) um
loop no tick 3600 custaria ~80 s de freeze por volta.

**Os call sites do backfill já existiam nos dois lados** — o replay do scrub
consulta `should_record`/`record` por tick (CPU `lib.rs:401`, GPU `cook()`); a
regra "estritamente à frente" era o que os matava. A reforma mora INTEIRA
dentro dos dois tipos de ring; pump e sequencer não mudam uma linha.

## Decisões

### 1. Backfill ordenado: grava-se o que NÃO está presente, em qualquer direção

`should_record(tick)` vira *"o tick (na grade, no GPU) ainda não está no
ring?"*; `record` insere NA POSIÇÃO (busca binária), nunca só no fim. A volta 1
de um loop reconstrói a cobertura durante o próprio re-sim; a volta 2 ancora
onde a 1 gravou.

### 2. Evicção = THINNING por gap-mínimo, com a janela recente PROTEGIDA

O mais-velho-primeiro é exatamente errado para um loop (derruba as âncoras que
o wrap vai pedir). A vítima agora é a entrada **mais redundante**: a que, ao
sair, cria o MENOR gap entre vizinhos (`t[i+1] − t[i−1]`; a primeira entrada usa
o seed virtual do tick 0 como vizinho esquerdo). Isso adelgaça a história
uniformemente — um alvo qualquer fica a ≤ meio-gap de uma âncora — em vez de
amputá-la de um lado.

**Protegidas** (nunca vítimas de thinning): as entradas de tick mais ALTO —
até `RECENT_DENSE` delas, **e nunca mais que METADE do ring vivo** — no CPU (a
garantia de hoje, 300 ticks ≈ 5 s densos, fica intacta sempre que o orçamento
comporta ≥ 600 entradas; sob pressão de bytes o ring se DIVIDE entre recência e
história adelgaçada) e a entrada mais nova no GPU; e a recém-gravada, nos dois
(a regra que o ring GPU já tinha). ⚠️ A meia-divisão é estrutural, não detalhe:
a 1ª versão protegia por CONTAGEM pura e a fase espremida do gate O(1) starvou
101/101 de novo — com poucas entradas a proteção engolia o ring inteiro e só
sobrava o fallback oldest-first, a doença original vestida de reforma. Último
recurso (nenhum candidato): evicta o mais velho — fallback, nunca política.

### 3. Orçamento em BYTES nos DOIS rings (§B2 — contagem é multiplicador)

O ring GPU já capava em bytes (VRAM). O CPU capava em CONTAGEM
(`RECENT_CAPACITY = 300`) — a classe ADR-0117: 300 checkpoints de uma cena de
262k elementos são ~GB, e a regra não piscava. Agora `CookCheckpoint` responde
`approx_bytes()` (soma das colunas dos streams — estimativa, documentada como
tal) e o ring CPU aceita um orçamento em bytes (`CPU_RING_BYTES = 128 MB`, a
mesma classe do GPU `RING_BYTES`): cena pesada ganha **janela mais curta**,
nunca conta maior. `MAX_ENTRIES = 2048` fica como *backstop* de custo de
inserção (o insert ordenado desloca O(n)), não como orçamento — está uma ordem
acima de qualquer espalhamento real.

## Consequências

- A volta 2+ de um loop ancora na cobertura da volta 1: o custo cai de
  O(posição do loop) para O(gap na âncora) — **o gate é a própria medição
  virada** (`second ≤ 2` evals onde era `> 90`), rodando sem `#[ignore]`.
- Loop maior que o orçamento: thinning uniforme dentro do alcance — degrada em
  RESOLUÇÃO (gap maior), nunca de volta à re-sim completa.
- Scrub recente: garantia inalterada (a janela densa é protegida por regra).
- O seed do tick 0 continua implícito (ring vazio → `(0, default)`), então
  nenhum alvo é inalcançável — o fallback continua sendo o produto.
- ⚠️ O número deste ADR é PROVISÓRIO até a integração (dois donos já
  renumeraram ADRs neste repo duas vezes).
