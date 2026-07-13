# Reshape — a escultura de traço (W5)

> O que é: remodelar um traço **já desenhado**, com pincéis de raio + força + queda.
> Oito deles. Clean-room do sculpt do Grease Pencil 5.2 — a matemática e as constantes
> saíram do fonte (`sculpt_*.cc`), não do olho.
>
> Crate: [`ph2d-flip-reshape`](../../crates/ph2d-flip-reshape/) (CPU pura, headless) ·
> fronteira: [`shells/desktop/src/flip_reshape.rs`](../../shells/desktop/src/flip_reshape.rs) ·
> UI: modo **Sculpt** no painel do Flip. Referência: [`02 §7`](02_referencia_algoritmos_blender_5.2.md).

---

## §1 — As três decisões que definem a sensação

Estas não se re-derivam. Mexer numa delas muda **todos** os oito pincéis de uma vez.

**1. A dose é por AMOSTRA de input, não por tempo.** Mover devagar aplica mais; parar
o cursor não aplica nada (exceto no Randomize, que continua vibrando de propósito).
É assim que o GP "sente", e é o que dá controle fino: a velocidade da mão *é* o
controle de intensidade. Um fork que gerasse amostras por **timer** trocaria isso por
"quanto tempo você segura", e a ferramenta inteira mudaria de caráter.

**2. A máscara define O QUE; o traço define QUANTO.** O conjunto de traços que o gesto
pode tocar é **congelado no pen-down**. Arrastar o pincel para fora não recruta um
traço novo no meio do gesto — senão o resultado dependeria do *caminho* do mouse, e
duas passadas "iguais" dariam coisas diferentes.

**3. Em 2D-ortográfico a projeção colapsa.** O GP esculpe em espaço de **tela** e
converte o delta de volta ao objeto (`compute_orig_delta`). A nossa câmera é uma
similaridade — escala uniforme, sem perspectiva —, então distância, direção e ângulo
são os mesmos nos dois espaços, e tudo roda em espaço **local** do objeto. **Uma**
constante escapa: a amplitude do Randomize, que no GP é literalmente "pixels de tela";
ela é convertida explicitamente (`px_to_local`).

---

## §2 — A infra: uma influência, oito pincéis

Tudo passa por um funil só (`brush_point_influence`, `paint_common.cc:98`):

```
influência = força · pressão · falloff_multiframe · queda(distância / raio)
```

A **queda** é o smoothstep em `p = 1 − d/r`: 1 no centro, 0 na borda, **derivada zero
nas duas pontas** — é o que faz a marca do pincel não ter degrau nem no meio nem na
borda. Polinomial (HR-5: zero transcendental, e bit-determinística entre plataformas).

O **falloff multiframe** já está na assinatura, valendo `1.0`. Não é enfeite: quando a
tira ganhar seleção de múltiplos quadros (T5.7), o mesmo gesto esculpirá N quadros com
atenuação por distância temporal — e retrofitar o parâmetro depois custaria tocar os
oito pincéis. Um gate afirma que ele MULTIPLICA (senão ele apodreceria em silêncio).

| Controle | De onde vem |
|---|---|
| **Size** (raio) | o mesmo slider do pincel e da borracha — o raio é metade dele |
| **Strength** (força) | o mesmo Strength da borracha (`opacity` da tool) |
| **Ctrl** | inverte — **só** nos pincéis que têm direção (Pinch/Twist/Thickness/Strength) |

*Por que não um raio e uma força próprios?* Seriam estado duplicado para as mesmas duas
grandezas, e trocar de modo obrigaria a re-ajustar tudo. (Nos pincéis sem direção, o
Ctrl é **inerte** — e isso é honesto: "alisar ao contrário" não significa nada.)

---

## §3 — Os oito (com a fonte de cada constante)

