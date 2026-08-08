# Handoff de integração — `line/motion-value` (os PARÂMETROS dos nós)

> DIRETRIZ §1.5.9. **A linha NÃO integra e NÃO pusha** — este documento é o que o Enio passa ao
> agente integrador. Escrito por MEDIÇÃO: todo número aqui saiu de um comando, não de memória.

---

## 1. Identidade

| | |
|---|---|
| Branch | `line/motion-value` |
| HEAD | `dc8ac591e` |
| Merge-base com `main` | `a4018d203` |
| Commits | **48** |
| Diff | **181 arquivos, +15.507 / −2.674** |
| Janela | 2026-08-05 → 2026-08-08 |

---

## 2. O que a wave entrega (quatro clusters, e eles são independentes)

**(A) A GPU do `source.object` + o VETOR VIVO** (`044abbf8e` … `ecb5232f2`, 4 commits) — o objeto
**cozinha e renderiza no device**; um `source.object` de VETOR renderiza *crisp* em vez de virar tile
raster; LOD híbrido + cache de tesselação por-frame. ⚠️ Isto **reabre** a recusa que o ADR-0155
instalou em 04/08 (*"um documento com fonte de APARÊNCIA recusa o cook GPU"*): a recusa continua,
mas agora só onde ela ainda é necessária — o gate `the_gpu_cook_recusal_placement` é quem pina
**onde** ela mora, e `gpu_texture_id` prova que o lowering do device escreve o id REAL.

**(B) O doc 88 — os parâmetros dos nós** (o corpo da wave, ~20 commits; plano novo em
`docs/Motion Nodes/88_plano_parametros_nos_unidades_e_slider.md`):

- o **vocabulário de UNIDADE** (`ParamUnit`) — *o que o número É*, nunca como se mostra;
- a **fronteira de DISPLAY** — o número sai UMA vez na face do artista e volta pela mesma porta;
- o **piso duro** por param (`ParamHardMin`) — e a assimetria morava em DOIS lugares;
- a unidade chega a **43 nós** por varredura de opt-in, com **censo que a tranca**;
- as **SEÇÕES** de params — a parede de treze sliders vira três perguntas (**10 nós**);
- o **reset ao default** (a seta que devolve o valor de fábrica);
- o **teto de linhas** do painel (escondia params: 8 contra os 13 do `field.remap`);
- o **oscilador** ganha régua de tempo; o **ruído** fecha o ciclo e o WGSL dele deixa de existir
  em três cópias.

**(B2) O SLIDER DUAL — a caixa vai ALÉM do slider** (`f60999baa` · `779cf4f9f` · `f9e6cf597`, o
item A1 do doc 88). O `ParamUiHint.max` passa a ser **só a faixa confortável do arrasto**, e o
`ParamHardMax` — canal *side-metadata* que já existia no registry — diz **onde o disfuncional
começa**: o número que a caixa de texto ainda aceita. Onze nós de contagem carregam hoje um teto
**MEDIDO**, com a tabela ao lado dele no doc-comment do próprio nó.

⚠️ **A feature NÃO funcionava em lugar nenhum, e três gates ficavam VERDES sobre isso.** O
espelho chip↔slider re-escrevia o chip com a re-projeção do slider **saturado**, e depois — já com
o chip certo — o slider **também** emitia, com o thumb parado no topo, e *o último vencia*. Os
gates de faixa do painel escrevem o valor com `set_number_value` e **nunca PINTAM**; a faixa
registrada e o link com o slider nascem no `paint`, então a fixture deles não continha o espelho
que diziam exercitar. A cura: a faixa do CHIP é a autoridade, e o evento do slider passa por uma
**porta única** (`push_mirrored_slider_event`) que se cala quando o thumb é um substituto saturado
— *dois sítios emitem esse evento hoje, e um `if` copiado é a regra que o terceiro nasce sem*.
Ferramenta nova que tornou o gate possível: **`MockPanelHost::type_into_number`** — o testkit só
sabia ESCREVER no store, que pula o caminho de commit inteiro. **Smokado pelo Enio.**

