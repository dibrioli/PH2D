---
name: reference-topic-oracle-discipline
description: "Disciplina de oráculo (13) — o pixel/valor exato é o juiz; e um oráculo que só responde ao rato ainda se corre"
metadata:
  node_type: memory
  type: reference
---

- [[feedback_oracle_must_model_appearance_not_implementation]] — derivado do shader fica verde com o bug
- [[feedback_render_and_look_when_a_green_gate_is_contradicted]] — gate verde contradito? RENDERIZE
- [[feedback_heuristic_needs_false_positive_rate]] — verde num fixture é sorte do seed
- [[feedback_loose_oracle_hides_systematic_bias]] — asserte o valor EXATO na unidade do usuário
- [[feedback_a_magnitude_bound_misses_a_systematic_off_by_one]] — gate com tolerância conta TAMBÉM quantos diferem; erro de convenção é pequeno E onipresente
- [[feedback_a_mutation_that_does_not_bleed_may_indict_the_oracle_not_the_finding]] — mutação sem sangue pode acusar o oráculo; meça a propriedade que a mudança É
- [[feedback_a_request_to_draw_a_property_is_first_a_question_of_whether_the_product_has_it]] — desenhar X sobre quem não faz X é desenho que mente
- [[feedback_a_mutation_that_does_not_bleed_may_indict_the_oracle_not_the_finding]] — meça a propriedade que a mudança É antes de descartar o achado
- [[feedback_a_wrapping_coordinate_is_a_bad_oracle_measure_the_rate]] — ângulo mod 2π vira ruído acima de 1 volta; leia angvel, não rotation
- [[feedback_a_presence_only_oracle_is_blind_to_the_smear]] — oito gates de presença verdes sobre uma mancha; exija um pixel VAZIO fora da peça
- ⛔ **Uma ENTRADA defeituosa fabrica diferenças contra o oráculo** (Teste Cascadeur, 13/09): a animação que mandámos ao Cascadeur tinha o pé a afundar 10 cm entre chaves e o braço a dar uma volta no ombro; a comparação «achou» que a física dele *inclina o corpo 17°/24° na decolagem* e *decola 2 quadros antes* — com a entrada corrigida, **0°/0,2° e os mesmos quadros**. Quem derrubou os dois foi o smoke do dono, não a comparação. ⇒ *antes de ler uma diferença contra o oráculo, meça a sanidade da ENTRADA (pé no chão, junta dentro do limite) com a mesma régua que vai usar na saída* ([[project_teste_cascadeur_2d_bones_testbed]]).
- ⛔ **Um repositório de referência pode trazer DUAS variantes do mesmo algoritmo que discordam** (Pixel
  Lab W25, 13/09): o `rotsprite-webgl` (MIT) tem o shader WebGL, que lê cada pixel de saída no CENTRO, e a
  versão JS, que reduz o ×8 tomando o CANTO de cada bloco. A W6 transcreveu a JS; o RotSprite afim da W25
  lê no centro, e o comentário dizia *"no giro puro é o mesmo algoritmo"* — medido, **48 %** das células
  pintadas diferem (mais que nearest × RotSprite, 33,9 %). ⇒ *"transcrevi o algoritmo X" nomeia a
  VARIANTE (arquivo e fase da amostra), e "é o mesmo" se MEDE antes de ir para um comentário.* A fase
  escolhida ficou pinada por uma LEI (onde o EPX não mexe, RotSprite = nearest), com controle de que a
  outra fase a quebra (40 de 40).
- ⛔ **Doze amostras "variadas" aprovaram um compressor que o corpus do ORÁCULO reprovou** (Pixel Lab
  W26, 13/09): o DEFLATE voltava byte a byte no zlib e ficava a ≤ 1,06× do zlib-9 em vazio, corridas,
  aleatório, quadros de pixel art — e o corpus do APNG (Pillow + ffmpeg) deu **1,249×**, todo o
  buraco num caso só: ruído espalhado num 256×256, casamentos CURTOS a distâncias variadas, onde a busca
  rasa perde. ⇒ *amostras escolhidas por "variedade" cobrem o fácil; o corpus tem de conter o caso DURO
  do algoritmo* — e a cura saiu de uma CURVA medida (profundidade × tamanho × pior tempo), não de
  "copiar o nível 9". Irmã: a minha barra "nunca pior que guardar" reprovou 70 031 bytes que eram
  EXATAMENTE a saída do zlib — a barra se calibra contra o lado aprovado (o próprio zlib) antes do código.
- ⛔ **Fixture que o ORÁCULO recusa nunca é comparada — e a recusa pode vir de um byte que não desenha nada**
  (Pixel Lab W27, 13/09): a FreeType 2.14 recusa um `.bdf` INTEIRO (erro 3) por um "é" (UTF-8 ou Latin-1) ou
  um TAB no COMEÇO de uma linha `COMMENT` (no fim aceita; numa propriedade, aceita). A fonte de casos-limite
  do oráculo tinha acento no comentário. ⇒ *toda fixture leva a conferência "os DOIS leitores abrem" antes
  de qualquer comparação* (foi ela que pegou), e a divergência medida vira conferência declarada — se o
  oráculo mudar, a nota envelheceu. Irmãs da mesma wave: o Pillow usa a FreeType mas não expõe glifo por
  índice nem o mapa de BYTES de um `.fnt` ⇒ a porta sem interface é a BIBLIOTECA por ctypes, com um controle
  de layout das structs contra o Pillow (struct desalinhada não dá erro, dá números).
