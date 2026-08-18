# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, **GRUPO Q + a PORTA DE TEMPO: a folha 06 fica com UM P1**

**Data:** 2026-08-18 · **Linha:** `line/motion-value` · **Worktree:** `Worktrees/line-motion-value`
**Commits desta wave:** `11130adcd` · `75c8bb5dd` · `d74360575` · `103f6a3d0` · `a060eacd6`
· `699895771` (§9-bis, o relógio que expirava) · `3fd0f29d6` (§9-ter, a porta de tempo)
**Cenas de smoke:** `PH2D_GPU_COOK_DEMO=59` (a porta) e `=58` (re-smoke pós-correção)
**Sondas:** `measure_wave_edges` · `measure_preroll` · `measure_size_axes` · `measure_curve_lut`

> ⚠️ **A ordem de leitura NÃO é a numérica.** As §1..§8 são o grupo Q como ele fechou;
> **§9-bis** é a correção que o smoke do Enio forçou (foundational, `ph2d-nodegraph`) e
> **§9-ter** é a wave da porta de tempo, que veio depois. Quem integra precisa das três.

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
| **23 · 28 · 44** | a **PORTA DE TEMPO** (`SUPERAR 1`) | ✅ **FECHADAS** — §9-ter | `time_port.rs` + paridade GPU |

**Placar da folha 06, DERIVADO** (`placar_conferencia.py`): **P1 7 → 1** · P2 17 → 18 ·
✅ 6 → 11. Placar da conferência inteira: **P0 = 0 · P0/P1 = 0 · P1 = 61**.

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

### §7-bis — O batch de FECHO da linha (depois de §9-bis e §9-ter)

- `cargo fmt --all -- --check` **EXIT 0** · clippy `--all-targets --workspace` **zero**.
- `cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast`, depois de tudo
  (§9-bis + §9-ter + §9-quater): **16 380 testes, 16 380 verdes, `EXIT 0`**.
  ⚠️ **Uma corrida ANTERIOR marcou duas falhas, e as duas eram CARGA, não código:**
  `ph2d-timeline::the_cost_of_depth_is_linear_not_explosive` (a flake **nomeada no
  CLAUDE.md §5** desta exacta classe) e
  `ph2d-host-desktop::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` — aquela
  corrida partilhou a máquina com duas suítes de GPU, com `load average` em **14,8**, e as
  duas passam sozinhas e passaram nesta. O CLAUDE.md §5.0 já diz que *nenhuma leitura de
  relógio desta workstation vale nada acima de `load ~5`*; fica o registo de que **a suíte
  inteira é um desses relógios**.
- Paridade de GPU, suíte `--ignored` inteira: **76 de 77 verdes** em `gpu_cpu_parity` e
  **29/29** em `gpu_cpu_parity_sim`. A única vermelha é
  `value_slope_kernel_matches_the_cpu_on_the_device`, **PRÉ-EXISTENTE** e fora desta linha.
- **LOC (HR-18):** o gate `workspace_src_files_under_loc_cap` apanhou `noise/lib.rs` (706) e
  `wiggle/lib.rs` (702) depois da porta. **Split para o irmão**, na costura que o
  `motion.oscillator` já tinha (`gpu.rs` no wiggle, `params_ui.rs` no noise) — **nada de
  allowlist**. Ficaram em 464 e 565.

---

## §8 — O smoke

**São DOIS.** O `=59` é a wave da porta de tempo; o `=58` é o re-smoke da wave anterior,
depois da correção do §9-bis (o relógio que expirava).

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && \
  env PH2D_GPU_COOK_DEMO=59 cargo run -p ph2d-host-desktop --release
