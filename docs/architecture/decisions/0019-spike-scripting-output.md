# ADR-0019: Output do spike de scripting (2026-05)

**Status:** Proposed (placeholder — preenchido ao fim do spike, 2026-05-29)
**Data:** TBD
**Decisor:** Enio

## Contexto

`docs/spike/2026-05-plan.md` define um spike de 3 semanas validando a fundação da camada de scripting da PH2D: Luau strict via mlua, escolha de ECS, modelo de hot reload reset+restore, mensageria estilo Defold, storage lateral, coroutines como primitiva canônica.

A premissa existencial é **LLM como único programador**. Critérios C8, C15 e C16 testam fluência de LLM com a API; falha em qualquer um significa que o projeto não tem developer.

## Decisão

TBD — preenchido com:

1. **Veredicto:** commit (mantém arquitetura proposta) ou repensar (volta ao quadro branco com aprendizados específicos).
2. **ECS canônico** (resultado de C11; atualiza ADR-0003-rev2).
3. **Lista de critérios passa/falha:** tabela completa do spike com linhas vermelhas e plano por linha vermelha.
4. **Mudanças no SKILL.md** aplicadas: §11.7 reescrito para Luau, HR-16 e HR-17 integradas, §5 com versões pinadas finais.
5. **Aprendizados não previstos:** o que o spike ensinou que não estava no plano.

## Consequências

TBD.

## Alternativas consideradas

TBD — se veredicto for "repensar", listar aqui as alternativas exploradas no spike e por que cada uma falhou em qual critério.

## Inputs deste ADR

- `docs/spike/2026-05-plan.md` (plano operacional)
- `docs/spike/2026-05-report.md` (relatório final do spike)
- ADR-0003-rev2 (ECS choice — derivado deste spike)
- Branch `spike/scripting-foundation` (código preservado mesmo se branch for descartado)
