# ADR-0038 — UX baseline de nós artista-primeiro (esconder contexto, viewport-first, presets, ferramentas terminais)

**Status:** Accepted (ratificado pelo Enio 2026-05-21; implementação pendente)
**Data:** 2026-05-21
**Decisor(es):** Enio + Claude (arquiteto), com investigação dedicada de UX.
**Estende:** ADR-0023 (UI/UX baseline) ao domínio de nós.
**Depende de:** ADR-0030, ADR-0031.

## 1. Contexto

O objetivo do Enio é uma engine **super potente E intuitiva para artistas** — a tensão central. A investigação de UX trouxe a verdade desconfortável: **nós não REDUZEM carga cognitiva, eles a REALOCAM** (sintaxe → raciocínio espacial). "Fazer com nós" ≠ "intuitivo". Houdini é modelo de **poder**, não de **usabilidade** (curva notoriamente íngreme). Os sistemas intuitivos (Blender Geometry Nodes pós-fields, TouchDesigner, Cables, Origami, Substance) convergem em padrões que Houdini viola.

## 2. Decisão

Adotar **7 princípios de UX como gates** (não opcionais), e tratar UX de nó como preocupação arquitetural de 1ª classe:

1. **Viewport-first, graph-second** — artista faz 80% das edições sem abrir o grafo (gizmos no viewport editam parâmetros de nó).
2. **Live-preview em TODO nó**, não só na saída.
3. **Progressive disclosure** via sub-grafos colapsáveis nomeados (sub-grafo com params expostos = preset/tool).
4. **Presets / nós-compostos result-named como porta de entrada**; primitivos são o escape hatch achado depois.
5. **Result-named, zero jargão** em nós **e portas** (o "artista-primeiro" da PH2D; gate de UI string).
6. **Wiring restrito anti-espaguete** — portas tipadas+coloridas que **recusam conexão inválida**; auto-layout, reroute, comment frames.
7. **Escape textual ("code node") opcional, não a estrada principal.**

**Esconder "contexto" totalmente:** contexto = o editor que você abriu, nunca um modo dentro do grafo; **zero taxonomia SOP/VOP na UI**; **paleta por-editor** (só os nós daquele domínio).

**Ferramentas imperativas são TERMINAIS, não rampa pro grafo** (ADR-0031): para pintar/mascarar/retocar são a ferramenta *correta*, não versão inferior do nó. Bridge bidirecional nó↔manipulação-direta.

## 3. Consequências

**Aceitas:**
- O "artista-primeiro" do projeto vira moat de UX defensável por gate.
- Múltiplos domínios não aterrorizam o artista porque "contexto" nunca é um conceito que ele carrega na memória de trabalho.

**EVITAR a todo custo:** trabalho de baixo nível obrigatório · taxonomia de contexto exposta · jargão de implementação · onboarding de canvas-em-branco-de-primitivos · paleta-mega única · posicionar nós como prestígio e ferramentas como rodinhas.

## 4. Alternativas consideradas

- **Modelo de UX do Houdini (poder primeiro, descoberta por tutorial):** rejeitado — é o anti-exemplo de usabilidade; copiar só o poder.
- **VEX/wrangle obrigatório pra ser produtivo:** rejeitado — o escape textual é opcional (ADR-0033), não pré-requisito.
- **Um canvas único com todos os nós:** rejeitado — tab-menu-explosion; viola "esconder contexto" e a navegabilidade (alinhado ADR-0030).
