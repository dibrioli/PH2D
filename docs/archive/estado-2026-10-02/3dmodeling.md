# ARQUIVO — CLAUDE.md (história, 485 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`CLAUDE.md`](../../../CLAUDE.md) em 2026-10-02, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-1046,1532-1543` do original.
>
> ⚠️ **A única alteração ao corpo:** 42 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

- **3D Modeling (campo implícito)** — modelador **SDF** editável para sempre
  ([ADR-0161](../../architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md)):
  `ph2d-field` (documento + primitivas + modificadores) · `-field-eval` (avaliador híbrido, bordo
  da peça) · `-field-ecs` (a árvore de modelagem **é** a hierarquia da cena) · `-field-mesh`
  (Surface Nets) · `-field-render` (traçado) · `ph2d-panel-model3d`. Abre pelo pill **MODEL**.
  ⚠️ **A hierarquia da cena É o documento** — o `FieldDoc` é **cozido** dela a cada quadro, e é por
  isso que o undo, o olho, o cadeado e o reparentar da casa valem aqui sem código próprio.
  ⚠️ **Só uma OPERAÇÃO pode ter filhos**, e a lei impõe-se na **derivação** (`promote_leaf_hosts`),
  nunca em cada gesto. ⚠️ **O painel oferece EXATAMENTE o que o gesto faz** (W34) — a lei está em
  [`reach_tests.rs`](../../../crates/ph2d-app-field3d/src/reach_tests.rs), e ela apanha os dois
  lados (botão mudo · gesto inalcançável).
  ⚠️ **A peça ATRAVESSA o arquivo** (W35) — ela é uma árvore de entidades e o `ProjectState` é o
  mundo inteiro, então o `PROJECT_SCHEMA` **não se mexe**; a nota que dizia o contrário era velha.
  ⚠️ **Tomar o canvas LIBERTA quem o tinha** (W40+W42): pegar noutra ferramenta fecha o MODEL **e
  desarma o módulo** — e a **vista** (câmera · prato parado · verbo e referencial do gizmo ·
  isolamento) sobrevive ao fecho (W43), enquanto o cache do quadro não. ⛔ Um campo novo no `Smoke`
  é **erro de compilação** em `field3d_view::View::of` até alguém dizer se é vista ou cache.
  ⭐ **A CÂMERA é alcançável** (W47–W52): seis **vistas nomeadas** (`Numpad1/3/7` + `Ctrl` para a
  oposta) com botões que dizem o atalho, o **gizmo de navegação** por bolas de eixo — pesquisado a
  pedido do Enio, ⛔ o *ViewCube* da Autodesk está **patenteado** (US 7 782 319, expira 2029) e o
  próprio paper deles mediu que o ganho está no **arrasto**, não no clique —, que se **desloca para
  fugir à moldura** (`panel_ops::panel_rects`, a fuga mais barata), e a **viagem** animada entre
  vistas. ⚠️ **`Role::Viewpoint` SOBREVIVE ao `reduced_motion`**, sozinho entre todos os papéis, por
  decisão do Enio com a alternativa na mão: *aqui o CORTE é pior do que o movimento*.
  ⭐⭐ **O DESENHO VIRA PEÇA, e continua a ser a FONTE** (W53–W55): `+ Extrude` / `+ Revolve` cozem o
  contorno escolhido no editor vetorial (o fluxo do MoI, com a caneta que a casa já tem) — o motor
  existia **desde a W3** e nenhum botão o alcançava. O `FieldProfileSource { path, level }` mantém o
  vínculo **vivo**: editar a curva remodela a peça, e a linha **Resolution** (1..16) afina a
  conversão. ⚠️ **Sem cache, de propósito** — recozer custa 7 µs e comparar 0,2 µs contra um quadro
  de 16,7 ms, e um resumo guardado seria estado derivado a envenenar o undo.
  ⚠️ **A régua da suavidade é a NORMAL, não a silhueta** (W54): a polilinha erra **0,079 %** da peça
  (invisível) e a normal salta **6,43°** — é isso que a luz mostra. A tolerância é `1e-4` pelo joelho
  medido, e ⛔ a tabela de 2026-08-19 estava **desmentida por 2,4×**.
  ⭐⭐ **O traçado é 2,5× mais rápido e o vínculo é ALCANÇÁVEL** (W56e–W58d): a marcha especializa a
  árvore por **ladrilho × fatia de profundidade** (`167 → 66 ms` a 168 arestas) e o passo dela sai do
  **documento** — auditado construtor a construtor, **só o arredondamento exacto infla** (`√2`), e o
  `Taper` **desce** a `0,844`. O vínculo desenho→peça vê-se na Hierarquia (selo `LNK`) e tem gesto
  (`Unlink` / `Link Drawing`), e a **selecção múltipla nasce no canvas** (`Shift`+clique alterna ·
  `Shift`+arrasto **soma**, apanhando também **o que está tapado** — as formas nascem empilhadas no
  alvo da câmera). ⚠️ **Um desenho com contorno interior já virava peça com FURO** — a composição do
  `VecPath` exprimia-o desde a v6 do formato, e o que faltava era o gate.
  ⭐⭐⭐ **A jornada de 24-26/08 (W59–W80)** — a **exportação caiu de 8 min 17 s para 6,4 s** (77×, arquivo
  idêntico) e **saiu da thread que desenha**, porque declarar o congelamento curava a mensagem e não
  o congelamento (a 12 s o KDE dá a janela por morta e oferece forçar o encerramento); a Hierarquia diz
  qual linha está **isolada** (selo `ISO`, que ganha do `LNK` por ser estado da VISTA) e a exportação
  diz **onde** a peça está; o `Mirror` passa a ter **três eixos**; duas formas escolhidas viram **duas
  peças**, cada uma ligada ao seu desenho; e uma escultura que perdeu o ficheiro tem **`Relink
  Sculpture…`**. ⭐⭐ **O custo de MOVIMENTO virou CONSTANTE (~53 ms em qualquer nível)** — o contorno
  também engrossa enquanto a mão mexe —, e por isso o teto de `Resolution` fica em **64**, medido com o
  relógio certo (o quadro **assente**, não o de movimento). ⚠️ **E o passo da marcha estava ERRADO**:
  arredondamentos exactos **encadeados** compõem o factor, e a cena 1 do smoke marchava acima do seguro
  desde que existe. Mecanismo, tabelas e provas de mutação: [doc 06 §69–§81](../../3DModeling/06_resultados_cena_e_gizmo.md)
  + [handoff de 26/08](../../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_2026-08-26.md).
  ⭐⭐ **O catálogo de formas FECHOU** (W100–W103): **16 entradas** numa **paleta com busca** (`A` ou
  *+ Add shape…*), agrupadas por família — a fileira de chips cortava em `MAX_MODES = 8` e já tinha 8.
  O `Primitive` tem **14** famílias, e cada linha do catálogo carrega o **próprio construtor**
  (⛔ as quatro constantes `SHAPES.len() − N` morreram: acrescentar no fim fazia o botão *Extrude*
  abrir o diálogo de escultura, **sem erro nenhum**). ⭐⭐⭐ **E o filete alcança TODA aresta de toda
  forma** (W104): `0,0 %` da superfície sobre um vinco com o filete a metade do limite, nas dez formas
  que o têm — medido por uma sonda que **acha** as arestas pela variação da normal, e não por uma lista
  escrita à mão. ⚠️ Antes disso o `round` do **cone** e do **prisma** era **inerte** (`+0,0 %` de
  volume, campo bit a bit igual) e o da **cunha** fazia a peça **crescer 41 %**; o **arco de toro** não
  tinha controle de filete nenhum. `FIELD_DOC_VERSION` **4 → 10**. Cena **`=11`**.
  ⭐⭐ **E o catálogo tem 68 entradas sobre 58 primitivas** (07/09, W131–W135: *Triangle*, *Polygon(N)*
  com os vértices **arrastáveis no canvas**, *Torus Knot* `(p,q)`, *Thread* e *Knurled Grip*), com a
  fila de formas por fórmula a fechar de 10 para **6** ([doc 09](../../3DModeling/09_plano_das_dez_que_faltam.md)).
  ⛔ **Conte os três no código**, nunca aqui: `PrimitiveKind::ALL` · `grep -c 'key: "panel.model3d.add'`
  · as `CENAS` do smoke. `FIELD_DOC_VERSION` **17 → 18** — ✅ **e o `collision-surface.sh` JÁ o vê**
  (esta nota dizia o contrário e envelheceu: o `f065b17bb`, de 13/09, acrescentou-o ao mapa, que hoje
  imprime `FIELD_DOC_VERSION … (base: …)` como os outros quatro). Cenas **`=26`..`=29`**.
  ⚠️ **O `Thread` NÃO é a `Helix` com um cilindro à volta** — a mola mede a distância a uma CURVA, a
  rosca é um **perfil varrido por movimento de parafuso**, e é isso que fecha o factor da tangente em
  forma fechada. ⏳ **ABERTO:** o arranque da rosca fica afiado (cura medida e **recusada** — a saída
  é um chanfro de entrada). ✅ **As outras duas desta lista FECHARAM em 10/09** — a `sd_helix` que
  engordava o tubo `1/c` (curada na W140, tabela no doc-comment de `ops_spiral.rs`) e o corte do
  doc 06 — *audite a lista contra o CÓDIGO antes de pegar um item dela: ela manda reconstruir
  trabalho já pago, que é o defeito de que este §5 avisa sobre si mesmo*.
  [Handoff de 07/09](../../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_2026-09-07.md) (⚠️ o §8
  tem **seis** coisas que uma leitura rápida do diff entende ao contrário, e o §9 as **seis** premissas
  que a medição derrubou).
  ⭐⭐⭐ **E a AVALIAÇÃO PONTO A PONTO deixou de ser o tecto (W147, 10/09):** o `Field::at` passou de
  interpretador da `fidget` a **fita `f64` achatada** com o gradiente a mandar as seis amostras numa
  passagem só — **bit-a-bit a mesma resposta** (gate sobre o valor e sobre o gradiente), `~3,6×` no valor
  e `~6,3×` no gradiente. ⛔⛔ **E a cura que este §5 prescrevia — o gradiente ANALÍTICO da `fidget` —
  está RECUSADA com número:** sobre pontos POSTOS numa aresta ela discorda `1,876e-1` contra uma folga
  de `2,0e-2` (**`9,4×`**), porque num vinco a derivada não existe e a diferença central e a analítica
  são **grandezas diferentes** — passar a `f64` não cura. ⚠️⚠️ **A 1.ª medição disse o CONTRÁRIO**
  (`2,174e-6`): um vinco é uma **superfície**, e uma grelha nunca lá cai — *os pontos do vinco PÕEM-SE,
  não se procuram*. ⭐⭐⭐ **E o tecto de verdade não era o algoritmo, era o PERFIL DE BUILD:** as crates
  do campo nunca tinham entrado na lista `[profile.dev.package.*]` `opt-level = 2` do `Cargo.toml` da
  raiz — a mesma lista, com a mesma justificação escrita, que já existia para o Painter e o áudio.
  Quatro linhas: a suíte das 3 crates do campo **`372,3 s → 57,1 s`** (`6,5×`), o teste mais longo
  `307 → 43 s`, o `field3d` do shell `22,8 → 4,1 s`. ⚠️ E a varredura em lote que quase foi deitada
  fora por dar `5 %` passou a valer `14 %` medida no perfil certo — *a mesma cura mede-se cinco vezes
  menor no perfil errado*. ⛔ **Os «14 ciclos em série» do censo são RECUSA MEDIDA** (o corredor já
  satura os núcleos). ⭐⭐ **E o [doc 06](../../3DModeling/06_resultados_cena_e_gizmo.md) virou ROTEADOR**
  — `901 KB → 91 KB`, história **verbatim** em
  [`docs/archive/3dmodeling-06-2026-09-10/`](../3dmodeling-06-2026-09-10/06_resultados_cena_e_gizmo.md)
  com `sha256` e as 144 secções indexadas por §. ⚠️ **A configuração de perfil é o ÓPTIMO MEDIDO, não a
  primeira que funcionou:** `opt-level = 3` não dá diferença real, juntar mais três crates **parte um
  teste**, e tirar a `fidget` custa `21 %`. Cenas **`=30`..`=32`**
  ([handoff de 10/09](../../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_2026-09-10.md)).
  ⭐⭐ **O modo Render tem CHÃO, e ele NÃO se desenha** (16/09, `W4` do render): a peça deixa de
  flutuar e pousa num **chão invisível** que não entra na marcha — só recebe a sombra das lâmpadas e o
  escurecimento de contacto do céu (decisão do dono, com as quatro saídas na mesa). Custa **`+0,43 ms`
  no dispositivo** a `1080p`, contra `+86,3 ms` da CPU **pela mesma resposta**. ⛔⛔ **A oclusão do
  chão NÃO são os cones da peça** — `48` cones num recetor **plano** desenham **ANÉIS** (`17` extremos
  contra `1` da referência convergida) ⇒ lei própria por campo de distância, ajustada contra `2 048`
  cones; e a cerca da penumbra do chão é **alargada** por `dist/HARDNESS`, porque a bola simples a
  cortava numa **elipse dura**. ⚠️ A altura lê-se **uma vez**, quando o Render liga (levantar a peça
  afasta a sombra — é isso que a régua pede), e a **caixa** da peça não serve para a pousar (num
  cilindro inclinado ela desce `0,02` abaixo). No mesmo dia, o **arco** do perfil passou a ter a barra
  que está escrita (`ERRO_DO_QUARTO`, pico do desvio em `t = (3−√3)/6`) e o preview deixou de trocar
  arcos por polilinha ao engrossar.
  [chão](../../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_O_CHAO_2026-09-16.md) ·
  [arco](../../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_O_ARCO_2026-09-16.md)
  ⭐⭐⭐ **E a LUZ INDIRECTA são SONDAS desde 17/09** ([`ph2d_field_render::probes`](../../../crates/ph2d-field-render/src/probes.rs),
  [`08` §14](../../Render3d/08_a_luz_indirecta.md)): a recolha por pixel com direcções fixas é uma soma de
  **projecções duras** da peça (o *«reflexo mal feito»* do dono, com a fita direcção a direcção no
  cabeçalho do módulo) e nenhuma quantidade dela cura isso — ela fica só como referência convergida dos
  gates. Paridade `100,000 %` nos dois motores; ⚠️ a régua que separa as duas leis é a
  `banda::estrutura` (média frequência), **não** os terraços a um pixel.
  ⭐⭐⭐ **E A LUZ ATRAVESSA A PEÇA desde 17/09** (ordem do dono, [`10`](../../Render3d/10_a_luz_que_atravessa_a_peca.md);
  cena **`=33`**): a subsuperfície do OpenPBR, e são **DOIS caminhos e não um grau** — a **parede
  fina** (a folha, que acende com a luz ATRÁS: `mx_translucent` nega a normal e mais nada) e a
  **maciça** (o jade, o perfil de Burley integrado sobre a curvatura). ⭐⭐ **A lei maciça é a que esta
  casa JÁ tinha escrita** — o `sss.rs` do esculpir integra **o mesmo integral**, com a mesma corda e a
  mesma normalização; muda o perfil publicado e o facto de a nossa estar pré-integrada. ⭐ E ela
  depende **só** do ângulo e do quociente `mfp/raio` (medido `0,00e+00` sobre `16×`), que é o eixo
  adimensional que aquela tabela já declara. ⭐⭐⭐ **A CURVATURA sai do CAMPO e não de `fwidth`**, e o
  motivo é estrutural: o `fwidth` é por quad de `2×2` e a CPU é por pixel ⇒ a paridade de `100,000 %`
  cairia. Ela sai da **mesma soma** do estêncil da normal (`∇²f` dos mesmos deslocamentos), e falta-lhe
  **uma** amostra — a do centro. ⛔⛔ **E o passo dela NÃO é o da normal:** a 1.ª redacção dizia que
  sim e o gate reprovou em voz alta (uma face plana lia curvatura `1,49`) — uma segunda diferença
  divide por `ε²`, logo o óptimo é `~ulp^{1/4}`, medido em **`0,0064 × o tamanho da peça`** com o vale
  no mesmo sítio em três raios. ⛔ **O caminho maciço tem gate PRÓPRIO, e o porquê está medido:** a
  curvatura implícita de cada amostra do oráculo espalha-se de `0,02` a `54` numa esfera de curvatura
  `1` — e **não é a tesselação** (uma esfera `256×128` nossa dá a mesma) ⇒ um MÁXIMO ali mede o
  estimador DELE. O gate afirma a mediana (`5e-4`), os quartis e que o mínimo em `κ = 1` é **AGUDO**
  (`100×` pior a `κ = 0,5` ou `2`). ⚠️ **O oráculo ganhou uma LUZ DE TRÁS** — sem ela a fixture não
  contém o fenómeno que a parede fina existe para produzir. Paridade **`100,000 %`** nos dois
  caminhos. ⚠️ **`PROJECT_SCHEMA` 144 → 145** e o `MAX_ROWS` do painel **79 → 85** — conte o DELTA;
  a alternativa era baixar o `MAX_POLYGON_VERTICES`, e *um tecto de REGISTO a mandar num tecto de
  FORMA é o caminho lento a definir o rápido*. ⛔⛔ **E DUAS premissas escritas em três sítios
  morreram** (*«o material FECHOU: são as 15 entradas do OpenPBR»* · *«este tecto deixa de crescer por
  material»*): elas fecharam contra a **fatia** construída e não contra o modelo, que tem `41`
  entradas. ⏳ **ABERTO:** a **espessura** como entrada (nenhum dos dois caminhos da referência a lê, e
  num campo de distância ela MEDE-SE — é o candidato a superar, e fica fora por não ter oráculo) · o
  `Thin Walled` pintar-se como caixa e não como escolha · a cena não pôr a luz atrás sozinha · **e o
  relógio desta wave não foi medido** (a máquina esteve a `load 13`–`27` a jornada inteira), o que a
  manda para a `W9`.
  ⛔⛔⛔ **E o report de 18/09 (*«em `Thin Walled: Solid` … não há transição suave … mas uma linha
  dura»*) tem resposta, e ela foi achada por uma FOTO e não por uma régua** ([`10`
  §11](../../Render3d/10_a_luz_que_atravessa_a_peca.md)): ⭐⭐⭐ **a linha é a borda da SOMBRA QUE A
  PLACA LANÇA SOBRE A BOLA** — o experimento que o decide é de uma linha (`PH2D_TERM_SO_A_BOLA=1`:
  mesma câmera, mesma luz, **sem a placa** ⇒ a bola sai **perfeitamente lisa**). Ela é dura porque a
  luz é um **PONTO**, e só incomoda no `Solid` porque ali há contraste através da borda. ⛔ Correcto
  como geometria e **errado como produto**: num jade a luz espalha-se por baixo da superfície para
  dentro da sombra — *a difusão está na lei do `N·L` e não está na lei da SOMBRA*. ⏳ A cura tem
  endereço e é wave própria: **um segundo canal de visibilidade, borrado pelo raio `mfp/κ`, lido só
  pela closure de subsuperfície** (a maquinaria do borrão com guarda de normal já existe; muda a
  fronteira do `ph2d-material`, que hoje tem UMA radiância por lâmpada para todas as closures).
  ⛔⛔ **DUAS curas foram construídas inteiras e a foto refutou-as** (a cura da sombra foi
  revertida no commit seguinte — ⚠️ um commit não pode citar o próprio sha, e o endereço estável é o
  [`10` §11.3](../../Render3d/10_a_luz_que_atravessa_a_peca.md)): (a) *marchar o raio de costas e só contar depois de sair do próprio corpo* — tinha
  gémeo em WGSL, **6/6** paridades, **4/4** mutações e a banda de `p99 9,21 → 3,71`, e **pintou um
  FIO escuro SERRILHADO** na borda, porque *um `if` por pixel desenha a fronteira entre os dois
  ramos*; (b) *apagar o ramo* devolve **acne** (`p99 13,06`). Os dois gates que a cura A teria
  partido FICAM, porque são onde a 2.ª tentativa bate. ⭐⭐ **O que FICA do dia é outra coisa, medida
  na lei SOZINHA:** o `max(cos(θ+x),0)` amostrado no MEIO da célula punha um vinco em
  **`N·L = −0,0980` = `sin(π/32)`** que **não se movia com o `Subsurface Radius`** — *uma feição
  cuja posição não depende de nenhum parâmetro físico é da discretização* —, e integrar o cosseno
  **exactamente dentro da célula** leva o pior vinco visível de `67`–`95` para **`0,4`–`0,5`**
  (o opaco lê `276`, a parede fina **aprovada pelo dono** `137`). ⛔ **Divergência DECLARADA** (a
  mediana contra o oráculo `5,0e-4 → 2,9e-3`) e **inevitável**: o integral verdadeiro é um só e é o
  ponto médio a `N = 32` que está longe dele. Ela é **load-bearing** — revertê-la reprova no
  `o_macico_nao_e_mais_duro_que_o_lado_que_o_dono_aprovou`, cuja barra sai do lado APROVADO.
  ⭐⭐⭐ **E A SOMBRA PASSOU A TER A BORDA MOLE NUM MATERIAL TRANSLÚCIDO** (ordem do dono, 18/09:
  *«sim. faça»* — [`10` §12](../../Render3d/10_a_luz_que_atravessa_a_peca.md)): *a visibilidade que
  uma closure TRANSLÚCIDA lê é a MÉDIA da vizinhança, sobre a distância de espalhamento do material*
  ([`ph2d-field-render/sss_shadow.rs`](../../../crates/ph2d-field-render/src/sss_shadow.rs)). A difusão já
  estava na lei do `N·L` e **não estava na lei da SOMBRA**. ⭐ O raio é o `subsurface_radius` **por
  canal**, em unidades do MUNDO convertidas a píxeis pela câmera — *é por ser por canal que a borda
  fica avermelhada*, e um raio escrito em píxeis seria uma borda que encolhe ao aproximar.
  ⭐⭐ **E não foi preciso partir o `compose`:** a composição do OpenPBR é **linear na resposta das
  closures**, logo compor com e sem o peso de subsuperfície e ficar com a diferença dá **exactamente**
  a parcela dela através da pilha ⇒ `Surface::direct_sss` com DUAS radiâncias, e com as duas iguais
  ele é o `direct` **ao bit**. **Medido:** a quebra na banda do terminador `9,21 → 1,36`, o material
  sem subsuperfície **byte a byte o de sempre**, e a foto da `=33` com a linha **desaparecida** e a
  sombra ainda lá. ⛔⛔ **DUAS mutações sobreviveram primeiro, e cada uma nomeou uma régua em falta:**
  a cena de jade tem `subsurface_weight = 1`, logo *trocar quem lê o quê no resto da pilha não movia
  um byte lá*; e a bola é lisa, logo *uma cena sem quina nenhuma não testa a guarda da quina* — e a
  régua da quina **sobreviveu outra vez** por medir as PONTAS da tira, que ficam fora do alcance
  dela. **4 de 4 sangram** hoje. ⏳ **DECLARADO: o DISPOSITIVO ainda não tem o gémeo** (ele calcula a
  visibilidade dentro da pintura; dá-la mole pede a passagem que a escreve, duas de borrão separável
  e a leitura — o desenho que o ricochete lá já tem) ⇒ **hoje vê-se no caminho de REFERÊNCIA**
  (`PH2D_FIELD_GPU=0`), e as paridades continuam verdes porque nenhuma delas assa o canal.
  ⭐⭐⭐ **E A COMPARAÇÃO COM O PADRÃO-OURO EXISTE, com número** (18/09, pergunta do dono *«temos o
  padrão ouro? Temos a Unreal instalada»* — [`10` §13–§14](../../Render3d/10_a_luz_que_atravessa_a_peca.md)):
  ⭐ **a Unreal NÃO é a verdade** (a subsuperfície de tempo real dela é também uma aproximação), logo
  o oráculo é um **traçado de caminhos CONVERGIDO** — que corre em **7 s** sobre a nossa cena, com o
  enquadramento tirado das **portas do produto**. ⭐⭐ **O CONTROLO valida a montagem e é ele que dá
  direito ao resto:** na bola OPACA a largura da transição bate **`43 px` contra `43 px`** e a cor
  **`R/B 1,33` contra `1,32`**. **O veredito:** a **largura** do jade está perto (`70` contra `80 px`,
  `12,5 %` mais apertada — a wave da §12 pôs a borda no regime certo), e ⛔⛔⛔ **a COR é uma lei
  SURDA**: varrendo três profundidades, a verdade balança **`3,19 → 1,00`** e nós balançamos
  `1,42 → 1,61`; e com o **raio IGUAL nos três canais** nós lemos **`1,41` nas TRÊS** — *a nossa
  matiz não depende da profundidade*, e o mecanismo é a ESTRUTURA da lei (`sss = subsurface_color ×
  integrate_burley`, que com os canais a partilharem o `mfp` devolve o mesmo nos três). ⚠️⚠️ **E isto
  NÃO é defeito do nosso porte — é da aproximação que a indústria publica**, que reproduzimos com
  paridade medida ⇒ *fechar este buraco é SUPERAR a referência, não alcançá-la*. ⛔ **Uma leitura
  minha foi CORRIGIDA pela varredura:** com um ponto só e máscaras diferentes eu escrevi *«a cor
  move-se no sentido oposto»*; com três pontos e a mesma máscara o que há é uma lei que **não se
  move**. ⚠️ **Piso de ruído do oráculo, medido: `~1e-4`** — o Cycles em CPU não é bit-reprodutível
  entre invocações, e ⛔ o `sha256` de um EXR não compara píxeis (ele embute `Date`/`RenderTime`).
  ⚠️ **INC-R1 registado:** o método manda a corrida do alvo com parede ser acto de uma janela **E**, e
  eu corri-a — contaminação realizada **zero** (li só a IMAGEM de uma cena nossa), mas o valor da
  parede é o protocolo; desde então as corridas vão para o **E**, e foi o relatório dele que apanhou
  **dois defeitos meus** no leitor de PFM (big-endian lido como little; a linha da escala nunca lida)
  que eram a origem do «desvio de `96 %`».
  ⭐⭐⭐ **E A CAMADA DE ESTILO EXISTE desde 19/09** (ordem do dono: *«8 e depois do smoke o 7»* — o
  ingrediente **`8`** do [`01`](../../Render3d/01_o_alvo_decomposto.md), ⇒ **SETE dos oito**;
  mecanismo na [`11`](../../Render3d/11_a_camada_de_estilo.md), cena **`=35`**): a crate-folha
  [`ph2d-style`](../../../crates/ph2d-style/src/lib.rs) (zero deps, o molde da `ph2d-view-transform`) com
  **quatro** botões — contorno · tinta por **curvatura** · grade por **zona** · saturação da
  **indirecta** —, gémeo em WGSL e **dez fileiras** no painel, só no Render. ⭐⭐ **A omissão é a
  identidade AO BIT por CONSTRUÇÃO** (toda tinta é uma cor cujo valor de fábrica é o branco, toda
  soma tem um peso cujo valor de fábrica é zero) ⇒ as paridades já pagas ficam de pé sem serem
  re-medidas (`0` píxeis fora, pior `0`). ⛔⛔ **E uma premissa minha caiu por DUAS mutações
  SOBREVIVENTES:** a forma ingénua `a·(1−w) + b·w` **não é** insegura — `(1−w)+w` dá `1,0` exacto em
  `2 044 824` amostras de `f32` (o erro é meia ULP e o desempate é para o par) —; o perigo é
  **reconstruir `b` por `a + (b − a)` quando `a ≠ b`**, que é onde a mutação sangrou. ⭐⭐⭐ **E o
  SINAL da curvatura já existia, deitado fora UMA LINHA antes de alguém o poder ler:** `H = ∇²f/2`
  distingue bossa de cova, e o consumidor que a estreou (a subsuperfície) pede um COMPRIMENTO — hoje
  ela toma o módulo do lado dela (imagem byte a byte a mesma) e a tinta lê o sinal, que é a diferença
  entre um contorno e uma sujidade (medido: `584` de `3 631` píxeis negativos, `min = −3,361` =
  `−1/0,30`, o raio da cratera ao 3.º decimal). ⭐⭐⭐ **A lição do §24 foi aplicada ANTES da 1.ª
  linha:** o estilo entra na **ASSINATURA** (`ph2d_field_render::Presentation`, o MESMO tipo nos dois
  motores), a apresentação é montada **uma vez e antes** do ramo do dispositivo, e a curvatura é
  medida quando **o material OU o estilo** a lê — ⛔ perguntar só ao material faria o artista mexer na
  tinta de aresta e a peça não mudar um pixel. ⛔⛔ **A paridade achou uma divergência PRÉ-EXISTENTE
  que só um consumidor LINEAR revela:** a contagem satura em `~165` píxeis e a magnitude cresce com o
  ganho (`nitidez 0,2 → 0` fora · `1 → 7` bytes · `2 → 13` · `8 → 45`) ⇒ os dois motores medem a
  curvatura por caminhos diferentes e o único consumidor que ela tinha **satura**-a numa tabela
  pré-integrada; ⏳ dívida nomeada, e **a barra não foi afrouxada** — a tinta mede-se onde não
  amplifica. ⛔ **O MATCAP fica FORA por decisão** (ele é a luz do OLHO; tingi-lo faria o artista
  medir a peça através de uma mentira), com censo a nomear a excepção. ⚠️ O estilo é **VISTA** ao lado
  do olhar e passa a documento no mesmo dia que ele. ⚠️⚠️ **E a FOTO reenquadrou a cena** — as covas
  ficavam de lado (a câmera de omissão é uma três-quartos: as posições passam a ser DERIVADAS do
  olho), a barra saía do ecrã, e ⛔ **fotografar com o `$HOME` do dono fotografa a BANCADA dele**
  (`~/.ph2d/layout.txt`), não a cena.
  ⛔⛔⛔ **E o SMOKE reprovou-a — *«se modifico qualquer cor em style, todas mudam ao mesmo tempo»*
  (19/09): CINCO fileiras, UM controlo** ([`11` §9](../../Render3d/11_a_camada_de_estilo.md)). O
  selector de cor da casa é **UM** e flutua, e um painel entra nele **registando o `NodeId` da
  amostra** — o id era cunhado `(entidade, campo)` por um `match` cujo braço final dizia, por
  escrito, *«uma amostra sobre um param sem índice não existe hoje; `0` é a resposta estável»*.
  ⭐ **Era verdade no dia em que foi escrita** e ficou falsa quando chegaram cinco cores cujo
  `entity` é `0` **por desenho** (o estilo é da CENA) ⇒ as cinco partilhavam
  `hash("model3d.color.swatch.0.0")`, as cinco liam *«aberto em mim»* e as cinco pediam a escrita.
  ⚠️⚠️ **Os seis gates da wave mediam a LEI e o DRENO, e o defeito vive ENTRE os dois** — na
  IDENTIDADE com que a fileira é pintada; o gate de costura alimenta o dreno com a âncora **já
  certa**, logo entra **abaixo** da rotura. *Nenhum instrumento perguntava se duas fileiras são o
  **MESMO** controlo.* ⛔ **E a segunda metade estava na outra ponta:** a lista que fecha um selector
  órfão derivava o id por um **segundo `match`** que só conhecia `Param::Material` — *duas respostas
  à mesma pergunta*, e elas **já divergiam para a LUZ** desde a wave dela (um selector aberto sobre
  a cor de uma lâmpada nunca era fechado). ⇒ **uma PORTA com dois leitores** (`swatch_id`), espaço de
  nomes **próprio** para a família sem entidade (senão a não-colisão depende do acidente de
  `Entity::to_bits()` nunca valer `0`) e `None` no braço final, que faz a fileira cair para o
  controlo normal — *visível e diferente lê-se como uma falta*. Mutação **4 de 4**, com controlo.
  ⚠️ O arnês mentiu **duas** vezes antes: um filtro que casou **zero** testes imprimiu `ok`, e o
  parser contava `running N tests` quando com UM teste o libtest escreve `running 1 test`.
  ⭐⭐⭐ **E A AUDITORIA DE 19/09 (ordem do dono, quatro frentes em paralelo) devolveu o MECANISMO das
  «bordas muito duras sem ajustes finos»** ([`11` §10](../../Render3d/11_a_camada_de_estilo.md)).
  ⭐⭐⭐ **O campo de curvatura NÃO é contínuo — é um punhado de PLATÔS, um por feição** (`0` na face ·
  `1,501` na bola · `−3,38/−4,22/−5,20` nas crateras · `11,3/33,8` nos filetes), e a prova é a
  resolução: o salto por pixel tem o `p99` a **encolher** ao dobrar os píxeis e o **max NÃO**
  (`19,3 → 17,1` sobre `4×`) — *um campo suave amostrado com metade do pixel tem metade do salto; um
  degrau tem o mesmo*. ⇒ **uma função POR PONTO de um campo constante por troço é constante por
  troço: nenhum botão aplicado a `H` pode produzir um gradiente**, só um operador de VIZINHANÇA. E o
  único do caminho é o **`ε` do estêncil**, escolhido por PRECISÃO (`~ulp^{1/4}`) — *um acidente de
  diferenciação, não um controlo*. **Medido:** a tinta põe um penhasco de **`169` bytes** entre
  píxeis vizinhos numa imagem cujo sombreamento nunca passa de **`16`**; o `ε` tem **`6,5×`** de
  autoridade sobre a dureza e o `Curvature Sharpness` **`1,5×`**. ⛔ **TRÊS explicações plausíveis
  caíram**, uma delas MINHA: o `clamp` a saturar (`4,19×` o controlo já a `1,9 %` de saturação) · o
  anti-serrilhado (`169 → 161` sem os píxeis de borda) · e o **joelho suave** (`smoothstep`), que
  **PIORA** (`168` contra `162`) — *um joelho actua no domínio do VALOR e a dureza vive no do
  ESPAÇO*. ⛔ **E há uma PAREDE:** a `ε/raio ≥ 0,2` o `p05` fica positivo, as crateras deixam de ser
  côncavas e a **`Cavity Tint` morre** (janela útil `[0,03 ; 0,10]`). ⭐⭐⭐ **E a fábrica está
  EXACTAMENTE no ponto de saturação:** `Point::curvature` é `H · raio`, que **numa esfera vale
  exactamente `1`**, logo a nitidez de fábrica (`1,0`) põe uma peça arredondada precisamente onde o
  clamp satura (`2` de `4 593` amostras na banda, contra `4 231` a `0,2`). ⭐⭐ **O ORÁCULO (triagem
  primeiro, e ela PARTE A MEIO — OpenVDB MPL-2.0 e VTK BSD-3 são portas abertas e respondem só à
  metade do ESTIMADOR, que o nosso já BATE em `7`–`74×`; a metade dos CONTROLOS só existe walled,
  logo foi CORRIDO):** ele compra **`6,0×`** de faixa de suavidade com a amplitude parada, por um
  **raio/distância** que nós **não temos**, e dá intensidade própria a aresta e cova (quatro
  factores contra a nossa nitidez partilhada) ⇒ *a borda que o dono fotografou é `8×` mais dura que a
  mais dura que o alvo produz e `50×` mais dura que a mais suave*. ⚠️ **Se um raio entrar, ele é da
  MEDIDA e não do CAMPO** (filtrar o SDF move a superfície `0,84` voxel e dá as MESMAS rampas), e
  ⛔ um borrão de ECRÃ está recusado pelo mecanismo que a própria crate já escreve (ele lê os
  VIZINHOS ⇒ é um passe, e a crate deixaria de ser a lei partilhada pelos dois motores). ⭐⭐⭐ **E o
  *«Zone pivot parece morto»* é LITERAL e tem companhia: QUATRO das dez fileiras são inertes no
  estado em que o painel ABRE** (`Rim Color` e `Rim Width` porque `strength = 0`; `Curvature
  Sharpness` e `Zone Pivot` porque as tintas nascem brancas — *a mesma lei que faz a omissão ser a
  identidade ao bit*), **e as dez shipam `inert: None`** — ⛔ o cabeçalho do `estilo.rs` **cita** a
  lei que o ficheiro não implementa, e o `ParamRow::inert` carrega a **decisão do dono de 18/09**, um
  dia antes do report. ⭐ Armado, o pivô é o botão **MAIS FORTE da camada** (`100 %` da peça, pior
  byte `233`). ⛔ **TRÊS dos cinco tectos são palpites** (`sharpness` mede `2` e não `8` — **`87,5 %`
  do curso compra `1 %` do efeito**; `pivot` pica em `0,5`; `saturation` cresce até `≥ 16`), e três
  knobs multiplicativos correm em pista LINEAR com metade do efeito nos primeiros `3`–`7 %`
  (a porta `link_slider_number_curved` já existe). ⛔ **O painel pode ENGOLIR a secção inteira**: com
  `27` vértices as dez fileiras caem fora do `MAX_ROWS` e o gate que o defende chama `param_rows`
  **directamente**, sem ver as dez apendadas. ⛔ **`Rim Width` tem o nome ao CONTRÁRIO** (subir
  «Width» ESTREITA: `50 %` da silhueta a `w=1`, `99 %` a `w=64`). ⭐⭐⭐ **E a §5 deste doc estava mal
  lida por mim, em dois pontos:** a premissa `f64`-vs-`f32` é **falsa** (os dois motores são `f32`; o
  `f64` é o `Field::at`, que ninguém pinta) e a divergência **não chega ao corpo** — dos `86` píxeis,
  **`84` são de borda anti-serrilhada** e a contagem é a MESMA a `nitidez 2` e `8`, porque a
  `nitidez ≥ 1` **zero** píxeis de cobertura cheia estão dentro da banda e **o clamp absorve**.
  ⭐ A causa fecha em forma fechada: **UM ULP** da avaliação de campo amplificado por
  **`1/(4ε²) = 16 403×`** (`ULP(0,55)/(4ε²) = 9,7769e-4` contra `9,778e-4` medido) ⇒ **não é
  afinável** — e como a amplificação é `1/(4ε²)`, **um `ε` maior divide a divergência pelo quadrado**:
  a alavanca que suaviza a borda cura esta dívida de graça. ⭐ A lei do estilo está **ILIBADA**
  (`0` ULP a `nitidez 2` e `8` com a curvatura entregue) e a borda é **igual nos dois motores**
  (`lados trocados = 0`). ⛔⛔ **E o instrumento apanhou um gate VERDE sobre promessa falsa: a placa
  CONTRAI `a*b + c` num `fma`** — o cabeçalho da `ph2d-style` promete *«nenhuma conta desta crate usa
  `mul_add`»*, honrado no FONTE e violado pelo COMPILADOR (`1 680/1 680` fundido contra `1 463`
  solto), e a contracção **não é exprimível em WGSL hoje**; mais o `pow` do WGSL que não é o `powf`
  do Rust (pior `44` ULP, o erro a escalar com o expoente) e os **subnormais esvaziados a zero**.
  ⭐⭐⭐ **E A CURA SHIPOU no mesmo dia** (ordem do dono: *«siga como achar melhor mas coloque no
  estado da arte»* — [`11` §11](../../Render3d/11_a_camada_de_estilo.md)). **(1) A curvatura ganhou uma
  ESCALA** (`Curvature::softness`, fracção do raio da peça, fábrica `0,064`): medido no caminho do
  produto com o CONTROLO ao lado, o degrau de byte entre píxeis vizinhos cai de **`3,57×`** para
  **`1,40×`** a imagem sem estilo, e as covas continuam côncavas (`p05` de `H·R` `−3,358 → −2,256`).
  **(2) A nitidez PARTIU-SE EM DUAS** (`edge_sharpness` · `cavity_sharpness`), e a justificação é
  nossa e medida: filetes a `H·R ≈ 11`–`34` contra covas a `≈ −3`–`−5` ⇒ *nenhum limiar partilhado
  serve os dois*. **(3) O tecto delas desceu de `8` para `2`**, que é o que a auditoria mediu.
  ⭐⭐⭐ **A decisão de assar vive numa PORTA com DOIS consumidores** (`curvatura::assar_canais`) — o
  quadro e o arnês dos gates —, e ⛔ *escrita em linha ela divergiu no dia em que nasceu*: o produto
  assava dois canais, o arnês assava **um**, e o gate acusou um botão VIVO de não chegar ao pixel.
  ⚠️ **O preço é ZERO no caminho de omissão** (cada canal só é assado se o consumidor DELE estiver
  vivo) e ⭐⭐ a escala maior **cura a dívida da paridade de graça, MEDIDA nos dois pontos**
  (`PH2D_CURV_EPS_ESCALA`, instrumento versionado com a tabela): `|ΔH|` p50 `9,778e-4 → 9,775e-6`,
  **exactamente `100×`**, com `δf/ε²` a prever os dois lados. ⛔⛔ **E a metade que NÃO segue:** a
  contagem de PÍXEIS mal se move (`86 → 85`), porque ela não é feita de `H` — `84` dos `86` são de
  **cobertura parcial**. *Duas grandezas com o mesmo nome, e só uma obedece ao `ε`* (esta linha
  dizia só a primeira metade). ⭐⭐⭐ **E o painel DIZ porque uma fileira
  está apagada** — a `Linha::apagada` liga o `ParamRow::inert` que existia desde 18/09 e que o
  cabeçalho do `estilo.rs` **citava sem cumprir**; o gate tem as **duas** metades (de fábrica há `≥ 4`
  apagadas **e** com os gestos feitos não sobra nenhuma), senão «apagar» viraria licença. ⚠️ A
  arrumação vai de `20` para `24` floats com **RESERVA declarada** (`5` cores + `7` escalares = `22`,
  que o `vec4` arredonda) — *uma posição sem dono e sem régua é onde o campo seguinte aterra por
  engano*, e ⚠️ a do **passo do estilo** não é fileira nem reserva: o dono dela é a MONTAGEM do
  dispositivo (ela precisa do raio da PEÇA), e a **terceira categoria do censo vem com a metade que a
  impede de ser licença** — o `pack` tem de a deixar a ZERO, senão há dois escritores e ganha o
  último a correr. ⛔ **E a premissa de uma recusa MORREU:** o doc da const do shader recusava o `ε`
  por argumento *«para não mudar o produto ao serviço do instrumento»* — verdade enquanto o PRODUTO
  tivesse um `ε` só; hoje tem dois, e a nota é reescrita com a morte à vista (§0.0).
  ⛔⛔⛔ **E a PROVA DE MUTAÇÃO achou que os TRÊS testes de paridade no pixel eram IMPRESSORAS com o
  veredito escrito no NOME** — o corpo de um deles dizia-o por escrito e mandava para *«o gate irmão,
  que é quem tem a barra»*, e o irmão **também não tinha nenhuma**. Medido: devolver o arnês da
  paridade ao estado montado à mão (o defeito real deste dia) põe **`4 677` píxeis a divergir, `4 519`
  no MIOLO**, e os três fechavam **VERDES** ⇒ *o que apanhou aquela regressão foi eu ler uma tabela
  impressa, e ninguém lê uma tabela que passa*. Hoje o `os_pixeis_…_e_nao_os_do_miolo` **afirma**
  (`MIOLO_MAX = 60`, `4×` a medição contra `4 519` da mutação) e leva o CONTROLO de que alguma bateria
  mexeu um pixel contra a peça crua. ⚠️⚠️ **E o arnês mentiu uma QUARTA vez: `running N tests` CONTA
  OS IGNORADOS** — um filtro que casa três `#[ignore]` sem `--ignored` imprime `running 3 tests`,
  corre **ZERO** e lê-se exactamente como *«a mutação sobreviveu»*; a população honesta é
  `passed + failed` do `test result:`. **Mutação 7 de 7**, com controlo; portão `15 067` verdes. ✅ **Smoke do dono APROVADO (19/09).**
  ⛔⛔ **E a FOTO da cena apanhou o que a suíte não vê:** o roteiro da `=35` mandava o dono carregar
  em **três** nomes que esta mesma wave tinha renomeado (`Rim Width` → `Rim Falloff`, e o
  `Curvature Sharpness` partido em dois) — *um `println!` não é compilado contra nada* —, e o painel
  À FRENTE na bancada dele é o do **Sculpt 3D**, logo o passo (1) mandava procurar a secção num
  painel que não está na tela. ⇒ o roteiro nomeia o separador `Model`, e o gate
  `o_roteiro_da_cena_nomeia_controlos_que_existem` **deriva** os candidatos do texto (corridas em
  Maiúscula Inicial, que é a forma de todo rótulo e que a ênfase em CAIXA ALTA não tem) e exige que
  cada um esteja na **mesma tabela que pinta as fileiras**. ⚠️ A 1.ª redacção aceitava CAIXA ALTA e
  acusou a minha própria ênfase — *a cura barata era uma lista de excepções, que é como uma lista
  dessas cresce até não medir nada*.
  ⭐⭐⭐ **E A SOMBRA DE BORDA MOLE CHEGOU AO MODO NORMAL** (19/09, ordem do dono que a tinha adiado
  de manhã e a trouxe para a frente à tarde — [`10` §25](../../Render3d/10_a_luz_que_atravessa_a_peca.md)):
  o **dispositivo** assa o canal ele próprio, em duas passagens separáveis, com o gémeo do
  `Surface::direct_sss` escrito em WGSL — a quebra na banda do terminador **na placa** vai de
  **`9,20` para `1,00`**, que é o número da referência. ⭐⭐ **As duas fronteiras que o plano nomeava
  abriram-se onde já viviam:** o buffer de luz ganhou um bloco RGB **por lâmpada**
  (`1 + n + 6 + mole·6·n` — seis e não três, porque o borrão é separável), e o `mx_direct_sss` compõe
  **com e sem** o peso da subsuperfície e fica com a diferença — a composição do OpenPBR é **linear**
  nesse peso, logo isso é **exactamente** a parcela dela através da pilha, e com as duas radiâncias
  iguais ele é o `mx_direct` **ao bit**. ⛔⛔⛔ **E A PROVA DE MUTAÇÃO DERRUBOU O GATE QUE EU TINHA
  ESCRITO:** com o dispositivo a não pedir o canal, a paridade lia **`99,579 %` contra a barra de
  `99,5`** — *passava por `0,079`* — com o pior byte a **`12`** contra `1`. ⇒ *paridade é uma
  RELAÇÃO (dois motores que ignoram a mesma feature concordam a 100 %), uma FRACÇÃO afoga um
  fenómeno que ocupa `10 %` da imagem, e uma metade que mede o canal no BUFFER não prova que ele
  chega ao PIXEL*. Duas curas: a barra do **pior byte** (`≤ 4`, num vale entre `1` e `12`) e a
  condição de fecho que o plano escreveu — *«as duas colunas lerem o mesmo»* — **a deixar de ser uma
  TABELA IMPRESSA** (`as_duas_colunas_da_banda_leem_o_mesmo`, com o controlo do contraste primeiro);
  ⚠️ a impressora era a **sonda que a própria wave encomendara** como critério de fecho, a **mesma
  forma** que a `W8` pagou seis dias antes. ⭐ E a wave fechou primeiro os **dois buracos de régua**
  que o §24.4 nomeava (a paridade de materiais nunca testara subsuperfície — hoje tem jade e folha,
  `9,9e-6` e `1,6e-6` contra `1e-4`; e o arnês de pintura não assava o canal do lado da CPU).
  ⛔ **Fronteira DECLARADA:** com **dois ou mais** raios de espalhamento distintos na cena o
  dispositivo **recusa o quadro** e a CPU pinta — *nenhuma imagem errada, e o caminho lento não
  define o produto*. ⚠️ Tecto de LOC curado por **CORTE** (o borrão saiu para `paint_wgsl_mole.rs`;
  o `FILE_OVERAGE_OK` está **vazio**) e o `trace.rs` fica com **uma** linha de margem, nomeada no
  §25.7. ⚠️ **E o relógio desta wave NÃO foi medido** — ela entra na tabela da `W9`. Mutação
  **6 de 6**.
  ⛔⛔⛔ **E A LINHA FECHOU COM UM GATE VERMELHO, por ORDEM DO DONO** (20/09, *«2»*: fundir com a
  dívida nomeada e tratá-la na `W9`): o `com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento`
  lê **`10` de `22`** cenas nítidas em movimento contra **`15` de `18`** no `main`. ⚠️ Ele é
  `#[ignore]` ⇒ **o CI não o corre e o `ship.sh` também não** — só aparece a quem corra a bateria de
  GPU, e foi o **fecho** que o apanhou. ⭐⭐ **O achado é UMA cena e a assinatura é a INVARIÂNCIA À
  CARGA:** a `=30` custa `96`–`98 ms` a `68 %`, `2 %` e `14 %` de CPU ociosa (contra `13,45` no main,
  e ainda por cima com o divisor em `3` — **um nono dos píxeis**), com a peça **idêntica** dos dois
  lados ao pormenor (`308 instr` · `190,9 passos/acerto`) ⇒ *o que ficou caro é o DESENHO*. ⛔ **Cinco
  suspeitos ELIMINADOS** (sondas · curvatura · subsuperfície · borda mole · brilho — todos atrás de
  guardas que os valores de fábrica não abrem), e a hipótese que fica tem **contradição à vista**: a
  cena `28`, com o DOBRO dos passos por acerto, **melhorou**. ⚠️⚠️ **E o gate reprova por DUAS contas
  somadas** — a cena **e** o denominador ter subido de `18` para `22` com as cenas novas das waves;
  *acrescentar uma cena cara a um gate que mede uma FRACÇÃO baixa-a sem que nada tenha regredido*.
  ⛔⛔ **E DOIS erros de MÉTODO foram pagos a medir isto, os dois registados:** partilhar o
  `CARGO_TARGET_DIR` entre worktrees **troca os `.rlib`** e a 1.ª leitura deu **`6,5×` onde o limpo
  dá `1,4×`** (eu reportei um alarme que era meu); e este gate é tão sensível à carga que a MESMA
  árvore deu `10`, `9` e `6` — *meça a ociosidade real, o `loadavg` mente a decair*.
  [Handoff §10](../../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_A_LINHA_2026-09-20.md) ·
  [plano `W9`](../../Render3d/03_o_plano.md)
  ⭐⭐⭐ **E A LINHA FECHOU OUTRA VEZ em 25/09 (105 commits, já rebaseados): a lei que acende o sprite (`ph2d-form-pbr`), «o que se vê é o que se assa», o catavento (`Mesh3D`, cena `=53`) e a `W9` até à oclusão a passo — o gate vermelho herdado está VERDE; `PROJECT_SCHEMA` +5 e registos +1, com o atrito MEDIDO contra `line/components`, `line/sculpt3d` e `line/UIUX`; ⏳ a foto do dono de 25/09 (grosso a mexer + contorno pontilhado) fica aberta** — [handoff do INTEGRADOR](../../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_A_LINHA_2026-09-25.md).
  ⭐⭐⭐ **E O RENDER PASSOU A SER TEMPO REAL em 01/10 (9 commits): a oclusão guardada no MUNDO entre quadros, o zoom de perto sem perder resolução, o ruído de girar curado, e a caixa nova/cor nova sem os segundos de compilação (a lei do dono INTERPRETADA, os passes numa rodada só, o arrasto sem re-assar as sondas); zero contadores, zero contrato; ⚠️ o `ph2d-run.sh` deixou de vazar o cadeado da placa para o `sccache`** — [handoff do INTEGRADOR](../../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_O_RELOGIO_DO_RENDER_2026-10-01.md).
  **Aberto:** ⏳ **O filete só é um ARCO a 90°** — o operador recua o vértice `(1 − 1/√2)·r/sin α` e um
  arco verdadeiro recua `r·(1/sin α − 1)`; numa ponta de estrela (19°) isso é **`2,29×` menos** filete
  do que o número diz. Hoje compensa-se **só nas quinas AGUDAS** (`max(1, factor)`), e as duas curas
  gerais estão **medidas e rejeitadas** (doc 06 §102.5 e §104.3) · ⏳ o teto de `round` da **estrela** é
  `12,3 %` do bordo, contra `43–60 %` de todas as outras formas — ela é a única em que a mistura é uma
  faixa estreita a atravessar uma face grande ·
  ⛔ **A BASE FICA:** o quadro de movimento custa `26,7 ms` contra um orçamento de `16,7`
  (era `69` antes de 26/08) — a marcha é `80 %` dele, com `8,7` amostras por pixel, e o custo é **por
  aresta tocada**; ⛔ a **sobre-relaxação** está fora (a contagem de passos já é mínima) e atacar a
  **montagem** tem tecto **medido de `20 %`** · ⏸️ baixar as arestas do contorno a mexer
  (`PREVIEW_MAX_EDGES`) tem o preço medido e **muda a FORMA** — decisão de quem vê · ⏸️ o 2.º degrau
  do assentar custa `504 ms` numa peça densa (a escada tirou-o do caminho; o número fica) · ⏸️ um laço
  que **SUBTRAI** — mecanismo medido (os **três** modificadores são um vocabulário só) e as 4 saídas
  com preço, **decisão do Enio** · ⏸️ a barra **demonstrável** da interpolação trilinear é `√3` e
  ship-se o `√2` medido (dívida nomeada) ·
  ✅ **o panic do `ph2d-gridmap` TEM ENDEREÇO e deixou de matar a tentativa** (30/08): era
  `map.uv[p][l]` na `solve.rs` — a nota procurava a **linha** `336` e a função tinha descido para
  `358/359`, *um número de linha obsoleto não desmente o ficheiro* — mais o irmão `partners[p][l]`
  na `assembly.rs`; os dois **contam e saltam** agora (`SolveReport::mismatched_locals`). ⛔ Ele
  matava **2 das 3** candidatas do botão na escultura mais recente do Enio, e a rede devolvia-as
  como *«a malha é grossa demais»* (hoje `RemeshRefusal::Panicked`). ⏳ A causa a montante fica
  ABERTA — `CutReport::side_patch_flips` dá `0` numa peça e **`2`** na dele
  ([handoff §8-quateretvicies](../../3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-30.md)).
  ⛔⛔ **RECUSAS MEDIDAS — não as reconstrua** (mecanismo no doc 06 §65, §69 e §70): **os níveis de
  exportação NÃO podem mandar na densidade dos quads** — a escada foi implementada inteira e o `Max`
  custou **27 min 29 s** para sair com `316` arestas de bordo e `6` não-manifold; *o limite da cadeia
  não é o tempo, é a TOPOLOGIA da extracção*, e **REVERTEU** (a densidade fina tem de FECHAR primeiro —
  achado da `line/quadextract`) · especializar a **2.ª passagem** do traçado por ladrilho é neutro a pior ·
  o vínculo à escultura **viva** custa `229–389 ms` a 128³ · a grade **fina** para a cadeia de quads é
  **107×** o preço para a mesma resposta, e piora a fidelidade · e a especialização por ladrilho está
  **ilibada** (sem ela o traçado vai de `58` para `565 ms`).
  ⚠️ **SEIS notas deste módulo estavam desactualizadas contra o código** (auditadas em 25 e 26/08): o
  traçado «2,4×» · o teto de `Resolution` · o paralelogramo (feito na W59) · o sítio da peça · o
  `Mirror` «não demonstrável» (**demonstra-se**, e já tem três eixos) · o gradiente da escultura («não
  medido» com um gate a medi-lo). ⛔ E **dois gates prometiam «erro de compilação» sobre listas
  escritas à mão** — os dois estão derivados agora. *O §5 só se edita na integração, então ele acumula
  trabalho já pago — audite a lista antes de pegar um item dela, e confira o CÓDIGO antes de acreditar
  numa ausência.*
  ⭐⭐ **E A FAMÍLIA SAIU DA SHELL** (11/09, W2/L0 — o **piloto** de partir a `shells/desktop`): os
  90 ficheiros `field3d_*` (29 234 LOC) vivem em [`ph2d-app-field3d`](../../../crates/ph2d-app-field3d), a
  moldura 3D que a escultura também consumia saiu para a folha
  [`ph2d-viewport3d`](../../../crates/ph2d-viewport3d), e o `impl App` virou trait de extensão sobre
  [`ph2d-app-host`](../../../crates/ph2d-app-host). ⚠️ **14 cenas de smoke foram PODADAS** (nenhum doc as
  citava; a lista é `scenes::PODADAS` e o roteador diz o porquê a quem pedir uma delas) — as vivas
  são as mesmas de antes, com os **mesmos números**. Molde, armadilhas e provas:
  [`HOWTO_partir_uma_familia_da_shell.md`](../../IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md).
  **Smokes:** pill **MODEL** · `PH2D_FIELD_SMOKE=<n>` (a camada de ESTILO é a cena **`=35`**; o roteador é
  [`smoke_scenes.rs`](../../../crates/ph2d-app-field3d/src/smoke_scenes.rs)).
  ⚠️ **Preferência fora do repo:** `~/.ph2d/prefs.txt` — um `reduced_motion=1` esquecido reprova
  smokes sobre produto correto **em todo o resto do app**, e a viagem entre vistas é a excepção.
  **Ler:** [`docs/3DModeling/`](../../3DModeling) ·
  [`06_resultados_cena_e_gizmo.md`](../../3DModeling/06_resultados_cena_e_gizmo.md) §1–§104 (uma seção
  por wave, com a tabela medida e as provas de mutação; o **§13.0** é a lista viva do que está aberto,
  **auditada contra o código** em 26/08) ·
  [`07_fillet_e_chanfro_por_aresta.md`](../../3DModeling/07_fillet_e_chanfro_por_aresta.md) ·
  [`08_formas_por_formula.md`](../../3DModeling/08_formas_por_formula.md) ·
  [handoff de 29/08](../../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_2026-08-29.md)
  (⚠️ o §9 tem **quatro** coisas que uma leitura rápida do diff entende ao contrário — entre elas que o
  `round` do cone e do prisma era **inerte**, não «fraco» — e o §10 as duas premissas que a
  implementação refutou) · [handoffs](../../3DModeling/handoffs/README.md)
