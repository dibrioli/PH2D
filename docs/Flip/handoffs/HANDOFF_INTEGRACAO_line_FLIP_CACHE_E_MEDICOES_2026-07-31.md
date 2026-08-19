# HANDOFF DE INTEGRAÇÃO — `line/FLIP`: O CACHE DO AJUSTE E AS MEDIÇÕES QUE FECHARAM ITENS (2026-07-31)

**Status:** FECHADO 2026-07-31 · no `main` em `ed832dcaf` (o commit que trouxe este arquivo).

> ⚠️ **SUPERSEDED por [`HANDOFF_INTEGRACAO_line_FLIP_PONTAS_2026-08-01.md`](HANDOFF_INTEGRACAO_line_FLIP_PONTAS_2026-08-01.md)**,
> que é o handoff MESTRE da linha (10 commits). Este cobre só os **5 primeiros** e **não** menciona o
> bump de schema nem as pontas do traço — integrar por ele deixaria `PROJECT_SCHEMA` para trás.
> O conteúdo abaixo segue válido como DETALHE daqueles cinco.

> **Para o agente integrador.** Esta é a **continuação** da linha depois de a jornada do motor novo
> ter integrado em 2026-07-30 (handoff mestre
> [`HANDOFF_INTEGRACAO_line_FLIP_MOTOR_NOVO_2026-07-30.md`](HANDOFF_INTEGRACAO_line_FLIP_MOTOR_NOVO_2026-07-30.md)).
> São **5 commits** sobre o `main` daquele dia.
>
> ⚠️ **PENDENTE DE SMOKE.** A wave de perf (§2) tem contrato de **byte-identidade** provado por gate,
> então ela não pode mudar o desenho — mas *não pode* e *não mudou* são afirmações diferentes, e só o
> Enio fecha a segunda. As outras quatro entregas são **medição e gate**, com **zero mudança de
> produto**.

---

## 0. Identificação

| | |
|---|---|
| branch | `line/FLIP` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP` |
| tip | `02c4baeb2` |
| commits | **5** |
| base | `main` de 2026-07-30 (pós-integração do motor novo) |
| diff | **13 arquivos, +1131 / −111** |
| `PROJECT_SCHEMA` | **46** — INTOCADO |
| `FLIP_SCHEMA_VERSION` | **12** — INTOCADO |
| contrato congelado | **intacto** (`Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4`) |
| `Cargo.toml` / `Cargo.lock` | **ZERO tocados** — nenhuma dep, nenhuma crate, nenhum ADR |

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP
git log --oneline main..HEAD
git diff --stat main..HEAD
```

**Os cinco commits:**

| | |
|---|---|
| `c2267f00d` | **perf** — o ajuste do preview guarda a resposta: quadro **2,01 → 0,12 ms** a 9000 amostras |
| `d8fe36d42` | **measure** — o cache de tiles de MUNDO cobra uma decisão de CÂMERA, e o número está medido |
| `f4b3c5b57` | **fix** — a caneta NÃO chega, e dois gates de relógio flakavam sob carga |
| `4fead3e20` | **measure** — o item 3c FECHA: o resíduo de quina é MENOR que o erro da lei que o curaria |
| `02c4baeb2` | **measure** — a 3ª lei DESLIGA o Self Overlap: a segunda metade do preço do item 5 |

---

## 1. O que a linha entrega, em uma frase cada

1. **Um ganho de perf real** no único número que o artista sente enquanto desenha (§2).
2. **Um item aberto FECHADO por medição** — o resíduo de quina (§4).
3. **Um item aberto COMPLETADO** — a terceira lei agora tem o preço inteiro (§5).
4. **Uma frente re-precificada** — o cache de tiles de mundo cobra uma decisão de câmera (§3).
5. **Uma nota falsa MORTA e substituída por sentinela executável** — a caneta (§6).

⚠️ **Só o item 1 muda código de produto.** Os outros quatro são medição, documentação e gate.

---

## 2. `c2267f00d` — O AJUSTE DO PREVIEW GUARDA A RESPOSTA (2,01 → 0,12 ms)

### O número que estava aberto

O handoff mestre deixou escrito: *"cache **incremental** do ajuste (o frame do preview custa 0,33 ms
a 1200 amostras e **2,42 ms a 9000**, 95% é o ajuste)"*. Um traço longo e lento acumula milhares de
amostras, e **cada frame re-ajustava o traço inteiro desde o começo**.

