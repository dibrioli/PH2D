# HANDOFF DE INTEGRAÇÃO — `line/Painter`: **o journal por tile** (S3, degraus 1–3b) + **o Wet Paint a 4 FPS**

> Para o **agente integrador**, munido pelo Enio. DIRETRIZ §1.5.9.
> O handoff da leva anterior (já no `main`) é
> [`HANDOFF_INTEGRACAO_line_Painter_undo_delta_2026-07-26.md`](HANDOFF_INTEGRACAO_line_Painter_undo_delta_2026-07-26.md).
> O detalhe técnico vive em [`docs/Painter/28_otimizacoes_o_que_funcionou.md`](Painter/28_otimizacoes_o_que_funcionou.md) §5.20–§5.30 + §7.

## 1. Branch / HEAD / base

| | |
|---|---|
| branch | `line/Painter` (worktree `Worktrees/line-Painter/`) |
| commits à frente de `main` | **26** (`git log --oneline main..HEAD`) |
| base | `main` — **já é ancestral**, `git rebase main` é no-op |
| árvore | limpa |

## 2. ⚠️ Leia isto primeiro: o que esta leva É, e o que ela NÃO é
Ela é o **degrau de INFRAESTRUTURA** do S3 — o journal por tile que captura o "antes" na hora da
escrita, as portas que sabem **nomear** o plano, a rede que confere o invariante em toda rodada da
suíte, e o **pré-requisito que bloqueava a metade paga** (§4: o journal retinha o canvas inteiro;
agora retém a PEGADA — 67,11 → 0,13 MB, constante na tela).

**A metade que PAGA (~19 ms/traço de fork) NÃO está aqui**, e agora está **desbloqueada** em vez de
bloqueada — o desenho e as seis restrições estão na §7 do doc 28.

O ganho de perf desta leva é **marginal e não é o motivo dela**. O motivo é que o S3 sem esta base
seria construído sobre uma afirmação não-verificada, e o preço de errar é *undo que perde texels em
silêncio*.

**O que muda para o artista: nada visível** — salvo o bug latente do §3.

⚠️ **E a leva tem uma SEGUNDA metade, que não é infraestrutura e é bem visível: o Wet Paint a 4 FPS**
(§4.5 abaixo). Ela veio de um smoke do Enio no meio da leva e é independente do S3 — mexe só no
`ph2d-wet-paint` e na ponte dele no tool.

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

## 4.5 A segunda metade: **o Wet Paint caía a 4 FPS** — três mecanismos

**Report do Enio:** *"IMG 4096, 1 pincelada grande e molhada, FPS para 4."* O log dele com
`PH2D_FLUID_PROFILE=1` nomeou a fase: **`tool-tick = 57,49 de 69,99 ms (82%)`**, com o
`painter-dispatch` em 2,51.

| # | mecanismo | número |
|---|---|---|
| a | **realimentação positiva**: `on_tick(dt)` com `dt` = o frame ANTERIOR, cap `WET_MAX_STEPS = 5` ⇒ frame lento pede mais passos ⇒ frame mais lento | dt 16,6 → 2,08 ms · dt 250 → **50,93 ms**; cap 1/2/3/5 = 2,69/5,79/9,74/50,93 ⇒ **cap = 2** |
| b | o **OVER do composite** era o único laço serial que sobrava no tick (ADR-0109: linhas disjuntas, leitura pura) | 16,17 → **11,71 ms** |
| c | **a sim varria a CAIXA, não a poça** — a bbox é o CASCO, e num diagonal ela é 27,9% da tela com **2,4%** de células ativas | soma dos passes **48,6 → 11,35 ms (4,3×)**; tick do produto por ablação **31,42 → 13,04 (2,41×)** |

**(c) é a FAIXA VIVA** — um intervalo **por LINHA** (`Grid::row_lo`/`row_hi`) no lugar do casco.
Byte-idêntico **por construção** nos seis passes cujo ramo inativo é um `continue` puro; o `advect` é a
exceção declarada (o ramo dele **escreve**, zerando `vel`) e se apoia num invariante que uma **rede de
debug** afirma a cada passo. ⚠️ **A rede se pagou na PRIMEIRA execução da suíte**, achando uma
divergência real (velocidade fóssil no rastro de um drip, e o fingerprint da sessão inclui `vel`).

