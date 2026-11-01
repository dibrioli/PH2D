# E10b — a lei «Pigment mix» (núcleo, sem painel) — 31/10

Decisão: F3 §7 R8 (coordenador) sobre a F2b §4.2. Um escalar por pincel `p ∈ [0,1]`: `0` = Oklab pelo volume (a
lei de hoje, igual ao bit), `1` = média geométrica pesada pelo volume da reflectância a 10 bandas (WGM do libmypaint,
ISC). `p` é extensivo como o meio (`Σv·p`).

## 1. Fórmulas

Por fatia (filme ou aberto), além da `Soma {v, Σv·o, Σv·t, Σv·m}`, o `Espectro` lateral:

| componente | conteúdo | floats |
|---|---|---|
| `ln_r` | `Σ v·log₂ R_λ`, `R_λ = SR_λ·r + SG_λ·g + SB_λ·b`, `(r,g,b)` = RGB LINEAR da tinta levado a `[EPS, 1]` (`EPS = 0,001`, o `WGM_EPSILON`) | 10 |
| `delta` | `Σ v·δ`, `δ = o − o_WGM(R)` — o resíduo Oklab da ida-volta de CADA tinta | 3 |
| `p` | `Σ v·p` | 1 |

Cor da fatia: `p̄ = Σv·p / v`; se `p̄ = 0` devolve `ō = Σv·o / v` (o ramo nem calcula bandas ⇒ igual ao bit);
senão `o_W = Oklab( clamp((T · 2^{Σv·ln R / v} − EPS)/(1 − EPS)) ) + Σv·δ / v` e
`cor = ō + (o_W − ō)·p̄` (por componente). Transparência: a da `Soma` (inalterada).

Tudo é soma pesada pelo volume ⇒ associativo e comutativo; `transfere_pigmento` move a MESMA fracção `f` da `Soma`
e do `Espectro` (o mesmo `move_um` de `transfere`). A cor misturada nunca volta às bandas — a única subida é
`Espectro::da_soma` quando o campo lateral NASCE sob tinta que já existia (com `p = 0`, logo sem efeito na cor até
um pincel `p > 0` lá pousar).

**O resíduo `δ` (decisão desta etapa, refinamento da R8).** O WGM cru não é idempotente: `mix(c,c)` dá a ida-volta
da base, medida aqui em **0,3910 ΔE×100** (a F2b mediu 0,39 — confirma que a base é o RGB LINEAR). O atalho
`base == tinta` do `espectral.rs` cura-o só na igualdade exacta (descontinuidade a 1/255). Somar o resíduo de cada
tinta pelo volume (o mesmo truque do resíduo aditivo do Mixbox, aqui sobre a nossa base ISC) torna a identidade
exacta até ao arredondamento **sem descontinuidade**, continua extensivo (associativo), e custa 3 floats por fatia.
Para tintas diferentes desloca o WGM pela média dos resíduos (≤ 0,0039 em Oklab).

`log₂`/`exp₂` vêm do `libm` pinado (`log2f`/`exp2f`), não dos polinómios do `espectral.rs`: a crate só usa
transcendentais do `libm` (determinismo CPU, porta GPU).

## 2. Memória (medida)

`CelulaTinta` = 72 B. `Espectro` = **56 B** (`size_of`, testado); lado espectral por célula `[filme, aberto]` =
**112 B** (testado). Grade do produto: razão de fluxo por omissão `1` (`grid_map.rs` `DEFAULT_RATIO`) ⇒ uma célula
por pixel da tela, `(w+2)(h+2)` células, POR CAMADA:

| tela | células | `tinta` 72 B | lateral +112 B | total com Pigment mix |
|---|---|---|---|---|
| 900×450 (`DEFAULT_*`) | 407 704 | 28,0 MiB | 43,5 MiB | 71,5 MiB |
| 1920×1080 | 2 079 604 | 142,8 MiB | 222,1 MiB | 364,9 MiB |
| 2048² | 4 202 500 | 288,6 MiB | 448,9 MiB | 737,4 MiB |
| 3840×2160 | 8 306 404 | 570,4 MiB | 887,2 MiB | 1 457,6 MiB |
| 4096² | 16 793 604 | 1 153,1 MiB | 1 793,8 MiB | 2 946,9 MiB |

