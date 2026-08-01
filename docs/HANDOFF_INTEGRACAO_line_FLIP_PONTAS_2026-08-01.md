# HANDOFF DE INTEGRAÇÃO — `line/FLIP`: AS PONTAS DO TRAÇO (2026-08-01)

> **Para o agente integrador.** Este é o handoff **MESTRE** da continuação e **supersede**
> [`HANDOFF_INTEGRACAO_line_FLIP_CACHE_E_MEDICOES_2026-07-31.md`](HANDOFF_INTEGRACAO_line_FLIP_CACHE_E_MEDICOES_2026-07-31.md),
> que descreve só os **5 primeiros** commits. O conteúdo daquele segue válido como detalhe das
> §2–§6; o que ele **não** cobre é tudo o que veio depois (as pontas).
>
> **Smokes APROVADOS pelo Enio** — a ponta chata (2026-07-31), a quadrada e a borracha (2026-08-01).
> A ordem de integração é dele.

---

## 0. Identificação

| | |
|---|---|
| branch | `line/FLIP` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP` |
| tip | `e942d6bb3` |
| commits | **10** |
| base | `main` de 2026-07-30 (pós-integração do motor novo) |
| diff | **37 arquivos, +2633 / −611** |
| `main` andou desde o fork? | **NÃO** (`git rev-list --count HEAD..main` = 0) — sem rebase pendente |
| **`PROJECT_SCHEMA`** | **46 → 47** ⚠️ |
| **`FLIP_SCHEMA_VERSION`** | **12 → 13** ⚠️ |
| tripla do pin | **`(47, 13, 13)`** |
| contrato congelado | **intacto** — `architecture_tool_contract_surface` **4/4 verde** (rodado, não auto-relatado) |
| `Cargo.toml` / `Cargo.lock` | **ZERO tocados** — nenhuma dep, nenhuma crate, nenhum ADR |
| ids novos | **3** (`FLIP_CAP_ROUND` · `FLIP_CAP_FLAT` · `FLIP_CAP_SQUARE`), todos hash-de-string |

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP
git log --oneline main..HEAD
```

### ⚠️ O ÚNICO PONTO QUE EXIGE ATENÇÃO NA INTEGRAÇÃO: o número do schema

`PROJECT_SCHEMA` **47** foi contado contra o `main` de **30/07**, que dizia 46. **O valor se CONTA,
não se escolhe** — se outra linha bumpou na mesma janela, 47 está errado e o certo **não está em
nenhum dos dois lados** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). Esta linha já
colidiu com a `line/physics` **duas vezes** por isso (o 30 em 25/07, os 32/33/34 em 27/07).

Confira no `main` do dia e re-conte:

```bash
grep -n "const PROJECT_SCHEMA" shells/desktop/src/project.rs   # nas DUAS árvores
```

Os três sítios a acertar: `project.rs` (a const) · `project_schema_tests.rs` (a tripla) · a nota de
escada no `project_schema_tests.rs`.

---

## 1. O que a linha entrega

Duas metades. A primeira (§2–§6) é **perf e medição**; a segunda (§7–§10) é **produto visível**.

| | |
|---|---|
| `c2267f00d` | perf — o ajuste do preview guarda a resposta: **2,01 → 0,12 ms** a 9000 amostras |
| `d8fe36d42` | measure — o cache de tiles de mundo cobra uma decisão de CÂMERA |
| `f4b3c5b57` | fix — a caneta NÃO chega; e duas flakes de relógio |
| `4fead3e20` | measure — o item 3c FECHA por medição |
| `02c4baeb2` | measure — a 3ª lei DESLIGA o Self Overlap |
| `8121616dc` | docs — o handoff dos 5 (**superseded por este**) |
| **`6d51a9443`** | **feat — a PONTA do traço ganha PORTA** |
| **`bd88a1cbc`** | **fix — a ponta CHATA cortava só o 1º segmento** |
| **`29125dd66`** | **feat — a ponta QUADRADA (e ela é GEOMETRIA, não máscara)** |
| **`e942d6bb3`** | **fix — a borracha não faz o traço crescer para dentro do vão** |

---

## 2. `c2267f00d` — O AJUSTE DO PREVIEW GUARDA A RESPOSTA

O quadro do preview re-ajustava o traço inteiro desde o começo, todo frame.

| amostras | antes | depois |
|---|---|---|
| 1 200 | 0,33 ms | **0,04 ms** |
| 9 000 | **2,014 ms** | **0,117 ms** (17×) |

**Isto só era possível agora**, e é a colheita de outra wave: a caminhada da esquerda para a direita
(feita por outro motivo — *o começo do traço parava de tremer*) dá **estabilidade de prefixo**
(`fit(a[..n])` é prefixo de `fit(a[..m])`). Com a decisão **global** de antes, um prefixo em cache
**mentiria**.

