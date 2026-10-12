# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, a 2.ª onda da VELOCIDADE (2026-10-11, prompt 20)

## As regras (R1–R9, [PROMPT_LINHAS_RAPIDAS §2](../../archive/integracao-jornadas/PROMPT_LINHAS_RAPIDAS_2026-10-11.md))

- R1 janela curta: ~150 chamadas ou a onda fechada ⇒ commit, handoff ≤ 8 KB, PARE.
- R2 edite pelo `Edit` (novo = `Write`); script só para a MESMA troca em N ficheiros, com `assert` de contagem.
- R3 edição e teste nunca no mesmo comando: `Edit` → `cargo-check-narrow.sh` (⚠️ cego a `tests/it`: use
  `cargo check -p ph2d-tool-painter --test it`; e para os testes da lib `--lib --tests`) → teste.
- R4 no laço, teste por NOME; a suíte e o `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- R5 nada de `until … sleep`: > 2 min = `run_in_background`. R6 chamadas independentes na MESMA mensagem.
- R7 doc > 40 KB lê-se por `grep -n` + `Read` com offset. R8 escreva menos.
- R9 nenhum teste da suíte normal passa de 10 s em `dev`. Cura: (a) `opt-level = 2`; (b) fixture menor que
  ainda morda (prove com mutação); (c) só `diag_*`/`sonda_*`/`measure_*` vão para `#[ignore]`. ⛔ Lei nunca.
- Comando pesado por `bash scripts/ph2d-run.sh …`. Fish/zsh: listas de caminhos num `bash -c '…'`.

## §1 — Coordenadas

Base `ed59256e1` (o prompt 20; `main` = `8f20c60de`, não andou). Um commit desta janela (testes + nextest + docs).
Sem push.

## §2 — ⚠️ `paper_rows`: NÃO é corrida nem carga — é CÓDIGO ERRADO GERADO no perfil `ci-test` (ABERTO)

Primeiro em laço com carga (`yes`; sem `stress-ng`), binário `dev`: o2 0/1 200 e o0 0/1 200 a load ≤ 109. Depois
o `nextest-impacted` do fecho voltou a reprovar no MESMO índice (17545/17641) — e ele corre `--cargo-profile
ci-test`. O binário `target/ci-test/…/it-*` reprova **100/100, determinístico**. A rota PARALELA é a errada:
7 403 células de 373 464, em TODAS as 530 linhas (~14 por linha), nenhuma com o papel do construtor (escreve, mas
o texel errado); a 1.ª `(1, 1)` = 0,5268 contra o tile 0,6995. A série bate com o oráculo.

Matriz (cada uma num `CARGO_TARGET_DIR` próprio na scratchpad; ✗ = as mesmas 7 403 células):

| variante | | variante | |
|---|---|---|---|
| `ci-test` | ✗ | `ci-test`, `ph2d-wet-paint` o0 / o1 | ✓ / ✓ |
| `ci-test`, `ph2d-wet-paint` o3 | ✗ | `ci-test`, `rayon` + `rayon-core` o0 | ✗ |
| `ci-test`, deps `"*"` o0 | ✗ | `ci-test`, `codegen-units = 256` · `debug = true` | ✗ · ✗ |
| `ci-test`, `debug-assertions = false` | ✗ | **`ci-test`, `overflow-checks = false`** | **✓** |
| **`ci-test`, `lto = false`** (thin local) | **✓** | **`dev`, `lto = "off"`** | **✗** |
| `dev` · `dev` o1 · `dev` deps o3 · `release` | ✓ | | |

⇒ a condição é **`ph2d-wet-paint` ≥ o2 + `lto = "off"` + `overflow-checks`**. Nenhum overflow acontece (seria um
pânico, não um valor), logo ligar a verificação não pode mudar um resultado correcto: é **mau código gerado**
(rustc `1.98.0 88d9e12ae`). O corpo é seguro (`forbid(unsafe_code)`); o genérico do `rayon`
(`par_chunks_mut().enumerate()`) é monomorfizado DENTRO desta crate, por isso o `rayon` a o0 não muda nada.
⚠️ **Quem cai:** só o `ci-test` (`ship.sh`, `nextest-impacted`). `release`/`smoke` não têm `overflow-checks` →
o produto e o smoke do dono não. ⚠️ **Exposto por esta linha:** no `main` a crate está a o1 no `ci-test` (passa); o
`[profile.dev.package.ph2d-wet-paint] opt-level = 2` do prompt 19 é herdado pelo `ci-test`.

**Próxima janela, PRIMEIRO:** reduzir (copiar `bake_paper_rows` + `walk_rows` para um teste mínimo e cortar
até ao menor que ainda difere; olhar o `--emit=llvm-ir`/asm do fecho paralelo nas duas configurações), procurar
o issue no rust-lang/rust, e decidir a cura no mecanismo (outra forma do laço que não gere o código mau, ou o
toolchain). ⛔ Não silenciar com `[profile.ci-test.package.ph2d-wet-paint] opt-level = 1` sem dizer que é
contorno de compilador — e há outros `walk_rows` no motor (`parallel_rows::*` passam no `ci-test`, mas são
portões de OUTROS fechos). Reproduz em 0,03 s:
`cargo test -p ph2d-wet-paint --test it --profile ci-test -- --exact paper_rows::the_paper_bake_by_rows_is_the_serial_bake`.

