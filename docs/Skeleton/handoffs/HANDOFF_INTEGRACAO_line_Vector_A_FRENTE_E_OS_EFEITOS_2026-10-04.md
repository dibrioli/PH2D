# HANDOFF de INTEGRAÇÃO — `line/Vector`: a imagem COSIDA, os EFEITOS cozidos no Bind e a FRENTE que tapa a de trás (F48–F59, 2026-10-04)

> **Para o agente INTEGRADOR, noutra janela, por ordem do dono** (*«antes de seguir vamos escrever
> handoff para integrar ao main»*). Leia-o inteiro antes do primeiro comando. ⚠️ Descreve **só** os
> commits desde `1ad60a1ce`; a rodada anterior já está no `main` e o documento dela é o
> [handoff de 2026-10-01](HANDOFF_INTEGRACAO_line_Vector_A_SILHUETA_DA_PELE_2026-10-01.md). O
> mecanismo de cada wave, com as tabelas e as recusas, vive na [fila](../01_a_fila.md) §F48–§F59.

## 0. Onde está e o que fazer

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| ramo | `line/Vector` |
| base | **`1ad60a1ce`** = o `main` de quando este doc foi escrito (`behind 0`) ⇒ **nenhum rebase foi preciso** |
| forma | **fast-forward** enquanto o `main` não andar |
| commits | **113** = 112 (produto, gates, sondas, docs, duas tentativas revertidas no próprio ramo) + o deste handoff |
| ⚠️ `CARGO_TARGET_DIR` | a worktree usa o `target/` DELA. ⛔⛔ Nunca partilhe o target entre worktrees (troca os `.rlib`) |

**Passos** (DIRETRIZ §1.5.3; `/pd-integracao`): (1) no primário, `git status` — `project-memory/` de
outras sessões não entra; esta linha **não toca** em `project-memory/`. (2) Se o `main` andou: `git -C
Worktrees/line-Vector rebase main` e reconte com `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`
(caminho ABSOLUTO; a coluna `base:` é o merge-base). (3) `bash scripts/foundational-integrate.sh
line/Vector` e `bash scripts/censos-da-arvore-combinada.sh`. (4) `git merge --ff-only line/Vector`.
**Ship/push só por ordem explícita do dono.** (5) Uma linha do `CLAUDE.md` §5 — texto no §7.

## 1. A superfície de colisão (`collision-surface.sh`, 2026-10-04)

```
merge-base 1ad60a1ce · 112 commit(s) · 92 arquivo(s)
PROJECT_SCHEMA 178 (base: 178) · tripla (178, 13, 22) · VEC_SCENE_SCHEMA 22 (base: 22)
FLIP_SCHEMA 13 · DOC_VERSION 18 · FIELD_DOC_VERSION 23              — todos = base
ph2d-ecs 108 · ph2d-render (espelho) 109 · ph2d-script (espelho) 109 — todos = base
crates/ph2d-nodegraph/src/node.rs intocado · crates/ph2d-editor-core/src/tool.rs intocado
ADR: esta linha não cria ADR (próximo livre no disco: 0176) · Cargo.lock: nenhum '+name' novo
marcadores de conflito: nenhum · tectos de LOC nos ficheiros tocados: nenhum passa
```

⇒ **Zero contador partilhado, zero contrato congelado, zero ADR, zero pacote externo, zero schema.**
⚠️ A F51 chegou a subir `VEC_SCENE 23 / PROJECT_SCHEMA 179` (o botão «antes/depois dos ossos») e foi
**revertida** pelo dono no mesmo dia (`fd7b0750b`): o saldo é zero — confira que nenhuma linha
paralela «herda» o 179.

## 2. Foundational / partilhado tocado — e porque é seguro

