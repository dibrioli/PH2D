# Prompt de continuação — linha PainterWatercolor · 14/11 n.º 11 (depois da rodada n.º 10)

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-14_desempenho_11.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor` (confira `pwd` e
`git branch --show-current` antes de ler), sobre o `main` `36a312482a` (não rebaseie se `git log HEAD..main` for vazio). Ordem do dono 14/11:
atacar todo o plano de desempenho, **integrar e fazer ship no fim; smoke só depois**. Não integre nem envie por conta própria: só quem o dono nomear.

## Leia primeiro
1. O handoff: [HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-14_N10_DESEMPENHO.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-14_N10_DESEMPENHO.md)
   (§0 colisão, §3 ABERTO, §4 recusas, §5 regras). Soma-se ao [N8_N9](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-13_N8_N9_DESEMPENHO.md).
2. [`../missao_4k/medicoes/N10_MATRIZ_FINAL.md`](../missao_4k/medicoes/N10_MATRIZ_FINAL.md) (quais alvos ✗ restam) e os `N10_*.md` da família que for tocar;
   [REGRAS_N10.md](../missao_4k/REGRAS_N10.md).

## Fila (por ordem)
1. Corrida de fecho na janela principal (os `--ignored`: B44.9 na placa, B17.4/B17.5 em ms, B18.7, B20.8) e o app 4K do pen-up adiado («soltar +1 s» do Flat Bristle 240).
2. Gate de tempo do Wet vermelho sozinho (B26.5): bissecção.
3. B17.6 (2 casos), B18.3 (a 1.ª troca de cada pincel Wet), B18.11 (Ctrl+Z Wet 60–74 ms), B26.1–B26.3, B25.8 só com a paridade ao bit da placa.
4. Bloqueados/dono: B23.6 (weston), B44.11–13, B08.6, B13.3/B14.2/B30.5.

⛔ Não reconstrua o que o §4 do handoff recusa (reuso do acumulador, pool próprio da compressão, FilmLut paralela, `cos` por célula, memo da mistura…).
Subagente não corre `--ignored`: a janela corre. Meça sempre no mesmo processo, intercalado, o mínimo, 4K retangular (`PH2D_FPS_TELA=3840x2160`).
