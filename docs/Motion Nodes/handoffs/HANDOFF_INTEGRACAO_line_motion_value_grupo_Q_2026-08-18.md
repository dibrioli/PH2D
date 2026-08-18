# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, **GRUPO Q: a folha 06 fica com QUATRO P1**

**Data:** 2026-08-18 · **Linha:** `line/motion-value` · **Worktree:** `Worktrees/line-motion-value`
**Commits desta wave:** `11130adcd` · `75c8bb5dd` · `d74360575` · `103f6a3d0` · `a060eacd6`
**Cena de smoke:** `PH2D_GPU_COOK_DEMO=58` · **Sondas:** `measure_wave_edges` · `measure_preroll` ·
`measure_size_axes` · `measure_curve_lut`

> ⚠️ **A linha NÃO integra e NÃO pusha.** Ordem explícita do Enio ⇒ agente integrador
> dedicado (DIRETRIZ §1.5.3). Este documento é o que ele precisa ter na mão.

---

## §1 — O que a wave é, numa frase

Os **três P1 restantes da folha 06 que não pedem nó novo** — e a medição separou o que
já era exprimível do que era vão real: **uma célula ENVELHECEU inteira** (a nona desta
conferência), **um pedido foi refutado com número** e **dois vãos fecharam**.

| linha | pedido | veredito | evidência |
|---|---|---|---|
| **36** | Reflect Edges · Pre-roll · falloff · Narrowness | **P2 (três) + ⛔ NATUREZA (o Pre-roll)** | `measure_wave_edges` · `measure_preroll` |
| **39** | `Size X ≠ Size Y` | ✅ **FECHADO** (os canais 10 e 11) | `measure_size_axes` |
| **45** | curva arbitrária tempo→tempo | ✅ **FECHADO** (o 6º modo `Curve`) | `measure_curve_lut` |

**Placar da folha 06, DERIVADO** (`placar_conferencia.py`): **P1 7 → 4** · P2 17 → 18 ·
✅ 6 → 8. Placar da conferência inteira: **P0 = 0 · P0/P1 = 0 · P1 = 64**.

---

## §2 — A célula 36 envelheceu, e o Pre-roll foi CONSTRUÍDO e REVERTIDO

A justificativa escrita (*"todos vivem dentro do kernel"*) é **verdadeira sobre o KERNEL
e não responde à pergunta da conferência**, que é sobre o CATÁLOGO.

- **Narrowness/Width é o `period` da fonte.** Num PDE `λ = c/f`, e a frequência é a da
  `value.lfo` que o artista já liga na porta `drive`. Medido numa grade **61×61 antes de
  a frente voltar da parede** (a 21×21 ela volta em ~17 tiques e o raio passa a medir
  INTERFERÊNCIA): meia-onda média **0,837 · 1,299 · 1,813 · 2,962** para os períodos
  0,10 · 0,15 · 0,20 · 0,30 — monótona e quase linear, com a subida da razão sendo a
  **dispersão numérica** da grade discreta.
- **O decaimento espacial e a borda absorvente são a MESMA coisa** — um amortecimento que
  varia no espaço — e saem da **máscara do próprio drive**: com o alvo em zero, a mistura
  `h*(1−f) + alvo*f` é literalmente `h *= (1 − f)` por tique. Com `field.box(invert)` a
  máscara vale **zero EXACTO** no miolo e a moldura de duas células vai de **0,1872 para
  0,0000**.
- **O CONTROLE mede o fenômeno que isso dissolve:** com a borda de hoje e `damping = 0`,
  um pulso único deixa a energia a **oscilar sem cair** (130 → 775 → 303 → 565 em
  30/60/120/240 tiques) — a assinatura de uma caixa fechada.
- ⚠️ **O `wave_prev` tem de levar a mesma esponja:** amortecer só metade do par do
  leapfrog custa 4× a energia do campo (0,1466 contra 0,6343).

### ⛔ O Pre-roll: construído, medido, revertido