- ⭐⭐⭐ **Um oráculo que só responde ao RATO ainda se corre — mas um evento sintético do Qt NÃO é entrada**
  (Teste Cascadeur, 13/09; a rede neural de pose automática, seis rotas por script todas inertes).
  Um `QMouseEvent` montado à mão e mandado à janela **seleciona** (o «pressionar» chega) e **não arrasta**:
  o Qt Quick refaz o teste de acerto a cada evento sintético e o primeiro MOVIMENTO passa a pertencer a
  outro item. ⇒ a porta é a do PLATAFORMA — `qt_handleMouseEvent`, exportada pela `libQt6Gui` do próprio
  alvo e chamada por **ctypes**, com o `QWindow*` de `shiboken6.getCppPointer`; é o mesmo caminho do QtTest,
  que não carrega quando o alvo traz um Qt mais velho que o do sistema. ⚠️ **E o XTest do X11 também não
  serve** num Xwayland aninhado: o ponteiro é do compositor. ⛔⛔ **Cada peça em falta é MUDA** — janela não
  ativa, área de rato por cima a roubar o movimento, arrasto longo demais, gizmo de girar em vez de mover,
  seleção anterior que persiste e responde «há algo aqui» a um clique que não acertou nada. ⭐ **As duas
  réguas que destravaram tudo:** um ESPIÃO de eventos nos itens da tela (quem recebe o quê) e a própria
  IMAGEM da janela (`grabWindow` → numpy) para saber em que pixel está o controlador — *adivinhar o pixel
  pela câmera erra 10–25 px, e um controlador tem 5 de raio*. Detalhe e as seis condições:
  [[reference_cascadeur_oracle_door_measured]].
- ⛔ [[feedback_a_law_parity_corpus_does_not_measure_the_product_the_owner_uses]] — lei a 1e-8 e o pincel piorava a superfície: peça a 2.ª missão de PRODUTO (valores de fábrica, gesto real, cursor na superfície, ablação)
- [[feedback_an_attested_spec_is_refutable_by_the_corpus_and_the_blind_spot_is_the_corpus]] — a fórmula ATESTADA concorda com a certa em toda peça CENTRADA na origem; 4 fixturas deslocadas refutaram-na (43 → 49 de 61)
- [[feedback_a_parity_measured_upstream_of_a_conversion_says_nothing_about_it]] — 51/61 fixturas verdes sobre uma ponte que invertia a curva: a bancada corre a lei DIRECTAMENTE e nunca atravessa a conversão do produto

---

### ⭐⭐⭐ Uma relação que a API do oráculo não expõe pode ser DERIVADA — e aí é medição, com resíduo e com CONTROLO (2026-09-19)

Ao trazer um segundo esqueleto do Cascadeur, a pergunta era «quem é pai de quem». A introspeção da
API pública diz que **não há porta**: nenhum método de pai no `model_viewer`, nenhum dado «Parent»
numa junta. A saída barata seria adivinhar a cadeia pelos NOMES dos ossos — e um nome não diz de que
lado do corpo um osso se pendura.

O que a API expõe é, em cada junta, a **posição LOCAL** e a **matriz GLOBAL**. ⇒ *o pai é a única
junta cuja matriz global leva a posição local do filho à posição global dele.* Isso não é um
palpite: é uma equação com **resíduo**, e o resíduo imprime-se.

Três coisas que a tornam honesta, e nenhuma é opcional:
- **a convenção também se mede** — por LINHAS o resíduo é `1e-5 cm` e por colunas `100 cm`. Isso não
  é um empate, é um discriminador;
- **o segundo candidato** — se o segundo melhor também fechar, o pai é ambíguo e a derivação não vale
  (aqui: zero ambíguos, uma raiz só);
- ⭐ **o CONTROLO num caso conhecido** — a mesma derivação corrida sobre o boneco cuja cadeia já
  estava escrita à mão reproduziu **12 de 12** ligações, com pior resíduo `2,31e-5 cm`. *Sem esse
  controlo isto seria um palpite com cara de medição.*

⛔ **E a mesma porta não serve para tudo**: os pontos de controle dele não são filhos de juntas (36 de
43 não carregam posição local; dos 7 que carregam, a derivação dá disparate — um ponto da bacia
«daria» o dedo do pé, a 22,6 cm). Tentado, medido, **revertido com a razão escrita no extractor**.
Para eles ficou uma tabela derivada da ESTRUTURA do nosso rig (o osso que COBRE a junta, para os
pontos principais; o que PARTE dela, para os outros), conferida contra a tabela escrita à mão do
primeiro boneco: **38 de 43** batem, e das 5 que diferem **3 são ossos FIXOS** (movem-se igual) e 2
são uma junta que o primeiro boneco não tem.