⚠️ **E a varredura dos tetos achou mais defeito na SONDA que nos nós** — a lei que sobra está
gravada em dois gates: **um teto digitável não pode passar do que o kernel HONRA** (`lattice` 400,
`kaleidoscope` 256; uma caixa que aceita 5.000 sobre um clamp de 400 *aceita e mente*). E o
`motion.boids` media 3,2 ns por agente porque **nunca dava um passo**: o estado dele chega por uma
aresta DELAYED que o editor auto-liga e que `Graph::add_node` não faz. Com o grafo certo o
quadrático aparece — 500 → 0,475 ms · 2.000 → 10,392 · 8.000 → **186,388**.

**(B3) O CORPO DO PAINEL ROLA** (`a27ee6e81`, o §B3 do doc 88). `MAX_PARAM_ROWS` respondia a
**duas** perguntas com um número: quantos ids o pool tem, e quantas linhas cabem na altura do
inspector. **Medido:** uma linha escalar ocupa **34,0 px** e o dock comporta **24** delas, contra
um teto de **16** e um pior nó (`motion.tint`) de **15** params — oito linhas de folga para uma
varredura que promete a TODO nó o conjunto PRO. O gate `a_full_panel_of_rows_fits_the_inspector`
já dizia o que fazer no dia: *"o painel precisa ROLAR antes de o teto subir mais"*.

⚠️ **O FATO FICA NOMEADO, não escondido: a rolagem está INERTE hoje.** O teto mora **dentro** do
`paint_rows` (`.take(MAX_PARAM_ROWS)`), então o corpo mede no máximo **~544 px contra um dock de
880** e nenhum nó transborda. Ela não é enfeite — é o que **remove o dock da lista de razões para
o teto não subir**, que é o pré-requisito da varredura B3 propriamente dita.

⚠️ **As QUATRO edições que um painel rolável exige** (o arch-gate `scrollable_panels_intercept_the_wheel`
as nomeia): id do thumb → arm no `scrollbar_panel_for_id` → o painter lê `panel_scroll` e publica
`content_h`/`visible_h` → **e o id no `cursor_over_hero_panel`**, que é a que *compila, pinta,
arrasta, e deixa a roda dando ZOOM na câmera em silêncio*. A mutação que tira a quarta produz
exatamente essa mensagem no gate.

⚠️ **E a cerca cuja premissa isto matou foi ENCARADA em vez de deixada verde:** o gate que afirmava
*"as linhas CABEM"* pediria um teto **MENOR** no dia em que uma não coubesse — o oposto da cura.
Ele virou **`the_reported_height_is_the_true_height_of_the_rows`**, a propriedade que passou a ser
load-bearing: o scrollbar deriva `max_scroll` de `content_h − visible_h`, e uma altura que
saturasse convenceria o painel de que tudo cabe — com as linhas desenhadas, o thumb ausente e **a
cauda perdida em silêncio**.

⚠️ **Uma mutação SOBREVIVEU e o defeito era da FIXTURE:** clampar a altura na altura do **dock**
passa despercebido, porque o cap do `.take()` mantém o conteúdo abaixo dela — só um clamp abaixo
de 544 é observável hoje. **E o nome que eu dei ao 1º gate de rolagem afirmava mais do que ele
mede** (*"mais conteúdo do que o dock mostra"*), corrigido para o que ele prova de fato.

**(B4) A VARREDURA POR FAMÍLIA COMEÇA — e um CENSO a escolhe** (`0e3e6270b` · `87d57bdcb`,
o §B3 propriamente dito; o mapa curado vive na **§9 nova do doc 88**).

A §6 do plano dizia *"nó a nó, ou família a família"*, e a §0 manda medir antes de decidir
⇒ a wave abre com uma **sonda de censo** (`param_census`, na `ph2d-node-registry-init` — a
crate que registra os 118 nós, e o build mais barato que os enxerga). **Retrato:
118 nós · 411 params · 395 com hint · 105 com unidade**; **um** nó tem param sem hint
nenhum e **cinquenta** têm ≤ 2 controles. ⚠️ *Magro por natureza* e *magro por omissão*
são coisas diferentes e o censo não as distingue — quem distingue é a referência, e é
disso que a §9 do doc 88 é a tabela (com o veredito **recusado-com-motivo** para as
famílias VALUE, ESTRUTURAIS e RIG, para ninguém as "completar" depois).

