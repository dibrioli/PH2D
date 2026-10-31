# E7 — o escoamento transporta as somas (F4_2 §4 «escoamento» e «ladeira do óleo», 31/10)

Com `Sim::tinta == Pigmento` toda rotina do solver que move massa de `susp`/`sett` move a MESMA fracção da `Soma`
da célula (`Grid::tinta[i].filme` ↔ `susp`, `.aberto` ↔ `sett`). A cor de chegada é a soma das somas (extensivas):
nenhuma média de RGB, nenhum K-M (`km_mixing`/`ColorMix` não são lidos por este caminho). O código não precisa de
`M_UM` (E6a): só move FRACÇÕES (`massa_movida / massa_de_origem`), logo o invariante `filme.v·M_UM == susp` é
preservado para qualquer `M_UM`.

## O corpo novo

`crates/ph2d-wet-paint/src/solver/tinta_transporte.rs` (filho de `advect_jacobi.rs` por `#[path]`, para ler
`backtrace`/`take_scale`; re-exportado como `crate::solver::tinta_transporte`). Duas formas de porta:

1. **Gather LINEAR em Jacobi** (`gather_linear`): `filme'ᵢ = Σₖ coefₖ · filme(fonteₖ)` lido do estado do início,
   com os MESMOS coeficientes da massa. Cada componente de `Soma{v, lab, t}` é linear na massa ⇒ conserva igual e a
   razão `filme.v/susp` fica constante por célula. Usado pelo advect Jacobi (semi-Lagrange conservativo: célula
   fica com `1 − leave`, puxa `wₖ·take_scale(outflowₖ)` de cada canto se `want > 0`) e pelo `sedimenta`
   (fica `1 − ox − oy`, recebe `da` das duas vizinhas de cima).
2. **Mudança LOCAL in-place**: a fracção é lida da massa. `move_filme`/`junta_cantos` (advect sequencial, `transfere`
   de cada canto com `wₖ`, o canto que é a própria célula não move nada — a massa dele sai e volta);
   `levanta` (backrun sequencial: `sett·0,1` ⇒ `transfere(aberto → filme, 0,1)`); `fotografa_sett` +
   `levanta_pelo_sett` (backrun Jacobi: `f = 1 − sett_depois/sett_antes`); `fotografa_susp` + `tira_pelo_susp`
   (o levantar do tubo molhado: o filme perde `1 − susp_depois/susp_antes`; o destino — o pincel — é da E6b).

## As portas (um `if`/`match` por função nos ficheiros velhos)

| rotina | ficheiro:linha | porta |
|---|---|---|
| advect sequencial | `solver/advect.rs:31`, `:214` | `liga(p, g)` ⇒ `junta_cantos` depois do gather de `susp` |
| advect Jacobi | `solver/advect_jacobi.rs:142-162` | com `Pigmento` a rota FUNDIDA é desligada (lê-se o `uv`/`outflow` da não fundida, mesmo resultado ao bit) e `tinta::advect_jacobi` corre antes do `commit` |
| advect fundido / commit | `advect_fundida.rs`, `advect_commit.rs` | sem mudança (o fundido não corre com `Pigmento`; o commit não toca a tinta) |
| difusão | `solver.rs` `diffusion_pass` | `move_filme(from, to, dm/susp[from])` antes de mexer no `susp` |
| backrun sequencial | `solver.rs` `build_flow_field` (lift) | `levanta(nb, 0,1)` |
| backrun Jacobi | `solver/flow_jacobi.rs` `build_flow_field_jacobi_rows` | fotografia do `sett` ⇒ `levanta_pelo_sett` |
| sedimentar (Heavy Edge) | `solver/sedimenta.rs` `sedimenta_rows` | `gather_linear` com a combinação do passe. Sem `Params`: a porta é `existe(g)` (o campo só existe se a lei nova já correu) |
| tubo molhado | `trail/molhado.rs` (`molhado_levanta`) | fotografia do `susp` ⇒ `tira_pelo_susp` |
| `estreito::arredonda` | — | não move massa entre células (arredonda o plano, sonda de formato); sem porta: o desvio é o do arredondamento |
| ladeira do óleo | `solver/ladeira_do_oleo.rs`, `solver.rs:104`, `flow_jacobi.rs` | **não move massa**: lê `sett`/`susp` para uma ALTURA que entra no nivelamento do fluxo; a massa move-se depois pelo advect, que já leva as somas. Sem mudança de física |