Feito: o portão compara CADA rota com o oráculo e diz onde; a lei é a mesma.

## §3 — R9: antes × depois (sozinho, mínimo de 2, A/B intercalado com o binário do HEAD; load ao lado)

Cura única: as cenas de cada teste são independentes e corriam em série; agora uma por thread
(`std::thread::scope`), as mesmas contas, as asserções na mesma ordem — nenhuma imagem muda (sem mutação nova).

| teste | sob a suíte antes | sozinho antes | sozinho depois | load |
|---|---|---|---|---|
| `a_separacao_das_pocas…` (195; Aquarela, isenção do dono) | 151 s | 32,9 s | **7,3 s** | 15–21 |
| `o_escorrimento_do_s_nao_se_corta_em_rectangulos` (6) | 138 s | 19,8 s | **7,6 s** | 17–21 |
| `a_chave_muda_com_cada_parametro_e_o_traco_tambem` (12) | 111 s | 28,6 s | **7,7 s** | 12–15 |
| `a_heavy_edge_a_meio_pesa_na_orla…` (8) | 90 s | 17,3 s | **6,0 s** | 13–14 |
| `com_as_opcoes_desligadas_…_ao_byte` (6) | 89 s | 14,3 s | **5,4 s** | 13–14 |
| `sem_celulas_soltas` (3) | 65 s | 19,9 s | **7,7 s** | 11–12 |
| `a_borda_de_um_traco_sobre_tinta_tambem_amacia` (4) | 38 s | 7,0 s | **3,8 s** | 6–9 |
| `o_preview_mostra_o_dente_do_papel` · `…papel_de_cor` · `…smear_ativo` (2 cada) | 20–22 s | 5,6 s | **3,0 s** | 7–8 |

`.config/nextest.toml`: o `sem_celulas_soltas` sai do override de 6 × 60 s (no mesmo commit).
Suíte do `ph2d-tool-painter`: 173 → 161 s (§6).

## §4 — Ficou acima de 10 s (sozinhos, load 16–22) — e porquê

- **Camadas 44,7 s:** já paralelo; a parede é a «cena do dono» (Live Water, 10 passadas), sequencial. Próximo:
  medir por passada e se uma tela menor dá o MESMO miolo ao byte.
- **Largura 24,7 s:** 104 cenas já paralelas. Sonda (raio 300): Wet Round 0,2 s (q0) → 2,55 s (q480) e a largura
  anda até ao fim (599,5 · 583 · 590 · 594 · 597,5 a q0/60/120/240/480) ⇒ menos quadros muda a régua; encurtar o
  traço também (mediana das colunas, capuzes de 300 px). A parede é CPU total.
- **Mídias 14,8 s** (o anterior dizia 41–46, a load mais alta). Sonda no `wet_step_sync` (revertida): o fluido
  acaba ~250 quadros depois do traço e daí cada quadro custa 0 ms ⇒ o «ponto fixo» não tem o que ganhar; o custo é
  a água a correr (Wet Liner 3,0 s/cena, os outros ~0,7 s).

## §5 — ⛔ Recusas MEDIDAS (não refaça; somam às do handoff anterior §5)

| proposta | medida | veredicto |
|---|---|---|
| parar a cena das mídias no «ponto fixo» | os quadros depois de secar já custam 0 ms | nada a ganhar |
| largura com menos quadros depois do traço | Wet Round 583 → 597,5 px de q60 a q480 | muda a régua |
| `paper_rows` como corrida de carga | 0 / 2 400 no `dev` a load ≤ 109; 100/100 no `ci-test` sozinho | é o perfil (§2) |

## §6 — Fecho

Bateria (agente `verificador`, em série): cerca ✓ (as poças: isenção do dono 11/10, «pode fazer as mudanças» — `cerca_isencoes.tsv`, 20 linhas; a regra
no `cerca.sh` ganha a classe «só a forma de correr um teste de outro meio»; `os_outros_meios_nao_mudam` ✓) · `ph2d-wet-paint`
148/148 · `ph2d-tool-painter` 1574/1574 em **161 s** (antes 173 s; load 10→58, outras sessões) · `paper_rows` em
laço `dev` 300/300 a load 62–67 · censos 134 · 237 · 17 ✓ · `ph2d-app-painter` 103/103 · fmt ✓ · clippy
`-D warnings` ✓ · tecto 700 ✓ (403 · 195 · 568 · 561 · 293 · 116) · `nextest-impacted` 17 640/17 641 em 311 s —
a única falha é o `paper_rows` (§2). Mutações: nenhuma nova (as curas não mudam uma imagem — só a forma de correr).

## §7 — O que NÃO foi tocado (a fila espera; §8 do handoff PRESETS_MIDIAS)

O relevo que some ao trocar de pincel · o óleo longe do Rebelle · a tinta/caneta (dono) · o Staining com K–M.