**Ele escolheu a família TRANSFORM, porque dois nós CONTRADIZIAM a coluna que escrevem:**

- **`motion.scale`** escrevia `size`, uma coluna **Vec2**, a partir de **UM** número.
  *Squash & stretch* não era difícil no grafo — era **inexprimível**. Agora `uniform` é o
  *link* de corrente do AE/Cavalry/Figma e destravá-lo revela `amount_y`.
- **`motion.mirror`** pregava a linha de espelho no **CENTROIDE**, então a simetria só
  sabia acontecer contra si mesma. `offset` move a linha — e é medido **a partir do
  centroide**, porque um offset absoluto não teria como exprimir *"no centroide"* e o
  zero deixaria de ser o comportamento antigo.

⚠️ **Os dois defaults são byte-idênticos ao que shipava** (`uniform` nasce ligado, `offset`
nasce zero), cada um reduzindo **literalmente** à expressão anterior — a demo de boot
sozinha põe treze `motion.scale`, e uma varredura de params é o que mais facilmente
destrói arte já autorada.

⚠️ **Duas lições de gate desta wave, as duas por MUTAÇÃO:**

1. **O gate de paridade GPU antigo era CEGO ao ramo novo.** Com o link no default o
   `select` do WGSL nunca entra no braço do segundo eixo — mutar o kernel para ignorá-lo
   deixa `scale_kernel_matches_the_cpu_within_epsilon` **VERDE** e só o irmão novo
   (`the_unlinked_scale_axes_match_the_cpu_within_epsilon`) sangra. *Um ramo de kernel que
   nenhum gate percorre é o que ship quebrado com a suíte verde.*
2. **Uma capacidade sem PORTA passa em todo gate de função pura.** Mutar o `eval` do mirror
   para não ler `ctx.param("offset")` deixa os cinco gates do kernel verdes; só o gate que
   atravessa o **COOK** sangra.

**A 2ª família — ECHO** (`5867f2c22`): o censo mediu `motion.trail` com **3 params contra
os 8** da referência, e o que faltava não era enfeite — um fantasma por TICK é um rastro
**contínuo**, que a 60 fps lê como borrão; o *sprite echo* (que o catálogo traz com
`spacing 2` no default DELE) era **inexprimível**. ⚠️ **O `spacing` não custou estado
novo**, e é o desenho inteiro: a coluna `trail_age` já sabia há quantos ticks o último eco
foi deixado, então a promoção da cabeça é uma pergunta ao **ESTADO**, não a um contador —
e com `spacing = 1` a faixa consultada é vazia, a promoção acontece sempre, e o motor é
**byte a byte** o que sempre shipou.

⚠️ **E o gate do SMOKE achou o que a suíte do nó não via:** com a janela ingênua
(`length × spacing`) o `length` passava a significar **duas coisas** — LINHAS em
espaçamento 1 e linhas + 1 acima dele. A janela correta é `(length − 1) × spacing + 1`, e
quem a pina é a **igualdade de CONTAGEM** entre as duas esteiras da cena. *Uma cena que
compara dois ajustes lado a lado mede o que uma suíte que olha um ajuste por vez não
alcança.*

⚠️ **O `hueShift` do mesmo catálogo NÃO entrou, com o motivo no próprio nó:** girar matiz
em RGB linear com uma matriz YIQ é o atalho que todo motor de partícula usa e que este app
**não usa em lugar nenhum** — a cor aqui passa por OKLCH. Seria uma segunda resposta a
*"o que é girar uma cor"*, divergindo no único lugar onde ninguém lê um número.

**Smokes: `PH2D_TRANSFORM_SMOKE=1`** · **`PH2D_ECHO_SMOKE=1`** (duas esteiras iguais em
órbita, só o espaçamento difere — se os dois rastros forem iguais, o param não chegou).
O primeiro: — a cena monta 4 pontos esticados 2.2×/0.45× e
espelhados numa linha a 1.2 m do centroide (8 instâncias), com 4 testes na mensagem.
⚠️ **Se os oito saírem quadrados, PARE**: o eixo Y não chegou.