**Medido pela porta do produto** (`flip_preview_data`, o caminho do artista):

| amostras | antes | depois |
|---|---|---|
| 1 200 | 0,33 ms | **0,04 ms** |
| 9 000 | **2,014 ms** | **0,117 ms** (17×) |

### Por que isto era POSSÍVEL agora e não antes

A jornada anterior trocou o ajuste de divide-e-conquista (fila por pior erro **global**) por uma
**caminhada da esquerda para a direita**. Aquela mudança foi feita por outro motivo — *o começo do
traço parava de tremer* —, e ela entrega de graça a propriedade que um cache exige:

> **Estabilidade de prefixo:** `fit(amostras[..n])` é prefixo de `fit(amostras[..m])` para `m ≥ n`.

Com a decisão **global** um prefixo em cache **MENTIRIA** — cada amostra nova podia re-decidir o
começo. Este commit é, literalmente, a colheita daquela wave.

### O desenho: VERIFICAR, nunca PROMETER

`FitCache` guarda a **entrada exata** do último ajuste e mede o prefixo comum **bit a bit**
(`f32::to_bits()`), em vez de guardar `n` e confiar em *append-only*.

⚠️ **E isto não é paranoia — é obrigatório**, porque a entrada do ajuste é o array **SUAVIZADO**, e
`active_smooth` **reescreve a cauda** a cada amostra nova. Uma promessa de *"só cresceu"* seria
**falsa em todo frame**, e o modo de falha não é um erro: é um traço plausível decidido sobre dados
que não existem mais.

**Quantos nós são reaproveitados** sai da regra de finalidade do próprio motor — o *espelho*: a
decisão do nó `k` a partir do predecessor `a` lê índices até `2·(k+1) − a`. Um nó só é **congelado**
quando todo o alcance dele já está dentro do prefixo comum.

### O que mudou de forma pública

- `flip_fit.rs`: `fit_resumido` (a caminhada, agora retomável a partir de uma semente) + `FitCache`.
- `fit` e `simplify_to_curve` viraram **`#[cfg(test)]`** — o segundo como **oráculo congelado**
  (⚠️ um `pub(crate)` sem chamador não é código morto silencioso: é uma **segunda resposta**
  esperando alguém chamá-la — a lição do `warp_axis`/`serial_side` da `line/Painter`).
- `FlipDraw` ganhou o campo `fit` (limpo em `begin()`); `samples()` **morreu** (ficou sem chamador);
  nasceu `preview_parts()`.
- `stroke_from_samples` virou **delegação pura** para `stroke_from_samples_cached` com um cache
  descartável — ⚠️ é isso que garante que **não há duas respostas**: a rota do bake e a do preview
  executam o MESMO corpo.

### Gates (4 + 1 sonda, `flip_fit_cache_tests.rs`)

- **`the_cached_fit_is_identical_to_the_fresh_one`** — índice por índice, frame por frame, sobre o
  pipeline do PRODUTO.
- **`the_cache_actually_reuses_work`** — a premissa declarada, senão o gate de identidade fica
  **vacuoso** (um cache que nunca acerta é trivialmente idêntico).
- **`a_changed_tail_is_re_fitted`** — a suavização reescreve a cauda; o cache tem de perceber.
- **`a_new_stroke_starts_from_nothing`**.

**Mutações: 3 sangram.** ⚠️ **A quarta sobreviveu e ACUSOU O MEU DOC, não um buraco:** a condição de
fim-de-array em `congelados` é **implicada** pela condição de alcance (`a < k` ⇒ `2(k+1)−a ≥ k+3`),
logo é defesa em camada e **não é observável** — documentada como tal, em vez de vendida como gate
([[feedback_layered_defenses_need_per_layer_gates]]).

### O gate de arquitetura que foi reforçado

`tests/the_flip_preview_bakes_through_the_same_door.rs` agora afirma **as duas metades**: o preview
chega em `stroke_from_samples_cached(`, **e** o corpo de `stroke_from_samples` contém só a delegação
(nenhum `active_smooth(` / `resample_smooth(` / `build_stroke(`). Sem a segunda metade, alguém
reconstrói o pipeline dentro da porta antiga e os dois caminhos divergem em silêncio.

---

## 3. `d8fe36d42` — O CACHE DE TILES DE MUNDO COBRA UMA DECISÃO DE CÂMERA