O param **foi escrito** (`preroll`, com cap e tudo) e a medição o matou. A porta `drive`
entrega **UM número por tique, não uma função do tempo**: durante um pre-roll o nó não
pode re-avaliar a fonte a montante em instantes fictícios, então o valor fica
**CONGELADO**. Um pre-roll de `K` passos é, ao bit, **`K` tiques com fonte constante** —
e uma fonte constante constrói um **DOMO PARADO**:

| fonte congelada | 30 passos | 120 | 500 | cruzamentos de zero |
|---|---|---|---|---|
| **0,0** (uma senoide no instante da semeadura) | 0,000000 | 0,000000 | 0,000000 | **0** |
| 0,5 | 0,500000 | 0,500000 | 0,500000 | **0** |
| 1,0 | 1,000000 | 1,000000 | 1,000000 | **0** |
| **CONTROLE — fonte VIVA** | 0,9849 | 0,9849 | — | **4** |

No caso comum o knob é **inerte** (uma senoide vale zero em `t = 0`), e o AE consegue
porque os *Producers* dele são paramétricos **DENTRO** do efeito. A nossa fonte é um
**FIO** — estritamente mais geral (é dela que a composição de N produtores depende) — e o
preço é não ter passado. ⇒ **natureza, não omissão**, e a sonda fica no repo para ninguém
reconstruir o param.

---

## §3 — Os DOIS eixos do tamanho (linha 39)

⚠️ **Metade do pedido já era exprimível, e a medição separou as duas:**

| rota | pior `\|x−y\|` | o que ela é |
|---|---|---|
| CONTROLE `drive(Size)` | **0,000000** | o mundo de antes |
| `drive(Size) → motion.scale(2,0 / 0,5)` | **1,500000** | anisotropia **FIXA** (razão 4:1 em toda peça) sobre magnitude DIRIGIDA |
| `drive(Custom "size")` | **0,000000** | recusado: um `Scalar` sobre um `Vec2` mudaria o TIPO |

⇒ o squash-and-stretch inteiro já sai em dois nós; **dois campos INDEPENDENTES** não saem
de lado nenhum. Entram **`Size X` (10)** e **`Size Y` (11)**.

- **Apendados depois do `CH_CUSTOM`**, pela lei que ele próprio escreveu: o `channel` é um
  param que o **documento GUARDA**.
- O braço é o **espelho exacto do `0 | 1` do `P`**; no device é **UM kernel a ramificar em
  `params.channel`** (o molde do `DRIVE_P`), porque os dois escrevem a MESMA coluna com a
  MESMA binding.
- **Paridade na RTX: 1600 instâncias, `max |Δ| = 0e0`**, com os dois eixos dirigidos por
  campos `Random` de sementes diferentes.
- ⚠️ **A tabela de unidades do shell ENUMERA canais** — o par cairia no `_` e leria unidade
  nenhuma, a mesma família que a faixa por-canal pagou em 14/08 —, então os três canais de
  tamanho entram no mesmo braço, com gate a afirmar que **concordam** e um CONTROLE de que
  a tabela ainda discrimina.

### ⚠️ E uma mutação achou um buraco que NÃO é desta wave

Apagar um rótulo do menu deixa a suíte inteira VERDE e o canal do fim **inalcançável**. E o
`max` do hint **não** é quem o guarda: **medido, a row de enum do painel clampa por
`labels.len()` e IGNORA `min`/`max`** ⇒ para um enum aquela faixa é decorativa. Um gate
sobre a faixa **foi escrito, medido e DESCARTADO** — ele acusava **três nós corretos**
(`source.text.align`, `source.text.pivot`, `source.shape.kind`). O que ficou pina a lista
contra o **último índice implementado**.

---

## §4 — A CURVA dobra o relógio (linha 45)

**Trap 1 primeiro, e ela mede o mecanismo:** os cinco modos são fechados e compô-los dá
mapas **afins por pedaços** — a segunda diferença de qualquer cadeia deles mede
**≤ 4,5e-16** (o ULP da aritmética) contra **1,1e-4** de um ease. *É a curvatura que é
inexprimível, não o deslocamento.*

