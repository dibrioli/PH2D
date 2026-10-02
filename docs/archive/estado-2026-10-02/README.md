# `CLAUDE.md` — a história até 2026-10-02 (arquivo)

> **O que é esta pasta.** O texto do `CLAUDE.md` **verbatim** como estava no `main` @ `faad97fa5`,
> recortado por assunto: o núcleo (§0–§4 e a abertura do §5), as leis transversais (§5.0), um
> ficheiro por módulo (§5.1) e as §6–§7. Os cortes foram feitos pelo `scripts/doc-split.py`, que
> **aborta se as duas metades não remontarem o original por sha256** — nenhum corte abortou.
>
> ⚠️ **Isto NÃO é o estado atual de nada.** O vivo é o [`CLAUDE.md`](../../../CLAUDE.md) (o §5 é o
> roteador de cada módulo, com o link do último handoff de integração). Use este arquivo para
> responder ***«porque é que isto ficou assim?»*** — nunca para decidir a próxima ação.
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado.

## Porque ele existe — e porque é a SEGUNDA vez

Em 2026-08-18 o `CLAUDE.md` já tinha sido cortado de 917 KB para 41 KB
([`estado-2026-08-18/`](../estado-2026-08-18/README.md)). A regra que devia impedir a volta —
*«fechar uma linha escreve no handoff e edita UMA linha do §5»* — **contava linhas e não bytes**, e
nenhum gate a media. Em seis semanas cada «linha» virou um parágrafo de 5 a 40 KB:

| data | `CLAUDE.md` | contexto inicial de cada janela (medido nos transcripts) |
|---|---|---|
| 2026-08-19 | 58 KB | — |
| 2026-09-13 | 265 KB | 175 mil tokens |
| 2026-09-20 | 620 KB | 342 mil tokens |
| 2026-10-02 | **710 KB** (§5 = 675 KB; a entrada da Escultura sozinha 338 KB) | **380 mil tokens** |

E o preço era pago **por todos, a cada passo**: medido em set/2026 (148 mil chamadas), **82 % do
custo do Claude é reler o contexto**, a média era **606 mil tokens relidos por passo**, e este
arquivo era cerca de metade disso. Com ele acima de 200 mil tokens, **nenhum subagente de modelo
menor conseguia sequer começar** (o pedido estourava a janela antes da primeira palavra).

**A cura desta vez tem régua:** o gate
`architecture_claude_md_cabe_no_orcamento` (em `crates/ph2d-editor-core/tests/it/`) reprova o
`CLAUDE.md` acima de 40 KB e qualquer entrada de módulo do §5.1 acima de 700 bytes. A lista de
flakes de carga, que crescia dentro do §5.0, tem doc vivo próprio:
[`FLAKES_DE_CARGA.md`](../../DevOps/FLAKES_DE_CARGA.md).

⛔ **As recusas medidas do §5 estão indexadas em [`RECUSAS.md`](RECUSAS.md)** (109 trechos, derivados
por varredura) — o `CLAUDE.md` não tem tabela de recusas própria. ⚠️ O `contratos-s6-s7.md` não passou
pelo `doc-split.py` (é uma cópia das §6–§7, com os links reancorados); todos os outros sim.

<!-- INDICE-DERIVADO -->

## O que está nesta pasta

| arquivo (história, verbatim) | tamanho | o doc VIVO de onde saiu |
|---|---:|---|
| [`sculpt3d.md`](sculpt3d.md) | 357 KB | — |
| [`components.md`](components.md) | 98 KB | — |
| [`motion-nodes.md`](motion-nodes.md) | 60 KB | — |
| [`3dmodeling.md`](3dmodeling.md) | 51 KB | — |
| [`vector.md`](vector.md) | 33 KB | — |
| [`nucleo-s0-s5.md`](nucleo-s0-s5.md) | 33 KB | — |
| [`leis-transversais.md`](leis-transversais.md) | 31 KB | — |
| [`RECUSAS.md`](RECUSAS.md) | 26 KB | — |
| [`sprite-inspector.md`](sprite-inspector.md) | 14 KB | — |
| [`uiux.md`](uiux.md) | 9 KB | — |
| [`physics.md`](physics.md) | 8 KB | — |
| [`painter.md`](painter.md) | 8 KB | — |
| [`editor-shell.md`](editor-shell.md) | 6 KB | — |
| [`timeline.md`](timeline.md) | 4 KB | — |
| [`contratos-s6-s7.md`](contratos-s6-s7.md) | 4 KB | — |
| [`flip.md`](flip.md) | 4 KB | — |
| [`audio.md`](audio.md) | 3 KB | — |
| [`image-tools.md`](image-tools.md) | 3 KB | — |
| [`runtime.md`](runtime.md) | 3 KB | — |
| [`retirados.md`](retirados.md) | 2 KB | — |
| [`planos-de-nos.md`](planos-de-nos.md) | 2 KB | — |

**21 arquivos · 757 KB** de história fora do caminho quente.

> ⚠️ Cada recorte foi feito por `python3 scripts/doc-split.py`, que **aborta se as duas
> metades não remontarem o original byte-a-byte (sha256)**. Nenhuma linha foi editada.
>
> ⛔ As **recusas medidas** que viviam aqui continuam alcançáveis: cada doc vivo leva no
> fim uma tabela `⛔ Recusas MEDIDAS` com o link para a linha exata neste arquivo.
> *Arquivar sem indexar as recusas seria apagá-las.*
