# Oráculo da PALAVRA (W7) — segmentação e raiz, corridos sobre entradas NOSSAS

> Estudo: [`../../05_facilitacao.md`](../../05_facilitacao.md) §1.3. Para o cronómetro, o voto e o agrupar
> NENHUM oráculo corre (Scrumlr precisa de servidor; Retrospected/Parabol são GPL/AGPL; Miro/FigJam
> proprietários; tldraw/Excalidraw não têm nada disto) — a fonte desses é o Miro medido.

## Triagem de licença (por ARTEFACTO instalado)

- `libstemmer` 3.1.1 — `pacman -Qi libstemmer`: **BSD-3-Clause**; `/usr/share/licenses/libstemmer/COPYING`.
- Node 24.19 `Intl.Segmenter` — ICU 78.3 (licença ICU, permissiva), Unicode 17.

## O que corre e o que grava

| script | oráculo | entrada | saída (fixture, com gate) |
|---|---|---|---|
| `segmenta.mjs` | `Intl.Segmenter('en', {granularity: 'word'})`, `isWordLike` | `entradas/*.json` (20 notas: inglês, português, apóstrofos, `ß`, ligadura) | `crates/ph2d-board-words/tests/fixtures/segmentos_icu.json` — gate `segments_match_icu` |
| `grava_raizes.py` | `libstemmer` 3.1.1 por `ctypes` (`sb_stemmer_stem`, UTF_8) | cada 10.ª palavra de `snowball-data/{english,portuguese}/voc.txt` + as palavras das entradas | `crates/ph2d-board-words/tests/fixtures/raizes_libstemmer_3_1_1.tsv` (7 515) — gate `stems_match_libstemmer` |

Corridas (09/10): duas de cada, iguais byte a byte (`cmp`).

```sh
node docs/MiroClone/ferramentas/facilitacao_oracle/segmenta.mjs <todas.json>   # as 3 entradas concatenadas
python3 -I docs/MiroClone/ferramentas/facilitacao_oracle/grava_raizes.py <english/voc.txt> <portuguese/voc.txt> <saida.tsv>
```

`voc.txt`: `https://raw.githubusercontent.com/snowballstem/snowball-data/master/<lang>/voc.txt`.

## Medido com ele (09/10)

Sobre o vocabulário INTEIRO (42 649 inglesas, 32 016 portuguesas), contra a `libstemmer` 3.1.1:
`rust-stemmers` 1.2.0 — 57 inglesas diferentes (`added → ad`); `waken_snowball` 0.1.0 — 17 (todas `inter*`);
o Rust gerado pelo compilador Snowball 3.1.1 — **0** e **0** (vendorizado em
`crates/ph2d-board-words/vendor/snowball/`).
