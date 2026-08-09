# ADR-0157 — Uma deformação de Liquify é uma LISTA DE DABS autorada, cozida no device; o campo denso é cache, nunca estado

**Status:** Aceito (linha `line/Painter`, integrada 2026-08-08). ⚠️ **Este número nasceu 0156 e foi CONTADO para 0157 na integração** — a `line/sculpt3d` levou o 0156 (o AO por-vértice) na mesma janela e chegou ao `main` primeiro; como os NOMES de arquivo diferem o git **nunca conflitou**, e quem pega é o gate `architecture_adr_numbers_are_unique` (a **8ª** vez no repo, depois de 0130→0131, 0134→0140, 0145→0148/0149/0150). *Um número escolhido numa linha paralela é PROVISÓRIO; o `main` do dia manda.*

**Data:** 2026-08-08 · **Contexto do pedido (Enio):** *"O Reshape implementado é o estado da arte? … Twist nas imagens: veja linhas sumindo."* → *"nessa engine tudo será animado em runtime inclusive a deformação do Liquify. Não é melhor pensar logo no modo mais propício para alta performance?"*

## Contexto — a força que obriga a decidir AGORA

A decisão não pode esperar por três coisas que chegaram juntas:

1. **O Reshape vai ser promovido ao rail.** A medição de hoje mostrou que o pill `Sculpt` é **inerte** (traço muda 0 bytes no meio Digital) e que o pill `Deform` é uma **antessala** (`DEFORM_TEMPERAMENT_NONE`, cujo braço do roteador é `_ => true`). A cura é dar o Reshape ao pill — mas promover ao rail uma ferramenta que **destrói arte** seria promover o defeito.
2. **A ferramenta destrói arte, e está medido.** Twist no centro de uma linha preta de 3 px, tela 128:

   Twist parado, pincel `r=100`, sonda a `r=30`, linha preta de 3 px — a fixture do gate
   `measure_the_divergence_of_the_sum`, que é **a mesma** de `a_twist_is_a_rotation_not_a_runaway_shear`:
   tabela e gate concordam por construção, e qualquer um pode re-rodar os dois números.

   | dabs | \|D\| **soma** (hoje) | \|D\| **composto** | tinta soma | tinta comp. |
   |---:|---:|---:|---:|---:|
   | 1 | 3,47 px | 3,47 px | 100,5% | 100,5% |
   | 5 | 17,33 px | 17,10 px | 91,4% | 104,2% |
   | 20 | **69,34 px** | 54,92 px | 57,3% | 113,8% |
   | 60 | **208,01 px** | 19,28 px | **28,1%** | 103,9% |
   | 200 | **693,36 px** | 50,61 px | **4,7%** | 119,0% |

   ⚠️ **A soma é exatamente `N × corda`** — 3,47 × 200 = 694 — uma reta, sem teto. A composição **nunca
   passa de 60** e *oscila* dentro do intervalo (54,92 → 19,28 → 50,61 conforme a rotação total cruza
   180° e volta): essa oscilação **é** a assinatura de uma rotação, e nenhum ajuste de constante a produz
   a partir de uma soma. ⚠️ E a tinta composta passa de 100% porque girar uma linha horizontal a deixa
   **diagonal**, que cobre mais texels — tinta não está sendo criada, está sendo espalhada.

   ⚠️ Uma **rotação** em torno de um centro não pode deslocar um ponto de raio `r` mais que **`2r`** — o diâmetro do círculo dele, atingido a 180°. A r=30 o teto é **60 px**; a 3,47 px por dab a soma o cruza em **~18 dabs** e segue crescendo **linearmente, sem limite** (693 px em 200). O mapa deixou de ser uma rotação e virou um **cisalhamento tangencial divergente**: cada destino busca a fonte longe demais, a linha é esticada até virar fio translúcido (os arcos finos da foto do Enio) e depois some no branco.

   **A causa era uma linha:** `warp/apply.rs` acumulava `d[0] += a[0]; d[1] += a[1]` — uma **soma de cordas eulerianas**. Somar a corda `R(θ)v − v` N vezes dá `N·(corda)`, uma reta tangente; compor dá `R(Nθ)`, limitado. **Somar é composição exata para TRANSLAÇÃO e para mais nada** — e é exatamente por isso que só o **Push** parecia bom.

   ⚠️ **E a resposta certa já está neste repo, num arquivo irmão.** `ph2d-painter-brush::smear_field` compõe (`disp_new(p) = v(p) + disp_old(p − v(p))`, semi-lagrangiano) e o doc-comment dele diz textualmente: *"a acumulação óbvia, `disp[i] += step·w(i)`, é ERRADA, e errada de um jeito que vale registrar porque ela PARECE certa"*. O Deform fazia o que o irmão documenta como errado — **a travessia landou em 2026-08-08** e as duas frases ficam aqui porque o CONTEXTO é o que obrigou a decidir; o estado de hoje está no §preço.