**Zero schema, zero contrato congelado, zero dep, zero `Cargo.toml`.**

---

**(C) A wave B** (`bd0bc6d7a` … `2378cfd10`) — a **paleta vira SWATCHES** (sem limite de
comprimento, por construção) e o **look-at ganha alvo por NOME e pelo CURSOR**. Inclui o fix do
**drift crônico do Motion** (o cursor era projetado pela janela CHEIA — **terceira vez** que este
defeito aparece no módulo).

**(D) O painel e a row dirigida** (`178fab5b1` … `9f1b8ff63`, 5 commits) — o editor **abre VAZIO**
(a neve sai do boot e vira fixture `#[cfg(test)]`), o painel **prova que CABE** no dock, e a **row
dirigida diz QUEM a dirige** (elo + nome do card, pela porta única `card_title`).

---

## 3. Foundational / compartilhado tocado, e por quê

Tudo **aditivo** salvo onde marcado.

| Arquivo | O quê | Forma |
|---|---|---|
| `ph2d-color/src/palette_text.rs` **(NOVO)** + `lib.rs` (+2) | o formato TEXTO de uma paleta, ao lado do do gradiente — *uma crate é dona de como uma cor se escreve* (precedente: `motion-color-ramp`) | arquivo próprio ⇒ isolado por construção |
| `ph2d-editor-core/src/screens/layout.rs` | **`INSPECTOR_MAX_H`** — const `pub` NOVA (era literal solto no `Rect::new`) | ⚠️ **símbolo novo, ver §4** |
| `ph2d-editor-core/src/project.rs` | ⚠️ **SÓ doc-comments.** Os dois que contradiziam o `Default` real | **zero mudança de comportamento — ver §6** |
| `ph2d-editor-core/src/interaction/dispatch/{mod,tick}.rs` | ⚠️ **MUDANÇA DE COMPORTAMENTO** no espelho chip↔slider: a **faixa do chip é a autoridade** (um valor digitado além da trilha sobrevive) e o evento do slider sai por **`push_mirrored_slider_event`**, que se cala com o thumb saturado | `pub(super) fn` nova + os 2 sítios de emissão roteados por ela |
| `ph2d-editor-core/src/widget/scrollbar.rs` + `widget/mod.rs` | **`MOTION_PARAMS_SCROLLBAR_ID = NodeId(839)`** — ⚠️ **símbolo colidível**; próximo livre **840** | aditivo (const nova + re-export) |
| `ph2d-editor-core/src/interaction/dispatch/scroll.rs` | o braço que ARMA o painel para a roda | aditivo (um `match` arm) |
| `shells/desktop/src/forwarding.rs` | `MOTION_PARAMS_PANEL` no `cursor_over_hero_panel` | aditivo — **sem ele a roda dá ZOOM** |
| `ph2d-ui-testkit/src/lib.rs` | **`type_into_number`** — digitar de verdade (foco → `dispatch_text_input` por caractere → Enter) | aditivo (`pub fn` novo) |
| `ph2d-nodegraph/src/graph.rs` | `clear_param` / `clear_text_param` | aditivo (`pub fn` novos) |
| `ph2d-nodegraph/src/external.rs` | o **namespace reservado `$`** (`RESERVED_PREFIX`, `is_reserved`, `CURSOR`, `position_of`) — o alvo do look-at pelo cursor | ⚠️ **símbolos novos, ver §4** |
| `ph2d-node-registry/src/` (+ `unit.rs` NOVO) | **6 canais side-metadata** novos: `param_units` · `param_groups` · `param_hard_min` · `live_vector_source` · `object_source` · `card_title` | **o padrão canônico** — nenhum toca `NodeManifest` |
| `ph2d-render/` (`clip_pass`, `renderer_draw`, `sprite/instance`, `sprite/mod`) | os **texture runs** do draw extra da GPU | gate próprio novo |
| `ph2d-vec-render/` (`instance.rs` NOVO, `lib.rs`) | o **vetor vivo** do `source.object` | |
| `ph2d-gpu-cook/` (`tex_runs.rs` NOVO + 6 arquivos) | o lowering do objeto no device | |
| `ph2d-panel-motion-graph/src/snapshot_build.rs` | passa a chamar `card_title` | **porta única** (era escada de fallbacks duplicada) |
| `shells/desktop/` | o bridge de params partido em duas metades, os censos, as 4 cenas de smoke, o schema | o grosso do diff |

