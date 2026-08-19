# 38 — Plano: o card **Line**, o Style Line/Solid e os traços procedurais

> Ordem do Enio, 2026-08-12: *"crie um plano de implementação e salve o plano. Quero essas novas
> features dentro de um card acima de Composite Brush. Dentro do card um checkbox para Style —
> Line/Solid e um Dropdown com cada uma das opções de traço/linha. Ao escolher um tipo de traço,
> sliders de ajustes específicos para aquele tipo aparecem no card. Por padrão o dropdown é none."*
>
> A pesquisa que decide **quais** tipos entram é o doc [37](37_pesquisa_tracos_procedurais.md).
> Este doc é o **como**: a forma exata do card, onde cada coisa mora, as waves em ordem de custo, os
> gates de cada uma, e o que é decisão do Enio em vez de engenharia.
>
> ⚠️ **Clean-room:** o Alchemy é GPL-3 e o Krita também. Tudo aqui é **comportamento**, lido de
> manual — nenhuma linha de fonte de nenhum dos dois foi lida, e nenhuma será.

---

> ⚠️ **Recorte de 2026-08-18.** As waves **W0 · W1 · W2 · W3 · W4 · W5 · W6 · W7** fecharam, e o
> corpo delas foi movido **verbatim** para
> [`docs/archive/docs-2026-08-18/Painter/38_plano_linha_procedural.md`](../archive/docs-2026-08-18/Painter/38_plano_linha_procedural.md).
> Ficou aqui: **a forma do card** (§1), **onde cada coisa mora** (§2), **as três leis que toda wave
> obedece** (§3), as **⛔ recusas e as ⚠️ leis** que as waves compraram, o que segue **Aberto,
> nomeado**, as **decisões do Enio** (§5), o que fica **FORA** (§6) e a **ordem** (§7).
> ⛔ Nada foi resumido — as duas metades remontam o original byte-a-byte (sha256).

## 1. A forma do card (o que o Enio pediu, desenhado)

O card fica **imediatamente acima do Composite Brush** — em `paint_brush.rs`, entre o `4b′` (Inpaint)
e o `4c` (Composite).

```
┌─ Line ───────────────────────────────────┐
│  ☐ Solid                                 │   ← Style: desmarcado = Line, marcado = forma sólida
│  Type   [ None            ▾ ]            │   ← default None
│                                          │
│  … as rows DO TIPO escolhido …           │   ← nada aqui enquanto Type = None
└──────────────────────────────────────────┘
```

Com **Type = Sketchy**, por exemplo:

```
┌─ Line ───────────────────────────────────┐
│  Type   [ Sketchy         ▾ ]            │
│  Reach        ▬▬▬▬▬▬●▬▬▬   0.35          │
│  Density      ▬▬▬●▬▬▬▬▬▬   0.20          │
│  Line Width   ▬▬●▬▬▬▬▬▬▬   0.12          │
│  Opacity      ▬▬▬▬●▬▬▬▬▬   0.30          │
│  ☑ Magnetify                             │
└──────────────────────────────────────────┘
```

**Três leis de forma, e cada uma tem precedente executável neste repo:**

1. **O card pinta SÓ as rows do tipo escolhido.** É o `each_kind_paints_only_the_rows_it_uses` da
   §12 da física e o `knob_family()` do card do Sculpt. Row de outro tipo pintada aqui é knob morto.
2. **O checkbox `Solid` só é pintado onde o tipo tem caminho fechado para preencher**
   (`LineKind::honours_style()`). Sketchy e Wire produzem *muitos fios curtos*, não uma
   silhueta — oferecer Solid neles seria um checkbox que não faz nada. Ver §5.1: isto é uma decisão
   e não um detalhe. ⚠️ **A frase dizia "Sketchy/Wire/Spray" e o Spray saiu dela na W5**: ele não é
   um tipo de linha (não está no dropdown), e o que ele produz são **dabs** sobre o mesmo
   caminho-base — a silhueta continua lá, só carimbada `n` vezes.