**O oráculo é DIFERENCIAL** (`crates/ph2d-wet-paint/tests/spans.rs`): a mesma sessão roda com
`Grid::spans_enabled` ligado e desligado — o **mesmo laço com o intervalo mais largo**, não uma segunda
implementação — e **todo campo persistente** tem de sair idêntico ao byte, em **seis formas de sessão**,
mais o Fast Dry e a rota de undo. **O fingerprint pinado do engine está INTACTO.**

⚠️ **Para o integrador, o que pode colidir:** `Grid` ganhou 5 campos públicos (`row_lo`, `row_hi`,
`live_lo`, `live_hi`, `spans_enabled`) e `GridSnapshot` ganhou 2 (`row_lo`, `row_hi`) — **construções
literais de `Grid`/`GridSnapshot` fora do crate quebram**; hoje não existe nenhuma (`Grid::new` e
`snapshot_grid` são as portas). Nada disso é serializado em disco.

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
| `grid::{SPAN_PAD, span_x_of, verify_spans}` | **`ph2d-wet-paint`** — a faixa viva: o pad, a porta livre, a rede de debug |
| `Grid::{span_x, span_window, note_live, clear_spans, clear_live, publish_spans_from_live}` | as portas da faixa (`verify_spans` é `cfg(debug_assertions)`) |
| `Grid::{row_lo, row_hi, live_lo, live_hi, spans_enabled}` · `GridSnapshot::{row_lo, row_hi}` | ⚠️ **campos públicos novos** — quebram construção literal fora do crate (hoje não há nenhuma) |

⚠️ **`cargo clippy -p` roda em DEBUG.** Um `cfg` errado em `ReliefPlane` só falha em `--release`, e
foi assim que ele foi pego. **O gate de fechamento tem de rodar os dois perfis.**

## 7. Parágrafo para o `CLAUDE.md` §5 (apendar ao FIM do bloco do Painter)

