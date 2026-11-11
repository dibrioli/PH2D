# Prompt de continuação 11/11 n.º 2 — Painter depois das ondas 15–20

Cole numa janela NOVA. Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch
`line/PainterWatercolor` (`cd` + `pwd` + `git branch --show-current` ANTES de ler). Nada integrado; integração só por ordem do Enio.

## Leia primeiro (só isto)

1. [Handoff pós-missão](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-11_POS_MISSAO_4K.md) — §4 o que entrou, §6 smokes,
   §7 decisões do dono, §8 o ABERTO, §9 riscos.
2. [REGRAS_DO_AGENTE.md](../missao_4k/REGRAS_DO_AGENTE.md) — continuam a valer (sem RTX, red-first, ao bit ou prancha, alarmes vermelhos ficam).
3. O bloco do [BACKLOG](../missao_4k/BACKLOG.md) que for trabalhar (`grep -n '^| B<nn>'` + leitura por offset; o ficheiro tem 381 KB).

## O que foi feito (11/11, `e81e56bb1f..1d40807b12`, 82 commits)

Ondas 15–20 + fecho ([despacho](../missao_4k/DESPACHO.md)). BACKLOG: 117 → **151 feitos** · 26 parciais · 10 recusas · 135 abertos.
Medição «depois» no app feita ([ANTES_DEPOIS](../missao_4k/medicoes/ANTES_DEPOIS.md) §M1–M5): Ctrl+Z Impasto 4K 42,8 → 34,3 ms;
soltar Filbert 4K 47,8 → 34,0 (a luz cheia no gesto daria 19,1 — decisão do dono). Wet mistura pela lei espectral (vermelho + azul
= violeta), fator de aparência do documento (PROJECT_SCHEMA 232 → 233), tecto largo u16 (Flow baixo chega ao preto), sem aro
branco no Impasto, Oil Wash, Clone/Heal pré-multiplicados, controlos do Wet/Shape/Tuning arrumados, 64 camadas 4K na integrada.
Sono da água (B13.3) feito e DESLIGADO. Alarmes de pincel: largura 2 + miniatura 74 vermelhos, não regravados.

## ABERTO, por ordem

1. **Smoke do dono** (handoff §6, 9 passos) — nenhum visto. PT-BR simples, passos numerados, pranchas como links clicáveis
   relativos à raiz do repo principal (`Worktrees/line-PainterWatercolor/docs/Painter/missao_4k/pranchas/…`) e o relatório
   `Worktrees/line-PainterWatercolor/target/relatorio-pinceis/index.html` (74 miniaturas + 2 larguras, por grupo de origem).
2. **Decisões do dono** (handoff §7): luz cheia no gesto do Óleo Real (B14.2: soltar 34 × 19 ms); Impasto com dente (brilho/cera
   mais fracos, B29.3); sono da água desligado (B13.3); aprovar larguras/miniaturas.
3. **Parciais técnicos:** B28.3 (Color mixing grande, ~4 ms/q acima do alvo; o carimbo) · B17.5 (A×B no app, dois cantos) ·
   B13.3 (ε > 0) · B34.1b (sombra do óleo, orla 48 px, warp) · B31.2 (prancha da crista) · B35.13 · B39.6 · B42.3–.5 · B43.6.
4. **Wet 4K fora do alvo:** toque 62,5 ms e Ctrl+Z dos dois cantos 56,0 ms (alvo 33,3).
5. **Por começar:** B09 oráculos Krita/MyPaint · B18 · B27 · B30 · B33 · B35b–d · B37 · B44 · B45 · B46.

## Smokes pendentes

Os 9 do handoff pós-missão §6, mais os do [handoff da missão](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-10_MISSAO_4K.md) §6
e os herdados do [handoff de 10/11](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-10_PEN_UP_E_MISTURA.md) §6.
Binário: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop`.

Esforço: o dono pediu custo baixo (11/11) — agentes em esforço médio, uma janela nova por onda.
