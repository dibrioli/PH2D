# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, 2026-08-09 (MESTRE)

**Status:** FECHADO 2026-08-09 · no `main` em `5bfe201e0` (o commit que trouxe este arquivo).

> **Para o agente integrador.** Este documento **supersede** o
> [`MESTRE_2026-08-08`](HANDOFF_INTEGRACAO_line_sculpt3d_MESTRE_2026-08-08.md)
> apenas como *o que integrar agora*; ⚠️ **o detalhe de mecanismo das waves até a
> W10.7 continua LÁ e não foi copiado para cá.**
>
> **Estado:** a linha está FECHADA. **Todos os smokes aprovados pelo Enio**, o
> último em 2026-08-09 (*"Perfeito! Smoke OK"*).

---

## 1. O que é, em uma frase

**Vinte commits** que fecham a W14→W17 do módulo 3D — o **Extract**, o
**Transform**, o **ambiente com direção**, o **alpha por IMAGEM** (uma sprite da
cena vira o padrão do pincel) e o **pill SCULPT** que dá ao modo uma porta de
entrada — mais as três rodadas de report do Enio sobre o carimbo, que terminam
com ele **preso ao viewport e sem âncora nenhuma**.

```
98 arquivos · +10.656 / −597
```

---

## 2. A tabela de colisão — leia esta primeira

| Eixo | Valor | Como foi conferido |
|---|---|---|
| **`PROJECT_SCHEMA`** | **INTOCADO** | `git diff main...HEAD -- shells/desktop/src/project.rs` → **vazio** |
| **ADR novo** | **NENHUM** | `--diff-filter=A` sobre `docs/architecture/decisions/` → vazio |
| **Registro do `ph2d-ecs`** | **INTOCADO** | `git diff --stat -- crates/ph2d-ecs/` → vazio (⚠️ e por isso os **três** espelhos também) |
| **Crate nova** | **NENHUMA** | `--diff-filter=A` sobre `*/Cargo.toml` → vazio |
| **Dep EXTERNA nova** | **NENHUMA** | `git diff -- Cargo.lock \| grep "^+name"` → vazio |
| **`Cargo.toml` tocado** | **UM** | `ph2d-panel-sculpt3d` ganha `ph2d-mesh` — **aresta interna de path**, para o painel ler o `Extract::default()` em vez de guardar uma 2ª cópia dos dois números |
| **Contrato congelado** | **4/4 + 3/3** | `architecture_tool_contract_surface` + `architecture_contract_surface`, rodados |
| **ids novos** | 17, **todos `hash_node_id`** | string-hash ⇒ **nenhum gate de contagem**; cobertos pelo `node_id_collisions` |
| **Scrollbar id** | **nenhum novo** | o painel já tinha o **840** |

⇒ **Esta linha fica FORA de toda disputa de número da janela.** Ela não pede
degrau de schema, não reivindica ADR e não move contador nenhum.

---

## 3. ⚠️ A superfície FORA do módulo — os 5 arquivos que podem conflitar

O módulo 3D é drop-crate por desenho (ADR-0150), e **o único assunto que sai
dele é o PILL**. São cinco edições, todas **aditivas**:

| Arquivo | O que entra | Risco |
|---|---|---|
| `ph2d-editor-core/src/action_bus.rs` | variante **`ToggleSculpt3d`** (apendada) | add/add se outra linha apendar variante — resolve mantendo as duas |
| `ph2d-editor-core/src/ids/chrome/topbar.rs` | `TOPBAR_SCULPT3D` (hash de string) | baixo |
| `screens/hero/topbar/mod.rs` | o cluster **depois do FLIP** + o registro + o tooltip | ⚠️ **a POSIÇÃO é load-bearing** — ver abaixo |
| `screens/hero/topbar/chip_name.rs` | um braço `"Sculpt 3D"` | baixo |
| `screens/hero/chrome/mod.rs` | `mod sculpt3d_toggle;` + um `\|\| sculpt3d_toggle::apply(...)` na cadeia | baixo |

