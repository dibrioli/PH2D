# 03 — O plano: oito waves, cada uma com o que se mede

⛔⛔⛔ **ESTE PLANO DEIXOU DE SER A FILA EM 2026-09-20 — a fila é a do [`15` §5](15_as_metas.md).**

Ele continua a ser **o registo das oito waves** (o que cada uma mediu, a régua de cada uma e o que
ficou de fora com motivo), e é isso que se vem aqui buscar. ⛔ **O que NÃO se vem aqui buscar é
«qual é o próximo passo»:** nesse dia o dono deu uma ordem nova (*«faça tudo que for necessário para
superar o objetivo»*) e a fila passou a ser as **cinco obras** do [`15` §5](15_as_metas.md), com o
mecanismo no [`14` §6](14_a_ordem_de_superar.md).

⚠️⚠️ **Este bloco existe porque o erro foi COMETIDO:** em 21/09 leu-se esta página para responder
*«qual a próxima etapa?»* e respondeu-se `W9`. *Uma página que descreve uma fila cumprida sem dizer
que ela foi substituída não tem como ser lida certo.*

**O que aqui ficou aberto, e onde ele vive hoje:**

| aberto nesta página | onde vive na fila em vigor |
|---|---|
| **`W9`** (medição + o gate VERMELHO que o dono mandou tratar aqui) | **3.ª obra** — *a luz sobrevive ao movimento* ([`14` §6 `F1`](14_a_ordem_de_superar.md)), que a absorve com a causa já identificada |
| **`W6`** (autoria: o grafo MaterialX no módulo de nós) | ⛔ **FORA da fila nova** — nenhuma das cinco obras a contém. Fica **NOMEADA e sem posição**, e pô-la numa é decisão do dono |
| **`W7d`** (profundidade de campo) | ⛔ **FORA, por DECISÃO DO DONO (2026-09-22)** — ver [`12` §W7d](12_o_acabamento.md) |

---

⚠️ **A ordem é por SALTO VISUAL POR UNIDADE DE TRABALHO**, e não por dificuldade nem por vontade.
Cada wave acaba num **smoke** (a lei da casa) e traz a régua que a julga.

⚠️ **ESTADO, 2026-09-19 — auditado contra o código, não contra a memória:** `W1`..`W5` estão
**FECHADAS** e vivem no modo *Render* do modelador (a marca de cada uma diz onde se lê o mecanismo),
e com elas fecharam **SEIS dos oito ingredientes** do [`01`](01_o_alvo_decomposto.md) — o `6` (a
translucidez) entrou em 17/09 fora desta numeração.

⭐⭐ **E a `W8` FECHOU em 19/09, por ordem do dono** (*«8 e depois do smoke o 7»*) ⇒ são **SETE dos
oito ingredientes**. Faltam a **`W6`** (autoria), a **`W7`** (o pós — a seguir, por ordem dele), mais
a **`W9`** (medição, que ele pôs ao fim da fila em 17/09). ✅ **A `W10`** (o gémeo do amaciamento no
dispositivo) foi adiada e **trazida de volta para a frente por ele no mesmo dia**, e FECHOU em 19/09.

⚠️ **São DUAS numerações e elas não coincidem:** as `W` são waves de TRABALHO, os `1`..`8` do `01`
são INGREDIENTES do alvo. A `W6` (autoria) e a `W9`/`W10` não são ingredientes de nenhum — *contar
umas pelas outras é como esta página já errou uma vez (ver o fim do ficheiro)*.

⛔ **A linha que aqui esteve — *«nada aqui está construído»* — era verdade em 2026-09-09 e falsa
desde 13/09.** *Um plano que se declara por começar sobre cinco waves shipadas manda reconstruir
trabalho já pago*, que é o defeito de que o `CLAUDE.md` §5 avisa sobre si mesmo.

## W1 — A gestão de cor (o fundamento barato) ✅ **FECHADA no modelador (13/09 + 14/09)**

Espaço de trabalho **linear** declarado, **exposição** autorada (em *stops*), e um tonemapper a
sério (**AgX** ou **ACES**), com a saída em sRGB.

- **Porquê primeiro:** é a diferença entre «parece de 2005» e «parece moderno», e **tudo o que vem
  depois mede-se errado sem ela** — um bloom sobre valores que não são HDR é um borrão.
- **Já temos:** `tonemap.wgsl` e `bloom.wgsl` no `ph2d-render`.
  ⛔ **Remedido em 13/09 ([`04`](04_a_remedicao_contra_a_arvore.md) §1):** o tonemap está em
  **bypass** e o `game_rt` é **partilhado com a arte 2D**; o modelador pinta **matcap** fora do HDR.
  ⇒ a `W1` sozinha não tem consumidor honesto, e a primeira fatia que se vê é o modo *Render* do
  modelador com `W1` + `W2`/`W3` mínimas juntas (`04` §4).
- **Falta:** o espaço de trabalho **declarado**, a exposição como número do artista, e a escolha do
  tonemapper como chip.
- **Régua:** uma rampa de luminância de `0` a `16` tem de sair monótona e sem clipping colorido; e o
  branco de `1,0` tem de cair no mesmo pixel antes e depois de mudar a exposição em `+1` e `−1`.
- **Smoke:** a mesma peça com exposição `−2`, `0`, `+2` lado a lado.
- ✅ **O que shipou:** a [`ph2d-view-transform`](../../crates/ph2d-view-transform/src/lib.rs) — a
  exposição em *stops* e a vista (`Standard` · `Neutral`, esta medida contra o OCIO do Blender nos
  nós do LUT a `8,9e-6`), com o olhar a ser da **cena** e o sombreamento de cada viewport a ser do
  viewport ([`05` §1..§8](05_o_modo_render_do_modelador.md)); o modelador abre em **`Neutral`** por
  decisão do dono, com a tabela do branco chapado ao lado ([`05` §15](05_o_modo_render_do_modelador.md)).
- ⛔ **O que NÃO entrou, com motivo:** o **AgX** — não por preço, por **LICENÇA** (não há neste disco
  um AgX cuja licença permissiva se leia no artefacto: [`04` §3](04_a_remedicao_contra_a_arvore.md)).

## W2 — A superfície OpenPBR ✅ **FECHADA no subconjunto declarado (13/09 + 14/09)**

Uma crate `ph2d-material` com os **41 números** do `open_pbr_surface`, e o shader **gerado** do
padrão (ver `00` §3).

- **A 1.ª medição da wave é a ponte para WGSL**, e ela tem três candidatos por medir: **Slang → SPIR-V**,
  **GLSL → `naga`**, ou um `GenWgsl` próprio. ⚠️ *Escolher sem medir os três é o erro que o §0 proíbe.*
  ✅ **MEDIDA em 13/09 ([`04`](04_a_remedicao_contra_a_arvore.md) §2):** `WgslShaderGenerator` →
  `naga` `glsl-in` valida (`5 598` linhas); a rota por SPIR-V reprova no fragmento; o `GenWgsl`
  próprio não é preciso. ⚠️ Validar não é sombrear igual — o `MaterialXView` continua a ser a régua.
- **Lobos mínimos para o alvo:** base difusa · especular GGX com **compensação de energia** ·
  metalness · **coat** · emissão. O `transmission` e o `fuzz` entram depois.
- ⭐ **O `subsurface` já existe** (`sss.rs` + tabela pré-integrada) e passa a ser **uma entrada do
  OpenPBR** em vez de um sistema ao lado.
