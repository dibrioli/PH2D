# Handoff de INTEGRAÇÃO — `line/FLIP` → `main` (2026-07-18, 2ª rodada do dia)

> **Para o agente INTEGRADOR.** Ordem do Enio: integrar esta linha ao `main`.
> A linha está **fechada e smokada** (*"SMoke OK"*, 2026-07-18); o implementador parou aqui
> (§0.7 do CLAUDE.md).
>
> **Base:** `Worktrees/line-FLIP`, branch `line/FLIP`, **25 commits à frente** do `main`.
> **`main` NÃO andou** desde o fork (`git rev-list --count HEAD..main` = **0**) ⇒ este é um
> **fast-forward limpo**, sem merge, sem resolução de conflito.
>
> ⚠️ Estes 25 commits incluem os 14 do handoff anterior
> (`HANDOFF_line_FLIP_INTEGRACAO_2026-07-18.md`), que **não chegou a ser integrado**. Este
> documento **supersede** aquele — leia os dois, mas o comando é um só.

---

## 1. O comando (o caminho feliz)

```bash
cd /home/enio/Documentos/Projetos/PH2D           # árvore primária
git status --short                                # tem de estar limpa
git merge --ff-only line/FLIP
```

Se o `--ff-only` recusar, **PARE**: a `main` andou depois desta escrita. Aí vale a
DIRETRIZ §1.5.5 — resolva pelos **ESTÁGIOS do índice** (`:1` base, `:2` ours, `:3` theirs),
nunca pelos marcadores, e rode `cargo check --workspace` depois (merge limpo pode estar
semanticamente quebrado).

**Depois do merge, rode o ship COMPLETO** (`./scripts/ship.sh`). O `nextest-impacted` já teve
false-green em RAM baixa, e esta rodada mexe em crate foundational (`ph2d-flip-fill`).

---

## 2. O que este delta entrega

### 2.1 A saga do balde de tinta (BUGS #19 → #22) — **smoke aprovado**

| # | Commits | O que o usuário ganha |
|---|---|---|
| **#19** | `5e545fb3` | O fill de uma forma que se CRUZA volta a seguir a linha (critério de área → abraço nos dois sentidos). |
| **#20** | `f5e486ad` | A dilatação era **100× grande demais** (unidade px vs mundo) **e** era uma MÉDIA global; virou local e adimensional. |
| **#21** | `d5bf01ff` `1f2392dc` | A margem extra vai a **ZERO** — ela era contagem dupla da compensação por ponto. |
| **#22** | `27cfa20c` `6d2dbedf` | **A dilatação INTEIRA era contagem dupla.** A lei virou `2s` (só o erro de vetorização, com sinal); o termo da espessura da linha morreu. |

**O ALVO VIVO morreu** (`8b480d62`, ordem do Enio): um traço desenhado é um FATO, não um
preview — `flip_live.rs` (177 LOC) foi removido.

### 2.2 A wave REGIÃO POR CURVAS (R0 · R1 · R2 · buracos)

A malha do preenchimento passa a nascer dos **vértices das próprias linhas** — a queixa do
smoke (*"a malha que o fill cria não usa os vertex das linhas… diferente do Draw:Filled"*).

| fatia | commit | o que é |
|---|---|---|
| **R0** | `0aed78c6` | O **arranjo planar** (`ph2d-flip-fill::arrange`): a ordem vem da TOPOLOGIA (vizinha angular no nó), nunca da proximidade — é o que mata o BUGS #16 (*"proximidade não é ordem"*). |
| **R1** | `6adaf367` | O balde **escolhe a rota**, com quatro recusas que protegem o caminho antigo. |
| **R2** | (de graça) | Nesta rota `s = 0` por construção ⇒ **não há nada a dilatar**. |
| **buracos** | `189933d0` | O **donut** atravessa: um buraco é uma **COMPONENTE conexa**, e a caminhada de half-edge nunca a enxerga porque nunca atravessa entre componentes. |

### 2.3 A wave COLORIZE, fatia C1 — **Trap** (`6fe88937`)

