# PROMPT W12 — os cruzamentos em Wet Paint sem «nó» (a marca do artista como cobertura no meio molhado)

> Rota «linha reaberta» de `docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md`:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`
> antes de ler; **sem reset nem rebase** (W8–W11 integram juntas, o integrador rebasa). A W11 está fechada e o smoke do
> dono passou em 20/10 ([handoff W11](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W11.md)).

## O que falta

A W10 ([plano](../08_plano_W10_cobertura.md), [handoff](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W10.md) §8 item 1) fez a
marca do artista passar de bit a cobertura — `c` (`u16`) + o píxel por baixo `B`, e o Play mostra
`atual + (1 − c)·(palco − B)` — **só para o Digital**. Wet Paint, Watercolor, Impasto e os outros modos de mistura
ficaram com `c` cheia (plano W10 §5): onde uma pincelada nova cruza uma gravada ainda por nascer, o Play mostra um
«nó» (o fim da gravada aparece ali antes do tempo). O Wet Paint é o meio do dono e o único que evolui
(memória `project_wet_paint_is_the_only_painter_mode_to_evolve_2026_10_07`).

## O que a W12 faz

1. **Medir antes de desenhar.** A cena `PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=cruza` corre em Digital desde a W10;
   acrescente-lhe a variante em Wet Paint (ou parametrize o meio) e meça o nó (o gate da app
   `a_pincelada_nova_a_cruzar_as_gravadas_nao_deixa_resquicio`, desvio máximo em níveis — W10 §2-bis) para Wet Paint,
   e também para Watercolor e Impasto (§0.10: a família inteira numa rodada).
2. **Gate vermelho** com essa medição, antes da cura.
3. **A cura no Wet Paint**: o compósito molhado é `blend_over(base, wet, pa)` e o `pa` continua a mudar DEPOIS do passo
   (a água anda): a cobertura tem de ser a que a tinta do artista deixou quando assenta, não a do dab. Leia a lei da
   água (`tool/paint/wetpaint*.rs`, `fita.rs`) e decida onde a cobertura se mede (no assentar? na marca da água?). O
   mesmo para Watercolor (velatura: mais clara que a força) e Impasto (relevo e luz) — ou registe, com a medição, porque
   um deles fica cheio.
4. Mutação das leis novas, smoke conduzido e fotografado (`docs/Components/ferramentas/fotografa_cena.sh`), handoff.

## Na fila (outra onda, outra família)

- O tutorial «fazer» salta a etapa Fill (W11 §8 item 1).
- W8 §8 itens 1–2 (o salto da água no fim do clipe; a espera pela água funde acções globais).
