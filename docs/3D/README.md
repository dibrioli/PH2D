---
titulo: "PH2D Sculpt — cofre do módulo 3D"
tags: [modulo/3d, tipo/moc, status/ativo]
status: ativo
modulo: 3D
atualizado: 2026-07-30
resumo: "Porta de entrada do cofre Obsidian do módulo de escultura 3D da PH2D. Comece por aqui."
---

# 🗿 PH2D Sculpt — cofre do módulo 3D

> **Cofre Obsidian.** Abra esta pasta como *vault* no Obsidian: `Abrir pasta como cofre` →
> `/home/enio/Documentos/Projetos/PH2D/docs/3D`.
> Para uma LLM: leia **[[00-INDEX]]** primeiro — é o índice de uma linha por nota.

## O que é este módulo

Uma ferramenta de **escultura 3D para artistas 2D**, embutida na PH2D, com duas razões de existir:

1. **O1 — Escultura que ilumina a pintura.** A forma 3D doa *normal · profundidade · AO · cavidade ·
   material* ao [[01.2-Avaliacao-de-viabilidade|passe de luz que o Painter já tem]], para o artista
   pintar sobre forma real em vez de imaginá-la.
2. **O2 — Forma 3D por baixo do sprite.** Um objeto de jogo é um **sprite que tem uma malha 3D como
   FILHA**; a malha empresta ao sprite seu shader avançado (PBR + SSS + AO + Cavity). O jogo continua
   2D e ganha iluminação 3D de verdade, em runtime.

**Referência maior: [Nomad Sculpt](https://nomadsculpt.com).** Facilidade de uso acima de tudo — mais
fácil que o ZBrush, com o poder do ZBrush.

## Mapa do cofre

| Pasta | O que vive lá |
|---|---|
| [[01.1-Missao-objetivos-e-nao-objetivos\|01-Visao]] | por que o módulo existe, o que ele **não** é, e o mercado |
| [[02.1-Representacao-malha-primaria\|02-Arquitetura]] | as decisões estruturais e o mapa de crates |
| [[03.1-PROMPT-MVP\|03-MVP-Web]] | **o grande prompt do MVP** e o contrato de porte para Rust |
| [[04.1-Pinceis\|04-Ferramentas]] | pincéis, primitivas, blocagem tipo ZSphere, topologia |
| [[05.1-Shader-de-runtime\|05-Shading]] | o shader de runtime e a doação de sombreamento ao 2D |
| [[06.1-Waves-riscos-e-alvos\|06-Plano]] | waves, riscos com solução, alvos de performance |
| [[99.1-Taxonomia-e-convencoes\|99-Meta]] | taxonomia de tags e convenções de escrita |

## Estado atual

```
STATUS: proposta fechada · aguardando ordem do Enio para abrir a linha
LINHA:  line/sculpt3d (a criar)
ADR:    ADR-0145 (proposto, revisado por 02.1)
CÓDIGO: nenhum ainda
```

## Início rápido por intenção

- **"Quero COMEÇAR agora."** → [[03.3-HANDOFF-implementador-W0]] (os dois blocos a colar, nesta ordem).
- **"Quero mandar construir o MVP web."** → [[03.1-PROMPT-MVP]] (copie o arquivo inteiro como prompt).
- **"Por que malha e não campo?"** → [[02.1-Representacao-malha-primaria]].
- **"Como o sprite 2D recebe a luz 3D?"** → [[02.2-Sprite-com-malha-filha]] + [[05.2-Doacao-de-sombreamento-para-2D]].
- **"E se não der certo?"** → [[02.3-Modulo-removivel-e-mapa-de-crates]].
- **"Quanto isso custa em frame?"** → [[06.1-Waves-riscos-e-alvos]].

## Regras da casa que valem aqui

Este cofre está **dentro** do repositório da PH2D e obedece ao [`CLAUDE.md`](../../CLAUDE.md):
o alvo é o extraordinário e **o teto é o do hardware** (§0.0 — nenhum `MAX_*` entra sem a tabela de
medição ao lado), contrato congelado não se toca sem ADR (§6), e feature nova é **drop-crate** (§0.1).
