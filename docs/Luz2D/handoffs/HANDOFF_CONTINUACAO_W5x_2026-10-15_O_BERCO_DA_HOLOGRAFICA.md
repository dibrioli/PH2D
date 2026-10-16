# Handoff de continuação — Luz 2D, W5x: o berço da holográfica (2026-10-15)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `0aab2c1cb6`, por cima da W5w ([handoff](HANDOFF_CONTINUACAO_W5w_2026-10-15_FECHO_E_O_REGIME_DA_C2.md)), W5v
> ([handoff](HANDOFF_CONTINUACAO_W5v_2026-10-15_O_FUNDO_DOS_OPACOS.md)), W5u
> ([handoff](HANDOFF_CONTINUACAO_W5u_2026-10-15_O_CUSTO_POR_FIO.md)), W5t
> ([handoff](HANDOFF_CONTINUACAO_W5t_2026-10-15_A_SUBIDA_PAGA_POR_FIO.md)), W5s
> ([handoff](HANDOFF_CONTINUACAO_W5s_2026-10-14_A_TRACA_SALTA_O_VAZIO.md)), W5r
> ([handoff](HANDOFF_CONTINUACAO_W5r_2026-10-14_HIGH_MAIS_LEVE.md)) e W5q
> ([handoff](HANDOFF_CONTINUACAO_W5q_2026-10-14_CUSTO_DO_QUADRO.md)), todas não integradas. Nada integrado, nada
> enviado. Commits da W5x: `71e32e432a` (feat: berço, `SondaDeMemoria`, instrumento, sondas) · `be44b3481e` (fix:
> `Config::hrc_guarda_os_grandes`). O smoke do dono (OK em 15/10) cobriu W5s–W5w; **a W5x ainda não teve
> smoke** (§7). Placa RTX 5060 Ti 16 GB, driver NVIDIA proprietário, `gpu-allocator` 0.28, `wgpu` 29.0.4,
> `MemoryHints::Performance` (blocos de 128 → 256 MiB). `c2` = ms de GPU do passe holográfico.

## 1. Gate batched do fecho (régua = merge-base `0aab2c1cb6`)

Gate sobre `71e32e432a`: `fmt` verde · `check --workspace --all-targets` (`CARGO_BUILD_WARNINGS=deny`) verde ·
`clippy --workspace --all-targets -D warnings` (`PH2D_SUITE=fecho`) verde · `nextest-impacted`: **11 753 corridos,
11 752 verdes, 1 vermelho real**: `os_tetos_da_placa::a_luz_recua_quando_a_placa_recusa_a_memoria` (4/4 vermelho
isolado, placa só com kwin; «Buffer 'ph2d-render luz desligada ubo' is invalid»). Os dois flakes de relógio da W5w
passaram. `check-standalone-optional` verde · `cargo machete` verde · caps de LOC verdes
(`shell_files_respect_hr18_loc_cap`, `workspace_src_files_under_loc_cap`). Por nome: holográfica 7 verdes / 9
ignorados · `smoke_cena_1` 1 · `smoke_cenas_w4` 6 · `a_nitidez_da_luz` 6 · `o_interior_do_opaco` 2 (3 ign.) ·
`o_miolo_das_formas` 1 · `redimensionar_a_janela_nao_recompila_a_luz` 1.

**Causa do vermelho (medida).** NÃO é o berço (vermelho também com `PH2D_LIGHT2D_HRC_BERCO=0`); com os ficheiros da
W5w, verde. O que o provocava era a `Holografica` GUARDAR clones dos buffers grandes (campo `grandes`): depois da
recusa da vista, o entorno já não nascia (recusa como `ConfigInvalida` «Texture 'luz2d/tex/emissao' is invalid» em
vez de `SemMemoria`) e o ubo do render saía inválido. Com `grandes` vazio: verde. Cura (`be44b3481e`):
`Config::hrc_guarda_os_grandes` (padrão `false`; o app liga-o só com `PH2D_LIGHT2D_MEMORIA=1`; a sonda de teste
liga-o).

**Delta depois do fix:** `check --workspace --all-targets` deny verde · `clippy -p ph2d-light2d-gpu -p
ph2d-app-light2d --all-targets -D warnings` verde · `fmt` verde · holográfica 7 / 9 ign. · `smoke_cena*` 7 verdes ·
`os_tetos_da_placa` 7/7 pelo nextest e 7/7 em série (`--test-threads=1`) ×2 · `a_luz_recua` isolado 3/3.
**Observação (não defeito):** com `cargo test` em PARALELO no mesmo processo, `a_luz_recua` reprova na asserção da
linha 302 («a luz não recuou»): dois testes de placa disputam o orçamento de memória no mesmo processo; o gate
(nextest, um processo por teste) não vê isto.