- **Régua:** o *furnace test* — sob um ambiente branco uniforme, uma esfera de `base_color = 1` e
  qualquer rugosidade tem de devolver branco. *É o teste que apanha energia perdida, e não perdoa.*
- **Oráculo:** o `MaterialXView` renderiza o mesmo material; comparam-se as imagens **por passo**.
- ✅ **O que shipou:** a [`ph2d-material`](../../crates/ph2d-material/src/lib.rs) — o port
  **Apache-2.0** do GLSL de referência, com base · especular GGX · metal · **verniz** · **emissão**,
  o *furnace test* de pé, e **material por OBJECTO** com as 15 fileiras no painel
  ([`05` §11, §20, §21, §22](05_o_modo_render_do_modelador.md)).
- ✅ **E a SUBSUPERFÍCIE entrou em 17/09** ([`10`](10_a_luz_que_atravessa_a_peca.md)) — o
  ingrediente **`6`** do [`01`](01_o_alvo_decomposto.md), escolhido pelo dono. ⭐ E a frase que aqui
  estava — *«a tabela pré-integrada dele já existe no `ph2d-mesh-render`: a wave que o traz é uma
  ENTRADA do OpenPBR, não um sistema ao lado»* — estava **certa nas duas metades**, e a medição
  mostrou-a mais forte do que ela dizia: aquela tabela integra **o mesmo integral**, com a mesma
  corda e a mesma normalização.
- ⛔ **O que NÃO entrou, e cada um é uma closure com gate próprio a escrever:** `transmission_*` ·
  `fuzz_*` · `thin_film_*` · `geometry_opacity` · a anisotropia (o traçador não tem tangentes).

## W3 — O céu como FONTE de luz (IBL) ✅ **FECHADA (14/09 + 15/09)**

Ambiente pré-filtrado: irradiância difusa + especular por rugosidade + a BRDF integrada.

- **Porquê agora:** é o que põe cor no lado escuro sem o lavar, e é o que faz o metal existir.
- **Substitui** o `env_ambient` constante do `ph2d-light`, que é a razão de a peça de hoje flutuar.
- **Régua:** a mesma esfera contra o mesmo ambiente, comparada com o `MaterialXView`.
- ✅ **O que shipou:** o **estúdio** ([`studio.rs`](../../crates/ph2d-app-field3d/src/studio.rs),
  [`05` §24](05_o_modo_render_do_modelador.md)) — o céu deixa de ser uma rampa e ganha uma **fonte
  com forma** (uma gaussiana esférica no eixo da própria rampa, com a energia a SAIR do ambiente e
  não a somar-se a ele), pré-filtrada por tabela; mais a leitura do céu na **direcção média do
  lóbulo** ([§16](05_o_modo_render_do_modelador.md)) e as **lâmpadas como objectos da cena**, com
  gizmo próprio e tecto tirado da PLACA ([§25, §26, §40](05_o_modo_render_do_modelador.md)).
- ⛔ **O que NÃO entrou:** um **HDRI** de ficheiro (o céu é analítico, e é isso que dá a forma
  fechada do pré-filtro da rampa).

## W4 — Sombras que POUSAM o objecto ✅ **FECHADA (14/09 + 16/09)**

Cascatas para o sol + **endurecimento no contacto**.

- ⚠️ **SSAO não é isto**, e o `01` §4 explica porquê — nós já temos SSAO e o objecto continua a
  flutuar.
- **Régua:** um objecto a `0`, `1` e `10` cm do chão tem de dar três sombras diferentes.
- ✅ **A peça tapa-se a si própria** em 14/09 ([`05` §27](05_o_modo_render_do_modelador.md)) e o
  **chão INVISÍVEL** em 16/09 ([`07`](07_o_chao_que_so_recebe.md)), com a régua corrida — ela não
  tinha sujeito até lá, porque não havia chão. ⛔ **Fica NOMEADO o que não entrou:** luz de área, cone,
  e o chão a DEVOLVER luz (isso é a `W5`).

## W5 — ⭐⭐⭐ A luz indirecta, traçada contra o NOSSO campo ✅ **FECHADA (15/09 + 16/09 + 17/09)**

A wave que decide se a engine é bonita, e a que só nós podemos fazer assim (ver `02` §5.1).

- **Candidato principal:** **cascatas de radiância** com sondas esparsas — sem ruído, logo sem
  denoiser, que é o que preserva um look de cores chapadas.
- ⭐ **O traçado percorre o campo de distância verdadeiro** (`ph2d-field-eval`), não um proxy.
- ⛔ **O preço está medido e é o risco da wave:** o quadro de movimento do modelador custa `26,7 ms`
  contra um orçamento de `16,7`, e a marcha é `80 %` disso. *A wave começa por medir quanto de GI
  cabe, e o resultado pode ser «cozida e não em tempo real» — que é uma resposta legítima.*
- **Régua:** a caixa de Cornell. Ela tem resposta conhecida e não deixa mentir.
- ✅ **O que shipou, em três actos:** a lei são **SONDAS de irradiância** (`32³ × 256` direcções, SH
  de 9 coeficientes — [`08` §14](08_a_luz_indirecta.md)), depois do terceiro report do dono ter
  mostrado que a recolha por pixel com direcções fixas é uma soma de **projecções duras** da peça; o
  **dispositivo** ([`08` §12](08_a_luz_indirecta.md), paridade `100,000 %`); e o **chão a receber a
  COR da peça** ([`09`](09_a_cor_que_a_peca_devolve_ao_chao.md)), que é um campo 2D próprio — ⛔ *as
  sondas da peça não servem a um plano, e a medição está no §2 daquele doc.*
- ⛔ **O preço que a wave temia foi pago pela BANDEIRA que já existia** (`antialias`, «grosso a
  mexer, nítido ao assentar»): o quadro de MOVIMENTO fica byte-idêntico e quem paga é o assente.
  ⚠️ *É exactamente isso que a `W9` vai reabrir.*

## W6 — A AUTORIA: o grafo MaterialX no módulo de nós

O artista vê **um nó** com a foto que o dono mandou. Quem quiser mais, abre o grafo — e o grafo é
**MaterialX**, logo entra e sai do Blender, do Houdini e do Substance.

- ⭐ O módulo de nós desta casa já tem cartão com params, paleta com busca, e a lei do *nenhum knob
  morto*. **A ligação é a tabela de 807 nodedefs como DADO**, que é o padrão que já deu o único
  painel `42/42` limpo do repositório.
- **Régua:** um `.mtlx` do Blender abre aqui e devolve os mesmos pixels que o `MaterialXView`.

## W7 — O pós ⏳ **`a`+`b`+`c` FECHADAS (19/09)**, `d` fora por proposta → [`12`](12_o_acabamento.md)

Bloom sobre HDR verdadeiro · profundidade de campo · anti-aliasing temporal.

⭐ **O oráculo é o Godot 4.7.2 (MIT, lido no artefacto) corrido sem interface**, e a lei já está
colhida: o limiar é **duro** e a cadeia é um **mip chain** cujo raio dobra por nível
([`12` §3](12_o_acabamento.md)). ⛔⛔ **A primeira peça não é um efeito, é um BUFFER:** os dois
motores fecham o quadro **pixel a pixel** e nada guarda o HDR que o brilho precisa de ler.
⛔ **A profundidade de campo FICA DE FORA, por decisão do dono (2026-09-22).**

