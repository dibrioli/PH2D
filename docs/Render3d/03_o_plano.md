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

   ⛔⛔ **E a auditoria de 2026-09-23 apanhou o `3` a chamar-se «o joelho MEDIDO» SEM TABELA.**
   Medida, ela diz três coisas e nenhuma é essa: **`N = 1` está catastroficamente errado** (`2` de
   `22`, pior cena a `1,6 s` — ele mede a compilação); **o joelho é `2`**; e ⚠️ **o `N` compensa
   CARGA** — a `66 %` de ociosidade o `3` lê `12` e só o `4` chega a `14`. ⇒ *o `3` é uma amostra de
   margem, e a margem não o torna imune à máquina.* ⚠️ E o veredito oscila **±1 cena** entre duas
   corridas calmas: há uma cena exactamente na fronteira dos `16,7 ms`, logo *uma leitura a uma cena
   da barra não distingue produto de ruído* — o que reenquadra o `14 de 22` como `13`–`14`.

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
vez de texto) e pede paridade nos dois motores.

### ⛔⛔ E O TECTO DESTA OBRA FOI MEDIDO em 2026-09-23 — a frase acima estava OPTIMISTA

A redacção anterior dizia, pelo modelo de custo (`0,039 ms` por instrução), que tirar `~880`
instruções vale *«mais do que os `31,96 ms` da cena inteira»*. ⚠️ **Um modelo ajustado sobre `22`
cenas prevê a MÉDIA delas**, e um ganho maior do que o total é aritmeticamente impossível: *o tecto
de um ganho mede-se, e a forma de o medir é fazer o custo desaparecer*.

⭐ **A sonda é a `diag_o_tecto_da_wave_do_torno`** (`ph2d-app-field3d`): a MESMA silhueta do vaso
com o contorno reamostrado de `k` em `k` — a marcha vê a mesma peça e só a contagem de primitivas
se move. `1920×1080`, mínimo de `3`, `--release`, `98 %` de CPU ociosa:

| peça | primitivas | linhas de WGSL | vivos | quadro |
|---|---:|---:|---:|---:|
| **cilindro analítico (o PISO)** | — | `18` | `5` | **`4,32 ms`** |
| vaso de 64 em 64 | `3` | `87` | `17` | `5,51` |
| vaso de 16 em 16 | `7` | `198` | `29` | `8,56` |
| vaso de 4 em 4 | `25` | `684` | `31` | `23,01` |
| vaso de 2 em 2 | `50` | `1 342` | `31` | `43,34` |
| vaso de 1 em 1 | `99` | `2 639` | `33` | `116,31` |

⭐⭐ **A curva é LINEAR em `0,028`–`0,031 ms` por linha ao longo de duas ordens de grandeza**, e
extrapolada às `931` linhas da cena `5` dá `31,8 ms` contra os `31,96` medidos — *o modelo desta
cena é este, e a inclinação dela é `0,030` e não `0,039`*. ⛔ Acima de `1 342` linhas ela vira
super-linear (`0,056`), que é onde a ocupação começa a morder (`vivos` `31 → 33`).

⭐⭐⭐ **E a leitura que decide o DESENHO da wave:** ser linear na contagem ao longo de `18` → `1 342`
é a assinatura de **trabalho dinâmico por amostra**; se o custo fosse o TAMANHO DO TEXTO a curva
seria um degrau. ⇒ *uma consulta que troque texto por laço não compra nada — o que compra é a PODA*,
e a poda mede-se contando.

### ⭐⭐⭐ Quanto a PODA deixa — e a população certa é `(u, v)`, não a região de MUNDO

A `diag_quantas_linhas_a_poda_deixa` conta as linhas da árvore **ESPECIALIZADA** que o produto já
sabe construir ([`ph2d_field_eval::RegionCompiler`]), pela mesma porta de fita que o dispositivo
usa. ⭐ **Zero constantes inventadas** — é o número que uma consulta de facto executaria naquela
célula.

⛔ **Medido primeiro em regiões de MUNDO, o pior caso ganha só `2,5×`**, e a causa é geométrica: o
`u` do torno é `√(x² + z²)`, logo **toda** região que toque o eixo vê a largura INTEIRA do perfil.
*Uma consulta POR AMOSTRA não tem esse problema: ela conhece o `u` do ponto.* ⇒ a população é a
célula em `(u, v)`, e a caixa de mundo que a produz é **degenerada em `z`**.

O perfil da cena `5` tem `24` primitivas (`12` arcos) e paga **`931`** linhas em toda amostra:

| grelha em `(u, v)` | p50 | p90 | pior | ganho p50 | ganho pior |
|---|---:|---:|---:|---:|---:|
| `4×4` | `254` | `363` | `468` | `3,7×` | `2,0×` |
| `8×8` | `131` | `217` | `376` | `7,1×` | `2,5×` |
| **`16×16`** | **`79`** | `139` | **`270`** | **`11,8×`** | **`3,4×`** |
| `32×32` | `56` | `103` | `243` | `16,6×` | `3,8×` |
| `64×64` | `55` | `86` | `194` | `16,9×` | `4,8×` |

⇒ **o tecto honesto da wave**: com a inclinação medida (`0,030`) e o piso (`4,32 ms`), a cena `5`
aterra entre **`6,7 ms`** (a mediana) e **`12,4 ms`** (a pior célula) contra os `31,96` de hoje e um
orçamento de `16,7`. ⚠️ **E as linhas de uma consulta são mais caras que as de uma árvore
especializada** — nesta as arestas são constantes dobradas no texto, naquela são leituras de um
buffer —, logo o número real fica acima da tabela. *A wave continua a valer, e vale `2,6×`–`4,8×`, não
o tempo inteiro da cena.*

⚠️ **A população é uniforme na caixa e isso é declarado:** a marcha dá passos grandes longe da
superfície, logo a tabela **sobre-representa** as células baratas. *A coluna do PIOR CASO é a que não
mente, e é ela que tem de caber no orçamento.*

### ⛔⛔⛔ E o §0.0 da wave achou um DEFEITO no caminho que ela ia usar

A `diag_que_celulas_degeneram` explica por que o «pior caso» das grelhas finas lia `931` — a fita
**inteira**. O [`ph2d_field_eval::profile::sd_profile_in_region`] tirava as arestas que assentam no
EIXO com um `continue` **depois** do corte, e tinha escrita ao lado a nota de que um corte vazio é
*«impossível — a regra do corte guarda sempre pelo menos a aresta que realiza o `dmax`»*.

⚠️⚠️ **A nota fala do CORTE e havia DOIS filtros.** O corte podia devolver exactamente a costura, o
`continue` tirava-a, e a região caía no degenerado, que reconstrói a árvore INTEIRA. Medido no vaso
(`1` das `24` arestas assenta no eixo), grelha `32×32`:

| | antes | depois |
|---|---:|---:|
| células que pagam a fita inteira | **`5` de `1 024`** | **`0`** |
| pior caso da grelha | `931` linhas | **`243`** (`3,8×`) |
| p50 | `56` | `56` |

⭐ **E as cinco eram as células SOBRE O EIXO à altura da costura** — dentro do sólido, por onde a
marcha passa —, a pagar **`18×`**. ⇒ *a pergunta «esta aresta assenta no eixo?» passa a ter uma
PORTA* ([`ProfileIndex::no_eixo`]) e o corte corre sobre a população que sobra
([`ProfileIndex::distance_edges_fora_do_eixo`]), o que devolve à nota do degenerado a verdade que
ela afirmava.

⚠️ **O preço da cura está medido e é pequeno:** com a costura fora da população, o `dmax` sai de um
conjunto menor e pode ser maior, logo sobrevivem mais arestas onde ela era o realizador — p50 `129 →
131` a `8×8` (`+1,6 %`), e `0` nas outras quatro grelhas.

⛔ **E ela NÃO muda o quadro que o artista vê hoje:** o caminho do DISPOSITIVO não usa o
`RegionCompiler` (a fita dele é da peça inteira). Ela cura o traçado de **CPU** — que é o motor de
referência das paridades e o recurso de quem não tem placa — e a **extracção**, e prepara o terreno
da consulta no dispositivo. *Dizer que ela acelera o quadro seria vender o que não foi medido.*

Gates: `profile_index::eixo_tests::uma_regiao_sobre_a_costura_nao_paga_a_arvore_inteira` (a LEI,
sobre a fixtura mínima — um contorno cujo fecho assenta no eixo — com o CONTROLO de que a fixtura
contém o fenómeno) e `nenhuma_regiao_do_vaso_paga_a_arvore_inteira` (a mesma coisa sobre **o desenho
do dono**, com piso de população). **8 mutações, 8 sangram** — ⚠️ e uma delas mostrou que a 1.ª
redacção do segundo gate tinha **duas metades e a de cima era IMPLICADA pela de baixo**: um
degenerado paga a fita inteira, logo `pior × 2 > inteira` já reprova. *Uma linha que a mutação não
consegue matar não é lei, é comentário com sintaxe de código.*

### ⛔⛔⛔⛔ E O DESENHO DA WAVE FOI REFUTADO POR MEDIÇÃO: na GPU a poda COBRA divergência

A frase *«a cura do TORNO já existe — para a CPU; a obra que sobra é levá-la ao DISPOSITIVO»*
supõe que uma poda por ramo-e-limite transfere. ⚠️ **Numa CPU uma poda é grátis** (um `if` que salta
trabalho); **numa GPU ela cobra DIVERGÊNCIA**, porque threads vizinhas caem em células vizinhas e
lêem listas **diferentes** — e o hardware serializa isso.

⭐ **O instrumento é a `diag_o_custo_de_uma_aresta_lida_contra_dobrada`**: a MESMA aritmética
(distância² ponto-segmento, com `e` e `e/‖e‖²` pré-calculados) em seis formas, sobre os mesmos
`1 048 576` pontos, com a **compilação fora do relógio** (porta nova
[`ph2d_field_gpu::probe::evaluate_medido`], que existe por causa desta medição: a irmã compila
dentro da chamada, e cronometrá-la mede o driver). A coluna lida é a **inclinação em `N`**, que
cancela o despacho, a leitura de volta e a compilação. ⭐ E a 1.ª asserção é que **as seis dão o
mesmo número** — sem ela a tabela cronometraria leis diferentes.

