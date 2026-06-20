# SESSION_ACTIVE — coordenação leve (DIRETRIZ §1.1)

**Propósito:** post-it compartilhado da posse VIVA da orquestração multi-agente —
quem está escrevendo o quê AGORA, para evitar colisão de git entre agentes
paralelos. **Modelo atual (DIRETRIZ §6.8):** 1 Coordenador (absorve PR/CI/ship) +
N Implementadores; os Implementadores **leem antes de cada burst** e não escrevem aqui.

**Não é log histórico nem fonte de estado.**
- Estado por-módulo (waves/tasks) → **CLAUDE.md §5**.
- Contratos congelados → **CLAUDE.md §6**.
- Histórico (o que já fechou) → **git log** + `docs/HANDOFF_*` / `docs/archive/`.
- Entradas concluídas saem daqui para o git log. **Limpe ao encerrar a sessão.**

---

## Estado da orquestração

**Sem sessão multi-agente ativa.** Nenhum slot de implementador aberto; sem posse
reservada. Próximo trabalho parte de CLAUDE.md §5 (planos ativos) + §1 (roteador por tarefa).

> Ao abrir uma sessão: registre aqui um **MAPA DE POSSE** (agente → pasta(s) que vai
> escrever, zero overlap) antes do primeiro burst, e o limite de RAM (≤3 cargos
> simultâneos). Ao fechar: mova o que concluiu para o git log e volte este arquivo
> ao estado idle acima.
