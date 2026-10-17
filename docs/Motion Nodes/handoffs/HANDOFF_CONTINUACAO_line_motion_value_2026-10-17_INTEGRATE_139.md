# CONTINUAÇÃO — `line/motion-value`: a `=139` simula a galáxia na placa (17/10) — o condutor com estado entra na MARCHA

> Continua o [PASSE_UNICO](HANDOFF_CONTINUACAO_line_motion_value_2026-10-16_PASSE_UNICO.md) (cujo aberto 2 fecha aqui;
> os abertos 1 e 3–7 dele seguem abaixo). Mecanismo, tabelas e recusas: [doc 120 §8.3.3](../120_ciclo_12_os_tectos_confortaveis.md).
> **Este ficheiro SUBSTITUI o PASSE_UNICO como fila viva.** Regras R1–R9 do `…_SO_O_ESTADO.md` valem.

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, base = `main` `0aab2c1cb6` (não andou). **Linha NÃO integrada**; só integra por ordem do Enio.
- Esta onda: `cb99619a93` (a cura + os gates + o doc 120 §8.3.3) · `22f433c65b` (fmt dos quatro ficheiros) · o commit de docs deste handoff.
- **Superfície de colisão:**
  - `crates/ph2d-gpu-cook/src/plan.rs`: campo público NOVO `GpuPlan::condutores_na_marcha`. Quem construa `GpuPlan` por literal parte; hoje só o `plan_uniao.rs` o faz.
  - `plan_uniao.rs`: `boundary_is_static` passou a `pub(super)`; a guarda da porta lateral antes do recuo; import de `GpuSource`.
  - `plan_condutor.rs`: `deixa_fronteira_temporal`, `condutores_com_estado`.
  - `tests/it/gpu_cpu_parity_condutor.rs`: +2 testes.
  - `crates/ph2d-app-motion/src/motion_bridge_gpu.rs`: o ramo Hybrid chama `marcha::marcha`; FullyGpu → Hybrid quando `condutores_na_marcha` não é vazio (693 → 646 linhas).
  - `motion_bridge_gpu_marcha.rs` NOVO; `motion_state_demo_fronteira_integrate_tests.rs` NOVO (2 gates + 4 sondas); `motion_state_demo_rota_anunciada_tests.rs` (o `mod`).
- **Contadores como DELTA:** `motion-gate-placa` 64 → 65 (`a_galaxia_da_139_na_placa_e_a_da_cpu`); `ph2d-gpu-cook` +2 testes de plano e +2 de paridade na placa (`gpu_cpu_parity_condutor`); zero ADR, zero schema, zero contrato §6.
- **Um merge pode partir:**
  - quem acrescente um canal CPU → placa para um valor COM ESTADO tem de o pôr na marcha (a lição do envelope congelado: o estado só anda na marcha);
  - quem construa `GpuPlan` por literal.

## O que mudou (não reconstruir)

- **Diagnóstico:** o que mantinha a `=139` na CPU era o RECUO do plano, não o kernel. Com os condutores na placa, o envelope era encenado como condutor do `strength` e deixava a `batida` (`value.input` dirigido pelo LFO) como fronteira não estática; isso expulsa toda fonte de `pre`, e o `Integrate` com ela. Ablação em tabela no doc 120 §8.3.3.
- **Planeador:** o condutor cuja cadeia deixaria uma fronteira temporal não vai à placa (`deixa_fronteira_temporal`); o plano lista `condutores_na_marcha` (fonte de param de estágio vinda do mapa da CPU cuja cadeia tem `pre` ou `Stateful`).
- **Rota:** a bomba marcha as fronteiras E os condutores na marcha; a entrega à placa filtra-se às fronteiras; com laço na placa a marcha anda em passo com ela (cada tique devido marcha-se e o mapa lê-se logo a seguir). Plano sem fronteiras e com condutores na marcha vai pelo ramo híbrido.
- **Guarda:** fronteira de porta ≠ 0 recua o estágio que a lê (a entrega é a saída 0 de cada nó, chaveada por nó).
- ⛔ **A 1.ª cura estava ERRADA** (condutor com estado cozido à parte pelo `valores_dirigidos` ⇒ envelope congelado, `max|ΔP|` 2,247) e o controlo do envelope do gate apanhou-a. Recusas MEDIDAS no §8.3.3; não reabrir.

## Resultado (tabela completa: [doc 120 §8.3.3](../120_ciclo_12_os_tectos_confortaveis.md))

- Mesmo processo, load 27–35: ×4 783 → 254 µs, ×11 1 533 → 321 µs (plano da cura).
- Depois da cura, load 29–46: B ×1 277 · ×4 280 · ×11 400 µs, contra A (CPU) 118 · 810 · 3 552.
- A ×1 a CPU ainda ganha: é decisão de rota, da mesma família da `=120`.

## Gates e mutações