3. **Tudo nesta engine é animado em runtime — e o estado atual não é animável.** O que hoje representa uma deformação é o campo denso `disp` (`[f32; 2]` por texel):

   | | por POSE |
   |---|---|
   | denso 2048² | **32,0 MB** |
   | denso 4096² | **128,0 MB** |

   Dois keyframes de um Liquify a 4K são **256 MB**, e interpolá-los é cruzar dois blobs de pixels — uma operação sem significado editorial: não se pode *afinar* o twist, só fundir duas fotos dele.

**E não existe caminho de GPU.** `grep -rn "gpu|device|wgpu"` em `warp/` volta **vazio**. O custo medido é `O(pegada)` — **plano na tela** (o gate `warp_perf_kill_criterion` mede razão 1,00× entre 2048² e 4096²) e quadrático no raio:

| tela | Size | ms/move |
|---|---:|---:|
| 2048² | 25% | 0,50 |
| 2048² | 50% | 2,54 |
| 2048² | **100%** | **40,84** |
| 4096² | 100% | 51,40 |

⚠️ **A leitura honesta desta tabela:** a lentidão que o artista sente **não é da tela, é do raio** (a observação dele — *"velocidade em imagens não muito grandes e pincel não muito grande"* — é literalmente esta linha), e **60% do custo é a advecção do RELEVO**, não o warp da cor.

## Pesquisa — e o ABANDONO, que é o achado mais barato

**O abandono central tem 34 anos e ninguém voltou atrás.** Beier & Neely, *Feature-Based Image Metamorphosis* (SIGGRAPH 1992), trocou o campo/malha denso por **pares de linhas autorados**, dos quais o campo é derivado — porque especificar e interpolar a coisa densa é inautorável. Todo warp animado desde então herda essa forma.

| Sistema | O que é o ESTADO | O que é animável |
|---|---|---|
| **Photoshop Liquify** | malha (salvável em disco: *Save Mesh*) | ⚠️ **nada** — a Adobe nunca a ofereceu como propriedade animável; é artefato de EDIÇÃO |
| **After Effects Puppet** | pinos sobre malha ARAP | os **pinos** (um punhado de pontos 2D) |
| **After Effects Mesh Warp** | grade de controle grossa | a **grade**, como UMA propriedade |
| **Live2D · Spine · Rive** | malha + pesos | a pose dos **ossos** |
| **MLS** (Schaefer 2006) | handles | os **handles**; a deformação é forma fechada, GPU-friendly |

**Ninguém keyframa o campo denso.** Ele é sempre derivado de algo pequeno.

⚠️ **O contra-exemplo, e ele é de ergonomia, não de representação:** o Krita põe a Liquify **dentro** do Transform Tool, entre cinco modos dirigidos por **alças** — ela é a única dirigida por **pincel**, a estranha da lista, e a que os usuários não acham. Mesmo lá o estado é uma malha.

⚠️ **E o abandono DENTRO de casa, medido:** o `smear_field` tentou a soma, mediu a trilha **parar ~35 px depois da crista num arrasto de 160 px**, e trocou por composição. O mesmo erro está vivo no Deform — o que prova que a lição não viaja sozinha entre dois arquivos do mesmo módulo.

## Decisão

> **A fonte de uma deformação é a lista de dabs que o artista pintou; a grade de deslocamento é um CACHE cozido por frame no device, e nunca o estado.**

- **Autorado** (pequeno, animável, interpolável): `DeformDab { center, radius, mode, strength }` — **~16 B por dab**; um traço de 640 dabs são **~10 KB**. O que a timeline anima são *força*, *centro*, *ângulo* — propriedades que um artista entende e que interpolam com significado.
- **Cozido** (por frame, no device): a grade de deslocamento reconstruída + **um gather** — a forma exata do `ImpastoLightPass` e do `ph2d-paint-gpu` que esta linha já shipou.
- **A composição acontece NO COOK, em ordem** ⇒ exata por construção. ⚠️ **O bug do Twist morre como CONSEQUÊNCIA, não como patch:** a soma só existia porque alguém precisava de um acumulador barato *entre eventos de ponteiro*, e o cook não tem esse problema.

