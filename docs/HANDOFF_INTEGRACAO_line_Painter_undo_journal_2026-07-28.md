# HANDOFF DE INTEGRAÇÃO — `line/Painter`: **o journal por tile** (S3, degraus 1–3b)

> Para o **agente integrador**, munido pelo Enio. DIRETRIZ §1.5.9.
> O handoff da leva anterior (já no `main`) é
> [`HANDOFF_INTEGRACAO_line_Painter_undo_delta_2026-07-26.md`](HANDOFF_INTEGRACAO_line_Painter_undo_delta_2026-07-26.md).
> O detalhe técnico vive em [`docs/Painter/28_otimizacoes_o_que_funcionou.md`](Painter/28_otimizacoes_o_que_funcionou.md) §5.20–§5.29 + §7.

## 1. Branch / HEAD / base

| | |
|---|---|
| branch | `line/Painter` (worktree `Worktrees/line-Painter/`) |
| commits à frente de `main` | **17** (`git log --oneline main..HEAD`) |
| base | `main` — **já é ancestral**, `git rebase main` é no-op |
| árvore | limpa |

## 2. ⚠️ Leia isto primeiro: o que esta leva É, e o que ela NÃO é

Ela é o **degrau de INFRAESTRUTURA** do S3 — o journal por tile que captura o "antes" na hora da
escrita, as portas que sabem **nomear** o plano, e a rede que confere o invariante em toda rodada da
suíte. **A metade que PAGA (os ~21 ms/traço) NÃO está aqui e está bloqueada por um pré-requisito
MEDIDO** (§4 abaixo).

O ganho de perf desta leva é **marginal e não é o motivo dela**. O motivo é que o S3 sem esta base
seria construído sobre uma afirmação não-verificada, e o preço de errar é *undo que perde texels em
silêncio*.

**O que muda para o artista: nada visível** — salvo o bug latente do §3.

## 3. O único achado de PRODUTO: `impasto_material` escrevia fora de toda porta

`apply_material_to_stroke` (o Material do "Adjust Last Stroke") escrevia o plano `mats` com um
`Arc::make_mut` **cru**. As três metades que faltavam:

1. o journal não aprendia o byte velho (o passo ficava INCOMPLETO);
2. o fork era **serial** em vez de paralelo;
3. — e o que importa — ele **não abria acesso**, então o contador de escritas não-declaradas não o
   via, e o commit podia aceitar uma janela declarada por *outro* sítio como se cobrisse também esta
   escrita.

⚠️ **É LATENTE, não vivo, e o handoff não o vende como vivo:** o `rect` do material é o do traço, que
os dabs já declararam, então a janela o contém *na prática*. O defeito é que a garantia do S1 —
*esquecer é lento, nunca errado* — **só vale para quem passa por uma porta**, e este sítio não
passava. Fechado em `484684695`.

O gate que o achou é o alargamento do arch-gate: ele varre `tool/paint/**` inteiro em vez de uma
lista de arquivos, com controle positivo nas duas pontas.

## 4. O pré-requisito que bloqueava a metade paga — **ABERTO E FECHADO nesta leva**

`fork_canvas` capturava o **plano inteiro** (`None`), porque nenhum sítio conhecia a sua região no
momento do fork. Medido num traço real (`what_the_journal_retains_for_one_real_stroke`, lido **antes**
do pen-up — o commit zera o journal):

| tela | documento | journal ANTES | **journal DEPOIS** |
|---|---|---|---|
| 1024² | 16,8 MB | 4,19 MB | **0,13 MB** |
| 2048² | 67,1 MB | 16,78 MB | **0,13 MB** |
| 4096² | 268,4 MB | 67,11 MB | **0,13 MB** |

Os 67,11 MB eram `n × 4` — o plano do canvas. Com eles a troca do S3 seria **lateral** (um fork de
67 MB por uma captura de 67 MB); agora ela é **positiva**, e o número é **constante na tela**.

**A cura:** a footprint de um dab é função pura do centro e do raio ⇒ respondível **antes** do laço.
`ph2d_painter_brush::dab_write_bounds` é o **superconjunto declarado** das duas rotas de blit (que
diferem de propósito: `radius` × `radius + aa_pad`) — a porta promete apenas contê-las, e o gate
compara o que elas **devolvem** com o que ela prevê. `region::dabs_bounds` soma as footprints; os 8
sítios de depósito a passam, os 22 frios seguem em `None`.

