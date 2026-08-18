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

## Por que foram movidos

Medido em 101 sessões de agente (transcripts de `~/.claude/projects/`), estes três somam
**1 leitura** — contra 138 da `DIRETIVA_IMPLEMENTACAO.md` e 90 da `DIRETRIZ.md`. Ocupavam 23 KB numa
pasta de 11 arquivos para a qual o roteador do [`CLAUDE.md §1`](../../../CLAUDE.md) manda o agente
novo — o mesmo custo navegacional da parede de 208 handoffs na raiz de `docs/`
([DIRETRIZ §1.5.9](../../IntegracaoMultiAgente/DIRETRIZ.md)): *plano e registro de sessão
indistinguíveis*.

⚠️ **O ganho é navegacional, não de tokens** — eles não eram lidos, então mover não economiza
contexto. Está escrito aqui para ninguém "medir de novo" esperando ver o número cair.
