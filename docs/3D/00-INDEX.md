---
titulo: "Índice do cofre 3D (para LLM)"
tags: [modulo/3d, tipo/indice, status/ativo]
status: ativo
modulo: 3D
atualizado: 2026-07-30
resumo: "Uma linha por nota do cofre: caminho, tags e o que a nota responde. Leia isto antes de abrir qualquer outra."
---

# 00 — Índice do cofre (leia primeiro)

> **Para a LLM:** cada linha traz o **link**, as **tags** e a **pergunta que a nota responde**.
> Abra apenas as notas cuja pergunta case com a sua tarefa. Todas as notas têm frontmatter com
> `resumo:` — se precisar decidir sem abrir, leia o `resumo`.

## O estado, em quatro linhas

```
DECISÃO: ADR-0150 (proposto) — malha primária · escrito direto em Rust/wgpu · SculptGL (MIT) de referência
LINHA:   line/sculpt3d  (branch + worktree JÁ existem)
CÓDIGO:  W1 FECHADA, SMOKE APROVADO (2026-07-30) — 6 commits, rebaseada em cd8513b76,
         gate verde sobre a árvore combinada. Aguardando ORDEM DE INTEGRAÇÃO.
MEDIDO:  a pegada manda (10x a malha = 0,79x o dab). K1/K2 disparam só no pincel gigante,
         e 88% do custo é DESCOBRIR a vizinhança, não as normais. Decisão: HANDOFF_INTEGRACAO
MUDANÇA: 2026-07-30 — o MVP em TypeScript foi CANCELADO (ver 02.4)
```

## Convenção de leitura

| Se sua tarefa é… | Leia, nesta ordem |
|---|---|
| **começar a implementar (é aqui)** | [[03.6-HANDOFF-implementador-W1]] |
| entender a decisão inteira | `ADR-0150` → [[02.1-Representacao-malha-primaria]] |
| **saber onde o motor roda (CPU × GPU)** | [[03.5-Onde-roda-o-motor]] |
| usar o SculptGL como referência | [[03.4-Referencia-SculptGL]] |
| saber por que não há protótipo web | [[02.4-Por-que-nao-ha-MVP-em-TypeScript]] |
| decidir arquitetura | [[02.1-Representacao-malha-primaria]] → [[02.2-Sprite-com-malha-filha]] → [[02.3-Modulo-removivel-e-mapa-de-crates]] |
| implementar um pincel | [[04.1-Pinceis]] |
| implementar primitivas / blocagem | [[04.2-Primitivas-e-blocagem]] |
| implementar remesh / topologia | [[04.3-Topologia]] |
| implementar o shader | [[05.1-Shader-de-runtime]] |
| ligar o 3D ao 2D | [[05.2-Doacao-de-sombreamento-para-2D]] |
| planejar/priorizar | [[06.1-Waves-riscos-e-alvos]] |
| escrever uma nota nova | [[99.1-Taxonomia-e-convencoes]] |

## Todas as notas

### 01 — Visão

- **[[01.1-Missao-objetivos-e-nao-objetivos]]** · `visao` `objetivos` — *Por que este módulo existe, os
  dois objetivos (O1 painter, O2 sprite), e a lista explícita do que ele **não** é.*
- **[[01.2-Avaliacao-de-viabilidade]]** · `visao` `pesquisa` — ⚠️ **HISTÓRIA, não instrução.** *A avaliação
  original: as 7 peças que a PH2D já tem e a pesquisa que a fundamentou. O cabeçalho dela diz, seção por
  seção, o que envelheceu (§3 · §5 · §7).*
- **[[01.3-Referencias-de-mercado]]** · `visao` `pesquisa` `benchmark` — *Nomad, ZBrush, Blender,
  SculptGL, Dreams, Clip Studio: o que cada um resolve, o que copiamos e o que recusamos.*

### 02 — Arquitetura

- **[[02.1-Representacao-malha-primaria]]** · `arquitetura` `decisao` — *A malha é primária e o campo (SDF)
  é gerador. O detalhe do que morreu da decisão anterior e o preço da nova.*
- **[[02.2-Sprite-com-malha-filha]]** · `arquitetura` `decisao` `runtime` — *O modelo do objeto misto:
  sprite pai, malha filha, o G-buffer no meio, e as duas rotas de runtime (bake × ao vivo).*