⚠️ **A premissa:** a lista de dabs que chega às rotas é a **FINAL** — Tiling e Symmetry expandem
cópias **na lista** (`tiled_dabs_grouped`), não dentro do blit. O gate
`every_texel_the_stroke_changed_is_described_by_the_journal` pinta com Tiling nos dois eixos e falha
no dia em que isso mudar.

⚠️ **A mutação *"só o 1º dab"* sobreviveu a DUAS fixtures antes de sangrar:** passos de 18 px põem os
dabs de um batch no mesmo tile de 32 px, e os cinco sítios do `stamp_cache` ainda passavam `None`.
Com salto de 200 px num batch e os cinco ligados: **22.767 de 38.928 texels** sem descrição.

## 5. Foundational tocado — **NENHUM**

54 arquivos, **todos dentro de `crates/ph2d-tool-painter/`**, mais **um** doc
(`docs/Painter/28_…md`). Zero shell, zero crate compartilhada, zero `Cargo.toml`.

- **Contratos congelados (§6): INTACTOS** — conferido por diff de caminho, não por auto-relato.
- **`PROJECT_SCHEMA` = 37, não tocado** por esta linha.
- **Nenhum id de widget, nenhum token, nenhuma chave i18n, nenhum ADR.**

Risco de conflito de integração: **mínimo** — não há lista compartilhada nesta leva. ⚠️ O parágrafo
do `CLAUDE.md` §5 (§7 abaixo) **não foi escrito pela linha de propósito**: ele é a única superfície
compartilhada e cabe ao integrador apendá-lo contra o `main` do dia.

## 6. API de crate nova (privada; para detectar colisão)

| símbolo | o que é |
|---|---|
| `undo_journal::TileJournal<T>` | módulo novo — o journal por tile (TILE = 128 elementos) |
| `undo_window::WriteState::{begin_step, capture_*, note_absent_relief, relief_*}` | o canal do journal |
| `undo_window::ReliefPlane` | ⚠️ **não pode ser `cfg`-gateado** — atravessa a assinatura de uma porta não-gateada |
| `plane_fork::{fork_heights, fork_covers, fork_mats}` | as portas NOMEADAS (ganharam `area`) |
| `plane_fork::fork_par` | ⚠️ virou **`#[cfg(test)]`** — referência congelada, sem chamador de produto |
| `tool::undo_audit` | a rede de verificação, opt-in por `PH2D_UNDO_AUDIT=1` |

⚠️ **`cargo clippy -p` roda em DEBUG.** Um `cfg` errado em `ReliefPlane` só falha em `--release`, e
foi assim que ele foi pego. **O gate de fechamento tem de rodar os dois perfis.**

## 7. Parágrafo para o `CLAUDE.md` §5 (apendar ao FIM do bloco do Painter)

