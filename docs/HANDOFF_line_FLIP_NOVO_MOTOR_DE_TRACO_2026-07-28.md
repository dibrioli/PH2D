# HANDOFF — **DESCARTAR o rasterizador de traço do Flip e construir um novo, padrão-ouro**

**Data:** 2026-07-28 · **Linha:** `line/FLIP` · **Autor:** o agente que passou a jornada tentando
consertar o motor atual · **Decisão:** do Enio.

> *"Creio que não tem cura. Escreva um handoff para outro agente encontrar um modo completamente
> novo de renderização do stroke e descartar completamente o atual, de modo que alcancemos o
> visual de um painter digital normal que não apresenta problemas como cruzamento de linhas com
> hardness. Ele deve pesquisar o estado da arte, o padrão ouro. E descartar totalmente o modelo
> atual de renderizar o stroke."*

---

## 0. Antes de ler qualquer código

Você assume uma linha **que já existe**. Execute a **FASE 0** do
[`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
— `cd Worktrees/line-FLIP && pwd && git branch --show-current` — **antes** de abrir um arquivo.
O mesmo path relativo existe na raiz (que está em `main`) e na worktree, e editar a errada
compila, testa e commita **sem um único erro**.

Não copiei o bloco para cá de propósito: duas cópias da mesma regra divergem.

**HEAD desta linha quando este handoff foi escrito:** `487ad5d34`.

---

## 1. A ordem, e o que ela NÃO é

**A ordem:** pesquisar o estado da arte, escolher o padrão-ouro, e **substituir** o rasterizador
de traço do Flip. Não é afinar o atual. Não é mais uma rodada de correção.

⚠️ **O que ela não é:** não é jogar fora o *módulo* Flip. O documento (`ph2d-flip`), a autoria
(`ph2d-tool-flip`, `flip_draw`), a tira, o tween, o colorize, o fill, o multiplano e a timeline
**ficam**. O que sai é a resposta à pergunta *"dado um `FlipStroke`, que pixels ele acende?"* —
hoje `crates/ph2d-flip-render/src/shaders/flip.wgsl` + `neighbors.rs` + a metade de `pack.rs` que
os alimenta.

⚠️ **E um fato que você precisa saber antes de comparar com qualquer coisa:** o último conserto
desta linha (`7ca83d6fb`) está **pendente de smoke** — o Enio deu o veredito sem tê-lo visto na
tela. Rode o smoke uma vez (§7) só para saber de que baseline você está partindo. Isso não
reabre a decisão dele; é para o seu número de partida ser o número real.

---

## 2. O que o motor atual É (em três frases)

1. Cada segmento da polilinha vira um **quad** que apenas COBRE a fita (retângulo + tampas,
   conectado por miter com `miter_break`).
2. O fragment calcula a cobertura **analiticamente**, como `perfil(distância normalizada do pixel
   à linha-de-centro)`, tomando o **mínimo** sobre as cápsulas alcançáveis (a própria, os dois
   vizinhos de sequência, e uma **lista lateral** de vizinhos geométricos que a CPU descobriu).
3. Um teste de **depth GREATER estrito** elege **UM** fragmento por pixel por traço — os demais
   são descartados.

É um clean-room do Grease Pencil 5.2 (`draw_grease_pencil_lib.glsl`), adaptado ao 2D ortográfico
e depois divergido de propósito em vários pontos. Detalhe: o cabeçalho de `flip.wgsl` e
[`docs/Flip/03_traco_rasterizacao.md`](Flip/03_traco_rasterizacao.md) §8.6–§8.7.2.

---

## 3. Por que ele não converge — o diagnóstico ESTRUTURAL

Não é uma lista de bugs. São **três propriedades do desenho**, travadas uma na outra, e toda cura
tentada teve de preservar as três:

**(A) A cobertura é função da DISTÂNCIA, não da TINTA DEPOSITADA.**
Um pincel real carimba uma fileira de dabs sobrepostos, e a tinta num pixel é
`1 − Π(1 − dab_k)` sobre **todos** os dabs — uma integral de caminho. Um perfil 1-D da distância
à linha-de-centro só reproduz isso **exatamente numa reta infinita**. Em toda quina, todo
cruzamento e toda ponta o modelo precisa de um **remendo**; cada remendo é um caso especial, e
cada caso especial desta jornada produziu um defeito novo (§4).

**(B) O fragment precisa que alguém lhe CONTE que caminho está perto, por um canal lateral de
tamanho FIXO.**
A tinta num pixel depende de **todo** o caminho dentro do alcance (~3× o raio). O shader conhece
o próprio segmento, dois vizinhos, e uma lista capeada. **Qualquer teto nessa lista é um teto em
quanto caminho pode influenciar um pixel** — e o comprimento de caminho dentro do alcance é
ilimitado, porque a densidade da polilinha é escolha do MOTOR (RDP + reamostragem), não do
artista. Foi exatamente isto que quebrou na 4ª rodada.

**(C) O depth elege UM fragmento por pixel.**
Então o peso inteiro de *"quanta tinta há aqui"* cai sobre esse fragmento — o que **força** (B).
E quando se inverte a eleição (`self_overlap`, depth por-segmento), aparece contagem dupla: cada
face compõe a união GLOBAL, então `N` faces sobrepostas dão `1−(1−u)^N` com o **mesmo** `u`
(**defeito aberto e medido: até 43/255**, `measure_the_self_overlap_double_count`).

⚠️ **A conclusão que a jornada pagou:** (A) obriga a remendo, (B) impõe um teto que não pode ser
seguro, e (C) fecha a porta de saída de (B). **Um motor novo tem de quebrar pelo menos duas das
três.** Se a sua proposta preserva as três, ela vai reencontrar esta família de artefatos.

---

## 4. O CADÁVER — tudo que já foi tentado e MEDIDO. Não repita nada disto.

Cada linha custou uma rodada de smoke do Enio. Os números são contra o **depósito REAL do
Painter** (o oráculo do §6), em nível de alfa de 255.

| # | tentativa | veredito |
|---|---|---|
| 1 | perfil = a queda de **UM DAB** do Painter | **−112** de falta. REPROVADO |
| 2 | perfil = a fileira de dabs do Painter (o perfil de **TRAÇO**) | **−4**. Certo, e **fica** |
| 3 | união (`min`) nas passagens de um cruzamento | vinco na bissetriz; lê como dobra |
| 4 | **composição** por passagem (`1−(1−a)(1−b)`) | certo para cruzamento; **fica** |
| 5 | partição de passagem por **ARCO** | ERRADA: curva fechada volta com arco grande e compunha tinta consigo mesma (196 onde a união pede 184) |
| 6 | partição de passagem **ESPACIAL** (a caminhada sai do alcance) | certo; **fica** |
| 7 | transbordo do teto **carimbado** (some das 2 listas) | **BURACO −252** |
| 8 | transbordo **não carimbado** (vira estranho) | **+63** de tinta a mais |
| 9 | transbordo ao grid, **marcado como própria** (dois carimbos) | certo na época… |
| 10 | teto de vizinhos por **CONTAGEM** de segmentos | **o penhasco**: −184 em passo `0,10·r`, **−255 (a tinta SOME)** em `0,05·r` |
| 11 | teto por **CÁPSULA** (funde segmentos quase-colineares) | desvio **constante −3** de `0,80·r` a `0,04·r`. É o `7ca83d6fb` |
| 12 | critério de passagem por **mínimo local** do perfil de distância | **NÃO MOVEU o número** (−63 → −63). Revertido |
| 13 | supersamplear o oráculo do depósito 4×4 | PIOROU a leitura (mede uma verdade que nenhum dos dois produtos computa). Revertido |

⚠️ **E o item 9 MORREU no item 11, por medição:** com as cápsulas fundidas a caminhada absorve a
passagem inteira, então as duas mutações que o par de carimbos equilibrava pararam de sangrar em
fixture nenhuma. Código morto MENTE, então saiu.

**Dois fatos de diagnóstico que valem mais que a tabela:**

- **O produto atravessa o penhasco DESENHANDO DEVAGAR.** Não é caso patológico: o RDP tolera
  `0,1·r` e a reamostragem **só acrescenta** pontos ⇒ arco de mão a 400 amostras dá passo mínimo
  **0,137·r**; a 1200 (mão lenta) dá **0,108·r**, com **125 de 251** segmentos abaixo da cerca de
  `0,1875·r` (`flip_draw_tests::the_real_pipeline_step_in_radii`).
- **O defeito só aparece com traço ÚNICO porque dois traços têm depth diferente e compõem por
  `over`** — o parceiro do cruzamento **não precisa** da lista lateral. A observação do Enio
  (*"com vários traços fica melhor"*) é o diagnóstico da propriedade (B), não uma preferência.

---

## 5. O que pesquisar — o espaço de soluções, com o que cada uma responde

Você deve pesquisar de verdade e **medir** antes de escolher. O que segue é o mapa que a jornada
deixou, não a resposta.

### Candidato 1 — **ACUMULAR o traço num buffer e compositar UMA vez** (o que os pintores fazem)

GIMP, Krita, Procreate, Photoshop e **o nosso próprio Painter** fazem a MESMA coisa: carimbam
dabs ao longo do caminho num **buffer de cobertura por-traço**, com uma lei de acumulação, e
compositam esse buffer sobre a camada **uma vez**.

Isto quebra as três propriedades de uma vez: não há modelo de distância (o buffer **É** o
depósito), não há canal lateral (o buffer acumula), não há eleição por depth.

⚠️ **É o que o Enio pediu ao pé da letra** (*"o visual de um painter digital normal"*), e a
arquitetura já existe DENTRO deste repo: leia a §13.9–§13.13 de
[`docs/Painter/25_avaliacao_gpu.md`](Painter/25_avaliacao_gpu.md) — inclusive a **lei** de
acumulação (GIMP = taxa · Krita = alvo guardado por `max`/*Alpha Darken*), que **já foi
comparada e decidida** lá, e o achado de que o `max` produz **contas** (beading) num traço.
**Não re-derive isso; leia.**

**A pergunta difícil, e é toda a wave:** o Flip é um renderizador **VETORIAL, re-rasterizado a
cada frame, em qualquer zoom**, com N traços por desenho, ghost frames, multiplano e fill. Um
buffer por traço por frame é caro. Investigue: acumulação por **CAMADA** (não por traço) com a
lei aplicada no composite · alvo `R8`/`R16` de escopo de desenho · batching por material ·
invalidação por traço sujo (o `TessCache` do shell já existe e já é por-desenho).

### Candidato 2 — **Binning por TILE** (o modelo de todo rasterizador vetorial de GPU moderno)

O teto existe porque o canal lateral é **por-segmento e de tamanho fixo**. Um renderizador que
**bina o caminho em tiles** dá a cada tile a lista COMPLETA do caminho que a toca, limitada por
memória e não por uma constante. É como Vello (que já roda neste repo, em `ph2d-vec-render`),
Pathfinder e Slug funcionam.

Isto quebra (B) sozinho. Combinado com uma lei de tinta correta, quebra (A) também.

⚠️ Vello por si só resolve **cobertura de área de uma forma preenchida** — que é o caso
`hardness = 1` e nada além. O pincel macio continua precisando da lei de depósito.

### Candidato 3 — **A integral de caminho analítica** (evolução, NÃO substituição)

Trocar `perfil(min distância)` pela integral de arco
`α = 1 − exp(−(1/spacing)·∫_caminho f(d(s)) ds)`, com `f(d) = −ln(1 − dab(d))`. É o **limite
contínuo** da fileira de dabs: aditiva sobre pedaços de caminho ⇒ fronteira de segmento exata,
quina e cruzamento **compõem sozinhos**, e a partição de passagem e a dicotomia união/composição
**deixam de existir**.

⚠️ **Eu derivei isto e NÃO construí** — está aqui para você não gastar um dia redescobrindo.
E sou obrigado a dizer o que ele é: uma mudança da **lei de cobertura** dentro do mesmo desenho,
não um motor novo. Ele quebra (A), **mantém (B)** (o fragment continua precisando que lhe contem
o caminho) e mantém (C). Pelo brief do Enio ele não é a resposta; pela relação custo/benefício
ele pode ser a melhor coisa a **medir primeiro**, porque é barato e responde numericamente se
(A) sozinha explica o que se vê. **Se você o medir, reporte ao Enio antes de adotá-lo** — a
decisão de escopo é dele, não sua.

### Candidato 4 — as referências a LER

- **Ciallo** (`Ciallo: a stroke rendering engine`) — renderização analítica de traço em GPU, com
  leis por-dab e por-traço. **Já usamos uma peça dele** (o airbrush de Beer-Lambert, `03 §8`);
  o resto do modelo nunca foi lido a fundo. É o candidato acadêmico mais próximo do problema.
- **Blender Grease Pencil** — o ancestral do motor atual. A mordida é **defeito ABERTO lá**
  (issue **#140075**), escondida pelo default `hardness = 1.0` + SMAA. O GP foi reescrito no
  ciclo 4.x/5.x: descubra **o que eles fizeram**, porque é a mesma pergunta.
- **Krita** (`KisPaintOp` / dab accumulation) e **GIMP** (`gimppaintcore`) — a lei de acumulação,
  em código legível.
- **Rive · Lottie · Harmony · TVPaint · Clip Studio** — o que um produto de animação 2D de fato
  shipa para um traço de borda macia, e como resolvem o auto-cruzamento.
- **O nosso Painter** — `ph2d-painter-brush` (`Falloff`, `spec_default.rs`) é o alvo literal, e a
  função dele **já é o oráculo dos gates** (§6).

---

## 6. O ORÁCULO — já existe, é agnóstico de modelo, e é o que decide

Isto é o mais valioso deste handoff. **Não escreva oráculo novo antes de usar estes.**

| ferramenta | onde | o que responde |
|---|---|---|
| `painter_deposit_sized` | `tests/painter_look.rs` | **o depósito REAL do Painter** (chama `ph2d_painter_brush::Falloff`, dabs a `0,1×diâmetro` de arco, compostos por `over`, amostrado no centro do pixel). Qualquer motor pode ser medido contra ele |
| `the_ink_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled` | `tests/sampling_invariance.rs` | **a lei**: a mesma figura, de `0,80·r` a `0,04·r`, tem de pintar a mesma tinta. Qualquer modelo novo tem de passar |
| `measure_the_star_one_stroke_against_separate_strokes` | `tests/painter_look.rs` | **o oráculo do Enio**: um traço vs traços separados, nos DOIS sentidos |
| `render_flip_painter_and_the_difference` · `render_the_slow_hand_star` | `tests/painter_look.rs` | **render-and-look**: FLIP · PAINTER · MAPA DE DIFERENÇA, lado a lado, em BMP |
| `hardness_law.rs` | `tests/` | paridade **termo a termo** com a função Rust do Painter |
| `gpu_render.rs` (36 gates) | `tests/` | tudo o mais que o traço promete |

⚠️ **Duas armadilhas de oráculo que esta jornada pagou, e que você vai reencontrar:**
- **A franja da silhueta tem de ser EXCLUÍDA** (`in_the_fringe`): o Flip tem AA analítico e o
  oráculo do depósito não tem nenhum. Comparar ali mede AA, não a lei da tinta.
- **Não supersampleie o oráculo.** Foi tentado (4×4) e **piorou**: o Painter também amostra no
  centro do texel, então supersamplear mede uma verdade que **nenhum dos dois** computa.
- **A fixture tem de conter o fenômeno.** Quatro rodadas de smoke passaram por cima do defeito
  porque a cena encenava só o traço de **mão rápida** — o lado seguro da cerca. Hoje a cena tem
  um gate provando que ela contém o caso
  (`flip_hardness_smoke::tests::the_slow_hand_star_is_denser_than_the_old_neighbour_fence`).

---

## 7. O SMOKE, e o gesto que julga

```
env PH2D_FLIP_HARDNESS_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

Quatro grupos: X duro (o **controle**, byte-idêntico) · X macio de dois traços · estrela de um
traço **mão rápida** (passo mínimo `1,495·r`) · estrela de um traço **mão lenta** (`0,106·r`).

**O julgamento final é do Enio, e é este:** desenhar **devagar, cruzando o próprio traço, sem
levantar a caneta**, com hardness baixa, e comparar com o **Painter digital** fazendo o mesmo
rabisco. Se a diferença for perceptível, não fechou.

Os outros smokes do Flip que o motor novo não pode quebrar: `PH2D_FLIP_MULTIPLANE_SMOKE=1` ·
`PH2D_FLIP_SELF_OVERLAP_SMOKE=1` · `PH2D_FLIP_AIRBRUSH_SMOKE=1` · `PH2D_FLIP_RESAMPLE_SMOKE=1` ·
`PH2D_FLIP_PRESSURE_SMOKE=1` · `PH2D_FLIP_TIP_SMOKE=1` · `PH2D_FLIP_STRIP_SMOKE=1` ·
`PH2D_FLIP_COLORIZE_SMOKE=1`.

---

## 8. A SUPERFÍCIE que o motor novo tem de entregar (nada aqui é negociável sem o Enio)

Do `FlipStroke` (`crates/ph2d-flip/src/stroke.rs`):

- por-ponto: **posição · largura · opacidade · cor** (a largura é MUNDO; a espessura de tela é
  `raio × px_per_world` ⇒ zoom engrossa);
- por-traço: `closed` · `cap` (Flat/Round, **por ponta**) · `hardness` · `tip`
  (Continuous/Dots/Squares) + `dot_spacing` · `self_overlap` · `airbrush` · `material` ·
  `fill` + `holes` · `hide_stroke` · `selected`.

Do desenho / do shell: **multiplano** (paralaxe por camada) · **ghost tint** (onion: silhueta
recolorida) · **fade sub-pixel** (traço mais fino que um pixel perde opacidade, não largura) ·
**fill** com furos · o **overlay do colorize** · e o **dobramento do preview ao vivo**
(`FlipGpuData::append`, que rebasa índices para o traço em curso compor na camada ativa).

⚠️ **`DEFAULT_HARDNESS = 1.0` é o default do produto** e todo o acervo já desenhado passa por
ele: um motor novo tem de deixar o traço duro **byte-idêntico**, ou trazer a medição que
justifique a diferença. É o CONTROLE de todos os smokes.

---

## 9. Raio de explosão

- **Sai / é reescrito:** `crates/ph2d-flip-render/src/shaders/flip.wgsl` (750) ·
  `neighbors.rs` (587) + `neighbors_tests.rs` (397) · a metade de `pack.rs` (631) que monta
  `seg_extras`/`seg_extra_range` · partes de `pipeline.rs` (522).
- **Provavelmente fica:** `fill.rs` · `fill_holes.rs` · `composite.rs` + `composite.wgsl` ·
  `flip_fill.wgsl`.
- **Testes:** 71 `#[test]` em 10 arquivos. Muitos são do MODELO (união, vizinhos, first-wins) e
  morrem com ele; os do §6 são de COMPORTAMENTO e **sobrevivem** — são a sua rede.
- **Fora da crate:** `shells/desktop/src/render_loop/flip_pass.rs` (multiplano, preview) e o
  `TessCache`.
- **Schema:** nenhum. `PROJECT_SCHEMA` **37** · `FLIP_SCHEMA` **12** · tripla do pin
  `(37, 12, 13)`. Um motor novo **não deve** precisar tocar em nenhum: ele lê o mesmo documento.

---

## 10. O que NÃO re-litigar (decisões pagas, com dono)

1. **A lei de dureza é a do PAINTER, não a do Grease Pencil** — decisão do Enio, 2026-07-28, com
   foto lado a lado. E é a do **TRAÇO** (a fileira de dabs composta), não a de um **DAB**
   (medido: em hardness 0,4 e `dn = 0,70` um dab pesa 0,500 e o traço pesa 0,916).
2. **Determinismo é contrato** (replay-hash): mesmo desenho ⇒ mesmo buffer. Ordenações precisam
   de desempate estável.
3. **HR-5** no caminho determinístico. O `exp` do airbrush vive só na GPU.
4. **Zoom:** largura em MUNDO; nada de limiar em px de tela na lei da tinta.
5. **A reamostragem (`RESAMPLE_STEP_FRACTION = 0.4`) e o RDP (`0.05`) são da AUTORIA**, não do
   render — e o motor novo **não pode depender da densidade que eles produzem**. Foi exatamente
   essa dependência que matou o atual.

---

## 11. A primeira coisa a fazer

1. FASE 0 (§0). `git rebase main`.
2. Rode o smoke uma vez (§7) e as sondas do §6 — **saiba de que baseline você parte**, em números
   e em imagem, antes de propor qualquer coisa.
3. **Pesquise** (§5). Traga ao Enio uma comparação com **o que cada candidato quebra das três
   propriedades do §3**, o custo por frame, e o que ele faz com as features do §8.
4. **Não escreva motor antes do plano aprovado.** Esta linha já gastou quatro rodadas de smoke
   consertando a peça errada; a quinta tem de começar pela pergunta certa.