| forma | ns por aresta por amostra | × a dobrada |
|---|---:|---:|
| **dobrada** — literais no texto, desenrolada (a forma da FITA) | `0,00063` | `1,00×` |
| `storage`, limite em `arrayLength` | `0,00200` | `3,19×` |
| `storage`, contagem no texto | `0,00209` | `3,33×` |
| **`uniform`, acesso UNIFORME no grupo** | `0,00099` | **`1,58×`** |
| `storage`, acesso **DIVERGENTE** (4 listas por grupo) | `0,00760` | **`12,11×`** |
| `uniform`, acesso **DIVERGENTE** (4 listas por grupo) | `0,01083` | **`17,26×`** |

⭐⭐⭐ **E a curva da divergência é LINEAR no número de listas distintas que um grupo lê** — a
assinatura de leituras SERIALIZADAS (`N = 32`, pegada `≤ 8 KiB`, que cabe na cache de 1.º nível ⇒ o
que a coluna move é a divergência e não a memória):

| listas distintas por grupo | `storage` | `uniform` |
|---:|---:|---:|
| **1** (uniforme) | `0,110 ms` | **`0,075`** |
| 2 | `0,167` | `0,169` |
| 4 | `0,290` | `0,351` |
| 8 | `0,533` | `1,129` |

⚠️ **E o `uniform` inverte-se:** ele é o mais barato quando o acesso é uniforme (é para isso que um
banco de constantes existe) e o **mais caro** quando não é (`43×` do próprio D=1 a oito listas,
contra `8×` do `storage`). *A escolha do tipo de buffer depende da coerência do acesso, não do
tamanho dos dados.*

### ⇒ A conta, e o que sobra do desenho

Com o factor de poda do censo (`11,8×` na mediana, `3,4×` no pior caso de uma grelha `16×16`) contra
o imposto de acesso:

| desenho | poda | imposto | ganho |
|---|---:|---:|---:|
| **consulta por AMOSTRA** (lista da célula, 4 por grupo) | `11,8×` | `12,1×` | **`0,97×`** |
| a mesma, no pior caso | `3,4×` | `12,1×` | **`0,28×`** — `3,6×` mais LENTA |
| **lista por GRUPO, em `uniform`** (mediana) | `11,8×` | `1,58×` | **`7,5×`** |
| a mesma, no pior caso | `3,4×` | `1,58×` | **`2,2×`** |

⛔⛔⛔ **A consulta por amostra está REFUTADA: a poda compra `11,8×` e a divergência cobra `12,1×`, e
elas cancelam-se.** *É a wave inteira salva por uma sonda de um dia — e a razão de a nota original
estar errada não é o número, é o MECANISMO: uma poda por ramo-e-limite é um algoritmo de CPU.*

⭐⭐⭐ **O desenho que sobrevive é o COERENTE POR GRUPO:** uma lista por grupo de threads, num buffer
`uniform`, indexada pelo `workgroup_id` — e a unidade coerente por construção é o **ladrilho do
ecrã**, que é exactamente por onde o traçado de CPU já corta. ⇒ com o censo de regiões de MUNDO
(`16³`: p50 `109` linhas, pior `372`) e o imposto de `1,58×`, a cena `5` aterra em **`~9,5 ms`** na
mediana e `~22` na pior região, contra `31,96` hoje e um orçamento de `16,7`.

### ⛔⛔⛔⛔ E ESSA MEDIÇÃO FOI FEITA — A WAVE DO TORNO ESTÁ RECUSADA POR MEDIÇÃO

A pergunta que o desenho abria era: *quanto a poda ainda compra na granularidade que é coerente por
construção?* A sonda é a `diag_a_poda_coerente_por_grupo`, e ela percorre a **aritmética de região
do PRODUTO** ([`ph2d_field_render::linhas_por_ladrilho_for_test`], que chama o `tile_t_range`, o
`slab_bounds` e o `slab_region` da marcha) — *reconstruí-la mediria outro programa*.

`1920×1080`, a peça do vaso, `931` linhas na fita inteira, e a última coluna já com o imposto de
`1,58×` de um buffer `uniform` lido coerentemente:

| ladrilho | fatias | regiões | p50 | pior | poda p50 | poda pior | **ganho p50** | **ganho pior** |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `64 px` | 1 | `300` | `382` | `861` | `2,4×` | `1,1×` | **`1,5×`** | **`0,7×`** |
| `32 px` | 1 | `1 072` | `356` | `835` | `2,6×` | `1,1×` | `1,7×` | `0,7×` |
| **`8 px`** | **1** | `15 058` | `351` | `835` | `2,7×` | `1,1×` | **`1,7×`** | **`0,7×`** |
| `8 px` | 4 | `39 406` | `178` | `505` | `5,2×` | `1,8×` | `3,3×` | `1,2×` |

⭐⭐⭐ **A leitura é uma frase: a granularidade que é COERENTE não poda, e a que PODA não é
coerente.** Um grupo de `64` threads cobre um ladrilho de `8×8` px, logo a linha `8 px · 1 fatia` é
**a** granularidade coerente por construção — e ali a poda compra `2,7×` na mediana e **`1,1×` no
pior caso**, que o imposto de acesso transforma em `1,7×` e **`0,7×`**.

⭐ **E encolher o ladrilho não ajuda**, o que nomeia a causa: de `64` para `8 px` a poda vai de `2,4×`
a `2,7×` — *o que limita não é a largura do ladrilho, é a PROFUNDIDADE do tronco dele*, que atravessa
a peça inteira e por isso vê quase todo o perfil em `(u, v)`. Repartir em fatias de profundidade cura
isso (`5,2×`) e é **exactamente** o que rompe a coerência: numa esfera-marcha as threads de um
ladrilho estão em `t` diferentes, e a tabela da divergência diz que cada lista extra custa `~1×` o
custo uniforme.

⇒ ⛔⛔⛔ **A `W9` não leva o índice de perfil ao dispositivo.** O melhor caso honesto é `1,7×` na
mediana e uma PERDA no pior caso, e mesmo o `1,7×` deixa a cena `5` em `~19 ms` contra um orçamento
de `16,7`. *A obra não alcança o objectivo que a justificava.*

### ⭐ O que fica no lugar, com o número: a lever é o custo POR PRIMITIVA

O perfil do vaso tem `24` primitivas e paga `931` linhas — `38,8` por primitiva. Medido pela
[`diag_o_tecto_da_wave_do_torno`] com a polilinha densa, uma primitiva **RECTA** custa `26,7`
(`2 639 / 99`) ⇒ as `12` **ARCOS** custam `(931 − 12 × 26,7) / 12 =` **`51,0`**, ou seja **`1,9×`
uma recta**, e elas são **`66 %`** da fita.

⇒ *a contagem é o desenho do artista e não tem gordura; o que tem é o CONSTANTE.* Duas direcções,
nenhuma medida ainda:

1. **O arco mais barato.** `51` linhas para uma primitiva é muito, e a metade do enrolamento dele (a
   meia-lua) é uma lei própria.
2. **O SINAL.** Numa recta, `~10` das `26,7` linhas são o enrolamento; um sinal tirado da primitiva
   MAIS PRÓXIMA (a normal pseudo-angular, exacta em 2D) trocaria `N` contribuições por **uma**.
   ⚠️ Isto muda a lei nos DOIS motores e tem juiz (`the_query_is_the_same_law_as_the_tape`).

### ⭐⭐⭐ E a decomposição por linha FOI medida — e ela põe um CHÃO na cena `5`

A `diag_a_decomposicao_de_uma_primitiva` (`ph2d-field-eval`) monta a metade da DISTÂNCIA pelas
**mesmas portas** que o produto monta (`dist2_recta_tree` · `dist2_tree`, mesmo acumulador de `min`)
e lê o SINAL por diferença, com a **inclinação em `N`** a tirar o `O(1)` (a raiz, o produto do sinal,
a redução do enrolamento):

| primitiva | total | distância | sinal |
|---|---:|---:|---:|
| **recta** | `26,52` | `15,82` (`60 %`) | `10,70` (`40 %`) |
| **arco** | `50,86` | `25,64` (`50 %`) | **`25,21`** (`50 %`) |

⭐ O `50,86` do arco confirma, por outro caminho, o `51,0` que saíra da subtracção no vaso. E a conta
da cena `5` (`12` rectas + `12` arcos) fecha: distância `12 × 15,82 + 12 × 25,64 =` **`498`** ·
sinal `12 × 10,70 + 12 × 25,21 =` **`431`** · total `929` contra `931` medidas.

⭐⭐⭐ **O SINAL é `46 %` da fita, e no ARCO ele custa o mesmo que a distância** — porque o
enrolamento de um arco é o da CORDA **mais** a meia-lua ([`meia_lua_raio_tree`]), e a meia-lua só
existe por o enrolamento ser uma soma sobre cordas. *Um sinal tirado da primitiva MAIS PRÓXIMA
(a normal pseudo-angular, exacta em 2D) trocaria `N` contribuições por uma e apagaria a meia-lua
inteira.*

⛔⛔ **Mas o CHÃO que isso deixa está acima do orçamento, e é ele o achado:**

| cenário | linhas | quadro previsto |
|---|---:|---:|
| hoje | `931` | `31,96 ms` (medido) |
| com o sinal em `O(1)` | `~513` | **`19,7 ms`** (`1,6×`) |
| **distância sozinha, sinal GRÁTIS** | `498` | **`19,2 ms`** |
| o orçamento | — | `16,7 ms` |

⇒ ⛔⛔⛔ **Nenhuma cura que mantenha a distância como um `min` desenrolado sobre `24` primitivas põe
a cena `5` dentro do orçamento** — o chão é `19,2 ms`. E encurtar a distância exige **poda**, que a
divergência recusa. ⚠️ **E as `24` primitivas são o DESENHO**: o vaso tem `12` âncoras com `10` raios
de quina não-nulos, e uma quina arredondada é um arco mais o que sobra das duas arestas que ela
corta — *não há gordura na contagem*.

### ⭐⭐⭐⭐ E O DONO PERGUNTOU A COISA CERTA: «não seria possível criar vasos com FÓRMULAS?»

**Ordem/pergunta do dono, 2026-09-23:** *«vamos retirar a possibilidade de usar paths para criar
formas no modelador. Não seria possível criar vasos com fórmulas para que tudo fique rápido?»*

⭐ **O mecanismo que ele aponta está CERTO**, e a decomposição acima diz porquê: um contorno
desenhado custa **`O(pedaços)`** (`26,5` linhas por recta, `50,9` por curva) e uma fórmula custa
**`O(grau)`** — independentemente de quantas curvas a silhueta pareça ter.

