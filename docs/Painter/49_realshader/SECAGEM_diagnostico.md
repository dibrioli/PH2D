# Secagem do Wet Paint — diagnóstico e cortes AO BIT simulados (31/10)

Sonda: `crates/ph2d-wet-paint/examples/sonda_secagem.rs` (cópia de `drying_pass_jacobi_rows` → `dry_cell` →
`assenta`/`levanta`, base `207e660793`, com `const C` de cortes). Cena = `nucleos_wash.rs` (4096×2560, 100 %,
`Lei::Cobre`, `esconde 5`, `filme 0`, `vela 0`). Binário `--profile smoke` corrido DIRECTO (32 fios rayon; o
`ph2d-run` corta a 50 % e mudaria o paralelo). `loadavg` 2,86 no início, 1,54 no fim.

## Trabalho por célula (contado, 1 secagem, 8 952 235 células)

| o quê | por célula |
|---|---|
| `assenta` Cobre · `levanta` Cobre | 1,000 · 0,991 |
| avaliações do ecrã no Newton (`sobre_branco_tapando`) | 8,03 (2,01 voltas) |
| `powf` do tapar | 2,97 |
| `diferenca_de_pigmento` (sqrt + div) | 9,02 |
| `a_mesma_tinta_filtrada` (6 `ln`) · `velatura` do filme | 0 · 0 (vela 0, filme 0) |
| células sem assentar (`lost = 0`) | 0,000 |
| `levanta` com coberturas IGUAIS (`a_s2 = a_s`, `a_f2 = a_f`) | **0,989** |
| `assenta` com coberturas iguais | 0,001 |

Leitura: o custo é o Newton do `assenta` (~6 avaliações), cada uma a refazer `diferenca_de_pigmento` inteira; a
derivada numérica avalia os 3 canais para ler 1. Toda célula tem água (nenhuma salta). Bytes por célula: lê/escreve
film·susp·sett (12) + 2×rgb (24) + lê paper·vela·aparência·óleo·edge — a memória não é o gargalo (1,7 µs/célula série).

## Cortes AO BIT simulados (A/B intercalado no mesmo processo, aquecido)

| variante | paralelo mín/med (ms) | série mín/med (ms) | hash N=6 |
|---|---|---|---|
| crate (`Orig`) | 348,2 / 349,1 | 7545 / 7546 | `1aae71329821396d` |
| cópia verbatim | 348,8 / 349,6 | 7540 / 7541 | igual |
| + LADO (metade fixa de `d` içada; `d(sc,uc)` partilhado alvo↔palpite) | 330,5 / 331,0 (−5 %) | 7309 | igual |
| + CANAL (derivada avalia só o canal lido) | 248,1 / 249,6 (−29 %) | 5396 | igual |
| + MEMO (`powf` reaproveitado se a cobertura repete) | **244,5 / 244,9 (−30 %)** | **5355 (−29 %)** | igual |

Hash igual em paralelo e série para os cinco (FNV-1a de film, susp, sett, susp_rgb, sett_rgb após 6 secagens).
Ganho no passo inteiro (secagem 1 em 6, passo médio ~130 ms): −104 ms por passo que seca ≈ **−17 ms/passo médio
(~−13 %)**.

## No produto (31/10, `assenta.rs` + `assenta/ponto.rs` + `colorops::Perfil`)

LADO · CANAL · MEMO (por chamada) · **SALTA** (`levanta` com coberturas iguais devolve `x0` sem alvo nem Newton),
um commit cada. Identidade: `assenta_bit_tests` (hash das portas — 4 leis, Cobre com filme/Glaze/vidrado/óleo,
coberturas 0…1 iguais e diferentes — e de 6 secagens encadeadas, gravado em `8cf2382d6b` antes dos cortes) verde
após cada corte; sonda: hash `1aae71329821396d` igual nas 6 variantes em paralelo (crate cortada, cópia, +Salta).

| passo inteiro (`nucleos_wash 12`, smoke, directo, A/B/A/B entre binários) | antes `47d8c108` | depois |
|---|---|---|
| passos/s | 7,83 · 7,81 | **9,96 · 9,94 (+27 %)** |
| ms/passo | 127,8 · 128,0 | **100,4 · 100,6** |

`loadavg` 1,53 no início; sobe com o próprio binário (32 fios, nada mais a correr). A régua ms/secagem da sonda
não chegou a correr (prazo); a do passo inteiro já mostra −27,5 ms/passo médio, acima dos −17 previstos sem o SALTA.

## Grupo (b) — só se o (a) não chegar

- `powf(esconde)` por tabela sobre o índice inteiro de `alpha_of_mass` (só onde `esconde` é o da sessão) — exacto
  para `a_f`/`a_f2`, mas `a_dm` não é da tabela; já hoje é ~3 % ⇒ pouco a ganhar.
- f32 no avaliador do Newton: a tolerância do `resolve` é 0,01 nível; a sonda C (f32) não mediu este caminho.
- Derivada ANALÍTICA em vez da numérica (muda o resultado — tolerância a medir contra o Rebelle).
