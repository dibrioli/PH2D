# E10e2 — o preto do K–M, o portão do re-molhar e a placa com a Absorbency

Agente E2, 01/11. HEAD `4e725aff41`, `--profile smoke`. Prazo de 45 min. Os três itens estão abaixo, cada um com
o que ficou aberto.

## 1. A média K–M sai do invólucro das entradas — curado na média, ABERTO no `Km.mix`

- **Mecanismo** (E10d3 §2): `ks_of_srgb255` prende a reflectância em `R_FLOOR = 1/255` linear
  (`ph2d-pigment/src/transfer.rs:175`). Abaixo de 12,709/255 toda cor tem o MESMO K/S e a volta devolve 12,709. Há
  ainda o erro da volta tabela→`sqrt` (≤ 4e-6). Em aritmética exacta, a média convexa de K/S fica entre o mínimo e o
  máximo das entradas, porque `K/S → R` é monótona. Só o piso e o erro da volta a fazem sair.
- **A lei** (geral, não é um caso especial): **a média não sai do invólucro das entradas com peso > 0**. Daí sai a
  idempotência (cores iguais devolvem a própria cor, ao bit). Um canto de peso 0 não alarga o invólucro.
- **Cura:** `crates/ph2d-wet-paint/src/colorops.rs` `km_weighted_mean_color` (≈ l.380–410). O `lo`/`hi` de cada
  canal sai dos cantos pesados, e o resultado é `v.clamp(lo, hi)`. Sem nenhum canto pesado, fica a conta.
- **Testes novos:** `a_media_km_de_cores_iguais_e_a_propria_cor` e `a_media_km_fica_no_involucro_das_entradas`,
  os dois verdes. A sonda `diag_vies_do_piso_perto_do_preto` está `#[ignore]`.
- **Viés do piso nas misturas DIFERENTES** (medido 1:1, a sonda acima; o invólucro não as toca):

  | entradas | média |
  |---|---|
  | preto + 10 | 10 (o invólucro; antes 12,709) |
  | preto + 12,709 | 12,709 |
  | preto + 20 | 15,62 |
  | preto + 40 | 19,23 |
  | preto + (40,90,220) | (19,23 · 21,10 · 21,54) |
  | preto + branco | 21,55 |

  Fica no invólucro, mas o «preto» do modelo tem `R = 1/255`. Um preto ideal (K/S → ∞) daria ≈ 0 em todas as
  linhas. Baixar o piso escurece TODA mistura que tenha um canal 0 (o vermelho puro + branco deixaria de clarear o
  verde). É uma decisão de desenho para o coordenador e não a tomei. A volta já é a identidade (≤ 4e-6) no conjunto
  codificável (≥ 12,709). O defeito estava só abaixo do piso, e o invólucro resolve-o onde ele sai das entradas.
- **Hashes:** `advect_jacobi_bits_are_the_recorded_ones` e `…_by_each_commit_route` mudaram SÓ nos dois casos K–M
  ([1], [3]). Prova no MESMO binário, com o invólucro desligado por uma variável de ambiente temporária (já saiu):
  sem ele voltam os gravados `0x1cb2f71655da9adf`/`0x011807f8c6bca2fd`; com ele dão `0x2bf143335a9bf102`/
  `0x0418b1ad31b24a60`. Por célula (sonda temporária, 1 passo da cena K–M de tecto 1): mudam 1021 células. A tinta
  `10` fica `10` (antes `12,709223`, Δ −2,709). As outras 542 diferenças são ≤ 4e-6 (por exemplo `59,999996 → 60`:
  o erro da volta encostado à entrada). Regravado em `solver/advect_bit_tests.rs` (`GRAVADO`, com o porquê).
- **Portões:** `advect_jacobi_` 4 ✓ · `o_invariante_soma_massa` 1 ✓ · `tinta` 71 ✓ · `km_` 5 ✓ · `colorops` 7 ✓ ·
  ph2d-pigment `transfer` 3 ✓ (`pigment` não casa nenhum nome) · tp `portao_mae_f0_lei_pigmento_minimo` ✓.
- ⚑ **ABERTO — `a_identidade_do_modo_nao_pinta_sobre_a_tela` continua vermelho (5458 texels, Δ 9, o mesmo número).**
  A média da advecção não é a única porta. `ColorMix::Km.mix` (`ph2d-pigment/src/lib.rs:≈74–103`) com peso
  interior `0 < w < 1` passa pelo mesmo piso: preto sobre preto dá 12,709. Só os extremos `w = 0` e `w = 1` são
  exactos. A cura é a MESMA lei, `out[ch] = v.clamp(min(d,s), max(d,s))` em cada canal do braço `Km`. Não a apliquei:
  muda o erro da volta (≤ 4e-6) em TODA chamada K–M interior (depósito, assentar, lift, Blend/Smear), e esses
  portões ao bit eu não consegui correr no prazo. Portões a correr depois: `flow_bit_tests`, `assenta_bit_tests`,
  `advect_jacobi_`, `tinta`, `oleo_sobre_oleo`, as miniaturas, `blend_tests`. Outras portas K–M a varrer pela mesma
  lei: `solver/sedimenta.rs:162` (já passa pela média curada) e `km_glaze_channel_linear`.