⚠️ **Mas «possível» tem duas metades e só uma é o relógio:** *quanto custa* e *quanto do DESENHO a
fórmula consegue dizer*. A `diag_o_vaso_por_formula` mede as duas.

**A forma da fórmula, e porque é esta:** um sólido de revolução é, no plano `(u, v)` com
`u = √(x² + z²)`, a região entre **duas funções da altura**. O sólido é *«dentro da parede externa e
na faixa de altura»* **menos** a cavidade *«dentro da parede interna **E** acima do fundo»*. As duas
funções são polinómios de **Chebyshev** avaliados por **Clenshaw** (`3` operações por grau).
⚠️⚠️ E o campo é **normalizado por um majorante global da inclinação**: `u − fora(v)` **não** é a
distância à curva — ela é MAIOR quando a curva é inclinada, e uma esfera-marcha que acredite num
valor maior do que a distância dá um passo **para dentro do sólido**.

O vaso da cena `5` tem `24` primitivas e `931` linhas. Ajustado à silhueta que a própria peça
produz (`512` alturas, pelo sinal do campo do perfil):

| grau | erro da parede externa | erro da interna | linhas | × a desenhada | quadro previsto |
|---:|---:|---:|---:|---:|---:|
| `4` | `0,0699` | `0,0483` | `49` | `19,0×` | `5,8 ms` |
| `8` | `0,0196` | `0,0066` | `73` | `12,8×` | `6,5 ms` |
| `12` | `0,0091` | `0,0048` | `97` | `9,6×` | `7,2 ms` |
| **`16`** | **`0,0027`** | **`0,0028`** | `121` | `7,7×` | **`7,9 ms`** |
| `24` | `0,0016` | `0,0011` | `169` | `5,5×` | `9,4 ms` |

⭐⭐⭐ **Ao grau `16` as duas paredes ficam a `0,003` — `0,9 %` do raio do vaso — e a cena aterra em
`~7,9 ms` contra `31,96`, bem dentro do orçamento de `16,7`.** ⇒ *a resposta à pergunta dele é SIM, e
o número é `4×`.*

⭐⭐ **E o desenho NÃO tem de sair para isso:** o ajuste acima foi feito **a partir da peça
desenhada**. O caminho é *«o artista desenha e o modelador ajusta a fórmula»*, não *«o artista deixa
de desenhar»*.

### ⛔⛔⛔ Três limites da rota da fórmula, e o terceiro é o que decide o alcance

1. **Ela é APROXIMADA.** `0,9 %` do raio ao grau `16`; a paridade de `100,000 %` que esta casa
   mantém entre os dois motores sobrevive (a fórmula é a MESMA lei nos dois), mas a peça deixa de
   ser o desenho **ao bit**.
2. **Um polinómio não tem QUINA.** Um lábio desenhado a esquadro fica macio; o desenho tem de dizer
   a quina por outro termo (o `max` entre paredes já produz uma).
3. ⛔⛔ **Ela serve o TORNO e não serve o EXTRUDE.** Um sólido de revolução é a região entre duas
   **funções da altura**; a secção de um extrude é uma **curva fechada** em `(x, y)`, que só é uma
   função (`r(θ)`) se a forma for **estrelada** em relação a um centro. *Um «C», uma espiral ou um
   contorno com um braço que envolve não são.* ⭐ A família estrelada é exactamente a que o catálogo
   já cobre por fórmula (a superfórmula de Gielis, `ops_gielis`), o que é uma confirmação
   independente de que o corte está aí.

### ⛔⛔ E o tamanho de RETIRAR os desenhos, medido

| o que a rota do desenho alcança | medido |
|---|---:|
| formas do catálogo que nascem de um contorno | `Extrude` · `Polygon` · `Revolve` |
| ficheiros que as nomeiam | **`73`** |
| ficheiros que carregam o vínculo vivo ao desenho (`FieldProfileSource`) | `16` |
| linhas de produto nos ficheiros envolvidos | **`45 053`** |

⇒ retirar a rota do desenho **não** retira só o vaso: retira *«desenho uma forma e ela vira
sólido»*, que é a costura das waves `W53`–`W58` (o fluxo do MoI, o selo `LNK` na Hierarquia, o
`Unlink`/`Link Drawing`, a resolução do contorno) — e, pelo limite `3`, a fórmula **não** substitui o
extrude de um contorno qualquer. *É decisão do dono, e ela está agora com o número dos dois lados.*

### ✅ E ELA FOI CONSTRUÍDA — decisão do dono: **«manter o lápis e ajustar a fórmula»**

Posto o número dos dois lados, ele escolheu a rota que mantém a caneta. ⇒ o `Primitive::Revolve`
**desce por fórmula quando a silhueta o permite**, e o artista não muda de gesto nenhum.

**O que ship, e o que NÃO ship:**

| | |
|---|---|
| decide | a **SILHUETA**, não um knob: `None` ⇒ o contorno desenhado, que sabe desenhar qualquer coisa |
| schema | **intocado** — nenhuma primitiva nova, nenhum campo novo |
| paridade CPU↔placa | **intacta por construção** — a árvore é UMA e os dois motores saem dela |
| bissecção | `PH2D_FIELD_TORNO_EXACTO=1` |

**A cena `5` do produto, medida** (`1920×1080`, mínimo de `3`, `--release`, A/B no mesmo binário):

| | linhas de WGSL | passos/acerto | quadro | divisor do prévio |
|---|---:|---:|---:|---:|
| contorno desenhado | `934` | `43,5` | `32,53 ms` | **`2`** |
| **por fórmula** | **`124`** | `92,7` | **`16,63 ms`** | **`1`** |

⭐⭐ **`1,96×`, e o divisor cai de `2` para `1`** — *é isso que o artista vê: o prévio do vaso deixa de
desenhar a meia resolução enquanto ele arrasta*. O corpus vai de `14` para **`15` de `22`**. ⚠️ E a
corrida da fórmula foi tirada a `66 %` de CPU ociosa contra `84 %` da exacta, logo o `16,63` é
**pessimista**.

⭐ Na silhueta densa (o mesmo vaso como polilinha, `2 639` linhas) o A/B a `93 %` ociosa dá
**`92,15 → 16,08 ms`, `5,7×`**.

⚠️ **E o que sobra tem endereço: os passos por acerto DOBRAM** (`43,5 → 92,7`), porque a
normalização usa um majorante **global** da inclinação (`3,4` no vaso ⇒ `k ≈ 0,28`). *A lever que
resta vale até `2,1×` e é uma normalização mais apertada* — ⛔ e ela tem de continuar a ser um
majorante VERDADEIRO, senão a peça fura.

### ⛔⛔⛔⛔ E O SMOKE REPROVOU-A — *«ao arrastar fica grosseiro ainda»*, e o dono tem razão

⭐ **O que ele apanhou não é o torno: é que o DISPOSITIVO NÃO MARCHA NO MODO DE OMISSÃO.**

O despacho do quadro ([`smoke_draw_thread::traca`]) chama a placa numa condição só:

```rust
let pelo_dispositivo = matches!(p.shading, Shading::Render) && !mundos.is_empty() && takes_the_frame(..);
```

⚠️⚠️ **Ela junta DUAS perguntas** — *«a placa sabe MARCHAR esta peça?»*, que é **geometria** e não
tem modo, e *«a placa sabe PINTÁ-LA?»*, que pede o material, o céu e o olhar. E o `#[default]` do
[`crate::shading::Shading`] é o **`Matcap`**, que o próprio doc dele chama *«a omissão de um
modelador»*. ⇒ **ao abrir uma cena, o arrasto vai todo pela CPU.**

**Medido** (`diag_o_arrasto_no_modo_de_omissao`, a cena `5`, `93 %` de CPU ociosa, `--release`, o
divisor pela porta do produto):

| tela | contorno desenhado | por fórmula | divisor |
|---|---:|---:|---:|
| `1920×1080` | `90,17 ms` | `88,23` | **`D=3` nos dois** |
| `1400×900` | `51,88` | `54,76` | `D=2` nos dois |
| `960×540` | `27,93` | `22,93` | `D=2` nos dois |

⛔⛔⛔ **Na CPU a fórmula não compra nada** (`2 %`, e a `1400×900` é `5 %` PIOR), e o divisor não se
move. ⭐ **A razão é que a CPU já tinha a poda**: ela especializa a árvore por ladrilho
(`RegionCompiler`) e corre a fita em JIT com SIMD, logo as `934` instruções não lhe custam `7,5×` as
`124` — o que lhe sobra são os **passos**, e a fórmula **dobra-os**. *A cura serve o motor em que a
contagem manda, e a CPU não é esse motor.*

⇒ ⚠️⚠️ **A `W9` inteira — o `14 de 22` nítidas, a fita inerte, a lei do dono, o torno por fórmula —
foi medida no caminho do DISPOSITIVO, que o artista não toma sem trocar de modo.** *É a classe de
defeito que o cabeçalho daquele mesmo ficheiro avisa por escrito, três parágrafos acima da linha que
a contém.*

### ⛔⛔⛔ E a cura foi CONSTRUÍDA e REVERTIDA no mesmo dia

Separar as duas perguntas é três linhas, e com a marcha aberta ao `Matcap` **três testes de
`view_menu`** — que **desenham** um quadro e **não** são `#[ignore]` — passaram a morrer com
`NVVM compilation failed: 3` e `SIGSEGV` **ao sair do processo**, depois de PASSAREM: `0/0/0`
estouros na árvore de base contra **`9/9/6`** com a separação, e `--test-threads=1` não cura.

⭐⭐ **O mecanismo é o ALCANCE e é ele que decide:** a cura faz **testes de unidade comuns tomarem a
placa**, e nesta máquina ela é **partilhada** — no meio desta medição outra linha segurava-a há
`300 s`. Isso colide com a lei da casa (*gates de GPU são `#[ignore]` e precisam de adaptador*) e com
o guarda de exclusão, que um teste comum não pede. ⇒ *abrir o caminho de omissão ao dispositivo é
abrir a placa a toda a suíte, e isso é uma decisão maior do que a cura.*

⚠️⚠️ **E a 1.ª bissecção deste estouro MENTIU por ser de UMA corrida** — ela deu a metade do `Matcap`
como verde, e a corrida seguinte do MESMO estado deu vermelho. *Numa família de sinais como esta, uma
corrida não bissecta nada* (e a lei do `CLAUDE.md` §5.0 di-lo por extenso).