⚠️ **O cache VERIFICA, nunca PROMETE:** ele guarda a entrada exata do último ajuste e mede o prefixo
comum **bit a bit** (`to_bits()`). Obrigatório, porque a entrada do ajuste é o array **SUAVIZADO** e
o `active_smooth` **reescreve a cauda** a cada amostra: uma promessa de *"só cresceu"* seria falsa em
todo frame, e o modo de falha não é um erro — é um traço plausível decidido sobre dados que não
existem mais.

**Superfície:** `FitCache` · `fit`/`simplify_to_curve` viraram `#[cfg(test)]` (o segundo como
**oráculo congelado**) · `FlipDraw::samples()` morreu · `stroke_from_samples` virou delegação pura.

---

## 3. `d8fe36d42` · `4fead3e20` · `02c4baeb2` — as três medições

- **Cache de tiles de mundo:** o pan custa **4,77 ms/camada** a 200 traços, e o cache só é **EXATO
  sob pan de pixel INTEIRO** (1 px → delta 1e-6; **0,5 px → 0,408 em 27% dos pixels**). ⇒ virou
  **decisão de CÂMERA** + foundational cross-line (`ph2d-render::LayerCompositor`, compartilhado com
  o Painter). Nada de produto foi tocado.
- **Item 3c FECHADO:** construí uma terceira cura (amostrar no **centroide**) e um oráculo
  **supersampleado** a reprovou (zigue-zague 0,8: média 4,45 vs 3,39; pior 96,38 vs 61,86).
  **Revertida.** E o número que fecha: o pior erro da lei que shipa contra a verdade é **22–62/255**,
  enquanto o resíduo de quina é **≤ 14,94/255** — *o artefato é menor que o erro da aproximação que o
  curaria*.
- **A 3ª lei DESLIGA o Self Overlap:** ela capa o cruzamento no valor **exato** do braço, em toda
  dureza (**1,50× → 1,00×**). ⇒ ela é **mutuamente exclusiva** com uma feature shipada. O preço do
  item 5 tem duas metades e agora as duas estão medidas. **Decisão do Enio; nada construído.**

---

## 4. `f4b3c5b57` — A CANETA NÃO CHEGA (e duas flakes)

A nota que dizia *"custa uma função"* era **falsa no winit 0.30** — não existe evento de caneta
(`Touch` é touchscreen · `TouchpadPressure` é trackpad da Apple · `AxisMotion` é valuator **cru** do
XInput2 com `AxisId` opaco · e o backend **Wayland** não tem nada). ⇒ **winit bump (classe ADR)** ou
caminho de tablet por plataforma.

Virou **gate** (`the_desktop_shell_has_no_pen_pressure`): os dois sliders do Flip são **inertes** na
pressão que esta shell entrega (21×21 combinações, desvio `< 1e-6`). Ele fica **VERMELHO no instante
em que alguém liga a caneta** — que é exatamente quando os defaults precisam ser re-calibrados.

**As duas flakes** (`flip_fit_budget_tests`) viraram razão + **min-de-3**, e a lição do eixo está no
handoff anterior §6(b): densidade dá separação de 1,4× (inútil), **comprimento** dá 3×.

---

## 5. `6d51a9443` — A PONTA DO TRAÇO GANHA PORTA

⚠️ **`Cap::Flat` era variante MORTA no produto.** O motor honra `FlipStroke::cap` ponta a ponta
desde que o percurso landou — o bit no `pack`, o semi-plano na silhueta, o ramo do `flip.wgsl` —,
tudo gateado e com paridade CPU×device provada. E **nenhum traço do artista jamais foi reto**: o
`build_stroke` não escrevia `s.cap` e o `FlipStyleSnapshot` não tinha o campo.

**Uma capacidade sem porta passa em TODOS os gates**, porque cada peça dela está certa.

Row `Cap` no painel do Draw, logo abaixo do Tip. **UM valor para as duas pontas**, por decisão: o par
existe no modelo porque a borracha, ao partir um traço, pode dar pontas diferentes às metades — não
porque o artista as autore separadas (SVG/Illustrator/Krita oferecem um controle só).

Este commit tem **zero schema** (o `cap` já viajava serializado) e **zero mudança de motor**.

---

## 6. `bd88a1cbc` — A PONTA CHATA CORTAVA SÓ O 1º SEGMENTO

Report do Enio com foto: a tampa `Flat` saía com um **domo raso** no meio do corte.

**Reproduzido antes de explicado**, e a previsão casou:

| 1º segmento | tinta além do plano | previsto (`r − gap`) |
|---|---|---|
| 40 px · 20 px | **0,50** (só o AA) | 0,00 |
| 8 px | **11,50** | 12,00 |
| 3 px | **16,50** | 17,00 |
| 1 px | **18,50** | 19,00 |

⚠️ **Defeito LATENTE que OUTRA WAVE DESTA MESMA LINHA tornou visível:** o maquinário de tampa foi
escrito quando o traço tinha pontos esparsos; o **ajuste 3× mais denso** encurtou o 1º segmento para
poucos px e o disco de ponta do **VIZINHO** passou a espiar.

**Cura:** o plano vira fato do **TRAÇO** (posição, normal, arco) e alcança todo segmento a menos de
`r` de **ARCO**. ⚠️ **É o arco, não a distância geométrica** — é isso que preserva a razão
documentada do desenho por-segmento: um traço que se **enrola de volta** está geometricamente perto e
a **arcos** de distância, e segue pintando ali.

Espelhado no `walk.wgsl`. **Paridade CPU×device 118/118 na RTX.**

⚠️ **O gate PRECISA de `arc_len` de verdade** — o `art` o zera, e com ele zerado todo segmento parece
colado na tampa ⇒ **verde pelo motivo errado**.

**LOC:** `binning.rs` cruzou 728 ⇒ split por responsabilidade em **`silhouette.rs`** (*que forma o
traço tem neste pixel*) contra o binner (*que segmentos alcançam este ladrilho*) — 558 + 180.

---

## 7. `29125dd66` — A PONTA QUADRADA É GEOMETRIA, NÃO MÁSCARA

⚠️ **Ela não podia ser um `max` na silhueta como a reta.** Neste motor a cobertura é a integral da
tinta **ao longo do CAMINHO**, e a quadrada **ACRESCENTA** região: fora do caminho não há o que
integrar, e ela sairia **VAZIA**.

⛔ **CONSTRUÍDO E DERRUBADO PELA ÁLGEBRA — não refaça:** empurrar o plano de corte por `r` é a
resposta intuitiva e é **geometricamente vazia** — a cápsula só alcança `r` além do ponto, então o
plano fica **TANGENTE** ao disco e não remove um texel.

**A cura é a definição do SVG ao pé da letra:** *square é o traço **estendido** por meia-espessura e
então cortado reto*. A extensão vira **geometria no empacotamento** (`append_drawing`) — um segmento
a mais, de raio constante, com o bit de corte no ponto **NOVO** — e o resto do motor vê um traço
normal de ponta `Flat`. **Zero lei nova, zero shader novo, zero risco de paridade.**

⚠️ **Só em traço ABERTO, de linha CHEIA e com largura.** Com contas a linha já é uma série de
carimbos, e estender um retângulo desenharia uma **BARRA** onde tem de haver uma conta. Nesses casos
`Square` rende como `Round` — nomeado, não acidental.

**O oráculo é o CANTO do quadrado** (a `(0,8r, 0,8r)` da ponta): ali `Square` tem tinta e
`Round`/`Flat` não. ⚠️ *"Até onde vai a tinta no EIXO"* **não separa as três** — ali `Round` e
`Square` ambos passam do ponto, e um gate escrito nesse eixo ficaria verde sobre a ponta errada.

**Este é o commit que bumpa o schema** (§0).

---

## 8. `e942d6bb3` — A BORRACHA NÃO FAZ O TRAÇO CRESCER PARA DENTRO DO VÃO

Regressão que a `Square` introduziu, **e o `new_like` já avisava** no próprio doc-comment: *"todo
campo novo do `FlipStroke` tem de passar por aqui"*. `Square` não é campo novo — é **VARIANTE nova**,
e ela muda o que `s.cap = src.cap` **significa**. Copiada verbatim, cada fragmento ganhava `Square`
nas pontas **CORTADAS**, e o buraco saía mais estreito que a borracha por uma espessura inteira.

**A lei:** uma tampa descreve onde o **ARTISTA** terminou o traço; uma ponta de corte é onde a
**borracha** passou, então ela nunca pode **ESTENDER**. O `split_by` já tinha a informação (o índice
do run diz quais pontas são originais) — ela só não era lida.

⚠️ **`Round` numa ponta cortada fica como está, de propósito:** é o comportamento de sempre, e
trocá-lo por `Flat` mudaria **toda borracha já usada**, em silêncio e sem um smoke. A porta é
`cut_cap` — se for reprovado um dia, é uma linha.

---

## 9. Estado medido no tip (`e942d6bb3`)