| ficheiro | o que muda | porque é seguro / o que vigiar |
|---|---|---|
| `ph2d-vec-skin/src/pesos.rs` + **novo** `pesos_aneis.rs` | o domínio do campo segue a **regra de preenchimento** da forma (F55; antes era par-ímpar sempre) e um índice dos anéis por faixa/célula (F53) | `EvenOdd` e formas sem sobreposição: ao bit (gate `o_dominio_segue_a_regra_de_preenchimento_da_forma`, `o_indice_dos_aneis_responde_como_a_varredura`). ⚠️ O campo de binds NOVOS de formas com contornos sobrepostos `NonZero` muda (antes eram furo sem peso) |
| `ph2d-vec-skin/src/curva_segundo_corpo.rs` | `fecha`: o ajuste só aceita a cúbica que anda no sentido da fonte (F50-e) | só o bake da pele lê; curou também o gancho da F43 |
| `ph2d-vec-boolean/src/{engine,overlap,lib}.rs` | `catch_unwind` na porta única do motor (o `linesweeper` 0.4 rebentava num *Repeater* denso), `overlaps_itself` (a pergunta sem correr a união), lascas `1e-4` descartadas | uma união que dava pânico passa a responder `None` (gate `a_dense_spinning_repeater_union_answers_instead_of_panicking`) |
| `ph2d-vec-scene/src/fx_warp.rs` | o *Bloat* deixa um segmento de comprimento ZERO com comprimento zero (F50-g) | sem ossos: a agulha na tampa de uma cápsula some; o resto ao bit (gate `a_capsula_com_pucker_nao_ganha_agulha_na_tampa`) |
| `ph2d-panel-vector` (`paint_effects`, `state*`) + `ph2d-i18n` (`panel.vector.fx.bound`) | uma forma PRESA não recebe efeitos: a secção diz só *«Bound to bones: effects are baked into the drawing»* (F51) | chave i18n NOVA (en) — conte-a no censo do i18n da árvore combinada |
| `shells/desktop` (+7 −2 linhas) | o despacho de efeitos RECUSA a edição numa forma presa (`fx_bridge::is_bound`); `fase_vector_view_and_drives` chama `skeleton_live::coze_os_efeitos_presos` antes da pele (A4/F54) | ⚠️ a catraca `the_shell_only_shrinks` soma entre linhas: **+5 líquidas** desta linha |
| `ph2d-editor-core/tests/it/architecture_who_reads_the_posed_skin_mesh.rs` | o MOTOR do censo ganha `skin_image_fecho.rs` (`[&str; 10]` → `11`) | só gate |
| `ph2d-app-skeleton` (2 testes) | `bind(&mut sim, &mut cena, …)` | ver §4 |

As crates do módulo (`ph2d-skeleton-live`, as cenas `ph2d-app-vec/src/smoke_bone*.rs`) são **da linha**.

## 3. O que a linha faz — por wave, com os números MEDIDOS

- **F48→F49 — a imagem presa COSE o fio** entre membros que a corrente encosta (`skin_image_fecho::costura`):
  vãos `< 2` texels entre partes a `> 1,25` osso; a bola da F48 foi RECUSADA pelo smoke (`58 ms`/imagem/quadro,
  «ora redonda ora pontuda»). Custo `0,22`–`0,31 ms` sem nada a coser, `0,70` cosendo. Mutação 12/12.
- **F48-c/F52 — a ordem das faces da imagem**: o osso mais FUNDO na hierarquia por cima (ordem do dono).
  ⛔ A coluna dos pesos vem por `to_bits` (decrescente no `bevy_ecs` 0.19) — a chave é a PROFUNDIDADE
  (`esqueletos::profundidades`). ⚠️ **Visível nas cenas `=3`/`=4`**: o último osso por cima.
- **F50/F51 — formas vetoriais com efeitos**: o **Bind COZE os efeitos** (o *Expand Appearance*) e uma forma
  presa não recebe efeitos (ordem do dono). O campo do domínio é o do contorno COZIDO (o *Twist* deixava de
  rasgar: `17`–`191×` → `≤ 3,6×`); a união só dos fechados e só neutra em repouso; o solver do campo numa
  thread (arrastar um efeito: `59 → 2,5 ms`). Desvio ao padrão-ouro `0,27`–`1,27 → 0,002`.
- **F52 (A2) — a FRENTE tapa as riscas de trás** (`skin_desenho_frente::so_o_que_se_ve`): tapadas pintadas
  `0,000` contra `0,99` sem a lei a `110°`–`150°`. Mutação 12/12.
