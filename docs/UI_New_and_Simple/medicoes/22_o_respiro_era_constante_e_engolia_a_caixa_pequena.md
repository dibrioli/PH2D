# 22 — O respiro era CONSTANTE e engolia a caixa pequena

> **Medido em 2026-09-18, `line/UIUX`.** A 19.ª fatia — a primeira conduzida **inteiramente pelo
> censo de elisões**: eu não li um painel, li a lista do que ele cortou.

## §1 — A lista encolheu de 23 para 5, e o que sobrou tinha uma causa só

Depois da cura da coluna (medições 20 e 21), o censo do Audio Mixer a `280 px` devolveu:

| | |
|---|---:|
| `"M"` → `""` (o botão de silenciar de cada strip) | **4** |
| `"Master"` → `"Mas…"` (o cabeçalho do strip) | **1** |

⛔ **Quatro botões a pintar NADA.** O `M` mede `10,1 px`, a caixa dele mede `25,0` — e o respiro
levava `16,0`, deixando `9,0`. *Nem a reticência cabia*, e a lei devolvia `None` em silêncio.

## §2 — ⭐⭐⭐ A causa é a mesma doença da coluna, um nível abaixo

O `label_budget` era uma **subtracção constante** (`w − 16`). Numa caixa de `25 px` isso é **64 %
dela**. É o mesmo mecanismo do `FX_LABEL_W = 32,0` que a medição 20 curou: *um número que não
escala com o que o rodeia parte no primeiro caso pequeno*.

⇒ o respiro passa a ser `max(w − 16, w/2)`: **ele nunca come mais de metade da caixa.**

⭐⭐ **A fronteira é DERIVADA e não escolhida:** as duas leis cruzam-se onde `w − respiro = w/2`,
isto é em `2 × respiro` = **`32 px`**. Acima disso **nada muda** — toda caixa normal deste app
continua byte a byte como estava — e abaixo o respiro deixa de engolir a palavra. *É uma melhoria
estrita: o que cabia continua a caber.*

⚠️ E a inversa (`rect_for_label`) inverte as **duas** leis: a caixa mais pequena que serve é
`min(t + respiro, 2t)`.

## §3 — A prova

- **Censo**: `5 → 1` cortes no mesmo painel, à mesma largura.
- **Foto do produto**: os `M`/`S` de cada strip mostram a letra, com a janela a `1200 px`.
- **Mutação**: repor a subtracção constante devolve *«a caixa de 10 px ficou com 1 px de orçamento
  — o respiro comeu mais de metade»*.
- **Varredura impactada**: `19 698` testes, `19 697` verdes. ⚠️ O único ✗ é
  `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` (`ph2d-tool-painter`), **razão
  de dois relógios**, `zero` linhas do diff naquela crate, verde sozinho a `load 50,35` — a família
  de flakes de fan-out do §5.0, com **três irmãs do mesmo ficheiro já listadas**. Peço promoção.

## §4 — ⏳ O que fica medido e NÃO curado

`"Master"` corta a `280 px` (o cabeçalho pede `36,8` e o strip tem `~50`). ⚠️ É a ponta estreita do
dock, onde a lei desta casa já declara que *o nome corta e essa é a troca escolhida* — a mesma nota
que o gate do painel de camadas escreve. A `360 px` ele está inteiro (fotografado).

## §5 — Os números

| | |
|---|---:|
| botões que pintavam NADA | **4 → 0** |
| cortes no painel a 280 px | **5 → 1** |
| caixas do app afectadas | só as **abaixo de 32 px**, por construção |
| gates novos | **1** (3 metades: o regime normal inerte · o pequeno · o caso medido do `M`) |
| clippy `--workspace --all-targets -D warnings` | **0** |
