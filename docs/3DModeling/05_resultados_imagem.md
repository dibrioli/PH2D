# A imagem: anti-serrilhado, resolução e composição — o que foi MEDIDO (2026-08-19)

> Motivo: *"smoke ok, contudo render pixelado"* (Enio, no smoke da W3). O módulo cuja razão de
> existir é a nitidez da aresta estava a entregar a aresta em degraus.
>
> **Ler antes:** [`01_resultados_spike.md`](01_resultados_spike.md) §1c (por que a tela não passa
> pela malha) · [`04_resultados_perfis.md`](04_resultados_perfis.md) §3 (o custo do traçado).

---

## §1 — Eram TRÊS coisas, e só uma era o que o nome dizia

| # | Causa | O que era |
|---|---|---|
| 1 | **Um raio por pixel, cobertura binária** | Cada pixel era peça ou fundo, sem meio-termo. É isto que se vê como escada — e é a causa principal |
| 2 | **Traçado num tamanho fixo, reamostrado para a área** | 640×480 desenhados numa área de outro tamanho e outra proporção: metade da informação perdida antes de chegar à tela |
| 3 | **Composição com alfa direto e filtro bicúbico** | O filtro mistura a cor de pixels transparentes — cuja cor não significa nada — e o bicúbico *toca* numa aresta de alto contraste |

⚠️ **Uma quarta hipótese foi levantada e NÃO confirmada:** a amostragem do matcap era
vizinho-mais-próximo, o que produz banda numa superfície lisa. Passou a bilinear porque é o certo em
qualquer tamanho — mas **no matcap da casa (749²) a grelha é fina o bastante para o efeito ser
pequeno**, e nada foi medido a atribuir-lhe o sintoma relatado. *Uma correção certa não precisa de
reivindicar um bug que ninguém provou que ela cura.*

---

## §2 — O anti-serrilhado é ADAPTATIVO, e a aritmética é a razão

Supersamplear a imagem inteira a 4× custa 4×. Mas a serrilha só existe onde há **aresta** — e
aresta é **0,5 % a 1,2 % dos pixels** (medido). Então:

1. um raio por pixel;
2. deteta-se onde a imagem tem descontinuidade;
3. **só esses** levam as quatro amostras do padrão **4-rook (RGSS)**.

### §2.1 — O detector olha DUAS coisas

| Sinal | Apanha |
|---|---|
| A **máscara** (`hit`) | A silhueta contra o fundo |
| A **normal** (`dot < 0,9`, ≈ 25°) | ⭐ A **quina viva**, e uma superfície que passa à frente de outra |

⚠️ Um detector só de máscara deixaria serrilhada **exatamente a aresta que este módulo existe para
entregar afiada** — a quina não muda a máscara, os dois lados acertam. Há gate
(`the_edge_detector_sees_a_sharp_crease_inside_the_mask`), e ele conta pixels de borda cujos quatro
vizinhos *e* quatro amostras acertam: só uma quina produz isso.

### §2.2 — Por que a grelha é ROTACIONADA

Quatro posições numa grelha alinhada dão **duas** posições distintas a uma aresta quase horizontal —
as duas amostras de cima caem do mesmo lado. A rotacionada dá **quatro**: o dobro dos níveis de
cobertura exatamente nas arestas que mais aparecem. É o resultado que a indústria mediu há trinta
anos, e é portado, não redescoberto.

### §2.3 — A resolução é em COR, e a média é em LINEAR

⚠️ Duas escolhas que não são gosto:

- **Cor, e não normal.** Onde uma superfície passa à frente de outra, as duas normais podem ser
  quase opostas, e a média delas aponta para um sítio do matcap que não é nenhuma das duas cores.
  Média de normais interpola a *geometria*; o que se quer é interpolar o que se **vê**.
- **Linear, e não bytes sRGB.** Metade de branco com metade de preto não é cinza-127, é cinza-188.
  Fazer a média em sRGB escurece **toda** borda — é o bug clássico de anti-serrilhado e o mais
  difícil de ver, porque parece só "um contorno".