⚠️ **E o "barato" da célula estava ERRADO, o que é a §0 mordendo em casa:** o *enabler*
existe (`ph2d-curve`), mas o **CARREGADOR** não — o `TimeMap` do substrato é `Copy` e entra
na **CHAVE DO MEMO** (`push_scope` mistura os bits dele), e um `ph2d_curve::Curve` é um
`Vec<Point>` que **aloca**.

**A saída é a do `LutSpec`, e é uma DECISÃO:** viaja uma **tabela de `f32`** que a crate do
NÓ preenche — *o substrato fica agnóstico de curva*; a alternativa exacta (os pontos de
controle) poria a LEI da curva dentro do `ph2d-nodegraph`, que é a segunda resposta que
aquele precedente recusou.

**O N é MEDIDO.** O erro de uma tabela é um erro de **TEMPO** e escala com a JANELA, cujo
teto é o `duration` (o slider capa em 10 s):

| amostras | erro a 2 s | a 10 s | pior Δ de velocidade |
|---|---|---|---|
| 16 | 0,842 quadro | 4,21 | 0,443 |
| 32 | 0,215 | 1,08 | 0,226 |
| 64 | 0,054 | 0,270 | 0,113 |
| **128** | **0,0135** | **0,0675** | 0,056 |
| 256 | 0,0034 | 0,017 | 0,027 |

⚠️ **A tabela entra na chave do memo SÓ no modo que a lê** — não misturá-la no `Curve`
serviria a sub-árvore de uma curva ANTERIOR (um erro); misturá-la sempre partiria a pista
de dois `Scale` que diferem num número que ninguém consulta (um miss de graça). **As duas
metades têm gate.**

⚠️ **O gate da lei mora no SUBSTRATO, que monta o `TimeMap` à mão, logo é CEGO à fiação** —
a metade *a forma autorada chega ao mapa* é gate próprio no nó.

⚠️ **Uma barra de `== 0.0` sobre a curvatura reprovou produto CORRETO** (o `Scale` mede
4,44e-16 de arredondamento) ⇒ a régua é o **fosso**, não o zero.

---

## §5 — A superfície de colisão, MEDIDA (não auto-relatada)

| eixo | medida |
|---|---|
| `PROJECT_SCHEMA` | **84 INTOCADO** — `git diff main...HEAD -- 'shells/desktop/src/project*'` **VAZIO** (os 16 arquivos da família, conferidos por `ls` antes do diff) |
| tripla | **`(84, 13, 14)`** viva em `project_schema_tests.rs:346` |
| contrato congelado | **INTOCADO** — diff vazio em `ph2d-nodegraph/src/node.rs` e `ph2d-core/src/tool.rs`; gate `architecture_contract_surface` **3/3** |
| registro do `ph2d-ecs` | **INTOCADO** ⇒ os **três** espelhos também |
| ADR | **nenhum novo** ⇒ a linha fica **FORA de toda disputa de número** |
| `ph2d-i18n` | **INTOCADO** ⇒ a cadeia `vector::tr(k).or_else(sculpt3d::tr)` fica intacta |
| `Cargo.toml` | **2** (`ph2d-node-motion-time-remap` e `shells/desktop`, os dois ganham `ph2d-curve` — **arestas internas de path**) |
| `Cargo.lock` | **nenhum `+name`** ⇒ **nenhum pacote externo novo** |
| crates novas | **nenhuma** |
| ids | nenhum id numérico novo; os canais são índices de um param que já existia |
| scrollbar id | **nenhum novo** |
| cenas de smoke | **=58** (o roteador ia a 57; **próxima livre: 59**) |
| censo | **125 nós · 547 params · 528 com hint · 159 com unidade — INALTERADO**, e ele reconcilia: a wave acrescenta **zero `ParamSpec`** (dois índices de canal + um índice de modo + um TEXT param) |

### ⚠️ Os DOIS pontos de merge sensíveis

