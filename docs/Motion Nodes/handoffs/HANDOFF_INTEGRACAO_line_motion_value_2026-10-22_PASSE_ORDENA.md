# INTEGRAÇÃO — `line/motion-value`: o passe de colisão ordena cada balde por PEÇA (22/10)

> Continua o [INTEGRATE_139](HANDOFF_CONTINUACAO_line_motion_value_2026-10-17_INTEGRATE_139.md), cujo aberto 3 fecha aqui.
> Mecanismo, censo, tabelas e recusas: [doc 120 §8.3.4](../120_ciclo_12_os_tectos_confortaveis.md) e [§8.3.5](../120_ciclo_12_os_tectos_confortaveis.md).

## O que entrou no `main`

Fast-forward sobre `af072d09d6` (a UIUX F18–F26). Rebase sem conflito; `git range-diff` contra os 5
originais: 5 `=`. Zero schema, zero ADR, zero contrato §6, zero `Cargo.toml`, zero registo (o re-sync não
gerou diff).

- `perf(gpu-cook)`: `pnp_ordena` é UM FIO POR PEÇA (o lugar de `i` no balde = quantos índices menores o balde
  tem), em vez de um fio por balde com inserção `k²` serial. Ficheiros: `crates/ph2d-gpu-cook/src/passe_na_placa.rs`,
  `passe_na_placa_varredura.wgsl`, `passe_na_placa_uno.wgsl`, `passe_na_placa_lei.wgsl` (o campo `pad` passou a
  `inverte`, só para o gate), `tests/it/passe_na_placa.rs`.
- Sondas (`motion_state_demo_carimbo_escala_tests.rs`): fases do quadro por cena × escala e por rota;
  `GpuCook::passe_sem_ordenar` (só sonda); régua `PH2D_SONDA_RAIO_POR_ESCALA`; censo das grelhas da C2.
- Docs: doc 120 §8.3.4–8.3.5; o aberto 3 do INTEGRATE_139 fechado.
- Dois commits da integração, ambos só na sonda: (1) o ficheiro estava a 704/700 (somou 262 linhas sobre
  442) — cura por corte de 5 linhas de comentário, sem código: 699; (2) `clippy -D warnings` reprovava a
  sonda — `unnecessary_mut_passed` (`&mut ph` → `&ph`) e `disallowed_types` (`HashMap` → `BTreeMap`; só
  `entry`/`get`, sem ordem de iteração).

## Números (mesmo processo, intercalado, mínimo, perfil smoke, RTX 5060 Ti; ms por quadro)

| `=137` | antes | depois | piso (sem ordenar) |
|---|---:|---:|---:|
| ×1 | 2,08 | 1,50 | 1,57 |
| ×4 | 42,7 | 22,0 | 22,5 |
| ×11 | 1 539 | **225** | 233 |

Régua realista (`PH2D_SONDA_RAIO_POR_ESCALA`, o maior disco fica o de ×1): 31 mil discos a ×11 = **7,5 ms por
quadro**, passe 6,96 ms (93 %), escala linearmente. O `amplia` puro mede contacto denso (toques ∝ `N²`),
não o tecto da grelha.

**C2 recusada pelo número (§8.3.5)** — três recusas: grelha fixa 5×5 (1,4× menos candidatos, 25 cursores:
perde) · adaptativa (2,1× menos candidatos mas cursores variáveis: ~1,1× líquido com a fusão a 1,9×, doc 132
§7) · máscara dos 9 cursores (construída, paridade 7/7: **0,96×** a ×11 realista, 1,15× denso; a divergência
no grupo come o ganho; o código saiu, a régua e o censo ficam).

## Gates

- Mutação **6/6** sangraram (M1 decrescente · M2 `<=` · M3 ordem de chegada · M4 sem ordenar em um grupo ·
  M5 metade das peças · M6 sem ordenar nos despachos). Gate novo
  `a_ordem_da_soma_num_balde_grande_e_a_da_cpu_ao_bit` (3 000 discos no MESMO balde, chegada normal e
  INVERTIDA; 0 de 3 002 peças com outros bits).
- Árvore combinada (tip `c40898a961`, load 36 durante a suíte): `CARGO_BUILD_WARNINGS=deny cargo check
  --workspace --all-targets` verde · `cargo fmt --all --check` verde · `clippy --workspace --all-targets
  --features ph2d-spike/bevy_ecs -D warnings` verde (depois da cura acima) · censos da árvore combinada
  121/121 · `nextest-impacted` com `PH2D_SUITE=fecho` **12 153/12 153** (18 203 fora do impacto; 21 SLOW,
  zero FAIL/TIMEOUT) · placa: `passe_na_placa::` 7 ok + 4 sondas `ignored` (mede, não julga),
  `colisor_nome` 5/5 · tectos de LOC: nenhum ficheiro da linha acima do tecto.

## Aberto

Os abertos 1 e 4–8 do [INTEGRATE_139](HANDOFF_CONTINUACAO_line_motion_value_2026-10-17_INTEGRATE_139.md#aberto-ordem)
continuam (custo fixo por tipo `=120`/`=132`/`=136`/`=111` · fronteiras `=98`/`=134`/`=51`/`=131`/`=120`/`=125`/`=119`
· `size` Scalar · cabeçalho do `handoffs/README.md` · os do LÍNGUA · (a)–(d) novos: entrega por porta lateral,
`value.input` Pure, flake `the_cost_of_depth_is_linear_not_explosive`, o filtro canónico do `--ignored`).
Smoke do dono: nenhum novo — `PH2D_GPU_COOK_DEMO=137` fica ~7× mais rápida a ×11.