E a saída passa a ser **pré-multiplicada**: a imagem vai ser filtrada ao ser desenhada, e num alfa
direto o filtro mistura a cor de pixels transparentes. O sintoma é a auréola escura à volta da peça.

---

## §3 — ⭐ O 73× que não era o preço do raio

A primeira medição do AA foi má:

| Cena | sem AA | com AA | delta |
|---|---:|---:|---:|
| perfil de 64 arestas, 640×480 | 23,3 ms | **65,4 ms** | **+180 %** |

Isso dava **73× por raio de borda** contra um raio comum — e os dois marcham o mesmo campo com o
mesmo código. *Um número absurdo é uma pergunta, não um resultado.*

Não era o raio. Era `EDGE_CHUNK = 4096`: com ~9 000 amostras de borda, a segunda passagem produzia
**três lotes** — e corria com **3 threads** enquanto a primeira usava as 32.

Com `EDGE_CHUNK = 64`:

| Cena | sem AA | com AA | delta |
|---|---:|---:|---:|
| junção de 3 cilindros, 640×480 | 7,1 ms | **7,3 ms** | **+3 %** |
| junção de 3 cilindros, 1600×1200 | 21,2 ms | **26,2 ms** | +24 % |
| perfil de 64 arestas, 640×480 | 26,0 ms | **29,9 ms** | **+15 %** |
| perfil de 512 arestas, 640×480 | 168,7 ms | **195,5 ms** | +16 % |

⚠️ **A lição, escrita para a próxima pessoa:** *um número de paralelismo dimensionado por uma
intuição sobre overhead — "grande o bastante para o custo de montar um tape desaparecer" — e não
pela **contagem de lotes** que ele produz, é um `for` sequencial com um `par_` na frente.* O gate de
byte-identidade passava o tempo todo: ele mede correção, não ocupação.

### A fração de borda, que é o que justifica a adaptação

| Quadro | bordas | % da imagem |
|---|---:|---:|
| 640×480 | 3 747 | 1,22 % |
| 1024×1024 | 7 998 | 0,76 % |
| 1600×1200 | 9 365 | 0,49 % |

A fração **cai** com a resolução (a borda é um perímetro, a imagem é uma área). Se algum dia ela se
aproximasse de 1, a adaptação deixaria de valer a pena e o certo passaria a ser 4× uniforme — e
`measure_antialias_cost` é o instrumento que diria isso.

---

## §4 — O traçado sai no tamanho REAL da área

O tamanho fixo virou o tamanho da área de desenho, e o desenho passou a ser 1:1 com filtro
**bilinear** em vez de bicúbico: no caso normal os dois são a identidade, mas o bicúbico *toca* numa
aresta de alto contraste — e um halo posto pelo filtro seria o próprio artefato que se acabou de
remover.

⭐ **E a rotação do prato passou a ser por SEGUNDO, não por quadro.** Com um passo por quadro, a
velocidade da peça era função do custo do traçado: baixar a resolução acelerava a rotação e subi-la
travava-a. Isso confunde as duas perguntas que um prato giratório responde — *"a forma está certa?"*
e *"isto corre depressa?"* — e faz a segunda mentir sobre a primeira.

---

## §5 — O que ficou por fazer, com o número ao lado

| Item | Estado |
|---|---|
| **8 amostras por borda** (9 níveis de cobertura em vez de 5) | ⛔ **Não feito.** Custaria ~+15 % em cima dos +15 % atuais — está **disponível**, e a qualidade ainda não o pediu. 4 amostras é o que a indústria shipa |
| **Perspectiva** | Aberto desde a W2. Muda o *feel* de um modelador e merece comparação lado a lado, não uma troca silenciosa |
| **Órbita por mouse** | ✅ **Feita** — ver §6 |
| **O sintoma de BANDA** no sombreado | ⚠️ Não reproduzido nem atribuído. Se voltar a aparecer, os suspeitos por ordem são: o próprio matcap (um matcap de esfera iluminada **tem** anéis tonais), a saída de 8 bits, e a normal por diferença central |

**Instrumento para olhar:** `dump_frame` escreve um PPM por variante (com e sem AA) em
`PH2D_FIELD_DUMP` — porque aqui a imagem **é** o produto, e nenhum número substitui um par de olhos.