- **[[02.3-Modulo-removivel-e-mapa-de-crates]]** · `arquitetura` `crates` `removivel` — *As crates novas,
  a feature flag única, e o procedimento literal de remoção se o módulo não vingar.*
- **[[02.4-Por-que-nao-ha-MVP-em-TypeScript]]** · `arquitetura` `decisao` `porte` — *A morte do protótipo
  web: o que ele de-riscaria, o que o SculptGL já de-risca, e para onde foram os riscos que sobraram.*

### 03 — Implementação

> ⚠️ Os números **03.1 – 03.3 estão VAGOS**: eram o capítulo do MVP web, removido em 2026-07-30. O cofre
> **não recicla número** ([[99.1-Taxonomia-e-convencoes]]) — um link a `03.1-PROMPT-MVP` escrito em
> qualquer lugar tem de continuar **quebrando**, e não resolvendo em silêncio para outro documento.

- **[[03.4-Referencia-SculptGL]]** · `porte` `arquitetura` — *O SculptGL como referência: licença MIT e a
  política de atribuição que ela obriga, a arquitetura real dele, o mapa arquivo→crate, e o que NÃO se
  porta (o undo, a lei do traço, o render).*
- **[[03.5-Onde-roda-o-motor]]** · `decisao` `performance` — **A decisão que o fim do MVP forçou.** *Malha
  na CPU com rayon, GPU dona do render e da doação, porta única, e kill-criterion escrito antes do build.*
- **[[HANDOFF_INTEGRACAO_line_sculpt3d_W1_2026-07-30]]** · `handoff` `integracao` —
  **O que a W1 entregou e o que ela MEDIU.** *Identidade da linha, o que foi tocado fora do
  módulo, os símbolos que podem colidir, e a tabela K1/K2 com a decisão que ficou para o Enio.*
- **[[03.6-HANDOFF-implementador-W1]]** · `handoff` `plano` — **O briefing para COMEÇAR.** *O bloco a
  colar, a ordem dos marcos, as decisões que não se re-litigam e o formato do reporte.*

### 04 — Ferramentas

- **[[04.1-Pinceis]]** · `ferramenta` `pincel` `matematica` — *Os 20 verbos com a matemática de cada um,
  a lei do traço, falloffs, simetria e alphas.*
- **[[04.2-Primitivas-e-blocagem]]** · `ferramenta` `primitiva` `zsphere` — *O conjunto paramétrico
  completo (incluindo o superelipsóide, o curinga do "qualquer bloco") e o Skeleton tipo ZSphere.*
- **[[04.3-Topologia]]** · `ferramenta` `topologia` `remesh` — *Voxel remesh, topologia dinâmica,
  multiresolução, decimação e booleanas: qual algoritmo, quando roda, quanto custa.*

### 05 — Shading

- **[[05.1-Shader-de-runtime]]** · `shading` `pbr` `sss` `runtime` — *O shader "nível Unreal, custo de
  jogo": GGX multiscatter, SSS pré-integrado, AO, Cavity, matcap, e a cadeia de post.*
- **[[05.2-Doacao-de-sombreamento-para-2D]]** · `shading` `2d` `integracao` — *Como a normal/AO/cavidade
  da malha chega às camadas do Painter e ao sprite do jogo, com o rig de luz único.*

### 06 — Plano

- **[[06.1-Waves-riscos-e-alvos]]** · `plano` `waves` `performance` `risco` — *As waves em ordem de
  pagamento (a doação é a W3, e depende só da W1), os riscos com a solução de cada um, e os alvos a MEDIR.*

### 99 — Meta

- **[[99.1-Taxonomia-e-convencoes]]** · `meta` `taxonomia` — *A taxonomia de tags, o frontmatter
  obrigatório e as regras de escrita deste cofre.*

## Documentos irmãos fora do cofre

- [`ADR-0150`](../architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md)
  — a decisão de arquitetura registrada no repositório. **Reescrito em 2026-07-30**; é ele que vence se
  divergir de qualquer nota daqui.
- [`CLAUDE.md`](../../CLAUDE.md) — o roteador operacional do projeto inteiro.
- [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) — as Hard Rules (HR-1..HR-18).