> **⬛ O JOURNAL POR TILE — a base do S3, e o pré-requisito que a mediu como bloqueada (2026-07-28,
> `line/Painter`, [doc 28](docs/Painter/28_otimizacoes_o_que_funcionou.md) §5.20–§5.29 + §7):** o
> "antes" de um passo passa a poder ser capturado **na hora da escrita** (`undo_journal::TileJournal`,
> primeira captura por tile é a que vale) em vez de derivado de dois snapshots completos. Nesta leva
> ele é **rede de verificação** (`cfg(debug)`, opt-in `PH2D_UNDO_AUDIT=1`): **0 divergências** em
> canvas·relevo·cursor sobre a suíte inteira, e o censo de relevo foi de `DESCREVE 42 / INCOMPLETO
> 260` para **`302 / 0`**. Três degraus fecharam: o journal é ancorado no **PASSO**, não no último
> commit (senão o escorrido do Wet Paint entra no lado `before` do passo anterior) · **todo** plano de
> relevo passa por uma porta que sabe **NOMEÁ-LO** (o journal é tipado e keyed por camada; uma porta
> que não nomeia deixa o journal incompleto) · e **AUSENTE ≠ INCOMPLETO** (um plano que não existia no
> começo do passo não tem *antes* a descrever — é o `OnlyAfter` do motor de delta; conflatá-los
> custava 202 passos). ⚠️ **O alargamento do arch-gate achou um sítio CRU** (`impasto_material`
> escrevia `mats` por `Arc::make_mut` direto, sem **abrir acesso**) — latente, porque o `rect` dele
> cabe na janela que os dabs declaram, mas a garantia do S1 (*esquecer é lento, nunca errado*) **só
> vale para quem passa por uma porta**. ⚠️ **E a metade que PAGA está BLOQUEADA, com número:**
> `fork_canvas` captura o plano INTEIRO (**67,11 MB a 4096², exatamente `n × 4`**) porque nenhum sítio
> conhece a sua região no fork ⇒ promover o journal hoje trocaria um fork de 67 MB por uma captura de
> 67 MB — **lateral, não positiva**. As três portas do relevo já passam a região nos onze sítios; o
> canvas é o único outlier, e as duas saídas estão na §7 do doc 28 (⚠️ o bbox dos dabs **não** é
> superconjunto seguro sob Tiling). ⚠️ **A sonda que mede isso lia o journal DEPOIS do pen-up e
> reportava 0,00 MB em toda tela** — o commit move o cursor e `set_cursor` zera o journal; *um zero
> lido no instante errado parece "não custa nada"*. ⚠️ **`fork_par` virou `#[cfg(test)]`** (referência
> congelada — um `pub(super)` órfão é uma segunda resposta esperando alguém chamá-la, a lição do
> `warp_axis`/`serial_side`), e ⚠️ **`ReliefPlane` NÃO pode ser `cfg`-gateado**: ele atravessa a
> assinatura de uma porta não-gateada, e **`cargo clippy -p` roda em debug** ⇒ só o check de
> `--release` pega. Nenhum schema, nenhum contrato congelado, nenhum id/token (`PROJECT_SCHEMA` 37).

## 8. Gate de fechamento — rodado, verde

| gate | resultado |
|---|---|
| `cargo test -p ph2d-tool-painter --release` | **892 passed, 0 failed** |
| `cargo test -p ph2d-tool-painter` (debug) | **892 passed, 0 failed** |
| `cargo test -p ph2d-host-desktop --release` | **1286 + 21 suítes, 0 failed** |
| `cargo clippy -p ph2d-tool-painter --all-targets` (debug **e** release) | limpo |
| `cargo check --workspace --all-targets` | limpo |
| `architecture_workspace_file_loc_cap` | ok |
| `file_loc_caps` (shell, HR-18) | ok |
| `arch_safe_clamp_only` | ok |

## 9. Smoke

Esta leva **não tem cena própria** — ela é infraestrutura de undo, e o que ela pode quebrar é o undo
existente. Os smokes que a cobrem são os que já existem:

```
env PH2D_IMPASTO_SMOKE=2 PH2D_PAINT_PERF=1 cargo run -p ph2d-host-desktop --release
env PH2D_WETPAINT_SMOKE=1                  cargo run -p ph2d-host-desktop --release
env PH2D_MASK_SMOKE=1                      cargo run -p ph2d-host-desktop --release
```

O que olhar: **pinte, desfaça, refaça** — a tinta **e o relevo** voltam iguais; no Wet Paint, o
escorrido de um traço desfeito vai embora junto com ele; e o `PH2D_PAINT_PERF` não pode ter
regredido (`dispatch max` na casa de 1 ms a 4096², não de centenas).

## 10. Aberto, com preço

| item | preço / estado |
|---|---|
| **S3 3b-v (a metade que paga)** | **DESBLOQUEADA.** ~19 ms/traço de fork a 4096² (medido pela porta do produto: covers 5,86 + heights 4,46 + mats 8,58) |
| o `cursor` e o `stroke_undo` seguram os 4 planos | por isso a troca é **tudo-ou-nada**: `make_mut` copia com **qualquer** coisa acima de um dono |
| ⚠️ o canvas **já tem dono único dentro do gesto** | medido: `canvas 1 · heights 4 · covers 4 · mats 4` logo após o pen-down — quem paga o fork é o **RELEVO** |
| a evidência que a wave precisa já existe | `cursor/RECONSTRUIDO: divergem=0` em **1113** reconstruções ⇒ o cursor É derivável de (vivo + journal) |

⚠️ **O desenho está escrito na §7 do doc 28, com as seis restrições** — inclusive a que decide a
primeira linha de código (o `absorb_foreign_writes` tem dois chamadores, e só um deles tem a
reconstrução provada).