⭐⭐⭐ **O que fechou:** o brilho existe e corre no **dispositivo** ([`12` §11](12_o_acabamento.md)) e
a borda **deixou de ferver** ([`12` §12](12_o_acabamento.md)) — esta última sem uma lei nova: a
segunda passagem da silhueta já existia nos dois motores e estava desligada no quadro de movimento
por uma tabela de CPU medida a `640×360`, antes de o quadro inteiro ir para a placa. No dispositivo
ela custa `1,03×`–`1,09×`, e sem ela a silhueta que a mão arrasta não tem **um único** pixel de
cobertura parcial. ⇒ **a `W7d` (DOF) está FORA por decisão do dono** e a `W7` fecha.

## W8 — ⭐ A camada de ESTILO ✅ **FECHADA (19/09, ordem do dono: *«8 e depois do smoke o 7»*)**

Os botões para mentir de propósito, por cima de um pipeline honesto — o mecanismo inteiro está na
[`11`](11_a_camada_de_estilo.md).

- ⚠️ **Era a última de propósito.** Estilo sobre um pipeline sem `W1` e `W5` é o protótipo que o
  `01` §4 nomeia. ⭐ **E essa razão EXPIROU em 16/09:** a `W1` e a `W5` fecharam, logo a cerca que
  punha esta wave no fim já não a prendia — *quem move o número que tornava algo inalcançável tem de
  reconferir a nota* (`CLAUDE.md` §0.0).
- ✅ **O que shipou:** a crate-folha [`ph2d-style`](../../crates/ph2d-style/src/lib.rs) (zero
  dependências, o molde da `ph2d-view-transform`) com **quatro** botões — o contorno, a tinta por
  **curvatura**, a grade por **zona** e a saturação da **indirecta** —, o gémeo em WGSL, e **dez
  fileiras** no painel, só no modo *Render*. Cena **`=35`**.
- ⭐⭐⭐ **O SINAL da curvatura já existia e era deitado fora uma linha antes de alguém o poder ler**
  ([`11` §3](11_a_camada_de_estilo.md)): `H = ∇²f/2` distingue uma bossa de uma cova, e o consumidor
  que a estreou pede um comprimento. Hoje a subsuperfície toma o módulo do lado dela — imagem byte a
  byte a mesma — e a tinta lê o sinal, que é a diferença entre um contorno e uma sujidade.
- ⭐⭐⭐ **A lição do [§24 da `10`](10_a_luz_que_atravessa_a_peca.md) foi aplicada ANTES da primeira
  linha:** o estilo entra na **assinatura** (`Presentation`), a apresentação é montada **uma vez e
  antes** do ramo do dispositivo, e há **gate estrutural** que corre sem placa.
- ⛔⛔ **A paridade achou uma divergência PRÉ-EXISTENTE** que só um consumidor **linear** na
  curvatura revela — a tabela dose-resposta está na [`11` §5](11_a_camada_de_estilo.md), e ⛔ a barra
  **não** foi afrouxada para a engolir.
- ⛔ **O CONTORNO desenhado fica de fora**, com o mecanismo: ele lê os VIZINHOS no ecrã, logo é um
  passe e não uma multiplicação — pô-lo na crate da lei obrigaria-a a receber um G-buffer e ela
  deixaria de ser a lei que os dois motores partilham.

## W9 — ⏱️ A AVALIAÇÃO DE PERFORMANCE (ordem do dono, 2026-09-17: *«coloque na fila ao final»*)

Uma wave de MEDIÇÃO e de corte de preço, **ao fim da fila e não antes** — o dono pô-la lá no mesmo
report em que aprovou o smoke do chão colorido, e a ordem é o que decide a posição.

- **⛔ O primeiro acto é RE-MEDIR, e não optimizar.** Os números de relógio espalhados pelos docs
  `05`..`09` foram lidos em builds diferentes, com a marcha ora na CPU ora no dispositivo, e com a
  máquina em cargas diferentes — ⚠️ o `26,7 ms` do quadro de movimento que o `CLAUDE.md` §5 cita é
  **anterior** ao quadro inteiro ir para a placa ([`05` §36](05_o_modo_render_do_modelador.md)).
  *Uma tabela de preço montada a partir de leituras de builds diferentes mede a história, não o
  produto.*
- **A régua, e ela já tem forma:** `1920×1080`, **mínimo de N**, A/B **intercalado no MESMO
  processo**, com o `/proc/loadavg` impresso ao lado de cada leitura (⛔ nenhuma leitura acima de
  `load ~5` vale — `CLAUDE.md` §5.0). As duas populações são **separadas e não se somam**: o quadro
  de **MOVIMENTO** (orçamento `16,7 ms`, e onde a bandeira `assente` desliga quase tudo — ⚠️ ela
  chamava-se `antialias` e o anti-serrilhado **saiu** dela em 19/09, [`12` §12](12_o_acabamento.md)) e o
  quadro **ASSENTE** (onde mora o preço destas cinco waves).
- **O que já está NOMEADO com preço, e é a matéria-prima da wave:**

  | dívida | preço medido | onde | forma da cura |
  |---|---:|---|---|
  | assadura do campo do chão, por quadro assente | `+4,98 ms` (CPU) | [`09` §6](09_a_cor_que_a_peca_devolve_ao_chao.md) | cache por cena-e-luz (o campo **não** depende da câmera) · ou kernel no dispositivo (`~0,04 ms`) |
  | assadura das sondas, por quadro assente | `+0,52 ms` | [`08` §14.7](08_a_luz_indirecta.md) | a **mesma** cache — as sondas também não dependem da câmera (`~1 ms` na placa a `32³`) |
  | o campo do chão é de **UMA** peça | não medido | [`09` §9](09_a_cor_que_a_peca_devolve_ao_chao.md) | um campo por peça, somados |

  ⭐ **As duas primeiras linhas são a MESMA cura** — *o que não depende da câmera reassa-se uma vez
  por cena e luz, não uma vez por quadro* —, e é isso que faz orbitar uma peça deixar de pagar.

  ⛔⛔⛔ **E o primeiro acto da wave foi pago DUAS vezes, porque a primeira leitura estava errada
  nos dois sentidos** (2026-09-20, corrigida em 2026-09-21). Ela dizia: *«o campo do chão é
  byte-idêntico sob `8` azimutes e sob `4×` de zoom ⇒ a chave não leva a câmera de todo, e a cache
  é exacta»*. **As duas metades dessa frase são falsas**, e o que as derrubou foi pôr a medição
  dentro de um GATE em vez de a deixar numa impressora:

  | eixo | `|Δ|` máximo | em bytes | células iguais |
  |---|---:|---:|---:|
  | **orbitar** `90°`/`180°`/`270°` | `6e-9` | `0,00002` | `262`–`506` de `1024` |
  | **zoom, com o clamp a morder** (`half_extent ≥ 0,8`) | `0,000000000` | `0` | `1024` de `1024` |
  | **zoom, abaixo do clamp** (`half_extent 0,005`) | `2,8e-4` | **`0,910`** | `130` de `1024` |
  | CONTROLO: a luz do outro lado | `0,0164` | `54` | `14` de `1024` |

  ⭐⭐ **Orbitar NÃO é byte-idêntico** — três quartos das células mudam no último bit —, e a
  impressora não o via porque escrevia `|Δ|` com **seis casas**, e `6e-9` lê-se `0.000000`.
  ⭐⭐⭐ **E o zoom É chave: a varredura anterior media um CLAMP.** A única porta por onde a câmera
  entra na assadura é a [`Sharpness::for_frame`], que faz `hit = min(HIT_EPS, half_extent/(2·lado_px))`
  — e nas três leituras de 20/09 o `hit` esteve **preso em `2e-4`**. O clamp solta-se com
  `lado_px > 2500 × half_extent`, que a `0,2` de enquadramento são **`500` píxeis**: *um zoom
  apertado numa janela normal já está do outro lado.*

  ⇒ **a chave leva a TOLERÂNCIA DE ACERTO e não leva a ORIENTAÇÃO**, e o que separa os dois eixos
  são quatro ordens de grandeza (`2,8e-4` contra `6e-9`). A barra NOMEIA o recurso e é **o BYTE de
  saída** (`1/(255 × 12,92)` de radiância, o troço linear do sRGB junto de zero): a orientação está
  `50 000×` abaixo dela, a tolerância chega a `0,91` dela.

  ⚠️ **A cache continua a valer o que valia** — orbitar é o gesto que paga os `+4,98 ms` por quadro
  e ele **não** mexe no `hit` —, mas ela deixa de ser *«por cena-e-luz»* e passa a ser *«por
  cena-luz-e-tolerância»*.

  ⭐ **Os dois gates que afirmam isto**, em `ph2d-field-render/src/tests/chao_ricochete.rs`:
  `o_campo_do_chao_nao_muda_o_que_um_byte_ve_ao_orbitar` e
  `a_tolerancia_de_acerto_entra_na_chave_da_cache_do_chao` — ⛔ o segundo tem o **controlo primeiro**
  (com o clamp a morder o zoom não move um bit), senão alguém lê a metade de baixo e põe o
  `half_extent` na chave, invalidando a cache em todo arrasto de zoom.
  ⚠️ **Elas são medições de VALOR e não de relógio**, e é por isso que correram com a máquina
  ocupada: contenção não move bytes. *A coluna do relógio desta wave continua por tirar, e essa
  precisa da máquina calma.*

  ⏳ **E fica NOMEADO o que a construção da cache ainda tem de resolver:** a chave precisa de uma
  identidade EXACTA para o `Registry` (as esculturas amostradas **não** são inlinadas na fita — o
  prefixo `escultura_` dela é uma ligação, não texto), logo *duas esculturas diferentes podem dar a
  MESMA fita*. Uma cache que se contente com a fita entrega o campo da escultura anterior, e o
  defeito é mudo.