Isto é a lei que esta casa já aplica em toda parte — **`fonte ≠ cozido`**: [ADR-0121](0121-vector-live-corners-authored-source-cooked-geometry.md) (a quina autorada × a geometria cozida), [ADR-0132](0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md) (a pilha de LPE re-cozida por frame), os deformers do Motion Nodes que cozinham **100% no device**. O Liquify é o mesmo formato de problema e é hoje **o único que guarda o cozido como se fosse a fonte**.

## Alternativas REJEITADAS, com o motivo MEDIDO

1. **Manter o campo denso e animá-lo.** ⛔ **128 MB por pose** a 4096². Não é uma questão de custo: interpolar duas poses é cruzar dois blobs de pixels, e o artista não ganha nenhum controle que consiga nomear.

2. **Só trocar a soma por composição, no campo denso.** ⛔ Conserta o Twist (é a metade certa) e **não torna nada animável** — o cozido continua sendo a fonte. E o trabalho seria jogado fora na wave seguinte: a composição tem de acontecer no cook.

3. **A GRADE como estado autorado, em vez de cache.** ⛔ **O erro de reconstrução depende do RAIO do pincel**, então guardá-la assa esse erro para sempre. Erro pior de reconstrução (px de deslocamento) de um twist de 160°:

   | raio | passo 4 px | passo 8 px | passo 16 px |
   |---:|---:|---:|---:|
   | 25 | **1,29** | 4,63 | 12,65 |
   | 50 | 0,67 | 2,58 | 9,30 |
   | 100 | 0,40 | 1,33 | 5,18 |
   | 200 | 0,21 | 0,79 | 2,66 |
   | 400 | **0,11** | 0,41 | 1,58 |

   O erro escala com `(passo/raio)²`. Como **cache**, o passo é re-escolhível a qualquer momento (e pode acompanhar o menor raio da cena); como **fonte**, ele é uma perda permanente decidida no dia em que o artista usou um pincel pequeno.

4. **Pinos ARAP / MLS puro (sem pincel).** ⛔ Rejeitado **como substituto**, não como técnica: muda o GESTO — o artista deixa de *pintar* a deformação e passa a arrastar handles. O Push que ele aprovou é um gesto de **pincel**, e o pincel é o que o Krita esconde e o que a nossa foto mostra funcionando. Podem coexistir depois; um não pode ocupar o lugar do outro.

## O PREÇO da decisão escolhida, explícito

- ⚠️ **Um cache é uma invalidação, e invalidação é onde moram bugs.** O `ImpastoLightPass` já paga esse preço e o registrou (*"uma versão teria de rastrear TODA entrada do fold, e o modo de falha de esquecer uma é uma luz velha que ninguém vê que é velha"*). A mesma armadilha nasce aqui.
- ⚠️ **Arte já deformada com o motor atual NÃO converte.** O campo somado não corresponde a nenhuma composição de dabs — ele não é o mapa de gesto nenhum. A migração honesta é **assar o que existe** (aplicar uma vez, como hoje) e seguir com o motor novo.
- **Um dab não é editável ponto-a-ponto depois.** Não se "pega" um dab e o move; o escape continua sendo o **Reconstruct**, que já existe e já relaxa o mapa em direção à identidade.
- **O cook passa a custar em cena ANIMADA, mesmo com ninguém pintando** — é a troca que torna a deformação uma propriedade viva em vez de pixels assados.
- **MEDIDO (2026-08-08) — e o resultado DISSOLVE o passo da grade.** O kill-criterion do W0 era o custo de compor **N dabs por pixel** no device, porque é ele que decide quão grossa a grade precisa ser. Medido na RTX (`cook_gpu::measure_the_device_cook`, Twist, pincel cobrindo a grade inteira = o pior caso de cobertura, o mesmo regime dos 31,0 ns/(nó·dab) **seriais** da CPU):

  | lado | N=16 | N=64 | N=256 |
  |---:|---:|---:|---:|
  | 512² | 0,047 ms | 0,162 | 0,568 |
  | 1024² | 0,150 | 0,539 | 2,132 |
  | 2048² | 0,557 | 2,190 | 8,450 |

  **0,008 ns por (nó · dab)**, estável nas células grandes. ⚠️ **A evidência de que o número é real veio de graça, e é a mesma FORMA que a sonda de CPU exige:** o custo é **linear nas duas metades** — 4× o `N` dá 4× o tempo (2,190 → 8,450) e 4× os nós dá 3,7× (0,150 → 0,557). Um kernel elidido pelo driver não escala; este escala.

  **O que isso decide:** um traço real cobre a PEGADA, não a tela — 600² nós com 64 dabs custam **0,18 ms**. Não há passo a escolher: **o passo é 1, e a "grade" é o próprio pixel**, com erro de reconstrução **zero**. A tabela de erro da alternativa 3 deixa de ser um trade e vira o que ela sempre foi por baixo: o preço de guardar, não o de cozinhar.

  ⚠️ **E o teto está nomeado, não escondido:** cozinhar a TELA INTEIRA a 4096² custa ~8,6 ms com 64 dabs e ~34 ms com 256 — mais que um quadro. Ou seja o cook tem de ser **limitado pela pegada**, exatamente a lei que o resto desta linha já vive; um cook canvas-inteiro é a forma de falha, e ela é de escopo, não de velocidade.