**(1) `TimeMap` ganhou um campo** (`curve: [f32; 128]`), e ele é construído por **literal**
em três sítios (`time.rs`, `cook_scope_tests.rs` e o `time_map_from` do nó). Uma linha que
apende **outro** campo ao mesmo struct toca os mesmos sítios; **resolver é UNIÃO** — ficar
com um lado deixa o struct com um campo que um construtor não preenche, o que **não
compila** (o modo de falha barato).

**(2) O `motion_state_demo_conferencia.rs` foi PARTIDO.** As cinco cenas da folha 06
(ANIMADORES) mudaram-se para o irmão **NOVO** `motion_state_demo_conferencia_animadores.rs`
(460 + 203 LOC), e o roteador passou a importar `demo_conferencia_animadores as animadores`.
*Uma linha que acrescente uma cena de animadores ao arquivo pai funde **LIMPO** contra um
arquivo de onde elas saíram* — o modo de falha que o corte do `project.rs` já produziu duas
vezes neste repo. ⚠️ E o corte arrastou **as consts do `rate` (=53)**, que ficaram do lado
errado na primeira tentativa e voltaram ao dono — o compilador as pegou, mas um corte
maior poderia não ter essa sorte.

---

## §6 — Gates e mutações

| onde | gates | mutações |
|---|---|---|
| `motion.drive` (`channel.rs`) | 5 novos | **6, 6 sangram** (os dois eixos na CPU · o eixo trocado no WGSL · o variant a rotear para o `DRIVE_SIZE` · a tabela do shell · o rótulo apagado) |
| `ph2d-nodegraph::time` | 4 novos | **6, 6 sangram** (o mapa ignora a forma · o braço ignora a tabela · a chave sem a tabela · a chave sempre com ela · a tabela neutra em zeros · o gate da row no modo errado) |
| a cena `=58` | 4 | **4, 4 sangram** (a semente partilhada · o canal `Size` de volta · o remap fora · a mensagem a citar um número que a cena não produz) |
| paridade na RTX | 1 | inclusa acima |

**Total: 16 mutações, 16 sangram.**

⚠️ **Três defeitos de gate/fixture, os três meus, e os três achados por medição:**

1. **O harness da cena cozia com um `Cook` cru** e o remap saiu **INERTE** (o
   `motion.time_remap` é passthrough; quem reescreve o relógio é o **puller**, a partir
   dos escopos que o `time_scopes` colhe). Agora ele passa pelo `MotionCookPump` com os
   escopos — o caminho do produto.
2. **O oráculo do relógio era a MÉDIA da fileira**, e com `phase_stagger` a média de uma
   onda a viajar é **CONSTANTE** — ele cancelava exactamente o movimento que existe para
   medir (mediu `−1,6000` em todo instante da janela).
3. **O gate de paridade usava `mode = 3.0`** num enum que só tem `0..2`. Ele passava (a
   mutação do eixo sangrava, então a fixture continha o fenômeno **por acidente do
   fallback**), e foi corrigido para `2.0` — com a mutação re-conferida depois.

⚠️ **E o CONTROLE POSITIVO do harness de mutação pegou uma âncora AMBÍGUA:** a linha
`if (dr_comp == 1) { dr_next.y = ... }` existe **duas vezes** no `motion.drive` (o `P` e o
`size`), e a primeira substituição mutou o kernel ERRADO — a mutação "passou" sobre um
produto não-mutado. O harness passou a exigir **ocorrência única** (`assert n == 1`).

---

## §7 — O que rodou

- `cargo fmt --all -- --check` **EXIT 0** na árvore inteira (⚠️ a workspace é **edition
  2024**; eu vinha chamando `rustfmt --edition 2021` e a porta certa é o `cargo fmt`, que
  lê a edição de cada crate).
- clippy `-p ph2d-nodegraph -p ph2d-node-motion-drive -p ph2d-node-motion-time-remap
  -p ph2d-node-registry-init -p ph2d-host-desktop --all-targets -- -D warnings` — **zero**.