⚠️ **A posição do cluster no topbar não é gosto:** os **sete primeiros** clusters
são o grupo da ESQUERDA (o `split` do `paint_top_bar`), então entrar entre eles
empurra o vizinho de baixo para o outro lado da tela. O pill entra **depois do
FLIP**. Se o merge o mover, o layout quebra sem nenhum gate reclamar.

⚠️ **`ToggleSculpt3d` NÃO é `ActivateTool`, e a distinção é o que o ADR-0150
protege:** a cena 3D **não é uma `Tool`** (a navegação orbital mora no shell de
propósito, e é isso que mantém `Tool=12` fora do caminho), então não há `tool_id`
que a registry saiba ativar.

Mais quatro do shell, todos do próprio dono (`app_state` +13 · `forwarding` +84 ·
`main` +2 · `render_loop/mod` +117).

---

## 4. As entregas

### 4.1 W14 — o **EXTRACT**: a máscara vira uma PEÇA (cena `=22`)

A região mascarada é destacada como um objeto novo, com **espessura** e
**passadas de costura** autoradas. Kernel na `ph2d-sculpt3d`, os dois números
vêm do `Extract::default()` do `ph2d-mesh` — daí a aresta de path nova.

### 4.2 W15 — o **TRANSFORM**: a máscara MOVE (cena `=23`)

Mover / girar / escalar **só a parte mascarada**, com três chips e o quarto Drag.
⚠️ O `W15b` corrigiu o giro para **acompanhar a mão** — o eixo vem do raio que
agarrou o barro, não de um ângulo de repouso.

### 4.3 W16 — o **AMBIENTE TEM DIREÇÃO** (cena `=24`)

O piso da difusa deixou de ser **um número para toda direção**: o topo da tela e
o fundo recebem valores distintos, **com a média sobre todas as normais
inalterada** — o termo *redistribui*, não *expõe*.

⚠️ **A cena traz LUZ PRÓPRIA, e a fixture anterior não continha metade do
termo:** com o rig de todo dia (que vem de CIMA, do mesmo lado em que este
ambiente põe o céu) apenas **11,5%** da sombra visível recebe a metade clara — o
artista veria *só escurecer*. A cena põe uma lâmpada **rasante pela direita**,
onde o efeito é o que ele é.

### 4.4 W17 — o **ALPHA POR IMAGEM** (cena `=25`)

Uma **sprite da cena** vira o padrão do pincel. ⚠️ **A escolha e os pixels são o
MESMO valor** (`Alpha::Image(Arc<AlphaImage>)`): *`Image` sem imagem é
inexprimível*. As duas alternativas (um id numa tabela · a imagem como parâmetro
do dab) deixam esse estado nascer, e ele significa *"liso"* em silêncio.

**A lei:** `peso = luminância(rgb) × alfa`, convertida **UMA vez na carga**, um
byte por texel. ⚠️ **Consequência NOMEADA:** um desenho de tinta PRETA sobre
transparência vira um alpha **vazio** — é o que o ZBrush faz, pela mesma razão
(*branco é cheio*).

⚠️ **E ela fica FORA do `Alpha::ALL`**, de propósito: aquela lista é o que a UI
oferece como CHIPS, e um chip é um nome. Uma imagem é uma coisa para a qual se
aponta ⇒ o gesto que a arma é um **BOTÃO**.

**O `Copy` do `Alpha` morreu**, e o preço foi **MEDIDO antes da decisão**: o
compilador conta **3 derives** + cinco `.clone()` num arquivo de teste — contra a
estimativa herdada de *"~20 arquivos"*, que era o que arquivava esta saída.

### 4.5 O **PILL SCULPT** — entrar e sair do modo