⇒ Fica **dívida NOMEADA e gateada**: `a_marcha_no_dispositivo_ainda_pergunta_o_modo_e_isso_e_divida`
é uma **catraca ao contrário** — ela reprova no dia em que alguém tirar o modo daquela condição, para
que a cura venha com a atribuição do estouro.

### ⛔⛔⛔⛔ E O 2.º REPORT — *«ainda perde resolução ao arrastar»* — ACHOU UM DEFEITO MEU: o ajuste corria POR REGIÃO

⭐ **A causa não é o modo nem a placa: é que a fórmula era ajustada por ladrilho.**

O [`ph2d_field_eval::RegionCompiler`] chama o `specialised_profile` **por ladrilho × fatia de
profundidade**, e a 1.ª redacção do torno por fórmula fazia **ali** a extracção da silhueta (`128`
alturas) e **dois** ajustes de mínimos quadrados `17×17`.

| | |
|---|---:|
| ajustar a fórmula | **`0,0748 ms`** |
| montar a árvore EXACTA da peça inteira | `0,0280 ms` |
| regiões por quadro a `1920×1080`, ladrilho `64` | `750` |
| ⇒ **só a ajustar** | **`56 ms`** por quadro (`2 948` com ladrilho `8`) |

⛔⛔ **E o A/B de relógio não o via, porque dois números grandes se cancelavam:** o traçado de CPU
lia `90,17 ms` pela lei exacta e `88,23` pela fórmula — *a poupança da marcha pagava o gasto da
montagem*, e eu escrevi «na CPU a fórmula não compra nada» sobre um defeito meu. ⚠️ *Uma diferença
de `2 %` entre dois caminhos muito diferentes é um sinal para procurar o que se cancelou, não uma
conclusão.*

⭐⭐ **A cura é a que esta casa já tinha escrita ao lado:** o `RegionCompiler` constrói a
[`ProfileIndex`] **uma vez** e não por região, com a razão no doc. ⇒ a árvore da fórmula passa a
viver no mesmo mapa (`RegionCompiler::formula`), e a região devolve-a por clonagem de um ponteiro —
*uma fórmula não depende da região, logo a resposta dela é a MESMA árvore*.

⭐ **E uma peça feita só de tornos por fórmula deixa de pedir LADRILHOS** (`is_worth_it`): a árvore
dela não tem arestas para cortar, logo ladrilhar faz o quadro pagar a montagem por região sem poupar
um passo.

**Medido depois da cura** (`diag_o_arrasto_no_modo_de_omissao`, a cena `5`, o traçado de **CPU** que
é o do modo de omissão; ⚠️ `43`–`52 %` de CPU ociosa nos dois lados, logo a RAZÃO vale e os
absolutos são pessimistas):

| tela | contorno desenhado | por fórmula | ganho | divisor |
|---|---:|---:|---:|---|
| `1920×1080` | `79,0 ms` · `D=3` | **`46,9`** · `D=2` | `1,68×` | melhorou |
| `1400×900` | `62,4` · `D=2` | **`25,2`** · `D=2` | **`2,48×`** | — |
| `960×540` | `38,0` · `D=2` | **`13,6`** · **`D=1`** | **`2,79×`** | ⭐ resolução CHEIA |

⇒ ⚠️ **E a verdade que fica:** a `1400×900` o quadro lê `25,2 ms` contra um orçamento de `16,7` —
**o dono ainda vai perder resolução numa tela grande**, e a razão já não é a peça: *o traçado de CPU
é ~`1,5×` lento de mais para aquela tela com qualquer peça*.

⭐⭐⭐ **A régua desta cura é uma CONTAGEM e não um relógio**, e é por isso que ela existe: a árvore
que a região devolve é a MESMA **ao bit** nos dois caminhos, e nenhuma régua de valor os distingue.
Gates: `a_formula_e_ajustada_uma_vez_por_peca` (o contador, com o piso de população das regiões) e
`uma_peca_so_de_formula_nao_pede_ladrilhos` (com um EXTRUDE como controlo — um `is_worth_it` sempre
falso desligaria a especialização de toda a casa).

⚠️ **E o meu sonda tinha uma coluna de outra rota:** ela media o `gpu_frame::march`, que traz o
G-buffer de volta pelo barramento (`~50 MB`) e que o Render só pede quando REFINA — `119`–`123 ms`
onde o pintor lê `16,6`. *Uma coluna de uma rota que o produto não toma lê-se como o preço da placa.*
Corrigida para o `gpu_frame::paint`.

### ⛔⛔ As três coisas que a construção achou, e nenhuma era a lei

1. **A decisão escrita em UM sítio divergiu na primeira corrida.** O todo descia por fórmula e o
   `RegionCompiler::specialised_profile` continuava a cortar as arestas do contorno **desenhado** —
   duas leis —, e três gates disseram-no juntos (`the_specialised_document_agrees_inside_its_region`,
   `the_tiled_march_draws_the_same_image_as_the_row_march` e o oráculo do toro). ⇒ a decisão é uma
   **porta com dois leitores** (`profile::torno_por_formula`), e a resposta da região é a MESMA
   árvore: *uma fórmula não tem arestas para cortar*.
2. ⛔⛔⛔ **Eu escrevi a guarda da marcha e não a gateei — uma mutação sobrevivente disse-o.** Apagar
   a normalização (`k = 1`) passou as **oito** paridades de imagem do `ph2d-field-render` e os três
   gates da família. ⚠️ E é a direcção **insegura**: um campo que promete mais do que a distância faz
   a esfera-marcha dar um passo para **dentro** do sólido. ⇒ `a_formula_nao_atravessa_a_peca` mede
   `‖∇f‖ ≤ 1` num domínio de `25³` pontos, **com o controlo por baixo** (`‖∇f‖ ≥ 0,2`), senão uma
   normalização grosseira passaria e faria toda marcha rastejar.
3. ⚠️ **Dois gates de exactidão passaram a medir a lei EXACTA pela porta que não decide**
   (`profile::probe_sd_revolve_exacto`), e um deles acusava o mecanismo errado: o
   `the_seam_of_a_lathe_lies_on_the_axis_and_is_not_a_wall` leu `−0,0171` contra `−0,0200` e disse
   *«a costura está a ser tratada como parede»* — ⛔ **a costura não existe na fórmula**; o que ele
   viu foi o minorante conservador. *Um gate cuja mensagem diagnostica uma causa tem de medir a lei
   em que essa causa vive.*

### ⭐ A cerca da fidelidade, e o que ela devolve ao contorno desenhado

`FIDELIDADE = 0,01` do raio da peça, medida contra a silhueta que a própria peça produz, nas duas
paredes. ⚠️ **É um limite de PRODUTO e o recurso dele é o OLHO** — o dono aprovou-o nesses termos com
o número ao lado (`0,8 %` no vaso). ⛔ **Ela não pode ser a `Profile::tolerance`**: a do vaso é `1e-4`
e o ajuste ao grau `16` erra `2,6e-3` — `26×` mais, *e com aquela barra a peça do dono seria
recusada*.

⭐ **E é a cerca que devolve o perfil DEGRAU ao contorno desenhado:** uma roldana com escalões tem
paredes verticais, que não são funções da altura — o erro do ajuste diz isso em números, e a recusa
sai de uma medição e não de uma lista de formas (gate
`a_cerca_da_fidelidade_recusa_um_perfil_de_escaloes`, com o copo como CONTROLO).

⚠️ **E a fixtura da fidelidade é uma parede ONDULADA, não um copo** — por uma mutação sobrevivente:
baixar o `GRAU` de `16` para `4` passava sobre um copo, cujas paredes são rectas. *Uma fixtura lisa
não testa o grau.*

**`6` mutações, `6` sangram.** Gates: `o_torno_por_formula_fica_dentro_da_fidelidade` (a POSIÇÃO da
superfície por bissecção contra a lei exacta — ⚠️ e o piso de população apanhou a 1.ª régua, que
exigia o eixo DENTRO do sólido e media `17` de `64` alturas) ·
`a_cerca_da_fidelidade_recusa_um_perfil_de_escaloes` · `na_formula_o_eixo_tambem_nao_e_uma_parede` ·
`a_formula_nao_atravessa_a_peca` · `o_vaso_desce_por_formula_e_a_fita_cabe_numa_mao`.

### ⏳ O que a fila leva daqui, com o mecanismo de cada candidato

1. ⭐⭐⭐ **O PERFIL COMO TEXTURA 2D** — o candidato que nenhuma medição desta wave recusa, e é a
   resposta da indústria. Uma consulta a uma textura é `O(1)`, passa pela cache de texturas e é
   **espacialmente coerente por construção** ⇒ *ela não paga o imposto de divergência que matou a
   consulta por lista*, que é exactamente o que as tabelas acima medem. ⛔ **Ela é APROXIMADA**, e
   isso é a decisão: um campo que SUBESTIMA a distância é seguro para a esfera-marcha (ela só dá
   passos menores), e o acerto final pode ser refinado pela fita exacta. ⚠️ A paridade de
   `100,000 %` que esta casa mantém entre os dois motores **não sobrevive** a isso sem uma
   fronteira nova — *é decisão de produto, não de engenharia.*
2. ⏳ **O sinal em `O(1)`** — `1,6×` medido, e apaga a meia-lua. ⚠️ Muda a lei nos **dois** motores e
   tem juiz (`the_query_is_the_same_law_as_the_tape`). Sozinho não alcança o orçamento.
3. ⏳ **A distância do ARCO a `25,64`** — uma distância a um arco é «a distância ao círculo, presa ao
   sector»: `~10` operações. O `25,64` medido diz que a implementação faz mais, e ninguém foi ver.

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

### ⭐⭐⭐⭐ E A DÍVIDA NOMEADA FOI PAGA: **o modo de OMISSÃO vai à placa** — `31,45 → 13,11 ms`, `D=2 → D=1`

**Ordem do dono, 2026-09-23:** *«nosso objetivo é ter tudo rodando em runtime com o melhor render
que uma game engine já viu (desde que compatível com tudo, inclusive mobile) … faça tudo que for
necessário para que tudo funcione em tempo real com boa resolução»*.

O `#[default]` do `Shading` é o **matcap**, e até aqui só o `Render` era pintado no dispositivo ⇒ ao
abrir uma cena o arrasto ia **todo** pela CPU. É o report *«ao arrastar fica grosseiro ainda»*.