- Suítes das crates tocadas: **zero falhas**.
- `cargo test --workspace --all-targets`: **uma** falha, e ela **NÃO é desta linha** — ver
  abaixo.
- Paridade de GPU na RTX: `the_two_size_axis_arms_match_the_cpu_on_the_device` **verde**,
  1600 instâncias, `max |Δ| = 0e0`, com o adapter **encontrado** (*skip gracioso não é
  verde*).

### ⚠️ Uma falha PRÉ-EXISTENTE, exonerada por TRÊS testemunhas

`ph2d-render --bench sprites_upload_144b_vs_72b` panica em
`assert_eq!(size_of::<RenderInstance>(), 176)` — medido **184**.

1. `git diff main...HEAD -- crates/ph2d-render/` é **VAZIO**.
2. Ele reproduz **no `main`, na árvore primária** (`adc2e3963`), com os números
   **idênticos** (184 contra 176).
3. O último a mexer no `RenderInstance` foi `d84f1f003` (*a GPU do `source.object`*), de
   outra linha.

⚠️ **Ele é um BENCH**, então só a varredura `--all-targets` o alcança — *um vermelho que só
o ship vê é invisível entre integrações*, a mesma causa estrutural que a integração de
16/08 achou em quatro arquivos do `main`. **A cura é do dono do `ph2d-render`:** ou o pino
sobe para 184 com a medição ao lado, ou o `RenderInstance` volta a 176.

---

## §8 — O smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && \
  env PH2D_GPU_COOK_DEMO=58 cargo run -p ph2d-host-desktop --release