```

⚠️ **DÊ PLAY — as quatro bandas só existem em movimento.** A cena imprime os quatro rótulos;
se a lista não aparecer, PARE.

1. **A de cima é UMA BARRA** (a fileira inteira sobe e desce junta) e a **segunda é uma ONDA
   QUE VIAJA**. O nó é o mesmo e os knobs são os mesmos — muda **um fio**. Se as duas
   ondularem, a cena perdeu o controle e não prova nada.
2. **A terceira é um bloco**, e a animação sai **do meio para fora** como uma ondulação. A
   leitura é *as peças à mesma distância do centro andam JUNTAS* — inclusive em cantos
   opostos, que é o que faz o relógio um CAMPO e não uma defasagem por índice.
3. **A quarta vai e volta e não deriva**, para sempre. Fique olhando um minuto: ela tem de
   continuar exactamente no mesmo laço.
4. **Puxe o fio da porta `time`** de qualquer banda: ela vira a banda 1. Desligada, a porta é
   o relógio global — byte-idêntico ao que o nó sempre fez.

⚠️ Se a banda 4 **congelar** em vez de vaivém, é o defeito do §9-bis a voltar.

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

## §9-ter — A PORTA DE TEMPO (`SUPERAR 1` da folha 06) — três P1 de uma vez

Uma porta `time` **opcional**, de tipo VALUE, em `motion.oscillator` · `motion.noise` ·
`motion.wiggle`. Desligada ⇒ `ctx.playhead()`, **bit-a-bit**. Ligada ⇒ **um relógio por
elemento**, e o relógio pode ser qualquer stream de valor.

### O que a medição mudou no orçamento

O §9 desta linha tinha orçado **três saídas, todas com preço** (offset · sentinela NaN ·
opt-in com `applicable`), porque *"a identidade de uma porta de TEMPO não é uma constante e
o seletor de variante só enxerga params"*. As duas premissas são **verdadeiras** e a
conclusão era **falsa**: o substrato já tinha o canal certo.

| a pergunta | quem já a responde | onde |
|---|---|---|
| *"a porta está ligada?"* | `const HAS_<porta>_<col>: bool`, emitido pelo codegen ao lado de cada leitor — **fixo por pipeline compilado** (a cache é chaveada na assinatura), logo ramificar nele é de graça | `ph2d-gpu-cook/src/codegen.rs` |
| *"1 valor ou N?"* | `ColumnAccess::ReadBroadcast` (a regra 1→N do `motion.drive`, doc 12) | `ph2d-nodegraph/src/gpu.rs` |
| *"uma 2ª porta VALUE cabe num nó com kernel?"* | o `motion.drive` já é isso desde sempre — `read_in_P` / `read_value_v` | `ph2d-node-motion-drive` |
| *"o planeador aceita porta 1 vazia?"* | `GpuSource::Empty` | `ph2d-gpu-cook/src/plan.rs` |

⇒ **`VariantFn`, `GpuKernel` e `ph2d-nodegraph` ficaram INTOCADOS.** O diff é 3 crates-folha
+ os gates. *A lei: antes de orçar um mecanismo novo, meça se o substrato já o exprime — o
TRAP 1 da conferência vale para a foundation, não só para o catálogo.*

### O que mudou em cada nó

- **Manifesto:** `PortSpec { name: "time", ty: VALUE }` **apendado** no índice 1 — aresta de
  documento salvo guarda o índice, então a porta 0 continua a 0 e um doc de ontem abre igual.
- **CPU:** `clock_at(times, i, playhead)` em cada `channel.rs` (cópia por drop-crate, como o
  `falloff_at`): `0 ⇒ playhead` · `1 ⇒ broadcast` · `N ⇒ por elemento`. ⚠️ **Ausente não é
  zero — é o relógio global**; um `0.0` cravado congelaria a behaviour no instante zero.
- **GPU:** um binding `v`/`ReadBroadcast`/porta 1 em **todo** variant, mais
  `fn <ns>_time(i) { if (HAS_time_v) { return read_time_v(i); } return params.playhead; }`.
  Os leitores da porta 0 passaram a ser qualificados (`read_in_P`, `read_in_falloff`, …),
  que é o que o codegen faz assim que o nó tem 2 entradas.
- **`noise` e `wiggle`:** o wrap do `loop_len` (`ph2d_fbm::loop_times`) mudou-se para **dentro**
  do laço, nos dois lados — com a porta ligada cada elemento fecha o **próprio** ciclo; sem
  ela os `n` cálculos partem do mesmo número e dão o mesmo resultado.

### Os gates (7 novos) e as 5 mutações

`crates/ph2d-node-registry-init/tests/time_port.rs`, os três nós num laço:

| gate | o que prende |
|---|---|
| `an_unconnected_time_port_is_bit_identical_to_a_neutral_clock` | `==` sobre `f32`, em 3 instantes. O oráculo é o próprio catálogo: um `value.time` neutro ligado à porta tem de dar o MESMO que porta nenhuma |
| `a_staggered_time_field_gives_each_element_its_own_clock` | + **o CONTROLE**: com `phase_stagger = 0` a fileira sem porta tem UM valor — sem isso, *"as peças diferem"* provaria o knob que o nó já tinha |
| `a_wrapped_ramp_closes_the_cycle_exactly_where_a_cross_fade_only_approximates` | `value.time → value.wrap(Repeat) → time`: `y(t) == y(t+L)`, e o controle SEM wrap muda |
| `a_single_clock_value_is_held_across_every_element` | a regra 1→N, e que o relógio é o do `value.time` (`rate = 2`) e não o playhead por outra via |
| `the_time_port_is_a_column_not_a_cook_scope` | a resposta à **cerca 6**: um `motion.spring` a montante coze sem `CookError::SequentialInTimeScope` |
| `the_time_port_matches_the_cpu_on_the_device_for_every_animator` | paridade CPU×GPU **ligada**, 3 nós, com a contagem de estágios a provar que o planeador não recuou |
| `an_unconnected_time_port_still_matches_the_cpu_on_the_device` | paridade **desligada** — o caminho que TODO documento existente percorre |

Paridade medida (adapter local): pior `|Δpos|` **7,6e-5 · 1,2e-5 · 9,1e-6** ligada e
**7,5e-5 · 9,5e-7 · 4,8e-7** desligada, em 1600 instâncias.

**Mutações — 5 lançadas, 5 sangram:** `clock_at` a ignorar o campo · sem broadcast ·
ausente⇒`0.0` · `HAS_time_v`⇒`false` · `read_time_v(0u)` (bug de broadcast).

### A cena `=59` — «o relógio é um campo»

Quatro bandas, e **entre uma e a seguinte muda UM FIO**; o nó e os knobs são os mesmos.
1. CONTROLE (porta desligada): a fileira é **uma barra** (1 altura medida em 21 peças).
2. `value.time(stagger)` → **onda que viaja** (21 alturas distintas).
3. Um bloco 9×9 com relógio `t + |P|` → **ondulação radial**. O gate prova que o relógio é
   função do **RAIO** e não do índice (quinas opostas partilham o instante a `< 1e-5`).
4. `value.wrap(Mirror)` → o ciclo fecha: resíduo **1,9e-6** a uma volta e **7,6e-6** a dez
   (não cresce), contra **1,80** de deriva da banda 2.

⚠️ **Dois números da cena são MEDIDOS e o comentário diz porquê:** `STAGGER = 0,24` (com
`0,25` o passo é `1/8` exacto e a fileira exibe só **8** alturas — um carimbo) e
`WRAP_S = 2,5` (com `3,0` o período eram 3 ciclos exactos e o **controle** do gate também se
repetia, medindo 1,9e-6 contra 7,6e-6 — a cena teria "provado" o que a aritmética já dava).

---

## §9-quater — A POSIÇÃO não tinha chip, e o `Custom…` não a alcançava

**Veio de uma pergunta do Enio a olhar o painel da cena `=59`:** *"em Custom temos um P —
de onde vem esse P e o que significa?"*. `P` é a coluna de POSIÇÃO do stream (o `(x, y)` de
cada peça, nome herdado do Houdini) — e a resposta completa é que ela **não tinha entrada no
picker do `value.attribute`**, então eu tive de a digitar à mão no `Custom…`.

⚠️ **E o `Custom…` NÃO a alcança.** O picker de coluna viva escreve o nome **com o modo 0**
(escalar), e uma `Vec2` em modo 0 cai no `_` da escada do `field()`: **zeros no comprimento
cheio**, em silêncio, indistinguível de um nome mal digitado. A cena `=59` só funciona porque
o construtor escreve `mode = 1` em código. *O valor existia no modelo, o cook o lia, e não
havia gesto que chegasse lá* — a frase que o `snapshot_ids.rs` já tinha escrito para outro
caso.

⚠️ **A célula 121 da folha 15 marcava isto ✅ FECHADO**, e estava certa **sobre o mecanismo**:
o degrau `MODE_COMPONENT_BASE` existe desde 12/08. O que ninguém reconferiu é que um degrau
sem chip **não é alcançável**. *Um ✅ de mecanismo e um ✅ de artista leem igual numa tabela.*

**Quatro chips**, as duas leituras de um vetor nas duas bases:
`Position X` · `Position Y` (lanes) · `Radius` · `Angle` (polar, em torno da origem do mundo —
o rótulo diz a referência, que um `Distance` esconderia).

**Trap 1, medido antes de escrever:** as lanes a composição **não dá**; o `Radius` daria em
**seis** nós; o `Angle` é **inexprimível** (sem `atan2` no domínio de valor, `ph2d-expr`
FROZEN). ⇒ nenhum dos quatro é conveniência.

**Gates (3 novos, 2 mutações, 2 sangram):** o oráculo é o triângulo 3-4-5 · o gate do
MECANISMO (`a_vec2_column_is_unreachable_without_a_picker_entry`: o modo que o `Custom`
escreve dá zeros, a entrada dá o número) · e o gate de **CLASSE**
(`no_entry_reads_a_vec2_column_in_the_scalar_mode`, com controle positivo) — *uma coluna
`Vec2` sem entrada no picker é inalcançável pelo artista, por mais que o cook a leia*.

**Fica P2, medido e nomeado na folha 15:** as lanes de `vel` e `size` também não existem
(elas têm reach polar, então não é o silêncio que o `P` tinha) — ⚠️ e essa é uma assimetria
que **esta mesma jornada** criou do outro lado: o `motion.drive` ganhou `Size X`/`Size Y` e o
valor não lê de volta por eixo. Quem as acrescentar orça a **altura** da row (o picker já paga
5 linhas de chips), não só o cap de 48.

### ⚠️ E o placar tinha uma exceção por NÚMERO DE LINHA

Acrescentar uma linha à folha 15 **desalinhou o `placar_conferencia.py` em silêncio**: a
tabela `HAND` das cinco linhas cujo veredito não está na coluna `P` era chaveada por
`(arquivo, nº)`, a exceção da linha 141 caiu sobre a vizinha, e o placar imprimiu **um ✅ a
menos** — o único sintoma foi um `!!` numa linha que ninguém tinha tocado.

A chave passou a ser um **TRECHO DA LINHA**, que viaja com ela. E a troca só é segura porque
se **verifica**: cada exceção tem de casar **exactamente uma** linha da sua folha, senão a
ferramenta sai **vermelha** com o nome da chave (mutação: uma chave morta ⇒ `EXIT 1`).
*Um número de linha é uma referência que o próprio ato de editar invalida.*

**Placar depois de tudo:** folha 15 **P1 2 · P2 19 · ✅ 31**; TOTAL **P0 0 · P1 61 · P2 123 ·
✅ 126**, `EXIT 0`.

---

## §9 — O que fica ABERTO

- ✅ **A PORTA DE TEMPO foi construída** — ver §9-ter. Os três P1 que ela fecha eram o
  `stagger` do `motion.noise`, o *Time* do `motion.oscillator` e o retiming por-INSTÂNCIA
  do `motion.time_remap`. **A folha 06 passou de 4 P1 para 1.**
- O P1 que RESTA é o `motion.noise` **transform do CAMPO** (rotation/scale do espaço do
  ruído — o `offset` já sai de `motion.move(+d) → noise → motion.move(−d)`).
- ⛔ **O GESTO das três metades P2 da célula 36** — a cadeia da esponja são quatro nós e o
  nome de uma coluna de **ESTADO** que nenhum picker oferece. Fechar isso é **UI** (um
  preset de cadeia, ou o picker aprender colunas de estado), não motor — e é o mesmo
  aberto que a wave `=57` deixou.
- ⛔ **O bench do `ph2d-render`** (§7) é do dono daquela crate.