⛔⛔⛔ **E a 1.ª cura que eu desenhei era ABRIR A MARCHA ao matcap — ela PIORAVA.** O
`gpu_frame::march` devolve o G-buffer pelo barramento (`49,8 MB` a `1920×1080`, `119`–`123 ms`), que
é **mais lento do que a CPU inteira**. ⭐ *O ganho nunca foi a marcha estar na placa; é a IMAGEM não
atravessar o barramento* — e é por isso que a cura é um **PASSE que pinta**, e não uma condição
alargada.

**MEDIDO** (`--release`, `87 %` de CPU ociosa, cena `5`, o matcap real de `749²`):

| tela | CPU (modo de omissão) | divisor | **placa** | divisor | ganho |
|---|---:|---|---:|---|---:|
| `1920×1080` | `31,45 ms` | `D=2` | **`13,11`** | **`D=1`** | `2,40×` |
| `1400×900` | `20,04` | `D=2` | **`9,56`** | **`D=1`** | `2,10×` |
| `960×540` | `7,61` | `D=1` | `4,62` | `D=1` | `1,65×` |

⇒ a `1080p` o quadro de omissão passa a caber no orçamento de `16,7 ms` **com resolução cheia**.

**Paridade verde à 1.ª corrida** contra o `shade_with` da CPU sobre a MESMA marcha: pior byte `≤ 1`.
⚠️ A barra é o **PIOR BYTE** e não uma fracção — em 19/09 uma paridade escrita como fracção leu
`99,579 %` contra `99,5` e *passava por `0,079`* sobre um dispositivo que não pedia o canal.

#### ⭐⭐⭐ Porque é um passe PRÓPRIO, e não um modo do pintor de material — três razões medidas

1. **ARMAZÉNS.** O pintor de material liga **`12`** contra o piso garantido de **`8`** do WebGPU ⇒
   numa placa que fique no piso o caminho de render **não corre**. Este liga **`8`**: *o modo de
   omissão corre em toda placa conforme, e o de material só onde há folga.*
2. **Um matcap não lê o CAMPO da peça.** O texto do shader não tem o `FIELD_SLOT`, logo o cache de
   pipelines (cuja chave é o TEXTO) acerta para sempre — *acrescentar uma forma não recompila o modo
   de omissão*.
3. **Compilar é o caro.** Arrastar o OpenPBR, as sondas, o estilo e a lei do dono para um quadro que
   não lê nenhum deles.

#### ⛔⛔⛔⛔ E o `SIGSEGV` que bloqueava isto está ATRIBUÍDO, com controlo positivo nos dois lados

A dívida dizia que separar os modos fazia três testes de `view_menu` morrerem com
`NVVM compilation failed: 3` + `SIGSEGV`. Reproduzido **`3` de `3`**, e a assinatura é exacta:

```text
test result: ok            ← o teste PASSA
NVVM compilation failed: 3 ← o driver, DEPOIS
(signal: 11, SIGSEGV)      ← o PROCESSO não consegue sair
```

| sonda | em voo | veredito do **processo** |
|---|---:|---|
| `diag_esperar_pelo_quadro_cura` | `1` · chegaram `1` | **sai `0`** |
| `diag_o_quadro_em_voo_mata_o_processo` | `1` | **`SIGSEGV`** |

⇒ **o discriminador é a ESPERA, não a placa.** O mecanismo é trabalho na placa numa thread
**DESANEXADA** (`std::thread::spawn`, sem `join`) a sobreviver ao processo — e os **77** gates de
placa desta crate nunca estouraram porque chamam as portas **em série, na thread do teste**.

⚠️⚠️ **A 1.ª redacção das duas sondas era VÁCUO:** ela drenava FORA do `armed_with`, onde o módulo já
largou o `Receiver`; leu **`esperei por 0 quadro(s)`**, estourou nas DUAS metades, e eu quase a li
como *«esperar não cura»*. ⇒ *o CONTROLO passou a ser uma ASSERÇÃO*, e uma corrida que não deixe
quadro em voo reprova **alto** em vez de mentir.

⭐ **A lei que fica:** *a placa é do PRODUTO; um teste de unidade comum não a entrega à thread que
desenha* (`gpu_frame::para_o_quadro`, distinta do `shared` que os gates usam). É a mesma lei que o
`CLAUDE.md` já escreve para os gates de GPU, aplicada ao único consumidor que a pedia **sem pedir**.

⏳ **E a dívida de PRODUTO fica NOMEADA com a reprodução na mão:** ao fechar a janela com um quadro
em voo o app pode morrer igual, e **isso já era verdade no `Render` antes desta wave**. A cura é o
processo **DRENAR** os quadros em voo antes de sair — uma thread de desenho que o processo POSSUI em
vez de N desanexadas — e ela é wave própria. A semente já existe
(`Smoke::espera_pelos_quadros_em_voo`), sob `cfg(test)` e **sem consumidor de produto**, que é a
forma que esta casa caça: *uma lei viva e órfã*.

### ⭐⭐⭐⭐ E O NÚMERO DOS ARMAZÉNS ESTAVA **`3` ABAIXO** DO REAL — o guarda aceitava o que a `wgpu` recusa

`paint::ARMAZENS` era o literal `9`, com o doc a contar *«seis do grupo `0` e três do grupo `1`»*.
**Contado: o grupo `1` liga SEIS, e o passe liga `12`.** A contagem envelheceu quando o passe ganhou
as sondas, o campo do chão e a cena do brilho.

⛔ **A consequência é exactamente o que aquele número existe para impedir:** o guarda é
`storage_slots() < ARMAZENS`, logo com `9` ele **aceita** uma placa que anuncie `9`, `10` ou `11`
ranhuras — e ali a `wgpu` recusa o layout **a meio de um quadro**. *(O produto escapava por sorte: o
piso é `8`, e `9 > 8` já recusava a placa mínima.)*

⇒ **os dois números passam a ser CONTADOS** das listas de ligação (`trace::entradas_da_marcha` ·
`paint::entradas_do_pintor` · `matcap::entradas`, com `conta_armazens`):

```text
[armazéns] matcap 8 · material 12 · piso 8
```

⚠️⚠️ **E a história deste gate em TRÊS formas vale mais que o número:**

1. um `#[test]` sobre dois **literais** — o clippy apanhou-o (`assertions_on_constants`): *um
   `assert!` sobre duas constantes é dobrado pelo compilador*, logo nunca podia reprovar numa árvore
   que compila;
2. um `const _: () = assert!(…)` — falha a compilar, **e ainda media o literal**;
3. ⭐ **este**, que mede as **LISTAS DE LIGAÇÃO** que os passes de facto ligam.

⇒ *a forma 2 NUNCA teria apanhado este defeito*. **Uma asserção sobre um número escrito à mão
confirma o que alguém escreveu, nunca o que o código faz.**

#### ⏳ E a metade *«compatível com mobile»* da ordem tem ENDEREÇO e ARITMÉTICA

O pintor de **material** liga `12` contra o piso de `8` ⇒ numa placa mínima conforme ele **não
corre**, e o quadro cai na CPU. ⭐ **Três dos doze são estruturalmente opcionais, e os comentários do
próprio passe já o dizem:** as **sondas** (*«existe mesmo sem ricochete»*), o **campo do chão**
(*«existe sempre — um armazém vazio não é ligável»*) e a **cena do brilho** (*«com o brilho
desligado ele tem UM texel»*). Eles são declarados e ligados **mesmo quando o quadro não os lê**,
porque um `BindGroup` recusa uma entrada em falta.

⇒ **a conta do caminho:** declarar o layout **por quadro** (só o que aquele quadro lê) leva `12 → 9`;
o último degrau é fundir o **`conta`** (um `atomic<u32>` num buffer de `16 B`) na primeira palavra do
**`borda`**, e dá **`8`**.

⚠️ **Isto é um caminho com números, não uma wave feita:** ele muda a chave do cache de pipelines (o
layout passa a variar com o quadro) e pede gate de paridade por configuração. ⛔ E a ordem é
importante — *o modo de OMISSÃO já corre em toda placa conforme*, logo isto é o que falta ao modo de
**material**, não ao que o artista vê ao abrir uma cena.

### ⭐⭐⭐⭐ E A PERGUNTA QUE DECIDE A GRELHA DE VOLUME FOI MEDIDA

A proposta (o mecanismo do MagicaCSG) troca **avaliar o campo** por **uma consulta trilinear** ⇒ ela
só paga se o campo for a maior parte do quadro.

⛔⛔ **E a coluna da PLACA desta sonda media o `march` — a SEGUNDA vez nesta família** (a sonda irmã
já carrega a mesma correcção escrita ao lado dela): ela lia `90`–`177 ms` onde a rota do produto lê
`3`–`95`, e **ia decidir uma wave**. Repontada ao `pinta_matcap` (`42 %` ociosa, logo os absolutos
são pessimistas e a FORMA é o que decide):

| peça | linhas | passos/acerto | CPU | **placa** | ns/amostra (placa) |
|---|---:|---:|---:|---:|---:|
| esfera | `8` | `38,0` | `27,9 ms` | **`3,2`** | `0,041` |
| vaso (cena `5`) | `121` | `110,4` | `44,6` | **`15,7`** | `0,069` |
| nó de toro (cena `28`) | `721` | `410,2` | `641,6` | **`95,5`** | `0,112` |

⭐⭐⭐ **A decomposição fecha com um factor CONSTANTE:** `placa ≈ 2,06 × passos × ns/amostra`
(`2,05` · `2,07` · `2,08`) sobre **`90×`** de tamanho de fita. ⇒ *a avaliação do campo é cerca de
METADE do quadro do dispositivo, e o tecto de «tornar a amostra grátis» é `~2×`.*

⛔⛔ **E esse tecto é o preço BRUTO, não o líquido:** a própria proposta declara que a marcha sobre
uma grelha tem de dar um passo **conservador** (`valor − meia diagonal da célula`), porque a
interpolação trilinear **não é Lipschitz-1** ⇒ **mais passos**. Com o quadro a ser
`passos × custo_por_passo`, uma grelha que faça o passo `2×` mais barato e a contagem `2×` maior
**empata**.

⭐⭐⭐⭐ **⇒ O QUE A TABELA DIZ É QUE A GRANDEZA DOMINANTE SÃO OS PASSOS, não o custo de cada um:**
eles variam `38 → 110 → 410` (**`10,8×`**) enquanto o `ns/amostra` varia `0,041 → 0,112`
(**`2,7×`**) sobre `90×` de fita. ⇒ *se uma grelha for construída, o propósito MEDIDO dela tem de
ser **saltar espaço vazio** (menos passos), e nunca «uma amostra mais barata»* — e é essa a
diferença entre a wave que paga e a que empata.