O item *"cache em tiles de MUNDO (sobreviver ao pan)"* estava aberto sem número. Agora tem os dois
que decidem:

- **O preço do pan:** **4,77 ms por camada** a 200 traços — é ele que o cache compraria.
- **⛔ E o bloqueador:** o cache só é **EXATO sob pan de pixel INTEIRO**. Medido: pan de 1 px dá
  delta **1e-6** (ruído de formato); pan de **0,5 px** dá **0,408 em 27% dos pixels** — porque a lei
  do percurso é sub-pixel por construção (é ela que dá a borda macia).

⇒ **O item deixou de ser tarefa de engenharia e virou decisão de PRODUTO:** ou a câmera **quantiza o
pan** para pixels inteiros (mudança de política de câmera, que afeta todo módulo), ou o cache mente
em toda fração. Mais: ele mora no `ph2d-render::LayerCompositor`, **compartilhado com o Painter** ⇒
é foundational cross-line.

**Nenhum código de produto foi tocado.** A sonda
(`measure_whether_a_pan_can_reuse_the_previous_frames_pixels`, `walk_perf.rs`, `#[ignore]`) é o que
sobrevive, para o próximo não re-derivar.

---

## 4. `4fead3e20` — O ITEM 3c FECHA: O RESÍDUO É MENOR QUE O ERRO DA LEI QUE O CURARIA

O motor novo trouxe **anti-aliasing por área exata do pixel** e, com ele, um resíduo nomeado:
**13 px de 1115, ≤ 14,94/255**, nas QUINAS. O handoff mestre listava duas curas candidatas.

**Construí uma terceira e MELHOR** — amostrar o perfil no **centroide** da região coberta, em vez do
ponto médio do intervalo — e **um oráculo novo a REPROVOU**: uma referência de tinta
**supersampleada** (a verdade, não outra aproximação).

| cena | lei que SHIPA | a cura do centroide |
|---|---|---|
| zigue-zague, dureza 0,8 | média **3,39** · pior **61,86** | média 4,45 · pior **96,38** |

⇒ **A cura é PIOR** — ela lava os flancos para melhorar as junções. **Revertida inteira.**

⚠️ **E o número que FECHA o item:** o pior erro da lei que shipa contra a verdade supersampleada é
**22–62/255**, enquanto o resíduo de quina é **≤ 14,94/255**. *O artefato é MENOR que o erro da
aproximação que o curaria* — perseguí-lo é afinar ruído abaixo do piso do modelo.

⚠️ **Duas afirmações minhas foram falsificadas pela medição, e as duas ficam escritas** (§22.10 do
doc 12): eu previ que uma borda paralela a um eixo ficaria byte-idêntica sob a cura (mediu **21,48/255
de deslocamento** — as tampas têm normais diagonais), e atribuí o pior pixel a uma região coberta
desconexa (o detalhe mostrou 93% de área, 3 planos, centroide a 0,028 px do centro).

**Zero mudança de produto.** O que ficou é o oráculo supersampleado
(`measure_which_profile_sample_point_is_closer_to_the_truth`) e o parágrafo.

---

## 5. `02c4baeb2` — A 3ª LEI DESLIGA O SELF OVERLAP

O item 5 (a terceira lei, o `Soft` do Krita) tinha **metade** do preço medido: *"funciona exato, mas
muda a borda de UMA passada em +69%"*. Faltava a outra metade, e ela é maior:

| dureza | braço | cruzamento HOJE | ganho | 3ª LEI | ganho |
|---|---|---|---|---|---|
| 1,0 · 0,5 · 0,0 | 0,50 | 0,75 | **1,50×** | 0,50 | **1,00×** |

A 3ª lei limita cada pixel pela cobertura do **próprio bico** ali, e isso não capa só o endurecimento
da borda: capa **toda acumulação dentro do traço**. O **Self Overlap** — feature shipada em
2026-07-27 — é exatamente isso.

⇒ **Ela não é "um modo ao lado do que existe": é MUTUAMENTE EXCLUSIVA com uma feature shipada**, como
o One-Way e a zona de força da física. O preço do item 5 tem duas metades, e agora as duas estão
medidas. **Decisão do Enio, e nada foi construído.**

⚠️ **TRÊS defeitos de fixture, todos reportando `1,00×` sobre um motor que FUNCIONA** (§22.11):