O modo 3D tinha de ser aberto por variável de ambiente. Agora há um pill, e com
ele três correções que o smoke pediu: a cena 3D **nunca toma um clique que é da
MOLDURA** (era por isso que o pill não saía) · o painel do modo **segue as bordas
do barro** (fecha ao sair, reabre ao voltar) · e a luz assada **só é re-autorada
por um GESTO**.

### 4.6 As três rodadas do CARIMBO — e a última é a que vale ler

**(a)** O carimbo ganhou **NOME** e o terceiro ajuste: **onde ele pousa**.

**(b)** *"Pattern Offset parece sem efeito"* — ⚠️ **o motor sempre esteve certo**
(medido: um passo do slider muda **10.159 dos 13.682 vértices**). Quem estava
cego eram as **duas superfícies que o mostram**: as duas chaves de cache
enumeravam `az`/`elev` e não o deslocamento. *Uma chave que enumera as entradas
de um valor é como a próxima é esquecida* ⇒ as duas passaram a carregar o
**`AlphaFrame` inteiro**, que é exatamente o que o `weight_at` consome, então uma
entrada nova chega **por dentro do tipo**.

**(c)** *"a projeção da imagem externa (e apenas dela) seja screen"* — a imagem
virou **estêncil preso ao viewport**, e as duas metades do pedido (não girar com
o objeto, não mudar com o zoom) saíram de **um número só**.

**(d) E então o report que fechou a jornada:** *"a tinta da máscara projetada no
objeto não corresponde ao que realmente está sendo esculpido"*.

⚠️ **O defeito, medido antes de qualquer hipótese:** o estêncil media *quantas
unidades de objeto a altura da tela abrange* — um número que só é verdade **numa
profundidade**. Quem o montava tinha de escolher ONDE perguntar, e os dois
consumidores escolheram diferente: **o dab no ACERTO, o preview no CENTRO da
peça**. A régua vale **3,3456 na frente e 4,1740 no centro** ⇒ o carimbo
desenhado saía **24,8% maior** que o depositado, e o erro **mudava conforme o
artista andava pela peça** (−16,6% atrás; razão frente/atrás **0,6688×**).

⚠️ **A cura não foi escolher a âncora certa — foi TIRAR A PERGUNTA.** O
`AlphaStencil` guarda hoje o **FRUSTUM** (`eye` + `height_per_depth`) e a
profundidade entra **por vértice**; a `stencil_for(pose)` **não recebe ponto
nenhum**, e os dois consumidores fazem literalmente a mesma chamada. É a
representação apagando o caso especial — o movimento da bola limitada do Inflate,
que matou quatro cercas do Painter de uma vez.

E ganha-se o que **nenhuma âncora podia dar**: dois pontos que caem no mesmo
pixel recebem a mesma coordenada de carimbo, no nariz ou na nuca do modelo (com
âncora, o carimbo ainda "nadava" ~18% de tamanho aparente até a silhueta).

---

## 5. Três propriedades que mantêm isto barato — e verificável

* **A razão do frustum é ADIMENSIONAL** (mundo por mundo) ⇒ ela **não** é
  dividida pela escala da peça; a pose entra **uma vez só**, no olho. Dividi-la
  duas vezes é a mutação que o gate da peça escalada mata (**0,7502** de desvio).
* **`eye = [0,0,0]` + `persp = 0` reduzem LITERALMENTE** (`p − 0.0` é `p` ao bit,
  e o ramo não é tomado) ⇒ **os nove procedurais atravessam byte a byte**, com
  gate que passa a vista aos **DOIS** lados (não basta um procedural sem estêncil
  não mudar — ele tem de ignorar um estêncil que **está** ali).
* **O retrato do painel** amostra o plano `z = 0`, cuja profundidade contra o
  olho CANÔNICO é exatamente `1` ⇒ a divisão vira `1.0/(1.0·1.0)` e o swatch sai
  **sem distorção**, que é o que um retrato deve ser.

