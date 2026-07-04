---
name: project-diretriz-v68-2026-05-22
description: DIRETRIZ.md v6.8 (pós-ADR-0030..0040) — Tool↔Nó simétricos via §3.8 unificado; §3.9 deletada; §3.1 redirect; trip de auditores adversariais pegou 4 críticos no stub de painel novo (§3.2 não compilava)
metadata: 
  node_type: memory
  type: project
  originSessionId: 63f0541b-2d6a-46d4-a2c7-353454ee98fe
---

# DIRETRIZ v6.8 — 2026-05-22

Reescrita do `docs/IntegracaoMultiAgente/DIRETRIZ.md` (1146 LOC → 1093 LOC) para refletir
TOTAL coerência com a arquitetura pós-[[project-tool-isolation-freeze-2026-05-22]] (ADR-0040
fechado em TG-A..TG-E).

**Why:** A v6.7 (rascunho rápido do FREEZE) tinha duplicação massiva entre §3.8 (node fan-out)
e §3.9 (tool fan-out) — duas seções espelhadas, ~280 LOC, que diziam a mesma coisa com nomes
trocados. A simetria que acabamos de codificar tornava essa duplicação anti-DRY: amanhã
alguém edita um e esquece o outro. O Enio questionou o tamanho (1100+ LOC) e autorizou
condensação (Opção 1 dos 4 propostos).

**How to apply:** quando próximo agente entrar na PH2D, ler a DIRETRIZ inteira é o ponto-zero.
A v6.8 vai sustentar até o próximo evento arquitetural grande (mudança de contrato, novo
balde de crate, ou nova família tipo node/tool).

## Mudanças estruturais (v6.7 → v6.8)

1. **§3.8+§3.9 unificados em §3.8 "Fan-out drop-crate (A) — node OU tool"**:
   - §3.8.1 tabela node↔tool (pasta / codegen / wiring gerado / gate / contrato impl /
     cap arch-gate / entry points / vocab de canal / membrana / templates / pegadinhas).
   - §3.8.2 briefing parametrizado pronto-pra-colar com placeholders `<family>` +
     blocos `[node]` / `[tool]` (convenção: quem cola apaga o bloco da família errada).
   - §3.8.3 tabela 3 sabores de tool (one-shot stateless / palette modal / stateful+panel).
   - §3.8.3.1 **heads-up importante**: `ImageEditTool` está congelado no contrato mas
     **zero tools de produção implementam hoje** (BgRemoval/Padding usam métodos próprios
     via `as_any_mut` downcast). Migração pra esse canal é fan-out futuro, não scope creep
     de tool nova.
   - §3.8.4 garantia sem-colisão (uma só, vale pras duas famílias).
   - §3.8.5 checklist do revisor com seção comum + node-específica + tool-específica.

2. **§1.4 Triagem** promove "Tool nova ⇒ (A) Implementador-só" (era (B) Coord+Impl no v6.7
   por inércia textual). Só Painel/Widget/Chrome ficam em (B) caminho invertido.

3. **§1.2 reescrito** como "Dois caminhos: fan-out drop-crate vs fluxo invertido" — o antigo
   "fluxo invertido como mudança fundamental" virou caso particular do (B).

4. **§3.1 reduzida a redirect histórico** pro §3.8 (todo o scaffold pré-ADR-0040 + os 14
   substeps do Coord saíram). `editor-core/src/tools/` foi deletado em TG-D `c4063b7`.

5. **§3.5 "modificar existente"** ganhou mapa pasta-canônica-por-feature. Nota explícita:
   se LLM antiga apontar pra `editor-core/src/tools/`, ignore — confie no `ls`.

6. **§3.6 foundational** tabela simétrica de **2 contratos congelados** (sistema de nós
   + sistema de tools) com arquivos, caps, ADR, procedimento de bump.

7. **§4 gates table**: adicionou `architecture_tool_contract_surface` 🔒 + `architecture_contract_surface` 🔒
   + staleness gates de ambas as families + `cycle_prevention` corrigido pra "3 invariantes + 1 smoke".

8. **§9.2 caminhos físicos canônicos**: Tool/Node lado-a-lado como (A); contratos congelados
   explicitados; registry-init marcados GERADO; entries para codegen-crates `tools/ph2d-{tool,node}-sync`
   + workspace.members glob + shell init (3 linhas, 2 chamadas).

9. **§10 referências** lista ADRs 0027/0028/0029 (background) + 0030..0040 (sistema de nós +
   tools) + briefings §3.8 + planos closed.

10. **§11 histórico enxuto**: só v6.7 + v6.8 detalhados; v6.0..v6.6 resumidos em 1 linha
    + `git log`. Header também encurtado (era 30+ linhas de changelog inline).

## Auditoria adversarial (2 agentes paralelos, lentes distintas)

**Lens 1 (coerência com código):** rodou ~85 verificações contra `grep`/`Read`. Pegou
**4 críticos** no §3.2 "Painel novo" (stub NÃO COMPILAVA):
- Assinatura `Panel::paint` errada (2 params, não 3; `PanelHostInternal`, não `PanelHost`).
- `hash_node_id` está em `ph2d-tool-registry`, não em `ph2d-a11y`.
- API de registro do panel registry é `reg.push(ErasedPanel::new::<T>())`, não `build.register::<T>()`.
- `ImageEditTool` vendido como caminho ativo de produção quando nenhum tool real implementa.

Mais 4 Altos (clippy command sem `--features ph2d-spike/bevy_ecs` no §9.3; `cycle_prevention`
descrito como 4 sub-checks quando são 3 invariantes + 1 smoke; "alfabético" afirmado para
panel-registry-init e chrome sem gate) + 5 Médios + 12 Baixos.

**Lens 2 (coerência interna):** 0 críticos, 6 altos. Pegou ambiguidade A6 — §1.1 dizia "Coord
convocado apenas para (B)/(C)" mas §7 dizia "Coord sempre faz push+babysit". Resolvido com nota:
"Quando jornada foi 100% (A), Enio promove um dos Implementadores a Coord no momento de ship —
ship é sempre serializado por uma única sessão." Também: número de surfaces/gates errados no §1.2
quando comparado a §3.8.1 (3 staleness gates valia só pra tool, não pra node). E "Pingo dolor
especial" — texto garbled escapou na tabela §3.8.1.

**Todos os Crítico/Alto/Médio aplicados.** Baixos cosméticos ignorados (typos opinativos,
agrupamento opcional de refs).

## Métricas finais
- LOC: 1146 → 1093 (corte de 53 LOC; menor que projetado ~400 porque o briefing parametrizado
  ficou denso pra cobrir bem as duas famílias, e a tabela §3.8.1 + §3.6 foundational ganharam
  bastante conteúdo).
- §3.9 só aparece 2× (ambas no histórico §11 — confirmado por `grep -c §3\.9`).
- 0 referências mortas (`editor-core/src/tools/` nunca aparece como caminho ativo).

## Pendências
- Outro agente em vôo durante a edição (9 arquivos staged que não são meus). Commit final foi
  ESCOPADO via `git commit -- docs/IntegracaoMultiAgente/DIRETRIZ.md` pra não pegar staging
  alheia — vide [[feedback-scoped-commit-shared-index]].
- Sem push (Enio faz ship).