## 2. `remolha_tests::a_aguada_re_molha_o_oleo_soluvel` — bissecção INCOMPLETA

O alvo do teste é «caiu 0,25–0,45». Medido na árvore inteira (`target/e2_bisect/`, worktree destacada com o seu
próprio `CARGO_TARGET_DIR`, já removida):

| commit | caiu |
|---|---|
| `0eaf6fa1ed` (pai da E4) | **0,470** (vermelho por pouco) |
| `1279289911` (pai da E8) | **0,888** |
| HEAD (E10d3) | vermelho |

- A queda andou em DOIS degraus. Um está em `6c243555f0..0eaf6fa1ed` (0,35? → 0,47). Suspeitos pelas mensagens:
  `847b472b3d` (a queda cresce com a diluição), `3eb13d38db` (K–M de fábrica: a aguada mistura em K–M),
  `6032edee62`/`c87ef602f9` (óleo sobre óleo), `c5e6daa0a9` (Bingham). O outro está em `7a4bc540b7..1279289911`
  (0,47 → 0,89). Suspeitos: E4 `7a4bc540b7` (a água sem a pressão inversa: mais água na aguada), E6a `8a5b179b48`,
  E7 `ed0549721e`.
- A hipótese do briefing (E10 A2, o meio passa para `Knob::Medium`) fica **refutada como causa inicial**: o teste
  já estava vermelho muito antes do E10. Não há classificação final (regressão ou cena velha) e não há cura.
  Próximo passo: 2 builds, `7a4bc540b7` e `847b472b3d^`.
- `sem_re_molhar_a_aguada_sobre_o_oleo_e_a_de_sempre` está verde em `0eaf6fa1ed`.
- ⚠️ O alvo `target/e2_bisect/target` (build de árvore inteira) continua no disco. Peça para apagá-lo.

## 3. A placa segue a Absorbency — feito, por compilar só

- `crates/ph2d-wet-gpu/src/seca.rs:55`: `retention: p.retencao() as f32` (era `p.k(Knob::Retention)`). É a mesma fuga
  escalada da CPU (`DryConst::new`, E10d2). No neutro (`absorcao = 1`), `retencao()` devolve `r` ao bit.
- `cargo-check-narrow ph2d-wet-gpu` ✓. Os testes de placa NÃO correram (NVIDIA proibida), e também não corri
  `--no-run`, por falta de tempo.

## E3 (01/11) — o `Km.mix` no invólucro e os dois degraus do re-molhar

### E3.1 `ColorMix::Km.mix` fica no invólucro — feito

- `crates/ph2d-pigment/src/lib.rs`, braço `Km`: cada canal sai `v.max(min(d,s)).min(max(d,s))` (a mesma lei da média
  da E2; `max/min` em vez de `clamp` para não entrar em pânico com `NaN`). Teste novo `a_mistura_km_fica_no_involucro`
  (preto sobre preto = preto em todo peso interior; preto/branco e cinzas abaixo do piso no invólucro) ✓.
- ✅ **`a_identidade_do_modo_nao_pinta_sobre_a_tela` VERDE** só com esta cura (o `Km.mix` era a porta que faltava).
- O oráculo de `tests/it/transfer_accuracy.rs` (`exact_km_mix`) leva o MESMO invólucro: o
  `a_still_wash_tracks_the_exact_transfer_over_thousands_of_re_mixes` media a deriva da cadeia `libm` SEM invólucro
  (12 → 12,7) — com o invólucro ligado e o oráculo velho falhava; com o invólucro desligado (variável temporária, mesmo
  binário) passava. A lei mudou de propósito, o oráculo segue-a. Verde.
- **Nenhum hash gravado mudou** em ph2d-wet-paint (corridos com a cura): `advect_jacobi_` 4 ✓ · `bit` 30 ✓ ·
  `o_invariante_soma_massa` 1 ✓ · `tinta` 71 ✓ · `km` 5 ✓ · `colorops` 7 ✓ · `deposit` 14 ✓ · `settle` 1 ✓ ·
  `assenta` 26 ✓ · `lift` 3 ✓ · `blend` 5 ✓ · `flow_bit` 2 ✓ · `oleo_sobre_oleo` 4 ✓ · `transfer_accuracy` 8 ✓
  (`pickup` não casa nenhum nome; `mix` = os `transfer_accuracy` + 5 ✓). ph2d-pigment: `transfer` 3 ✓ · os 4 da lib ✓.
  tp: `blend_tests` 2 ✓ · `portao_mae_f0_lei_pigmento_minimo` ✓.
- ⚑ **Vermelho que NÃO é desta cura:** `a_agua_do_smear::o_water_do_smear_no_meio_e_a_lei_de_antes` (wp) dá
  `0x76d1277d6f22d420` com e sem o invólucro (mesmo binário) — vem da árvore (E1/E2 ou antes).

### E3.2 `a_aguada_re_molha_o_oleo_soluvel` — os dois degraus achados; cura do produto ABERTA

