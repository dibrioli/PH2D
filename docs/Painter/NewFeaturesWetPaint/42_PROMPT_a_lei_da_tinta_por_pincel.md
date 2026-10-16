# 42 — PROMPT: a lei da tinta POR PINCEL — tinta, caneta e marcador escurecem ao cruzar (doc 18 §5 bloco A) (16/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current`; `git status --short` limpo. Leia SÓ: este prompt · [doc 18](18_mapa_das_midias.md)
§2 (a régua e a barra), §3 (linhas tinta · caneta · marcador) e §5 · [05c](05c_estado_mistura_e_claridade.md) e
[05d](05d_estado_mistura_II_e_presets.md) nas secções da mistura (as caixas K–M e o Glaze da OBRA) · lista
[26](26_ABERTOS.md) (B13 é da mesma família) · §6–§6b do [handoff P41](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-16_P41_SHAPE_COLOR_WET.md).
Regras de 13–14/10 (teste POR NOME, suíte a cada 10 commits, clippy só no fecho, edição provada). Oráculo: o Rebelle
corre-se (CLAUDE.md §0.9); nada dele se lê nem se copia (texturas, Mixbox).

## De onde vimos

Prompt 41 fechado e aprovado no smoke (16/10): a Shape Color pinta no Wet Paint, o traço cortado por uma recta
(a janela do rasto) e a prévia sem a Shape Color curados. O plano 27 (o Shape casa dos pincéis) está completo; a
lista 26 não tem itens A abertos. Do mapa das mídias (doc 18, 11/10) ficam de fora três blocos, por ordem
proposta: **A** a lei da tinta por pincel · B o pó fino (aerógrafo) · C as formas (Flat, guache). Este prompt é o A.

O defeito que o A cura: a tinta e a caneta do Rebelle **escurecem no cruzamento sem nenhuma opção ligada**
(cruzamento: marcador 0,016 · Ink Bullet 0,12 · Dip Pen 0,22 · Ink Brush 0,24; aquarela 0,93, guache 0,99); as
nossas dão **1,00** — só escurecem com a «mistura de pigmentos» da OBRA ligada (com ela o Ink Brush entra na barra
inteira). **Decisão técnica (padrão-ouro, delegada; o dono pode vetar): a opção (2) do 18 §5** — a lei vai para o
PINCEL (o preset) e a obra fica como está; a (1) mudaria a obra inteira por causa de um pincel.

## Pedido

1. **Medir primeiro** (um lote só): a régua do cruzamento do 18 §2 sobre os cinco do oráculo (`fixtures/rebelle/`
   `ink` · `dippen` · `inkbrush` · `marker` · `markerbullet`) contra os nossos candidatos, com as caixas da obra
   desligadas e ligadas; onde o motor decide a mistura e o Glaze por célula (`colorops` — `velatura`,
   `km_glaze_channel_linear`; `Params::mix`/`glaze`, hoje da obra); o custo por traço do Glaze multiplicativo; e o
   traço de hoje AO BYTE dos 12 pincéis de fábrica (miniaturas + `fingerprint`) — o portão de identidade nasce aqui.
2. **A lei:** a mistura e o Glaze passam a ser também do PINCEL — um campo do preset com omissão «a da obra», de
   modo que todo preset existente pinta ao byte; a tinta/caneta/marcador escurecem no cruzamento pela lei do
   Rebelle (a claridade é o pigmento; Glaze = camadas transparentes que multiplicam). O formato dos presets é por
   NOME (campo novo com omissão; nada de degrau sem necessidade) — conte se `SCHEMA` sobe e porquê.
3. **Os pincéis de fábrica do bloco:** caneta (Bullet, Dip Pen), nanquim (Ink Brush), marcador (Bullet, Liner),
   medidos na barra do 18 §2 (as 7 métricas, a largura prometida ± 2 px) — a contagem da fábrica conta-se no
   código (`the_factory_presets_are_counted`).
4. **Painel:** onde a lei do pincel se mostra (a secção Paint do Wet Paint, i18n, tokens) e como convive com as
   caixas da obra (qual manda, dito na tela); censo dos controlos — nenhum morto.
5. Portões vermelhos primeiro; mutação por lei; prancha (Rebelle × nosso, o cruzamento de cada um).
6. Fecho: suíte afetada + clippy; handoff ≤ 8 KB; smoke ao dono em passos numerados. Pergunte no smoke o **B13**
   (a Shape Color no pincel salvo) — é a mesma família: o que o preset leva.
