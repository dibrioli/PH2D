# ADR-0033 — `ph2d-expr`: substrato de compute compartilhado (Fields + escape textual → WGSL|Luau)

**Status:** Accepted (ratificado pelo Enio 2026-05-21; implementação pendente)
**Data:** 2026-05-21
**Decisor(es):** Enio + Claude (arquiteto).
**Depende de:** ADR-0030, ADR-0032.

## 1. Contexto

O Houdini ensina que uma **camada de compute compartilhada** (VEX, autorada visualmente por VOPs, compilada no cook) reusada em todos os contextos é o que evita reimplementar math/noise/conditionals por domínio. O Blender melhora o mecanismo: **Fields / atributos anônimos** — compute lazy por-elemento que viaja inline num socket, sem a sub-rede VOP separada (mais intuitivo para artista). No protótipo MiniCavalry isto aparece como "value-plugs / árvore de expressão".

## 2. Decisão

Criar `ph2d-expr` como o "VEX/VOP" da PH2D, com três princípios:

1. **Fields inline, não sub-rede VOP** — o poder por-elemento vive dentro do grafo principal.
2. **Escape textual obrigatório desde o dia 1** ("code node"): nó = fluxo de **dados/composição**; texto = fluxo de **controle/expressão/iteração**. Força texto quando o nó codificaria loop, aritmética densa ou hot-loop ("17 linhas de texto > dezenas de nós ilegíveis").
3. **Lowering plural**: a mesma expressão compila para **WGSL** (domínios GPU: shader, campos de motion) ou **Luau/bytecode** (domínios CPU: gameplay, value-plugs). Um nó é **spec de operação abstrata + N lowerings**, não código de um runtime só.

`ph2d-expr` permanece **mínimo** (expressão, não linguagem completa). Luau continua a linguagem de gameplay (ADR-0019).

## 3. Consequências

**Aceitas:**
- Acaba com o "Wrangle quádruplo" (a mesma operação reimplementada por domínio) — fonte única de verdade pra compute.
- O escape textual é a mitigação primária do limite de escala visual (a lição VEX/wrangle + match-mismatch conjecture).

**Riscos:**
- `ph2d-expr` virar uma segunda linguagem completa → manter mínima; controle de fluxo de gameplay vive em Luau, não em `ph2d-expr`.
- Divergência entre lowerings WGSL e Luau da mesma op → testes de paridade golden por op.

## 4. Alternativas consideradas

- **Sub-rede VOP separada (Houdini puro):** rejeitado — força o artista a entrar numa rede separada pra escrever compute; Fields inline são mais intuitivos.
- **Só nós visuais, sem escape textual:** rejeitado — é o erro que gera espaguete; texto vence em iteração/aritmética/hot-loop.
- **Só texto (sem Fields):** rejeitado — perde a composabilidade visual e o live-preview por nó (ADR-0038).