- ⚠️ **MEDIDO na travessia (2026-08-08) — e é o que torna a LISTA obrigatória em vez de gosto arquitetural.** O `apply.rs` passou a compor, e o cache denso é avançado **incrementalmente** (`D_k(p) = v_k(p) + D_{k−1}(p − v_k(p))`, lendo o mapa antigo por bilinear). Isso mata o cisalhamento divergente — o teto `2r` passa no produto —, mas a reamostragem do mapa **num campo de ROTAÇÃO amplifica**, porque o erro de um passo entra na POSIÇÃO de leitura do seguinte:

  | N dabs | deriva contra a lei exata | \|D\| no probe |
  |---:|---:|---:|
  | 1 | **0,0000 px** | 3,47 |
  | 60 | 1,8709 | 19,28 |
  | 200 | **41,4538** | 50,61 |

  **A 200 dabs a deriva tem a ORDEM do próprio sinal**, e 200 dabs é um *hold* normal — o gesto reportado. Ou seja: **o cache incremental não substitui o re-cook exato**, e a decisão deste ADR deixa de ser preferência para virar a única forma que não acumula. O re-cook exato é pagável **só no device** (0,008 ns/(nó·dab)) — o §0 outra vez, em que o caminho lento não define o produto. `N = 1` sai **exato**, que é o teste da fiação: o que sobra depois dela é deriva, não bug.

  ⚠️ **Preço de perf da travessia, medido:** o Deform saiu de **4,18/4,14 para 5,23/5,37 ms/move** (~+25%, kill 8) e a razão entre as duas telas segue **1,03×** — o kernel continua limitado pela pegada.

  ⚠️ **A razão contra a CPU não é o argumento, e não é vendida como tal:** os 31,0 ns são **seriais num núcleo** e esta caminhada satisfaz as condições do [ADR-0109] (linhas disjuntas, leitura pura), então uma CPU row-parallel encurtaria a distância por algo da ordem do número de núcleos. O que decide é o número ABSOLUTO acima: 0,18 ms por traço cabe num quadro com folga de duas ordens de grandeza.

## O que fica GATEADO — para ninguém re-litigar por prosa

