# W16 — o losango livre e as pinceladas sobrepostas (feito, 22/10)

> Pedido: [PROMPT_W16](handoffs/PROMPT_W16_pinceladas_sobrepostas_2026-10-22.md) · plano: [14](14_W16_plano.md).
> Handoff: [W16](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-22_W16.md). Fotos: [smoke_2026-10-22_W16/](handoffs/smoke_2026-10-22_W16/).
> Dono, 22/10: «deve permitir sobreposições» · «os keys em qualquer lugar da timeline» · «mover key não deve empurrar outros. siga».

## 1. O losango anda sozinho (núcleo `ph2d-macro`)

`EdicaoDeEtapa::desloca_us`: a etapa anda Δ µs, as outras ficam, **o relógio fica** (um relógio por quadro).
`efetiva` → `desloca_as_etapas` move os eventos NÃO-relógio das etapas deslocadas e funde por tempo
(estável; empate = ordem anterior). Faixas pela família (`LeitorDaFamilia::na_faixa(carga, n)`, `n = k+1`);
marca do pincel (`marca_da_faixa(n)`) no instante GRAVADO da etapa, preso ao novo fim.
**Barreira** (`e_barreira`, Desfaz/Refaz): a última etapa que acaba antes de um Desfazer gravado tem de ficar
a mesma, sem empate no instante dela, e nenhuma etapa movida a pode sobrepor de novo →
`MudaOQueODesfazerDesfaz`; `ComecaAntesDoZero`. A **cauda** (fim da última etapa → fim da gravação) segue a
última etapa (o clipe encolhe e estica; relógio a mais cai, a menos sintetiza-se por `espera`; `Para` vai ao
novo fim). Ficheiro: tag 9 (zigzag) da `Edicao`, MINOR 2 e `exige "etapa.desloca"` só quando usado. A
Espera do painel não muda (continua a empurrar; recusa da W15 mantida).

## 2. Duas pinceladas ao mesmo tempo (`ph2d-tool-painter`)

Antes só havia um traço aberto. Agora um **traço guardado por faixa** (`tool/paint/tracos.rs`):

- `Passo.faixa` (tag 15, nunca gravada em disco) e o derivado `Passo::PincelDaFaixa` = **200** (longe dos
  53–56 da PainterWatercolor). O passo leva o pincel do seu instante GRAVADO (cor/tamanho/meio):
  `pinceis_das_faixas` lê, numa ferramenta de rascunho, os passos de config da faixa 0 até cada marca.
- `TracoGuardado` troca os 27 campos DO_TRACO do `PaintState` + `live_gesture`/`pending_deposit` do wetpaint,
  `mancha_na_aguada` e as faixas da sessão húmida. Censo `cada_campo_do_paint_state_e_do_traco_ou_partilhado`
  (196 campos): campo novo de outra linha reprova até alguém o classificar.
- **Partilhado** (`PaintState::tracos_fora`): com outros traços abertos o pen-down não reinicia o que é de
  todos (passo do desfazer, sessão húmida/acumuladores da aguarela, relevo, escultura, quadro sujo) e o pen-up
  não o fecha (bake da aguarela, relevo, vidro, commit do desfazer, traço directo do Wet Paint). **O ÚLTIMO
  pen-up fecha: o grupo sobreposto é UM passo do desfazer.** `paint_tick` = global + `paint_tick_do_traco`
  (o tocador corre cada faixa guardada).
- `wetpaint/faixas.rs`: índices de faixa do motor húmido disjuntos (faixas mortas incluídas) — o 2.º traço
  reutilizava o rasto 0 e o vermelho virava roxo.
- Leitor/tocador: `Recomeco` (o instantâneo recomeça com respostas + pincéis de faixa; recusa se uma faixa que
  já pintou receberia outro pincel); `etapas_tocadas.rs::tocadas()` = a porta única «etapa gravada → os seus
  eventos na trilha que toca» (usada por `etapas()` e pelo tutorial `pinceladas_em`);
  `etapas_desloca.rs::desloca(trilha, k, us)` recusa `SoPinceladasSeSobrepoem` (só pinceladas simples, modo
  Paint/WetPaint, método não-forma) e `OutroMeio` (meios diferentes, ou estado armado do Wet Paint diferente
  no novo instante). Edição vazia nunca se guarda.

## 3. UI

`CampoDaEtapa::Instante(f64)` (editor-core, `desloca_para`). `panel-timeline::macro_steps`: a pré-visualização
move só o losango arrastado; ao largar `pedido_ao_largar` pede `Instante(fim+Δ)` (porta única, também usada
pela `=6`). `app-macros`: Instante → `pi::desloca`; «Edited from step n» conta o `desloca_us`.
**Motion RECUSA** `Instante` (`panel.macros.error.moves_with_the_next`): o Quadro dele regista se o quadro
sujou a simulação, e um gesto movido sozinho não o diria. Chaves i18n novas: `before_start`,
`changes_what_undo_takes_back`, `only_strokes_overlap`, `other_medium`, `moves_with_the_next`.

## 4. Medição (sonda 22/10, 128², vermelho em cima, azul em baixo, o azul começa a meio do vermelho)

| | Antes | Depois |
|---|---|---|
| Digital | o Down azul levava o traço vermelho: topo 4911 texels (série 3938); o azul saía vermelho (R 756712 · B 287704) | byte-idêntico à série |
| Wet Paint | vermelho virava roxo (R 1038970 · B 1050033) | cada região a sua cor |
| Aguarela / Impasto | idem | cada região a sua cor, contagens a 2 % |
| A meio da sobreposição | — | as duas regiões mudam no mesmo quadro (Digital topo 3074/3938, baixo 1580/3938) |
| Dois traços que se CRUZAM, ordem trocada | — | byte-idêntico a gravar de novo na ordem trocada; ▷ repete ao byte |

Cena `PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=sobrepor`: etapas `["1.62","3.78"]` → `["1.62","1.94"]`, laço
4,76 → 2,899 s.

## 5. Prova

Mutação 18: 15 sangraram à 1.ª; sobreviveram M3 (sem `tique_das_faixas`), M4 (`sozinho=true`), M8 (sem a guarda
`wetpaint_stroke_end`) → 3 gates novos (`sobrepor_fecho_tests`) → **18/18**. Testes e fecho: no handoff §2/§6.

## ⛔ Recusas

| Recusada | Porquê |
|---|---|
| Mover uma etapa empurrando as seguintes (como a Espera) | o dono: «mover key não deve empurrar outros» |
| Um relógio por faixa | um relógio por quadro; as faixas só dividem os eventos |
| Gravar a faixa em disco | é derivada da edição; a gravação fica igual à de antes |
| Sobrepor preenchimento, formas ou meios diferentes | não há lei de «ao mesmo tempo» para eles; recusa em voz alta |
| Mover um gesto do Motion sozinho | falta a lei do `sujou` no Quadro; pergunta ao dono (handoff §8) |
| Reordenar pinceladas à força quando a água as une | a sessão húmida contínua é UMA lavagem; a aguarela mancha o anterior com o pincel posterior (reportado, não mudado) |