**Nenhuma crate nova.**

---

## 4. Símbolos que podem COLIDIR (literais, para o integrador grepar)

| Símbolo | Valor | Onde |
|---|---|---|
| ⚠️ **`PROJECT_SCHEMA`** | **56** (main dizia **55**) | `shells/desktop/src/project.rs:247` |
| `INSPECTOR_MAX_H` | `f32 = 880.0` | `ph2d-editor-core/src/screens/layout.rs` |
| `RESERVED_PREFIX` | `char = '$'` | `ph2d-nodegraph/src/external.rs` |
| `CURSOR` | `&str = "$cursor"` | idem |

⚠️ **O `56` é PROVISÓRIO e se CONTA, não se escolhe** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
Ele carrega **um** degrau: `ProjectFile.settings` (`SavedSettings` — a escala e a unidade do
projeto passam a viajar no arquivo). Se outra linha da janela também bumpar, o valor certo é
contado a partir do `main` do dia — e ⚠️ **este é o caso que já passou MUDO três vezes no repo**:
duas linhas escrevendo o mesmo literal **não conflitam no git**, porque o git não tem opinião
sobre o que o número significa. O sinal é o conflito no `project_schema_tests.rs` ao lado.

**Não há:** `NodeId(NNN)` numérico novo · chave i18n nova · token novo · id de gizmo novo.

**`Cargo.toml` tocados — 3, todos arestas de PATH, zero pacote externo novo:**

- `ph2d-gpu-cook` → `ph2d-node-source-object` em **`[dev-dependencies]`** (só o gate de paridade;
  o `src/` não o usa ⇒ **machete-safe**, o padrão das 5 crates-nó de 23/07);
- `ph2d-node-motion-color-array` → `ph2d-color` (dep real: o formato texto da paleta);
- `shells/desktop` → **o bloco `[dev-dependencies]` é o PRIMEIRO da shell** (`ph2d-ui-testkit`,
  para o censo de ALTURA medir os retângulos que o painel de fato registra).

---

## 5. Contratos congelados (§4) — **nenhum encostado**

Rodado, não auto-relatado:

```
cargo test -p ph2d-nodegraph  --test architecture_contract_surface       → 3 passed
cargo test -p ph2d-editor-core --test architecture_tool_contract_surface → 4 passed
```

`NodeOp=2` / `OpResolver=1` / `NodeManifest=8` e `Tool=12` / `RasterEditTool=5` /
`CanvasPaintTool=1` / `PanelEvent=4` intactos. **Nenhum ADR novo.** É o que os 6 canais
side-metadata do §3 compram: todo fato novo sobre um param mora no REGISTRY, nunca no manifesto.

---

## 6. ⚠️ Duas coisas que um integrador vai ler errado se este parágrafo não existir

**(a) `ph2d-editor-core/src/project.rs` parece trocar dois defaults de produto. NÃO troca.**
O diff mostra `Meters → Pixels` e `PixelArt → Smooth`, mas **só nos doc-comments**: o `impl
Default` real já dizia `Pixels` e `Smooth`, e é **byte-idêntico ao `main`** (medido). Os
comentários é que estavam mentindo. Commit `5bc53584e`, e ele é `docs(...)` de propósito.

**(b) `1735bc726 style(fmt)` é drift PRÉ-FORK**, não formatação desta wave — sete arquivos que o
`ship.sh` acusaria como vermelho latente. Se o rebase conflitar ali, o lado do `main` ganha.

---

## 7. O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **machete** — as 3 arestas novas do §4. As três são usadas; machete é quem confirma, e o caso
  de risco é o `[dev-dependencies]` da `gpu-cook` (usada só por `tests/`).
- **typos** e **fmt do repo inteiro** — inclusive o drift pré-fork do §6(b).
- **clippy `--all-targets --all-features`** — a linha rodou `--all-targets` no que tocou; a
  matriz de features não.
