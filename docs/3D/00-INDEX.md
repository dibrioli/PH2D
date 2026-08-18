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
LINHA:   line/sculpt3d  — ABERTA de novo em 2026-08-10, em Worktrees/line-sculpt3d, a partir
         do main 76788440a e com ZERO commits proprios. (A anterior integrou e foi encerrada
         no mesmo dia; esta e outra.) Assumindo-a? HANDOFF_CONTINUACAO_..._2026-08-10.
CÓDIGO:  W1..W18 INTEGRADAS — o estado vivo do módulo é o CLAUDE.md §5, não este bloco.
MEDIDO:  a pegada manda (10x a malha = 0,79x o dab). K1/K2 disparam só no pincel gigante,
         e 88% do custo é DESCOBRIR a vizinhança, não as normais. Decisão: HANDOFF_INTEGRACAO
MUDANÇA: 2026-07-30 — o MVP em TypeScript foi CANCELADO (ver 02.4)
```

## Convenção de leitura

| Se sua tarefa é… | Leia, nesta ordem |
|---|---|
| **ASSUMIR A LINHA E IMPLEMENTAR (comece aqui)** | [[HANDOFF_CONTINUACAO_line_sculpt3d_2026-08-18]] — onde a linha está e o que está ABERTO com o preço ao lado (⚠️ ele supersede os de 2026-08-06 e 2026-08-10; **a lista aberta de um handoff envelhece a cada jornada**, e as duas anteriores já perderam 3 de 4 e depois 3 de 4 itens) → [[06.1-Waves-riscos-e-alvos]] (o roteiro) → [[03.8-HANDOFF-implementador-W4]] (o protocolo) |
| **mexer em QUALQUER ferramenta / pincel** | [21_plano_modos_e_ferramentas.md](21_plano_modos_e_ferramentas.md) — o plano dos 3 modos (S/B/L), do Basic×Pro e das ferramentas que faltam; ele **supersede** o [[06.1-Waves-riscos-e-alvos]] no que toca a tools |
| **saber por que um tool nosso difere da referência** | [20_divergencias_tools.md](20_divergencias_tools.md) — D1-D27, os negativos, o catálogo nos três apps e o padrão-ouro |
| **portar QUALQUER COISA do SculptGL** | [[03.4-Referencia-SculptGL]] (a política + o livro-razão) → [[03.7-Oraculo-de-fidelidade]] (o protocolo) → [19_paridade_sculptgl.md](19_paridade_sculptgl.md) (o estado da paridade, kernel a kernel) |
| **saber o que vem agora** | [21_plano_modos_e_ferramentas.md](21_plano_modos_e_ferramentas.md) §7 (as waves) · [[06.1-Waves-riscos-e-alvos]] para o resto do módulo |
| entender a decisão inteira | `ADR-0150` → [[02.1-Representacao-malha-primaria]] |
| **saber onde o motor roda (CPU × GPU)** | [[03.5-Onde-roda-o-motor]] |
| o briefing histórico da W1 | [[03.6-HANDOFF-implementador-W1]] |
| saber por que não há protótipo web | [[02.4-Por-que-nao-ha-MVP-em-TypeScript]] |
| decidir arquitetura | [[02.1-Representacao-malha-primaria]] → [[02.2-Sprite-com-malha-filha]] → [[02.3-Modulo-removivel-e-mapa-de-crates]] |
| implementar um pincel | [[04.1-Pinceis]] |
| implementar primitivas / blocagem | [[04.2-Primitivas-e-blocagem]] |
| implementar remesh / topologia | [[04.3-Topologia]] |
| mexer na UI da cena 3D | [[04.4-O-painel]] |
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

- **[[03.4-Referencia-SculptGL]]** · `porte` `arquitetura` `fidelidade` — **O documento central do porte.**
  *A licença e o que ela obriga; a **POLÍTICA de fidelidade** (o default é PORTAR — divergir custa uma
  entrada escrita); o **LIVRO-RAZÃO** com toda divergência que sobreviveu à refutação adversarial,
  marcada `✅ deliberada` / `⚠️ dívida` / `⛔ defeito`; e o mapa arquivo→crate com o tamanho medido de
  cada bloco.*
- **[[03.7-Oraculo-de-fidelidade]]** · `fidelidade` `gate` — **Por que ler o original não basta.**
  *67 alegações de divergência, 37 refutadas: 55% de falso-positivo, medido. As três formas de oráculo
  que funcionam aqui (fixture que contém o fenômeno · oráculo de propriedade · transcrição congelada),
  e por que rodar o JS deles NÃO é o plano.*
- **[[03.5-Onde-roda-o-motor]]** · `decisao` `performance` — **A decisão que o fim do MVP forçou.** *Malha
  na CPU com rayon, GPU dona do render e da doação, porta única, e kill-criterion escrito antes do build.*
- **[[HANDOFF_INTEGRACAO_line_sculpt3d_W1_2026-07-30]]** · `handoff` `integracao` —
  **O que a W1 entregou e o que ela MEDIU.** *Identidade da linha, o que foi tocado fora do
  módulo, os símbolos que podem colidir, e a tabela K1/K2 com a decisão que ficou para o Enio.*
- **[[03.8-HANDOFF-implementador-W4]]** · `handoff` `plano` `fidelidade` — **O briefing de QUEM ASSUME A
  LINHA AGORA.** *Onde você está e o `cd` que não se pula; o que já foi feito e não se refaz; e os seis
  itens da W4 com `arquivo:linha`, a cura, o gate red-first com o número que ele dá HOJE, a fixture e a
  armadilha de cada um — os seis auditados por um cético, os seis **corrigidos**.*
- **[[03.6-HANDOFF-implementador-W1]]** · `handoff` `plano` — ⚠️ **HISTÓRICO** (a W1 fechou e integrou).
  *O briefing original: o bloco a colar, a ordem dos marcos e o formato do reporte.*

### 04 — Ferramentas

- **[[04.1-Pinceis]]** · `ferramenta` `pincel` `matematica` — *Os verbos com a matemática de cada um — a CONTAGEM não é citada aqui de propósito (`Verb::ALL` é a fonte; um número escrito ao lado de uma lista que cresce envelhece calado),
  a lei do traço, falloffs, simetria e alphas.*
- **[[04.2-Primitivas-e-blocagem]]** · `ferramenta` `primitiva` `zsphere` — *O conjunto paramétrico
  completo (incluindo o superelipsóide, o curinga do "qualquer bloco") e o Skeleton tipo ZSphere.*
- **[[04.3-Topologia]]** · `ferramenta` `topologia` `remesh` — *Voxel remesh, topologia dinâmica,
  multiresolução, decimação e booleanas: qual algoritmo, quando roda, quanto custa.*
- **[[04.4-O-painel]]** · `ferramenta` `ui` `painel` — *A UI da cena (W12): as seis seções, a tabela
  única de knobs, o anel do cursor que é também o instrumento do pick, e o que segue no teclado.*

### 05 — Shading

- **[[05.1-Shader-de-runtime]]** · `shading` `pbr` `sss` `runtime` — *O shader "nível Unreal, custo de
  jogo": GGX multiscatter, SSS pré-integrado, AO, Cavity, matcap, e a cadeia de post.*
- **[[05.2-Doacao-de-sombreamento-para-2D]]** · `shading` `2d` `integracao` — *Como a normal/AO/cavidade
  da malha chega às camadas do Painter e ao sprite do jogo, com o rig de luz único.*

### 06 — Plano

- **[[06.1-Waves-riscos-e-alvos]]** · `plano` `waves` `performance` `risco` — **O plano vivo, re-cortado
  pelo censo medido (2026-08-01).** *Cada wave é UMA coisa que o artista abre e usa, nomeia os
  arquivos-fonte que traduz e traz o tamanho em linhas; os três CANAIS que destravam famílias inteiras;
  os riscos com solução; e os alvos a MEDIR.*

### 99 — Meta

- **[[99.1-Taxonomia-e-convencoes]]** · `meta` `taxonomia` — *A taxonomia de tags, o frontmatter
  obrigatório e as regras de escrita deste cofre.*

## Handoffs de integração (um por entrega da linha)

⚠️ **A coluna que envelhece é a de ESTADO** — um handoff que ainda diz *"pendente de smoke"* depois
de a wave ter integrado faz a próxima LLM desconfiar de um verde legítimo.

| Handoff | O que entrega | Estado |
|---|---|---|
| [[HANDOFF_INTEGRACAO_line_sculpt3d_W1_2026-07-30]] | **W1, a MALHA** — malha residente, octree, normais, o passe wgpu, o kernel de pincel; a tabela K1/K2 | ✅ **no `main`** |
| [[HANDOFF_INTEGRACAO_line_sculpt3d_W2_2026-07-31]] | **W2, o BARRO** — o pick, a lei do traço, os verbos, máscara, simetria, octree e upload incrementais, undo e o gesto | ✅ **no `main`** |
| [[HANDOFF_INTEGRACAO_line_sculpt3d_W3_2026-07-31]] | **W3, a DOAÇÃO** — a malha doa a normal e a tinta chapada sai acesa pela forma | ✅ **no `main`** |
| [[HANDOFF_INTEGRACAO_line_sculpt3d_W4-W8_2026-08-02]] | **W4..W8.2** — o traço honesto · a malha que puxa · a resolução · o remesh · **a cena é uma LISTA** | ✅ **no `main`** |
| [[HANDOFF_INTEGRACAO_line_sculpt3d_W8.7_2026-08-04]] | **W8.3..W8.7** — o documento · import · export · **o OBJETO MISTO** · ★ **os canais no DOCUMENTO** (a rota A) | 🟡 **fechada, smoke OK 2026-08-04, aguarda ordem de integração** |

## Documentos irmãos fora do cofre

- [`ADR-0150`](../architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md)
  — a decisão de arquitetura registrada no repositório. **Reescrito em 2026-07-30**; é ele que vence se
  divergir de qualquer nota daqui.
- [`CLAUDE.md`](../../CLAUDE.md) — o roteador operacional do projeto inteiro.
- [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) — as Hard Rules (HR-1..HR-18).