```

⚠️ **A cena imprime as quatro bandas nomeadas; se a lista não aparecer, PARE.**
⚠️ **Ela tem DUAS naturezas de propósito:** o par **1-2 julga-se PARADO** (é FORMA) e o par
**3-4 precisa de PLAY** (é TEMPO).

1. **1-2 — ponha os olhos na RAZÃO entre largura e altura, não no tamanho.** A de cima tem
   **UMA** razão em toda peça (quadrada, pior `|x−y|` = **0,0000**); a de baixo tem **24**
   razões distintas em **25** peças (pior `|x−y|` = **1,0843**).
   ⚠️ Se as duas fileiras forem quadradas, os canais não chegaram. Se as **DUAS** tiverem
   retângulos, a cena perdeu o controle e não prova nada.
2. **3-4 — DÊ PLAY e olhe QUANDO a fileira de baixo para.** As duas oscilam com a MESMA
   amplitude de propósito (o remap reescreve o **relógio**, nunca a amplitude). A pausa
   desenhada vai de **2,4 a 3,6 s**; medido no meio dela, a de cima move **1,7815** e a de
   baixo **0,0000**.
   ⚠️ **A pausa VOLTA a cada 6 s, para sempre** — medido na TERCEIRA volta: maior passo
   **1,8243**, dentro da pausa **0,0001**. A primeira versão desta cena **clampava** a
   janela e a banda morria depois de 6 s; ver §9-bis.
3. **Abra o painel de PARAMS na banda 4:** o editor de curva só aparece no modo `Curve`.

**Controles que têm de continuar iguais:** as cenas **`=54`..`=57`** (elas mudaram de
ARQUIVO, não de comportamento) e o `PH2D_MOTION_NODE_PATH_SMOKE=1`/`=2` da wave anterior.

---

## §9-bis — A CORREÇÃO pós-smoke: `TimeMode::Curve` REPETE (foundational)

O Enio smokou a `=58` e reportou *"não há movimento usando Curve, tudo parado"*. Medido: a
banda 4 andava, mas **só nos primeiros 6 s** — a lei clampava `u = t·scale/duration` em
`[0,1]`, então depois da janela o relógio ficava cravado no último valor da curva, **para
sempre**. Os gates existentes mediam a pausa **dentro** da primeira janela: verdes sobre
produto morto.

**A cura é a lei, não a cena** — `crates/ph2d-nodegraph/src/time.rs`, uma linha:
`clamp(0.0, 1.0)` → `rem_euclid(1.0)`. Com ela o `Curve` deixa de ser uma sexta opção ao
lado dos cíclicos e passa a ser o **superset dos dois**:

| tabela | é, exactamente | gate |
|---|---|---|
| identidade (rampa) | `Loop` | `an_undrawn_curve_is_exactly_the_loop_it_generalises` (5 janelas, `< 1e-6`) |
| triangular | `PingPong`, à resolução da célula | `a_triangle_curve_is_the_pingpong_it_generalises_to_table_resolution` (a quina em `u=0,5` cai entre duas das 128 amostras ⇒ a barra é a célula, `6/127`; e um CONTROLE longe da quina pede `< 1e-6`) |
| qualquer | não expira | `no_authored_clock_expires_the_curve_still_moves_far_past_its_window` (37ª janela; **traz o controle negativo — a lei antiga escrita à mão — dentro de si**) |

**Prova de mutação:** repor o `clamp` faz sangrar **3 de 3** dos gates novos (medido).

⚠️ **É a mesma classe do `fade` do `motion.oscillator`** (cerca 1 da folha 06), construído e
removido no mesmo dia: um controle cuja unidade é *"segundos desde um zero que ninguém vê"*.
O gate irmão daquele (`no_control_of_this_oscillator_expires_with_the_clock`) guarda a classe
num nó; este guarda-a no substrato.

**Arrasto nos consumidores** (dois testes que afirmavam a lei antiga, ambos corrigidos para
a nova e **alargados para além da janela**, que é o que os fazia cegos):
`ph2d-node-motion-time-remap::the_authored_shape_reaches_the_map_the_cook_applies` (controle
passou de `Scale` para `Loop`, varredura 0..6 s sobre uma janela de 2 s) e a `=58`.

**Na cena `=58`:** `frequency` 0,35 → **0,5 Hz**, para que `FREQ · WINDOW_S` seja **inteiro**
(3 ciclos) — na volta da janela o relógio salta de 6 s para 0 como todo `Loop`, e com um
número inteiro de ciclos as duas pontas têm a mesma fase, então a emenda **some**. Gate novo:
`the_curved_band_still_moves_two_windows_after_the_first_and_still_pauses` (3ª volta: maior
passo **1,8243**, dentro da pausa **0,0001**).

---

## §9 — O que fica ABERTO

- **A folha 06 tem 4 P1**, e três deles são a **PORTA DE TEMPO** (`SUPERAR 1`): o
  `stagger` do `motion.noise`, o *Time* do `motion.oscillator` e o retiming por-INSTÂNCIA
  do `motion.time_remap`. ⚠️ **Medido nesta sessão e nomeado para a próxima:** a porta é
  mecanicamente alcançável (os kernels já leem uma coluna por-elemento — o `falloff` é
  `ColumnBinding` com identidade **materializada quando ausente**), mas a **identidade de
  uma porta de TEMPO não é uma constante** (o neutro é `ctx.playhead()`), e o `applicable`
  só enxerga **params**, nunca conectividade. As saídas medidas são três, cada uma com
  preço: a porta carregar um **OFFSET** (identidade `0`, byte-idêntica, mas o *loop por
  construção* do SUPERAR não sai dela) · um **sentinela NaN** na identidade (o kernel
  detecta e cai no `params.playhead`) · ou um **param de opt-in** com `applicable`
  recusando o device (o precedente do `Custom…`). **Decisão de desenho, não trabalho
  mecânico.**
- O 4º P1 é o `motion.noise` **transform do CAMPO** (rotation/scale do espaço do ruído).
- ⛔ **O GESTO das três metades P2 da célula 36** — a cadeia da esponja são quatro nós e o
  nome de uma coluna de **ESTADO** que nenhum picker oferece. Fechar isso é **UI** (um
  preset de cadeia, ou o picker aprender colunas de estado), não motor — e é o mesmo
  aberto que a wave `=57` deixou.
- ⛔ **O bench do `ph2d-render`** (§7) é do dono daquela crate.