- **F53 (A3)** — prender um *Repeater* `39²` deixa de parar a tela: malha `743 → 21 ms`, Bind `343 → 64 ms` (release).
- **F54 (A4)** — uma forma presa de projecto antigo com efeitos vivos coze-os no quadro: desvio `0` ao bit.
- **F55 (A6) — sem união, o TRAÇO dos fechados de trás não pinta por cima**: o quadro sai em DUAS camadas
  (`Desenhado { forma, traco }`), o traço cortado do PRÓPRIO assado (`≤ 3·10⁻⁵` da borda até `180°`). Cena `=6`.
- **F56 (A7) — saída rápida do recorte**: sem par de triângulos posados sobrepostos nada se amostra; o recorte de
  uma forma com riscas sem dobra `174`–`216 → 77 µs` (ao bit: gate `0°…150°`).
- **F57 (A9) — a ponta de um corte do traço acerta no CRUZAMENTO desenhado** (a menos de uma largura): na `=6`
  tiques `0,13`–`0,98` largura → `0,00`–`0,02`. ⭐ O memo do quadro é por THREAD — um gate que compara duas leis
  desenha cada lado numa thread nova.
- **F58 (A8)** — fechado sem cura: as janelas tapadas que passam entre amostras ficam `≤ 0,45` da largura (gate).
- **F59 (A5)** — (b) fechar os buracos que a linha cobre: feito e **recusado pelo dono no smoke** (revertido,
  `501daabf4`); (a) a cúspide da imagem: **tentada e revertida** (`ac8246764` → `9e39c48a5`) — melhorava o report
  e regredia `−149,5°`/`−160°` nos gates da F49.

## 4. O que muda para quem chama (outras linhas que prendam formas ou leiam o desenho)

- `skin_live::bind(&mut SimWorld, **&mut VecScene**, …)` — coze os efeitos na cena (era `&VecScene`).
- `skin_desenho::quadro(…, ordem: &[f64])` — a profundidade dos tendões; `Leis` ganhou `frente` (literais
  completos de `Leis` noutras linhas precisam do campo).
- `SkinDesenhado = BTreeMap<_, Desenhado>` (`Desenhado { forma, traco }`, `Deref` para a forma); `Quadro`
  ganhou `traco`; `skin_desenho::funde` põe a camada do traço DEPOIS da forma.
- `skin_image_fecho::ordena_pelo_osso(…, prof)`; `smoke_bone::NIVEIS = 6` (cena `=6`, `smoke_bone_copias`).
- Na base (`git grep` em `1ad60a1ce`, fora das crates da linha) só `skin_live::bind` tinha chamadores de fora:
  `ph2d-app-skeleton/src/{censo_dos_verbos_do_osso,sonda_do_ponto_novo}_tests.rs` e
  `shells/desktop/src/skeleton_live_tests.rs` — os três já atualizados nesta linha. Uma linha PARALELA que prenda
  formas noutro sítio parte no `&mut`: quem prova é o `cargo check --workspace` da árvore combinada.

## 5. Prova de fecho

Gate batched 1× sobre o diff acumulado (2026-10-04, máquina partilhada a `load ~46`–`63` por outras linhas):

| passo | resultado |
|---|---|
| `BASE=1ad60a1ce … nextest-impacted.sh` | **20 442 / 20 443** verdes (264 s). A única falha: `ph2d-timeline::nesting_clock::the_cost_of_depth_is_linear_not_explosive` (`3,13×` contra a barra `2,9`) — ⚠️ **flake de carga CATALOGADO** ([FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md) linha 48 e a faixa do `.config/nextest.toml`); a linha **não toca** em `ph2d-timeline` (`git diff --stat` vazio); sozinho passou **3/3** a `load` `17,95` · `63,67` · `58,65` |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ |
| clippy `--all-targets -D warnings` (`ph2d-skeleton-live`, `ph2d-app-vec`, `ph2d-vec-skin`, `ph2d-vec-boolean`, `ph2d-panel-vector`, `ph2d-vec-scene`, `ph2d-app-skeleton`, `ph2d-i18n`, `ph2d-editor-core`, `ph2d-host-desktop`) | ✓ |
| shell: `file_loc_caps` (4) · `fn_loc_caps` (2) · `arch*` (16) | ✓ 22/22 |
| `ph2d-editor-core --test it architecture` (inclui `the_shell_only_shrinks`) · `arch_safe_clamp_only` | ✓ 102 · ✓ 2/2 |
| `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh` · `cargo fmt --all -- --check` | ✓ · ✓ · ✓ · ✓ (o fmt pendente da linha, A11, foi pago em `eaa53edb1`) |
| máquina | nenhum `cargo`/`rustc`/`nextest` desta worktree vivo; a placa só com processos de outras linhas |