- **RUSTSEC / `cargo deny`** — nenhuma dep externa nova, então o risco é herdado do `main`.

---

## 8. Ordem, dependências e o que smoke-testar

**Ordem:** os 39 commits são sequenciais e o rebase deve preservá-los. O cluster **(A)** (GPU do
objeto) é **independente** de (B)/(C)/(D); (B) → (B2) → (C) → (D) compartilham o painel `motion-params` e
o bridge, então **não reordene**.

**Smokes (todos `--release`, da worktree):**

| Cena | Comando | Estado |
|---|---|---|
| Unidades nos params | `env PH2D_UNITS_SMOKE=1 cargo run -p ph2d-host-desktop --release` | aprovado |
| Régua do oscilador / loop | `env PH2D_OSC_RULER_SMOKE=1 …` | aprovado |
| Objeto/vetor vivo na GPU | `env PH2D_MOTION_OBJ_SMOKE=1 …` | aprovado |
| Caminho de nós | `env PH2D_MOTION_NODE_PATH_SMOKE=1 …` | aprovado |
| **Row dirigida** | `env PH2D_DRIVEN_ROW_SMOKE=1 …` | **aprovado 2026-08-07** |
| **Slider dual** | a MESMA cena: `Grid` → caixa **Rows** → digite `5000` → Enter | **aprovado 2026-08-08** |

⚠️ **O slider dual se julga digitando, nunca arrastando.** O número tem de **FICAR** (a fileira
cresce para 5.000 e o thumb estaciona em 20, que é a ponta da faixa confortável). E o controle da
outra ponta é o **`Scatter`**: ali digitar `50000` ainda clampa em **3.000**, porque aquele teto é
um RECURSO medido (o quadro quebra entre 3.000 e 4.000), não ergonomia.

⚠️ **A cena da row dirigida imprime o que montou.** Se a linha `[driven-row smoke]` não aparecer,
pare: o resto do smoke não diz nada.

⚠️ **O que MUDA para quem abre o app e não roda smoke nenhum: o editor de Motion abre VAZIO.**
A neve (`motion_demo_strobe`) saiu do boot por ordem do Enio (*"tire a cena da cachoeira"*) e virou
fixture `#[cfg(test)]` — ela **não foi deletada**, e `MotionState::with_snow()` é a porta única que
a monta para os gates que dependiam dela. Um integrador que abrir o app e vir tela vazia está
vendo o produto correto.

---

## 9. Gate de fechamento (rodado nesta worktree)

- **`cargo test --workspace` → 12.851 passed / 0 failed**
- `cargo fmt --check` nas crates tocadas → limpo (exit 0)
- `cargo clippy --all-targets` na shell + nas 7 crates-nó da varredura → limpo
- `cargo test -p ph2d-host-desktop` → 2.439 passed / 0 failed
- LOC: todo arquivo tocado sob o teto (o maior é `verlet_rope/lib.rs` em **652**;
  `motion_count_ceiling_tests.rs` 282 · `motion_bridge_range_tests.rs` 348).

⚠️ **Rode a suíte, não o `cargo check`.** Esta wave deu verde no `cargo check -p` sobre **dois
erros de compilação** — ele não compila código `#[cfg(test)]`, e a sonda de medição vive lá.

---

## 10. Aberto e NOMEADO (não é dívida escondida)

- **O doc 88 não fechou inteiro, mas o A1 (o slider dual) FECHOU** — mecanismo, produto e
  varredura. Onze nós de contagem carregam teto **medido**: `motion.grid` rows/cols ·
  `motion.fibonacci` · `motion.distribute_radial` · `motion.distribute_curve` → **1.000.000** ·
  `motion.pin_constraint` first/count → **1.000.000** (o eixo é PLANO: 0,534 / 0,490 / 0,516 ms) ·
  `motion.clone` → **10.000 CÓPIAS** (1 M instâncias, 4,05 ms) · `motion.verlet_rope` → **50.000**
  (linear) · `motion.scatter` → **3.000** e `motion.boids` → **2.000** (os dois O(n²)) ·
  `motion.lattice` → **400** e `motion.kaleidoscope` → **256** (ali o teto **é o clamp do kernel**).
