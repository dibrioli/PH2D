# CodeEditor — decisões do dono (15/10/2026)

> Respostas do Enio às quatro decisões de produto do
> [relatório-plano](reports/Editor%20de%20scripts%20estado%20da%20arte.md) §1. Decisões técnicas seguem o
> padrão-ouro e vivem no ADR da linha. Quem mudar uma destas pergunta ao dono antes.

| # | Pergunta | Decisão | Consequência no plano |
|---|---|---|---|
| 1 | Onde mora o script | **Dentro do projeto** (`.ph2dproj`) | O texto do script passa a ser dado do projeto: degrau novo de `PROJECT_SCHEMA` (formato gravado ⇒ suíte imprescindível). O `LuauScript` deixa de apontar um caminho e passa a apontar um script do projeto por id durável. Projetos antigos (caminho) importam o arquivo no load, sem perda; o caminho em falta vira recusa legível, nunca script vazio. Exportar para arquivo continua possível. |
| 2 | Levar o programa ajudante (`luau-lsp`, ~13 MB por sistema) | **Sim** | `luau-lsp` fixado como processo-filho empacotado (ADR: não é plugin de runtime, ADR-0075). Plano B (shim de diagnósticos no processo) só se o spike da Onda 0 reprovar. |
| 3 | Como a IA entra | **Desligada por enquanto. Depois entra da forma mais completa e poderosa possível — no FIM da implementação** | A Onda 6 passa a ser a ÚLTIMA onda executada e deixa de ser «tutor por omissão»: entra completa (tutor, escrita verificada, agente com MCP da cena), sempre verificada contra o `ph2d.d.luau` e com prévia + um passo de undo. Até lá, zero superfície de IA. |
| 5 | (16/10, plano da Onda 2) Ir para a definição, cores por significado (tokens semânticos), paleta `@`/`:`/`>` e `--!strict` por ficheiro — que o pedido da Onda 2 não listava | **Incluir as quatro nesta onda** | Entram na Onda 2 ([03_onda2.md](03_onda2.md)), com prova cada uma. |
| 6 | (16/10) Formatar ao salvar: StyLua (MPL-2.0, padrão do mundo Luau) como programa à parte, ou formatador nosso | **Levar o StyLua como mais um programa ajudante fixado, como o `luau-lsp`** | Processo-filho NÃO modificado, versão + sha256 por SO num manifesto (molde do `luau-lsp.toml`); a obrigação MPL é indicar onde está o fonte (créditos/licenças). Nunca vendorar nem modificar o código dele. |
| 4 | Script aponta outro objeto pelo nome (arrastar da Hierarchy, como no Godot) | **Sim — e tudo o que torne mais fácil, rápido e intuitivo** | Reverte a regra «scripts só falam por sinais» (handoff SCRIPT 16/09 §9): precisa de ADR. A referência é durável (`StableId`/nome, nunca bits — CLAUDE.md §5.1 Editor/shell), sobrevive a renomear o objeto e é verificada pelo editor (nome inexistente = diagnóstico). Arrastar da Hierarchy, do Inspector (propriedade, componente) e de assets gera código tipado. |
| 7 | (17/10, I4 da Onda 2) A tecla que abre a lista de sugestões à mão — `Ctrl+Espaço` (VS Code, Godot) já é o modo foco do app | **`Ctrl+Espaço` só dentro do editor de scripts** | Com o TEXTO do script em foco abre a lista na hora; na busca, na lista de scripts e no resto do app continua a ser o modo foco ([03_onda2.md](03_onda2.md), Estado I4). |
