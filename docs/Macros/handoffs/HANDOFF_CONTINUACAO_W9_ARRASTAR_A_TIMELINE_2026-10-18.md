# CONTINUAÇÃO — W9: arrastar a cabeça da Timeline deixa buracos retangulares nos traços (2026-10-18)

> Linha `line/Macros-Automação`, HEAD `3efd4bb9a7` (W8 + W9 por integrar; NÃO reset nem rebase). O defeito do
> passo 2 está CURADO e aprovado pelo dono (o Play de cima e o Autoplay da 2.ª tomada:
> [continuação anterior](HANDOFF_CONTINUACAO_W9_SMOKE_REPROVADO_2026-10-18.md), fim do ficheiro).

## O relato do dono (18/10)

«Foi corrigido. Contudo arrastar manualmente a timeline cria artefatos nos traços.» Cena `PH2D_MACRO_SMOKE=5`,
gravação própria (4 traços pretos sobre o vermelho), depois arrastar a cabeça de leitura na aba Keys. Fotos:
[1 — buracos](smoke_2026-10-18/arrastar_1_buracos.png) · [2 — certo](smoke_2026-10-18/arrastar_2_certo.png) ·
[3 — buracos](smoke_2026-10-18/arrastar_3_buracos.png). Os buracos são RETÂNGULOS com bordas retas que cortam o
traço (setas verdes): pedaços de traço que já deviam estar pintados faltam, como se parte da imagem ficasse de
outro instante.

## A causa provável (lida, NÃO medida)

- `PalcosDoPainter::sobe` (`crates/ph2d-app-painter/src/macro_track/mod.rs`) actualiza o espelho da ranhura só
  na caixa que o tocador diz ter mudado: `p.em_cena.caixa_da_tela()` → `TocadorDoPainter::caixa_da_tela`
  (`crates/ph2d-tool-painter/src/macro_track/tocar.rs`) → `PainterTool::take_preview_upload_bbox()` = o que
  mudou desde a última leitura DESTE tool.
- Arrastar para TRÁS faz `MacroEmCena::recomeca_ate` (`em_cena.rs`) trocar o tocador por uma CÓPIA de um
  instantâneo (`copia` → `TocadorDoPainter::instantaneo`, `mudou: true`) ou por um tocador novo. A caixa suja
  da cópia é a dela (a do instante em que se fotografou, mais o que avançou depois), não «tudo o que difere do
  que a ranhura tem». O espelho fica com o instante ANTERIOR fora dessa caixa ⇒ retângulos de outro tempo.
- Para a frente, de uma etapa a outra, a caixa acumulada está certa — por isso o ▷ e o ⏮ (que recomeça do
  estado inicial, tela branca INTEIRA) passam, e só o arrasto, que salta para trás a meio, mostra o defeito.
- Nenhum gate o via: o «ranhura = palco ao byte» da continuação anterior foi medido a tocar para a frente e
  com ⏮, nunca com saltos para trás a meio de uma pincelada.

## O próximo passo, num bloco

1. **Medir antes de curar:** uma variante `PH2D_MACRO_CAMINHO=arrasta` da `=6`
   (`crates/ph2d-app-painter/src/macro_track/caminho.rs`): depois do Parar, uma sequência de `Scrub(t)` para trás
   e para a frente (ex.: 3,0 → 1,4 → 2,8 → 0,9 → 3,4 → 2,0), com o diagnóstico a comparar o espelho com uma
   reprodução À PARTE da mesma macro no mesmo `t` (texels diferentes, não só a contagem de pintados). Esperado
   antes da cura: diferenças > 0 depois de cada salto para trás.
2. **Gate no palco** (`palcos_*_tests.rs`, com a dupla de `RanhurasDaCena` que guarda as subidas por região): a
   mesma sequência de saltos; em cada quadro a ranhura = a reprodução fresca em `t`, ao byte. Vermelho antes.
3. **Cura provável:** depois de um salto (`recomeca_ate`, `reedita`), a 1.ª subida é a tela INTEIRA — p.ex. o
   `MacroEmCena` marca «saltou» e o `caixa_da_tela` devolve `None` uma vez (e descarta a caixa da cópia). Medir o
   custo de 1 subida inteira (1024² RGBA) por salto contra a régua de custo da W9 (A3: 0,70 ms por quadro).
4. Fotografar a variante `arrasta` (`fotografa_cena.sh`, rajada) e dar ao dono o binário `smoke` com os passos.
   ⚠️ Na tela virtual a captura pode congelar alguns segundos (fotos com o MESMO hash enquanto o registo anda):
   confira o hash e o `Time(s)` de cada foto antes de as usar.
- Aberto (baixo, da continuação anterior): o piscar de 1–2 quadros depois do Parar (a marca da época antiga
  com `troca=true`).

## Prompt para a janela nova

