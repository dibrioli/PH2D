# Prompt de continuação 10h — Painter depois da missão 4K (11/11)

Cole numa janela NOVA. Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch
`line/PainterWatercolor` (`cd` + `pwd` + `git branch --show-current` ANTES de ler). Nada integrado; integração só por ordem do Enio.

## Leia primeiro (só isto)

1. [Handoff da missão](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-10_MISSAO_4K.md) — §4 o que entrou, §7 o ABERTO, §8 riscos.
2. [REGRAS_DO_AGENTE.md](../missao_4k/REGRAS_DO_AGENTE.md) — continuam a valer (sem RTX, red-first, ao bit ou prancha, alarmes vermelhos ficam).
3. O bloco do [BACKLOG](../missao_4k/BACKLOG.md) que for trabalhar (`grep -n '^### B<nn>'` + leitura por offset; o ficheiro tem 311 KB).

## O que foi feito (10–11/11)

288 commits sobre o main `bc4a6ce930`, 14 ondas paralelas ([despacho](../missao_4k/DESPACHO.md)). BACKLOG: 117 feitos · 16 parciais ·
9 recusas medidas · 174 abertos. 28 defeitos curados (BUGS #86–#113: perda de dados nas camadas/undo/Wet entre abas, traço preso
fora da janela, pânicos da placa e do worker Wet, macros meio-aplicadas). Desempenho 4K (Radeon integrada, A×B por item): quadro
38,8→10,2 ms; toque/traço/Ctrl+Z 32–38→15–17 ms; arrasto de ajuste 1 149→134 ms; undo Impasto 265→32 MB; Wet dab 897→26 MB
(motor); planos Gaussiano/Nitidez 434→3 ms; janela oculta 97–100 %→44–46 % CPU. ADR-0202 (rayon no compositor), ADR-0203
(quadro a pedido + `winit` vendorizado). PROJECT_SCHEMA 231→232. Clippy workspace e fmt verdes no fecho; mutação 15/17 sangram.

## ABERTO, por ordem

1. **Smoke do dono** (handoff §6, 9 passos) — nenhum visto. Mande-lhe em PT-BR simples, passos numerados, com as pranchas
   ([INDEX](../missao_4k/pranchas/INDEX.md)) como links clicáveis relativos à raiz do repo principal
   (`Worktrees/line-PainterWatercolor/docs/Painter/missao_4k/pranchas/…`) e o relatório `target/relatorio-pinceis/index.html`
   (46 presets mudaram desde `3f843b81b1`; só ele aprova).
2. **Medição «depois» no app (B08c)** — receita em [DEPOIS.md](../missao_4k/medicoes/DEPOIS.md); segurar a vez da placa para a
   matriz toda; escrever `medicoes/ANTES_DEPOIS.md` com `compara_antes_depois.py`.
3. **Decisões do dono que mudam a aparência, por começar:** B33 (papel sem repetição a 256 px), B35b–d (controlos do Wet),
   B36 (Oil Wash), B28 (meia definição/Color mixing grandes), B34, B32, B31, B30. Cada um com prancha (3 fundos, 1080p e 4K).
4. **Parciais técnicos:** B17.5 (dirty de 1 rect publica 270× o restaurado) · B13 (`frow_tiles`, `ladeira_do_oleo`) · B16b
   (Rewet esparso com caminho híbrido) · B12b (Ctrl+Z Wet 4K no app) · B14.1/.2 · B26.1 · B19.4/.6/.12 · B35.3/.5–.7 · B06.7 ·
   B23.6 (KWin virtual panica na WSI) · B27.1 (INPUT/p95 com load < 5).
5. **Por começar, técnicos:** B09 oráculos Krita/MyPaint · B18 camada `Arc` · B27 fila da caneta · B37 · B38 · B39 · B42 · B43 ·
   B44 · B45 pente fino (refactor > 700 linhas) · B46.
6. **Gates em falta** (mutações sobreviventes): `funde_as_caixas` com > `MAX_CAIXAS` caixas; rejeição por banda com folga 1 (B25.1).
7. **Árvore suja:** 25 ficheiros `M` em `ph2d-tool-painter` de outro agente (reformatação), não commitados — confira com
   `git diff` antes de qualquer trabalho; não os comite sem saber de quem são.

## Smokes pendentes

Todos os do handoff §6 (pintar 4K, ajustes, undo de camadas, Wet entre abas, soltar fora da janela, borracha com relevo, as
aparências que mudaram, relatório de pincéis, app parado), mais os herdados do [handoff de 10/11](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-10_PEN_UP_E_MISTURA.md) §6
(prévia/menu de presets, cartão Paint Mixing, «Save as default» do Wet Wash, Wet Wash Mix em papel com relevo, Color mixing em
coluna estreita). Binário: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop`.

Esforço: o dono pediu custo baixo (11/11) — agentes em esforço médio, uma janela nova por onda.
