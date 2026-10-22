# PROMPT W15 — o passo da macro e as chaves na Timeline (smoke do dono 22/10)

> Rota «linha reaberta» de `docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md`:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`
> antes de ler; **sem reset nem rebase** (W8–W14 integram juntas). A W14 está fechada
> ([handoff W14](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W14.md)); o dono correu o smoke dela
> (`PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=fim`) e interrompeu-o para reportar os dois defeitos abaixo — o veredito
> da água no fim do clipe **não foi dado**: pergunte-o no fim desta onda.

## Os relatos do dono (verbatim, 22/10)

1. «quando clicamos no passo da macro no painel lateral, a playhead da timeline não se desloca exatamente para o
   local do keyframe.»
2. «Mover keyframes tem um bug: às vezes alguns são apagados (mesmo se não colocados um sobre o outro — que seria
   normal apagar).»

## O que a W15 faz

1. **Reproduzir antes de curar** (`/pd-bug-causa`): na cena `=6`, clicar cada etapa no painel MACROS e medir a
   distância entre a cabeça da Keys e a chave da etapa (em segundos e em quadros); para o 2, arrastar uma ou várias
   chaves (seleção múltipla, para a frente e para trás, por cima de vizinhas sem coincidir) e contar as chaves antes
   e depois. Ache o MECANISMO (onde o defeito aterra ≠ o que o provoca): arredondamento ao quadro, o instante da
   etapa vs o da chave, fusão por tolerância de chaves «iguais», índices que mudam a meio do arrasto…
2. Confirme de quem é o código (Macros vs `ph2d-timeline` / `ph2d-panel-timeline`); se for da Timeline, a cura
   pode ser aqui (Modo L, aditiva) — diga-o no handoff para o integrador.
3. Gate vermelho com cada um, cura, mutação, cena de smoke conduzida e fotografada, handoff.

## Na fila

- ⛔ Para o integrador: os `Passo` 53–56 colidem com a `line/PainterWatercolor` (W12 §4); a W14 toca ficheiros do
  Wet Paint (superfície de colisão, handoff W14 §4).
- W14 §8, W13 §8, W12 §8.
