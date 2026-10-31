# E8 — o tempo: assentar, re-molhar, abertura e fecho (31/10)

Plano: `F4_2_plano_wet_paint.md` §2.2 «A regra do fecho», §4 (linhas assentar · abrir/curar · evaporar), §4.1
R2–R7/R16, §7 E8. Com `Sim::tinta == Pigmento` (e o campo `Grid::tinta` presente) o tempo move as SOMAS pela mesma
fracção que move a massa; invariante `filme.v·M_UM == susp`, `aberto.v·M_UM == sett` em cada caminho.

## O que mudou (`wp/` = `crates/ph2d-wet-paint/src/`)

| sítio | o quê |
|---|---|
| `wp/drying/tinta.rs` (NOVO) | `TAU_OLEO_S`, `O_FECHA`, `PASSOS_DA_CURA`, `tau(m)`, `assenta`, `levanta`, `dry_jacobi` (a rota Jacobi com a 6.ª faixa `tinta`), `cura`, `fecha_celula` |
| `wp/drying.rs:21-23` | `pub mod tinta` + `pub use tinta::{PASSOS_DA_CURA, cura}` |
| `wp/drying.rs:163` `dry_cell` | parâmetro `tinta: Option<&mut CelulaTinta>` (Velha passa `None`) |
| `wp/drying.rs:218` | porta do ASSENTAR: `Some` ⇒ `tinta::assenta(t, dm/s_mass)`; `None` ⇒ o composto de hoje (inversa `assenta_memo_f32`) intacto |
| `wp/drying.rs:275` | porta do RE-MOLHAR: `Some` ⇒ `tinta::levanta(t, b)`; `None` ⇒ `lift_settled` intacto |
| `wp/drying.rs:329,374` | Gauss-Seidel (`drying_pass`/`fast_dry`): `g.tinta.get_mut(i)` com Pigmento |
| `wp/drying.rs:518` | Jacobi: com Pigmento `return tinta::dry_jacobi::<P>(…)` (mesmo `dry_cell`; o `walk_rows5` não leva 6 faixas) |
| `wp/grid/canvas_ops.rs:~59` `dry_canvas` | com o campo presente: `tinta::assenta(t, 1.0)`; sem ele `settle_composite` intacto (porta sem `Params`: o interruptor é o campo, como na E5) |
| `wp/sim.rs:497-499` (estágio 1, longe de `pickup_assentado` `:80,177,290,391`) | com Pigmento, de `PASSOS_DA_CURA` em `PASSOS_DA_CURA` passos: `drying::cura(g, PASSOS_DA_CURA/40 s)` |
| `wp/render/tinta.rs` `altura_fechada_em_massa` | o `TODO(E8)` fechado: `altura_fechada · M_UM` |
| `wp/drying/tinta_tests.rs` (NOVO) | os testes abaixo |

## A lei

- **Assentar** (`dry_cell`, `dry_canvas`): a massa `dm` de `susp → sett` (granulação incluída) ⇒
  `transfere(filme → aberto, dm/susp)`. Sem a inversa de Newton (R16 morre no ramo Pigmento): a óptica junta filme e
  aberto (`optica_celula`), logo o ecrã não muda por construção. A abertura do aberto é média pelo volume com o que
  chega fresco (`o += (1−o)·chega/(antes+chega)`; `o = 1` fica `1` ao bit). Espelho `sett_rgb ← srgb(aberto)`.
- **Re-molhar** (a lift do passe de secagem): fracção `b·o` do aberto volta ao filme, na massa com a aritmética de
  `lift_settled` (com `o = 1` a massa é a da lei velha ao bit) e nas somas por `transfere`. Espelho `susp_rgb`.
- **Evaporar**: a água (`film`) segue a lei de hoje (`retention`, `evap_base`, borda); nenhuma soma lê ou escreve
  a água ali — a cor não mexe na evaporação (o «molhado escurece» é `s_fator(m, film>0)` no ecrã, E5).
- **Abertura**: `o ← o·e^{−Δt/τ(m)}` em toda célula com aberto e `m > 0`, Δt = `PASSOS_DA_CURA/40` s (40 passos =
  1 s; exponenciais compõem-se, por isso juntar o tempo é exacto com `m` constante).
