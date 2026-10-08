# HANDOFF de integração — `line/motion-value`, CICLO 14 (2026-10-08)

> Os oito abertos que o ciclo 13 deixou, num ciclo (ordem do dono, 08/10). Doc do ciclo:
> [123](../123_ciclo_14_os_oito_abertos.md) — o estado medido, os desenhos e os kill-criteria de cada
> item foram escritos ANTES de construir; as medições estão nas tabelas de lá. A linha NÃO integra e
> NÃO pusha: integrar é por ordem do dono.

## §1 — Base e commits

- Base: `main` `2a925859d`. A linha leva os `11` commits do ciclo 13 (handoff
  [CICLO_13](HANDOFF_INTEGRACAO_line_motion_value_2026-10-07_CICLO_13.md), por integrar) e, por cima,
  os commits do ciclo 14 (`git log --oneline 2a925859d..line/motion-value`).
- `git diff --stat 2a925859d` (os dois ciclos): `~254` ficheiros. Do ciclo 14: `~120` são só a RAZÃO de
  um `#[ignore]` (o prefixo de classe, item 5); o resto está no §3.

## §2 — O que fechou (os oito, e o que a família destapou)

| # | item | veredito | gate / prova |
|---|---|---|---|
| 1 | imunidade do `pulse.beat` a FMA sem régua | régua de CPU (`f32::mul_add`, o FMA verdadeiro) sobre toda `i < 2^17` no fio do empate, com o `sc` do produto e controlo com o `sc` cru; e uma fixtura NA PLACA no fio (`FIXTURA_DO_FIO_DO_FMA`). ⭐ Medido: com a mutação «sem quantizar» a paridade da placa reprova nas DUAS placas (RTX e iGPU fundem) | `the_beat_index_is_the_same_fused_or_not` · `the_device_fixture_sits_on_the_fma_edge` · `parity_pulse_beat_on_the_fma_edge` |
| 2 | `write_the_value_figures` lia `0` | a `=117` carimba PRIMEIRO; porta do produto (`publish_all` + o `Cook` da bomba) e a PROPORÇÃO saída/carimbo (a 1.ª figura regenerada saía recta com os gates verdes — foto lado a lado) | o gate do item 3 |
| 3 | figuras dos tutoriais velhas | nove geradores partidos em produzir/escrever, registo único, três gates; tutoriais `04`/`05` tinham a tabela COLADA (passou a incluída); os `01`–`05` ganharam o gate do cartão (sete passos impossíveis curados no texto); as promessas de «milhão» (`01`/`02`/`03`/`07`/`08`/`09`) reescritas com números MEDIDOS a `32 761`; PDFs refeitos (os nove) | `as_figuras_dos_tutoriais_sao_as_que_os_geradores_produzem` (vermelho com `7` figuras velhas) · `todo_gerador_de_figuras_esta_no_registo` · `toda_tabela_gerada_e_incluida_e_nenhuma_e_colada` · `every_row_the_{arrangement,animators,deformers,fields,simulation}_tutorial_names_is_on_the_card` |
| 4 | cinco sondas reprovavam sozinhas | derivadas das constantes de hoje; a `measure_the_source_group` desliga a lei na BOMBA. ⭐ A `=53` era a ponta de uma FAMÍLIA: `45` arneses avançavam o tique ANTES de cozer (cada nó sequencial cozia duas vezes). Postos na ordem do produto; o que isso destapou: a guarda de NaN da mola deixava sair UM quadro não finito (curado na CPU e na WGSL), a injecção da `=57` estava calibrada sobre o depósito duplo (`0,25 → 0,42`), o gate do corpo mole media a régua que o próprio cabeçalho proibia (Bug #15 aberto). ⭐ E a lei do dono tinha DUAS fontes (a rota da placa e quatro leitores liam o ambiente) — uma só agora, a da bomba | `the_ceiling_is_honoured_on_every_tick_including_the_turn` (saiu do `#[ignore]`) · `nenhum_laco_avanca_o_tique_antes_de_cozer` · `the_device_route_reads_the_pumps_law` · `the_environment_law_is_read_in_one_place` |
| 5 | a suíte da placa não cabia na fatia | `298` `#[ignore]` com classe (`30` `placa:` · `259` `sonda:` · `9` `gerador:`); `scripts/motion-gate-placa.sh` corre só os `placa:` — **`30/30` em `96 s`** com a compilação (eram `>1800 s`, por duas sondas que não acabam em `900 s`) | `every_ignore_in_the_motion_crate_names_its_class` |
| 6 | SIGSEGV ao sair do `it` do `ph2d-gpu-cook` | medido no MESMO binário: um contexto por ficheiro `8/8`, contexto ÚNICO `0/16` (com e sem gancho de saída). ⇒ o contexto único; o gancho `atexit` (e a crate de excepção que ele pedia) SAIU | `only_this_file_holds_a_gpu_context_in_a_static` |
| 7 | a `=27` esvaziava a tela | ela SUBIA e saía pela direita (`0/41` à vista aos `6 s`); `force.attractor` + `force.drag` (com kernel) ⇒ `106/106` aos `8 s`, fotografado | `the_cascade_stays_in_view` |
| 8 | `PH2D_PAN_DIAG` cego ao mundo da placa | lê o `P[0]` do primeiro sink da LEITURA DOS CARTÕES que já existe (a tomada da bomba recozinharia na CPU — o desenho da lista estava errado, e a auditoria disse porquê) | `pan_diag_o_mundo_da_primeira_copia_e_o_da_cpu` · `pan_diag_sem_leitura_dos_cartoes_nao_ha_mundo` · `pan_diag_a_sonda_desligada_nao_le_nada_da_placa` · `pan_diag_o_mundo_da_placa_e_o_da_cpu` (placa) |

## §3 — Superfície de colisão (para o integrador)

- **`crates/ph2d-app-motion/src`** (`~138` ficheiros, quase todos só a razão de um `#[ignore]`) — módulos
  novos: `motion_bridge_tutorial_figuras.rs`, `motion_ignore_census.rs`, `motion_state_tutoriais_01_a_05_tests.rs`,
  `pan_diag_tap.rs` (+`_tests`); `lib.rs` ganhou `pub mod pan_diag_tap;`, os `mod` de teste novos e
  `motion_bridge_ciclos_mods.rs` o das figuras. `motion_bridge_gpu.rs` ganhou `estilos_das_saidas`.
- **`crates/ph2d-app-motion/tests/it`**: `a_ordem_do_tique_e_cozer_e_depois_avancar.rs` + `main.rs`.
- **`crates/ph2d-gpu-cook/tests/it`**: `placa.rs` novo; `37` ficheiros trocaram o `OnceLock` próprio pelo
  `use super::placa::try_headless_gpu`; `gpu_cpu_parity_pulse.rs` e `gpu_cpu_parity.rs` (sonda nova).
- **`crates/ph2d-node-registry-init/tests/it`**: `22` laços na ordem do produto, `soft_body_radius.rs`.
- **`crates/ph2d-node-pulse-beat/src`** (`FIXTURA_DO_FIO_DO_FMA`, `pub` + `#[doc(hidden)]`),
  **`crates/ph2d-node-motion-spring/src`** (`lib.rs` + `kernel.rs`: a guarda pós-passo — muda o WGSL),
  **`crates/ph2d-pan-diag/src/lib.rs`** (`note_device_world`, o `DEVICE` ganha o mundo).
- **`shells/desktop`**: `render_loop/present.rs` e `render_loop/fase_motion_gizmos.rs` (uma linha cada:
  `motion.pump.a_lei()`); `tests/it/the_sink_style_is_one_answer_for_both_routes.rs` (o censo da porta
  única aceita a forma nova). A catraca da shell não cresce (troca de linha).
- **`scripts/motion-gate-placa.sh`** novo. **Docs**: doc 123 novo, nota no doc 103 §5, Bug #15 no BUGS,
  `README.md` do módulo (regenerado), os nove tutoriais (src + PDFs), `fig/params*.html` e `valor_*.svg`.
- ⚠️ Nenhum contrato congelado tocado (§6 do CLAUDE.md). Nenhum `PROJECT_SCHEMA` mexido.

## §4 — Gate batched (sobre `2a925859d..` a árvore)

| passo | resultado | carga |
|---|---|---|
| `nextest-impacted.sh` (`BASE=2a925859d`, inclui `file_loc_caps`/órfãos/`claude_md_cabe`) | ✅ `4932/4932` na árvore final (antes da saída do gancho: `16387/16388`, o vermelho era o censo da porta única da shell, curado) | `5–33` |
| `RUSTFLAGS=-D warnings cargo check --all-targets` (as nove crates tocadas) | ✅ | `4,6` |
| `cargo clippy --all-targets -D warnings` (as nove) | ✅ | `15,8` |
| `cargo fmt --all --check` | ✅ depois de `cargo fmt` (10 hunks) | — |
| `censos-da-arvore-combinada.sh` | ✅ `114/114`, `12/12` censos | — |
| `doc-index.sh --check` | ✅ `17` índices | — |
| placa `scripts/motion-gate-placa.sh` (os `30` `placa:` do `ph2d-app-motion`) | ✅ `30/30`, `96 s` | `8,4` |
| placa `ph2d-gpu-cook --release --include-ignored` (`it`) | ✅ `340/340` nas corridas sem o flake de fan-out (`crossing_the_reach_boundary_does_not_step_the_cost`, já em `FLAKES_DE_CARGA`, reprovou em `6` de `24` sob carga); `0` SIGSEGV em `26` corridas | `4–39` |
| paridades do `pulse.beat` na RTX e na iGPU | ✅ `3/3` e `3/3` | — |

## §5 — Provas de mutação (pré-voo das âncoras, alvo visto VERDE antes, corrida limpa no fim, `touch`)

| # | mutação | gate | resultado |
|---|---|---|---|
| 1 | `quantiza` = identidade | `the_beat_index_is_the_same_fused_or_not` + `the_device_fixture_sits_on_the_fma_edge`; e na placa `parity_pulse_beat_on_the_fma_edge` | sangrou (CPU linha `5`; RTX e iGPU) |
| 2 | o offset da fixtura `159 000 → 158 000` | `the_device_fixture_sits_on_the_fma_edge` | sangrou |
| 3 | o `advance_tick` de volta antes do `cook` na `=53` | `the_ceiling_is_honoured_…` | sangrou (`0,1678`) |
| 4 | a guarda pós-passo da mola apagada | `spring_ceiling::far_above_…` | sangrou (`Explodes`) |
| 5 · 5b | `ATRACAO 0` · `ARRASTO 0` | `the_cascade_stays_in_view` | sangraram (aos `4 s`) |
| 6 | `note_device_world` fora do `if on()` | `pan_diag_a_sonda_desligada_nao_le_nada_da_placa` | sangrou |
| 7 | um espaço no fim de `fig/grid.svg` | `as_figuras_dos_tutoriais_…` | sangrou |
| 8 | `kill_radius.rs` invertido | `nenhum_laco_avanca_o_tique_antes_de_cozer` | sangrou |
| 9 | a rota da placa ignora a lei da bomba (`so_com_forma: true`) | `the_device_route_reads_the_pumps_law` | sangrou |
| 10 | um 2.º leitor do ambiente (`lei && so_com_forma_por_ordem()`) | `the_environment_law_is_read_in_one_place` | sangrou (o comportamental sobrevive a esta, por construção: com o ambiente ligado as duas leis coincidem) |
| 11 | um `static … GpuContext` num 2.º ficheiro do `it` | `only_this_file_holds_a_gpu_context_in_a_static` | sangrou |

Os gates de texto/censo das figuras (`todo_gerador…`, `toda_tabela…`), do `#[ignore]` e dos tutoriais
tiveram o vermelho-primeiro sobre a árvore de partida (o disco velho, as tabelas coladas, os sete passos).

## §6 — Conhecidos que NÃO são deste ciclo

- `crossing_the_reach_boundary_does_not_step_the_cost` — flake de fan-out registado (os testes do `it`
  correm em paralelo na mesma placa); não reaberto.
- O `ph2d-render` tem o mesmo desenho do SIGSEGV (`~20` `OnceLock` de contexto nos `tests/it`) — não
  medido (fora do módulo). A cura medida aqui é o contexto único.
- A sonda `FIELD_DOC_VERSION` do `scripts/collision-surface.sh` (herdada do ciclo 13).

## §7 — O que fica ABERTO

- **Bug #15** — o colisor no laço de um corpo mole não segura a distância mínima (pior par não vizinho
  `0,21` contra `0,56`); nunca segurou. A cura é um desenho do `motion.collide` (ignorar pares ligados
  por restrição) — fora do âmbito; o gate mede a lei que vale (a extensão, `+20 %`).
- ✅ **Decidido pelo dono em 08/10**: o `motion.soft_body` passa a respeitar o tecto de `32 768` (hoje vai
  a lado `512`, `262 144` pontos), e o Bug #15 cura-se pelo padrão-ouro — os dois vão ao ciclo 15:
  [prompt](HANDOFF_CONTINUACAO_line_motion_value_2026-10-08_CICLO_15.md).

## §8 — Smoke e binário

Smoke ao dono (em `--release`): a `=27` com a cascata à vista (fotografada a `5`, `9` e `11 s` de relógio
— o arranque come `~3 s` —, `target/prova/c14/fotos/`) e o tutorial `01` (texto e figuras novos). O
binário `ph2d-host-desktop --release` está compilado; a 2.ª corrida:

```
▸ linha line_motion_value · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `release` profile [optimized] target(s) in 0.22s
```