## 2. O mecanismo (item 1 do §8 da W5w: o regime da c2 do High)

**Instrumento.** `Device::generate_allocator_report` (pública no `wgpu` 29) dá nome (o rótulo do buffer), bloco,
offset e tamanho. Rodada no app (1920 e 2560 High, `HRC_DUAS` 1 e 0, 2 processos cada, `RENASCE=2`): **o SÍTIO
não explica.**

- 2560 `HRC_DUAS=0` é determinística e igual entre processos (21,6 → 23,9 → 11,8 → 17,0 → 12,2 → 20,4 → 12,2 →
  20,6 → 13,5 → 17,5…).
- g0 e g1 têm a MESMA disposição interna em blocos diferentes e ambas são lentas.
- Nenhuma alocação é dedicada à 1920/2560 (o maior buffer, ~73 MiB, é menor que o bloco).
- Todos os buffers `GpuOnly` caem no tipo de memória 1 (o único `DEVICE_LOCAL` não visível ao host).
- O offset dentro do bloco muda muito entre gerações rápidas (processo de calibração: 21 gerações rápidas, 9,0–9,7,
  com disposições diferentes).

**Régua por buffer** (`sonda_regime_da_memoria_na_holografica`): a c2 de cada geração segue EXATAMENTE a soma do
tempo de LER, em sequência, cada buffer grande da holográfica sozinho (Σler).

| caso | c2 rápida / Σler | c2 lenta / Σler |
|---|---|---|
| 1920 duas=1 | 8,70 / 0,43 | — |
| 1920 duas=0 | 9,10 / 0,43 | 12,75 / 0,77 |
| 2560 duas=1 | 12,46 / 0,55 | 19,80 / 1,19 |
| 2560 duas=0 | 18,35 / 1,04 | 24,14 / 1,49 |

Buffers rápidos leem a ~1,9–2,0 TB/s (≈ 4,5× a DRAM de ~448 GB/s); os lentos a ~420 GB/s (a DRAM). O acumulado lê
sempre ~420 (é escrito inteiro a cada quadro).

**Sonda isolada** (`sonda_regime_da_memoria_isolada`):

| buffer | banda | leitura |
|---|---|---|
| soltos de 48 MiB: frescos, reciclados (nascem/largam/nascem), com e sem `COPY_SRC` | ~610–700 GB/s em tudo; zero = ruído | sem compressão |
| dedicado de 320 MiB | ~434 | só tamanho (os de 48 MiB ainda acertam parte do L2 de 32 MB) |
| HERDADOS: nascem onde texturas morreram (um buffer de 1 MiB segura o bloco) | ler zero ~2 000; ruído ~608; a compressão FICA (zero por shader de novo ~2 000; clear depois do ruído ~2 000) | vale para textura de render limpa, render crua nunca usada, e armazém (`STORAGE_BINDING\|TEXTURE_BINDING`) crua |
| vizinhos de uma textura viva | os do mesmo bloco comprimidos (mas o bloco já tivera texturas); os de bloco novo não | «bloco inteiro × só as páginas da textura» NÃO decidido |

Um «reciclado» caiu por acaso em página comprimível (1 996).

**Conclusão.** O driver NVIDIA deixa COMPRIMÍVEIS as páginas onde uma textura foi ligada. Os buffers dos níveis T/R
da holográfica são quase todos zeros, e nessas páginas os zeros não tocam na DRAM. Os degraus da c2 = a fração dos
bytes da holográfica que caem em páginas ex-textura. Explica o «nascer 2× às vezes cura» (W5p: recicla buracos que
às vezes foram de texturas) e o «buffer único sempre lento» (alocação nova, dedicada, que nunca teve textura).

## 3. A cura (commit `71e32e432a`)

- `Config::hrc_berco` (padrão `true`; `PH2D_LIGHT2D_HRC_BERCO=0` no app compara; vista e entorno). Com
  `hrc_nasce_duas_vezes`, o 1.º nascimento dos buffers grandes (T, R, acumulado) é em TEXTURAS cruas `Rgba32Uint`,
  `STORAGE_BINDING|TEXTURE_BINDING`, largura = `max_texture_dimension_2d`, altura = `ceil(bytes/16/largura)` (`None`
  se não cabe: o buffer nasce onde calhar). Largam-se, `poll`, e os buffers nascem nos buracos
  (`solver_hrc.rs`, `fn berco`). **Bits inalterados** (só a alocação muda).
- `SondaDeMemoria` (`ph2d-light2d-gpu/src/sonda_memoria.rs`, `doc(hidden)`, `Medida::{Ler,Saltar,EscreverZero,
  EscreverRuido}`); `Solver::buffers_da_holografica` (`doc(hidden)`); `Holografica.grandes` guarda clones dos buffers.