3. **O dropdown não é pintado enquanto tiver uma opção só.** Na W1 o card tem apenas o checkbox; o
   dropdown nasce na W2, junto com o primeiro tipo. Um dropdown de uma linha é o mesmo controle
   morto com outra roupa.

---

## 2. Onde cada coisa mora (levantado por `grep`, não de memória)

| O quê | Onde | Nota |
|---|---|---|
| O card | **arquivo novo** `crates/ph2d-panel-painter-layers/src/paint_line.rs` | molde: `paint_composite.rs` (178 LOC, card com borda própria) — ⚠️ `paint_brush.rs` está em **611**, o card não cabe lá |
| A chamada | `paint_brush.rs`, entre o `4b′` e o `4c` | uma linha |
| Os ids | `ph2d-editor-core/src/ids/chrome/painter.rs` | **`hash_node_id("painter_line.…")`** ⇒ **nenhum gate de contagem**, nenhum id numérico a alocar |
| O registro | `populate.rs` (499) | sem ele o widget pinta, registra hit e fica **morto sob o mouse** |
| A rota do Click | `event.rs` (567) | checkbox + dropdown |
| A rota do slider | `event_brush_forward.rs` | a whitelist `is_forwardable_brush_slider` |
| O estado | `BrushSpec` (`ph2d-painter-brush/src/spec.rs`, 445) | ⚠️ **`BrushSpec` não é serde** ⇒ **zero `PROJECT_SCHEMA`**, zero save antigo afetado |
| O espelho do painel | `BrushSettings` (`brush_settings.rs`, 581) | o snapshot que o card lê |
| O emissor de dabs | `ph2d-painter-brush/src/stroke.rs` — **691/700** | ⚠️ todo produtor novo nasce em **módulo irmão**, obrigatoriamente |

**Estado novo, num bloco só:**

```rust
pub enum LineKind { None, Speed, Sketchy, Wire, Spray }   // discriminantes de wire 0..4

// em BrushSpec:
pub style_solid: bool,      // false = Line (o mundo de hoje), true = forma sólida
pub line_kind: LineKind,    // None por default
pub line: LineParams,       // TODOS os params de TODOS os tipos, num struct só
```

⚠️ **`LineParams` é um struct ÚNICO com neutro, nunca um struct por tipo.** É o molde do
`KernelResolver`/`SkinParam` e dos canais de side-metadata do registry: um tipo que não consome um
campo não sabe que ele existe, e `LineParams::default()` é o mundo de hoje **byte a byte**. Um enum
com payload por variante espalharia `match` por todo sítio de construção e faria o tipo N+1 tocar
todos eles.

---

## 3. As três leis que toda wave obedece

1. **O neutro é BYTE-IDÊNTICO.** `line_kind == None && !style_solid` ⇒ o depósito produz os mesmos
   bytes de hoje. Não é promessa: é **gate de fingerprint** rodado em toda wave, e é a rede que
   torna cada uma reversível.
2. **Uma porta.** Os tipos que **moldam** um dab entram no `walk_dab` (onde Symmetry, Tiling, shape
   editors, pressão, Jitter, Shape e Grain já se penduram — foi assim que o smear field herdou tudo
   de graça). Os que **emitem dabs a mais** entram no emissor, **antes** do `push_symmetric`, para
   que as cópias de simetria espelhem os fios e não só o eixo. Isto é gate, não comentário.
3. **Medir antes de limitar (§0 do `CLAUDE.md`).** Nenhum slider ganha faixa sem a sonda ao lado.
   Um teto que só diz "por segurança" é um palpite esperando um smoke.

---

# AS RECUSAS ⛔ E AS LEIS ⚠️ QUE AS WAVES COMPRARAM

> Recortes das waves fechadas — o contexto de cada uma está no
> [arquivo](../archive/docs-2026-08-18/Painter/38_plano_linha_procedural.md).

## W2 (Speed) — ⛔ o `Curved` NÃO se constrói