⚠️ **O que fica por medir antes de a construir:** quanto do orçamento de passos é gasto **longe** da
superfície (o único que um salto de tijolos pode devolver). Sem esse número, o `2×` de tecto acima é
a única fronteira honesta.

### ⛔⛔⛔ E A GRELHA FOI CONSTRUÍDA COMO SONDA E MEDIDA — o «tecto de `~2×`» acima está REFUTADO na peça complexa

Estudo do MagicaCSG (2026-09-23, app corrido sob Wine, formato `.mcsg` lido como texto): ele assa
cada **grupo** de peças numa grade de `20³`–`64³` (o campo `res` de cada `object`) e marcha a
grade; a árvore fica como fonte, e um segundo motor (`F2`, *path tracing* + denoise) desenha o
exacto. ⇒ o que a sonda pergunta é *a mesma composição no NOSSO motor*.

⭐ **Zero motor novo:** a sonda ([`device_probes_w9_grade.rs`](../../crates/ph2d-app-field3d/src/device_probes_w9_grade.rs),
`diag_a_grade_contra_a_arvore`) assa o documento em lote (`Hybrid::eval`, o avaliador da exportação)
sobre a caixa da peça com `8` células de folga, e entrega a grade por um nó
`NodeKind::Sampled` — ⇒ na placa ela entra pela lei que a escultura já shipa
(`ph2d_field_gpu::sculpt`), com a cache por `Arc`. *O que muda entre as duas colunas é o
DOCUMENTO, nunca a marcha.* Qualidade medida contra o traço exacto a `480×270` (CPU, mesma lei);
relógio da placa pela rota do produto (`pinta_matcap`, `1920×1080`, mínimo de 4), `release`,
**`87 %` ociosa**:

| peça | fonte | passo | passos/acerto | **placa** | assar | MB | silhueta trocada | desvio p99 | normal p50/p99 |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| vaso (`5`, 121 l.) | árvore | `1,000` | `59,6` | **`12,96`** | — | — | — | — | — |
| | grade `32` | `0,707` | `234,4` | `14,28` | `0,5 ms` | `0,3` | `500` px | `2,08` cél. | `2,5°`/`60,2°` |
| | grade `64` | `0,707` | `179,7` | `14,16` | `1,6` | `1,1` | `184` | `1,00` | `1,1°`/`42,0°` |
| | grade `128` | `0,707` | `155,6` | `14,19` | `7,4` | `5,6` | `72` | `0,57` | `0,5°`/`36,3°` |
| nó de toro (`28`, 721 l.) | árvore | `1,000` | `224,3` | **`59,46`** | — | — | — | — | — |
| | grade `32` | `0,707` | `2 302` | `6,42` | `8,0` | `0,1` | `12 093` | `3,04` | `40,8°`/`118°` |
| | grade `64` | `0,707` | `795` | `17,98` | `19,6` | `0,3` | `5 116` | `5,56` | `28,3°`/`109°` |
| | grade `128` | `0,707` | `532` | **`20,09`** | `66,2` | `1,0` | `650` | `1,70` | `10,3°`/`52,3°` |

⭐⭐⭐ **Os PASSOS sobem, como a nota acima previa** (`2,4×`–`4×`: o passo cai a `0,707` porque a
interpolação de uma escultura vale `L = √2`, e a trilinear alisa o campo junto da superfície —
⚠️ **corrigido em 2026-09-24:** a coluna dizia `0,841` porque a sonda imprimia `1/√L` em vez do
`safe_march_step`; a marcha sempre andou `0,707`, e os relógios da tabela são os dela) — ⛔⛔ **e mesmo assim o nó
fica `3×` mais barato**: `532` passos a `20,09 ms` contra `224` a `59,46` ⇒ *o passo na grade custa
`~7×` menos que na árvore de `721` linhas*. ⇒ **a leitura «a avaliação do campo é METADE do quadro»
estava errada para a peça complexa**: o factor `2,06` da tabela acima é constante porque o
`ns/amostra` foi DERIVADO do próprio relógio da placa — *uma decomposição que divide o relógio pelos
passos e volta a multiplicá-los não pode dizer quanto do relógio é o campo*. Quem o diz é tirar o
campo, e tirá-lo deu `~85 %`.

⛔⛔ **Mas no vaso a grade NÃO PAGA** (`12,96 → 14,2`, em todas as resoluções): com `121` linhas o
passo na árvore já é barato, e os passos a mais comem o ganho. ⇒ *a grade é alavanca da
COMPLEXIDADE da cena, não da cena típica* — que é exactamente o que o MagicaCSG explora (100 formas
num `object`) e nunca a peça solta.

⛔⛔⛔ **E o preço é a QUALIDADE, e ele é o report do dono sobre o MagicaCSG à letra** (*«algumas peças
parecem de baixa resolução»*): no nó a `128³` a normal erra `10°` na mediana e `52°` no p99 — facetas
que a luz mostra —, e a `64³` (o `res` máximo nos cinco exemplos do autor) o nó está **desfeito**
(`5 116` píxeis de silhueta trocados, normal `28°` na mediana). A causa é geométrica: o tubo do nó
tem poucas células de largura, e *a unidade de resolução é o CONJUNTO, nunca a peça* — o `car.mcsg`
do autor parte-se em **15** objects de `res` diferentes exactamente por isto, e o `robot.mcsg` (1
object) tem peças de `1,3` célula.

⏳ **O que a medição NOMEIA como próximo passo, e não constrói:** a grade como **salto de espaço
vazio** e o traço **exacto junto da superfície** — marchar a grade enquanto o valor é grande,
trocar para a árvore nas últimas amostras e para a normal. Isso ataca as duas colunas de uma vez
(os passos longe ficam `~7×` mais baratos; a silhueta e a normal voltam a ser as exactas), e é a
pergunta do parágrafo anterior (*quanto do orçamento de passos é gasto LONGE da superfície*)
respondida pela construção em vez de por uma sonda. ⚠️ **Ele só vale para peças complexas** — o
vaso diz que numa peça de `~100` linhas não há o que ganhar —, logo o gatilho de assar tem de ser
**medido** (linhas da fita, ou relógio) e não um interruptor.

### ⛔⛔⛔ E O SALTO DE ESPAÇO VAZIO FOI CONSTRUÍDO, MEDIDO e RECUSADO — o que ficou foi o RECORTE

✅ **Smoke do dono APROVADO (2026-09-24)** — cena `=28`, o recorte pela caixa da marcha.

Ordem do dono (2026-09-24, *«se esse é o padrão ouro, então siga»*). Construído inteiro:
[`ph2d_field_gpu::longe`](../../crates/ph2d-field-gpu/src/longe.rs) — a grade é **assada na PLACA**
pelo mesmo `field()` da marcha (um despacho no mesmo encoder, antes dela), mora no armazém das
esculturas depois delas, e o raio salta pelo **limite inferior provado**
`s·f(p) ≥ s·f̃(p) − (√3/2)·h` enquanto ele passa de `h`, e avalia a árvore exacta perto da
superfície (a silhueta, o ponto de paragem e a normal continuam os da árvore).

⭐ **A desigualdade está MEDIDA dos dois lados, e segura:** `0` violações em `65 536` pontos por
cena nas oito cenas, com a grade assada na CPU (`Hybrid::eval`) **e** com a grade e o campo lidos
da PLACA (`parity::compare`, desvio placa-CPU nos nós `≤ 2,4e-7`), e `s·‖∇f‖ ≤ 1,000`. A caixa da
peça **contém** a peça (`0` pontos dentro dela em `131 072` amostras fora da caixa).

⛔⛔ **E ela não compra nada** — duas corridas a `1920×1080` (matcap, a `92 %` e `70 %` ociosa; a
placa partilhada com outra janela do app, logo `±10 %`), com a coluna que a separa do RECORTE:

| cena | árvore ms | só recorte | recorte + grade `64` |
|---|---|---|---|
| `=5`  | `16,4` / `16,0` | `1,31×` / `1,12×` | `1,05×` / `1,07×` |
| `=28` | `103,6` / `103,1` | `1,19×` / `1,16×` | `1,19×` / `1,18×` |
| `=1`  | `10,4` / `9,9` | `1,15×` / `0,95×` | `0,99×` / `0,94×` |
| `=11` | `18,9` / `17,9` | `1,21×` / `1,17×` | `1,07×` / `1,09×` |
| `=26` | `5,5` / `4,9` | `1,26×` / `1,10×` | `1,18×` / `1,05×` |
| `=27` | `14,8` / `15,0` | `1,19×` / `1,13×` | `1,21×` / `1,21×` |
| `=29` | `10,0` / `10,2` | `1,10×` / `1,11×` | `1,03×` / `1,08×` |
| `=30` | `15,5` / `15,2` | `1,04×` / `1,09×` | `1,04×` / `1,02×` |

⇒ **o que o salto poupa, o recorte já tinha poupado.** O custo que sobra mora PERTO da superfície —
os passos finais, a normal, as bordas re-amostradas —, e a sonda da grade SOZINHA (a secção acima,
`3×` no nó) só ganhava porque ali TODO passo ficava barato, a qualidade incluída; com a árvore
exacta perto da superfície, o `~7×` por passo aplica-se a uma minoria dos passos. ⛔ Também não é
afinação: `32`, `64` e `128` células dão o mesmo.

⭐⭐⭐ **E a construção achou um defeito de PARIDADE que não era dela.** A 1.ª medição contra a CPU
dava Δt máximo de `4,0e-1` e `12` pixels de silhueta trocados na cena `=29`, e parecia o salto a
passar da superfície; a coluna que o decidiu foi **«só o recorte»**, que lia **exactamente** os
mesmos números que «recorte + grade» ⇒ a grade estava ILIBADA e o culpado era o PONTO DE PARTIDA do
raio. A CPU recorta pela [`march_clip`](../../crates/ph2d-field-eval/src/bounds_clip.rs) — a caixa
**com `1 %` de margem**, e o doc dela já diz que quem lê o `aabb` cru mede outra região — e o
dispositivo recortava pela crua. Com a mesma porta (`gpu_frame::a_caixa_da_marcha`), o
`o_gbuffer_do_dispositivo_e_o_da_cpu` lê **`0` pixels de silhueta e Δt `p99 ≤ 5,96e-7` em TODAS as
cenas** — contra `8` pixels e `1,8e-4` SEM recorte nenhum. ⚠️ Esse gate corria com `longe: None`,
logo comparava um dispositivo sem recorte com uma CPU com ele; hoje mede a porta do produto, e a
barra do Δt desceu de `1e-3` para **`1e-5`** (um vale: `17×` acima do produto, `18×` abaixo da
regressão mais pequena, que é a caixa crua a `3,7e-4`).

