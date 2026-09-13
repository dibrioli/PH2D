# Registros de jornada de integração (arquivo)

> **O que é esta pasta.** Registros de **uma jornada específica** — o que o integrador daquele dia
> viu, resolveu e mediu. Saíram de [`docs/IntegracaoMultiAgente/`](../../IntegracaoMultiAgente/) em
> 2026-08-18 porque aquela pasta é o **processo vivo** (diretivas, modelos, guias que um agente novo
> lê para trabalhar), e um registro datado não é processo.
>
> ⚠️ **Não são instruções.** Descrevem o mundo no dia em que foram escritos. O processo corrente é a
> [`DIRETRIZ.md`](../../IntegracaoMultiAgente/DIRETRIZ.md) + a
> [`DIRETIVA_IMPLEMENTACAO.md`](../../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md).

| Arquivo | O quê |
|---|---|
| [HANDOFF_INTEGRADOR_jornada_2026-08-08.md](HANDOFF_INTEGRADOR_jornada_2026-08-08.md) | Handoff do integrador da jornada de 2026-08-08 |
| [REGISTRO_integracao_jornada_2026-07-13.md](REGISTRO_integracao_jornada_2026-07-13.md) | Registro da integração de 2026-07-13 (citado por 5 handoffs de módulo) |
| [NOTAS_INTEGRACAO_vector_cutover_2026-07-06.md](NOTAS_INTEGRACAO_vector_cutover_2026-07-06.md) | Notas do cutover do Vector, 2026-07-06 |

## Movidos em 2026-09-13 — a W2, a auditoria de arquitectura e a refatoração final

Em ordem de rodada. Todos eram registos de UMA rodada: blocos já colados, handoffs já integrados, o
estado de um dia.

| Arquivo | O quê |
|---|---|
| [BRIEFING_INTEGRADOR_2026-09-10.md](BRIEFING_INTEGRADOR_2026-09-10.md) | Briefing do integrador da rodada de 10/09 (seis linhas) |
| [BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md](BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md) | Briefings das linhas L0–L5 da W2 («partir a shell») |
| [BLOCOS_ABERTURA_W2_2026-09-11.md](BLOCOS_ABERTURA_W2_2026-09-11.md) | Os seis blocos de abertura da W2 |
| [BLOCOS_ABERTURA_W2_FASE_B_2026-09-11.md](BLOCOS_ABERTURA_W2_FASE_B_2026-09-11.md) | Os cinco blocos de reabertura da Fase B |
| [BLOCOS_REABERTURA_W2_FASE_B2_2026-09-12.md](BLOCOS_REABERTURA_W2_FASE_B2_2026-09-12.md) | Os três blocos da 2.ª volta (motion · vec · flip) |
| [BLOCO_ABERTURA_LINHA_FOLHAS_2026-09-12.md](BLOCO_ABERTURA_LINHA_FOLHAS_2026-09-12.md) | Abertura da `line/shell-folhas` (as folhas partilhadas que prendiam três famílias) |
| [HANDOFF_INTEGRACAO_line_shell_folhas_2026-09-12.md](HANDOFF_INTEGRACAO_line_shell_folhas_2026-09-12.md) | Handoff da `line/shell-folhas` |
| [BLOCOS_REABERTURA_W2_FASE_C_2026-09-12.md](BLOCOS_REABERTURA_W2_FASE_C_2026-09-12.md) | Blocos da Fase C (3.ª rodada) |
| [BLOCOS_REABERTURA_W2_FASE_D_2026-09-12.md](BLOCOS_REABERTURA_W2_FASE_D_2026-09-12.md) | Blocos da Fase D (4.ª rodada) |
| [ESTADO_W2_2026-09-12.md](ESTADO_W2_2026-09-12.md) | ⭐ O estado da W2 ao fim de 12/09, com os acrescentos da refatoração final (13/09): §4 as leis, §6 o que ficou aberto e a velocidade de compilação medida |
| [AUDITORIA_ARQUITETURA_2026-09-12.md](AUDITORIA_ARQUITETURA_2026-09-12.md) | Auditoria de arquitectura do fim da W2 (o grafo medido, os achados, as ondas propostas) |
| [BLOCOS_ABERTURA_ARQUITETURA_2026-09-12.md](BLOCOS_ABERTURA_ARQUITETURA_2026-09-12.md) | Abertura da `line/editor-core` + `line/render-loop` |
| [HANDOFF_INTEGRACAO_line_editor_core_2026-09-12.md](HANDOFF_INTEGRACAO_line_editor_core_2026-09-12.md) | Handoff A5b + A10 — os ids descem para as crates que os lêem; módulos da fundação em DAG |
| [HANDOFF_INTEGRACAO_line_render_loop_2026-09-13.md](HANDOFF_INTEGRACAO_line_render_loop_2026-09-13.md) | Handoff A9 + o quadro — o `run_render_frame` vira um índice de fases |
| [BLOCOS_ABERTURA_REFATORACAO_FINAL_2026-09-13.md](BLOCOS_ABERTURA_REFATORACAO_FINAL_2026-09-13.md) | Abertura das três linhas da refatoração final |
| [HANDOFF_INTEGRACAO_line_input_dispatch_2026-09-13.md](HANDOFF_INTEGRACAO_line_input_dispatch_2026-09-13.md) | Handoff da `line/input-dispatch` (cliques, arrastos, roda e teclas) |
| [HANDOFF_INTEGRACAO_line_render_bodies_2026-09-13.md](HANDOFF_INTEGRACAO_line_render_bodies_2026-09-13.md) | Handoff da `line/render-bodies` |
| [HANDOFF_INTEGRACAO_line_loc_caps_2026-09-13.md](HANDOFF_INTEGRACAO_line_loc_caps_2026-09-13.md) | Handoff da `line/loc-caps` (o resto das listas de LOC) |
| [briefing-node-crate.md](briefing-node-crate.md) | Briefing antigo do fan-out de node-crate — o processo vivo é a [DIRETRIZ §3.A](../../IntegracaoMultiAgente/DIRETRIZ.md) + [`examples-fan-out.md`](../../IntegracaoMultiAgente/examples-fan-out.md) |

### ⚠️ O que estes registos ainda têm de ABERTO

Arquivar não fecha nada. Antes de abrir uma linha nova sobre a shell ou a fundação, leia:

- [ESTADO_W2 §3](ESTADO_W2_2026-09-12.md) — *o que FALTA, e o que decide cada peça*;
- [ESTADO_W2 §6](ESTADO_W2_2026-09-12.md) — *o que está ABERTO para além da W2*;
- [AUDITORIA §6](AUDITORIA_ARQUITETURA_2026-09-12.md) — *o que a jornada FEZ com a lista* (e o que não fez).

## Por que foram movidos

Medido em 101 sessões de agente (transcripts de `~/.claude/projects/`), estes três somam
**1 leitura** — contra 138 da `DIRETIVA_IMPLEMENTACAO.md` e 90 da `DIRETRIZ.md`. Ocupavam 23 KB numa
pasta de 11 arquivos para a qual o roteador do [`CLAUDE.md §1`](../../../CLAUDE.md) manda o agente
novo — o mesmo custo navegacional da parede de 208 handoffs na raiz de `docs/`
([DIRETRIZ §1.5.9](../../IntegracaoMultiAgente/DIRETRIZ.md)): *plano e registro de sessão
indistinguíveis*.

⚠️ **O ganho é navegacional, não de tokens** — eles não eram lidos, então mover não economiza
contexto. Está escrito aqui para ninguém "medir de novo" esperando ver o número cair.
