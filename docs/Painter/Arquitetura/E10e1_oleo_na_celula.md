# E10e1 — o «óleo sobre óleo» lê a CÉLULA, não o pincel na mão (01/11)

Agente E1, HEAD `4e725aff41`. Spec: `E10d3_vermelhos_antigos.md` §1. `wp/` = `crates/ph2d-wet-paint/src/`,
`tp/` = `crates/ph2d-tool-painter/src/tool/paint/`.

## Mecanismo (regressão de `776a9ff173`)

A lei da SESSÃO levava `PINCEL_DE_OLEO` (−1) em `Ecra::Cobre::oleo` quando o Real Oil estava na mão
(`wp/sim.rs` `gather_params`). O render, a secagem e as transferências usam essa lei, e `Ecra::com_oleo` escolhia
o ramo pelo sinal: `< 0` → `com_opaca` (o tapar extra fica), `≥ 0` → o extra sai pela fracção de óleo da célula.
Ligar ou desligar a caixa trocava o ramo de TODA a célula com óleo já pousada.

## Cura

| sítio | mudança |
|---|---|
| `wp/grid/oleo_celula.rs:33` | `OleoCelula::pincel_de_oleo: bool` — o Real Oil da ÚLTIMA mão que pousou na célula. `NEUTRA` = `false`, `do_pincel` = `true` com a caixa (`p.oleo > 0`). |
| `wp/grid/oleo_celula.rs:115` | `pousa` SOBRESCREVE o flag (não mistura pela massa); `veste_virgens` já copia a `nova` inteira. `restaura_janela` só larga o campo se o flag também é `false`. |
| `wp/assenta.rs:158` | `com_oleo`: ramo opaco = `c.oleo > 0 && c.pincel_de_oleo`; o outro = `c.oleo > 0`. A constante `PINCEL_DE_OLEO` saiu. |
| `wp/sim.rs:418` | `oleo: 0.0` sempre na lei da sessão. O doc de `Sim::oleo` («não muda a tinta pousada») passa a ser verdade, sem edição. |

Por que um campo novo e não o `oleo` existente: `oleo` é a fracção pela MASSA. Uma aguada sem óleo sobre óleo
deixa `oleo` entre 0 e 1, como deixaria óleo sobre óleo com pouca massa. O valor não diz quem foi a última mão.

Ordem no depósito: `trail/transfer.rs:174` (`pousa`) corre antes de `:241` (Glaze → `com_oleo`), por isso o dab
que pousa já lê o flag do pincel atual. Os outros chamadores (`render.rs:448`, `drying.rs:177`, `transfer.rs:658`,
`tools/smear.rs:276`, `tools.rs:272`, `grid/canvas_ops.rs:65`) ficam sem mudança.

**Tamanho:** `OleoCelula` 24 → **28 B** (1 B do `bool` + 3 B de alinhamento), portão em `oleo_celula_tests`.

**GPU:** nada a mudar. `ph2d-wet-gpu` não espelha `OleoCelula`, e `placa_sabe` (`passo.rs:31-33`) exige `p.oleo == 0`
e `g.oleo` vazio. A placa nunca vê óleo. `seca.wgsl:231` lê `u.oleo >= 0`, que agora é sempre o caso. GPU não corrido.

**Ao bit sem Real Oil, por construção:** sem a caixa, a sessão já dava `oleo = 0` e as células têm o flag `false`.
Uma cena sem Real Oil passa pelas mesmas contas.

## Testes (por nome, `--profile smoke`)

| teste | antes | depois |
|---|---|---|
| `tp` `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` | ✗ | ✓ |
| `tp` `o_real_oil_na_mao_nao_empalidece_a_aguada_sobre_o_oleo` (J11 item 0) | `#[ignore]`, ✗ | ✓ — `#[ignore]` retirado |
| `tp` `pintar_tudo_com_o_real_oil_e_a_imagem_de_antes_ao_byte` (o óleo cobre o óleo) | ✓ | ✓ |
| `tp` `portao_mae_f0_lei_pigmento_minimo` · `painel_so_no_futuro` (6) · `pincel_nao_recompoe` | ✓ | ✓ |
| `wp` `assenta` (26, incl. `assenta_bit_tests` e `oleo_sobre_oleo_tests`) · `ecra` (10) | ✓ | ✓ — hashes iguais |
| `wp` `oleo_celula` (6, tamanho 28 B e um assert novo sobre a sobrescrita) · `meio_corpo` (3) · `bingham` · `historia` (6) · `oleo_real` (42) · `tinta` (71) · `o_invariante_soma_massa` | ✓ | ✓ |
| `wp` `advect_jacobi_bits_are_the_recorded_ones` / `_by_each_commit_route` | ✓ | ✗ **não é desta cura**: só mudam os dois casos `km = true` (índices 1 e 3). A cena não usa o Real Oil, e a mudança vem do `wp/colorops.rs` do E2, ainda por commitar (K-M com cantos iguais, E10d3 §2). |

Testes alterados sem mudança de lei: `assenta_bit_tests` (a linha da cena passa a `-1.0` literal, porque a porta
ainda prende o negativo a 0; o hash é o mesmo) e `oleo_sobre_oleo_tests` (`lei(PINCEL_DE_OLEO)` → `lei(0)` com a
célula `OLEO_DO_PINCEL`).

## Aberto

1. `advect_jacobi_bits` são do E2 (K-M). Regravar com a prova dele.
2. Um projeto salvo: `OleoCelula` não é serializado campo a campo (snapshot e patch copiam a struct), por isso não
   há formato a migrar. Se a `.ph2dproj` passar a guardar o campo, o flag vai junto.
3. `seam_wetpaint_oleo_real` (painel, 8 testes): ✓.
4. `remolha_tests::a_aguada_re_molha_o_oleo_soluvel` continua ✗ na linha 154 («o azul debaixo da aguada caiu …»),
   o mesmo vermelho que a E10d3 §4 já mediu no HEAD antes desta cura. Pede uma bissecção própria. Os outros
   testes de `remolha_tests` estão verdes, incluindo o J11 sem `#[ignore]`.