**Provas de mutação** (cada wave, detalhe na fila): F49 12/12 · F50 por sub-wave · F51 11/11 (o botão, retirado) ·
F52 12/12 · F53 5/6 + 1 equivalente · F54 7/7 · F55 6/6 · F56/F57 14 corridas → 6 + os sobreviventes viraram
gate (M8, M11, M13, M14 re-mutados: sangram) ou a lei saiu (M3); M5/M6/M7 equivalentes · F58 a constante
`AMOSTRAS` `32 → 8` sangra.

**Auditoria (2 lentes):** CORREÇÃO — o recorte com e sem a saída rápida sai AO BIT (`0°…150°`, 3 fixturas,
`Debug` do `f64`); o encaixe do A9 medido por régua só da geometria DESENHADA (sem malha nem chave), ao longo do
contorno, com controlo de 6 tiques. WIRING — as duas portas do recorte (`so_o_que_se_ve`, `cortes_dos_fechados`)
chamam a saída (contador `AMOSTRAGENS` = 0 quando ela dispara); `traco_sobre_o_assado` chama o encaixe (gate de
ponta a ponta sobre `recook_leis`); a shell coze antes da pele (gate de costura `a_shell_coze_antes_de_desenhar_a_pele`).
NÃO-CHECADO pela compilação: o aspecto na tela — por isso os smokes do dono (§8).

**O binário do smoke está quente nesta worktree** (2.ª corrida, sem `Compiling`):
```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.45s
```

**`bash scripts/agent-loop-profile.sh`** (20 sessões):
```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                242   alvo: <= 800
  ✗ cargo test : cargo check                738 : 250   alvo: <= 1,0  razao 3.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%
  ✗ contexto relido por passo (media)         487 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```

## 6. ABERTO — com dono (a lista viva: [handoff de continuação](HANDOFF_line_Vector_CONTINUACAO_A5_A9_2026-10-04.md) §1)

- **A5-a** — a cúspide da imagem junto à tampa: tentativa no histórico (`ac8246764`); próximo passo medido = o
  contorno da máscara por *marching squares* em vez da marcha axial (fila §F59).
- **A10** — a ponta do traço na ponta do VINCO passa `0,4`–`3` larguras (a cura de fundo: «tapado» pela pele EXACTA).
- **A12** — reentrâncias abertas mais estreitas que a linha leem-se como manchas escuras (perguntar ao dono).
- Herdados: o efeito ANIMADO numa forma presa paga o solver por quadro (não medido).

## 7. A linha do `CLAUDE.md` §5 (para o integrador escrever)

✅ **Já escrito no ramo** (`CLAUDE.md` §5, entrada **Vector + Esqueleto**, só o link «Último»; gate `architecture_claude_md_cabe_no_orcamento` verde):
`Último: [handoff 04/10](docs/Skeleton/handoffs/HANDOFF_INTEGRACAO_line_Vector_A_FRENTE_E_OS_EFEITOS_2026-10-04.md)`.

## 8. Smoke — o que o dono vê (aprovado por ele em cada wave; o último: *«smoke oK»*, 04/10)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=6 cargo run -p ph2d-host-desktop --profile smoke
```
(1) abre com duas barras de cópias dobradas em S; (2) na **Hierarchy** escolha **Copias bone 2** → **Bones →
Transform** e dobre quase sobre si; (3) em cada dobra o contorno e as riscas da parte de baixo param na borda da
de cima, sem ganchos; (4) errado = linhas a atravessar por cima ou pontinhas para lá da borda.
Outras cenas aprovadas: `=5` (os efeitos dobram com a barra, as riscas de trás tapadas), `=4` (a imagem cosida, o
último osso por cima). Depois da fusão o dono smoka o **main**: o binário dele compila-se lá.