Bissecção na árvore inteira (`target/e3_bisect/`, uma worktree destacada, 3 builds, já removida):

| commit | caiu |
|---|---|
| `7a4bc540b7` (E4) | 0,470 — a E4 NÃO é degrau (o pai dá o mesmo 0,470) |
| `c5e6daa0a9` (Bingham) | **0,888** |
| `b7f54356d6` | 0,888 |

Sonda no HEAD (`remolha_tests::diag_a_queda_por_lei`, `#[ignore]`; o Bingham desligado pelo override de medição
`solver::PAPEL_BINGHAM < 0`, o K–M pelo `km_mixing` entre o preset e o traço):

| Bingham | K–M | óleo azul antes → depois | caiu | azul fora antes → depois |
|---|---|---|---|---|
| sim (produto) | sim (preset) | 656 354 → 98 812 | **0,849** | 471 644 → 510 278 (×1,08) |
| sim | não | 656 589 → 254 093 | 0,613 | 466 682 → 359 131 |
| não | sim | 656 549 → 338 800 | 0,484 | 474 715 → 1 153 023 |
| não | não | 656 779 → 416 045 | **0,367** ✓ | 469 612 → 826 312 |

- **Degrau 1 = `3eb13d38db` (K–M de fábrica, J11 item 1, decisão do dono) — cena VELHA, não regressão.** A régua
  `azul = b − max(r,g)` lê como «menos azul» o verde que o K–M faz do amarelo sobre o azul assentado: +0,12 sem nada
  dissolver a mais. (A E10d3 §2 já tinha `3eb13d38db` como primeiro vermelho do Blend pelo mesmo interruptor.)
- **Degrau 2 = `c5e6daa0a9` (Bingham, `RELEVO_oleo_ondas_e_crateras.md` §6) — REGRESSÃO fora do âmbito da lei.**
  O doc diz «sem óleo (`m = 0`: aquarela, guache…) `τ_y = 0`, ao bit». Mas `m` é o óleo da CÉLULA
  (`flow_jacobi.rs`, `da_celula(oleo, fi).oleo`), e a aguada do Wet Flat corre sobre a célula do Dabber. O re-molhar
  passa o assentado para o `susp`, e `massa_movel = susp` passa a contá-lo: `φ` sobe, `τ_y > 0`, a água da aguada
  pára sobre o óleo. O dissolvido não sai (fora ×1,08 contra ×1,75 sem Bingham; o 2.º assert também falha) e, com a
  água parada, o re-molhar continua a dissolver (0,85). O recurso J10 aprovado pelo dono («o dissolvido viaja na água
  para fora do óleo») morre.
- A hipótese «o E10 tirou o meio da cena» não se aplica: a cena não liga o Real Oil (`oleo = false`), e o óleo da
  célula vem do preset do Dabber.
- **Cura — ABERTA, decisão de desenho do coordenador** (o sítio, `solver/flow_jacobi.rs` + `ladeira_do_oleo.rs`,
  está fora dos meus caminhos e é lei do óleo). Candidatas, a medir com a sonda acima e a régua `sonda_relevo_oleo`:
  (a) `m` do Bingham = o óleo da tinta que ESCOA (o Real Oil/Medium do traço vivo), não o do chão; (b) o pigmento
  re-molhado não entra em `massa_movel` (precisa de saber a origem do `susp`). Depois da cura: a régua da cena passa a
  ler a massa (`1 − r/255`, a mesma do «fora») ou fixa `km_mixing = false`, para medir o dissolvido e não o verde.

### E3.3 Portões da tp que já estavam vermelhos, e o que a cura lhes faz (mesmo binário, invólucro ligado × desligado)

- `familias::mistura::cada_preset_da_agua_cai_na_foto_do_seu_rebelle` — vermelho IGUAL nas duas (Smear Soft e Spikey:
  `Fundo 91` fora da foto `119 ± 25`). Não é desta cura.
- `presets_de_fabrica::portoes::a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` — vermelho nas duas (31 miniaturas
  mudam sem o invólucro: a árvore de hoje, E1/E2 e antes). A cura MUDA só estas, de quase nada para muito:
  Blender Dots/Hard/Soft/Textured (225/11/13/698 px → 26–33 mil px), Smear Flat/Shattered/Soft/Spikey/Wet
  (19–251 px → 16–19 mil px) e Water Brush (18 425 → 31 375 px). Por canal (sonda temporária, já apagada): Blender
  Hard `max Δ 2 → 13`, 25 382 canais com Δ > 2; Smear Soft `1 → 13`, 14 159; Water Brush `223 → 234`. O Δ máximo
  é o piso (12,709): os canais escuros que a mistura levantava a 13 ficam no invólucro (0). É a lei pedida, não ruído.
  ⚑ **Regerar a fábrica é do coordenador** (o portão já estava vermelho por outras mudanças; regerar UMA vez depois
  de E1/E2/E3).
- A variável de ambiente temporária (`E3_SEM_INVOLUCRO`) e a sonda das miniaturas já saíram do código.