- **`τ(m) = TAU_OLEO_S / m`**, `TAU_OLEO_S = 3600 s`. Escolha física (ADR-0197 §1 item 5, F2 §2 «τ(1) ≈ horas»): só o
  veículo que OXIDA (a fracção `m` de linhaça) cura, à taxa `m/τ_óleo`; a goma arábica da aguarela re-dissolve sempre ⇒
  `τ(0) = ∞` — **a aguarela não fecha** (levantá-la seca é a lei dela; a «Staining» é o `k_u`, não o fecho). F2
  dizia `τ(0) ≈ segundos`: recusado — fecharia toda aguarela em segundos e mataria o levantar.
- **Fecho** (`o < O_FECHA = 0,01` E célula SECA: `film == 0` e `filme.v == 0`): `altura_fechada += aberto.v`;
  `fecha(&mut r_g, &mut aberto, s_fator(m, false))`; `o = 1`; `sett = 0` (invariante: `aberto.v = 0`). Só seco porque
  aí o fecho é invisível (o filme por cima trocaria a ordem das camadas, §2.2) e o `s_fator` do aberto é o seco.
- **Altura**: `massa_da_altura = sett + susp + altura_fechada·M_UM` — o Óleo Real (`oleo_campo.rs:67`) vê a mesma
  altura antes e depois do fecho.
- `m` continua `0` em toda célula até à E10 ⇒ **hoje nada fecha no produto**; o caminho do fecho só corre com `m > 0`
  (testes).

## Testes

| teste | resultado |
|---|---|
| Velha ao bit (`--test it`): `a_borda_graduada` (3, inclui `…a_secagem_de_antes_ao_bit` e `…bits_gravados`), `the_parallel_jacobi_drying_is_the_serial_one_to_the_byte`, `fast_dry_survives_the_live_span_to_the_byte`, `fingerprint` (3), `a_step_run_stage_by_stage_is_byte_identical_to_the_atomic_one`, `the_gauss_seidel_drying_leans…`, `product_rewet` (5), `t03_wet_and_dry_tools`, `t07_blend…` | verdes |
| `drying::tinta::tests::*` (`assentar_nao_muda_o_ecra_ao_bit_no_motor`, `fechar_e_o_que_se_via`, `a_secagem_conserva`, `a_altura_do_oleo_nao_some_no_fecho`, `a_aguarela_nao_fecha`) e `o_interruptor_da_tinta_nao_muda_um_pixel_com_a_lei_velha` | ver «Em aberto» |

Tolerâncias declaradas: assentar ≤ 1/255 por canal (as somas movem-se em f32 por `transfere`, ≤ 4 ulp, e o pixel é
`u8`; o «antes» é a grade vista seca — `film = 0` — para isolar o assentar do «molhado escurece»); fecho: RGBA ≤ 1/255
nas cores em que `α = max(1 − R_w)`, e sobre branco ≤ 1,5/255 em todas; conservação `Σ(filme.v + aberto.v +
altura_fechada)` rel. < 1e-5; altura rel. < 1e-6.

## Em aberto

- **Cobertura do fechado**: o ecrã «exacto sobre branco» (E5) dá `α = max(max(1 − R_w), média C)`. Depois do fecho a
  célula só tem `r_g`, logo `α = max(1 − R_g)`: numa tinta CLARA opaca (branco) o RGBA perde α (sobre branco é igual,
  sobre a camada de baixo deixa-a ver). Cura: um campo «cobertura do fundo» `c_g` na `CelulaTinta`
  (`wp/grid/tinta.rs`, fora dos paths da E8) ou `α` pelo par `(R_g|branco, R_g|preto)`. O teste `fechar_e_o_que_se_via`
  mede o branco só sobre branco.
- **Emboss/papel** (`wp/render.rs:542,552-555`) lê `sett + susp`: depois de um fecho o relevo do pigmento e o `fade`
  da granulação somem. Cura (fora dos paths): ler `tinta::massa_da_altura(lg, j)`. A ladeira do óleo (E7) idem.
- Quem ainda lê `sett` como «tinta» e passa a ver `0` no fechado: borda da secagem (`sett_c < 1000`), `pickup_assentado`
  e o re-molhar (correcto: o fechado não se levanta).
- O relógio da cura anda só enquanto o passo corre (`has_fluid`): a tela toda seca não envelhece. Um relógio próprio
  é da área Tempo (E10/E11).
- O depósito seco (E6a `pousa` ponta seca → `sett`) põe tinta fresca no aberto sem refrescar `o` (média pelo volume
  só no assentar); com `m = 0` é inerte.
- `drying/tinta.rs::dry_jacobi` recolhe as linhas num `Vec` antes de as andar (o `par::walk_rows5` não tem 6.ª faixa);
  o custo não foi medido (só corre com Pigmento).
