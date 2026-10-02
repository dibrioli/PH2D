# ARQUIVO — CLAUDE.md (história, 15 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`CLAUDE.md`](../../../CLAUDE.md) em 2026-10-02, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-1558,1574-1585` do original.
>
> ⚠️ **A única alteração ao corpo:** 4 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

- **Runtime — a saída de sinais (R0)** — crate-folha `ph2d-runtime` (`Signal`/`SignalOrigin`/`SignalOutbox`/`SignalReader`),
  onde a **timeline** ([ADR-0143](../../architecture/decisions/0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md)) e a **física** (`SignalOnHit`) se encontram: os produtores
  **publicam** e cada consumidor lê com o próprio cursor (ADR-0075 — o produtor não chama ninguém).
  ⚠️ **A ordem no quadro é load-bearing e tem gate:** o quadro vira **antes** do primeiro produtor, o dreno roda **depois**
  dos dois — fora dessa janela o sinal chega um quadro atrasado (invisível num toast, visível quando o consumidor for **som**).
  ⚠️ **MEDIDO:** 8 consumidores custam **1,00×** o de 2 — o custo mora no produtor. *O trabalho do R3 é a tabela
  **nome → ação**, que é conteúdo autorado e precisa de UI, não o fan-out.*
  **Aberto:** ⚠️ **adjacência de NOME com uma linha viva** — o plano de UI/UX aponta `ph2d-runtime` para o *runtime de UI*.
  Hoje não há conflito; a decisão é do Enio (a linha recomenda **crate irmã**, senão o gate `the_event_core_is_a_leaf` é
  deliberadamente revogado) · o envelope por seções (**F1.W0**) **não existe no `main`** — é recuperável de `37ff53467`,
  mas **o desenho volta e o diff não** · **R1** (`shells/game`) segue adiado por decisão do Enio.
  **Smokes:** `PH2D_SIGNAL_SMOKE=1|2` (+ `PH2D_SIGNAL_LOG=1`).
  **Ler:** [`docs/Runtime/`](../../Runtime) · [handoffs](../../Runtime/handoffs/README.md) ·
  [história](../estado-2026-08-18/runtime.md)