- ⚠️ **O tecto de cada número tem de NOMEAR O RECURSO** (`CLAUDE.md` §0.0): `GROUND_BOUNCE_SPAN`
  (`6` raios) e `GROUND_BOUNCE_FADE` (`0,25`) são os dois desta família ainda **sem tabela por
  baixo**, e quem lhes tocar mede-os como os outros dois foram medidos ([`09` §5](09_a_cor_que_a_peca_devolve_ao_chao.md)).
- ⛔ **E o tecto do módulo é o do HARDWARE, nunca o do caminho lento:** a assadura de CPU só existe
  porque comprava **uma lei em vez de duas** ([`09` §6](09_a_cor_que_a_peca_devolve_ao_chao.md)), e
  essa é uma decisão de ARQUITECTURA que a medição pode reabrir — mas só com a paridade de assadura
  gateada no dia em que houver duas.
- **Smoke:** a mesma cena, o mesmo gesto, com o número do quadro à vista antes e depois.

### ⭐⭐⭐⭐ O PRIMEIRO ACTO DA `W9` FOI PAGO, E ACHOU UM PREÇO QUE NENHUM DOC DESTE MÓDULO NOMEIA

**`1,4` a `4,4` segundos, e o artista paga-os cada vez que ACRESCENTA uma forma à peça.**

Medido 2026-09-21 numa janela de calma REAL (`97`–`100 %` de CPU ociosa, `--release`, três corridas
concordantes — ⚠️ o `loadavg` lia `2,6` a `6,3` e a régua é o `vmstat`, nunca ele). As sondas vivem
ao lado do gate, em `ph2d-app-field3d/src/preview_device_tests.rs`:

| gesto | ms |
|---|---:|
| a peça, **1.ª vez** | **`1 449`** |
| a mesma, outra vez | `4,45` |
| **arrastar um número** (raio `0,50 → 0,60 → 0,70`) | `4,24` · `4,28` |
| **ACRESCENTAR uma forma, 1.ª vez** | **`1 406`** |
| a mesma peça de duas, outra vez | `6,09` |
| voltar à peça anterior | `4,84` |

⭐ **Arrastar um número está CERTO e é barato** — o cache do pintor tem por chave o TEXTO do shader,
um slider muda o armazém `k` e o texto fica igual, exactamente como o `paint.rs` promete por escrito.
⛔⛔ **Acrescentar uma forma muda a ÁRVORE ⇒ muda a fita ⇒ muda o texto ⇒ compila tudo outra vez.**

⭐⭐⭐ **E o custo é da PLACA, não da CPU, com a prova a ser a segunda chamada:** a
[`gpu_frame::paint`] reconstrói o `DeviceField` e a fita **a cada chamada**, e a 2.ª custa `4`–`12 ms`
— *logo o segundo e meio é inteiramente a compilação do programa*.

⭐⭐⭐⭐ **E ele DECOMPÕE-SE, sobre as 22 cenas:**

```
  ms = 1 307 + 2,62 × instruções-da-fita      (resíduo p50 130 ms, pior 625)
```

⇒ **o termo constante — `1,31 s` — é o KERNEL DO PINTOR**, e a inclinação é a peça. *Isso reabre a
variante de shader com um valor completamente diferente do que eu lhe tinha dado:* ela não compra
`5 %` de um quadro, compra uma fracção de `1,31 s` pagos **em toda edição estrutural**.

⛔⛔ **E o VERMELHO é, em boa parte, a RÉGUA — o gate viola a régua que esta própria página
prescreve.** O §W9 acima escreve *«`1920×1080`, **mínimo de N**, A/B intercalado no MESMO
processo»*, e o `com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento` cronometra **UMA**
chamada por cena. Medido na mesma janela calma:

* a leitura de UMA chamada: **`8 de 22`**, estável nas três corridas;
* pelo **mínimo de 3**: `10` e `12 de 22` em duas corridas;
* e as cenas individuais movem-se até **`11,7×`** entre corridas do mesmo binário com a máquina a
  `100 %` ociosa.

⚠️⚠️ **Uma leitura anterior deste gate acusava a cena `=30` a `96`–`98 ms`; na janela calma ela lê
`17,9`–`18,2 ms`.** *Aquela leitura era da máquina.* ⇒ a atribuição do §10 do handoff da linha —
*«o que ficou caro é o DESENHO»* — não tem suporte, e a linha está ILIBADA por uma segunda via: o
diff dela no caminho do dispositivo são **4 linhas de WGSL** (`mx_at_base_color`) num kernel de
`687`, e a população do gate é **`22` nos dois lados** do merge-base.

### ⭐⭐⭐⭐ E A PRIMEIRA CURA FOI CONSTRUÍDA: A FITA DA PEÇA SAI DO SHADER DO PINTOR

**Acrescentar uma forma passa de `1 406 ms` para `74 ms` — `19×` — e a imagem é byte-idêntica.**

⭐⭐⭐ **A atribuição veio primeiro, e é ela que torna a cura possível** (instrumento novo e
permanente: `PH2D_PIPELINE_LOG=1`, no `FieldPipelines::entry_with_layout`):

| pipeline | linhas | a NOSSA tradução | o DRIVER |
|---|---:|---:|---:|
| `centro_e_luz` (marcha) | `344` | `0,74 ms` | `44,2 ms` |
| `bordas` (marcha) | `344` | `0,79 ms` | `24,7 ms` |
| `pinta` | `1 927` | `3,80 ms` | `347,3 ms` |
| **`pinta_bordas`** | `1 927` | `3,88 ms` | **`1 359,9 ms`** |