⛔ **`Curved` não foi construído, e não é adiamento:** num motor de dabs não existe *segmento*, então
"retas × curvas" não tem referente — qualquer significado seria inventado. O `Line Type` do Alchemy
governa o `GeneralPath` dele, que tem segmentos de verdade.

**9 gates · 8 mutações · 7 sangram** (a sobrevivente é a janela da EMA de velocidade, **medida** e
documentada: depois que a mira passou a fazer o trabalho pesado, o valor dela deixou de ser
observável nos regimes medidos).

**Smoke `=2`:** o mesmo gesto devagar e depressa; e um traço lento tem de ser indistinguível do
`None`.

#### O que a construção MUDOU no plano (fechada 2026-08-13)

**A `Curved` NÃO foi construída, e é decisão, não esquecimento.** O manual do Alchemy oferece *"Line
Type — toggle between drawing straight lines and curved lines"*, e ali a Speed Shapes produz uma
FORMA (um caminho fechado) cujos segmentos são retos ou curvos. **Num motor de dabs não há
segmento** — o traço já é um contínuo de carimbos —, então qualquer significado que eu desse a
`Curved` seria INVENTADO. Um knob cujo comportamento a referência não determina é um knob que mente;
ele volta no dia em que houver uma pergunta de produto por trás dele.

⚠️ **A velocidade NÃO virou campo do `Dab`, e o plano dizia que sim.** A justificativa era o consumo
futuro (Sketchy, Splatter), mas um campo com **um consumidor interno** e nenhum leitor é um campo
morto que atravessa o `Dab` inteiro. O que a lei exige é *um lugar computa, todos leem*, e isso é
satisfeito por **`Stroke::speed_px_s()`** — a porta que o Sketchy vai ler. O `Dab` fica intacto.

⚠️ **O `SPEED_LOOKAHEAD_S` é MEDIDO, e é um TEMPO.** A W0.1 mede ~39 px de arco por quadro num gesto
ligeiro de quarto de círculo ⇒ **~2 340 px/s**. A janela é **um quadro de 60 fps**, então `Amount = 1`
arremessa exatamente o arco de um quadro (~39 px naquele gesto) e o teto — `MAX_SPEED_AMOUNT = 8` —
diz **quantos quadros à frente** a tinta pode ir: 312 px ali, que é o *"and possibly off the screen
itself"* do manual. Acima disso a tinta está mais de um oitavo de segundo à frente da mão.

## W3 (Sketchy) — a medida por TIQUE

⚠️ **E a medida por tique NÃO pode ser abandonada** — é ela que a W0.1 provou ser a única
device-independente (o per-evento varia 73×, o per-dab é constante por construção). O que muda é
COMO ela é aplicada: a velocidade que cada dab cavalga **caminha** da anterior para a nova ao longo do
**arco daquele quadro**.

⚠️ **A rampa não tem constante mágica, e isso está GATEADO:** o comprimento dela é o arco do próprio
quadro, então ela dura *um quadro de percurso* a 300 px/s e a 3 000 px/s igualmente. A mutação que a
troca por uma constante em pixels **sangra** (vão 22,7 contra um diâmetro de 20 no topo da faixa) —
a auto-escala é load-bearing, não decoração.
## W3 — o que ficou aberto

