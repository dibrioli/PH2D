# 16 — Onda 3: Containers e Arrange no molde do Rive (versão B)

> Plano: [`15_plano_redesenho_molde_rive.md`](15_plano_redesenho_molde_rive.md) §Onda 3. Regra: só aparência e organização,
> os MESMOS ids e eventos, só no caminho em cartões (`PH2D_CARDS=0` = saída de referência, intocada).
> Maquete: `~/Documentos/GameEngineSamples/critica_ph2d.html#timeline`, painéis **B3** («Containers: dentro da 2ª
> instância»), **B4** («Arrange: arrastar um container») e **A3** (a lista com «used 2×»), capturados em chrome headless a
> 1300 px, escala 1, o `.ph` isolado por script; medidas por `getBoundingClientRect` + `getComputedStyle`, cores do CSS
> (`--c13-*`) e por amostra de pixel (moda de bloco).

## Medidas da maquete

| Peça | Maquete | No app (cartões) |
|---|---|---|
| Fileira de chips | rótulo «Clips»/«Containers» 10,5 px fg3, ≥ 64 px; chip pílula 20,3 px, ícone 11 px (filme = clipe, caixa = container), texto 11 px fg2, `<small>` 10 px fg3 depois do nome; vão 4 | a fileira 2 de hoje, **dois grupos rotulados na MESMA fileira** (a folha não perde altura); o chip do container leva «used 2×» / «not placed» |
| Coluna de nomes | 150 px, fundo (20,24,40) | a largura de hoje (não encolhe); fundo `TimelineTinta::NomesPropriedade` (é a mesma cor, ao bit) |
| Cabeçalho da coluna | «+ Lane», 26 px, 10,5 px fg3, fio branco 8 % | o `+` de hoje (`stack_add_header`) |
| Faixa (lane) | **30 px**, fio branco 3 %; ícone camadas 12 px · nome 11 px fg2 · peso numa pílula (10 px mono, fundo branco 6 %, raio 4, 26×15) · olho 18×12 fg3; muda = olho fechado `--bad` a 50 % e as peças a 40 % | `TIMELINE_FAIXA_H_PX` 30 nos cartões; peso `PesoPilula`, olho `TextoValor`, mudo `OlhoMudo`; o `+` de cada faixa FICA (sem ele perde-se o «colocar no cursor») |
| Peça (strip) | topo 4 px, pé 4 px (21 px numa faixa de 30), raio 5; fundo = gradiente da cor a 58 % → 34 % sobre a fita, contorno interno 1 px a 80 %; ícone 11 px + nome 10,5 px negrito; 7 px de margem | `TimelineTinta::PECAS[i % 5]` pela ordem do clipe, `PecaContainer` para container; `TIMELINE_PECA_{TOPO,PE,BORDA,MUDA}_ALPHA`, `TIMELINE_PECA_RAIO_PX` |
| Mistura (sobreposição) | listras 135° branco 34 % 3 px / fundo 25 % 4 px, contorno branco 55 %, raio 4 | `TIMELINE_MISTURA_ALPHA` / `_BORDA_ALPHA` sobre `TextoCabecalho` |
| Régua | 26 px, (16,20,37), fio branco 8 % | a régua da Keys (`FitaRegua`, `ReguaTraco`) |
| Fita | (12,16,33) | **a da Keys** (`FitaPropriedade` + `FitaFio`): as duas abas lado a lado leem-se iguais |
| Lista de containers (A3) | linha 30 px; ícone 12 · nome 11 fg2 · «used 2×»/«not placed» 10 px fg3 à direita; barra 3 px laranja; escolhida = azul 18 % | a linha mede o mesmo que a faixa; o nome com o uso à direita antes do lápis e da lixeira; a barra do tempo pintada como PEÇA de container (o mesmo desenho das strips); escolhida `ChipEscolhido` |
| Cores das peças (CSS) | idle #49E0AE · walk #FFCF4D · jump #B79CFF · swing #6CCBFA · shake #FF7AA8 · container #FF9F5A | `PecaA..PecaE`, `PecaContainer` (`ph2d-tokens/src/timeline_tinta.rs`); gate `as_cores_das_pecas_leem_se_diferentes_em_todo_tema` (≥ 0,10 OKLab nos 12 temas) |

Moldura, régua, cursor, listras depois do fim e texto centrado pelas maiúsculas: as peças da onda 2 (`linhas::moldura`,
`linhas::y_do_texto` / `paint::folga_do_texto`, `folha_tinta::tinta`), agora também nas abas Containers e Arrange.

## O dado novo (só leitura)

`ContainerView::uses` (`ph2d-timeline/src/snapshot.rs`) = quantas peças põem este container, na cena e dentro de todo
container (faixa muda conta: a peça continua posta); `0` = «not placed». Derivado no `rebuild`, nunca gravado
(`DOC_VERSION` e `PROJECT_SCHEMA` iguais).

## Fica de fora (função nova ou dado novo, ver o relatório da onda)

- Arrastar o chip para a faixa (o gesto da B4): é gesto novo; colocar continua pelo `+` da faixa.
- Marcas das voltas de um loop dentro da peça (`.c13lm`): o `StripView` não traz o modo de loop nem o tamanho do clipe.
- «plays 4.00 – 6.00 in Scene» na barra: já existe na migalha de hoje; não muda de sítio nesta onda.

## Cena do smoke

`PH2D_NEST_SMOKE=4` (abre no Arrange) e `=5` (abre na lista de Containers): os clipes idle, walk, jump, swing, shake; «Run &
jump» posto 2× na cena, «Idle pack» não posto, «Step» 1× dentro do Idle pack; faixas Base (1.0), Upper body (0.6), FX (muda).
[`smoke_nest.md`](smoke_nest.md).
