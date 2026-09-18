# 21 — O censo das elisões, e o botão que pinta NADA

> **Medido em 2026-09-18, `line/UIUX`.** A 18.ª fatia do HR-15 — e a que fecha o item aberto *«a
> elisão medida EM PIXELS»*, com o instrumento em vez de mais uma leitura à mão.

## §1 — Porque nenhum gate estático podia responder

A fatia 20 achou a coluna do mixer **lendo o painel**. A pergunta que nenhum instrumento deste repo
fazia é ***o que foi pintado coube?*** — e ela não tem resposta estática: a largura de uma coluna sai
de um `rect` que só existe durante um quadro, com a arrumação, o zoom e a dobra daquele instante.
*Uma régua que lê o fonte mede a INTENÇÃO; esta mede o que saiu.*

⇒ a lei da reticência ([`text_elide`]) passou a **registar quem foi cortado**, e um gate arma o censo,
pinta o painel inteiro pelo arnês e lê a lista.

## §2 — ⛔⛔ A 1.ª redacção leu ZERO sobre o defeito VIVO

Liguei o censo ao `paint_text_elided`. Com a coluna do mixer revertida ao literal — o defeito que a
fatia anterior tinha acabado de medir —, o censo leu **`CORTADOS: 0`**.

⭐ **O corte acontece em quatro entradas** (`fit` · `fit_weighted` · o pintor cortado · o pintor
centrado), e o mixer usa a que eu não tinha ligado. *Um censo ligado a um dos caminhos lê zero e
parece aprovação* — a forma exacta que esta casa já pagou com o censo por prefixo de nome e com a
vassoura cega ao `.gz`.

⇒ o registo mudou-se para a **LEI** (`elide`), por onde todos passam. Com ele lá, o mesmo controlo
positivo lê **23**.

## §3 — ⭐⭐⭐ O que o instrumento achou no Audio Mixer, a 280 px

| o que se queria | o que saiu | largura |
|---|---|---:|
| `Master` | `Mas…` | `36,8` |
| **`M`** (silenciar) ×4 | **`` (nada)** | **`9,0`** |
| `Low` · `Mid` · `High` · `Size` · `Time` · `Fbk` · `Depth` · `Return` | `L…` · `…` · … | `18,8` |
| `Music` · `SFX` · `Voice` ×3 | `…` · `S…` · `V…` | `18,8` |

⛔ **O botão de silenciar pinta NADA**: a letra `M` precisa de `10,1 px` e o espaço tem `9,0` — nem a
reticência cabe, e a lei devolvia `None` **em silêncio**. Era isto que a foto do dono mostrava como
`…` entre dois botões com letra. ⚠️ *Um controlo sem legenda e um controlo morto dão o MESMO report.*

⇒ o corte para NADA passou a entrar no censo em vez de sair pelo `return` silencioso, e tem gate
próprio.

## §4 — ⚠️ Ele nasce DESARMADO, e isso é medido

Um corte é normal no produto (o nome de uma faixa da timeline, o caminho de um ficheiro): registar
sempre seria uma `String` **por corte por quadro** — a forma do vazamento que o `leak_key` do
`ph2d-i18n` já custou aqui. ⇒ o caminho do produto paga **uma leitura atómica** no ramo que já
cortava, e o gate usa `elisao::medindo(|| …)`, que arma, corre e desarma.

⭐ **O gate tem as duas metades e um controlo negativo**: armado vê · desarmado é mudo · o que CABE
não é um corte. Sem a última, um censo que registasse tudo devolveria a lista cheia e ninguém a leria.

## §5 — ⭐ Duas catracas mexeram-se, e uma DESCEU sozinha

- O `the_label_column_is_one_answer` tem censo de obsolescência, e ele acusou
  `ph2d-panel-audio-mixer/src/paint_widgets.rs` de **STALE** na primeira corrida depois da cura da
  fatia 20 — o ficheiro deixou de escolher a própria coluna. ⇒ a linha saiu da lista de tolerância.
  *É assim que uma catraca desce.*
- O tecto de LOC acusou `equalize-sizes/src/paint.rs` a `604` (o meu `tr_with` e a doc do tipo),
  curado por **CORTE** (`paint_chip.rs`, o pintor do chip com nome) — `604 → 522`. ⛔ Nunca por uma
  entrada nova no `FILE_OVERAGE_OK`.

## §6 — ⏳ O que fica ABERTO, com o número

Os **23** cortes do §3 são a medição, não a cura: o `M` é um defeito claro, e os `18,8 px` das
fileiras de strip são uma pergunta de geometria (cinco strips dentro de 280 px). ⇒ o gate que
**proíbe o corte para NADA** entra com a cura do botão; os cortes que ainda se leem (`L…`) ficam
medidos e nomeados.

⚠️ E o censo não diz **ONDE** cada corte aconteceu — só o texto e a largura. Dizê-lo obriga a
`#[track_caller]` ao longo de quatro pintores; fica nomeado.

## §7 — Os números

| | |
|---|---:|
| entradas de corte na lei | **4**, e o censo mora na lei que as serve |
| cortes medidos no mixer a 280 px | **23** |
| cortes para NADA | **4** (o botão de silenciar de cada strip) |
| gates novos | **1** (2 metades + controlo negativo) |
| catracas que desceram | **1** · tectos de LOC curados por corte | **1** |
| suítes | editor-core 1 676 · equalize 10 · mixer 31 · clippy `-D warnings` **0** |
