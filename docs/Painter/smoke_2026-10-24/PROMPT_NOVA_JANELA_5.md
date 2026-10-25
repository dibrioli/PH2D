# Prompt da janela seguinte — line/PainterWatercolor, reaberta depois da integração de 24/10

> Estado: a linha foi integrada no main (handoff
> `docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-24_FERRAMENTAS.md`) e reaberta no
> `c3662cd8e4` (main com o CodeEditor). Onda anterior:
> `docs/Painter/handoffs/HANDOFF_line_PainterWatercolor_2026-10-24_FERRAMENTAS_2.md` (§4 aberto, §5 depois do smoke:
> orlas do traço em 7 voltas — bug #58 —, export branco — bug #53 —, Flow por tabela, pin).

```
Linha line/PainterWatercolor (Modo L), reaberta 24/10 depois da integração.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, o handoff de integração e a §4–§5 do FERRAMENTAS_2 (acima).
Delegue leitura e verificação a agentes; a janela principal decide. Suíte completa só antes da integração.

Decisões do dono por pedir (não decida por ele):
1. Testes lentos da fábrica: acelerar o passo da água (drying_pass_jacobi_rows, advect) ou medir menos cenas.
2. A Borracha na Watercolor apaga igual ao Digital: sair do menu ou «apaga a aguada molhada mantendo a sessão».
3. Wet edges do Digital = a orla da aguada (8c50482d18): mais larga/irregular como a da Watercolor?
Aberto técnico (corrigir quando o dono mandar a próxima onda):
- A gravação da família «mundo» das macros lê a sprite sem a tinta por assar do Painter (mesma família do #53).
- A Strength está escondida em todo o Wet, mas a Borracha do Wet lê o Flow — medir se lê a Strength.
- Preview da Borracha no Wet pinta o S do pincel molhado; pegada da Borracha Wet ~1,9× a do Digital.
- Texture Eraser: o grão deixa de morder à Strength 1.
- A shell está a 23 linhas do tecto: cortar antes de crescer.
```
