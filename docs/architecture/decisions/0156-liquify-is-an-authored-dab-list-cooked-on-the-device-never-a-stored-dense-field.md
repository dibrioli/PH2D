# ADR-0156 — Uma deformação de Liquify é uma LISTA DE DABS autorada, cozida no device; o campo denso é cache, nunca estado

**Status:** Proposto (linha `line/Painter`). ⚠️ **Número PROVISÓRIO** — um ADR escolhido numa linha paralela renumera na integração (precedente: 0130→0131, 0134→0140, 0145→0148/0149/0150). O `main` do dia manda; hoje o maior é **0155**.

**Data:** 2026-08-08 · **Contexto do pedido (Enio):** *"O Reshape implementado é o estado da arte? … Twist nas imagens: veja linhas sumindo."* → *"nessa engine tudo será animado em runtime inclusive a deformação do Liquify. Não é melhor pensar logo no modo mais propício para alta performance?"*

## Contexto — a força que obriga a decidir AGORA

A decisão não pode esperar por três coisas que chegaram juntas:

1. **O Reshape vai ser promovido ao rail.** A medição de hoje mostrou que o pill `Sculpt` é **inerte** (traço muda 0 bytes no meio Digital) e que o pill `Deform` é uma **antessala** (`DEFORM_TEMPERAMENT_NONE`, cujo braço do roteador é `_ => true`). A cura é dar o Reshape ao pill — mas promover ao rail uma ferramenta que **destrói arte** seria promover o defeito.
2. **A ferramenta destrói arte, e está medido.** Twist no centro de uma linha preta de 3 px, tela 128:

   | dabs | \|disp\| a r=30 | tinta restante |
   |---:|---:|---:|
   | 1 | 5,20 px | 98,6% |
   | 5 | 15,60 px | 82,0% |
   | 20 | 54,58 px | 24,0% |
   | 60 | **158,55 px** | **3,4%** |

   ⚠️ Uma **rotação** em torno de um centro não pode deslocar um ponto de raio `r` mais que **`2r`** — o diâmetro do círculo dele, atingido a 180°. A r=30 o teto é **60 px**; passamos disso em ~22 dabs e seguimos crescendo **linearmente, sem limite**. O mapa deixou de ser uma rotação e virou um **cisalhamento tangencial divergente**: cada destino busca a fonte longe demais, a linha é esticada até virar fio translúcido (os arcos finos da foto do Enio) e depois some no branco.

   **A causa é uma linha:** `warp/apply.rs` acumula `d[0] += a[0]; d[1] += a[1]` — uma **soma de cordas eulerianas**. Somar a corda `R(θ)v − v` N vezes dá `N·(corda)`, uma reta tangente; compor dá `R(Nθ)`, limitado. **Somar é composição exata para TRANSLAÇÃO e para mais nada** — e é exatamente por isso que só o **Push** parecia bom.

   ⚠️ **E a resposta certa já está neste repo, num arquivo irmão.** `ph2d-painter-brush::smear_field` compõe (`disp_new(p) = v(p) + disp_old(p − v(p))`, semi-lagrangiano) e o doc-comment dele diz textualmente: *"a acumulação óbvia, `disp[i] += step·w(i)`, é ERRADA, e errada de um jeito que vale registrar porque ela PARECE certa"*. O Deform faz hoje o que o irmão documenta como errado.

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
- **NÃO MEDIDO, e é o primeiro número da implementação:** o custo de compor **N dabs por pixel** no device. É a razão de o cache existir; o kill-criterion do W0 é essa medição, e ela decide o passo da grade. ⚠️ Nenhum passo entra no código antes dela (§0: *meça antes de limitar*).

## O que fica GATEADO — para ninguém re-litigar por prosa

| Gate | Afirma | Nasce |
|---|---|---|
| `a_twist_is_a_rotation_not_a_runaway_shear` | N dabs de θ ⇒ `\|disp\|` a raio `r` **≤ 2r** | **VERMELHO** (158,55 px hoje) |
| `the_thin_line_survives_a_twist` | a linha de 3 px sobrevive ao swirl | **VERMELHO** (3,4% hoje) |
| `no_dense_field_is_authored_state` | arch-gate: o campo denso não é serializado nem keyframeável | verde |
| `the_lattice_is_a_cache_you_can_throw_away` | descartar e re-cozinhar dá resultado **bit-idêntico** | verde |
| `the_cook_reads_no_history_per_pixel` | paridade serial × paralelo (a condição do [ADR-0109](0109-rayon-exception-watercolor-composite.md)) | verde |
| `the_lattice_pitch_is_measured_not_chosen` | a tabela passo × raio, executável | verde |

⚠️ **O gate que importa mais é o primeiro**, e ele é geométrico: não afirma um número nosso, afirma que **uma rotação é uma rotação**. Um oráculo que comparasse o mapa novo com o mapa antigo seria razão entre dois doentes.

## Escopo

Este ADR decide **a representação**. Não decide: qual pill do rail recebe o Reshape (wave própria, já planejada), se o Reconstruct muda, nem se a família ARAP/handles entra depois. E **não toca contrato congelado** (§6): `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` seguem intactos — `PaintMode` é enum interno e o campo `disp` não viaja em `project.rs` (conferido por grep: `temperament`/`disp` aparecem só no snapshot do PAINEL).
