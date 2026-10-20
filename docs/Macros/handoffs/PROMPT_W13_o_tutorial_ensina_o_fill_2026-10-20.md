# PROMPT W13 — o tutorial «fazer» ensina o Fill (o ColorDrop gravado)

> Rota «linha reaberta» de `docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md`:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`
> antes de ler; **sem reset nem rebase** (W8–W12 integram juntas, o integrador rebasa). A W12 está fechada e o smoke
> do dono passou em 20/10 ([handoff W12](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W12.md)).

## O que falta

A W11 ([handoff](HANDOFF_INTEGRACAO_line_Macros_2026-10-19_W11.md) §8 item 1) fez o ColorDrop do C&F gravar-se e tocar
como a etapa «Fill N». Mas o tutorial «fazer» (`crates/ph2d-tool-painter/src/macro_track/tutorial.rs`, W6) só ensina
`Pincelada`: numa macro com um Fill, o aluno nunca é levado a escolher a cor no C&F e a soltá-la na tela — o
tutorial salta a etapa e segue, e a imagem do aluno fica sem o preenchimento.

## O que a W13 faz

1. **Ler antes:** o doc da W6 (`docs/Macros/03_plano_W6.md`) e o da W11 (`docs/Macros/09_W11_o_cef_a_gravar.md`) — como o
   tutorial guia uma pincelada (o fantasma, a régua `tutorial_mede.rs`, o «feito») e como a etapa Fill se reconhece
   (`etapas.rs`: do `ColorDropComeca` ao passo que fecha o Fill).
2. **Medir antes de desenhar:** uma macro gravada com `Stroke · Fill · Stroke` (o roteiro do smoke `=6 colordrop`) a
   correr pelo tutorial — o que ele mostra hoje em cada etapa e onde salta.
3. **Gate vermelho** com isso, antes da cura.
4. **A cura:** a etapa Fill no tutorial — o que o aluno vê (a cor a escolher no C&F, onde soltar), como se mede que
   ele fez (o preenchimento na região certa com a cor certa, com a tolerância medida) e o texto (chaves i18n novas,
   HR-15). Produto final, UI incluída.
5. Mutação das leis novas, cena de smoke conduzida e fotografada (`docs/Components/ferramentas/fotografa_cena.sh`),
   handoff de integração.

## Na fila (outra onda, outra família)

- W8 §8 itens 1–2: o salto da água no fim do clipe; a espera pela água funde acções globais.
- ⛔ Para o integrador, não para esta onda: os códigos do `Passo` 53–56 colidem com a `line/PainterWatercolor`
  (handoff W12 §4).