⇒ **`99,5 %` é o DRIVER** (a nossa tradução WGSL→SPIR-V são `9,2` de `1 776 ms`), e o `pinta_bordas`
sozinho é `77 %` do total — `3,9×` o `pinta` a partir do MESMO texto.

⛔⛔ **DUAS explicações do `3,9×` foram construídas, medidas e REFUTADAS:** o laço das quatro
sub-amostras a ser desenrolado (com **uma** sub-amostra ele ainda custa `1 213 ms`) e o laço em si
(**sem laço nenhum**, uma sub-amostra em linha recta, `1 088 ms`). *O que resta é a pressão do
próprio núcleo, e não é afinação.*

⭐⭐⭐⭐ **Mas a cura não precisava dessa resposta.** O grafo de chamadas do WGSL diz que das seis
entradas deste passe, o `pinta` e o `pinta_bordas` alcançam a fita da peça por **UM caminho só** — a
CURVATURA —, o `assa_sondas` pela marcha, e o `pinta_ricochete` e as duas metades da borda mole **não
a alcançam de todo**. ⇒ *no quadro de MOVIMENTO, com nada a ler a curvatura, o pintor não precisa da
peça no texto dele* — e o cache de pipelines, que tem por chave o TEXTO, passa a acertar.

⭐ **O predicado não é novo:** é o mesmo que o caminho de referência já usa para decidir se assa os
canais da curvatura (`Surface::reads_curvature` ∪ `Presentation::reads_curvature`). *Uma terceira
resposta à mesma pergunta seria a que envelhece.*

**Três gates, e o do meio é o que a cura precisa:**

* `a_fita_inerte_no_pintor_nao_muda_um_byte` — a mesma cena **com chão**, os dois caminhos, bytes
  iguais (⚠️ com o CONTROLO de que a peça está na imagem: *duas imagens vazias também são iguais*);
* `a_fita_inerte_faz_o_cache_acertar_na_peca_seguinte` — **a CONTA**, porque a economia é invisível
  a toda régua de valor: `SEM a cura cresceu 4 pipelines · COM a cura cresceu 2`;
* `quem_le_o_campo_continua_a_leva_lo_no_shader` — o jade (subsuperfície maciça) e o quadro assente,
  que é o que torna os outros dois load-bearing.

⚠️⚠️ **E o ARNÊS mentiu duas vezes, as duas da mesma família:** a 1.ª redacção do gate da CONTA
variava o **raio** entre os dois lados — e um raio é uma **constante da fita**, logo os dois lados
davam o mesmo texto e o segundo lia crescimento ZERO, passando com a cura apagada; e o
`Tracer::compiled()` é um **contador global**, logo sob `cargo test` (threads num processo) um irmão
entrava na conta. *A cura foi a fixtura e o `--test-threads=1` no arnês, nunca uma barra mais
frouxa.* Mutação **3 a sangrar + 1 NOMEADA** (trocar as constantes da fita inerte por `Vec::new()`
é hoje inobservável: aquela porta lê só o `.source`).

### ⛔⛔⛔ E A CURA DA FITA INERTE NÃO ALCANÇAVA A PEÇA DO ARTISTA — a lei do dono repunha o preço

⚠️⚠️ **A medição que decidiu a fita inerte correu com `owners: None`, e o produto põe lei do dono em
TODA peça com mais de uma folha** (`materials::Table::build`: `(placed.len() > 1).then(...)`). E ela
emite **uma FITA INTEIRA POR FOLHA** (`dono_folha_0`, `dono_folha_1`, … em
[`ph2d_field_eval::owners_wgsl`]) ⇒ *o texto do shader do pintor volta a levar a geometria da peça, N
vezes*. Medido pelo caminho do produto (sonda `diag_a_fita_inerte_com_a_lei_do_dono`):

| folhas | lei do dono | fita inerte | 1.ª pintura |
|---:|---|---|---:|
| 3 | não | **SIM** | **`7,85 ms`** |
| 3 | não | não | `2 122 ms` |
| 3 | **SIM** | **SIM** | `2 311 ms` |
| 3 | **SIM** | não | `2 316 ms` |

⇒ **com lei do dono a cura era INERTE.** *Eu tinha medido um caso que o produto só alcança numa peça
de UMA forma, e reportei `19×` ao dono sobre ele.*

⭐⭐⭐⭐ **A cura da cura é UMA LINHA, e ela não é uma optimização do caso raro — é o caso NORMAL de
quem modela:** a lei do dono passa a pedir materiais **DISTINTOS** e não apenas duas folhas. Uma
peça a ser construída tem o material de omissão em toda folha, e só ganha materiais distintos quando
o artista os autora.

⚠️ **E é byte-idêntica por CONSTRUÇÃO:** com todos os materiais iguais o `dono_mix` devolve `(a, b,
t)` cujos `ler_mat(a)` e `ler_mat(b)` dão o MESMO `Mat`, logo a lei calcula `ca + (ca − ca) · t`, que
é `ca` **exactamente** (o termo é `0,0 · t`). Dois gates: o
`n_folhas_com_o_mesmo_material_nao_pedem_lei_do_dono` afirma a ESCOLHA e o
`a_lei_do_dono_e_inerte_numa_peca_de_material_unico` afirma o **PIXEL** — *uma cura de `300×` sobre
uma conta que ninguém correu é onde um defeito mudo se instala.* Os dois levam o CONTROLO dos
materiais distintos, senão a cura lê-se como *«nunca há lei do dono»* e a peça inteira passa a usar
o material da primeira folha.

⭐⭐ **E o PISO de um gate pré-existente apanhou a mudança, com a mensagem que o doc dele prevê por
escrito** (*«um `Table::build` que deixasse de compilar leria `0` nos dois lados e o gate ficaria
verde a medir nada»*): dois gates liam `t.owners` sobre a fixtura de materiais iguais e a premissa
deles morreu. A cura foi mudá-los para uma fixtura de materiais **distintos** — *onde o sujeito
vive* —, nunca afrouxá-los. Mutação **3 de 3**, e a que nunca constrói a lei sangra em **três**
gates.

### ⛔⛔ E O «SEGUNDO PRÉMIO» DA FITA INERTE NÃO EXISTE — refutado pelo A/B, menos numa cena

Eu tinha escrito que a cura da fita inerte também parecia acelerar o quadro (cenas de fita grande a
caírem de `50` para `15 ms` entre duas leituras). ⛔ **Era contaminação da máquina**, e o A/B
INTERCALADO no mesmo processo — a régua que esta página prescreve — diz outra coisa:

| corrida | CPU ociosa | razão `sem/com` em `21` das `22` cenas | a cena `4` |
|---|---:|---:|---:|
| 1 | `95 %` | `0,88×` – `1,21×` | **`3,03×`** (`17,46` contra `52,93 ms`) |
| 2 | `92 %` | `0,90×` – `1,10×` | **`3,57×`** (`15,30` contra `54,64 ms`) |

⇒ **a cura é NEUTRA no relógio do quadro** (a fita que sobrava no pintor já estava a ser eliminada
pelo compilador do driver: ela custava COMPILAÇÃO e não custava quadro) — *e numa cena ela vale
`3×`, reproduzido em duas corridas independentes.*