| suíte | release | debug |
|---|---|---|
| `ph2d-host-desktop` | **1687 / 0** | **1687 / 0** |
| `ph2d-flip-render` | **84 / 0** | — |
| `ph2d-flip-render -- --ignored` (GPU, RTX) | **118 / 0** | — |
| `ph2d-editor-core` | **908 / 0** | — |
| `ph2d-flip` · `ph2d-tool-flip` · `ph2d-panel-flip` | 150 · 23 · 29, **0 falhas** | — |
| `cargo fmt --check` · `clippy --all-targets` | **limpos** | — |

⚠️ **Rode em DEBUG E RELEASE** — um gate desta linha já reprovou **só em debug** (21,65 contra 1,92
ms): um bar de relógio mede o PERFIL do build.

⚠️ **Gates de GPU são `#[ignore]` e precisam de adapter.** Sem um fazem *skip gracioso*, **que não é
verde**. O `walk_gpu_parity` é o que prova que o motor que SHIPA concorda com a referência de CPU:

```bash
cargo test -p ph2d-flip-render --release -- --ignored     # 118/118 na RTX
```

⚠️ **Uma corrida NÃO REPRODUZIDA, registrada por honestidade:** numa execução o release deu
**1507/2**, com contagem menor que o debug — ela rodava **concorrente com o `clippy`** no mesmo
comando. As **três** seguintes deram 1687/0 e **não sei qual teste era**. Se reaparecer, o suspeito é
a família de gates de relógio.

---

## 10. Smoke

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP
env PH2D_FLIP_HARDNESS_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

**Nenhuma cena nova** — a porta é o próprio painel, que é o que tinha de ser exercitado. Painel do
Flip em **Draw**, pincel **grosso**, a row **`Cap`** logo abaixo de `Tip`:

1. **Round · Flat · Square** — as pontas têm de ser três formas distintas; `Square` passa
   meia-espessura do fim **com canto**.
2. **A borracha**: desenhe em `Square` e **apague o meio** — o vão tem a largura da borracha, sem os
   fragmentos crescendo para dentro dele.
3. **O traço longo e lento** (o cache, §2): olhe o **começo** enquanto a mão ainda anda — ele não
   pode tremer nem re-decidir.

Diagnóstico: `PH2D_FLIP_NEW_ENGINE=0` (A/B com o rasterizador antigo) · `PH2D_FLIP_STATS=1`.

⚠️ **Não verificado, e digo porque não medi:** não rodei o A/B com `Square`. Ele **deve** funcionar
de graça no rasterizador antigo — a extensão é geometria no `pack` e ele lê os mesmos `points` —, mas
isso é raciocínio, não medição.

---

## 11. Aberto — e o que é DECISÃO do Enio

| item | estado |
|---|---|
| **A 3ª lei** | preço INTEIRO medido (borda **+69%** *e* Self Overlap **1,50× → 1,00×**) ⇒ decisão de **LOOK** |
| **Cache de tiles de mundo** | precificado (4,77 ms/camada; exato só sob pan inteiro) ⇒ decisão de **CÂMERA** + foundational |
| **Caneta / tablet** | levantado e gateado ⇒ **winit bump (ADR)** ou caminho por plataforma |
| **JOINS** | **recomendo NÃO construir** — ver abaixo |
| Resíduo de quina (3c) | **FECHADO** por medição |
| Cache incremental do ajuste | **FEITO** |

### Por que eu recomendo não construir joins

No percurso a junção **já é redonda** (a união de cápsulas a dá de graça), e **isso é o correto para
PINTURA** — é o que Procreate/Krita/Photoshop fazem, porque um pincel é um carimbo arrastado.
Miter/bevel são conceito de gráfico **vetorial**, e este repo tem um módulo Vector que **já os tem**
(`VecOffset { d, join, side }`). O ganho é baixo e o risco é alto: mexe na **cobertura-união**, que o
handoff mestre do motor chama de *"a joia que custou uma semana de bugs"*.

Se o Enio pedir mesmo assim: **bevel ficou barato** (é subtrativo, e a máquina de plano-com-alcance-
por-arco da §6 já existe); **miter é aditivo** ⇒ mesma família da `Square`, e a cura é a mesma
(geometria no `pack`).

---

## 12. Ordem de integração

Fora do schema (§0), esta linha **não conflita com nada por construção**: zero `Cargo.toml`, zero
ADR, zero contrato congelado, e os 3 ids são hash-de-string (o `node_id_collisions` cobre).

O diff mora em: `crates/ph2d-flip` · `ph2d-flip-render` · `ph2d-panel-flip` · `ph2d-tool-flip` · três
ids em `ph2d-editor-core` · e `shells/desktop/src/flip_*`.

Território comum plausível: **`shells/desktop/src/flip_draw.rs`** (qualquer wave do Flip o toca) e
**`crates/ph2d-editor-core/src/ids/chrome/flip.rs`** (append-only — três linhas no fim de um bloco).