| Gate | Afirma | Nasce |
|---|---|---|
| `a_twist_is_a_rotation_not_a_runaway_shear` | N dabs de θ ⇒ `\|D\|` a raio `r` **≤ 2r** | **VERMELHO** (69,34 px contra teto 60, já em 20 dabs) |
| `the_thin_line_survives_a_twist` | a linha de 3 px sobrevive ao swirl | **VERMELHO** (28,1% da tinta) |
| `the_bounded_twist_still_turns_the_picture` | o campo limitado ainda DEFORMA (anti-vácuo) | verde nas DUAS leis, de propósito |
| `the_device_walk_reproduces_the_cpu_law` | a 2ª implementação (WGSL) responde o que a 1ª responde | **verde** (pior delta **0,000307 px** no Twist, 0,000009 no Push, 0,000004 no Pinch; **0 de 4096 nós** acima de 1e-3) |
| `the_authored_list_records_every_dab_in_order` | a lista existe e registra o que o artista carimbou | **verde** |
| `the_lattice_is_a_cache_you_can_throw_away` | descartar o mapa e re-cozinhar da LISTA reproduz a deformação | **verde** — média de diferença **2,16** contra os **79,33** que a deformação em si move (37×). ⚠️ **NÃO é bit-idêntico**, e a redação anterior prometia isso: o mapa do produto é incremental e carrega a deriva; o re-cook é exato |
| `reconstruct_says_the_list_no_longer_explains_the_map` | quem edita o MAPA baixa o `derived` | **verde** — sem ela a promessa quebra em silêncio no 1º Reconstruct |
| `undo_carries_the_list_in_lock_step_with_the_map` | o snapshot leva as duas metades | **verde** — a lição que o `mats` desta linha custou |
| `no_dense_field_is_authored_state` | arch-gate: o campo denso não é serializado nem keyframeável | da W2 |
| `the_cook_reads_no_history_per_pixel` | paridade serial × paralelo (a condição do [ADR-0109](0109-rayon-exception-watercolor-composite.md)) | verde |
| `the_product_composes_the_dab_list_instead_of_summing_it` | o `apply.rs` — não a lei — respeita o teto `2r` | **verde** (a travessia landou; a mutação *"lê no destino"* = a soma sangra) |
| `one_dab_advances_the_map_by_exactly_one_composition` | um dab avança o mapa por UMA composição, contra o mapa que o produto tinha | verde |
| `the_incremental_cache_drifts_from_the_exact_walk_and_this_is_the_number` | a deriva do cache incremental é ESTE número | verde — e ele afirma um **piso** a 200 dabs, para ninguém apagar a nota sem mexer no mecanismo |
| `the_cook_is_bounded_by_the_footprint` | o cook percorre a união dos dabs, **nunca a tela** | verde — ele **substitui** *"o passo é medido"*: a medição do device dissolveu o passo e deixou o ESCOPO no lugar dele |

⚠️ **Os OITO primeiros existem e rodaram** (`warp::compose_tests` e `warp::cook_gpu`, os dois de GPU sob `--ignored`); os três últimos são a lista da **W2** (o cook no device) e nascem com ela. Misturar as duas metades sem dizer qual é qual é como uma tabela de gates vira um relatório de coisas que ninguém escreveu.

⚠️ **O gate de paridade é a ÚNICA defesa desta wave, e é por desenho.** O irmão `ph2d-paint-gpu` consegue contenção **estrutural** (a lei do dab é 1-D em `t` ⇒ a CPU manda uma TABELA e o device só amostra, e a crate não tem como alcançar o `falloff_weight`). Aqui não existe tabela: `at` é um campo VETORIAL por dab, então o device **carrega a lei** e há duas implementações da mesma frase — a situação do `ImpastoLightPass`, não a do carimbo. Fingir contenção seria teatro; medir é o que sobra. Mutações provadas: a caminhada parar de retro-traçar (isto é, virar a SOMA que o ADR condena) sangra **58,53 px em 3938 de 4096 nós**, e matar o lerp do rotor — o *staircase* que o `twist_rotor` da CPU existe para curar — sangra **17,33 px**.

⚠️ **E a medição levantou uma bifurcação que é da W1, não desta decisão:** o `value_noise` é **splitmix64**, aritmética de **64 bits que o WGSL do core não tem**. Ela alimenta Push+Distortion, Pinch+Distortion, Fold+Distortion e o **Wrinkle**, cuja crinkle é intrínseca. As duas saídas — textura de ruído pré-assada, ou um hash de 32 bits — **MUDAM os bytes**, então é escolha com smoke próprio. Até lá o construtor do payload **RECUSA** um dab que carregue ruído (`crosses_to_the_device`), em vez de deixar o device responder outra coisa em silêncio.

⚠️ **O gate que importa mais é o primeiro**, e ele é geométrico: não afirma um número nosso, afirma que **uma rotação é uma rotação**. Um oráculo que comparasse o mapa novo com o mapa antigo seria razão entre dois doentes.

## Escopo

Este ADR decide **a representação**. Não decide: qual pill do rail recebe o Reshape (wave própria, já planejada), se o Reconstruct muda, nem se a família ARAP/handles entra depois. E **não toca contrato congelado** (§6): `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` seguem intactos — `PaintMode` é enum interno e o campo `disp` não viaja em `project.rs` (conferido por grep: `temperament`/`disp` aparecem só no snapshot do PAINEL).