⭐⭐⭐ **Essa cena é o penhasco de OCUPAÇÃO, finalmente medido.** A `4` tem `503` instruções e `43`
valores vivos; a `5` tem `934` e `62`, e lê `1,02×`; a `27` tem `751` e `69`, e lê `1,03×`. ⇒ *não é
o tamanho da fita — é um limiar de registos que só aquela cena atravessa*, e é por isso que o modelo
linear abaixo fica com `R² 0,80` e um resíduo grande. **Um penhasco não se ajusta com uma recta.**

⚠️⚠️ **E isto reatribui o modelo de custo:** se tirar a fita do PINTOR não move o relógio, o termo
`0,039 × instruções` mora na **MARCHA**, que é quem de facto avalia o campo. *O custo do quadro de
movimento é o campo ser caro POR AVALIAÇÃO, e não o raio dar muitos passos* — o que é coerente com
os passos explicarem `R² 0,165`.

### ⭐⭐⭐ O QUE O QUADRO DE MOVIMENTO CUSTA — o modelo, ajustado às 22 cenas na janela calma

O primeiro acto da `W9` é RE-MEDIR, e a tabela do gate já imprimia as colunas todas sem ninguém as
ter cruzado. Ajustadas ao mínimo de três corridas (`--release`, `99`–`100 %` de CPU ociosa):

| modelo | `R²` | pior erro |
|---|---:|---:|
| só as **instruções** da fita | `0,705` | `32,7 ms` |
| só os **valores vivos** (registos) | `0,689` | `34,2 ms` |
| só os **passos/acerto** | `0,165` | `50,3 ms` |
| **instruções + transcendentes + raízes** | **`0,803`** | **`18,7 ms`** |

```
  ms ≈ 5,0  +  0,039 × instruções  +  0,52 × transcendentes  +  0,03 × raízes
```

⛔⛔ **Os PASSOS DA MARCHA não explicam o custo** — sozinhos ficam em `R² 0,165`, e no ajuste
completo o coeficiente deles é **NEGATIVO**. *Uma cena com `410` passos por acerto (a `28`) e uma
com `43` (a `5`) custam o mesmo, e o que as separa é o tamanho da fita.* ⇒ atacar a marcha — que é
onde o `05` §36 e o `CLAUDE.md` §5 apontam — não é onde o dinheiro está.

⭐⭐ **Uma TRANSCENDENTE custa `13×` uma instrução comum** (`0,52` contra `0,039`), e é a coluna
mais alavancada do modelo: a cena `25`, com `180` instruções e **`40` transcendentes**, custa
`49,9 ms` — mais do que a `11`, com `393` instruções e **zero**.

⚠️ **O resíduo é grande e está nomeado** (`18,7 ms` no pior caso, `20 %` da variação): as quatro
cenas que o modelo mais erra estão na tabela do ajuste, e duas delas erram para lados opostos. *Este
modelo diz ONDE procurar, não o que uma cura vale.*

⏳ **A fila da `W9`, reordenada pelo preço medido:**

1. ✅ **`1,4 s` por edição estrutural** — **FECHADO** pela fita inerte (acima): `1 406 → 74 ms`.
   ⏳ O que sobra dela: a peça com **escultura** ou com **vários materiais** volta a pôr texto
   próprio no shader do pintor (o corpo da escultura e a lei do dono), e aí o preço regressa. E o
   `1,31 s` do kernel continua a ser pago **uma vez por sessão**, e o cache de pipelines **em
   DISCO** — a cura nomeada — **desceu na fila por VIABILIDADE medida** (sonda
   `sonda_a_placa_oferece_cache_de_pipelines`): a placa oferece-o (`NVIDIA RTX 5060 Ti`, Vulkan,
   chave `wgpu_pipeline_cache_vulkan_4318_11524`), **mas o `create_pipeline_cache` é `unsafe`** (o
   blob é entrada não confiável para o driver) e a workspace declara `unsafe_code = "forbid"` ⇒
   custa o molde do [ADR-0116](architecture/decisions/0116-audio-export-opus-isolated-unsafe-crate.md)
   (a crate desce a `deny` e só o módulo que toca a ABI o autoriza, com gate na lista). E o
   fornecedor diz que provavelmente não paga: *«most desktop GPU drivers will manage their own
   caches»*. ⚠️ **A evidência desta máquina é MISTA** — o `bordas` foi de `24,70` para `0,28 ms`
   entre corridas (o driver cacheou-o) e a 1.ª pintura leu `1 449` e `1 417 ms` em dois processos
   (o driver **não** ajudou o pipeline grande, que é onde a nossa ranhura poderia pagar). ⛔ A
   experiência de três vias que decidiria isto **não é escrevível nesta árvore**: precisa de
   `unsafe`, e um `forbid` não se contorna com um `allow`.
2. ⏳ **o quadro de movimento** — `5,0 ms` de base mais `0,039` por instrução e **`0,52` por
   transcendente** (o modelo acima). A alavanca é a FITA, não a marcha.
3. ✅ **`+4,98 ms` por quadro assente — o campo do chão está CACHEADO** (2026-09-22). A chave é a
   `ChaveDoChao` do `gpu_frame`: a **fita** da peça (texto + constantes), a altura do chão, as
   **lâmpadas**, os **materiais**, a **tolerância de acerto** e a grelha. ⛔ **Fora da chave: a
   ORIENTAÇÃO da câmera** — que é o gesto que paga a assadura, e é por isso que ela é a cura.
   Porta: `PH2D_FIELD_CHAO_CACHE=0`.

   ⛔⛔ **DUAS cercas conservadoras, as duas nomeadas:** uma peça com **escultura** não é cacheada
   (a fita não a inlina, logo duas esculturas dão a MESMA chave) e uma peça com **lei do dono**
   também não (a lei é função das FOLHAS e a chave só conhece a fita COMBINADA). ⭐ Desde a cura dos
   materiais a lei do dono só existe com materiais **distintos**, logo a segunda cerca quase nunca
   morde em quem está a modelar.

   **Quatro gates e quatro metades**, com a economia medida pela **CONTA** (`ACERTOS_DO_CHAO`)
   porque as duas rotas dão o MESMO campo: orbitar acerta · trocar a **luz** falta · um zoom
   **abaixo do clamp** da tolerância falta · uma peça com lei do dono não é cacheada. Mais o gate de
   **PIXEL** (`0 de 2 073 600` píxeis diferentes). Mutação **4 de 4**.

   ⚠️⚠️ **E a 1.ª redacção do gate de pixel era um VÁCUO**, com o furo escrito no comentário da
   minha própria sonda: o `tests_lampada` faz a luz **seguir a câmera**, logo orbitar trocava a LUZ,
   a chave faltava de qualquer maneira e as duas colunas comparavam **duas assaduras frescas**. ⇒ a
   lâmpada é **FIXA em mundo**, e a metade do ACERTO entrou no gate — *sem ela, uma cache apagada
   passa, porque duas assaduras frescas dão a mesma imagem por construção.*

   ⭐⭐⭐⭐ **E o RELÓGIO, tirado numa janela de calma REAL** (`98 %` de CPU ociosa, `--release`,
   A/B intercalado no mesmo processo, mínimo de `3` por lado, **a orbitar** — que é o gesto que a
   paga):

   | cena | com a cache | sem a cache | razão | poupa |
   |---:|---:|---:|---:|---:|
   | `1` | `22,69 ms` | `28,27 ms` | `1,25×` | `5,6 ms` |
   | `2` | `11,21 ms` | `16,35 ms` | `1,46×` | `5,1 ms` |
   | `30` | `37,78 ms` | `55,91 ms` | **`1,48×`** | **`18,1 ms`** |

   ⚠️ **A poupança é MAIOR do que os `+4,98 ms` nominais numa peça complexa** — aquele número saiu
   de uma peça simples, e a assadura lança raios contra o CAMPO, logo ela cresce com a peça. *Um
   preço medido numa fixtura não é o preço do produto.*

   ⛔ **As SONDAS NÃO seguem, e o veredito é o número delas:** `+0,52 ms` ([`08` §14.7](08_a_luz_indirecta.md))
   de um quadro assente que custa `11`–`56 ms` é **`1 %` a `5 %`**, e a cura ali **não é a mesma**:
   o campo do chão é dado de CPU e as sondas vivem num **buffer de GPU** criado por chamada — não
   re-despachar exige persistir buffers através de quadros, que é bem mais do que uma chave. *A
   mesma frase — «uma cache por cena e luz» — descreve duas obras de tamanhos muito diferentes.*
