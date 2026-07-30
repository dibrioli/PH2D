---
titulo: "PH2D Sculpt — cofre do módulo 3D"
tags: [modulo/3d, tipo/moc, status/ativo]
status: ativo
modulo: 3D
atualizado: 2026-07-30
resumo: "Porta de entrada do cofre Obsidian do módulo de escultura 3D da PH2D. Comece por aqui."
---

# 🗿 PH2D Sculpt — cofre do módulo 3D

> **Cofre Obsidian.** Abra esta pasta como *vault* no Obsidian:
> `Abrir pasta como cofre` → a pasta `docs/3D` do repositório.
> Para uma LLM: leia **[[00-INDEX]]** primeiro — é o índice de uma linha por nota.

## O que é este módulo

Uma ferramenta de **escultura 3D para artistas 2D**, embutida na PH2D, com duas razões de existir:

1. **O1 — Escultura que ilumina a pintura.** A forma 3D doa *normal · profundidade · AO · cavidade ·
   material* ao passe de luz que o Painter já tem, para o artista pintar sobre forma real em vez de
   imaginá-la.
2. **O2 — Forma 3D por baixo do sprite.** Um objeto de jogo é um **sprite que tem uma malha 3D como
   FILHA**; a malha empresta ao sprite seu shader avançado (PBR + SSS + AO + Cavity). O jogo continua
   2D e ganha iluminação 3D de verdade, em runtime.

**Referência de UX: [Nomad Sculpt](https://nomadsculpt.com).** Facilidade de uso acima de tudo — mais
fácil que o ZBrush, com o poder do ZBrush.
**Referência de CÓDIGO: [SculptGL](https://github.com/stephomi/sculptgl)**, do mesmo autor, **MIT** —
pode ser lida e adaptada ([[03.4-Referencia-SculptGL]]).

## Estado atual

```
DECISÃO: ADR-0145 (proposto, aguarda aceite do Enio)
         malha primária · direto em Rust/wgpu · SculptGL (MIT) de referência
LINHA:   line/sculpt3d  — branch e worktree JÁ existem
CÓDIGO:  nenhum ainda — a W1 (a malha) é a primeira
```

⚠️ **Mudança de rumo em 2026-07-30 (Enio):** o plano de 29/07 abria com um **MVP em TypeScript + WebGPU**.
Ele foi **cancelado** — temos um aplicativo de escultura completo e MIT como referência, então o protótipo
seria pagar para aprender o que já está escrito. O registro, com o que a decisão deixou em aberto, está em
[[02.4-Por-que-nao-ha-MVP-em-TypeScript]].

## Mapa do cofre

| Pasta | O que vive lá |
|---|---|
| [[01.1-Missao-objetivos-e-nao-objetivos\|01-Visao]] | por que o módulo existe, o que ele **não** é, e o mercado |
| [[02.1-Representacao-malha-primaria\|02-Arquitetura]] | as decisões estruturais, o mapa de crates, e a morte do MVP |
| [[03.6-HANDOFF-implementador-W1\|03-Implementacao]] | **a referência, a decisão CPU/GPU, e o briefing para começar** |
| [[04.1-Pinceis\|04-Ferramentas]] | pincéis, primitivas, blocagem tipo ZSphere, topologia |
| [[05.1-Shader-de-runtime\|05-Shading]] | o shader de runtime e a doação de sombreamento ao 2D |
| [[06.1-Waves-riscos-e-alvos\|06-Plano]] | waves, riscos com solução, alvos de performance |
| [[99.1-Taxonomia-e-convencoes\|99-Meta]] | taxonomia de tags e convenções de escrita |

## Início rápido por intenção

- **"Quero COMEÇAR agora."** → [[03.6-HANDOFF-implementador-W1]] (o bloco a colar).
- **"Onde o motor de escultura roda?"** → [[03.5-Onde-roda-o-motor]] (CPU, com kill-criterion).
- **"O que posso copiar do SculptGL?"** → [[03.4-Referencia-SculptGL]] (e o que **não** posso).
- **"Por que malha e não campo?"** → [[02.1-Representacao-malha-primaria]].
- **"Como o sprite 2D recebe a luz 3D?"** → [[02.2-Sprite-com-malha-filha]] + [[05.2-Doacao-de-sombreamento-para-2D]].
- **"E se não der certo?"** → [[02.3-Modulo-removivel-e-mapa-de-crates]].
- **"Quanto isso custa em frame?"** → [[06.1-Waves-riscos-e-alvos]].

## Regras da casa que valem aqui

Este cofre está **dentro** do repositório da PH2D e obedece ao [`CLAUDE.md`](../../CLAUDE.md):
o alvo é o extraordinário e **o teto é o do hardware** (§0.0 — nenhum `MAX_*` entra sem a tabela de
medição ao lado), contrato congelado não se toca sem ADR (§6), e feature nova é **drop-crate** (§0.1).