```
Assuma a linha line/Macros-Automação (worktree Worktrees/line-Macros-Automação). Rota «linha
reaberta» de docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md: cd + pwd + git branch
--show-current antes de abrir ficheiros; git cherry main HEAD (W8 e W9 por integrar: NÃO faça reset
nem rebase). Leia docs/Macros/handoffs/HANDOFF_CONTINUACAO_W9_ARRASTAR_A_TIMELINE_2026-10-18.md inteiro
(o meu smoke: arrastar a Timeline deixa buracos retangulares nos traços; a causa provável; o próximo
passo). Meça com a variante «arrasta» da cena =6, escreva o gate que fica vermelho, cure, fotografe, e me
devolva o binário de smoke com os passos numerados e as fotos.
```

---

## Resultado (18/10, mesma data — o bloco feito)

**Causa MEDIDA, não só lida**: a de cima, com um acréscimo — o defeito só aparece onde a leitura da cena
sobe POR CAIXA. Wet Paint e Watercolor recompõem a tela inteira a cada tique da água (a cópia sobe
inteira por acaso); **Digital e Impasto** sobem a caixa suja herdada. A `=5` do dono não arma o pincel
(`arm_brush_once` só arma na `=4`/`=6`), logo ele pintou em **Digital** — o meio que reprova.

- **Gate** `macro_track::palcos_arrasto_tests::arrastar_a_timeline_deixa_a_ranhura_igual_a_tela_inteira_em_cada_meio`
  (`ph2d-app-painter`): 4 pinceladas em cada um dos 4 meios, tocar até ao fim, 8 saltos, arrasto contínuo
  para trás e para a frente; em cada quadro a ranhura = a tela INTEIRA de uma 2.ª reprodução com os mesmos
  instantes (isola a lógica das caixas da reprodução), e depois de cada salto para trás = uma reprodução
  FRESCA lida uma vez. ⚠️ A régua «fresca em cada quadro» foi recusada: a tocar para a frente a cena lê a
  tela um tique da água atrás (diferenças ≤ 16/255 no Wet Paint) — mede a régua, não o defeito.
  **Vermelho antes**: `Digital [(293, 2.084 s, 98 texels), (294, 1.936 s, 98)]`, `Impasto [(294, 1.936 s, 13)]`.
- **Gate** `macro_track::composicao_tests::a_copia_de_um_tocador_sobe_inteira_na_primeira_leitura`
  (`ph2d-tool-painter`): `instantaneo()` E `recomeca()` (a cópia da etapa editada, `reedita`/`partida`) — a
  1.ª caixa é a tela inteira, a seguinte volta à caixa; controlo: o original, no mesmo instante, sobe só a
  caixa.
- **Cura** (`tocar.rs`): `TocadorDoPainter.por_ler` nasce `true` em `com_tool` e em `instantaneo` (logo em
  `recomeca`); `caixa_da_tela` drena a caixa do tool e devolve `None` na 1.ª leitura. A lei está no tocador
  (a cópia nunca foi lida pela cena), não no `MacroEmCena` — cobre o salto, a volta ao início e a etapa
  editada de uma vez.
- **Mutação**: `por_ler: false` na cópia ⇒ os dois gates vermelhos (os mesmos números); restaurado.
- Vizinhos verdes: `ph2d-app-painter --lib macro_track` 25/25, `ph2d-tool-painter --lib macro_track` 49/49,
  `ph2d-tool-painter --test it macro_` 27/27.

**No app** (`PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=arrasta`, tela virtual, o MESMO binário com e sem a cura
por um interruptor de execução que já saiu do código): o diagnóstico ganhou `inteira=` (texels do espelho ≠ a
tela inteira do tocador) e `fresca=` (≠ uma reprodução à parte do `Macro.doc` no mesmo instante, uma vez por
instante parado).

| | linhas com diferença | máx. texels | saltos errados (parado) | quadros/100 ms no arrasto contínuo |
|---|---|---|---|---|
| sem a cura | 35 de 192 | 12 578 | 1,416 s (12 184) · 2,0 s (12 578) — ficam no ecrã até outro salto | 7,00 |
| com a cura | **0 de 196** | 0 | nenhum (9 instantes, `fresca=0`) | 6,93 (mín 6; ruído) |

Custo da cura: uma subida inteira (1024² RGBA, premultiplicar + textura) por salto; no arrasto contínuo cada
quadro é um salto e o ritmo do app não muda (tabela). Fotos: [antes, 1,25 s — os buracos](smoke_2026-10-18/arrasta_antes_1250ms_buracos.png)
· [depois, 2,79 s](smoke_2026-10-18/arrasta_depois_2792ms.png) · [depois, 2,0 s](smoke_2026-10-18/arrasta_depois_2000ms.png)
· [depois, 1,96 s a arrastar](smoke_2026-10-18/arrasta_depois_1958ms_arrasto.png). ⚠️ A rajada «antes»
congelou 9 fotos (mesmo hash) — usou-se a 10.ª, conferida pelo `Time(s)`.

- Shell: +1 linha (`macro_smoke.rs`: o ⏮ e o arrasto no mesmo braço; o diagnóstico recebe o mundo) —
  187 971 ≤ `TETO_LOC` 189 041.
- Aberto (baixo, herdado): o piscar de 1–2 quadros depois do Parar (marca da época antiga com `troca=true`).
- Próximo: o smoke do dono (o arrasto à mão na `=5`); a linha continua por integrar (W8 + W9).