⚠️ **A razão sai da porta que JÁ EXISTE** (`view_height_per_depth` mede no alvo e
divide pela distância), e **não** da forma fechada `2·tan(fov/2)`: o doc da
`world_radius_for_screen_px` já adverte que a fechada seria a segunda cópia das
mesmas grandezas, divergindo por um fator que nenhum teste de nenhuma das metades
enxerga.

---

## 6. Gates e mutações

**Novos nesta jornada** (só o cluster do carimbo; as W14–W17 trazem os seus):

* **`the_stamp_does_not_swim_with_depth`** — o gate central. Com a régua presa a
  UMA âncora (o modelo anterior, reinstalado como mutação) o mesmo par de
  profundidades desvia **1,0** (o campo inteiro, de preto a branco), e a mesma
  mutação **derruba junto o gate do zoom**.
* **`an_offset_of_one_stamp_walks_exactly_one_tile`** — o deslocamento só era
  exercitado no retrato, onde a razão vale `1,0` e toda conversão espúria é a
  identidade. A mutação que o re-converte pela razão sangra em **1,0**.
* **`a_scaled_piece_wears_the_same_stamp_on_screen`** + o irmão da função pura,
  num módulo de teste NOVO (`sculpt3d_space_tests.rs`) — ⚠️ o `stencil_of` é uma
  função **LIVRE** justamente para um gate de CPU a alcançar: montar a cena exige
  `wgpu::Device`.
* **o arch-gate do preview ganhou a SEGUNDA METADE** — ele afirmava que o preview
  RECEBE a vista, e o defeito era o preview receber a vista e montá-la com **outra
  âncora**. Hoje ele afirma que os dois sítios fazem a **mesma chamada, letra por
  letra**, com controle positivo do lado do dab.

**5 mutações, 5 sangram.**

⚠️ **A barra dos oráculos foi MEDIDA, não escolhida:** a mesma fração de tela
alcançada por caminhos aritméticos diferentes não volta bit a bit (**9,85e-6**
entre as profundidades 4 e 6) e o defeito desvia **1,0** — cinco ordens de
grandeza separam o ruído do bug.

⚠️ **DOIS defeitos de fixture meus, os dois pegos por reprovarem produto
CORRETO:** o controle do gate do deslocamento andava **meio ladrilho**, e as
bandas do carimbo repetem a cada **quarto** de ladrilho — meio ladrilho são dois
períodos e devolve o mesmo campo. *Um controle tem de andar menos que o período
do que ele controla.* E o primeiro `assert_eq!` exato media **arredondamento** em
vez do modelo.

⚠️ **Um gate existente reprovou produto correto pelo motivo de sempre** — o
`the_brush_radius_is_screen_pixels…` contava as menções de uma ajudante da câmera
como procuração para *"quantas respostas há sobre o tamanho do pincel"*, e as
duas perguntas **deixaram de partilhar a ajudante** (o pincel pergunta *quanto
mundo cabe em N pixels AQUI*; o estêncil pergunta a RAZÃO, que não depende de
ponto). Reescrito para contar cada pergunta no seu sítio.

---

## 7. Números — e o controle ao lado de cada um

