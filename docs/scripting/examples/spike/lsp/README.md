# C5 — luau-lsp + ph2d.d.luau autocomplete fixtures

Manual verification (LSP é interativo — não automatizável puramente em CI).

## Setup esperado em VSCode

1. Instalar extensão `luau-lsp` (JohnnyMorganz.luau-lsp).
2. Settings (`.vscode/settings.json` no workspace):
   ```json
   {
       "luau-lsp.types.definitionFiles": [
           "docs/scripting/examples/spike/llm-tests/ph2d.d.luau"
       ],
       "luau-lsp.diagnostics.strictDatamodelTypes": true,
       "luau-lsp.completion.autocompleteEnd": true
   }
   ```
3. Abrir cada `case_*.luau` neste diretório.

## 5 casos canônicos (per docs/spike/2026-05-plan.md L86-89)

Para cada caso, validar manualmente:
- Hover em variáveis mostra tipo correto (não `any`).
- Autocomplete sugere campos válidos da API ph2d.
- Erros de tipo aparecem em red squiggle quando proposital.

| # | Arquivo | Caso |
|---|---|---|
| 1 | `case_1_query_ecs.luau` | Query ECS — `ph2d.query` retorna QueryResult tipado |
| 2 | `case_2_coroutine.luau` | Coroutine com `ph2d.wait` |
| 3 | `case_3_system.luau` | `ph2d.system` callback com `dt: number` |
| 4 | `case_4_message.luau` | `ph2d.message_handler` com payload |
| 5 | `case_5_fsm.luau` | FSM via state_table com flags tipadas |

## Threshold

5/5 casos: tipo correto sugerido sem fallback `any`. Validação manual com screenshot ou demonstração ao Enio na revisão de Semana 3.

## Status

**Pendente verificação manual.** Eu (Claude) gerei os 5 fixtures e o `ph2d.d.luau`; instalação do luau-lsp e validação visual no VSCode é parte de aceitação humana — `cargo run` não captura UX de autocomplete.

Per L181-182 do plano, **C5 é cortável** (substituível por verificação manual em editor) — não bloqueia outros critérios.