1. a sonda lia `d.cover` para perguntar *"escureceu?"*, e o `opacity` entra **depois** dele, no alfa
   da cor (a regra do Grease Pencil: *um traço a 0,5 não escurece sobre si mesmo*);
2. ela não ligava o **`FLAG_SELF_OVERLAP`** — media o toggle **desligado** fazer o que promete;
3. o X cruzava num **VÉRTICE**, onde as passagens são contíguas e não há duas.

**O que salvou a medição foi copiar a FORMA da fixture do gate que já continha o fenômeno**
(`crossing_x` do `cover_tests.rs`), em vez de escrever uma nova.

---

## 6. `f4b3c5b57` — A CANETA NÃO CHEGA, E DOIS GATES DE RELÓGIO FLAKAVAM

### (a) A nota que dizia *"custa uma função"* era FALSA

O handoff mestre listava o caminho do tablet como *"custa **uma função**"*. **Levantado no winit
0.30**: não existe evento de caneta. `Touch{force}` é touchscreen · `TouchpadPressure` é force-touch
de trackpad da Apple · `CursorMoved` não carrega pressão · `AxisMotion` é o valuator **CRU** do
XInput2 com `AxisId` opaco (sem API para perguntar *qual* eixo é pressão nem a faixa dele) · e no
backend **Wayland** — que é o que o Enio roda — **não há nada** (`zwp_tablet_v2` não implementado).

⇒ A cura é **subir o winit** (cross-line, classe ADR) **ou** um **caminho de tablet por plataforma**
(dep nova, `unsafe`, segunda fonte de eventos). Nenhuma é uma função.

⚠️ **E a nota velha do `painter_canvas_input.rs` era pior**: prometia *"real pressure arrives on the
iPad shell"* — uma shell que **nunca existiu** (`shells/` tem só `desktop`). Reescrita com o
levantamento medido.

**A consequência virou gate** (`tests/the_desktop_shell_has_no_pen_pressure.rs`): os dois sliders do
Flip (`Min Width`/`Response`) são **inertes** na pressão que esta shell entrega — varredura de
21×21 combinações, pior desvio `< 1e-6`, porque `min + (1−min)·1^γ = 1` para todo `min` e todo `γ`.
Ele fica **VERMELHO no instante em que alguém liga a caneta**, que é exatamente o instante em que os
defaults `0,05 / 0,5` precisam ser **re-calibrados contra pressão de verdade** (eles nunca foram
exercidos). *Uma nota envelhece em silêncio; um gate não.*

### (b) Duas flakes de relógio, e a lição do EIXO

`the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` falhava ~1 em 3 corridas completas em
debug. Era **kill de wall-clock**, que mede o **PERFIL do build e a carga da máquina**, não o código
(ADR-0124). Virou **razão** — e a primeira escolha de eixo foi **MINHA e errada**:

| eixo | saudável | com o defeito | separação |
|---|---|---|---|
| **densidade** (3× pontos) | **5,06×** | 7,13× | 1,4× — inútil |
| **comprimento** (3× amostras, densidade fixa) | **2,93 / 2,98×** | **8,60 / 8,88×** | 3× |

⚠️ Meu raciocínio *"triplicar a densidade triplica o custo"* estava errado porque **o vão cresce com
a densidade**, então até o ajuste local é superlinear ali. **O eixo é parte da fixture.**

⚠️ **E a ESCALA também:** a 600/1800 amostras o lado curto custava ~1 ms, e a MESMA corrida reportava
**5,63× em debug contra 2,99× em release** — ruído, não superlinearidade. Movido para 1500/4500.

⚠️ **E o REDUTOR:** o gate irmão flakava **mesmo já sendo razão** — medições únicas a ~1 ms oscilavam
**1,54× → 3,97× em release**. Os dois passaram a tomar **min-de-3** (dispersão 2,6× → 1,05×).

---

## 7. Gates, mutações e o estado das suítes

| arquivo | o que acrescenta |
|---|---|
| `shells/desktop/src/flip_fit_cache_tests.rs` | **NOVO** — 4 gates + 1 sonda do cache |
| `shells/desktop/tests/the_desktop_shell_has_no_pen_pressure.rs` | **NOVO** — 2 gates da caneta |
| `shells/desktop/tests/the_flip_preview_bakes_through_the_same_door.rs` | reescrito: afirma as DUAS metades da porta |
| `shells/desktop/src/flip_fit_budget_tests.rs` | 2 flakes curadas (eixo + redutor) |
| `crates/ph2d-flip-render/src/ink_drop_tests.rs` | oráculo supersampleado + a sonda da 3ª lei |
| `crates/ph2d-flip-render/tests/walk_perf.rs` | a sonda do pan (`#[ignore]`) |