```
PH2D_FIELD_DUMP=/tmp cargo test -p ph2d-field-render --release -- --ignored --nocapture dump_frame
```

---

## §6 — A navegação (W4, 19/08): o que se fez, e o teto que DEIXOU de existir

Órbita, pan e zoom pelo mouse — em [`field3d_input.rs`](../../shells/desktop/src/field3d_input.rs),
no **shell** e nunca numa `Tool` ([ADR-0150](../architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md)),
que é o que mantém o `Tool=12` congelado fora do caminho.

| Gesto | O que faz |
|---|---|
| Arrastar **esquerdo** ou **direito** | Roda — **livre**, em torno do eixo que o arrasto nomeia na tela |
| Arrastar **do meio** | Pan |
| **Roda** | Zoom |
| **Home** | Repõe a vista |

⚠️ **Os botões e as constantes são os MESMOS do módulo de escultura** (`ORBIT_RAD_PER_PX = 0,01`,
`1,1` por passo de roda). Não é herança por analogia — são duas janelas 3D no mesmo aplicativo, e
uma mão que aprendeu a girar numa tem de girar na outra. Divergir seria uma decisão, e não havia
nenhuma a tomar.

⭐ **O prato para de girar ao primeiro toque.** *Feature nova = auto-play* é a lei da casa, mas
continuar a girar **depois** de alguém a ter posto num ângulo é desfazer o gesto dele a cada quadro.
E a partir daí só se traça o que mudou: **uma peça parada custa zero**.

### §6.1 — ⭐ O teto de zoom não foi escrito; a causa dele foi removida

A tolerância de acerto era `2·10⁻⁴` **fixa**. Isso não é uma constante de conforto: é um **teto de
zoom disfarçado**. Assim que o pixel fica mais fino que ela, a superfície deixa de ganhar nitidez e
passa a ganhar franja — e numa peça pequena a forma sai **inchada**:

| Peça de raio 10⁻³, enquadrada a 2·10⁻³ | Área na tela |
|---|---:|
| A geometria | 0,196 |
| Com tolerância FIXA | **0,283** (+44 %) |
| Com tolerância derivada do pixel | 0,198 |

O 0,283 não é uma estimativa: é o número que a **prova de mutação** devolveu ao voltar a fixar as
tolerâncias, com o gate `zooming_in_does_not_inflate_the_part` vermelho.

⚠️ `CLAUDE.md §0` proíbe escrever um limite antes de medir — e o que a medição disse aqui foi que o
limite **não devia existir**. As duas tolerâncias (acerto e diferença central da normal) passam a
descer com o tamanho do pixel, e param num piso que **nomeia o seu recurso**: a precisão de `f32`
(`PRECISION_FLOOR = 10⁻⁶`). *A cura de um teto herdado quase nunca é um número melhor; é a causa.*

Os dois únicos limites de faixa que sobraram dizem de que recurso são:

| Limite | Recurso |
|---|---|
| `MIN_HALF_EXTENT = 10⁻⁴` (~8000× de aproximação) | **Precisão da representação** — abaixo, um pixel mede menos que o erro de `f32` |
| `MAX_HALF_EXTENT = 4,0` | **Alcance da marcha** — os raios cobrem ±4 de profundidade em torno do alvo |

### §6.2 — Os gates medem o MODELO NA TELA, nunca o sinal

`dragging_right_turns_the_model_right_and_dragging_down_shows_its_top` traça uma esfera na face de
cá, aplica o arrasto e mede **para onde o centro de massa foi**. É deliberado: a `line/sculpt3d`
errou os dois sinais de uma vez argumentando sobre `yaw += dx`, e o que os pegou foi um smoke.

⚠️ **Prova de mutação feita:** trocar os dois sinais deixa o gate vermelho com
`59,5 -> 46,5` — ele morde.

A peça em `+Z` (e não em `+X`) também não é decorativa: de frente ela cai **no centro** do quadro,
então qualquer giro a tira de lá e o lado para onde ela sai responde à pergunta sem ambiguidade.
Uma peça em `+X` começaria deslocada e giraria *para dentro* — a armadilha desta medição.