- `a_galaxia_da_139_na_placa_e_a_da_cpu` (placa, em `motion-gate-placa`, agora 65): quadros de 1 a 7 tiques, com e sem estrelas; `max|ΔP|` 7,2·10⁻⁷ ≤ `EPS_SIM`; controlos: 24 forças distintas e CPU com × sem envelope.
- `um_condutor_na_placa_nunca_custa_o_laco` (sem placa, 0,2 s).
- `ph2d-gpu-cook`: `a_temporal_driver_chain_stays_on_the_cpu_and_keeps_the_loop_on_the_device` e `a_side_port_boundary_recedes_its_reader`.
- **Mutação 6/6 sangraram**, todas por asserção: M1 sem a regra do `na_placa` · M2 lista vazia · M3 sem o passo-a-passo · M4 sem a troca para o ramo híbrido · M5 sem a guarda · M6 marcha sem os condutores (detalhe no §8.3.3).

## Fecho

Em `cb99619a93` (+ `22f433c65b`):
- `PH2D_SUITE=fecho` nextest-impacted **17 768/17 770**. Os 2 vermelhos são relógio, em crates sem diff nesta linha:
  - `ph2d-timeline nesting_clock::the_cost_of_depth_is_linear_not_explosive`: sozinho 2 PASS em 6 corridas a load ~17, razão ×2,91–2,94 contra o limite linear 2,67. Flake de carga que NÃO está na tabela do `FLAKES_DE_CARGA.md` (aberto c).
  - `ph2d-tool-painter mistura_camadas_tests::n_camadas_de_amarelo_sobre_azul_seco_convergem_sem_preto`: timeout 360 s a load > 100; sozinho 48,9 s verde a load 17.
- `motion-gate-placa` **65/65**.
- `ph2d-gpu-cook` paridade `--ignored` **250/250**, FILTRADA sem `measure`/`probe`/`sonda`. Sem filtro, a corrida pendurou nas sondas `passe_na_placa::measure_*` com a placa partilhada com a `line_luz2d`. Não é comparável com os 189 do handoff anterior (o filtro dele não ficou escrito; aberto d).
- Tecto de LOC ✓ 2/2 · clippy na forma do `ship.sh` ✓ · check `--workspace --all-targets` warnings=deny ✓.
- fmt ✗ na 1.ª passagem (4 ficheiros desta onda) → `22f433c65b`; `cargo fmt --all --check` ✓.
- `target/*/incremental` reclamado.
- Binário do smoke (`cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida, depois de reclamar 3,6 GB de `target/*/incremental`): `Finished `smoke` profile [optimized] target(s) in 0.21s` — zero `Compiling`.

## Aberto (ordem)

1. **O que sobra do custo fixo, por tipo:** `=120` é decisão de rota (+325 µs contra a rota toda-CPU, só com régua por cena) · `=132`/`=136` +48/+36 µs (a cura 1 foi recusada pelo número) · `=111` (24 → 9 passes: conte os fechos por sítio antes de propor).
2. ~~`=139` com a simulação na CPU por `Integrate`~~ — **FECHADO aqui.**
3. `=137` a ×11: 1,44 s por quadro nas DUAS rotas — outro gargalo, por achar.
4. Outras fronteiras: `=98` `transfer` = 1 e 3 · `=134` Time Remap · `=51` Boids e Soft Body · `=131` Verlet Rope · `=120`/`=125` Bones/Skin · `=119` Table/Text/Camera.
5. `size` Scalar nas duas portas (convenção da família, não medida).
6. O cabeçalho do [`handoffs/README.md`](README.md) diz «90 handoffs»: o número real conta-se (113 ficheiros `HANDOFF_*.md` a 16/10; a tabela tinha 80 linhas). Nenhum gate o vê.
7. Os abertos 2–8 do [LÍNGUA](HANDOFF_CONTINUACAO_line_motion_value_2026-10-15_LINGUA.md#aberto-ordem) continuam.
8. **NOVOS desta onda:**
   - (a) A entrega por porta lateral está GUARDADA, não resolvida. A cura de verdade é uma entrega por (nó, porta): `boundary_streams` do eval-motion + o upload do gpu-cook chaveados por nó. Só vale quando uma cena a pedir (censo `sonda_os_defeitos_latentes_da_entrega`: zero hoje).
   - (b) `value.input` é `Effect::Pure` mas lê um externo que muda por tique: `boundary_is_static` chamaria constante a um `value.input` sem condutor, e um quadro híbrido de vários tiques entregaria a entrada do ÚLTIMO tique a todos os sub-tiques. MEÇA antes de curar.
   - (c) Registar (e, se reincidir, promover) `the_cost_of_depth_is_linear_not_explosive` no `FLAKES_DE_CARGA.md`: tabela + faixa do nextest.
   - (d) Escrever o filtro canónico do `ph2d-gpu-cook --ignored` (o de 250 acima), para o número ser comparável entre fechos.

## Smoke ao dono

Nenhuma foto foi tirada nesta onda. A imagem é a mesma de antes (o gate prova-o); o ganho de velocidade é invisível no tamanho da demo (a CPU é mais rápida a ×1) e só aparece com uma grade maior.

1. Abra o terminal e rode: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && PH2D_GPU_COOK_DEMO=139 cargo run -p ph2d-host-desktop --profile smoke`
2. Quando a janela abrir, vá na ferramenta Motion: a galáxia de estrelas toca por 5 segundos e para.
3. O que tem de acontecer: a galáxia gira e leva um empurrão a cada 1,5 s (a batida), exatamente como antes.
4. Deu errado se: a galáxia gira sem o empurrão periódico (era o defeito que o teste pegou), ou as estrelas somem.