### Estado medido no tip (`02c4baeb2`)

| suíte | release | debug |
|---|---|---|
| `ph2d-host-desktop` | **1685 passed, 0 failed** (59 ignored) | **1685 passed, 0 failed** (59 ignored) |
| `ph2d-flip-render` | **82 passed, 0 failed** (117 ignored = GPU) | — |
| `ph2d-flip-render -- --ignored` | **117 passed, 0 failed** (na RTX) | — |

⚠️ **Rode em DEBUG E RELEASE.** Foi um gate desta família que reprovou **só em debug** (21,65 contra
1,92 ms) e produziu a política.

⚠️ **Gates de GPU do Flip são `#[ignore]` e precisam de adapter.** Sem um eles fazem *skip
gracioso*, **que não é verde** — e é por isso que a linha acima traz o número que eu de fato rodei,
não o herdado do handoff mestre:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP
cargo test -p ph2d-flip-render --release -- --ignored     # 117/117 na RTX
```

---

## 8. Smoke

**Nenhuma cena nova.** A wave de perf é invisível por contrato (byte-idêntica, gate índice-a-índice)
e as outras quatro não tocam produto. O que o Enio julga é o **smoke-mestre herdado**:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP
env PH2D_FLIP_HARDNESS_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

**A pergunta desta linha é a AUTORIA, e ela é de olho:** desenhe um traço **muito longo e lento**
(milhares de amostras — é onde o cache vale 17×) e olhe o **começo** dele enquanto a mão ainda anda.
Ele **não pode tremer, deformar nem re-decidir**. Se tremer, o cache está entregando um prefixo que
o ajuste fresco não entregaria — e o gate de identidade estaria mentindo, o que é o único modo de
falha desta wave.

Diagnóstico, se precisar: `PH2D_FLIP_NEW_ENGINE=0` (A/B contra o rasterizador antigo) ·
`PH2D_FLIP_STATS=1`.

---

## 9. Aberto — e o que é DECISÃO do Enio, não dívida

⚠️ **Quatro dos cinco itens abertos já voltaram para o Enio COM O NÚMERO.** Nenhum é trabalho
parado: são escolhas que eu não posso fazer no lugar dele.

| item | estado | o que falta |
|---|---|---|
| **4 — joins & caps** | ⛔ a premissa de *correção* foi REFUTADA (o `−64` era do rasterizador) | pergunta de **LOOK**, pura |
| **5 — a terceira lei** | ✅ preço INTEIRO medido: borda **+69%** *e* Self Overlap **1,50× → 1,00×** | decisão de **LOOK** (e ela desliga uma feature shipada) |
| **3c — resíduo de quina** | ✅ **FECHADO por medição** (§4) | nada |
| **2b — cache de tiles de mundo** | ✅ precificado: 4,77 ms/camada, exato só sob pan inteiro | decisão de **CÂMERA** + foundational cross-line |
| **caneta / tablet** | ✅ levantado e gateado (§6) | **winit bump (ADR)** ou caminho por plataforma |

E o que segue sendo engenharia de verdade, sem decisão pendente:

- **O ajuste ainda é o frame do preview**, só que 17× menor. A 9000 amostras ele custa 0,117 ms —
  já não é a fronteira, e **nenhuma medição desta linha nomeia qual é a próxima**. Um
  `PH2D_FLIP_STATS=1` sobre um traço longo é quem decide, não teoria.

---

## 10. Ordem de integração

Esta linha **não conflita com nada por construção**: zero `Cargo.toml`, zero schema, zero ADR, zero
id/token, zero contrato congelado. O diff mora em `shells/desktop/src/flip_*` +
`crates/ph2d-flip-render/src/ink_drop_tests.rs` + dois arquivos de teste novos.

O único ponto de atrito plausível é **`shells/desktop/src/flip_draw.rs`**, que é território comum de
qualquer wave do Flip — e a mudança lá é pequena e localizada (um campo, uma assinatura `&mut self`,
uma porta nova).