- `Config::hrc_guarda_os_grandes` (padrão `false`; `be44b3481e`): só então a `Holografica` guarda os clones
  (`grandes`). O app liga-o com `PH2D_LIGHT2D_MEMORIA=1`; a sonda de teste liga-o. Ligado de graça, partia a recusa
  da vista (§1).
- Instrumento: «[luz2d] memória g<g> · Σler <ms> · GB/s: t0 … acumulado …» (a banda de cada buffer grande da vista)
  só com `PH2D_LIGHT2D_CUSTO=1` **e** `PH2D_LIGHT2D_MEMORIA=1` (o `CUSTO` sozinho mede o produto, sem clones). O relatório de sítios do alocador que existiu durante a investigação SAIU
  (não decide e misturava vista e entorno, que partilham rótulos).

## 4. Números do app

Roteiro `docs/Luz2D/ferramentas/mede_custo_no_app.sh`, cena 5, `RENASCE=2`; c2 = mínimo da 2.ª janela de cada
geração (ms).

**Referência: o PRODUTO** (sem clones, placa livre, 1 processo por caso, High, `RENASCE=2`; c2 por geração, ms):

| caso | gerações |
|---|---|
| 1920 sem berço | 15,3 → 9,6 → 8,9 … com um salto 12,3 na 5.ª, depois 8,8 |
| 1920 com berço | 8,7 8,7 9,1 8,7 8,9 8,9 9,4 8,9 … 9,0 (máx 9,38) |
| 2560 sem berço | 16,9 → 12,8 → 11,4–11,7 |
| 2560 com berço | 11,7 11,9 11,5 12,3 12,3 12,0 12,1 11,6 (máx 12,35) |

Rodada anterior, **com os clones do instrumento** (placa livre, só kwin), High, 2 processos por caso:

| caso | 1.ª montagem | 2.ª | depois |
|---|---|---|---|
| 1920 sem berço | 15,7 / 15,5 | 9,5 / 9,5 | 8,8–9,6 |
| 1920 com berço | 9,3 / 9,7 | 9,9 / 10,2 | 8,8–9,9 (p2 oscila 8,8–9,8) |
| 2560 sem berço | 17,2 / 17,0 | 12,8 / 12,6 | 11,5–11,6 |
| 2560 com berço | 11,6 / 13,2 | 13,2 / 13,1 | 11,5–12,2 |

**Medida por buffer no app** (placa com apps de outras linhas). Sem berço, as montagens lentas são exatamente as
com buffers fora da compressão: 1920 g0 15,1 com Σler 0,80 (t1, t2, t10, t11, t12 lentos); 2560 g0 16,4 com Σler
0,82 (t12, r0, r1); 2560 g1 13,4 com 0,73 (t11, t12); outro processo g0 16,5 com 0,74 (r0, r1). Com berço, TODAS as
gerações de todos os processos com todos os buffers comprimidos (Σler 0,43 no 1920, 0,54–0,56 no 2560).

**Low e Medium** (placa partilhada, 1920/2560 × berço 1/0 × 2 processos): sem berço a 1.ª montagem fica ~0,2–0,5
acima do piso; com berço, piso desde a 1.ª (Low c2 1,2 / 1,4; Medium 1,9 / 2,3).

**Régua de teste** (sonda na holográfica, `App::novo` por geração): 2560 com berço 11,92 em todas as 12 gerações nos
2 processos; sem berço 19,8. 1920: 8,7 nos dois. Modo «como o app» (`PH2D_W5X_COMO_O_APP=1`: só a vista renasce,
com quadros em voo): com e sem berço iguais e determinísticos, 2560 12,0–12,7, 1920 8,8–9,4; todos os buffers
comprimidos (só o acumulado ~420).

## 5. ABERTO, por ordem

1. **O resto nas primeiras montagens do High.** Com os clones do instrumento sobrava +1 a +1,4 ms; no PRODUTO (sem
   clones) caiu para ≤ +0,5 ms no 1920 (máx 9,38 contra o piso 8,7–8,9) e ≤ +0,8 no 2560 (máx 12,35 contra 11,5).
   Esse resto já não é a compressão dos buffers da holográfica (na rodada com a medida, todos comprimidos).
   Candidatos NÃO medidos: outros buffers do solver (c3, miolo, ladrilhos) fora de páginas comprimíveis, aquecimento
   do driver, contenção. Próximo passo: estender a medida (`PH2D_LIGHT2D_MEMORIA`) aos outros buffers grandes do
   solver.
2. «Bloco inteiro × páginas da textura» (§2) não decidido: só importa se o berço falhar noutra placa.
3. Itens 2–4 do §8 da [W5w](HANDOFF_CONTINUACAO_W5w_2026-10-15_FECHO_E_O_REGIME_DA_C2.md): mais fina no Low, a
   cheia por DRAM, os dois portões de relógio sob placa partilhada.