| Pincel | O que faz | A matemática | Fonte |
|---|---|---|---|
| **Smooth** | alisa o tremor | kernel binomial `[1,2,1]/4`, **2 iterações**; a influência é o **peso de mistura** | `sculpt_smooth.cc:124` (`iterations = 2`, hard-coded lá também) |
| **Push** | empurra na direção do movimento | `pos += delta_mouse · influência` | `sculpt_push.cc` |
| **Grab** | **agarra** e carrega | máscara + pesos **congelados no pen-down** (com `pressure = 1.0` fixo); depois só `pos += delta · peso` | `sculpt_grab.cc:188` |
| **Pinch** | aperta (Ctrl: infla) | `s = influência²/25`; `pos += (cursor − pos)·s` | `sculpt_pinch.cc` |
| **Twist** | torce (Ctrl: inverte) | rotação rígida de `±1° · influência` ao redor do cursor | `sculpt_twist.cc` |
| **Thickness** | engrossa (Ctrl: afina) | `largura += ±influência · passo`, **aditivo** | `sculpt_thickness.cc` |
| **Strength** | opacidade (Ctrl: apaga) | `opacidade += ±influência · 0.125`, clampada | `sculpt_strength.cc` |
| **Randomize** | bagunça | ruído **perpendicular** ao movimento, re-semeado **por amostra** | `sculpt_randomize.cc:81-96` |

### O que a referência revela e a intuição erraria

- **Grab ≠ Push.** O Grab *segura* o que estava sob o pincel no toque e leva junto,
  mesmo que o cursor se afaste; o Push *varre* — quando o cursor sai de perto, o ponto
  para. Mesmo gesto, resultado oposto: por isso os dois existem. (Um gate roda o MESMO
  caminho nos dois e exige que divirjam.)
- **Pinch é quadrático E dividido por 25** — no máximo 4% de aproximação por amostra.
  Deliberadamente lento e "cremoso": ele afina uma silhueta em vez de colapsá-la.
- **Thickness é ADITIVO, nunca proporcional.** Um passo proporcional nunca sairia do
  zero (largura 0 ficaria 0 para sempre) e exageraria a diferença entre grosso e fino
  em vez de nivelá-la.
- **Randomize é perpendicular, não radial** (ruído radial engrossaria a silhueta; o
  perpendicular *ondula* a linha) e é **re-semeado por amostra** — parado, o pincel faz
  um passeio browniano. Uma semente por *gesto* congelaria o efeito exatamente quando o
  artista quer bagunçar mais.
- **O Smooth alisa sobre TODOS os pontos** e mascara só a *saída*: o kernel de um ponto
  lê os vizinhos, e os vizinhos podem estar fora do alcance do pincel. Mascarar a
  *entrada* faria a borda do pincel **deformar** o traço em vez de alisá-lo.
- **As pontas ficam ancoradas** no Smooth: alisar um extremo o puxa para dentro, e o
  traço **encurta** em silêncio a cada passada.

### As duas traduções de unidade (as únicas)

1. **Thickness.** O GP soma `influência · 0.001` ao **raio** em unidades de mundo, onde
   o raio default é `0.01` — ou seja, **10% do raio default por amostra**. A nossa
   largura é o **diâmetro em px de tela** (pincel absoluto) e o default é 6 px, então o
   passo equivalente é **0.6 px por amostra**: a mesma sensação, na nossa unidade.
   *Copiar o `0.001` cru daria um pincel que não faz nada visível (0,001 px!).*
2. **Randomize.** A amplitude é `influência · ruído` **pixels de tela** (as posições do
   GP ali são de view) → convertida por `px_to_local`.

### O Twist sem `sin`/`cos` (HR-5)

O giro é de no máximo **1° por amostra**. Nessa faixa a série de Taylor truncada é
exata para `f32`: o 1º termo omitido do seno vale ~1,3e-11 (o épsilon relativo do `f32`
é 1,2e-7). E um polinômio é **bit-idêntico entre plataformas**, que é a razão de o HR-5
existir (replay-hash é contrato do projeto). Um gate compara a rotação por Taylor com a
`sin_cos` de verdade em toda a faixa — o teste *pode* usar transcendental: ele é o
oráculo, não o produto.

---

## §3.1 — A COR ANDA COM A LINHA (o que o Suzanne ensinou)