⚠️ O plano dizia `sedimenta.rs:162,197` = suspenso → assentado; no código o `sedimenta` move SUSPENSO → SUSPENSO da
vizinha de baixo (o pigmento desce mais que a água). Assentar (susp → sett) é o `drying.rs`, da E8.

## A ladeira única (desenho, NÃO feito — depois do portão)

Hoje: `ladeira_do_oleo::do_passe` (8 rotas em `LADEIRA_ROTA`, `AtomicU8`) soma à altura da água a altura do óleo
assentado (`sett` e, numa rota, `susp`) com o freio de Bingham (`BINGHAM_PHI_C`, `PAPEL_BINGHAM`). Alvo (F4_2 §4):
UMA altura `h = w + vᶠ/φ + vᵃ/φ + altura_fechada` e o freio pela tensão de cedência `τ_y(m)` (Orchard); as rotas ficam
só para `Velha` até a E12. Depende de `M_UM` (E6a), de `m` por célula (E4/E10) e de `altura_fechada` (E8).

## Testes (por nome)

Ficheiro `crates/ph2d-wet-paint/src/solver/tinta_transporte_tests.rs`: dois traços com a lei velha, somas pousadas à
mão `filme = Soma::de(cor, susp·K)` (K = 0,37), `aberto` idem com `sett`, gravidade `[0; 0,04]`, N passos com
`Pigmento` pelos estágios 0 e 2–5 do `sim_step_stage` (flow Jacobi, difusão, advect Jacobi, sedimentar a=0,2,
fronteiras, projecção). O estágio 1 (secagem) fica FORA: ele assenta `susp → sett` sem as somas (E8) e o
`sim_step_stage` reescreve `dry_every` a cada passo, logo não se desliga por campo.

| teste | antes | depois |
|---|---|---|
| `a_advecao_conserva_somas` (60 passos) | — (novo) | verde: `Σ(filme+aberto).v / K` = massa a rel `< 1e-4`; `filme.v/(K·susp)` = 1 a rel `< 1e-4` em toda célula com `susp > 1 %` do máximo |
| `a_cor_nao_muda_quando_a_tinta_e_uma_so` | — | verde: `lab/v` e `t/v` iguais à cor a `1e-4` em toda célula com filme |
| `duas_cores_misturam_pela_soma` (azul/amarelo pela coluna, 80 passos) | — | verde: todo `lab/v` dentro da caixa dos dois; há células mistas |
| `o_interruptor_da_tinta_nao_muda_um_pixel_com_a_lei_velha`, `flow_bit_tests`, `sim_resumable_tests` | verde | verde |
| `advect_bit_tests::advect_jacobi_bits_are_the_recorded_ones` / `_by_each_commit_route` | **VERMELHO no HEAD `69a15a9b08`** | vermelho com os MESMOS bits (`0xbe1a63e73c04445b…`) |

O vermelho do `advect_bit_tests` é anterior à E7: medido numa worktree descartável no HEAD limpo (sem nenhum
ficheiro da E7 nem das outras etapas) — os mesmos valores `left`. Com a lei velha a E7 não muda nada (as portas são
`liga(p, g) == false`/`existe(g) == false`).

Medido também: SEM o dreno espelhado, a massa caía 1,7 % em 60 passos e as somas não (o anel de
`apply_boundaries`, `solver/project.rs:240-254`, zera `susp` e `film` na borda).

## Aberto

1. ⚠️ **O dreno da borda** (`solver/project.rs` `apply_boundaries`, fora dos ficheiros da E7): precisa de
   `super::tinta_transporte::drena_o_anel(g);` no fim (a função existe em `tinta_transporte.rs`; o teste chama-a à
   mão). Sem isso a soma fica na borda com a massa zerada.
2. A secagem (`drying.rs`, `susp → sett` e o levantar) é da E8: `transfere(filme → aberto, lost)`.
3. Desempenho só com `Pigmento`: o `gather_linear` aloca uma lista por passo do tamanho da faixa e corre em série;
   as fotografias do `sett`/`susp` (backrun Jacobi, tubo molhado) copiam o plano inteiro. Paralelizar/limitar à
   janela quando o portão-mãe passar (E9/E11).
4. A ladeira única (acima).