(Sem o resíduo seriam 88 B/célula: 1 409 MiB a 4096². O resíduo custa 384 MiB a 4096² — o preço da identidade
exacta sem atalho.) **Desenho: vector LATERAL** (`Vec<[Espectro; 2]>`), vazio até um pincel `p > 0` pousar pela
primeira vez ⇒ com `p = 0` em toda a tela, zero bytes e zero custo (o modo caro inerte não paga). Compatível com
«toda transferência move a mesma fracção»: o campo existe para a grade INTEIRA ou para nenhuma célula
(`grid/pigmento.rs::transfere` recusa o caso misto com `unreachable!`), e quando existe toda transferência leva as
duas metades com a mesma `f`. Um esparso por tile reduziria a memória mas obriga a migrar o tile vizinho a cada
fluxo que cruza a fronteira — não medido, fica aberto.

## 3. Código

- `crates/ph2d-paint-tinta/src/pigmento.rs` (novo): tabelas `T/SR/SG/SB` VERBATIM do
  `ph2d-painter-brush/src/espectral.rs` (com o aviso ISC), `log_espectro`, `oklab_de_log`, `Espectro`
  (`de`, `da_soma`, `escala`, `junta`, `transfere`), `transfere_pigmento`, `p_medio`, `oklab_wgm_cru`, `cor`
  (re-exportada como `cor_pigmento`).
- `crates/ph2d-paint-tinta/src/pigmento_tests.rs` (novo).
- `crates/ph2d-paint-tinta/src/lib.rs`: `mod pigmento` + `pub use`.
- `crates/ph2d-paint-tinta/src/transfere.rs`: `move_um` passa a `pub(crate)` (o `Espectro` move com a MESMA
  aritmética; nenhuma mudança de comportamento).
- `crates/ph2d-wet-paint/src/grid/pigmento.rs` (novo, submódulo de `grid/tinta.rs`): `LadoEspectral`, `migra`,
  `garante`, `transfere` (com ou sem lado espectral), `srgb_0_255` (o irmão do de `tinta.rs`).
- `crates/ph2d-wet-paint/src/grid/tinta.rs`: a linha `#[path = "pigmento.rs"] pub mod pigmento;`.
- `ph2d-painter-brush` NÃO foi tocado: partilhar as tabelas pediria a dependência `painter-brush → paint-tinta`
  (Cargo.toml fora dos paths). Duplicadas com o aviso; ver aberto.

## 4. Testes (por nome, `--profile smoke`, 31/10)

| teste | o que prova | medido | limiar |
|---|---|---|---|
| `pigmento_p0_e_o_oklab_de_hoje_ao_bit` | (a) `p = 0` ⇒ `cor == Soma::cor` bit a bit (Lab e transparência), 222 cores, após juntar e transferir; lado vazio idem | igual ao bit | `to_bits` |
| `pigmento_identidade_mix_c_c` | (b) `mix(c,c)` a `p = 1`, 222 cores, volumes 1 + 0,6 | cru **0,3910** ΔE×100 · com resíduo **2,4e-6** ΔE (0,00024 ×100) | cru ≤ 0,59 · resíduo ≤ 1e-5 |
| `pigmento_associativo_em_qualquer_ordem` | (c) 3 tintas (p 1 · 0,3 · 0,7), 5 ordens | **1,9e-9** ΔE | ≤ 1e-5 |
| `pigmento_azul_e_amarelo_dao_verde` | (d) `40,90,220` + `245,205,25` 50/50 | `p=1` **101,148,113** (verde) · `p=0,5` 121,153,139 · `p=0` **140,158,166** (o cinza Oklab de hoje, ao bit) | G > R e G > B |
| `pigmento_sem_deriva_20x` | (e) 20× (juntar a mesma tinta, tirar metade), 4 cores | **0** ΔE em `f32` | ≤ 2e-6 e 0 em 8 bits |
| `pigmento_canais_zero_nunca_nan` | (f) 222 cores + preto, `p ∈ {0; 0,5; 1}`; sem NaN/∞ | preto com resíduo: **0** ΔE; o WGM cru do preto dá L 0,0036 (sRGB `0,0006·0·0` — o piso `EPS` invisível) | finito |
| `pigmento_transfere_conserva` | `transfere_pigmento` = o que chega; somas conservadas a 4 ulp; `size_of::<Espectro>() == 56` | ok | 4 ε |
| `tinta_pigmento_p0_igual_ao_bit_no_motor` (wet-paint) | o lado nascido por `garante`/`migra` + transferências, lido por `grid::tinta::pigmento::srgb_0_255`, = `tinta::srgb_0_255` ao bit; `size_of::<LadoEspectral>() == 112` | igual ao bit | `to_bits` |
| `cargo-test-narrow ph2d-wet-paint tinta` | o resto da lei nova intacto | **63 verdes**, 1 ignorado (62 de antes + o novo) | — |
| `cargo-test-narrow ph2d-wet-paint o_invariante_soma_massa` | o invariante `filme.v·M_UM == susp` | **1 verde** | — |

