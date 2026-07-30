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

## Convenção de leitura

| Se sua tarefa é… | Leia, nesta ordem |
|---|---|
| **começar a implementar (é aqui)** | [[03.3-HANDOFF-implementador-W0]] |
| construir o MVP web | [[03.1-PROMPT-MVP]] → [[03.2-Contrato-de-porte-para-Rust-WGPU]] |
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
- **[[01.2-Avaliacao-de-viabilidade]]** · `visao` `pesquisa` `viabilidade` — *A avaliação original: as 7
  peças que a PH2D já tem, por que o 3D do Photoshop morreu, e o inventário que diz que dá.*
- **[[01.3-Referencias-de-mercado]]** · `visao` `pesquisa` `benchmark` — *Nomad, ZBrush, Blender,
  SculptGL, Dreams, Clip Studio: o que cada um resolve, o que copiamos e o que recusamos.*

### 02 — Arquitetura

- **[[02.1-Representacao-malha-primaria]]** · `arquitetura` `decisao` — *A malha é a representação
  primária e o campo (SDF) é auxiliar. **Revisa o ADR-0145**; explica o que mudou e por quê.*
- **[[02.2-Sprite-com-malha-filha]]** · `arquitetura` `decisao` `runtime` — *O modelo do objeto misto:
  sprite pai, malha filha, o G-buffer no meio, e as duas rotas de runtime (bake × ao vivo).*
- **[[02.3-Modulo-removivel-e-mapa-de-crates]]** · `arquitetura` `crates` `removivel` — *As crates novas,
  a feature flag única, e o procedimento literal de remoção se o módulo não vingar.*

### 03 — MVP web

- **[[03.1-PROMPT-MVP]]** · `mvp` `prompt` `especificacao` — **O GRANDE PROMPT.** *Especificação
  completa do MVP em TypeScript + WebGPU: dados, motor, verbos, primitivas, topologia, shader, UI,
  input, undo, arquivo, gates e entregável. Auto-contido: copie o arquivo inteiro.*
- **[[03.2-Contrato-de-porte-para-Rust-WGPU]]** · `mvp` `porte` `contrato` — *O que do MVP é
  permanente (WGSL, layouts, matemática dos verbos, formato) e o que é descartável (DOM, TS).*
- **[[03.3-HANDOFF-implementador-W0]]** · `mvp` `handoff` `plano` — **O briefing para COMEÇAR.**
  *Os dois blocos a colar, os 10 marcos com risco na frente, decisões já tomadas, o que fazer ao
  empacar, e o formato do reporte.*

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
  pagamento, os riscos com a solução de cada um, e os alvos de performance a MEDIR (não a supor).*

### 99 — Meta

- **[[99.1-Taxonomia-e-convencoes]]** · `meta` `taxonomia` — *A taxonomia de tags, o frontmatter
  obrigatório e as regras de escrita deste cofre.*

## Documentos irmãos fora do cofre

- [`ADR-0145`](../architecture/decisions/0145-3d-layer-is-a-field-that-donates-shading-not-an-embedded-dcc.md)
  — a decisão de arquitetura registrada no repositório (revisada por [[02.1-Representacao-malha-primaria]]).
- [`CLAUDE.md`](../../CLAUDE.md) — o roteador operacional do projeto inteiro.
- [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) — as Hard Rules (HR-1..HR-18).
