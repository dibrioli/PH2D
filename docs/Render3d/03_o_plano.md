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
| **`W7d`** (profundidade de campo) | proposta **FORA por omissão**, decisão do dono |

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
⚠️ A profundidade de campo está proposta **FORA por omissão** — decisão do dono.

⭐⭐⭐ **O que fechou:** o brilho existe e corre no **dispositivo** ([`12` §11](12_o_acabamento.md)) e
a borda **deixou de ferver** ([`12` §12](12_o_acabamento.md)) — esta última sem uma lei nova: a
segunda passagem da silhueta já existia nos dois motores e estava desligada no quadro de movimento
por uma tabela de CPU medida a `640×360`, antes de o quadro inteiro ir para a placa. No dispositivo
ela custa `1,03×`–`1,09×`, e sem ela a silhueta que a mão arrasta não tem **um único** pixel de
cobertura parcial. ⇒ **só a `W7d` (DOF) fica**, e ela está proposta FORA.

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

### ⛔⛔⛔ A `W9` COMEÇA COM UM VERMELHO JÁ MEDIDO — e ele é a primeira coisa a resolver

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