> **⬛ O JOURNAL POR TILE — a base do S3, e o pré-requisito da metade paga aberto e FECHADO (2026-07-28,
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
> vale para quem passa por uma porta**. ⚠️ **E o pré-requisito da metade PAGA abriu e fechou no mesmo
> dia:** `fork_canvas` capturava o plano INTEIRO (**67,11 MB a 4096², exatamente `n × 4`**) porque
> nenhum sítio conhecia a sua região no fork ⇒ a troca seria **lateral** (um fork de 67 MB por uma
> captura de 67 MB). A footprint de um dab é **função pura** do centro e do raio, logo respondível
> ANTES do laço: `ph2d_painter_brush::dab_write_bounds` é o **superconjunto declarado** das duas rotas
> de blit (que diferem de propósito — `radius` contra `radius + aa_pad` — e seguem donas da própria
> aritmética), com gate comparando o que elas **devolvem** contra o que ela prevê. **Journal 67,11 →
> 0,13 MB, e CONSTANTE na tela** ⇒ a troca do S3 é positiva. ⚠️ A premissa é que a lista de dabs seja
> a **FINAL** — Tiling e Symmetry expandem cópias **na lista** (`tiled_dabs_grouped`), não dentro do
> blit —, e o gate pinta com Tiling nos dois eixos para falhar no dia em que isso mudar. ⚠️ **A
> mutação *"só o 1º dab"* sobreviveu a DUAS fixtures:** passos de 18 px põem os dabs de um batch no
> mesmo tile de 32 px, e os cinco sítios do `stamp_cache` ainda passavam `None` (22.767 de 38.928
> texels sem descrição depois de corrigidos os dois). ⚠️ **A sonda lia o journal DEPOIS do pen-up e
> reportava 0,00 MB em toda tela** — o commit move o cursor e `set_cursor` zera o journal; *um zero
> lido no instante errado parece "não custa nada"*. ⚠️ **`fork_par` virou `#[cfg(test)]`** (referência
> congelada — um `pub(super)` órfão é uma segunda resposta esperando alguém chamá-la, a lição do
> `warp_axis`/`serial_side`), e ⚠️ **`ReliefPlane` NÃO pode ser `cfg`-gateado**: ele atravessa a
> assinatura de uma porta não-gateada, e **`cargo clippy -p` roda em debug** ⇒ só o check de
> `--release` pega. Nenhum schema, nenhum contrato congelado, nenhum id/token (`PROJECT_SCHEMA` 37).

> **⬛ E O WET PAINT CAÍA A 4 FPS — três mecanismos, e o maior era a sim varrendo a CAIXA (2026-07-28,
> `line/Painter`, [doc 28](docs/Painter/28_otimizacoes_o_que_funcionou.md) §5.30, pendente de smoke):**
> report do Enio (*"IMG 4096, 1 pincelada grande e molhada, FPS para 4"*) cujo log com
> `PH2D_FLUID_PROFILE=1` nomeou a fase antes de qualquer teoria — **`tool-tick` = 57,49 de 69,99 ms
> (82%)**, com o `painter-dispatch` em 2,51. ⚠️ **A instrumentação que responde isso já existia, atrás
> de outra flag**, e as minhas medições do dia mediam o tool com fixture própria (pior caso 11,7 ms)
> enquanto o produto pagava 57,5. **(a) O tick REALIMENTAVA um frame lento:** `on_tick(dt)` recebe o
> relógio do frame ANTERIOR e o acumulador dava `dt / WET_STEP_S` passos capados em 5 ⇒ frame lento
> pede mais passos ⇒ frame mais lento, **realimentação POSITIVA e invisível a qualquer sonda de `dt`
> fixo** (medido a 4096²: dt 16,6 → 2,08 ms · dt 250 → **50,93**; ablação do cap 1/2/3/5 =
> 2,69/5,79/9,74/50,93) ⇒ **cap = 2**, com o precedente do `max_substeps` da física: *sacrifica-se
> tempo SIMULADO, nunca o quadro*. **(b) O OVER do composite** era o que sobrava serial ⇒ row-parallel
> pelo ADR-0109, **16,17 → 11,71 ms** — e ⚠️ o gate que faltava não é de aritmética: o fan-out por linha
> quebra o **mapeamento linha → offset global**, e a mutação `gb = k*stride` sobreviveu às **895**
> existentes porque toda fixture irmã pinta com região suja começando na linha 1. **(c) A SIM PAGAVA
> PELA CAIXA, NÃO PELA POÇA**, e este era o grande: mesma água, mesmo comprimento de traço, só a FORMA
> muda — horizontal **8,15 ms** (bbox 2,1% da tela) contra diagonal **23,53** (18,6%). ⚠️ **Isso
> reinterpreta a tabela anterior** (400→3600 px, 2,37→16,17), onde eu variava o COMPRIMENTO de um traço
> *horizontal* e caixa e água cresciam juntas — as duas explicações casavam com os mesmos números, e só
> a forma as separa. **A bbox é o CASCO da água e um casco mente sobre um diagonal:** a caixa é 27,9%
> da tela e as células ATIVAS são **2,4% dela** ⇒ 97,6% de cada varredura era desperdício, e pincelada
> de artista nunca é horizontal. **A FAIXA VIVA** (`Grid::row_lo`/`row_hi`) troca o casco por um
> intervalo **POR LINHA**; todo passe já fazia early-out por-célula (`active[i] == 0` → `continue`),
> então pular célula fora da faixa é pular uma que não responderia nada — **byte-idêntico POR
> CONSTRUÇÃO** nos seis passes com essa forma. O invariante `active ⊆ faixa` **também** vale por
> construção (o rebuild escreve `active` só na própria janela e publica a faixa como a extensão viva
> dilatada por `SPAN_PAD = 5` — folgadíssimo: `maxVelocity` default é 0,2 célula/frame e o rebuild roda
> a cada 2 frames). ⚠️ **O `advect` é a EXCEÇÃO e está escrito como tal:** o ramo inativo dele
> **ESCREVE** (zera `vel`), então ele se apoia num invariante — *fora da faixa, `vel` já é zero* — e não
> em construção. ⚠️ **A rede de debug se pagou na PRIMEIRA execução da suíte:** invariantes 1 e 2
> passaram, o 3 falhou em `(26, 22): (0.0069, -0.0044)` — o rastro de um drip que se afasta mais de 5
> células ficava com velocidade **fóssil**, e o fingerprint da sessão inclui `vel_x`/`vel_y`, ou seja
> **divergência real** do motor original ⇒ `vel != 0` entrou na definição de VIVO. ⚠️ **Duas fugas do
> caso BASE da indução, as duas achadas pela mesma rede:** `empty_bbox` zerava a faixa — mas a água pode
> ACABAR com velocidade fóssil no rastro, e é a faixa que lembra onde (quem zera a faixa passou a ser
> quem zera a velocidade, `clear_canvas`) · e o rebuild varria só as linhas da bbox, e **a bbox de um
> traço NOVO não tem por que cobrir o rastro de um antigo** (agora varre toda linha de janela não-vazia,
> O(altura)). ⚠️ **E o SNAPSHOT carrega a faixa:** a alternativa — abri-la inteira no restore e deixar o
> rebuild reapertá-la — foi **MEDIDA e reprovada** (a varredura viva passa a cobrir a folha, **0,3 →
> 17,4 ms**, um quadro perdido a cada Ctrl+Z do motor). **MEDIDO a 4096², decomposição pelas portas
> públicas:** `rebuild` 31,05 → **1,51** · `project` 12,66 → **1,39** · `build_flow` 11,94 → **3,07** ·
> `advect` 10,51 → **2,38** ⇒ **soma 48,61 → 11,35 ms (4,3×)**, com a razão diagonal/horizontal por
> passe caindo de **20-25×** para **~2×**; o caso horizontal ficou **+8,7%** (a varredura viva é um
> passe a mais, e casco fino é onde ela não tem o que economizar) — nomeado, não vendido. **No PRODUTO,
> por ABLAÇÃO** (`Grid::spans_enabled`, o MESMO laço com intervalo mais largo — não há segunda
> implementação a divergir): tick p50 diagonal **31,42 → 13,04 ms (2,41×)**, horizontal 9,63 → 8,65.
> **O gate é DIFERENCIAL, não um valor pinado:** seis sessões (horizontal · diagonal · drip sob
> gravidade · dois traços com a sim indo a **idle** no meio · Wet+Blend sobre tinta seca · traço em L)
> rodam nos dois modos e **todo campo persistente** sai idêntico ao byte, mais Fast Dry e a rota de
> undo; e há um gate de **PROPRIEDADE** (a faixa é fração pequena da bbox num diagonal), porque uma
> mudança futura que devolvesse a bbox inteira continuaria **CORRETA** e teria jogado o ganho fora em
> silêncio. ⚠️ **E o meu gate de ablação no tool NASCEU MENTINDO "1,02×":** a sessão de água nasce no
> **pen-DOWN**, então armar o flag antes dele é um `if let` que não casa — *busca negativa sem controle
> positivo*, outra vez. **Onde ficou o tick, por metade:** sim **13,84** · composite **2,79** (18,8% da
> tela suja); sem realimentação (dt 16,6 → 1,71 · dt 250 → 5,58). **Fingerprint da sessão INTACTO**,
> nenhum schema, nenhum contrato congelado, nenhum id/token (`PROJECT_SCHEMA` 37). **Aberto e
> NOMEADO:** o retângulo sujo que o engine declara ao composite é um casco **pela mesma razão que a
> bbox era**, mas custa 2,79 ms medidos ⇒ **não é a fronteira**; e o resto é trabalho honesto sobre 111k
> células ativas × 7 passes, cuja próxima alavanca é paralelismo (⚠️ o **ADR-0134 declara o solver
> serial POR SEMÂNTICA** — não re-derive) ou porte para GPU.

## 8. Gate de fechamento — rodado, verde

| gate | resultado |
|---|---|
| `cargo test -p ph2d-tool-painter --release` | **896 passed, 0 failed** |
| `cargo test -p ph2d-tool-painter` (debug) | **896 passed, 0 failed** |
| `cargo test -p ph2d-wet-paint` (debug — **a rede da faixa roda aqui**) | verde, incl. o fingerprint pinado |
| `cargo test -p ph2d-wet-paint --release` | verde |
| `cargo clippy -p ph2d-wet-paint --all-targets` | limpo |
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

⚠️ **E o smoke que decide a §4.5 é o do Wet Paint, com o canvas em 4096 e uma pincelada grande, molhada
e CURVA** (a forma importa — num traço horizontal a caixa já era fina e não havia o que ganhar):

```
env PH2D_WETPAINT_SMOKE=1 PH2D_FLUID_PROFILE=1 cargo run -p ph2d-host-desktop --release
```

O número a ler é o **`tool-tick`** da linha `[frame]`: ele era **57,49 ms de 69,99** e tem de sair da
casa das dezenas. A água tem de continuar se comportando como água — o oráculo de aparência é o smoke,
o de corretude são os gates diferenciais.

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
