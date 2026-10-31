# Medida 31/10, rodada 5 — a secagem: memo de UMA linha, o atalho do levantar e os arrays à mão (ao bit), base `aeb67ca906`

Cena e régua as da [rodada 4](MEDIDA_31_10_RODADA4.md): o wash de `nucleos_wash` (4096×2560, `Lei::Cobre`), perfil
`smoke` com `PH2D_RELEASE="mede velocidade"` pela fatia da linha, `rayon::ThreadPool` de **16 fios**. Sonda temporária
`mede_rodada5_tmp.rs` (apagada, não comitada): quadros 6 · 42 · 78, em cada um `drying_pass_jacobi` sobre o instantâneo
restaurado, **6 rondas × 3 variantes intercaladas com a ordem rodada**, memo esquecido antes de cada corrida, resumo =
o MÍNIMO; hash FNV da grelha por variante. Depois o passo inteiro: 12 rondas de blocos de 6 passos, alternando.
As variantes são overrides EM EXECUÇÃO (estáticas `#[doc(hidden)]` temporárias, retiradas antes do commit).
loadavg 1 min 2–6 (outras janelas).

## Os cortes (pela ordem em que entraram; todos ao bit — hash da grelha igual ao do HEAD nos três quadros)

### 1. O memo numa linha de cache (`assenta/memo.rs`, `assenta_memo_f32`)

As coberturas vêm da tabela `f32` de `alpha_of_mass` ⇒ são `f32` exactas (a chave vai em `u32`, 16 B em vez de 32; se
alguma não for, contorna); e a secagem só guarda `f32` ⇒ o resultado guarda-se já em `f32` (12 B). Chave + resultado +
geração + lei = 60 B numa entrada `align(64)`: **uma** falha de memória por consulta em vez de duas (a rodada 4 tinha o
resultado `f64` num vector à parte). Os bits são os de `assenta(..).map(|v| v as f32)` — o portão novo
`assenta_bit_o_memo_f32_da_os_bits_da_porta` (coberturas exactas e não, fria e quente).

| quadro | memo r4 (HEAD) | memo de uma linha |
|---|---|---|
| 6 | 83,4 | **71,4** |
| 42 | 89,8 | **77,4** |
| 78 | 97,0 | **84,0** |
| passo inteiro | 63,7 | **60,1** |

### 2. O levantar: o custo NÃO era o Newton

Régua do chão, intercalada (ms por passe de secagem):

| quadro | produto | sem o levantar nas células do Newton | sem levantar nenhum |
|---|---|---|---|
| 6 | 70,8 | 72,9 | 38,5 |
| 42 | 76,8 | 76,6 | 42,1 |
| 78 | 83,4 | 79,3 | 44,6 |

As ~1 % de células que chegam ao Newton custam ~0; os ~33 ms eram o **atalho** das coberturas iguais (98,9 % das
células), que só devolve `uc` preso à caixa. O que pesava eram as cópias `[f32;3].map(f64::from)` → `[f64;3]` →
`.map(|v| v as f32)` e o `[0,1,2].map(..)` do `na_caixa`. Ablação (intercalada, mesma corrida):

| quadro | antes | só `#[inline(always)]` no atalho | + atalho em laço (sem `map`) | + arrays à mão no `lift_settled` | chão (sem levantar) |
|---|---|---|---|---|---|
| 6 | 70,9 | 70,4 | 55,9 | **45,2** | 38,0 |
| 42 | 77,0 | 75,9 | 61,6 | **50,8** | 41,4 |
| 78 | 82,7 | 82,4 | 68,0 | **58,0** | 44,9 |

O atalho fica agora à frente do `levanta` (`#[inline(always)]`, o resto em `levanta_resto` fora de linha), com a mesma
aritmética (`w`, `uc + (sc − uc)·w`, a caixa `[min, max]` de `uc` e `sc`).

### 3. A chave do memo e o `conta` sem `.map`

A mesma lição na consulta do memo: a chave (`cor`, `lei_k`, as coberturas) e as conversões da falha escritas à mão.
45,2 · 50,8 · 58,0 → **36,1 · 41,2 · 47,4** (corridas consecutivas do mesmo binário-base, não intercaladas; o
resultado final abaixo é intercalado).

### Resultado (intercalado, a árvore comitada)

| quadro | sem memo | **produto** | HEAD `aeb67ca906` (1.ª corrida desta rodada) |
|---|---|---|---|
| 6 | 188,0 | **36,8** | 83,4 |
| 42 | 198,3 | **42,7** | 89,8 |
| 78 | 215,4 | **47,5** | 97,0 |
| passo inteiro (ms/passo) | 84,7 | **54,9** | 63,7 |

Secagem **÷ 2,1** sobre o HEAD (÷ 5,1 sobre sem memo); passo **63,7 → 54,9 ms (−14 %)** a 16 fios.

**Em `release`** (a mesma sonda, `cargo build --release`, árvore comitada; o HEAD não foi medido em release): secagem
sem memo 103,9 · 110,1 · 120,7 contra **produto 33,8 · 36,6 · 40,2**; passo sem memo 68,3 contra **53,9 ms**. Hash
igual ao do `smoke` nos três quadros. ⚠️ O `smoke` paga ~1,8× no caminho sem memo (o Newton) — o perfil desta crate no
`smoke` (`codegen-units = 16`, incremental) é pior que o do pincel (`[profile.smoke.package.ph2d-painter-brush]`
tem `codegen-units = 1`); dar-lhe o mesmo é uma linha no `Cargo.toml` da raiz, fora desta linha — fica ao integrador.

## ⛔ Recusas MEDIDAS

| proposta | medida | veredito |
|---|---|---|
| fundir `edge_rows` no laço das células (uma passada) | sem `edge_rows` 34,8 contra 35,8 ms — ~1 ms | ⛔ não vale a cópia do `susp` que o Jacobi in-place exigiria |
| cortar o Newton do levantar (ao bit ou não) | as células do Newton custam ~0 (tabela do §2) | ⛔ o custo era o atalho |
| só `#[inline(always)]` no `levanta` | 70,9 → 70,4 | ⛔ o custo são as cópias de arrays, não a chamada |
| TOL / FIXA / UMA / memo do levantar | rodadas 3 e 4 | ⛔ continuam |

## Lição (para a próxima rodada)

⚠️ No perfil `smoke` desta crate (`codegen-units = 16`, incremental), `[T; 3].map(..)` e `[0,1,2].map(|c| ..)` num laço
por célula **não** saem de graça: aqui custavam ~10–15 ms por passe a 4K cada. Varra os laços quentes restantes
(advecção, fluxo, projecção, `sedimenta.rs:159/173`) por `.map(` em arrays antes de qualquer outra ideia. Não medido
nesta rodada: cada corte isolado em `release` (`codegen-units = 1`, LTO thin) — só o resultado final, acima.

## O que falta

- **Advecção (~23 ms/passo, transporte ~14)** — não tocada nesta rodada. O transporte já é escrito à mão; a conta de
  bytes (~45 B/célula × 10,5 M) põe-no perto da largura de banda: SIMD não ajuda sem mudar o arranjo dos planos.
- A secagem que sobra (~37–48 ms por passe, ~7 ms por passo médio): o chão ~38–45 ms (evaporação, `alpha_of_mass`,
  a escrita da célula) — o que resta do levantar e do memo são ~3–10 ms.
- Passo 54,9 ms contra **37,3 do Rebelle**: faltam ~18 ms — advecção, fluxo (8), momentum/projecção/suavizar.