O balde para de vazar por vão: flood por **bola de raio `trap_px`** (Zhang et al., TVCG 2009).
`trap = 0` deixa o pipeline **byte a byte** como era.

### 2.4 Cauda

- **`02677857`** — a simplificação do traço deixa de ser inerte (a cerca que a mantinha assim
  virou estrutura: preview e bake passam pela MESMA porta, com arch-gate).
- **`58193d54`** — cena de smoke do balde (`PH2D_FLIP_FILL_SMOKE=1`).

---

## 3. ⚠️ AS QUATRO COISAS QUE VOCÊ PRECISA SABER

### 3.1 **Nenhum bump de schema nesta rodada**

`git diff main..HEAD` sobre `crates/ph2d-flip/src/` e `project.rs` **não toca nenhum
`*_SCHEMA*`**. (O handoff anterior avisava do bump 15→16 do §4.C.6 — ele já está DENTRO
destes 25 commits, então continua valendo como fato do delta, mas não há bump NOVO.)

⚠️ **Se outra linha da jornada também bumpou schema:** o valor certo **não está em nenhum dos
dois lados** — números que SOMAM se CONTAM
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

### 3.2 **Nenhum contrato congelado foi tocado**

Zero diff em `ph2d-vector-doc` / `-traits`, `nodegraph`, superfície de `Tool`. Não há ADR
pendente por causa desta linha.

### 3.3 **`ph2d-flip-fill` é foundational e a API pública MUDOU**

