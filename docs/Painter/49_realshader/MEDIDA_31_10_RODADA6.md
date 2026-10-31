# Medida 31/10, rodada 6 — o perfil do passo hoje e a largura de banda da advecção, HEAD `f126dd0dc7`

Cena e régua as das rodadas 3–5: o wash de `nucleos_wash` (4096×2560 ⇒ 10,5 M células, `Lei::Cobre`), perfil `smoke`
(agora com `codegen-units = 1` no `ph2d-wet-paint`), `PH2D_RELEASE="mede velocidade"`, fatia da linha (CPU ≤ 1600 % de 32),
16 fios. loadavg 1 min 0,4–1,6 (calmo). Sondas temporárias (`banda_tmp.rs`, `mede_rodada6_tmp.rs`) apagadas, não
comitadas. **Nenhuma mudança de código nesta rodada**: o único corte tentado foi recusado pela medida (abaixo).

## 1. Onde estão os ~55 ms hoje (perf, 16 fios, 144 passos a 56,4 ms/passo, 83,6 k amostras)

Binário à parte (`CARGO_TARGET_DIR=target/perfsym`, `CARGO_PROFILE_SMOKE_STRIP=none`, `…_DEBUG=line-tables-only`);
`perf record -F 999 -g -D 12000 nucleos_wash 8`. ms = partilha × 56,4 (fases com cadência já vêm em média por passo).

| fase | % | ms/passo |
|---|---|---|
| **advecção inteira** | **55,8** | **31,5** |
| · `transport_rows` | 25,4 | 14,3 |
| · `momentum_rows` + `flow_at_point` (só ela o chama) | 12,8 | 7,2 |
| · `outflow_rows` | 9,6 | 5,4 |
| · `prepare_rows` | 8,1 | 4,6 |
| secagem (`dry_cell` 4,8 · `lift_settled` 3,0 · `assenta` 3,3 · `Parada::d` 1,9 · `prepara` 0,8 · `pow` 0,5) | ~14,3 | ~8,1 |
| suavizar (`smooth_velocity_rows`, 3 passos em 4 ⇒ ~5,5 ms por chamada) | 7,3 | 4,1 |
| fluxo (`flow_rows`, 1 passo em 4 ⇒ ~16 ms por chamada) | 7,2 | 4,1 |
| projecção (`project_rows` #4–#6) | 5,4 | 3,0 |
| região activa | 2,8 | 1,6 |
| resto (rayon, `wet_stamp`, …) | ~7 | ~4 |

⚠️ A rodada 3 contava a advecção em 23 ms **sem** o `momentum_rows` (que corre dentro de `advect_jacobi_rows`). Com
ele, a advecção é **31,5 ms — 56 % do passo**. A secagem, depois da rodada 5, já é só ~8 ms/passo.

## 2. A advecção está NA largura de banda (medido no mesmo binário, 16 fios)

Referência por fluxos `rayon` sobre planos do tamanho da grelha (10,5 M), mínimo de 30 repetições:

| fluxo de referência | ms | GB/s |
|---|---|---|
| só leitura, 32 B/célula (os planos que o transporte lê) | 5,26 | 63,9 |
| **o tráfego do transporte sem os cantos**: lê 32 B, escreve 20 B (film, susp, rgb) | **11,60** | 47,1 (sem contar o *write-allocate*) |
| `memcpy` de 3 planos, 20 B/célula | 9,80 | 42,9 |

⇒ `transport_rows` custa **14,3 ms contra um chão de 11,6 ms só de tráfego** (sem conta nenhuma): está a ~1,2× da
banda. SIMD, `.map` à mão, inline — nada disso o move. Contando os bytes de cada fase da advecção (leitura + escrita +
*write-allocate* das escritas): `prepare` ~27 B/célula (lê `flow_x/y` 8 + `active` 1, escreve `uv` 8 + `bt` 1) ·
`outflow` ~17 (lê `uv`/`bt`, escreve `outflow` 4) · `transport` ~73 · `momentum` ~29 (lê `film`, `active`, `flow_x/y`,
escreve `vel_x/y`) ≈ **146 B/célula ≈ 1,5 GB por passo ⇒ ~32 ms a 47 GB/s** — o que a advecção custa (31,5 ms). A
estimativa por fase é conta, não medida; o chão do transporte é medido.

**32 fios mais lentos que 16 (rodada 3):** com a advecção na banda, mais fios dentro do mesmo tecto de 16 núcleos só
somam *throttle* do cgroup; e sem tecto ganham pouco numa fase limitada pela memória. Não medido sem tecto nesta rodada.

## ⛔ Recusas MEDIDAS

| proposta | medida (intercalada, 12 rondas × blocos de 6 passos, mínimo) | veredito |
|---|---|---|
| `flow_at_point` em linha no `momentum_rows` (a amostra quente saía logo depois do `call`) | sem inline 52,8 · inline **52,7** ms/passo | ⛔ ruído: o custo do momentum é tráfego, não a chamada |
| SIMD / laço à mão no `transport_rows` | chão de banda medido 11,6 contra 14,3 ms | ⛔ não há onde ganhar sem cortar bytes |

## O que falta — e o único caminho que a medida deixa (cortar BYTES, ao bit)

Para 37,3 ms faltam ~17 ms; a advecção tem ~32 ms de tráfego. Dois cortes, por ordem de ganho:

1. **Fundir `momentum` + `prepare` + `outflow` + `transport` por bloco de linhas, com halo em cache (L2).** `uv`, `bt` e
   `outflow` deixam de ir à DRAM: hoje cada um é escrito por uma fase e relido pela seguinte. O halo é o alcance
   (`destination_reach` = 2 no default, +1 do canto) ⇒ ~3–4 linhas recomputadas por bloco (blocos de 64 linhas ≈ 10 %
   de conta a mais, numa fase onde a conta é grátis). Poupa ~45 B/célula ≈ **~10 ms**. Ao bit por construção: a ordem
   de soma do `outflow` é por linha-fonte e já é fixa (os `advect_bit_tests` e `flow_bit_tests` medem-no).
2. **Escritas não-temporais nos `dst_*` do transporte** (e no `vel_x/y` do momentum): tira o *write-allocate*
   (~20 + 8 B/célula) ≈ **~4–6 ms**. Medir contra o chão acima antes de comitar.
3. Planos mais estreitos (`uv` em `f16`, `susp_rgb` em `u16`) só com a tolerância invisível provada (max ≤ 1,
   p99,9 ≤ 0,5 níveis 8-bit na cor final) — e só depois de 1–2, porque com eles `uv` já não sai da cache.

Fases pequenas (suavizar, fluxo, projecção ≈ 11 ms) também percorrem a grelha inteira: depois da fusão, medir se
`smooth_velocity_rows` cabe dentro do mesmo bloco (lê `vel_x/y` que o momentum acabou de escrever).
