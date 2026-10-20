# PROMPT W14 — a água depois do Parar: o salto no fim do clipe e o desfazer que funde (2026-10-20)

> Rota «linha reaberta» de `docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md`:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`
> antes de ler; **sem reset nem rebase** (W8–W13 integram juntas, o integrador rebasa). A W13 está fechada e o smoke
> do dono passou em 20/10 ([handoff W13](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W13.md)).

## O que falta (W8 §8 itens 1–2, [handoff W8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md))

1. **O salto da água no fim do clipe** — consequência decidida no 2c da W8: a água que corre depois do Parar aparece
   de uma vez quando o clipe acaba.
2. **A espera pela água (≤ 120 s) funde as acções globais desse intervalo num passo** do desfazer.

## O que a W14 faz

1. **Ler antes:** o doc da W8 (`docs/Macros/` — o plano e o mecanismo da W8, o 2c) e o §8 do handoff W8; o que a
   W12 decidiu sobre a água da gravação parar no 1.º toque do artista (memória
   `project_wet_paint_after_stop_paints_over_the_recording_2026_10_20`).
2. **Medir antes de desenhar:** a cena `PH2D_MACRO_SMOKE=6` (o caminho do dono) em Wet Paint — o tamanho do salto
   no fim do clipe (texels e níveis que mudam no último quadro contra a água a correr), e que acções globais caem no
   mesmo passo do desfazer durante a espera.
3. **Gate vermelho** com cada um, antes da cura.
4. **A cura**, produto final; se a medição mostrar uma escolha de PRODUTO (o que o artista deve ver no fim do clipe),
   pergunte ao dono com os números ao lado.
5. Mutação das leis novas, cena de smoke conduzida e fotografada, handoff.

## Na fila

- ⛔ Para o integrador: os códigos do `Passo` 53–56 colidem com a `line/PainterWatercolor` (handoff W12 §4).
- W12 §8 itens 1–3; W13 §8.