⇒ **SHIPA O RECORTE, e a grade fica RECUSADA** (`LONGE_RES = Some(0)`: a caixa da marcha, sem grade
assada, sem buffer a mais, e o laço decide UMA vez por raio que não há grade — a pergunta por passo
custava `9,47 → 10,96 ms` na cena `=1`). `PH2D_FIELD_LONGE=off` volta ao raio sem recorte e
`PH2D_FIELD_LONGE=<n>` liga a grade para a voltar a medir; as sondas da recusa vivem em
[`device_probes_w9_longe.rs`](../../crates/ph2d-app-field3d/src/device_probes_w9_longe.rs).
Mutação **4 de 5** a sangrar (caixa crua · sem recorte de omissão · cabeçalho trocado · a LEI a ler
a célula no índice errado); a 5.ª — o laço perguntar à grade sem ver a célula — **sobrevive de
propósito**: é só relógio, e o relógio está na tabela.

### ⭐⭐⭐ ONDE OS PASSOS ACONTECEM — e QUANTO A PODA POR REGIÃO COMPRARIA (2026-09-24)

A recusa acima deixou a frase *«o custo mora perto da superfície»*; as duas sondas de
[`device_probes_w9_perto.rs`](../../crates/ph2d-app-field3d/src/device_probes_w9_perto.rs) medem-na
refazendo a marcha raio a raio com a lei do produto (recorte `march_clip`, `safe_march_step`,
orçamento, `Sharpness::for_frame`) a `1920×1080`.

**Onde** (`diag_onde_os_passos_acontecem`): `29`–`67 %` dos passos são de raios que FALHAM (roçam a
silhueta e seguem); no nó (`=28`) `72 %` acontecem a menos de `4` px da superfície; a normal pesa
`4`–`23 %`. ⭐ **E a divergência da placa pesa pouco:** em blocos de `32` (`8×4`, a largura desta
placa) a eficiência é `73`–`87 %` ⇒ no máximo `~1,3×` a ganhar por reordenar raios.

⚠️ **O número que muda a pergunta:** o nó faz `~21 M` avaliações de uma fita de `721` linhas em
`~59 ms` ⇒ `~45` instruções da placa por linha da fita. *O tecto não é o número de passos, é o
custo de CADA avaliação* — e ele cresce com o comprimento da fita.

**Quanto a poda compraria** (`diag_quanto_a_poda_por_regiao_compraria`, [`ph2d_field_eval::poda`],
a poda por intervalos da própria `fidget` — Keeter 2020): cada avaliação cai num pedaço
`(quadrado de ecrã, fatia de profundidade)`, a caixa exacta do que ali se avaliou vai à poda, e as
instruções que sobram pesam-se pelas avaliações do pedaço. *Razão = instruções médias por avaliação
÷ fita inteira*; `0` numa caixa que o intervalo prova vazia (coluna à parte):

| cena | fita | `64 px`×1 | `16 px`×1 | `32 px`×16 | **`16 px`×16** | vazias (`16×16`) | poda na CPU (`16×16`) |
|---|---:|---:|---:|---:|---:|---:|---:|
| `=5` vaso | `125` | `0,965` | `0,950` | `0,864` | **`0,821`** | `6,0 %` | `12 ms` |
| `=28` nó | `725` | `0,577` | `0,474` | `0,325` | **`0,279`** | `6,3 %` | `50 ms` |
| `=1` cilindros | `94` | `0,900` | `0,877` | `0,273` | **`0,247`** | `48,0 %` | `21 ms` |
| `=11` lote | `394` | `0,769` | `0,723` | `0,423` | **`0,363`** | `17,8 %` | `31 ms` |
| `=26` triângulo | `129` | `0,505` | `0,384` | `0,221` | **`0,183`** | `28,5 %` | `3 ms` |
| `=27` polígono | `752` | `0,846` | `0,762` | `0,596` | **`0,517`** | `5,1 %` | `41 ms` |
| `=29` rosca | `164` | `0,906` | `0,843` | `0,492` | **`0,400`** | `2,1 %` | `9 ms` |
| `=30` curvas | `309` | `0,756` | `0,601` | `0,442` | **`0,366`** | `11,8 %` | `15 ms` |

⭐⭐ **As FATIAS DE PROFUNDIDADE são metade da poda** (`16 px` sem fatias fica em `0,38`–`0,95`) —
a mesma lição que a W56e mediu na CPU. ⇒ a poda leva o nó a **`~3,6×` menos instruções** por
avaliação, e seis das oito cenas a um terço ou menos. ⛔ O vaso é a excepção honesta (`0,82`).

⚠️ **O que falta saber antes de construir, e é a medição seguinte:** (1) a poda na CPU custa até
`50 ms` por quadro — ela tem de correr **na placa** (intervalos em paralelo, como no artigo); (2) uma
fita por pedaço não se compila por pedaço, logo o dispositivo precisa de um **INTERPRETADOR** de
fita — exactamente a recusa do `ph2d_field_eval::wgsl` que ficou *«por medir»*. ⇒ *o ganho é
`razão × (custo por instrução interpretada ÷ custo por instrução compilada)`*, e é esse segundo
factor que decide a wave.

### ⛔⛔⛔ E A MEDIÇÃO SEGUINTE DECIDIU — CONTRA A PODA, e achou onde o tempo morava (2026-09-24)

As sondas de dispositivo vivem em
[`device_probes_w9_placa.rs`](../../crates/ph2d-app-field3d/src/device_probes_w9_placa.rs); a
pergunta andou por cinco degraus, e cada um fechou uma porta.

**(1) O ORÁCULO do estado da arte, CORRIDO** (§0.9): o renderizador de voxels do próprio Keeter
(`fidget-wgpu` 0.5, MPL-2.0 ⇒ corrido FORA da árvore, ligado e nunca copiado; as cenas saem pela
`diag_exporta_as_cenas_para_o_oraculo`), `1920×1080×1080`, tempo de GPU por carimbo:

| cena | nosso quadro | `fidget-wgpu` |
|---|---:|---:|
| `=28` nó | `59` ms | **`9 414`** |
| `=5` vaso | `13` | `566` |
| `=29` rosca | `9` | `472` |
| `=11` lote | `15` | `163` |
| `=30` curvas | `15` | `124` |
| `=27` polígono | `13` | `50` |
| `=1` cilindros | `9` | `6,7` |
| `=26` triângulo | `4,5` | `4,4` |

⛔⛔ **O método dele só empata nas uniões simples**; nas nossas formas por fórmula os intervalos saem
largos e ele avalia quase todos os voxels. ⇒ **portar o renderizador do estado da arte está
RECUSADO por medição** — a marcha por esferas é a certa para o nosso catálogo.

**(2) Quantas fitas podadas um quadro pede:** o nó **`4 218`** (`2 494` fazem `90 %`) a
`16 px × 16` fatias, e `136` mesmo a `64 px` sem fatias ⇒ compilar uma a uma está fora (`6`–`49 ms`
cada). ⇒ só com INTERPRETADOR.

**(3) O interpretador, MEDIDO** ([`ph2d_field_eval::interp`] +
[`ph2d_field_gpu::interp_bench`]; os casos do `switch` saem do MESMO emissor, e a resposta bate a da
compilada a `≤ 5e-4`): **`15×`–`35×` por instrução**. A poda compra no máximo `3,6×` ⇒ ⛔⛔ **a poda por
região está RECUSADA** — o interpretador come o ganho dez vezes.

**(4) E o número que mudou a pergunta:** a fita INTEIRA do nó, compilada, avalia `2 M` pontos em
**`0,22 ms`**; um laço de `10` passos mais a normal, `2,7 ms`. ⇒ *a avaliação é quase de graça.* Não
é a estrutura do kernel (a normal escrita à parte ou num laço custa o mesmo, logo NÃO é o tamanho
do código), nem a CPU do pedido (`≤ 0,4 ms`), nem recompilação (`0` durante a medida).

**(5) A MESMA marcha num kernel MAGRO** (`diag_a_marcha_magra_contra_o_produto`: os raios do produto,
o recorte, o passo, o orçamento, os limiares e a normal, e nada mais) custava **`18×`–`31×` MENOS**
do que o quadro que a hospedava. ⭐⭐⭐⭐ **O matcap marchava dentro do `centro_e_luz`** — a entrada do
RENDER, com o chão, as lâmpadas, a visibilidade e o ricochete, que o matcap nunca lê e que o
compilador da placa paga em registos. ⇒ entrada própria **`centro_so`** (a mesma `marcha`, só o
centro) e o buffer de luz no mínimo:

| cena | antes | **depois** | ganho |
|---|---:|---:|---:|
| `=28` nó | `86,3` ms | **`10,0`** | `8,6×` |
| `=5` vaso | `11,8` | **`2,2`** | `5,3×` |
| `=11` lote | `16,9` | **`3,2`** | `5,4×` |
| `=30` curvas | `14,0` | **`3,3`** | `4,3×` |
| `=27` polígono | `11,8` | **`3,0`** | `4,0×` |
| `=1` cilindros | `8,7` | **`2,0`** | `4,5×` |
| `=29` rosca | `8,3` | **`2,1`** | `4,0×` |
| `=26` triângulo | `3,9` | **`1,5`** | `2,6×` |

⚠️ **A imagem é a mesma ao bit** — o `o_matcap_e_o_mesmo_nos_dois_motores` compara o matcap da
placa (agora pela entrada magra) com a lei da CPU sobre o G-buffer da entrada pesada, e passa. ⭐ E
como nenhuma paridade pode ver a escolha, ela tem gate próprio (`o_matcap_marcha_no_kernel_magro`,
com o CONTROLO de que o G-buffer do Render continua a compilar o `centro_e_luz`); mutação **2 de 2**.

⏳ **O que fica, com endereço:** o quadro magro ainda custa `3,6×`–`6,7×` a marcha magra — a
passagem das BORDAS, a pintura e a leitura de `8 MB`, mais os buffers criados de novo em cada
quadro. ⏳ E o modo RENDER (`centro_e_luz`) tem a mesma forma de defeito: ali o chão e as lâmpadas
são LIDOS, logo a cura é partir o kernel por responsabilidade, não apagá-los.