**O alpha por IMAGEM já é ASSADO** (pergunta do Enio: *"a sprite está sendo usada
como procedural? se assar fica mais otimizado?"*). `AlphaImage::from_rgba`
converte RGBA8 em um byte de peso por texel **uma vez, na carga**; a amostra é
uma busca bilinear. Medido sobre a malha de **426k vértices**, máquina calma:

| padrão | ns/vértice |
|---|---|
| **Imagem 64²** | **17,1** |
| Imagem 2048² | ~20 |
| Scratches / Weave | 9,1 |
| Noise | 79,8 |
| Scales / Pores / Cracks | ~149 |

⇒ **a imagem é o mais barato dos dez** tirando os dois mais ralos, e o tamanho
quase não entra (64² → 2048² custa 17,1 → ~20 ns: o que se paga é **cache**, não
aritmética). **Não há o que assar.**

⚠️ **E a primeira medição pós-fix acusou +114% na imagem** — com o `Noise`, que
esta linha não toca, tendo subido **46% na mesma corrida** (`load average 23,7`).
Com a máquina em **1,70** o controle volta à linha de base e a imagem mede
**17,1 contra 16,8** ns/vértice: a divisão de perspectiva (uma subtração e uma
divisão por vértice) está **dentro do ruído**. *Um número que se move sobre
código intocado é a máquina, não o código.*

---

## 8. Verificação de fechamento (rodada, não auto-relatada)

| Bateria | Resultado |
|---|---|
| Shell (`ph2d-host-desktop`) | **2748 alvos verdes**, 0 falhas |
| Crates do módulo + editor-core + i18n | **565 alvos verdes**, 0 falhas |
| **Gates de GPU** (`--ignored`, na RTX) | **45 verdes** (eram 44 no MESTRE anterior) |
| clippy `--all-targets` (4 crates tocadas) | limpo |
| LOC (workspace · painel · widget · shell) | verde |
| Contrato congelado | **4/4 + 3/3** |
| `project.rs` | diff **VAZIO** |

⚠️ **Os gates de GPU são `#[ignore]` e precisam de adapter** — sem ele fazem
*skip gracioso*, **que não é verde**. Rode-os: `cargo test -p ph2d-mesh-render
--release -- --ignored`.

⚠️ **Rode a suíte em DEBUG também.** O precedente é do repo (o
`ph2d-flip-colorize` panicava só em debug, e a nota sobreviveu ao fato por três
integrações). Feito aqui: verde nos dois perfis.

---

## 9. Smokes

Todos `--release`, com `env PH2D_SCULPT3D_SMOKE=<n>`:

| n | O quê | O que julgar |
|---|---|---|
| `=22` | **O EXTRACT** | a máscara vira uma peça, com espessura |
| `=23` | **O TRANSFORM** | a máscara MOVE, e o giro acompanha a mão |
| `=24` | **O AMBIENTE TEM DIREÇÃO** | ⚠️ olhe a **metade ESQUERDA** (a que a lâmpada não alcança): topo e beiral de cada degrau têm de se separar |
| `=25` | **O ALPHA POR IMAGEM** | arme o sprite; **gire** (o carimbo não acompanha), **dê zoom** (não muda de tamanho), e **pinte olhando o carimbo** — o que a máscara desenha é o que o traço deposita, **em qualquer parte do modelo** |

⚠️ **Três cenas imprimem o número que as torna válidas** (quantas arestas de
beira, quanto mede a maior aresta, quantas peças abriu). **Se a linha não
aparecer, o resto do smoke não diz nada.** E **rode uma vez SEM a env var**: é a
metade que prova a inércia do frame 2D.

---

## 10. Aberto, com o preço ao lado

* **O preview no barro recalcula por quadro de órbita** enquanto um carimbo está
  armado — medido **0,36–0,46 ms** a 13,7k vértices e **7,9–10,1 ms** a 426k. É
  inerente: um estêncil muda de lugar no barro a cada movimento da câmera e
  **nenhum vértice se move**, então um bake por-vértice é invalidado pelo quadro
  seguinte, por definição. Os nove procedurais **não** respondem à câmera, e há
  gate de CONTROLE afirmando isso.
* **O K1/K2 do ADR-0150** seguem como o MESTRE de 08/08 os deixou.
* O resto da lista aberta do módulo (import/export, objeto misto, merge/isolate,
  marching cubes) não foi tocado por esta jornada.

---

## 11. O que a linha NÃO fez, de propósito

Ela **não integrou** e **não fez push**. Fecha aqui, entrega este documento e
**para** — integração e ship são do Enio, por ordem explícita, via agente
integrador dedicado (CLAUDE.md §0.7, DIRETRIZ §1.5.3–1.5.4).