Removidos (o termo que eles serviam morreu no #22):

- `FILL_TUCK_FRACTION`
- `contour_widths_with_margin`
- `mean_line_width`

Acrescentado: **`FillResult.scale`** — a resolução que a grade de fato ENTREGOU.

> ⚠️ **Por que este campo existe, e é o achado mais reutilizável da rodada:** o `hug_tol`
> saía da precisão **PEDIDA** enquanto o erro do contorno nasce da **ENTREGUE** (o
> `Grid::new` capa o `scale` no `MAX_SIDE`). Dois números sobre o mesmo fato, de fontes
> diferentes. Acima de ~3200 px de arte na tela a rota do arranjo **se recusava em
> silêncio** — e era essa a metade *"nenhuma mudança"* do relato do Enio.

Fora do módulo Flip **ninguém consome** essa crate (`git grep ph2d_flip_fill` só bate em
`shells/desktop/src/flip_*` e nos testes dela). O risco de integração é baixo.

### 3.4 **A fatia R3 foi REVOGADA por medição — não a "termine"**

O plano `docs/Flip/10_regiao_por_curvas.md` mandava **aposentar** o `filled_shape_target`.
**Não faça isso.** Medido na MESMA arte (§12 do plano), com só a linha selecionada, o
descolamento chega a **5,8 larguras de linha**:

| gesto | traço próprio | curvas, só a linha selecionada |
|---|---|---|
| Push | 0,0000 | **2,1397** |
| Grab | 0,0000 | **2,3134** |
| Smooth | 0,0000 | **0,8972** |

O mecanismo é o **auto-masking do W6**. A R3 virou **proteger** a propriedade:
`shells/desktop/src/flip_fill_identity_tests.rs`, 5 gates, **3 sangram** quando o ramo é
desligado. O plano já está corrigido no §12 — se você ler a linha da tabela de fatias, ela
está riscada.

---

## 4. Os gates (rode-os; são a prova, não a formalidade)

```bash
cd /home/enio/Documentos/Projetos/PH2D
cargo test -p ph2d-flip-fill                       # 64
cargo test -p ph2d-host-desktop                    # 795 no bin + os de integração
cargo test -p ph2d-flip-render -- --ignored        # 10 oráculos de PIXEL (precisa de GPU)
cargo test -p ph2d-tool-flip -p ph2d-flip -p ph2d-flip-reshape
```

Todos verdes nesta escrita. Clippy `--all-targets` limpo nas crates tocadas.

### 4.1 Os gates que nasceram VERMELHOS (e o que cada um impede de voltar)

| gate | onde | impede |
|---|---|---|
| `the_colour_never_passes_the_line_the_artist_can_see` | `gpu_fill_fit.rs` | a franja do #22. **Não escolhe raio**: renderiza a cena DUAS vezes (só a linha, e a linha com a cor) e compara raio a raio. |
| `the_bucket_paints_the_shape_the_way_draw_filled_does` | `gpu_fill_fit.rs` | o balde divergir da referência que o Enio nomeou. |
| `the_hug_tolerance_follows_the_resolution_the_grid_delivered` | shell | a rota do arranjo morrer em silêncio no zoom alto. |
| `a_region_with_an_island_keeps_the_hole_and_rides_the_lines` | shell | o donut regredir — **e o buraco ficar vetorizado** enquanto o anel externo é curado. |
| `flip_fill_identity_tests.rs` (5) | shell | a R3 ser "terminada" por alguém. |

### 4.2 ⚠️ Duas armadilhas de oráculo que esta rodada pagou — **leia antes de mexer nestes gates**

**(a) Onze oráculos de pixel ficaram VERDES durante quatro rodadas com o defeito na tela.**
Não era barra frouxa (engordar o fill 25% mata 3 gates de unidade). Eram três cegueiras
independentes: a fixture usava **linha opaca** (o único ponto onde o defeito é identicamente
zero); a janela de medição começava **2 px além do raio geométrico**, que é o mesmo `w/2` que a
dilatação perseguia (o bug não conseguia entrar na janela **por construção**); e **todas as 11
fixtures usavam UM traço fechado** — a topologia em que o produto vai pela rota que não dilata,
onde `contour_widths` **nunca é chamada**.

**(b) O gate de PRESENÇA reprovava a referência aprovada.**
`a_soft_line_never_shows_the_background_through_the_fill_edge` exigia que nenhum pixel do anel
`[eixo, silhueta]` fosse fundo. Medido: o **Draw:Filled** deixa 2956 pixels ali numa linha macia
de 32 px — e a lei nova deixa os **mesmos 2956**. O gate descrevia o modelo de quem o escreveu.
Foi reescrito para medir o lado de DENTRO (a cor ficar **aquém** do eixo, o defeito real do #15).

---

## 5. Mutações provadas nesta rodada

| mutação | resultado |
|---|---|
| ressuscitar o termo `w` da dilatação | mata os **2** gates novos de pixel e **NENHUM** dos outros 9 — que é exatamente por que o bug viveu cinco smokes |
| matar a compensação `2s` | mata o gate de cobertura por zoom |
| `hug_tol` de volta à precisão PEDIDA | 0 vértices sobre as linhas (rota morta) |
| o arranjo voltar a não produzir buracos | o buraco vira vetorizado (0 vértices) |
| **desligar o `filled_shape_target` (= o R3 do plano)** | 3 dos 5 gates de identidade sangram |

**Duas sobreviventes, ambas ACHADO e não gate faltando** (documentadas no código):

- o filtro de área zero em `holes_of` era **código morto** — o `a >= 0.0` uma linha acima já
  rejeita a face degenerada do rabisco aberto (área **exatamente** 0.0). Removido.
- o filtro de COMPONENTE sobrevive **por construção**: a silhueta de uma componente envolve
  todas as faces dela e todo vértice dela está SOBRE uma aresta que as limita ⇒ nenhuma entrada
  distingue tê-lo ou não. Fica porque sem ele a corretude penderia de um ponto-em-polígono
  avaliado exatamente sobre o anel, que é **indefinido**.

---

## 6. Riscos conhecidos, medidos e NÃO consertados

| risco | número | por que fica |
|---|---|---|
| O arranjo é **O(segmentos²)** | **80,8 ms** com 200 traços (5800 segmentos); critério de morte do §8 = 100 ms | passa com pouca folga. O caminho (broadphase por grade) é conhecido e não foi preciso. Sonda: `crates/ph2d-flip-fill/tests/probe_arrange_perf.rs` |
| Resíduo do contorno vetorizado vs Draw:Filled | até **314 px** (linha de 8 px, dureza 0,20) | é o erro de VETORIZAÇÃO, não de lei. A rota do arranjo o leva a zero; o gate mede a rota que sobra quando aquela recusa. Barra em 400 (27% acima do pior observado, **30×** abaixo do que a lei antiga produzia) |
| **Grow ≠ 0** e **Trap armado** derrubam a rota do arranjo | — | recusa deliberada: ela põe a fronteira no eixo e não sabe deslocar. **Aceitar seria ignorar um slider em silêncio.** |
| Perf do Trap: a EDT é 67% do custo | — | `rayon` está **BARRADO por ADR-0109**. Alavancas single-thread esgotadas, tabela pronta. **Exige ordem do Enio + ADR novo** |

---

## 7. Sondas que ficam no repo (diagnóstico documentado, não lixo)

| arquivo | o que decide |
|---|---|
| `crates/ph2d-flip-render/tests/probe_bucket_vs_draw_filled.rs` | a varredura que **escolheu a lei** (`w+2s` vs `2s` vs `zero`), contra a referência aprovada |
| `crates/ph2d-flip-render/tests/probe_halo_under_soft_line.rs` | a medição que provou que o gate de presença **reprovava o Draw:Filled** |
| `crates/ph2d-flip-fill/tests/probe_arrange_perf.rs` | a perf do arranjo contra o critério de morte do §8 |
| `crates/ph2d-flip-fill/tests/probe_offsets.rs` | o erro de vetorização é **de um lado só** |

Rodam com `-- --ignored --nocapture`.

---

## 8. Smoke (para reconferir depois da integração)

```bash
cd /home/enio/Documentos/Projetos/PH2D && PH2D_FLIP_FILL_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

Cena montada: moldura de **quatro traços** com uma **ilha** dentro, e uma forma fechada
sozinha à direita. Pincel **MACIO** (0,35) e arte **trêmula** — os dois de propósito: com
pincel duro a franja é identicamente zero, e em reta perfeita as duas rotas empatam. O roteiro
sai impresso no terminal.

---

## 9. O que fica ABERTO (para a próxima rodada, não para o integrador)

1. **Higiene do `docs/Flip/01_plano_waves.md`** — cabeçalho de 2026-07-12, **não menciona 8
   waves que landaram depois**, e lista **6 itens como abertos que EXISTEM no código** (gizmo
   de seleção, domínio Point, segment mode, multi-seleção na tira, modo Selected dos fantasmas,
   instância na UI). Ele chega a se contradizer. **É o modo de falha que o módulo de áudio já
   pagou: uma lista velha faz a próxima LLM construir o que existe** — e nesta sessão ela quase
   pegou duas leituras.
2. **C2 — LazyBrush** (`docs/Flip/09_colorize.md` §7). Zero `max_flow`/`min_cut` no repo. Abre
   com **duas decisões do Enio**: medir o corte binário na grade ANTES de construir UI (decide
   entre síncrono e o padrão `progress`), e o pedido de exceção `rayon`.
3. **Congelar o contrato do `ph2d-flip`** — não há gate de superfície. O `FLIP_SCHEMA_VERSION`
   acabou de ir a 8; a nota do plano diz *"quando o modelo assentar"*.

---

## 10. Uma correção ao que uma auditoria desta sessão afirmou

Uma auditoria reportou *"nenhum gate combina fill + reshape"*. **Impreciso.** Existe
`a_filled_stroke_is_one_geometry_so_sculpting_the_line_moves_the_colour`
(`flip_reshape_tests.rs`) — mas ele cobre o caminho do **Draw:Filled** (`stroke_from_samples`
com `draw_filled: true`), e **não** o do balde (`filled_shape_target`). A propriedade estava
desprotegida **no caminho do balde**, que é onde a R3 ia mexer. Os gates novos cobrem esse.

Registrar isto importa: um handoff que repete um fato errado de auditoria vira lei falsa duas
rodadas depois.
