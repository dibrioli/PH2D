# PROMPT W17 — o smoke da W16 reprovado sobre o main de 23/10 (dois defeitos)

> Rota «linha reaberta» (`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
> A linha está no main (`972f411a6f`, reaberta com `reset --keep main`; W8–W16 integradas em `eb081da774`).
> Leia antes: [handoff W16](HANDOFF_INTEGRACAO_line_Macros_2026-10-22_W16.md), [doc W16](../15_W16_pinceladas_sobrepostas.md).

## O relato do dono (verbatim, 23/10)

«vejo apenas a img branca ficar preta e nada mais» — corrida `PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=sobrepor`
(registo: `/tmp/ph2d-macro-smoke.log` da máquina do dono; as linhas-chave estão abaixo).

## Defeito 1 — o gesto do C&F da cena já não abre o seletor: enche a tela

Registo: `3000 ms: CeFDesce` · `CeFMove([-0.3, 0.4])` … `CeFSobe` · `EscolheCor([200, 30, 30]): the colour picker is not open`;
`editor=` salta para `1048576` (a tela 1024² inteira) logo a seguir; ao Parar `steps=["1.63", "2.24", "3.75"]` (TRÊS
etapas: traço, ColorDrop, traço) — em 22/10, antes do main de 23/10, eram duas (`["1.62", "3.78"]`, fotos em
`smoke_2026-10-22_W16/`). O explorador (23/10) diz: a porta «soltar fora = só o seletor» (`canvas_down_accepts`,
`crates/ph2d-app-painter/src/canvas_down.rs`; `painter_canvas_input.rs:472` `Down && !(in_x && in_y) ⇒ false`; o
`fill_drag_up` em `shells/desktop/src/input_dispatch/fill_drag.rs`) não mudou de lei; com `repeat_image` ligado
aceitaria `u ∈ (-1, 2)` — hipótese: os pincéis de fábrica do main de 23/10 (`line/PainterWatercolor` «Pincéis») ligam o
Repeat Image no Digital da cena, ou o mapeamento mudou. **NÃO confirmado: meça antes** (corra a cena; leia o
`repeat_image` do pincel no instante do `CeFDesce`). A cura do produto não é desta linha; a da CENA é: a cor da 2.ª
pincelada não pode depender de um gesto que o Painter reinterpreta (cena que ensina o contrário é pior que ausente).

## Defeito 2 (W16) — levar esse ColorDrop para trás apaga a 3.ª etapa e o clipe deixa de tocar

`6555 ms: ArrastaLosango(1, -1.85)` levou a etapa 1 (o ColorDrop) de `2.24` para `0.39`; depois `steps=["1.63",
"0.39"]` — a 3.ª sumiu — e o ▷ corre `playhead` até 14 s com `toca=false` (o clipe não toca a macro). Os gates
`levar_um_colordrop_para_tras_mantem_as_tres_etapas` e `levar_o_colordrop_para_antes_do_primeiro_traco_mantem_as_tres_etapas`
(`sobrepor_tests.rs`, com o ColorDrop do roteiro, que fecha no Done do modal) PASSAM: o ColorDrop da cena é outro
(provavelmente sem o Done — o modal «Fill adjust» fica aberto e a etapa fecha noutro passo, ou o 2.º traço cai dentro
dela). Reproduza com a gravação da CENA (o condutor de `caminho.rs` sem janela, ou grave o roteiro `sobrepor` pelo
tool), fixture que CONTÉM o fenómeno, vermelho primeiro. Suspeitos: `LeitorDoPainter::etapas` por faixa (`Aberta`,
`etapas.rs`) quando os passos do C&F ficam metade na faixa da etapa e metade na faixa 0; `tocadas()`
(`etapas_tocadas.rs`) que faz `break` quando falta uma tocada (apaga as seguintes em silêncio — deve ser erro ou
recusa); e se a edição devia ter sido RECUSADA em `etapas_desloca.rs` (um ColorDrop aberto não se desloca).

## Feito nesta janela (23/10, não comitado antes deste prompt)

- O diagnóstico por quadro das cenas `=5`/`=6` deixou a consola (o dono: «Eles não param e são muitos»): `regista`
  escreve só no ficheiro; a consola mostra só o anúncio (`caminho.rs`).
- Dois gates novos do ColorDrop deslocado (acima) e a fixtura `grava_com_entrada_o_traco_o_fill_e_o_traco` (`roteiro.rs`).