## 5. Aberto

1. **Campo em `Grid`** (`grid.rs`, fora dos paths desta etapa): `pub pigmento: Vec<LadoEspectral>`, e ele no
   `Clone`, no recorte/retoma (`patch.rs`/`snapshot.rs`, como `tinta::restaura_janela`) e no `bytes_retidos`.
2. **As portas de transferência** (`transfer.rs`, `assenta.rs`, `flow.rs`, pick-up, fecho, …): cada `transfere` da
   lei `Pigmento` passa a `grid::tinta::pigmento::transfere` com `Some(..)` quando o lado existe. O depósito: quando
   o lado existe, TODO depósito (também `p = 0`) junta `Espectro::de(tinta, v, p)` — senão `Σv·ln R` fica com
   menos volume que `v` e a média geométrica erra.
3. **O fecho** (`fecha`): o aberto que cura vira `R_g`; o seu `Espectro` zera com ele (a cor já foi lida pela
   `optica` com `cor_pigmento`). A `optica` da célula precisa de uma variante que receba a `Tinta` de
   `cor_pigmento` (hoje `optica(&Soma, …)` lê `Soma::cor`): `optica_de_rinf(Bandas(cor.linear()), …)` — trivial,
   não feito porque ainda nenhum leitor existe.
4. **O `p` do depósito** fica `0.0` até ao slider da E10 (o controlo só aparece no painel com leitor).
5. **Partilhar as tabelas** com `ph2d-painter-brush` (dependência nova `painter-brush → paint-tinta` e
   `espectral.rs` a importar `T/SR/SG/SB`); um teste de igualdade só é possível com essa dependência.
6. **E11 (GPU):** o kernel precisa de 14 floats por fatia (56 B) num buffer lateral opcional (binding só quando
   existe), `log2`/`exp2` com o MESMO bit do `libm` (`log2f`/`exp2f` não são as instruções nativas da GPU: portar a
   implementação do `libm` ou trocar ambos os lados por um polinómio comum, como o `espectral.rs` fez), a matriz
   `T` 3×10 em ordem fixa, e o ramo `p̄ = 0` que salta tudo (o mesmo bit que o Oklab).
7. **Esparso por tile** para telas ≥ 4096² (a lateral custa 1,8 GiB por camada): medir quando houver o campo.

## 6. Ligação (E10c) — 31/10

Itens 1–3 do §5 feitos; o 4 (o `p` do depósito) entra como `Sim::pigment_mix`/`Params::pigment_mix` (omissão `0`,
SEM painel). Abreviatura `wp/` = `crates/ph2d-wet-paint/src/`.

**Campo.** `Grid::pigmento: Vec<LadoEspectral>` (`wp/grid.rs`), VAZIO até `grid::tinta::pigmento::garante_para(g, p)`
com `p > 0` (`wp/grid/pigmento.rs:84`, migra a tinta de ANTES do depósito). Levado em: `Snapshot` (campo, clone,
restauro — `wp/grid/snapshot.rs`), tiles do histórico (`tile_fino!` + `comprimentos_vazios` 9→10 + `ajusta v[9]`),
recorte/retoma (`wp/grid/patch.rs` + `pigmento::restaura_janela`, `wp/grid/pigmento.rs:188`, o par do da tinta),
`bytes_retidos` dos dois (`wp/painter/instantaneo.rs`), limpar a tela (`wp/grid/canvas_ops.rs`) e sair de
`Pigmento` (`wp/painter/lei_da_tinta.rs`). A GPU só corre a lei `Velha` (`ph2d-wet-gpu/src/passo.rs:27`): nada a subir.

**Portas** (cada uma move o `Espectro` pela MESMA fracção da `Soma`; nada sem o campo):