- ⚠️ **Quatro nós ficam de FORA da varredura, cada um com o motivo escrito:** `motion.wave` e
  `motion.soft_body` já têm **soft == clamp do kernel** (60 e 512), então não há folga a abrir sem
  mexer no KERNEL — o que é wave própria, com a medição de `rows × cols` ao lado; e o `steps` de `field.remap` e
  `value.pattern` **não é uma contagem que carrega recurso** — no `value.pattern` ele é
  estruturalmente limitado pelos oito slots declarados (`max: SLOTS`), e no `field.remap` é uma
  quantização do falloff, custo O(1) por elemento. *Um teto sobre param que não carrega recurso
  seria cerimônia; e no `value.pattern` o soft JÁ é o limite estrutural.*
- ⚠️ **`motion.voronoi` fica NOMEADO e não capado**, de propósito: o soft dele é **165.000** (medido
  pela linha da GPU) e o cook de **CPU** já passa o quadro em ~1.500 (8.000 = 259 ms). Derivar um
  teto do caminho de REFERÊNCIA seria deixar o mais lento definir o teto do mais rápido — o erro
  que o §0 nomeia. Quem quiser fechar isto precisa medir o **device**, não a CPU.
- ⚠️ **E a sonda de contagem mentiu na 1ª versão** — vale ler antes de escrever a próxima: ela dava
  `0,00 ms` em toda célula enquanto o teste levava **1402 s**, porque o `Cook` **memoiza** e eu
  descartava a 1ª corrida (a cautela de *first-touch*). *A lição certa noutro lugar cega o
  instrumento*, e quem denunciou foi o relógio de parede. Ela hoje carrega um CONTROLE próprio.
- **O `value.gain` da cena de smoke ensina uma armadilha real e vale reler:** ele opera em `[0,1]`
  e **clampa**, então alimentá-lo fora da banda o torna mudo (a cena v1 fazia isso e o fio ficou
  inerte com a suíte verde). Quem for construir cena com ele: `map_range` antes e depois, como a
  doc do nó prescreve.
- ⚠️ **A lição de gate desta wave, para o integrador não repetir:**
  `the_wire_actually_moves_the_scene` media a extensão em Y — que só o fio da AMPLITUDE move — e
  ficou **verde sobre um fio de frequência morto**. *Um gate que mede uma metade fica verde sobre
  a outra morta.* Os dois gates que faltavam existem agora
  (`the_frequency_wire_walks_over_the_cycle`, `the_drivers_knob_steers_the_wire`).

---

- ⚠️ **A varredura B3 fechou UMA família (TRANSFORM) e o mapa das demais está na §9 do
  doc 88, com o veredito de cada uma** — inclusive as três **recusadas com motivo**
  (VALUE, ESTRUTURAIS, RIG), que existem para ninguém as "completar" por parecerem magras.
  **Duas fechadas** (TRANSFORM · ECHO); a próxima aberta é **DEFORMERS** (`spherize` e
  `slit_scan` com 1 param cada — a medir se são magros por natureza).
- ⚠️ **E o `motion.duplicator` tem ZERO params, o que está QUASE todo certo** — a tabela
  do Cavalry é grande e a leitura ingênua ("faltam sete params") é falsa: *Distribution*
  são 21 nós aqui, os transforms por-cópia são `motion.move`/`rotate`/`scale` a jusante, e
  *Skip Invisible* é o `motion.cull`. O gap REAL é **um**: *que forma vai em que ponto*
  (nós fazemos o produto cartesiano; a referência cicla ou fixa por id) — semântica de
  contagem, não um knob, ⇒ wave própria.

---

**Resumo:** linha `motion-value` pronta (HEAD `dc8ac591e`, 48 commits). Foundational tocado é
aditivo salvo os doc-comments do §6(a); símbolos colidíveis são `PROJECT_SCHEMA = 56`
(**provisório**), `INSPECTOR_MAX_H`, `RESERVED_PREFIX` e `CURSOR`; contratos congelados **3/3 +
4/4 verdes**; zero pacote externo novo, zero crate nova, zero ADR. **Aguardo ordem de integração.**