### ⭐⭐⭐⭐ A BORDA QUE ESPERAVA PELAS VIZINHAS, a LUZ num kernel próprio, e a OCLUSÃO que o Render paga a mexer (2026-09-24)

As sondas vivem em [`device_probes_w9_placa.rs`](../../crates/ph2d-app-field3d/src/device_probes_w9_placa.rs)
e [`device_probes_w9_ceu.rs`](../../crates/ph2d-app-field3d/src/device_probes_w9_ceu.rs). Todas as
tabelas são `1920×1080`, mínimo de 5, pela porta do produto.

**(1) Partido o quadro magro do matcap (`diag_as_pecas_do_quadro_magro`)**, a segunda passagem da
silhueta era a peça grande: no nó `7,38` dos `11,86 ms`. ⭐ **A causa era DIVERGÊNCIA, não trabalho:**
o `bordas` corria na imagem inteira e marchava as quatro sub-amostras DENTRO da thread do pixel de
borda, logo um punhado de bordas por warp marchava quatro vezes em série com o resto parado. ⇒ o
`bordas` só DETECTA e escreve a lista, e o `bordas_marcha` re-amostra com **uma thread por
(borda, sub-amostra)**, lendo a contagem no dispositivo (laço em passos do tamanho da imagem, logo
cobre também uma lista maior que `w·h/4`). Zero ida-e-volta nova.

| cena | quadro antes | **depois** | a borda antes | **depois** |
|---|---:|---:|---:|---:|
| `=28` nó | `11,86` | **`6,45`** | `7,38` | **`1,94`** |
| `=5` vaso | `2,50` | **`2,02`** | `0,93` | `0,40` |
| `=11` lote | `3,69` | **`2,68`** | `1,77` | `0,69` |
| `=30` curvas | `3,51` | **`2,68`** | `1,61` | `0,77` |
| `=27` polígono | `3,36` | **`2,70`** | `1,34` | `0,62` |

⚠️ O que sobra numa cena simples é um custo FIXO de `~1 ms` (a marcha magra do triângulo são
`0,22 ms`, o quadro sem borda `1,24`): buffers criados por quadro, duas travessias, a imagem de
`8 MB` (`0,34 ms` medidos). ⏳ Nomeado, não atacado.

**(2) A LUZ do Render num kernel PRÓPRIO** — `centro_so` e depois `luz_so` em vez do
`centro_e_luz` (a mesma lei que curou o matcap). `PH2D_FIELD_LUZ_SEPARADA=0` bissecta. Ganho
**modesto e medido** (máquina a `load 30`, logo a forma e não o absoluto): nó `69,3 → 54,9`, curvas
`27,8 → 16,5`, as outras `0`–`20 %`. O G-buffer é o mesmo ao bit (a paridade de materiais e as do
chão passam).

**(3) ⛔⛔ A NOTA DA OCLUSÃO ESTAVA ERRADA — o Render a MEXER paga os `48` cones.** O doc do
`OCCLUSION_PASSES` dizia *«o quadro de movimento não paga nada disto: o dispositivo só toma o quadro
assente»*; o `takes_the_frame` deixou de o exigir e o `ao_rays` do `MarchSetup` não lê a bandeira
`assente` (só o ricochete a lê). Medido (`diag_o_render_contra_o_matcap` + a oclusão desligada à
mão): o quadro de movimento do nó vai de `54,9` a `11,6 ms` e as outras de `12`–`17` a `5`–`7` ⇒
**a oclusão é `60`–`80 %` do Render a mexer**, que é o que põe o Render `5`–`10×` acima do matcap.
Nota corrigida no sítio (§0.0).

**(4) A OCLUSÃO NUMA GRADE ASSADA — o *Distance Field AO* dos motores de jogo — construída como
SONDA** (`Sonda::ceu_na_grade`, `Longe::so_ceu`: o raio primário continua exacto, só os cones marcham
a grade). ⛔⛔ **A 1.ª medição saiu PRETA** (média `−0,59` no triângulo, a PIORAR com a grade mais
fina), e a causa eram DUAS coisas minhas: fora da caixa o cone lia a *distância à caixa*, que é um
limite inferior (certo para saltar, errado para a penumbra, que lê `d/t` como «passa rente»), e a
grade era assada na caixa JUSTA da peça enquanto os cones andam até à ESFERA dela. Com a grade na
caixa da esfera o canal do céu CONVERGE (`p99` `0,020 → 0,013` de `128` a `512` no triângulo, `0,020`
no nó a `512`) — a ordem do erro que a própria lei de `48` cones tem contra `1 024`.

| cena | exacta | grade `256` (assada por quadro) | px `> 8 B` | máx `B` |
|---|---:|---:|---:|---:|
| `=28` nó | `51,77` | **`18,10`** | `357` | `38` |
| `=30` curvas | `14,65` | **`9,08`** | `10 988` | `43` |
| `=5` vaso | `15,00` | `11,61` | `3` | `9` |
| `=11` lote | `14,84` | `8,97` | `67` | `73` |
| `=29` rosca | `9,46` | `7,26` | `3 499` | `110` |
| `=1` cilindros | `9,99` | ⛔ `13,01` | `0` | `3` |
| `=26` triângulo | `4,86` | ⛔ `5,00` | `0` | `8` |

⚠️ Assada **em todo quadro**: numa cena simples assar custa mais do que poupa, e a grade é
independente da câmara — quem a torna ganho em toda cena é **guardá-la entre quadros do mesmo
documento** (o orbitar), que ainda não existe. ⚠️ E ela **perde paredes mais finas que uma célula**
(a gaiola, a rosca, as curvas não convergem no máximo): a luz passa por elas.

⛔⛔ **O HÍBRIDO — a árvore perto da superfície, a grade longe — foi construído, MEDIDO e RECUSADO:**
exacto na imagem (máx `≤ 3 B` com `1` célula) e **mais LENTO que a árvore sozinha** a `1`, `2` e `4`
células (nó `51,5 → 52,8`, cilindros `10,2 → 17,4`). *Os cones passam quase todos os passos RENTE à
superfície, que é exactamente onde a grade erra e onde ela poupava — o ganho e o erro moram no mesmo
sítio.*

⏳ **O que fica por decidir, com o preço de cada saída:** a grade pura com cache por documento
(`2,9×` no nó, erro localizado em paredes finas, e a CPU de referência teria de assar a mesma grade
para a paridade) · a oclusão a MEIA resolução com reconstrução guiada pela normal (a alavanca que o
doc do `OCCLUSION_PASSES` já nomeia, `4×` em toda cena, com halo na descontinuidade) · ou a oclusão
fora do quadro de MOVIMENTO (a lei W73 *grosso a mexer, nítido ao assentar*: nó `→ 11,6 ms`, com o
céu a «acender» ao largar).

### ⛔⛔⛔ A LUZ ENCOSTADA À PEÇA DESENHAVA ANÉIS NO CHÃO — dois defeitos de LEI, curados nos dois motores (2026-09-24)

Report do dono, com foto da cena `=28` e a lâmpada dentro de um nó: *«ao aproximar a luz do objeto
resultados muito ruins de render»* — riscas em leque, curvas duras e blocos claros no chão. A sonda
[`diag_os_canais_do_chao_com_a_luz_encostada`](../../crates/ph2d-app-field3d/src/device_probes_w9_ceu.rs)
grava os canais do G-buffer como imagens, e partiu-o em TRÊS coisas diferentes:

1. **Os anéis do CÉU do chão** (as curvas duras). A lei do `ground_sky` corta cada amostra na
   cerca da bola, o que só é contínuo num campo EXACTO — e o nó de toro (fórmula) SUBESTIMA a
   distância: o termo valia `> 0` logo dentro e `0` logo fora, um degrau por amostra, **seis anéis**.
   ⇒ a distância do termo é `max(campo, distância à bola)` (os dois são limites inferiores, o maior
   também, e vale `α·h` exactamente na cerca). Num campo exacto nada muda.
2. **Os anéis da SOMBRA da luz** (as riscas). A marcha parava no primeiro passo que passava a luz,
   logo a última amostra ficava num sítio que depende da FASE dos passos — e com a luz a `~0,025` de
   um tubo é ali que o raio passa rente: um anel por salto de fase. ⇒ o último passo encurta-se até
   à cerca e é amostrado lá (`march_visibility::ate_a_cerca`, gémea da `visivel` do WGSL). ⛔ **O
   estimador melhorado de Quilez (a aproximação mais rente entre duas amostras) foi construído e NÃO
   cura** — medido na mesma fixtura, as riscas ficam. ⚠️ O CONE da oclusão não muda: a lei dele foi
   calibrada contra `1 024` direcções com a paragem de sempre.
3. ⏳ **Os BLOCOS claros do quadro assente são a luz que a peça devolve ao chão** (`docs/Render3d/09`),
   e ficam ABERTOS: a grelha `32²` sobre `6` raios tem células de `~0,38` — maiores do que a altura
   da luz ao chão —, e foi calibrada com a luz LONGE (pico `33/255`). Com a luz encostada o campo tem
   um pico estreito que ela não resolve. A cura nomeada no `09` §6 (o kernel no dispositivo) é a que
   compra a resolução.

Gates novos em [`luz_encostada_gates.rs`](../../crates/ph2d-field-render/src/tests/luz_encostada_gates.rs),
os dois medindo a **continuidade** (o maior salto entre vizinhos numa linha de `3 000` pontos do chão)
— *nenhum gate media continuidade, só valores num ponto, e é por isso que nenhum via os anéis*. A
sombra leva o CONTROLO dentro (a mesma marcha pela porta do cone, que ainda pára no passo: `0,4335`
contra `0,0052` da lei nova) — ⚠️ com a luz no CENTRO do buraco as duas leis saltam `0,004` e a
fixtura não conteria o fenómeno. Mutação **2 de 2**, cada uma a sangrar só o seu gate. As paridades
de sombra, chão e luz na placa passam (`44/44`).

⏳ **E os travões ao girar no Render têm mecanismo medido e ficam por curar:** quando a mão hesita um
quadro, o app pede o quadro ASSENTE, que a placa não cancela — no nó são `240 ms` (o ricochete sozinho
`158`), nas outras `16`–`38` — e o quadro de movimento seguinte espera por ele.

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