## 6. Para o integrador

Nada de contrato congelado, sem schema, sem registo. Dois campos públicos novos na `Config`: `hrc_berco` (padrão `true`) e
`hrc_guarda_os_grandes` (padrão `false`): quem constrói `Config` por literal exaustivo precisa dos campos (procure com `grep`; os testes usam `..`). Módulo novo
`sonda_memoria.rs` e reexport `doc(hidden)` de `SondaDeMemoria`/`Medida`. LOC (teto 700): `quadro.rs` 677 ·
`solver_hrc.rs` 646 · `solver.rs` 667 · `config.rs` 674. A sonda de teste mudou de crate durante a investigação:
vive em `crates/ph2d-app-light2d/tests/it/sonda_regime_da_memoria.rs`. CLAUDE.md §5.1 Luz 2D: trocar só o link de
«Último:» para este handoff.

## 7. Smoke do dono

Binário do smoke (HEAD `be44b3481e`, `target/*/incremental` reclamado antes), 2.ª corrida:
```
▸ linha line_luz2d · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.22s
```
Régua do loop (`agent-loop-profile.sh`, 20 sessões): paralelismo 1,15 (✗) · respostas/sessão 156 (✓) · test:check
2,2× (✗) · edições sem conferir 1,1 % (✓) · contexto por passo 286 mil (✗) · contexto inicial 65 mil (✓).

Passos (prefs dele hoje: `light_quality=low`, `light_auto=0`):

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
2. Na Hierarchy, clicar «Lâmpada»; no Inspector, trocar Quality para High (manual). Ler «NN ms of GPU per frame».
3. Esperado: AOC 1920×1080 maximizado ~14–16 no High já ao abrir (antes podia abrir em ~21); LG 2560×1080 ~18–20
   (antes podia abrir em ~25). Fechar e abrir 3×: o número deve repetir-se, não saltar ~6 ms.
4. Deu errado se: a imagem muda em relação à W5w, a luz demora visivelmente mais a acender, ou o High ao abrir
   volta a dar ~21 no AOC. Comparar com `PH2D_LIGHT2D_HRC_BERCO=0` à frente do comando (o comportamento antigo).

## 8. ⛔ Recusas MEDIDAS

Código retirado; não reconstruir sem ler isto.

| Recusa | Medida |
|---|---|
| O SÍTIO do alocador (bloco/offset), o tipo de memória e dedicada × subalocada como causa do regime | Refutados (§2): a mesma disposição lenta em dois blocos; disposições muito diferentes todas rápidas; um só tipo `DEVICE_LOCAL`. |
| Compressão por usos do buffer (com/sem `COPY_SRC`) | Neutro. |
| Escrever zeros por shader antes de usar em buffers frescos | Não comprime (~610 GB/s). |
| A vista nascer 2× ou 3× (monta, larga, poll, monta) | Não cura: 1920 nasce 2×: 15,6 / 9,7 / 9,7; 3×: 13,6 / 8,9 / 13,5. (Sem o berço; a W5x explica porquê: §2.) |
| Buffers grandes da holográfica (acumulado, T, R) num buffer SÓ com as fatias seguidas, a 2 MiB ou com desvio de 4 KiB × k | Determinístico mas SEMPRE o regime mais lento: 19,9–21,0 (1920) e 25,9–27,7 (2560); a distância entre fatias não muda nada ⇒ «partition camping» refutado. Exigiu declarar `t_a`/`t_b`/`r_cima` `read_write` no shader e no layout (o wgpu vê o uso por buffer inteiro e exige o mesmo acesso shader/layout). Bits da holográfica 7/7. |
| `read_write` sozinho, com os buffers separados | Neutro (= hoje). |
| As da W5v/W5u/W5t/W5s/W5r/W5q | Continuam: ver os [W5v](HANDOFF_CONTINUACAO_W5v_2026-10-15_O_FUNDO_DOS_OPACOS.md), [W5u](HANDOFF_CONTINUACAO_W5u_2026-10-15_O_CUSTO_POR_FIO.md), [W5t](HANDOFF_CONTINUACAO_W5t_2026-10-15_A_SUBIDA_PAGA_POR_FIO.md), [W5s](HANDOFF_CONTINUACAO_W5s_2026-10-14_A_TRACA_SALTA_O_VAZIO.md), [W5r](HANDOFF_CONTINUACAO_W5r_2026-10-14_HIGH_MAIS_LEVE.md), [W5q](HANDOFF_CONTINUACAO_W5q_2026-10-14_CUSTO_DO_QUADRO.md). |
