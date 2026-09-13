---
description: Troca de janela / retomada pós-integração.
argument-hint: [Nome da linha] [Handoff a ler] [Próximo item]
---
Você está assumindo a linha `$1`, que já existe.

Execute o BLOCO de docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md com
`line/$1`, FASE 0 → 1 → 2: `cd` na worktree + `pwd` + `git branch --show-current` ANTES de
abrir qualquer arquivo; depois o `git cherry` da FASE 1 (uma linha integrada por rebase tem
hashes velhos, e um `git rebase main` cego reaplica-os) e só então o rebase.

Estado / handoff: $2

Me reporte, antes de tocar em código:
1. Em que árvore você está (saída literal do pwd + branch).
2. O que já existe e NÃO deve ser reconstruído.
3. O que está aberto, na ordem em que você pretende atacar.

Próximo item: $3