O smoke trouxe o Suzanne do Blender ao lado do nosso: *"o fill é atualizado em tempo real
como se line e fill fossem um só"*. Fui ao fonte, e o mecanismo é mais simples do que
"material pronto":

> **No Grease Pencil, o preenchimento é a TRIANGULAÇÃO DOS PONTOS DA PRÓPRIA CURVA**
> (`blenkernel/grease_pencil.cc:477`). Não existe geometria de fill separada: mover os
> pontos invalida o cache e o fill re-tria **no mesmo frame**.

E o sculpt de lá edita **todas** as curvas (`retrieve_editable_strokes`: só material
travado escapa) — inclusive as de preenchimento. Daí os dois comportamentos que faltavam:

**1. A escultura move as REGIÕES** (e os buracos delas). Um pincel que movesse a linha e
deixasse a cor para trás não seria escultura — seria uma ferramenta de quebrar o desenho.
Os buracos vivem fora do SoA (`FlipStroke.holes`), então nenhum laço sobre `positions_mut()`
os alcança: é exatamente o tipo de coisa que se esquece, e um "O" com o furo parado vira
uma mancha. Há gate para os dois.

*O que NÃO se esculpe numa região são os ATRIBUTOS:* o `width` do contorno de um fill é a
**dilatação da cor por baixo da linha** (BUGS #15), não a espessura de um traço —
engrossá-lo com o Thicken empurraria a cor para fora do desenho.

**2. O traço nasce PREENCHIDO** (`Shape: Line | Filled`, no modo Draw) — o material
`stroke + fill` do GP, que é como o Suzanne é desenhado. O traço carrega o próprio
preenchimento (o fill é a triangulação dos **pontos dele**), então **linha e cor são UMA
geometria**: esculpir a linha move a cor exatamente junto, no mesmo frame, sem nada a
re-preencher e sem nada para ficar para trás.

O balde continua existindo — e continua sendo a ferramenta certa para colorir uma região
delimitada por **vários** traços (o caso em que não há "uma curva" para carregar a cor).

---

## §4 — O que o Reshape herda do módulo (e não pode reinventar)

- **Autokey `Modify`** (a regra da W3, `05 §4`): esculpir é MODIFICAR o que está na
  tela. No rabo de um hold, a chave nova nasce **duplicata** — nunca em branco, senão o
  usuário esculpiria o nada enquanto o desenho que ele VÊ ficaria intacto num quadro
  anterior. Ponto único: `flip_autokey::target_drawing`.
- **Camada travada recusa** (e diz por quê: toast). Uma ferramenta que consome o clique
  e não faz nada parece quebrada.
- **Uma REGIÃO não se esculpe** (preenchimento / fechamento de gap): o contorno dela não
  é rasterizado, então mexer nele moveria uma borda invisível e destruiria os buracos.
  Mesmo critério da borracha de ponto (`flip_erase::is_region`).
- **O kernel binomial tem um dono só** (`ph2d_flip_reshape::blur`): o mesmo alisador
  serve o *active smoothing* do traço em curso (influência uniforme) e o pincel Smooth
  (influência por ponto). Duas cópias derivariam — e "o Smooth da caneta e o Smooth do
  pincel alisam diferente" é o tipo de bug que ninguém procura.

---

## §5 — Aberto (declarado, não esquecido)

- **T5.7 — multiframe**: o mesmo gesto esculpindo N quadros com falloff temporal. O
  parâmetro já existe e é respeitado; falta a **multi-seleção de chaves na tira** (a
  mesma dependência do fill multiframe — considere fazer os dois juntos).
- **Auto-masking fino** (por traço sob o cursor, por material, pela seleção): depende do
  modelo de **seleção** — é o Edit Mode, o pacote seguinte. Hoje a máscara é "os traços
  de linha do desenho ativo", congelada no down.
- **Clone**: é um **comando** (copiar/colar de traços), não um pincel — os modos
  contínuos do GPv2 são admitidamente quebrados e foram removidos lá.
- **Pressão real**: o mouse manda `1.0`. Quando a caneta chegar (curva de pressão
  editável), ela entra na influência sem tocar em pincel nenhum — o funil já a recebe.