| porta | onde |
|---|---|
| depósito molhado/seco (junta ou substitui `Espectro::de(tinta, v, p)` — com o lado vivo TODO depósito, também `p = 0`) | `wp/trail/transfer/deposito_tinta.rs:98-164` |
| tecto das 3000 (`tira`) · re-molhar do dissolve · Glaze (filme→aberto) | `deposito_tinta.rs:28`, `:125`, `:144` |
| arrasto (`Drag`: lerp das somas ⇒ lerp do `Espectro`) | `deposito_tinta.rs:196` (`pigmento::lerp`) |
| advecção sequencial e escoamento (`move_filme`, `junta_cantos`, assinatura passa a `&mut Grid`) | `wp/solver/tinta_transporte.rs:32` (chamadas `wp/solver/advect.rs:216`, `wp/solver.rs:605`) |
| Jacobi (`gather_linear`: mesma combinação linear) | `tinta_transporte.rs:149` (`pigmento::combina`) |
| backrun (`levanta`), `levanta_pelo_sett`, `tira_pelo_susp`, anel de dreno | `tinta_transporte.rs:50`, `:176`, `:194`, `drena_o_anel` |
| secagem: `assenta` / `levanta` (série e `dry_jacobi` paralelo, linhas do lado por `lados_por_linha`) | `wp/drying/tinta.rs:60`, `:85`, `:174`; `dry_cell` recebe `Option<Celula>` (`wp/drying.rs`) |
| lift activo da ferramenta · assentar tudo (`canvas_ops`) | `wp/tools.rs:251`, `wp/grid/canvas_ops.rs:59` |
| Blend/Smear (`Foto` leva o `Espectro` total; `fatia_e` = par de `fatia`) | `wp/tools/tinta.rs:74`, `:131` |
| pick-up do bico: a TELA perde `f` do filme e `f·o·assentado` do aberto | `wp/trail/bico_tinta.rs:89-90` |
| fecho (`fecha_celula` com a óptica do Pigment mix; zera o `Espectro` do aberto); `cura` em série quando há lado | `wp/drying/tinta.rs:221`, `:252` |
| ecrã e cobertura: `pigmento::optica_celula` (`cor_pigmento` → `optica_de_rinf`; sem lado = `optica_celula` ao bit) | `wp/render/tinta.rs:15`, `wp/grid/tinta.rs:127`; espelho `susp_rgb/sett_rgb` por `pigmento::espelho` |

**Testes** (por nome, `--profile smoke`, 31/10):

| teste | resultado |
|---|---|
| `pigment_mix_motor_azul_e_amarelo_dao_verde` (b: depósito com `p`, Jacobi 50/50) | `p = 1` ecrã **101,148,113** (verde; espelho 101,148,113) · `p = 0` **140,158,166** (o cinza Oklab de hoje) e `g.pigmento` vazio |
| `pigment_mix_motor_lado_p0_igual_ao_bit` (a: lado vivo com `p = 0` após Jacobi + `move_filme`) | ecrã igual ao bit |
| `pigment_mix_motor_conserva_o_espectro` (c: 50 passos `move_filme` + `junta_cantos` + `assenta`, 7 células) | pior relativa **1,2e-7** (≤ 1e-5) |
| `pigment_mix_motor_bytes_so_depois_de_p` (d) | `p = 0`: +0 B · 1.º `p > 0`: **+112 B × 180 células** (26 700 → 46 860) |
| `cargo-test-narrow ph2d-wet-paint tinta` | **67 verdes**, 1 ignorado (63 + os 4 novos) |
| `o_invariante_soma_massa` · `tinta_pigmento_p0_igual_ao_bit_no_motor` · `bico` (3) · `drying` (10) · `snapshot` · `patch` (5) | verdes |
| `jacobi` | ⚠️ **2 vermelhos**: `advect_jacobi_bits_are_the_recorded_ones`, `advect_jacobi_bits_by_each_commit_route` (lei `Velha`; a E10c só toca a advecção dentro de `if pigmento`) — NÃO conferido no HEAD sem o diff; ver aberto 1 |

**Aberto.**
1. Conferir os dois vermelhos `advect_jacobi_bits_*` no HEAD (`28c5db8bd0`) sem este diff: se já falhavam, não são da
   E10c; se não, bisectar o diff (suspeitos nulos por leitura: `solver.rs` só ganhou o campo no destructure).
2. **A ponta não leva o `Espectro`**: o pick-up tira o lado da tela pela fracção certa (a tela fica coerente), mas a
   cor que a ponta devolve é a da `Soma` (Oklab) e o depósito seguinte re-sobe às bandas por `Espectro::de` da cor
   RGB da ponta. Ligar `ponta_tinta` a um `Espectro` lateral exige `trail.rs` (699 linhas: mudar o TIPO de
   `ponta_tinta` para um par, sem linha nova) e `bico_tinta_tests.rs`.
3. O teste (b) mistura pela porta Jacobi com a grade montada à mão (o depósito pelos helpers da porta); falta o
   mesmo azul+amarelo pelo `Engine` inteiro (traço → Blend) quando a E10 der o controlo.
4. `cura` com lado corre em SÉRIE (a paralela `walk_rows2` só leva dois planos); medir o custo a 4096² antes da E11.
5. `trail/transfer.rs:476` tem um diff de `rustfmt` que não é desta etapa (não tocado).