4. ✅ **a régua do gate** — **FECHADA**: `QUADROS_MEDIDOS = 3`, o mínimo, com a 1.ª chamada a ficar
   na tabela ao lado (ela é um preço real e uma régua que a apaga faz uma cura desaparecer com ela).

### ✅✅✅ O VERMELHO ESTÁ RESOLVIDO (2026-09-22) — `14 de 22`, quatro corridas, VERDE

O dono mandou fundir o vermelho nesta wave e tratá-lo aqui (*«2»*, 20/09). Está fechado:

| régua · máquina | leitura | veredito |
|---|---:|---|
| UMA chamada por cena · máquina contendida | `8`–`10 de 22` | ⛔ vermelho |
| **mínimo de 3** · `44 %` de CPU ociosa | `11 de 22` | ⛔ vermelho |
| **mínimo de 3** · **`99`–`100 %` ociosa** | **`14 de 22`**, quatro corridas seguidas | ✅ **VERDE** |

⭐⭐⭐ **E a atribuição, que é o que impede isto de ser lido como sorte:**

1. **A RÉGUA** é a maior parte. O gate cronometrava **uma** chamada por cena e chamava-lhe *«o
   quadro de movimento»* — e a 1.ª chamada de uma cena nova pagava `1,4`–`4,4 s` de compilação do
   driver mais o que o escalonador desse. *A régua que a própria página prescreve — «mínimo de N» —
   não estava implementada.*
2. **A MÁQUINA** vale três cenas (`11` a `44 %` ociosa contra `14` a `99 %`). ⚠️ Isto mantém o gate
   na família das flakes de carga do `CLAUDE.md` §5.0, e a tabela acima é o que impede alguém de
   ler um `11` como regressão.
3. **E UMA CENA é da cura**: a `4` lia `51,26 ms` (`D=2`) e lê **`14,74`** (`D=1`) — a fita inerte
   tirou-a do penhasco de ocupação, medido `3,03×` e `3,57×` em duas corridas independentes.

### ⛔ E AS `8` CENAS QUE SOBRAM: a minha hipótese da VITRINA está REFUTADA

Com o gate verde, a leitura óbvia das acusadas era *«são cenas de smoke que põem quatro a seis
variantes lado a lado, e quem modela trabalha UMA peça»*. ⭐ **Ela vale para a pior de todas** — a
`28` são quatro nós de toro, e medido cada um **sozinho** cabe no orçamento:

| peça | relógio | instruções | passos/acerto |
|---|---:|---:|---:|
| o nó `(2,3)` sozinho | `7,67 ms` | `128` | `552,9` |
| o nó `(3,2)` sozinho | `8,03 ms` | `183` | `396,9` |
| o nó `(2,5)` sozinho | `12,31 ms` | `128` | `786,0` |
| o nó `(5,2)` sozinho | `10,53 ms` | `293` | `257,7` |
| **a cena `28` inteira** | **`64,17 ms`** | `724` | `410,2` |

⭐⭐ **E os passos vão ao CONTRÁRIO:** um nó sozinho dá `786` passos por acerto e custa `12 ms`; os
quatro juntos dão `410` e custam `64`. *É a terceira medição independente a dizer que o custo é a
FITA e não a marcha* — e esta é a mais limpa, porque as duas grandezas se movem em sentidos opostos.

⛔⛔ **Mas a hipótese NÃO generaliza, e o censo derruba-a:** as `8` acusadas têm em média **`4,0`
folhas** e as `14` nítidas **`3,6`** — a contagem de peças não as separa. E duas acusadas são de
**UMA peça só**:

* a **`5` — o TORNO** (`934` instruções, `31,96 ms`): um contorno DESENHADO revolvido, e a
  resolução dele já está no **mínimo** (`DEFAULT_PROFILE_RESOLUTION = 1`) — as `934` instruções são
  os arcos do desenho, **desenrolados** numa cadeia de `min`. *Este é o caso de produto a sério: o
  artista desenha um perfil e roda-o.*
* a **`6` — A PONTE** (`39` instruções, `16,95 ms`): uma **escultura** virada campo. As `39`
  instruções escondem uma consulta a uma grelha (oito amostras por avaliação), e ela dá `143,7`
  passos por acerto.

⭐⭐⭐ **A cura do TORNO já existe — para a CPU:** a [`ph2d_field_eval::profile_index::ProfileIndex`]
troca a cadeia desenrolada por uma **consulta**, e o doc dela diz que ela existe *«por uma
medição»*. ⇒ *a obra que sobra é levá-la ao DISPOSITIVO*, o que muda a geração da fita (um buffer em
vez de texto) e pede paridade nos dois motores. Pelo modelo de custo (`0,039 ms` por instrução), tirar
`~880` instruções vale **mais do que os `31,96 ms` da cena inteira** — *é a única obra desta fila cujo
tecto de ganho é maior do que o custo que ela ataca.*

⏳ **O que fica ABERTO, e é honesto dizê-lo com o gate verde:** as `8` cenas que sobram estão
quase todas entre `17` e `31 ms` contra o orçamento de `16,7` — perto —, e **uma** está longe: a
`28`, a `63,69 ms`, com `724` instruções, `28` transcendentes, `44` raízes e `95` valores vivos. *Ela
é a única cena do corpus que nenhuma destas waves aproximou.*

### ⛔⛔⛔ A `W9` COMEÇOU COM UM VERMELHO JÁ MEDIDO — e ele era a primeira coisa a resolver

**Ordem do dono, 2026-09-20:** a `line/3DModeling` fechou com o
`preview::device_tests::com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento` **VERMELHO**, e
ele mandou **fundir com a dívida nomeada e tratá-la aqui** (*«2»*).

| | `main` | a linha |
|---|---|---|
| cenas nítidas em movimento | `15` de `18` | **`10` de `22`** |
| cena `30` | `13,45 ms` (divisor `1`) | **`96`–`98 ms`** (divisor `3`) |

⭐ **A cena `30` é o achado e a assinatura é a INVARIÂNCIA À CARGA** (`96`–`98 ms` a `68 %`, `2 %` e
`14 %` de CPU ociosa) — custo real, não contenção. A peça é a mesma dos dois lados ao pormenor
(`308 instr` · `190,9 passos/acerto`) ⇒ *o que ficou caro é o DESENHO*.

⛔ **CINCO suspeitos já estão ELIMINADOS** (as sondas, a curvatura, a subsuperfície, a borda mole e
o brilho — todos atrás de guardas que os valores de fábrica não abrem), e a hipótese que fica tem
uma **contradição à vista**: se fosse o tamanho do shader a baixar a ocupação, a cena `28` (com o
DOBRO dos passos por acerto) devia sofrer mais — e ela **melhorou**.