---

## §7 — ⭐ A rotação LIVRE: um `clamp` é o remédio do sintoma; a causa era a representação

> *"smoke ok, mas só rotaciona em uma direção. não tem rot livre"* — Enio, 2026-08-19.

A primeira versão da navegação usava a câmera de dois ângulos que a casa já tinha
(`ph2d_mesh_render::camera`): `yaw` em torno do Y do **mundo**, `pitch` preso por um `clamp` a ±90°.

Isso tem **polos por construção**. Com o enquadramento inicial já a 30° de cima, arrastar para baixo
bate na parede ao fim de ~105 px — meio centímetro de rato. A partir dali o gesto vertical não faz
nada, e o que se vê é literalmente *"só roda para um lado"*.

⚠️ **Nenhum número melhor resolve isto.** Dois ângulos não conseguem exprimir uma orientação livre;
o `clamp` não é a doença, é o curativo obrigatório de uma representação que degenera no polo. A cura
foi guardar a **orientação inteira** (um quaternion) e compor a rotação **pela direita** — no
referencial da própria câmera:

```text
eixo (local) = normalizar(−dy, −dx, 0)     ⟵ perpendicular ao arrasto, no plano da imagem
ângulo       = |(dx, dy)| · 0,01 rad/px
q ← q ⊗ quat(eixo, ângulo)
```

⭐ **Nenhum eixo do mundo entra na conta** — e é exatamente daí que vem a ausência de polo. Some o
`clamp`, some a constante `MAX_PITCH`, some o caso especial. *A representação apaga o caso especial.*

| O que se perdeu | O que se ganhou |
|---|---|
| O horizonte deixa de ser fixo (uma câmera de dois ângulos nunca inclina) | Rotação sem parede, em qualquer direção, incluindo diagonais |

O preço está **aceite e resolvido**: a tecla **Home** repõe o enquadramento de abertura
(`Orbit::from_yaw_pitch`, que continua a existir para escrever vistas nomeadas). Rotação livre sem
uma volta seria uma armadilha, e é do tipo que só se nota depois de entregue.

⚠️ **Uma decisão de produto fica ABERTA:** a maioria dos CAD oferece as duas — livre e prato
giratório com o horizonte travado. Hoje só há a livre, porque foi ela que o Enio pediu; o interruptor
pertence ao painel (W4.2), e não a uma tecla escondida.

### §7.1 — O gate reprovou por medir a coisa errada, e isso vale mais do que o gate

A primeira tentativa de provar *"não há polo"* seguia o **centro de massa da peça na tela** e
chamava polo a qualquer passo em que ele não se mexia. Reprovou com **1 parada em 200** — e a parada
era real e não era um polo: o centroide de um ponto que gira num plano descreve uma **senóide**, e
uma senóide tem pontos de retorno onde a velocidade é zero **por geometria**.

*O sintoma que se procura e o sintoma que se mede podem ter a mesma forma por motivos diferentes.*

A versão que ficou afirma algo **exato** em vez de estatístico: cada passo é uma rotação de `|dy|·k`
em torno de um eixo local fixo, logo a **direção da vista** tem de virar exatamente isso — nos 400
passos, que são mais de uma volta inteira. Um polo torna algum passo menor; nada mais o faz.

### §7.2 — Duas coisas que a troca de representação obrigou a escrever

- **Re-normalizar a cada giro.** Uma rotação livre é uma composição *acumulada* — um arrasto longo
  são centenas de multiplicações, e o erro de `f32` faz a norma derivar. Um quaternion não-unitário
  deixa de ser uma rotação: ele passa a **escalar** a peça, e o sintoma é a forma a encolher devagar
  enquanto se gira. Há gate (a área da peça depois de mil giros).
- **Um gate de equivalência com a câmera antiga.** `from_yaw_pitch` tem de reproduzir a base
  exatamente — a troca foi feita para remover os polos, **não** para mudar o enquadramento. Sem ele,
  um sinal trocado na conversão mudaria toda cena de smoke e toda imagem de referência de uma vez, e
  o sintoma seria *"as imagens da W0 já não batem"*, longe da causa.
