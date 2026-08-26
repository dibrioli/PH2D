---
name: feedback-subtracting-two-clocks-from-separate-runs-gives-the-sum-of-the-noises
description: Um A/B lido de duas corridas separadas devolve números contraditórios sobre o MESMO código — as duas configurações têm de correr no mesmo processo, por mediana.
metadata:
  type: feedback
---

Para saber quanto o anti-serrilhado custava, subtraí **dois relógios de ~30 ms** medidos em
**corridas separadas** para ler um delta de ~10 ms. Ele devolveu **`+34 %` numa corrida e `+22 %`
noutra, sobre o mesmo código** — e eu quase concluí que uma mudança tinha ganho quando ela era
neutra.

**Why:** *subtrair dois números ruidosos não dá um número menos ruidoso: dá a soma dos dois ruídos.*
E nesta workstation a deriva entre corridas é grande de propósito (a mesma montagem já mediu
`14,4` e `22,1 ms`), o que torna o delta menor que o erro.

**How to apply:** a porta de medição recebe a **configuração como argumento**, e a sonda corre as
duas no **mesmo processo**, `N` vezes cada, reportando a **mediana** — com uma corrida de
aquecimento fora da conta (a primeira paga a montagem a frio). O molde vivo é
`ph2d_field_render::trace_stepped_for_test` / `measure_the_edge_pass_share`.

⚠️ **A lição já estava escrita no ARQUIVO que eu estava a editar**, no doc-comment da porta irmã:
*"ela existe para que as duas respostas sejam medidas no mesmo processo — um A/B nessas condições
mede o relógio da máquina, não a mudança"*. Ler o ficheiro não é o mesmo que estar do lado certo da
regra que ele contém ([[feedback-reading-the-rule-is-not-the-same-as-being-on-the-right-side-of-it]]).

⛔ Parente próximo: a família de flake de recurso sob fan-out (`CLAUDE.md` §5.0) — *toda leitura de
relógio desta workstation não vale nada acima de `load ~5`*.