⚠️⚠️ **E o gate reprova por DUAS contas somadas:** a cena `30` **e** o denominador ter crescido de
`18` para `22` com as cenas novas das waves. *Separe-as antes de perseguir a primeira.*

⇒ Tudo, com o método de medição que evita os dois erros que esta medição já pagou:
[`HANDOFF …_A_LINHA_2026-09-20` §10](../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_A_LINHA_2026-09-20.md).

## W10 — ✅ O GÉMEO DO AMACIAMENTO NO DISPOSITIVO — **FECHADA em 2026-09-19**

> Ele adiou-a de manhã (*«coloque a possibilidade de melhoramento na fila mais no fim»*) e **trouxe-a
> para a frente à tarde**: *«A sombra mole no modo normal»*. Mecanismo, tabelas e as duas fronteiras
> abertas: [`10` §25](10_a_luz_que_atravessa_a_peca.md).

A sombra de borda mole que uma closure translúcida lê ([`10` §12](10_a_luz_que_atravessa_a_peca.md))
e o raio por material ([`10` §23](10_a_luz_que_atravessa_a_peca.md)) existem, estão gateados ao bit
— e vivem **no traçado de CPU**. O caminho de OMISSÃO do produto **devolve antes de os chamar**
([`10` §24](10_a_luz_que_atravessa_a_peca.md)), logo o artista vê a sombra da chapa com a borda
**DURA** sobre o jade.

- ⛔ **Ela vinha DEPOIS da `W9` e o dono inverteu a ordem no mesmo dia.** ⚠️ A consequência mantém-se
  e é da `W9`: ela mede **o que ship**, e o que ship mudou hoje — *o relógio desta wave NÃO foi
  medido*, e a tabela de preço da `W9` tem de a incluir.
- ✅ **As duas fronteiras foram ABERTAS** (o bloqueador tinha endereço, e eram estas): o buffer de luz do traçador tem passo
  `1 + n_lâmpadas + 6` e **não tem slot RGB por lâmpada**; e o `mx_direct` do WGSL recebe **UMA**
  radiância onde o [`Surface::direct_sss`](../../crates/ph2d-material/src/lib.rs) da CPU recebe
  **duas**. *Nenhuma das duas é afinação: são fronteiras.*
- **As duas rotas, com o preço de cada uma:**

  | rota | o que custa | o que arrisca |
  |---|---|---|
  | escrever o gémeo em **WGSL** (um canal RGB por lâmpada + duas passagens de borrão separável + a leitura) | o desenho que o ricochete do [`08` §12](08_a_luz_indirecta.md) já pagou naquele passe | a fronteira do `ph2d-material` no dispositivo |
  | voltar pelo **`march`** e sombrear na CPU | traz o G-buffer pelo barramento — o que a `traca` evita **de propósito** | nenhuma lei nova, e perde a pintura no dispositivo |

- **Régua, e ela já existe:** a quebra na banda do terminador medida **na placa**
  ([`subsuperficie_dispositivo_tests.rs`](../../crates/ph2d-app-field3d/src/subsuperficie_dispositivo_tests.rs)) —
  `9,20` com a chapa **antes**, contra `1,00` da referência. ⭐ **A wave fechava quando as duas
  colunas lessem o mesmo**, e o gate estrutural
  [`a_borda_mole_e_inalcancavel_quando_o_dispositivo_pinta`] **tinha de reprovar nesse dia**, que era
  o desenho dele. ✅ **As duas colunas lêem `1,00` e `1,00`**, e aquele gate foi substituído por
  [`o_gemeo_da_borda_mole_esta_ligado_no_dispositivo`] — *a premissa dele morreu à vista, no diff*.
- ⛔ **Fica declarado o que ela NÃO faz:** com **dois ou mais** raios de espalhamento distintos na
  cena o dispositivo **recusa o quadro** e a CPU pinta — o buffer tem um raio para a cena inteira.
  *Nenhuma imagem errada; e o caminho lento não define o produto, só computa a mesma resposta.*
- ⚠️ **Dois buracos de RÉGUA que esta wave tem de fechar primeiro**, os dois medidos em 19/09: a
  paridade de materiais **nunca testou subsuperfície** (os seis materiais partem de
  `OpenPbr::default()`, que tem `subsurface_weight = 0`), e a única paridade com subsuperfície usa
  **uma esfera sozinha** cujo lado de CPU não assa o canal mole. *Construir o gémeo contra réguas
  que não o vêem repetiria, um nível acima, o defeito que esta wave existe para curar.*
- **Smoke:** a `=33` com o jade, **sem** `PH2D_FIELD_GPU=0`, e a borda mole a aparecer onde hoje
  está a linha dura.

---

## A ordem, num parágrafo

⛔⛔ **CORRECÇÃO de 2026-09-21: este parágrafo descreve a fila do ESTUDO, que já não é a fila** (ver
o topo do ficheiro). Ele fica porque explica **porque** as oito waves foram feitas nesta ordem — e
⚠️ **duas frases dele estavam mortas quando foram lidas**: *«a `W7` e a `W8` são o acabamento»* (as
duas fecharam em 19/09) e *«a `W9` fecha a fila»* (ela foi absorvida pela 3.ª obra da fila nova).
*Um parágrafo de ordem envelhece mais depressa do que a página que o contém.*


**`W1` e `W2` são o fundamento e são baratas.** `W3` e `W4` fazem o objecto existir no espaço. `W5`
é a wave grande e é a nossa vantagem estrutural. `W6` é o que responde ao *«intuitivo para
artistas»*. `W7` e `W8` são o acabamento — e o `W8` é o que faz a engine parecer-se com o alvo do
dono em vez de parecer-se com toda a gente. A **`W9`** (medição) fecha a fila. ⚠️ **A `W10` (o
gémeo do amaciamento no dispositivo) estava atrás dela e o dono inverteu a ordem em 19/09** — ela
FECHOU nesse dia, e a consequência é da `W9`: *o relógio dela não foi medido e entra na tabela.*

⚠️ **E as cinco primeiras estão FECHADAS** (topo deste ficheiro): dos oito ingredientes do
[`01`](01_o_alvo_decomposto.md), o que falta são **DOIS** — o `7` (o pós) e o `8` (o estilo) —,
mais a autoria da `W6`.

⛔ **CORRECÇÃO de 2026-09-19:** esta linha dizia **três** e contava o `6` (a translucidez) entre os
que faltam. Ele **fechou em 17/09** ([`10`](10_a_luz_que_atravessa_a_peca.md)), no mesmo dia em que
a `W5` fechou o ingrediente `2` — *o parágrafo envelheceu no dia em que foi escrito*, e é
exactamente o defeito de que o `CLAUDE.md` §5 avisa sobre si mesmo: **audite a lista contra o código
antes de pegar um item dela.** A translucidez tinha dívida (a `W10`), que FECHOU em 19/09 — ⚠️ e
ela **nunca** foi trabalho por começar.

## ⛔ O que este plano NÃO faz, e porquê

| ausência | razão |
|---|---|
| Nanite / geometria virtualizada | resolve um problema que não temos (`02` §3) |
| ReSTIR / hardware de raios | a Unreal chegou primeiro e melhor; e produz ruído, que é o inimigo deste look |
| um modelo de material próprio | o padrão está no disco e gera o código (`00`) |
| cel-shading como ponto de partida | é a camada `8`, não a `1` |