**Aberto:** os shape editors (o escopo acima) · o smoke `=3`.
## W6 (Ribbon / Rough) — as leis da FAIXA e do massa-mola

  - ⚠️ **A ASSIMETRIA é o desenho, não um compromisso:** o trilho de tinta é de **DABS**, o trilho de
    fora e as travessas são **FIOS** (o canal `Thread` do Sketchy/Wire, mesmo rasterizador, mesma
    tinta). Um segundo trilho de dabs seria uma segunda **pincelada** — a Symmetry o espelharia, o
    Spray o multiplicaria, o impasto construiria relevo nele e a taper o afinaria —, e um ribbon é
    UMA marca. É também o que evitou o segundo cursor de percurso que a tentativa revertida pedia.
  - ⚠️ **A interpolação NÃO é refinamento — é ela que impede o LEQUE:** um quadro emite várias
    travessas, e ligá-las todas ao dedo de AGORA faz um punhado de segmentos convergir num ponto
    (o leque das cristas). As duas pontas de uma travessa são do **MESMO instante**, carimbadas pela
    fração do quadro. Gate próprio, com mutação.
  - ⚠️ **A cadência é de ARCO e o resíduo atravessa os quadros** — uma travessa por DAB preenche
    SÓLIDO (a primeira tentativa: um dab sai a cada ~1,2 px) e uma por QUADRO faria o número de
    travessas mudar com a taxa de quadros. O gate afirma as duas metades (taxa de eventos **e** taxa
    de tiques).
  - ⚠️ **O PEN-UP não costura**, pela mesma lei do leque: ali o trilho do DEDO acabou, e ligar seja o
    que for ao ponto onde a caneta levantou é literalmente o leque. A faixa termina onde a mão
    terminou.
  - ⚠️ **`Rungs = 0` é uma DEGENERAÇÃO, não um modo** — uma faixa sem travessas é uma linha, e sobra
    o massa-mola sozinho (o *Dyna*). É por isso que a família **não** precisou de um segundo tipo nem
    de um interruptor: uma densidade cobre os dois looks, e um interruptor teria de escolher um nome
    para o estado desligado. O default nasce em **0,5** — uma fita É uma faixa, e um default em `0`
    daria a quem escolhe `Ribbon` o pincel de arrasto sob o nome da outra feature.
    - ⚠️ **Isto NÃO contradiz *a fita é fato do relógio*** — aquela lei é sobre integrar em
      SEGUNDOS (960 Hz desenha o que 125 Hz desenha) e continua de pé. O que morreu foi uma frase
      **minha, não da referência**: *"solte no ar e a fita continua a chegar"*, que eu tinha posto
      no roteiro do smoke como feature.
    - ⚠️ **A lei *"sem gesto, sem tempo"* NÃO é revogada** — ela curou a mola e os gates dela medem
      o que dizem. O que faltava não era grau, era **ENDEREÇO**: ela foi instalada num dos dois
      percorredores, e o outro corre no MESMO quadro parado com a resposta **oposta** (o tique
      congela, o `settle` salta até ao cursor). Das duas leis contrárias sobre o mesmo instante,
      sobrevivia a que emitia dabs.
  - ⚠️ **Ela NÃO é um segundo estabilizador, e a distinção é MEDIDA, não argumentada:** ela
    **ultrapassa** (`ζ < 1` passa do alvo e volta — o estabilizador é média corrida e converge por
    baixo, com nenhuma intensidade) e é **fato do RELÓGIO** (um mouse de 960 Hz desenha o que um de
    125 Hz desenha). Cada metade tem gate próprio. ⚠️ E o CONTROLE derrubou a 1ª versão desta nota,
    que dizia *"o atraso do estabilizador não depende da velocidade"*: **depende** (50,4 → 386,4 px
    para 8× a velocidade), porque um lag de 1ª ordem em regime também vale `v · τ`.
  - ⚠️ **O TETO LIMITA O TRABALHO, NUNCA A RESOLUÇÃO** — a lei do `FixedStep`, e a 1ª versão fazia o
    oposto: capava o número de sub-passos e deixava `h = dt/n` crescer, **desfazendo** a garantia de
    `ω · h = 0,25`. Custou **90,2 GB de RSS** e a janela do editor (achado externo). Detalhe,
    mecanismo e números: [`BUGS_painter.md` #23](BUGS_painter.md).
  - ⚠️ **O fundo do slider é INERTE e está medido:** até peso ~0,02 a fita não move a tinta um dab
    inteiro. É o comportamento certo para um mínimo que significa *desligado*, e o roteiro do smoke
    o diz em vez de deixar o artista descobrir.
  - ⚠️ **Ele move a TINTA, nunca o CAMINHO** — a mesma lei do `Speed`, pelo mesmo motivo escrito no
    doc do `throw`: `last_pos`, o `accum` do espaçamento e o `arc_len` continuam sendo o que a mão
    fez. Aqui a realimentação seria impossível de qualquer modo (o campo não acumula), e a lei fica
    dita porque o próximo tipo que a violar não terá aviso nenhum.

## W7 — a Symmetry dos fios, e o que ficou aberto

alguém afinasse uma delas.

⚠️ **A Symmetry dos FIOS não foi tocada** — ela já morava no motor (`push_symmetric_segment`, ao lado
da que espelha o dab que gerou o fio), e replicá-la outra vez no depósito duplicaria cada cópia.

#### Aberto, nomeado

- **A fita e a mancha discordam sobre onde o traço está, e por desenho:** o caminho do preenchimento
  é o do PONTEIRO, e o `Ribbon` deixa a tinta até 800 px atrás dele (o `Speed` a joga à frente). É a
  mesma família de *"a tinta não está sobre o caminho"* que os dois tipos existem para produzir, e o
  **smoke decide** se ela lê bem sob Solid.
- **Sob impasto a mancha é PLANA** e só o traço tem corpo — o preenchimento não escreve relevo. É
  coerente com o Flip (o `fill` não tem espessura) e fica nomeado.
- **Com `Strength < 1` a borda fica mais escura que o miolo**, porque a mancha e o traço se somam na
  faixa em que se sobrepõem. É inerente a pintar as duas coisas (o Grease Pencil tem o mesmo), e o
  default de 1.0 não o mostra.

---

## 5. As decisões — as três primeiras RESPONDIDAS pelo Enio em 2026-08-12

### 5.1 O `Solid` é oferecido em quais tipos? — ✅ **"para todos que forem possíveis"**

⚠️ **E a resposta honesta é: são TODOS**, porque todo tipo mantém o **caminho-base** do gesto — o
Sketchy, o Wire e o Spray são decoração ADITIVA por cima dele (é o que o *Paint Connection Line* do
Krita liga e desliga). Logo há sempre uma silhueta para preencher.

⚠️ **A porta única `LineKind::honours_style()` FICA mesmo devolvendo `true` em todos os casos de
hoje**, e não é cerimônia: ela é onde a resposta mora no dia em que nascer um tipo **sem** caminho-base
(um que só emita partículas). Sem a porta, esse tipo nasce com um checkbox morto e ninguém percebe.
Gate: `every_kind_with_a_base_path_offers_solid`.

### 5.2 A borda de uma forma sólida — ✅ **"pode ser sólida e específica para essa tool; faça o melhor possível"**

⚠️ **A W7 mudou de que borda esta seção fala.** A medição abaixo é da borda da **MANCHA** — a
cobertura exata por área, que continua sendo o melhor e o mais barato dos três candidatos. O que o
artista **vê** hoje é a borda do **TRAÇO**, meia espessura para fora dela e com o falloff do pincel:
a mancha é a região, o contorno é o pincel, e a borda analítica fica escondida sob o miolo opaco do
traço. Fazer a mancha crescer meia-espessura sozinha seria uma **segunda resposta** a *"que borda
este pincel tem"*, divergindo do falloff no dia em que alguém afinasse um dos dois.

Então é **borda da FORMA**, e *"o melhor possível"* virou uma medição em vez de uma opinião (W0.3 do
`line_probe` do tool): erro de cobertura contra a referência `SS = 32`, num disco de raio 300 —

| lei | níveis | erro médio | erro máx | **px > 8/255** | custo |
|---|---:|---:|---:|---:|---:|
| **`SS = 3`** (o de hoje) | 10 | 9,47 | 38,43 | **51,1%** | 11,3 ms |
| `SS = 4` | 17 | 5,96 | 26,15 | 22,7% | 19,9 |
| `SS = 8` | 65 | 1,92 | 13,45 | 1,4% | 72,5 |
| `SS = 16` | 257 | 0,57 | 7,47 | 0,0% | 284,0 |
| **ÁREA EXATA** (o que shipou) | 256 | **0,31** | **2,47** | **0,0%** | **0,8** |

⚠️ **O `SS = 3` que o composite usa hoje erra mais de um degrau visível em METADE da borda** — ele
foi calibrado para ser *traçado*, e um traçado joga a cobertura fora.

⚠️ **E a última linha decide sozinha: a acumulação de área com sinal é MAIS precisa que `SS = 16` e
catorze vezes mais BARATA que o `SS = 3`.** Ela não amostra — integra a área exata do pixel coberto
numa passada `O(arestas + área)`, sem buffer supersampleado. Não há trade a ponderar: *"o melhor
possível"* e *"o mais barato"* são a mesma escolha, e é ela que shipou
(`ph2d-painter-brush::solid`). O resíduo de 0,31 nível é o polígono de 2048 lados que aproxima o
círculo da fixture mais o erro da própria referência, não o rasterizador.

### 5.3 O que `Solid` faz com um gesto que não fecha? — ✅ **"fechar sozinho"**

Fecha do último ponto ao primeiro, como o Alchemy. **Sem limiar de área e sem caso especial**: uma
pincelada reta em Solid vira uma *sliver* de área quase zero, e essa consequência fica **nomeada** em
vez de remendada — um limiar seria um caso especial que a §0 manda medir antes de escrever, e não há
o que medir enquanto ninguém reclamar.

### 5.4 O Style é do PINCEL ou do documento?

Ele mora no `BrushSpec` (por-modo, como todo o resto), o que significa que **cada meio tem o seu**.
Se o Enio quiser um único Style atravessando Digital/Aquarela/Impasto, ele vira estado do tool com
fan-out para os slots — o molde do `toggle_brush_impasto`, que escreve nos três slots de relevo
porque *os três são o mesmo assunto*.

---

## 6. O que fica FORA, com o motivo

- **Pressure Shapes.** Temos a feature (`dynamics.rs`) e **não temos o dispositivo**:
  `shells/desktop/tests/the_desktop_shell_has_no_pen_pressure.rs` demonstra, varrendo a grade inteira
  de sliders, que nenhuma combinação move um pixel. A cura é **subir o winit** (fundação de janela do
  app inteiro ⇒ cross-line, classe ADR) ou um caminho de tablet por plataforma. Não é wave de pincel.
- **A matriz de dinâmica completa** (doc 37 §2). Depois da W2 e da W3 haverá duas entradas e três
  alvos; antes disso é infra sem consumidor.
- **Assistentes de perspectiva.** O motor de snap já é do Vector; um segundo no Painter é a segunda
  porta. É wave de ponte.
- **Generativo** (flow field, growth, voronoi). Os algoritmos já são nossos, no Motion Nodes, com
  paridade CPU×GPU gateada. Falta a **ponte**, e ela é ADR.
- **Blindness / Limit / Auto-Clear.** São ferramentas de ideação, num app que tem undo, save e
  camadas. Mudam a tese do produto; só com pedido explícito.

---

## 7. Ordem de execução, resumida

| Wave | Entrega | Tamanho | Depende de |
|---|---|---|---|
| **W0** | as três medições | pequena | — |
| **W1** | o card + **Solid** | média | W0 (2) |
| **W2** | o dropdown + **Speed** | média | W0 (1) |
| **W3** | **Sketchy** | média | W0 (3) |
| **W4** | **Wire** | pequena | W3 |
| **W5** | **Spray** — só o `Count`, e fora do dropdown | pequena | — |
| **W6** | Ribbon (a FAIXA) ✅ · massa-mola (*Dyna*) ✅ · **Rough** ✅ | `PH2D_LINE_SMOKE=1` | Fecharam em 2026-08-14/15. ⚠️ **Esta célula dizia `Rough ⛔ · só com pedido` e estava OBSOLETA** — o Rough foi construído no mesmo dia, com os sliders `Roughness · Bowing · Passes` |

⚠️ **A W1 e a W2 são independentes** — se o Enio quiser ver o card mais cedo, a W1 sozinha já entrega
um card com um controle vivo. O que não pode acontecer é o card nascer com o dropdown de uma opção
só.
