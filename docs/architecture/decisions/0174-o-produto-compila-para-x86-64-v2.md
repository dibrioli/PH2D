# ADR-0174 — O produto compila para `x86-64-v2`

- **Status:** Accepted
- **Data:** 2026-09-23
- **Linha:** `line/PainterWatercolor`
- **Decisão:** do DONO (Enio, 2026-09-23: *«Sim e Sim. Vamos fazer»*, sobre a pergunta *«aceitas
  passar a exigir processadores de 2009 em diante?»*)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — quem integrar reconta-o contra o `main` do
> dia, nunca o copia daqui.

## Contexto

Nenhum ficheiro do repo fixava o nível de processador: nem o `.cargo/config.toml`, nem o
`rust-toolchain.toml`, nem os perfis do `Cargo.toml`, nem o CI, nem os scripts (conferido). O
`rustc --print cfg` do produto lia só `fxsr`, `sse`, `sse2` — **o `x86-64` BASE de 2003**. Nesse
alvo, `f32::round`, `floor`, `ceil` e `trunc` não são instruções: são **chamadas** a rotinas de
software do `compiler_builtins` (~20 instruções inteiras com ramos cada, chamadas pela GOT).

Isto apareceu duas vezes em medições de PRODUTO:

1. **A pilha do Composite Brush** (ADR-0172): o `roundf` era a maior folha do laço da tinta, e o
   `-C target-cpu=x86-64-v3` mediu **20–40 %** em todo o pincel sem mudar código.
2. **A aquarela** (2026-09-23): o `objdump` do laço da composição mostra **14 `floorf` e 4 `roundf`**
   no corpo da linha, mais o `floorf` dentro de `sample_bilinear` (chamado de 8 sítios),
   `value_noise_pair` (4) e `value_noise_tiled` (4) — cerca de **64–68 `floorf` por texel** numa
   transição. Com `x86-64-v2` o mesmo binário tem **zero** chamadas de libm nas funções da aquarela.

## Medição (aquarela, uma thread, canvas 1024², traço de 724 px, mínimos de 5–7 corridas alternadas)

| cena | base | `v2` | `v3` | parcela do ganho do `v3` que o `v2` dá |
|---|---|---|---|---|
| fábrica, tamanho 0,4 | 644,6 ms | 493,6 (−23,4 %) | 468,8 (−27,3 %) | 86 % |
| fábrica, tamanho 0,15 | 74,9 ms | 59,0 (−21,2 %) | 54,7 (−27,0 %) | 79 % |
| rewet 0,6 cruzado | 2 711 ms | 2 308 (−14,9 %) | 2 198 (−19,0 %) | 78 % |

⚠️ A máquina estava a `load 17`–`112`; o A/B alternado com mínimos mitiga e não cura. O **hash do
canvas foi idêntico em todas as variantes**.

⚠️ **E uma cura em CÓDIGO da mesma coisa foi medida e NÃO fica:** substituir os `floor` por
truncamento exacto (provado sobre os 2³² padrões de bits) dá −10 a −13 % no alvo base, mas **por
cima do `v2` fica MAIS LENTO** que o `v2` puro (`544` contra `494` ms) — o `floor` de software
reescrito à mão custa mais que um `roundss`. Com o nível subido, essa cura é regressão.

## Decisão

`.cargo/config.toml` do repo:

```toml
[target.'cfg(target_arch = "x86_64")']
rustflags = ["-C", "target-cpu=x86-64-v2"]
```

- **`v2` e não `v3`:** o `v2` (SSE3, SSSE3, SSE4.1, SSE4.2, POPCNT) cobre praticamente todo
  processador x86-64 desde ~2009; o `v3` (AVX2, FMA, BMI) exige Haswell/2013+ e compraria só os
  14–22 % restantes do ganho. A troca é do dono e o dono escolheu o `v2`.
- **Por `cfg(target_arch)` e não por triplo:** vale para Linux e Windows (o CI de Windows é x86-64);
  o macOS do CI é `aarch64`, onde `floor` já é a instrução `frintm`.

## Consequências

- ⭐ **O resultado numérico é o MESMO byte.** O `v2` não traz FMA (a contracção `a*b + c` que muda
  bits é do `v3`), e `roundss`/`floor` por instrução arredondam exactamente como as rotinas de
  software. A prova é a suíte inteira do repo verde no alvo novo, mais o hash do canvas da aquarela
  idêntico nas três variantes.
- ⚠️ **Um processador sem SSE4.2 deixa de correr o app** (termina com instrução ilegal ao arrancar).
  É a decisão, não um efeito colateral.
- ⚠️ **Os `rustflags` de configs diferentes SOMAM-SE** (o `-fuse-ld=mold` do `~/.cargo/config.toml`
  da workstation continua — conferido com `cargo check -v`, as duas bandeiras em cada `rustc`), **mas
  a variável de ambiente `RUSTFLAGS` SUBSTITUI-OS TODOS.** Quem definir `RUSTFLAGS` à mão perde o
  nível em silêncio. O CI e o `ship.sh` já não usam `RUSTFLAGS` (usam `CARGO_BUILD_WARNINGS`, por
  outra razão escrita no `spike.yml`).
- ⚠️ **Toda árvore recompila uma vez** depois de fundir isto: os `rustflags` entram na impressão
  digital do cargo e na chave do `sccache`.
- O `composite_linhas::redondo_u8` (ADR-0172) fica: continua exacto e continua a valer num build com
  `RUSTFLAGS` à mão; a vantagem dele no alvo novo encolheu.
