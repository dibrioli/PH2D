# 00 · Plano de waves — o motor de física global (`line/physics`)

> Normativo. Companheiro da [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)
> (decide o *quê* e o *porquê*); este plano decide o *como*, wave a wave. Visão:
> [`01_visao.md`](01_visao.md). Estado vivo: [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md).
>
> **Plano VIVO:** waves seguintes o refinam. Cada wave fecha com o **gate batched** (nextest-impacted +
> clippy `--all-targets` + auditoria ≥2 lentes) e um **handoff de tracker**. Sequenciais.
>
> **Regra-mãe (DIRETIVA §3–§5):** verde-de-compilação é velocidade; no audit vale **ZERO**. Todo gate
> nasce **VERMELHO** sobre o bug real, com os **números do PRODUTO**, e morre por uma razão nomeável.
> Toda costura é **exercitada** (que clica, que dá o tick, que olha), não só compilada. Toda defesa em
> camadas ganha **gate POR camada** ([[feedback_layered_defenses_need_per_layer_gates]]).

## Mapa das waves

| Wave | Título | Entrega | Bloqueia |
|---|---|---|---|
| **W1** | Ponte ECS + tick no Playhead + hash no replay gate | o alicerce: sprite cai e assenta, determinístico | tudo |
| **W1.5** | Scrub bit-exato (checkpoint ring) | scrub pra trás sem re-sim O(t) | — (opcional; pode ir depois de W2) |
| **W2a** | Inspector body | a autoria do artista | joints, bake |
| **W2b** | Painel global de mundo | gravidade/solver/arrasto/sono | — |
| **W2c** | Camadas de colisão | a matriz + a camada por-corpo | — |  ✅
| **W3** | Joints | pino/mola/motor/distância; pêndulo, corrente, ragdoll | bake de joints |  ✅
| **W4** | Bake-to-timeline | runtime-truth vira animação editável | — |  ✅
| **W5** | Corpos FILHOS na hierarquia | o collider volta para debaixo do sprite | — |  ✅

### ⚠️ O mapa acima FECHOU — as waves seguintes vivem no tracker

**W1..W5 estão todas ✅, e o módulo continuou muito além delas.** As waves posteriores nasceram do uso (do
smoke do Enio, de uma falta encontrada, de um bug medido) em vez de um plano escrito de antemão, e por isso
são **normativas no [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md)**, cada uma com sua própria seção:

| Wave | Entrega | Smoke |
|---|---|---|
| **W6** | a escala do `Transform` alcança o collider (ball não-uniforme → elipse) | gates |
| **W7** | sensores / triggers | `=10` |
| **Weld** | o 5º joint (`FixedJoint`) | `=11` |
| **BakeChannels** | assar um subconjunto dos canais | — |
| **W8** | gravity scale por corpo | `=12` |
| **Capsule** | o collider de personagem | `=13` |
| **W9** | velocidade inicial por corpo | `=14` |
| **W-CCD** | detecção contínua por corpo | `=15` |
| **W-LockRot** | freeze rotation | `=16` |
| **W-Offset** | offset do collider | `=17` |
| **W-LockPos** | freeze position X/Y | `=18` |
| **W-Mass** | massa manual | `=19` |
| **W-Dominance** | prioridade de colisão | `=20` |
| **W-Material** | regras de combine (bounce/friction) | `=21` |
| **W-Damping** | drag por corpo + modo Combine/Replace | `=22` |
| **W-OneWay** | plataforma jump-through | `=23` |
| **W-Area** | campo de força (área que empurra) | `=24` |
| **W-Contacts** | quem toca quem, onde, sob que carga | `=25` |
| **W-AreaDrag** | a área RESISTE (vento vs água) | `=26` |
| **W-Buoyancy** | Arquimedes: a área sabe quanto do corpo está dentro | `=27` |
| **W-FormDrag** | o arrasto que sabe para onde o corpo aponta | `=28` |
| **W-ContactEvents** | *começou a tocar* / *parou de tocar* — e um scrub não é colisão | `=29` |
| **W-ImpactForce** | *quão forte foi o toque* — o pico entre os sub-passos, não a carga | `=30` |
| **W-TickContacts** | o toque RÁPIDO vira evento — diff por TICK sobre a união dos sub-passos | `=31` |
| **W-AreaTorque** | a MESA GIRATÓRIA — uma área que GIRA o que está dentro (a metade rotacional do campo de força) | `=32` |
| **W-AreaFrame** | o FRAME da zona — girar o sensor gira o vento (toggle `Force Axes: Zone \| World`) | `=34` |
| **W-AreaFalloff** | o FALLOFF da zona — a força e o torque desvanecem do centro até ZERO na borda; a régua é a silhueta da própria zona | `=35` |
| **W-AreaMirror** | o ESPELHO da zona — virar o sprite vira a correia; a força REFLETE (vetor) e o torque INVERTE (pseudoescalar) | `=36` |
| **W-BakeRange** | o INÍCIO do loop é honrado — um loop `[2s, 5s]` assa exatamente `[2s, 5s]`, simulando o front e descartando-o (a metade do bake que estava aberta desde o W4) | `=37` |
| **W-JointAnchor** | a âncora de um joint ganha um DOT âmbar agarrável no canvas (handle de PONTO — os 3 publicadores de `GizmoView` são caixas), arrasta = `Translate` da seleção | `=38` |
| **W-BakeJoint** | assar UM corpo de um rig articulado puxa o componente conexo DINÂMICO inteiro (`jointed_group`) — não há bake parcial coerente de um rig acoplado | `=39` |
| **W-JointAuthoring** | §12 redesenhada: linha por corpo (Body A/B + nome vigente + eyedropper que ARMA um canvas-pick) + smoke de autoria do zero; a criação já existia desde o W3, faltava descobribilidade | `=40` |
| **W-AnchorFollow** (padrão-ouro W1) | a âncora vira **body-local por corpo** (`PhysicsJoint.local_a/b`, rep nativa do rapier) e SEGUE o corpo — mover um corpo não desliza mais o pino (`PROJECT_SCHEMA` 30). A coluna do padrão-ouro; as ex-"waves 2-5" foram ABSORVIDAS pelo plano 02 (ver linha abaixo) | `=41` |
| **W-JointParams** (P0 — correção) | tunar um parâmetro de joint AO VIVO. Report do Enio (*"os parâmetros de Spring não mudam em nada"* + *"Rope a mesma coisa, inconsistente, às vezes funciona"*). **DUAS causas:** (1) a PONTE gateava o re-describe em `at_rest` — gate do W3 (proteger a âncora mid-swing) que o W-AnchorFollow tornou obsoleto (âncora agora body-local, semeada do REPOUSO); fix de 1 linha, cobre todos os params. (2) a COSTURA da UI (§12) enfileirava o `SetComponent` sem dar FLUSH — "às vezes funciona" = só landava quando outro edit drenava a fila; fix = flush por-edit no loop de joint. c9 byte-idêntico. **Smoke OK 2026-07-25** | `=42` |
| **W-J1** (plano 02) | **o joint DESENHA o que ele é.** Havia UMA figura para os 4 tipos; agora glifo por kind (anel · quadrado · zigue-zague · fio), linha de posse **A sólida / B tracejada** (a paleta está cheia ⇒ a diferença é de FORMA), arco de limite com paredes + agulha VIVA, glifo de motor (o mesmo da zona de torque — mesma pergunta), anel de comprimento em MUNDO (dá zoom, cresce) e o vermelho de *restrição não imposta*. Porta única: o desenho lê o `JointView` da ponte (o `desc` que o solver recebeu), nunca o componente. ⚠️ **Medido:** joint do rapier é RÍGIDO — 500× de massa e martelo de 400× abrem **0,00000 m**; quem abre o vão é o corpo **KINEMATIC** curva-dirigido (**1,50 m**), o estado de um rig ASSADO. c9 byte-idêntico | `=43` |
| **W-J2** (plano 02) | **a âncora tem DUAS alças, e um ímã.** Só a ponta A era autorável; a de body B era o que a política de semeadura produzisse (mesmo ponto num Pin/Weld, centro do corpo numa Spring/Rope) e **nenhum gesto do editor a movia**. Agora: 2ª alça (`GIZMO_JOINT_ANCHOR_B`, id 965) desenhada em **anel vazado no MESMO âmbar** — a gramática *sólido = A, vazado = B* das linhas de posse da W-J1; par COINCIDENTE (o caso normal de um Pin) fica concêntrico, com A no quadrado interno e B na faixa de fora; **snap por CTRL** aos 9 pontos do collider (centro/quinas/meios), 14 px, os MESMOS que a alça de pivô já oferece, com CRUZ marcando o capturado. Porta única `bridge/anchors.rs` (`joint_anchor_world` / `set_joint_anchor_world` / `joint_snap_targets`) — o `sync_joint_pivots` passa a ler dela. ⚠️ **O `anchored` MORREU como mecanismo de reposição:** ele é do JOINT INTEIRO, então arrastar A re-derivava B da política e jogaria fora a âncora recém-posta no outro corpo, **em silêncio**; um reposicionamento conhece o lado e escreve o local direto (o sentinela sobrevive só onde re-derivar AMBOS é a intenção: create, troca de kind, re-pick). ⚠️ **As alças agora são REST-ONLY** — o doc do `sync_joint_pivots` já afirmava isso desde a W-AnchorFollow e era **falso**. c9 byte-idêntico | `=44` |
| **W-J2b** (plano 02) | **as alças ficam MAIORES, aparecem sozinhas e ganham o pixel.** Os três pedidos do smoke da W-J2 são a MESMA coisa: **uma joint não tem sprite**, então o `pick_sprites_at_world` não a alcança e a SELEÇÃO era o único jeito de trazer as alças à tela — a rota até uma alça de canvas passava pela **Hierarquia**. Agora `PointGizmoView` carrega uma **LISTA** e toda joint em repouso publica as suas (mesma porta `joint_anchor_world`); vários registram a mesma alça ⇒ o id é **keyed por bits** (`point_handle_id` + `point_hit_map`, o padrão do `keyed_handle_id` dos extras — multiplicadores ímpares e distintos por lado, porque um scrambler LINEAR cancela na comparação e faz ids consecutivos colidirem); **pegar a alça SELECIONA a joint** (a §12 abre no que você pegou); disco 6→9 px e anel 10→15 **com os hit rects seguindo o VISUAL** (marca maior que o retângulo = clique que não faz nada, o modo de falha exato de "deixe maior"); e o z é **ordem de registro** — as alças pintam por ÚLTIMO entre os gizmos, então a âncora sobre a quina de uma sprite é pega como âncora. `joint_entities()` publica o `joints_seen` do reconcile (mais largo que `self.joints`: a joint **dormente** é vista sem ser construída e a ponta A dela segue autorável). ⚠️ **1 mutação sobreviveu e o defeito era do GATE** (pinava uma grafia da seleção, e o bloco a menciona legitimamente porque a ESCREVE) ⇒ o gate passou a afirmar a **lista de argumentos do `open_drag`**. c9 byte-idêntico | `=44` |
| **W-J3** (plano 02) | **pose, não digite.** O canvas MOSTRAVA o alcance de uma dobradiça e o comprimento de uma mola, e mudá-los era voltar ao §12 e digitar — olhar o efeito num lugar e escrever a causa noutro. Agora as **duas paredes do arco** e o **anel de comprimento** têm grip. ⚠️ **Estas duas e não o motor, e a razão é o que uma grandeza É:** um limite é um ÂNGULO e um comprimento é uma DISTÂNCIA — cada um já tem lugar, e arrastar até ele não converte nada; velocidade é uma **TAXA**, nenhum lugar da tela é 120 °/s, e a row do §12 é `num_row` livre **sem faixa** de onde tirar a constante px-por-°/s (as duas leis sem constante falham sozinhas: o arco SATURA em 270°, e uma volta = 360 °/s **DÁ A VOLTA**) ⇒ nomeado, não construído. Quatro espinhas: a geometria que se ARRASTA é a que se DESENHA (`limit_end_screen` é a função que o `limit_arc` usa para a marca radial — a que discordasse seria a INVISÍVEL, o hit rect) · o arrasto escreve pelo MESMO funil do número (`joint_with_edit`, a metade pura do `apply_joint_edit`, agora com dois consumidores) · ⚠️ **uma parede PARA na irmã** (`clamped()` TROCA limites invertidos — certo para quem digita, errado para um gesto: a troca entrega a OUTRA parede à mão no meio do arrasto) · e o **FANTASMA** de B na pose que a parede permite (o *L* do RUBE **sem modo**), que **desenha e nada mais** — o ângulo vem do COMPONENTE, já passado pelo muro, senão seria promessa que o solver quebra. c9 byte-idêntico | `=45` |
| **W-J4** (plano 02) | **criar onde se olha.** Um joint nascia de uma SELEÇÃO, e o preço só aparecia no gesto seguinte: **as âncoras nascem onde a política de semeadura decide**, nunca onde o artista apontava — amarrar uma corda na PONTA de uma prancha era criar, selecionar a joint e arrastar o dot. Agora **aperte o corpo A, arraste, solte no corpo B** e as âncoras nascem NOS dois pontos, com uma corda/mola ganhando de brinde o **comprimento que o arrasto mediu** (medido: a mesma prancha assenta em **rot 104,2°** pendurada pela ponta contra **0,0°** nivelada pela rota do botão — a diferença entre as duas rotas num número). ⚠️ **Uma porta com os pontos OPCIONAIS** (`create_joint_at(.., Option<(wa, wb)>)`; o `None` é a rota antiga byte-idêntica, e o `Some` marca `anchored` — sem isso o reconcile faria o seed e jogaria os dois pontos no lixo com o joint parecendo funcionar); kind que compartilha ponto usa a PRESSÃO nas duas pontas. **E a rota por seleção sobrevive porque ela virou a CORRENTE:** 3+ corpos marcados ⇒ **N−1 joints em UM passo de undo** e o botão passa a CONTAR (`Chain 4 Selected Bodies`) — ⚠️ `join_count: u8` substituiu o `can_join: bool`, porque um bool ao lado de uma contagem discordou dela no dia em que a corrente chegou. Banda âmbar TRACEJADA durante o arrasto, desenhada **FORA do gate `show`** do overlay (contorno é preferência de vista; um gesto em andamento não pode ser invisível por causa dela); release no vazio ou no MESMO corpo = toast e o gesto **segue armado**. ⚠️ **A M1 SOBREVIVEU primeiro e nomeou o buraco:** todos os gates chamavam a porta de criação DIRETO, então descartar os dois pontos no *release* deixava 8 verdes. ⚠️ **E o split de LOC expôs um gate por PROXY:** o `architecture_panel_wiring_parity` enumerava o NOME `populate.rs` ⇒ um code move puro o deixou VERMELHO acusando *"dead on click"* — passou a casar `populate*.rs` por PREFIXO, com mutação provando que ainda sangra. c9 byte-idêntico | `=46` |
| **W-J4b** (plano 02) | **a saída, e as alças fora de alcance.** Os dois ajustes do smoke da W-J4 são sobre o gesto ser MODAL. (1) O botão só ARMAVA, e o gesto come o press no canvas ⇒ **o único jeito de sair era completar um joint que o artista não queria**; agora é toggle, o rótulo vira **`Cancel Joint Drawing`** (nomeia a AÇÃO do clique, não o estado; e não *"Cancel Joint"* — não existe joint nenhum a cancelar, o que sai do ar é o MODO) e **Esc** faz o mesmo, como PRIMEIRO braço da família de Escapes (modal e tool-agnóstico ⇒ cancelar não pode depender da ferramenta na mão). ⚠️ **Desarmar são DUAS coisas e a 2ª carrega o peso:** o `input_dispatch` toma o Move/Up de qualquer gesto em voo **independente do armado**, então uma banda sobrevivente faria o release criar *exatamente o joint que o Esc cancelou* — uma porta (`disarm`), livre e não método (E0499 no sítio de ação, a razão do `join_chain`). (2) **`PointGizmoView::inert`** desenha e **não registra**: um flag, as duas metades, na MESMA função (dimmar sem parar de registrar é o *"dim não é recusa"*; registrar sem dimmar é arrastar achando que está desenhando), com as alças seguindo VISÍVEIS de propósito — durante o gesto você quer ver onde já há âncoras. Alpha 0,35 = degrau entre o ghost (0,28) e o dim (0,5) que o overlay já usa. ⚠️ **TRÊS versões do gate do Esc falharam sobre produto correto, todas por PROXY** (helper cru achando menção sem relação · janela de 600 B colando braços vizinhos · bounding por `KeyCode::Escape` engolido por outro construto) ⇒ o que ficou pergunta pela FAMÍLIA dos quatro braços e afirma ordem dentro dela. 7 mutações, 7 sangram; c9 byte-idêntico | `=46` |
| **W-J5** (plano 02) | **o TRILHO — o 5º tipo.** Espelho do Pin: um Pin deixa GIRAR e proíbe transladar, um **Slider** deixa transladar por UMA direção e proíbe o resto (elevador, porta de correr, pistão). Medido na cena 47: trilho vertical com curso 0,6 para o corpo em `(-4,000, 5,400)` — cai **exatamente 0,60 m** · a 45° o corpo corre a diagonal inteira, `dx = dy = 0,707` = 1,0 m **ao longo do eixo** · e o horizontal **não se move**, que é o CONTROLE (sem ele *"o carro desceu"* seria satisfeito por queda livre). ⚠️ **O eixo mora na ROTAÇÃO da entidade-joint** e nenhum campo novo o guarda — o modelo Godot/Unreal, e o que este componente já implicava (o `Transform` é onde a *colocação* vive; a translação é a âncora, a rotação é a direção) ⇒ **autorável no dia um pelo campo Rotation do §0, zero widget novo**. Conversão por `PhysicsWorld::axis_locals`, irmã de `local_anchor_at_pose` e sob a MESMA lei (ângulo de MUNDO convertido uma vez contra as rotações de REPOUSO); **duas** direções locais e não uma, porque os corpos podem ter rotações diferentes e o `PrismaticJointBuilder::new` põe UM vetor nos dois frames; `libm::sincosf` porque o número alcança o c9. ⚠️ **Consequência gateada:** girar o corpo A **não** re-aponta o trilho (um eixo prismático é direção da CENA, não propriedade do carro) — é por isso que ele é DERIVADO por reconcile em vez de guardado como o `local_a`. ⚠️ **`limit_min/max` passam a carregar a unidade do TIPO** (radianos num Pin, **metros** num Slider — o modelo do próprio rapier), com a porta `limits_in_metres` lida pelo rótulo, pela conversão e pelo solver, e a **troca de tipo RE-SEMEIA** o alcance quando a unidade muda (senão ±45° = ±0,785 rad viram ±0,785 **metros**); `is_hinge` foi SPLIT em **`has_limits`**, a mesma cirurgia que o Weld obrigou em `has_length`. Desenho: **trilho + tracinhos** nos fins de curso (fins de curso em MUNDO, o resto chrome; sem curso não há tracinho). **E o arrasto do W-J4 desenha o trilho** (o rumo do gesto É o eixo). Sem motor de propósito (o linear é W-J6). ⚠️ **A M20 SOBREVIVEU e o defeito era do ORÁCULO** — o round-trip é verde sobre um par de conversões *consistentemente* errado enquanto o trilho fica 57× curto; o oráculo certo é o número GUARDADO. ⚠️ **E a M24 é um bug meu que o seam não pegou:** o `seg_row` faz `ids.zip(labels)` e um `zip` **TRUNCA** ⇒ cinco rótulos com quatro ids deixaram o chip do Slider **sem pintar**, e o gate ficou verde porque iterava a lista CURTA. `PROJECT_SCHEMA` intocado (variant ao FIM, a lei do Weld); c9 **83→85**, hash `55fa97c5…` (MUDA, e é correto) | `=47` |
| **W-J5b** (plano 02) | **o *Join As* também.** Report do Enio com screenshot: *"ficou bom na simulação mas Slider não aparece no painel de joints"* — o painel da foto é o **§11 Join As**, o seletor do que o próximo gesto CRIA, com arrays de ids e rótulos PRÓPRIOS. ⚠️ **A lista de tipos existe DUAS vezes de propósito** (*o que a joint É* × *o que o gesto CRIA*) e o Slider chegou só na primeira ⇒ o pior formato possível: um tipo que a simulação tem, que se vê funcionar, e que o artista **não consegue pedir**. ⚠️ **E o defeito do GATE é a lição:** eu escrevi o gate de comprimento para UM par e o padrão tem dois — um gate que cobre uma instância de um padrão duplicado deixa a outra tão desprotegida quanto antes, com o mecanismo (`zip` que TRUNCA) já nomeado no comentário que eu mesmo escrevera. Agora há um gate por par, cada um apontando para o irmão; e o seam do §11 ganhou a metade que faltava — ele VARRIA os chips (pintado) e não os CLICAVA. M25 (o 5º rótulo removido) sangra nas duas camadas | `=47` |
| **W-J7** (plano 02) | **o joint que PARTE sob carga.** Uma corda rateada em N newtons; passou disso, ela se rompe — o joint é **DESABILITADO**, nunca deletado (a entidade, os parâmetros e a autoria sobrevivem; um Reset o traz de volta, porque nada do rompimento é estado autorado). ⚠️ **O teto é uma FORÇA e não um impulso, e a calibração é EXATA:** um peso pendurado lê o próprio peso (1 kg = 9,8100 N, razão **1,0000** em cinco massas) e **não se move quando nenhum dos dois divisores do solver se move** (9,8100 N a 1/2/4/8/16 sub-passos E a 1/2/4/8/16 iterações) — a primeira versão dividia só pelo `substep_dt` e lia **um quarto** de tudo, porque o island solver do rapier reparte cada sub-passo de novo em `num_solver_iterations`. ⚠️ **`ImpulseJoint::impulses` sozinho NÃO é a reação** (medido: Pin e Weld leem 9,81 N, **Rope e Spring leem 0,00**) — o rapier modela corda como *limite* e mola como *motor*, então a tensão vive em `limits[i].impulse` e `motors[i].impulse`; ler só o óbvio teria shipado um break force que **nunca dispara nos dois tipos que mais o querem**, com todo gate de Pin verde. ⚠️ **É um teto de CARGA, não de impacto:** o pico de uma pancada resolve DENTRO de um sub-passo (uma corda que para 1 kg vindo a 6,26 m/s reporta os mesmos 9,8 N que reporta parada; a 16/32/64 sub-passos o pico aparece — 11314/24584/37485 N — porque aí ele cai entre passos que dá para amostrar). ⚠️ **O teto de TORQUE é do Pin e de mais ninguém**, e é MEDIÇÃO: um eixo angular LIMITADO ou MOTORIZADO reporta exato (4,9050 e 4,9049 contra `m·g·r` = 4,905) e um **TRAVADO reporta 0,0000** enquanto segura os mesmos 4,905 N·m — a row num Weld seria controle que não pode disparar. Os tetos viajam no `user_data` do próprio joint (o checkpoint já o clona; um mapa paralelo esqueceria de rebobinar) com **`0` = ∞**, que é o que um joint anterior a esta wave carrega ⇒ toda cena que a precede é **byte-idêntica** (c9 `c9d4baee…`, 87, intocado). Visível: o joint tinge de **VERMELHO**, perde o envelope (limite/comprimento/motor não estão mais em vigor) e ganha um **estouro de seis pontas** onde partiu; a carga com que ele partiu vai num **toast** (o único lugar que pode carregá-la — um instante depois ele lê zero). `PROJECT_SCHEMA` **32→33**. **12 mutações, 12 sangram** — duas delas expuseram gate fraco meu (um controle que passava porque 10 kg ficam logo abaixo da semente de 100 N; um oráculo de envelope que contava só o vermelho e por isso **não podia** falhar) | `=49` |
| **W-J8** (plano 02) | **a higiene do par.** *Active* (desligar sem apagar: `JointEnabled` é nativo do rapier e nunca era escrito; o joint segue CONSTRUÍDO — pular o spawn o tiraria do canvas e *desligado* viraria indistinguível de *deletado*) · *Collide Connected* (default OFF, e agora com número dos DOIS lados: um hub pinado dentro do plank que ele gira vai de relativo **4.000** para **0.000** com contatos ligados — o motor completamente derrotado por uma interpenetração permanente; e uma caixa amarrada a um bloco **pousa nele** em `y = 0.899` com contatos ligados contra atravessá-lo até `y = −4.000` sem) · *Swap A↔B* · e o joint nasce chamado **"Post : Plank"**. ⚠️ **A medição decidiu o desenho do Swap:** um swap CRU reverte o motor (4.0000 → **−4.0000**), reverte o servo (44.9998° → **−44.9998°**) e **espelha a faixa de limites** (`[min,max]` é a faixa de `θb − θa`, então vira `[−max,−min]` — o plank assenta em −34.3775° em vez de −11.4592°); compensando, a coluna autorada é reproduzida **em toda linha ao 4º decimal** ⇒ a lei é *um swap troca qual ponta se chama A, e nada mais*, e o que ele muda é visível (as duas rows, o ponto âmbar que salta de ponta, a linha sólida × tracejada). Sem compensar, o botão seria o que reverte em silêncio a dobradiça que você afinou. ⚠️ **E o Active expôs um bug latente:** ele e uma RUPTURA escrevem a MESMA flag do rapier, e `JointView::broken` era `!joint_is_enabled()` — desarmar um joint o pintaria **VERMELHO com estouro de seis pontas**, dizendo que o rig cedeu sob carga; o que os separa é o `desc` (o autorado viaja nele, o runtime não). `PROJECT_SCHEMA` **33→34**; c9 **byte-idêntico** (`c9d4baee…`, 87). **15 mutações, 15 sangram** | `=50` |
| **W-JG** (plano 02) | **o grupo carrega o rig.** A W-AnchorFollow tornou a âncora **body-local**, e o preço só aparecia no gesto seguinte: mover UM corpo de um par jointado separa as duas âncoras, o joint nasce ESTICADO e o Play o resolve com um puxão que ninguém autorou. Agora **Alt+arrastar** um elo em repouso arrasta o rig — e a lei é o **componente conexo INTEIRO** (`jointed_rig`), não o do bake: um joint tem DUAS âncoras body-local, então um gancho Static ou uma plataforma Kinematic deixados atrás esticam o joint *exactamente* como um elo Dynamic deixado atrás esticaria (ordem do Enio: *"faça arrastar a cadeia inteira independente do tipo"* — a v1 reusava o `jointed_group` e a corrente andava SEM o gancho). ⚠️ **As duas portas moram na MESMA travessia** (`joint_group::walk`, política de tipo como parâmetro) e **divergem de propósito**, com gate por tipo provando isso: *quem CONGELA quando a física é desligada?* (bake, só Dynamic) contra *quem tem de andar junto para a pose ficar coerente?* (arrasto, todo corpo) — assinaturas idênticas, então a troca compila calada. Medido na cena 51: o elo do meio de uma corrente leva **3 corpos a mais** (os dois elos E o gancho Static); um de dois pêndulos no mesmo gancho Kinematic leva **2** (o gancho E o irmão — onde o grafo se RAMIFICA, um ramo leva o outro, e é o preço honesto da política); o par livre leva **1**. **Três condições** — é um `Translate` (girar/escalar um rig é uma decisão de PIVÔ que esta wave não tomou), o relógio está parado (o gate exato do `sync_joint_pivots`: o rig é carregado precisamente quando as âncoras seguem os corpos) e **Alt ESTÁ apertado** (o rig é opt-in por gesto; ⚠️ o preço é que a cura da pose violada passa a valer só quando o artista pede — sem Alt o joint ainda estica, o que se VÊ. O gate afirma o **SINAL** e não só a presença: `alt_key()` casa com as duas polaridades). ⚠️ **Os DOIS sítios de Down semeiam pela MESMA porta** (`joint_rig_drag`): a alça do gizmo e o pick de canvas carregavam cada um a sua cópia, e duas cópias é como arrastar pela alça passaria a carregar a corrente e arrastar pelo corpo, não. ⚠️ **Regra de parentesco, conservadora:** o translate de grupo soma o delta de MUNDO ao `Transform` **local** de cada extra, então um descendente de outro membro andaria **duas vezes** e o rig explodiria; o rig só acrescenta um corpo quando nenhum outro candidato lhe é parente (teste simétrico), e quem fica de fora deixa o joint esticado, que **se vê**. Zero componente, zero id, zero widget, `PROJECT_SCHEMA` **34** intocado, c9 intocado. **17 mutações, 17 sangram** | `=51` |
| **W-Grab** (plano 02 §8) | **a MÃO: pegar o corpo no PLAY.** Até aqui o Play era só de LEITURA — a pose de um corpo dinâmico é escrita pelo `readback` a cada dispatch, então um arrasto durante o play era sobrescrito no MESMO frame e a cena não podia ser cutucada; todo laboratório de física 2D deixa (Algodoo · testbed do Box2D · RUBE · play mode da Unity). **A lei é o RELÓGIO como interruptor:** em repouso arrastar AUTORA a pose (e com Alt carrega o rig, W-JG); tocando, arrastar um corpo dinâmico é a mão — o MESMO predicado `!is_playing()` da condição 2 do `joint_rig_drag`, do outro lado. ⚠️ **A mão é uma MOLA, não um teleporte:** uma `SpringJoint` entre o ponto pego e um corpo-âncora fixo invisível que É o cursor, então o corpo **colide no caminho**, **soltar não zera a velocidade** (o ARREMESSO cai de graça) e a mola é resolvida junto com as outras restrições (um PD explícito no mesmo ganho explode a 1/60 s). ⚠️ **`MotorModel::AccelerationBased` — a lei do MouseJoint do Box2D, vinda do rapier:** a mola do ARTISTA é `ForceBased` (física, o pesado afunda) e a mão não é, porque ninguém quer lutar contra a massa para reposicionar um caixote — divergência medida entre 1 kg e 25 kg: **0,0000 m na mão, 1,2 m na mola**. **O teto da rigidez é a PAREDE, medido:** k=400 → 5 mm de penetração sob um puxão de 5 m para dentro do muro · 6400 → 62 mm · **25600 → atravessa**; e o atraso (4 m/s) cai como `1/√k` — 400 dá 0,369 m com **sobressinal zero** (`d = 2√k`). ⚠️ **A primeira entrada NÃO-REPRODUZÍVEL do módulo** (params vivos são config, a pose kinematic é respondida pelo `SceneAtTick`; um puxão não está no documento) ⇒ duas regras gateadas: **pegar descarta o ring e nada é gravado com a mão em voo**, e **um rewind solta a mão** — sem elas a resposta para o MESMO tick dependeria do cache, o defeito que a auditoria do W4b nomeou. A metade visível é um **zigzag verde-limão** do cursor ao ponto de pega (a FORMA diz mola, a COR diz de quem é), desenhado sem o gate da tecla `B` porque é gesto. Zero componente, zero id, `PROJECT_SCHEMA` **34** intocado, c9 **byte-idêntico**. **17 mutações, 15 sangram** (2 documentadas como camadas externas) | `=52` |
| **W-Hand** (plano 02 §8) | **a seção da FERRAMENTA: tipos de segurar, explosão e campo de atração** — mais o BUG do *collider fantasma*. ⚠️ **O bug primeiro:** um corpo ESTÁTICO arrastado com o relógio ANDANDO movia o DESENHO e deixava o collider para trás — o solver não escreve a pose de um estático, a cena não a empurra por tick, e o `settle` só rodava PAUSADO; o comentário do `drive_kinematic` **declarava a cobertura completa** (*"caught by `settle`, while paused"*). Lei nova: *a pose de um estático tem UM autor*, logo ela é honrada em TODO dispatch (`settle_static` no `prepare`) — dinâmico e kinematic ficam de fora porque têm dono, e um 2º autor mid-play é o bug de ordem-de-frame do W4. Medido: laje descendo 1 m leva a bola de 0,799 a −0,201 (sem o fix ela fica em 0,799). **Três modos de SEGURAR** (`HoldSpec`), medidos no caminho do produto: Spring (atraso 0,369 m a 4 m/s, **respeita a parede** — 5 mm de penetração) · **Rigid** (atraso **0,000**, giro **0,000** — segura a ATITUDE) · Rope (atraso **exatamente** o slack). ⚠️ **Rigid e Rope ATRAVESSAM geometria**, e é o preço da palavra: uma restrição de distância é tão rígida quanto um weld ⇒ a tabela do teto vale só para a MOLA. ⚠️ O `local_frame1` do `FixedJoint` carrega a rotação VIVA do corpo — identidade ali chicotearia qualquer objeto inclinado para o prumo no press (gate próprio: o irmão pega um corpo em rotação 0, onde *manter* e *endireitar* são a mesma saída). **Dois knobs manuais** — Stiffness `10..6400` e Damping como **RAZÃO** `0..2` (`1` = crítico em QUALQUER rigidez): o valor certo do 2º é função do 1º, e um número solto seria a falha de ergonomia que o Conserve pagou. O achado: *"a mola está dura"* tem duas curas com preços diferentes — rigidez custa **geometria** (6400 → 62 mm de penetração; 25600 atravessa), razão custa **sobressinal** (0,25 → 0,132 m). **Explosão** (impulso `N·s`, uma vez, sem torque — o `AddExplosionForce` da Unity) e **campo de atração** (força `N` sustentada, negativa REPELE), pesados pela MESMA `blast_falloff` (zero exatamente na borda). ⚠️ **O campo PRECISA de resistência, medido**: sem ela é um oscilador e a distância final nem é monotônica na força (10 N → 0,68 m · 20 → 1,61 · 100 → 2,47); `ATTRACT_DAMPING = 4` (dist @2 s: 0 → 2,342 · 4 → 0,012) e **não é knob** — a mesma descoberta que o W-AreaDrag fez do outro lado da cerca. Teto do impulso = **o DETECTOR DE COLISÃO** (100 N·s dão 83 m/s, e a varredura do W-CCD mediu tunelamento entre 100 e 600). `is_grabbing` virou **`is_poking`** (mão OU campo) e as duas regras de determinismo valem para as três; ⚠️ para o estouro a razão da regra 1 é OUTRA (ele é instantâneo — o que quebraria o scrub é um checkpoint de ANTES dele). Seção **INTERACTION** no painel de mundo com **tabela própria** (`IROWS`, 4 consumidores) e um `shown` por row (o painel pergunta para OFERECER, o wrapper para HONRAR). **NADA é persistido**: descreve o PONTEIRO, não a cena ⇒ `PROJECT_SCHEMA` **34** e registro **21** intocados, c9 **byte-idêntico**. **8 mutações, 7 sangram** (a 8ª é redundância-por-construção, documentada). **Smoke OK 2026-07-26** | `=53` |
| **W-IK** ([ADR-0145](../architecture/decisions/0145-physics-ik-is-a-transient-posing-tree-not-a-second-joint-representation.md), [plano 03](03_plano_ik.md)) | **POSAR ARRASTANDO A PONTA** — o horizonte do plano 02 §8, escalonado pelo Enio. Autorar uma cadeia era cinemática DIRETA (gire o ombro, gire o cotovelo, a mão cai onde cair); agora arrastar a mão dobra a cadeia atrás dela. ⚠️ **O multibody é FERRAMENTA, não estado:** a IK do rapier exige coordenadas reduzidas (`Multibody`) enquanto simulamos com `ImpulseJoint`, e pôr um joint em duas representações é a falha de duas-portas que esta linha já pagou 3× — a saída é o CICLO DE VIDA: a árvore nasce no press, morre no release, o `MultibodyJointSet` do mundo continua VAZIO e o `ik_solve` toma `&self` (a IK LÊ o body set e escreve só no multibody). É a lei do bake noutro eixo — *assar não é simular de novo, é ANOTAR*; aqui **posar não é simular, é RESOLVER**, e o que sobra é `Transform` autorado. **A raiz decorre da CENA** (Static/Kinematic alcançável é a raiz; sem nenhum, o mais distante da ponta, e o rapier lhe dá raiz LIVRE) — a 3ª política sobre a MESMA travessia, ao lado do `jointed_group` (bake) e do `jointed_rig` (arrasto). Só joints RÍGIDOS viram elo (`is_rigid_link`): uma Spring é *soft*, e uma cadeia cuja pose depende de forças não tem coordenadas a resolver. ⚠️ **TRÊS premissas derrubadas pela medição, cada uma com gate vermelho antes do código:** (1) o `inverse_kinematics` do rapier **IGNORA limites** (`apply_displacement` é `integrate(1.0, disp)`, sem clamp) — dobradiça limitada a `[0, 0.3]` ia a **−90°**, e uma pose que o Play desfaz no 1º tick é uma promessa quebrada ⇒ PROJEÇÃO pós-solve, exata numa passada (⚠️ vale para o Pin, **não** para o Slider — nomeado e gateado); (2) alvo fora de alcance **COLAPSA** a cadeia (ponta a **0,245 m** do gancho, enrolada) ⇒ `clampMag` do Buss — e ⚠️ **o teto em METROS estava errado**: a instabilidade é relativa ao comprimento do elo, então 0,25 m protegia a cadeia de 1 m e deixava a de 0,2 m colapsar com o gate verde ⇒ teto **adimensional** (1 comprimento de elo; a degradação começa em 2, medido em 3 escalas); (3) a cadeia esticada **PARA DE GIRAR** — 28° fora do alvo, para sempre, com o gate de RAIO **VERDE** sobre isso (esticar e apontar são duas perguntas) ⇒ clamp do alvo ao ALCANCE, e o erro vai a **0,000 rad em 10 solves**. Números MEDIDOS: `damping` **0,1** e não o 1,0 do rapier (0,0787 → 0,0004 m de erro), faixa 0,05..1,0; `max_iters` 10 e **sem slider** (medido INERTE acima de 10 — knob morto fica no tipo, fora da UI); custo 0,002-0,006 ms/solve. 4ª ferramenta **Pose** na seção Interaction, com `runs_at_rest()` como porta única do relógio (as outras três pedem Play, esta pede Pause) e dica própria. **NADA persistido** — `PROJECT_SCHEMA` **37** e registro **21** intocados, c9 **byte-idêntico** (`c9d4baee…`, 87 corpos) | `=54` |
| **W-FK + W-JointTools** ([ADR-0145 §10](../architecture/decisions/0145-physics-ik-is-a-transient-posing-tree-not-a-second-joint-representation.md), [plano 04](04_plano_fk_e_modos_de_joint.md)) | **A CINEMÁTICA DIRETA, e os cinco modos numa seção própria.** Ordem do Enio logo após o smoke da IK: *"FK também é extremamente útil… merece uma seção exclusiva no Painel Physics. Deixe a Interaction para a simulação"*. A FK é a irmã que **NÃO precisa de árvore**: girar um elo em torno da própria junta e levar os descendentes é um **movimento rígido**, então nenhuma restrição interna é violada e não há o que resolver — a sessão colhe um pivô e as poses no press e daí em diante é aritmética **exata** (sem `max_iters`, sem damping, e **imune ao re-describe** que obrigou a árvore de IK a se re-montar). ⚠️ **O limite de um Slider É honrado aqui**, o oposto do ADR §6.1 — não por acaso: aquela projeção só tem o ângulo de `local_to_parent`, que num trilho não mede o curso, e a FK computa a coordenada pela porta que **desfaz os frames** (`joint_coordinate_at`). A hierarquia é a **MESMA** do IK (`ik_plan`), porque uma 2ª política de raiz seria uma 2ª resposta a *"para que lado desta cadeia é 'para cima'?"*. Pegar um elo **soldado** sobe até a junta seguinte e leva a peça inteira (um Weld é a afirmação de que os dois corpos são UM). **A seção Joints** junta os cinco modos: `Body` (o de sempre) · `Rig` (o rig inteiro, `jointed_rig`) · `Links` (só os elos móveis, `jointed_group` — a política que a W-JG tinha na v1 e que a decisão do Enio removeu; ela volta como ESCOLHA em vez de default) · `IK` · `FK`, com a política resolvida por **uma porta** (`jointed_by`). ⚠️ **O Alt é UMA decisão com DUAS metades:** apertado, `drag_reach` devolve `Whole` **e** `gesture` devolve `None`, em qualquer modo — separá-las faz o atalho "não funcionar às vezes", que é a forma mais cara de bug de UI. ⚠️ **Duas medições derrubaram premissas minhas:** o clamp do limite mora no **MAPEAMENTO** e não no acumulador (é manipulação direta — o ângulo do cursor É o da junta; clampar o acumulador faz o elo sair de sincronia com a mão e não voltar), e o curso de um trilho conta **a partir do repouso** (`0.6 + 1.0 = 1.6`, não 1.0 — o par de âncoras é semeado em repouso). ⚠️ E **uma mutação sobreviveu** e acusou o meu gate, não o código: num **Pin em repouso** as duas âncoras são o mesmo ponto de mundo, então apagar o `swap_anchors` é provadamente inerte ali — o gate que a mata precisa de um **Slider** com o carro deslocado e o joint autorado filho-primeiro. **Zero schema, zero componente, zero contrato** (`PROJECT_SCHEMA` **37**, registro **21**, c9 **byte-idêntico**, `c9d4baee…`, 87 corpos); dep nova: `libm` na `ph2d-physics-ecs`, mesmo pin das irmãs | `=55` |
| **W-Rod** ([plano 02 §8.1](02_plano_joints_ui_authoring.md)) | **A BARRA RÍGIDA — o 6º tipo, e o único vínculo que este conjunto não sabia dizer.** O vão é estreito o bastante para passar despercebido lendo a lista: um **Weld** segura a distância *e* congela o giro, uma **Rope** segura só o teto (afrouxa), uma **Spring** é bouncy de propósito. Uma biela quer a distância mantida com as duas pontas livres — que não é nenhum dos três. ⛔ **O desenho do plano foi MEDIDO e MORTO:** *"rope com `set_limits(LinX, [d, d])`"* não segura — `limit_linear_coupled` do rapier 0.28 traz o comentário literal `// FIXME: handle min limit too.`, lê só `limits[1]` e sai com `impulse_bounds = [0, ∞)`, ou seja **unilateral**; construído assim, o rod mede **0,0293 m** no pêndulo invertido, que é o número da CORDA a quatro decimais. **O que ele é:** um **motor de POSIÇÃO no eixo linear acoplado** (`ROD_STIFFNESS = 1e6`, medido — estica **0,1 mm sob 12,6 kg**, uma ordem abaixo da própria tolerância de contato do motor, com ripple **zero**). Mecanicamente a mesma família da Spring, e um TIPO próprio pelo que o artista vê: a mola shipa macia (stiffness 30, cede 6,5 cm sob 0,2 kg) e expõe três números, dos quais o damping tem de ser **função** do stiffness; o rod expõe **um número, o comprimento**, e deriva o resto. ⚠️ **`ROD_DAMPING` é MEDIDO INERTE** (pico 8 µm com damping 0 *e* com 20000, ripple 0) — o rapier integra motor como soft-constraint implícita, então nessa rigidez não há oscilação a amortecer; fica crítico para o caso de alguém amolecer o rod, e **sem gate**, porque uma barra sobre grandeza que não se move não pode falhar pelo motivo que alegaria. ⚠️ **Três mutações acusaram meus próprios gates:** o oráculo do pêndulo lia o ENDPOINT (um peso oscilando deixa até a corda tesa ⇒ os dois tipos terminam a 2 m; o que separa é o MÍNIMO da trajetória) · o do giro lia `rotation`, que **WRAPA** (6 rad viraram `6 − 2π = 0,28` e o gate concluiu que a prancha mal se moveu — a lição da W-AreaTorque outra vez, agora lê `angvel`) · e a rigidez ficou **sem gate** até a 3ª aresta do triângulo existir (*não CEDE sob carga*, contra a mola), porque a carga do pêndulo era leve demais. ⚠️ **A cena refutou a própria prosa:** com os pêndulos soltos na HORIZONTAL a corda mede 2,0000 (um arco em torno do gancho mantém a corda tesa) ⇒ eles nascem quase em CIMA do gancho, e a treliça saiu da configuração degenerada (ápice na linha entre as âncoras cedia 4,7 cm, passando raspando num limiar de 5 cm). Medido: corda **0,347** · mola **1,782** · barra **2,000** · treliça **0,000**. `PROJECT_SCHEMA` **37→38** (variante apendada — o bump é para o caminho INVERSO, como o Weld), registro **21** intocado, c9 **89 corpos**, `5071c4f3…` (debug ≡ release). LOC: `joints.rs` 719 ⇒ split *vocabulário* × *construção* (`world/joint_desc.rs`, o corte que `world/desc.rs` já fez para corpos) | `=56` |
| **W-Wheel** ([plano 02 §8.1](02_plano_joints_ui_authoring.md)) | **A RODA — o cubo que gira E cavalga uma suspensão, e o primeiro tipo do kit a deixar DOIS graus de liberdade livres.** ⚠️ **Não é um "preset" empilhando joints**, e a distinção é load-bearing: dois corpos unidos duas vezes (um prismatic *e* um revolute) dão ao solver duas restrições brigando pelo mesmo par, e ao artista dois objetos para manter em passo. É **UM** `GenericJoint` travando um único eixo (`LIN_Y`) — que é o que uma roda É: tudo livre menos de lado. A suspensão é motor de POSIÇÃO em `LinX` com a mola do artista (os MESMOS `stiffness`/`damping` de uma Spring, porque é a mesma coisa física), o giro é o motor dele em `AngX`, e os dois não colidem porque são eixos diferentes. ⚠️ **E o curso tem os DOIS batentes — o que o Rod não conseguiu:** o acoplamento em rapier é **EXPLÍCITO** (`coupled_axes`, que só `RopeJointBuilder` e `SpringJointBuilder` ligam), então numa roda o `[min, max]` vai pelo `limit_linear` bilateral em vez do `limit_linear_coupled` unilateral. Constantes MEDIDAS num carro no chão: `WHEEL_STIFFNESS = 400` (chassi leve afunda 6,4 cm = 13% da altura de marcha, pesado 16,7 cm; a 200 o pesado afunda 68% e *lê como encostado*, a 800 o leve afunda 6% e *lê como estai*) · `WHEEL_DAMPING = 20` (ultrapassa 1,35× e assenta em 1,1 s; a 0 nunca assenta, a 40 não quica) · `WHEEL_TRAVEL = 0.15` (régua PRÓPRIA — meio metro de suspensão é mais que a altura de marcha de qualquer veículo). ⚠️ **O Wheel REFUTOU a premissa de que `translates()` respondia a duas perguntas:** ele limita uma TRANSLAÇÃO (o curso, em metros) e motoriza uma ROTAÇÃO (a tração, em graus), então `limits_in_metres` e `motor_in_metres` viraram `match` exaustivo — um tipo novo **não compila** até declarar a unidade de cada uma. ⚠️ **QUATRO listas escritas à mão estavam podres e foram fechadas por construção:** a tabela de portas do ECS (parou em 5 tipos — o **Rod atravessou uma wave inteira sem ter uma única resposta conferida**), o gate por-tipo do painel (`0..5`), o gate de figuras distintas do overlay (4 dos 7) e o **"Join As" da §11**, cuja divergência com a §12 é literalmente o bug que aquele doc-comment narra (*"Slider não aparece no painel de joints"*). ⚠️ **Duas mutações acharam buracos nos meus próprios gates:** acoplar os eixos **passou** porque a faixa era SIMÉTRICA e um limite acoplado é sobre a MAGNITUDE (`|x| ≤ s` ≡ `−s ≤ x ≤ s`) — e assimétrico ele nasceu vermelho denunciando que **comprimir é POSITIVO** (meu comentário afirmava o contrário, e simétrico não podia revelá-lo); e colapsar o glifo da roda num anel de pino passou em tudo, porque **nada media a geometria dele** (a cicatriz `Layer`/`Layers`). ⚠️ **A CENA refutou a própria prosa duas vezes:** *"o suspenso passa nivelado"* mediu o oposto (23,4° contra 14,5° — duas suspensões independentes deixam uma comprimir enquanto a outra estende, que é o **mergulho** de um carro de verdade), e o SOLAVANCO varreu cinco alturas de lombada com melhor separação de **1,35×**, pequena demais para se ver; o que separa por três ordens de grandeza é o **curso** (0,150 m contra 0,000), que também é literalmente a feature. `PROJECT_SCHEMA` **38→39** (variante apendada; o bump é para o build ANTIGO recusar), registro **21** intocado, c9 **92 corpos**, `9e10ec40…` (debug ≡ release). LOC: `paint_joint_section` 201 ⇒ `paint_kind_params`; `inspector_joint_tests.rs` 606 ⇒ irmão `inspector_joint_kind_tests.rs` (*o clique produz um joint que SEGURA* × *o joint nasce com os números do TIPO*) | `=57` |
| **W-Pulley** ([plano 02 §8.1](02_plano_joints_ui_authoring.md)) | **A POLIA — uma corda por duas roldanas, e o primeiro vínculo do kit que NÃO é um joint do rapier.** `grep -rin pulley` sobre o `rapier2d-0.28.0/src` devolve **nada**, e o `PhysicsHooks` tem exatamente três métodos, os três sobre CONTATOS ⇒ não existe rota para injetar uma restrição própria no solver dele. Ela é imposta **de fora**, no mesmo lugar e pelo mesmo mecanismo que o arrasto, as zonas e o campo de atração: um passe de impulso por **sub-passo**. ⚠️ **NÃO é um laço de força PD** — a `world::grab` já mediu que aquilo explode no ganho equivalente a 1/60 s; é uma **projeção de velocidade com a massa efetiva EXATA do Jacobiano**, então um passo a zera e a corda segura **igual com 0,1 kg ou 100 kg** (medido: mesmo esticamento). ⚠️ **A tabela do `PULLEY_BIAS` foi escrita ANTES de medir e estava errada em todos os números e na conclusão** (afirmava 0,1462 m onde o real é 0,0043, e oscilação em β=1 que não existe) — as duas varreduras dizem que β **não é knob de estabilidade**: o erro cai como `1/β`, zero tremor em toda a faixa, e contra um CONTATO o resultado é idêntico de 0,05 a 2,0. **0,20 fica porque 1,1 mm já é menor que a tolerância de repouso do próprio rapier** (1,3 mm) — o teto legítimo é o da engine, não o da tabela. ⚠️ **BUG REAL que o gate pegou na primeira corrida:** um corpo `Fixed` reporta `effective_inv_mass = 1.0` — o rapier só zera massa por `LockedAxes`, **nunca pelo TIPO** — então uma parede entrava na conta com massa finita e a corda amarrada nela ficava **5,8× mais frouxa**; um corpo não-dinâmico agora contribui `k = 0`, e o `rate` dele **não** é zerado junto, o que faz um corpo **kinematic virar um GUINCHO** de graça (com o termo o atraso é constante na velocidade, 0,00053 m; sem ele cresce com ela). ⚠️ **TRÊS portas que compartilhavam resposta se separaram** — `shares_a_point` deixou de ser `!has_length()` · `length_is_a_radius` nasceu (o comprimento de uma polia é a SOMA dos dois ramos, então um anel em volta de uma âncora descreveria uma distância que não existe — e o publicador do anel **já ia pintá-lo**, a mesma classe do bug do trilho de 26/07) · `can_break` nasceu (nada mede a reação de algo fora do `ImpulseJointSet`, então a caixa seria um limiar que nunca dispara). ⚠️ **`joint_views` itera os joints VIVOS do rapier, então a polia seria INVISÍVEL** — o reconcile deixa um `PulleyRecord`, uma lista com duas leituras (a tabela do solver e o DESENHO), e a corda é desenhada como ela é: sobe até uma roldana, atravessa, desce até a outra ponta. As roldanas são agarráveis no canvas (`PointHandleKind::WheelA/WheelB`, gizmo ids 969/970) porque **não há row de Inspector para elas** e sem alça seriam semeadas e imutáveis; **sem ímã**, porque uma roldana não pertence a corpo nenhum. ⚠️ **A CENA refutou a própria prosa (3ª vez na jornada):** eu afirmava que razão 2 faz o leve erguer o pesado, e medido ele não só perde como a carga **cai o DOBRO** — com `l1 + r·l2 = L0` o lado B anda `1/r` e precisa pesar `r` vezes mais, então a vantagem de B é `1/r` e vem de `r` **menor** que 1; a cena usa **0,25** (o contrapeso de 1 kg desce 4,51 m e ergue 3 kg em 1,13 — quatro vezes mais caminho por três vezes o próprio peso, que é o que uma talha TROCA), e a doc do `ratio` nos dois lados estava invertida. ⚠️ **A dependência que o plano §8 declarava (*'a polia e o Pin-to-world andam juntas ou nenhuma anda'*) estava PAGA** — ela guarda os próprios pontos de mundo, e o gizmo de PONTO já existia desde a W-JointAnchor. `PROJECT_SCHEMA` **39→40** — e aqui o bump **não é cortesia**: são CAMPOS apendados a um struct posicional, então um blob v39 tem o comprimento errado. Registro **21** intocado, c9 **94 corpos**, `88c6e49d…` (debug ≡ release). 17 gates + 2 tabelas de porta exaustivas; **9 mutações, 9 sangram** (duas sobreviveram à 1ª rodada por defesa em camadas e ganharam o gate que só elas veem). LOC: quatro tetos, quatro splits por responsabilidade | `=58` |
| **W-Pulley W0..W3** ([plano 03](03_plano_polia.md)) | **O REDESENHO da polia**, sobre oito pontos de um smoke do Enio. **W0** as quatro correções da foto (a criação pelo canvas nascia com o semeio do RIG atrás do MESMO sentinela das âncoras ⇒ as roldanas ficavam na ORIGEM do mundo; o anel de comprimento perguntava `length.is_some()` quando numa polia o comprimento é a CORDA e não um raio; o readout `0 / 0 N` permanente; e a row Ratio **morta** — faltavam registro, sync, rota, variante e campo de uma vez). **W1** ⚠️ **o `ratio` SAIU por ser física errada** — numa corda única sobre roldanas livres a tensão é **uniforme**, logo os dois corpos sentem a MESMA força e a vantagem é 1, quaisquer que sejam os diâmetros; o que ele descrevia era uma talha DIFERENCIAL com os tambores invisíveis. No lugar dele: **uma roldana é uma ENTIDADE** (`PulleyWheel` + `Transform`), com RAIO, rota de N nós tangenciando a SUPERFÍCIE (tangente comum ponto↔círculo e círculo↔círculo — um ponto é um círculo de raio zero, e a fórmula já o contém), arco no comprimento e **não** no Jacobiano (teorema do envelope), lado por ponto fixo (**MEDIDO: 1 passada** em 18 montagens) com escape `Auto|Over|Under`, giro `ω = s/r`, botão Add Wheel e a §13. **W2** o **MOTOR** (uma roldana dirigida é um GUINCHO: `L0` encolhe a `Σ ω·r`; ⚠️ um alvo de VELOCIDADE não serviria — com `λ ≥ 0` pagar corda é clampado em zero e o guincho sobe mas não desce; o teto `PULLEY_CORRECTION_LAG = 6` é MEDIDO e **relativo à taxa do tambor**, com três guardas medidos e REJEITADOS) e a **RUPTURA** (⚠️ **UM** limiar para as duas pontas — a corda é inextensível, logo a tensão é uniforme e dois números contra uma carga são um limite e um controle inerte; o que difere é o **EIXO** de cada roldana, `T·|u_saída − u_entrada|`, 2T num enlace de 180°). **W3** a **TALHA** — a roldana montada num corpo que se move: o eixo vira **mais uma ponta da mesma restrição** e a vantagem mecânica volta **sem um número** (medido: um bloco de 2 kg equilibra com **1,00 kg** na talha e **2,00 kg** amarrado direto). ⚠️ **A primeira fixture não tinha a cadernal FIXA e a medição a derrubou** (sem ela os dois lados liberam energia e não existe equilíbrio nenhum a medir). `PROJECT_SCHEMA` 40→**44**; c9 **94 corpos**, `52767c92f7…` | `=58` · `=59` · `=60` · `=61` |
| **PLANEJADAS — [`02_plano_joints_ui_authoring.md`](02_plano_joints_ui_authoring.md)** (2026-07-25, pós-pesquisa Unity/Unreal/Godot/Fyrox/RUBE/Algodoo/Newton + rapier source; 44 screenshots em `~/Documentos/Recursos/UI_Reference/`) | **W-J1** o joint se DESENHA (glifo/posse/limites/rest/violação) · **W-J2** duas alças + snap · **W-J3** pose-não-digite (arco/anéis/seta arrastáveis) · **W-J4** criar onde se olha (press-A-drag-B; corrente por seleção ordenada) · **W-J5** Slider/prismatic · **W-J6** servo + guincho · **W-J7** break force · **W-J8** Active/Collide/Swap/nome "A : B" · **W-JG** grupo carrega o rig. Absorve as ex-"waves 2-5" do padrão-ouro | — |

⚠️ **Esta tabela estava faltando até 2026-07-21**, e um plano *normativo* que não menciona metade do módulo é
pior que um plano velho: ele faz a próxima LLM concluir que a linha parou no W5. A regra: **wave nova fora do
mapa entra AQUI na mesma sessão**, com uma linha; o detalhe fica no tracker.

## ⚠️ Toda wave chega à UI — a política, não a boa intenção

Pergunta do Enio no fim da jornada de 2026-07-21 (*"tudo isso está exposto na UI e é possível criar essas cenas
todas usando apenas os parâmetros expostos?"*). A resposta foi **sim**, mas a pergunta expôs que isso era um
hábito e não uma regra. Agora é regra, e vale para **toda wave futura desta linha**.

### O que "chegar à UI" significa, em quatro condições

Uma wave só fecha quando o que ela construiu é alcançável por um artista **sem escrever código**:

1. **Existe** — todo componente registrado tem um caminho de escrita a partir do Inspector.
   Gate: `shells/desktop/tests/every_physics_component_is_authorable.rs` (estrutural, sobre o fonte).
2. **É pintado e registrado** — o controle aparece e é focável.
   Gate: `architecture_panel_wiring_parity`.
3. **O clique chega ao barramento** — cada row/chip despachado, com a recusa no `event`, nunca no laço de pintura
   (*dim não é recusa*). Gate: a **varredura** de seam do painel, que clica **todos** os controles da seção.
4. **A SEQUÊNCIA leva a algum lugar** — o gesto composto produz uma coisa que funciona.
   Gate: `inspector_physics_gesture_tests`.

⚠️ **A (4) é a categoria que esta jornada descobriu, e ela não é implicada pelas outras três.** Todo edit pode ter
gate e o gesto ainda não levar a lugar nenhum: uma row que só aparece depois de outra, um default que atrapalha,
um passo que exige um número que o artista não tem como saber. Foi ela que pegou o passo *"converta para
Capsule"* que eu quase ensinei ao Enio — geometricamente correto, e destrói o tronco.

### A metade VISÍVEL conta como UI

Um controle autorável cujo efeito é invisível está meio construído. A precedência é do **W7** (*um sensor com
nada lendo suas sobreposições é um flag morto — torne-o VISÍVEL primeiro*) e ela se repetiu quatro vezes:

- força de área → **seta laranja** (*para que lado sopra?* não é inferível);
- contatos → **cruz branca**, do tamanho da carga;
- empuxo → **linha d'água** (o único número que o modelo calculava e a tela escondia — achado pelo Enio);
- arrasto → **nada, e é decisão**: um arrasto não tem direção para desenhar, ele se vê nos corpos desacelerando.

A pergunta a fazer no fim de cada wave é *"o que esta wave calcula que a tela não mostra?"* — e a resposta
**pode** ser "nada, de propósito", desde que seja escrita.

### E toda wave ganha uma CENA

`PH2D_PHYSICS_SMOKE=<n>`, com os números **medidos** (a sonda headless roda a cena e reporta; a mensagem
`eprintln!` cita os valores). Uma wave gateada e não-smokável é meia wave — foi o estado do W-FormDrag por uma
hora, e a cena `=28` nasceu para fechar isso.

⚠️ **A cena é uma FIXTURE e adoece como fixture.** Nesta jornada: dois controles foram *atropelados pelo próprio
experimento* (W-Area, W-Buoyancy), um V nasceu de cabeça para baixo (W-Contacts), o `=28` nasceu contaminado
**duas vezes** por geometria que eu não controlava, e uma mensagem afirmava *"fica a meia-água"* sobre uma caixa
que a medição mostrou ir ao **fundo**. **Rode a sonda antes de escrever a mensagem.**

### O que fica FORA da UI, e por quê

Um número que o artista não tem como calibrar não vira knob. Da jornada: o `EDGE_SAMPLES` do arrasto de forma,
o `LOAD_FULL_NS` da cruz de contato, o `ALLOWED_ANGLE` do one-way, o `STRIDE` do ring — todos são **régua de
implementação**, medidos e documentados no código, não superfície. A pergunta é *"o artista sabe o que este
número significa na arte dele?"*.

---

**Fora de TODAS as waves (D9):** soft-body XPBD (`ph2d-physics-soft`, M13+), fluidos FLIP/PIC
(`ph2d-fluids`, M13+), collider-gen vetorial + fratura (ADR-0063, aposentada com a 0108).

---

## W1 — Ponte ECS + tick no Playhead + hash no replay gate · *o alicerce*

**Objetivo:** um sprite com `RigidBody{Dynamic}` cai e assenta sobre um `Collider{Static}` no **ECS
REAL** ao dar play — e o mundo é **determinístico cross-OS**.

### Entregáveis
- **Crate-ponte nova `ph2d-physics-ecs`** (ou módulo no editor-core; a crate isola melhor — regra B'):
  - Components `RigidBody`/`Collider` + enums `BodyKind`/`ColliderShape` (append-only, defaults
    byte-neutros — ADR D3).
  - `register_physics_components(reg: &mut ComponentRegistry)` — a crate possui, o boot agrega em
    `shells/desktop/src/init.rs` ao lado de `register_render_components` (mantém a contagem-32 de
    `ph2d-ecs` intocada). **Registro no MESMO commit que cria os components.**
  - **Sem porta de escala:** o `Transform` já é METROS = rapier metros (1:1, sem conversão nem sinal
    trocado — os dois são Y-up + radianos CCW). A única conversão px→m é a que JÁ existe,
    `ProjectSettings.pixels_per_meter` no import — do projeto, não da física (ADR D4 corrigido no W1).
- **System de sync (o hot path `physics_step`):** components → `PhysicsWorld` (spawn/update do
  handle-map `Entity ↔ RigidBodyHandle`) → `step()` no tick do `Playhead`
  (`ticks_owed(last_stepped, target)`: play = `last+1..=target` sequencial, scrub/paused =
  `target..=target`; `target = round(playhead.time()/fixed_dt)`) → **readback** dos transforms para o
  `SimWorld`. O `PhysicsWorld` + handle-map vivem shell-side (precedente `MotionCookPump`), **NÃO** no
  `WorldSnapshot` (o mundo é rebuild das components — ADR D2). Gancho `should_record`/`record` do ring já
  no laço (W1.5 o usa).
- **Persistência mínima:** as components viajam no `WorldSnapshot` (já registradas) → bump
  `PROJECT_SCHEMA` (**15 → 16**, valor real; +a tripla-pin em `project_tests`). O `PhysicsWorld` é
  reconstruído no load (`rebuild()`; reconcile self-heal é o backstop).
- **Gate de determinismo estendido:** um bin/harness gêmeo `physics-ecs-c9` que exercita **a ponte + o
  caminho do tick** (não o wrapper cru): monta uma `SimWorld` com N entidades carregando
  `RigidBody`/`Collider`, roda sync + `ticks_owed` por 120 ticks, imprime `physics-ecs-c9 hash: <hex>`.
  Plugar em `.github/workflows/spike.yml`: etapa de matriz (ubuntu/macos/windows) + artifact
  `physics-ecs-c9-hash-${os}` + comparação `sort -u | wc -l == 1` no job `determinism-compare`.

### Gates (red-first, mutation-tested)
1. **e2e no app REAL** — sprite com `RigidBody{Dynamic}` sobre `Collider{Static}` **cai e assenta no
   chão** dirigindo o `SimWorld` + a ponte + N ticks do `Playhead` (NÃO um unit do wrapper —
   [[feedback_tool_unit_green_integration_dead]]). Nasce vermelho (sem ponte, o sprite não cai). Assenta
   a `y ≈ chão + raio` em pixels (converte via a porta, prova a escala de ponta a ponta).
2. **hash cross-OS estável do mundo ECS-bridged** — `physics-ecs-c9` byte-idêntico nos 3 OSes. **Mutação:
   trocar a ordem de iteração da ponte (map em vez de sorted) sangra** (o hash muda). Prova o código NOSSO
   no caminho determinístico.
3. **zero-alloc no `physics_step`** — dhat **por capacidade**, não contador global
   ([[feedback_zero_alloc_gate_capacity_not_global_counter]]). Mutação: um `Vec::push` que realoca no laço
   sangra.
4. **tick único** — play anda N steps, scrub anda 1. **Gate de emenda com advance FRACIONÁRIO** (taxa 1:1
   nunca lê o 2º frame — [[feedback_seam_gates_need_fractional_advance]]): um `wall_dt` que deve 2 ticks
   tem que simular os 2; um scrub tem que rodar `anchor..=target` uma vez. Mutação: `last+1..=target` →
   `target..=target` (perde ticks no play) sangra; e vice-versa.
5. **snapshot é ponto fixo** — parado (sem input), **nenhum passo de undo espúrio por frame**
   ([[feedback_a_snapshot_must_be_a_fixed_point_of_the_systems]]). A captura é DEPOIS de a ponte convergir.
   Mutação: capturar antes do readback sangra (pose muda entre captura e convergência).

### Smoke
`PH2D_PHYSICS_SMOKE=1` — cena **auto-play** que dropa 1 sprite sobre um chão (exemplo pronto pra smoke,
auto-play — [[feedback_ready_to_smoke_example]]). Comando com o `cd <worktree> &&` junto
([[feedback_run_command_include_cd]]).

### Fora de W1
Painel, joints, bake, scrub-back (o ring é W1.5; W1 deixa só o gancho).

---

## W1.5 — Scrub bit-exato (checkpoint ring) · *o relógio pra trás*

**Objetivo:** arrastar o playhead pra trás re-simula bit-exato **sem** custo O(t) (ADR D5). Pode vir
depois de W2 se o Enio priorizar a autoria; é listada aqui por ser a metade que falta do "relógio único".

### Entregáveis
- **`PhysicsCheckpoint`** = estado cross-frame completo do `PhysicsWorld` (os campos que `step()` muta:
  `bodies`/`colliders`/`impulse_joints`/`multibody_joints`/`ccd_solver`/`islands`/`broad_phase`/
  `narrow_phase` + `step_count`).
- **`PhysicsCheckpointRing`** à imagem do `ph2d-eval-motion::CheckpointRing`: `record(tick, cp)`,
  `anchor_at_or_before(target) → (tick, cp)` (newest ≤ target, senão seed do tick-0), `clear` no rebuild.
  **Cadência ESPARSA** (cada K ticks — o estado é maior que outputs de nó), K tunado contra o budget 20 MB.
- **`advance_or_scrub`** no laço do tick: play = `record` esparso + step forward; scrub-back = `restore` da
  âncora + re-sim ≤ K steps até `target`.

### ✅ kill-check RESOLVIDO (2026-07-18) — passou de primeira
Os 8 tipos cross-frame do rapier **são `Clone`** ⇒ sem `serde-serialize`, sem bincode. O `PhysicsPipeline`
não é `Clone` porque é *workspace*, não estado (confirmado pelo gate de bit-exatidão, não por prosa).
**Cadência decidida por MEDIÇÃO** (`measure_checkpoint.rs`): um checkpoint custa ~**um step** (11,2 µs vs
7,3 µs a 50 corpos), então denso dobraria o custo do play e comeria 17,4 dos 20 MB ⇒ **`STRIDE = 10`**
(1,74 MB de janela, pior caso 10 steps). **Cap em BYTES** (8 MB), nunca em contagem — contagem é
multiplicador (ADR-0117). Detalhe e números: tracker §W1.5.

### Gates (red-first, mutation-tested)
1. **scrub-back é bit-exato** — `restore(anchor) + re-sim até T` produz o **mesmo hash** que `re-sim from
   t=0 até T` (a definição de correção do ring). Mutação: `anchor_at_or_before` devolvendo a âncora errada
   (> target) sangra.
2. **memória do ring medida** — dhat/`size_of`, `tests/measure_physics_checkpoint.rs`, dentro dos 20 MB
   (HR-13, quem declara MEDE — [[feedback_a_rule_that_never_observes_cannot_fire]]). Mutação: cadência densa
   (K=1) estoura o teto.
3. **scrub é O(K), não O(t)** — ratio: scrub num t grande custa o mesmo que num t pequeno (bar é RATIO, não
   wall-clock — `ci-test` compila `opt-level=1`).

### Smoke
**`PH2D_PHYSICS_SMOKE=2`** (cena própria, não a 1 — a do W1 foi aprovada pelo Enio e não se mexe no que já
foi validado): 12 corpos caem numa **pilha**, o playhead se arrasta pra trás e pra frente, a pilha
reconstrói bit-exata sem trava. As cenas seguintes deslocam em 1: **W2 = 3 · W3 = 4 · W4 = 5.**

### Fora
Painel, joints, bake.

---

## W2 — Painel global + Inspector body · *a autoria*

**Objetivo:** o artista liga/desliga a física, seta gravidade/escala no painel de mundo, e edita
massa/restituição/atrito/tipo num sprite selecionado.

### Entregáveis
- **`ph2d-panel-physics` docado (categoria MUNDO — ADR D8):** gravidade (vetor), substeps/iterações do
  solver, damping global, sleep thresholds, **matriz de camadas de colisão** (a escala do mundo é
  `ProjectSettings.pixels_per_meter`, setting do projeto — o painel exibe, não duplica). Tokens +
  i18n (zero hex/`f32`/string hardcoded; inglês). Registrado nos **5 sites** (precedente
  `ph2d-panel-vector`):
  1. `impl Panel` — `ID="physics"`, `NODE_ID=ids::PHYSICS_PANEL` (próximo IconId/panel-node livre, anotar),
     `DEFAULT_VISIBLE=false`, `populate`/`paint`/`apply_event`.
  2. push no `ph2d-panel-registry-init` (GERADO por `cargo run -p ph2d-panel-sync`) + a const
     `EXPECTED_TYPED` à mão.
  3. feature Cargo `panel-physics = ["dep:ph2d-panel-physics"]`.
  4. **a lista de fallback de z-order em `hero/paint.rs`** — sem a entrada, o painel registrado+visível
     **NUNCA é pintado** (a armadilha "never painted").
  5. visibilidade dirigida pela ponte (`hero.panel_visibility.insert("physics", ...)` no `render_loop`).
- **Seção "Physics Body" no Inspector (por-seleção):** type (dynamic/static/kinematic), massa/densidade,
  restituição, atrito, collider-shape. NÃO no painel global. NumberInput com range/clamp const
  ([[reference_topic_panel_registration]]).

### Gates (red-first, mutation-tested)
1. **painel pintado E populado E clicado** — um teste do `ph2d-ui-testkit` que **DIRIGE o clique** em cada
   row e afirma o efeito ([[feedback_widget_is_done_when_a_test_clicks_it]] +
   [[feedback_painted_is_not_populated_paint_gate]]). Nasce vermelho (sem `populate`, o WidgetStore está
   vazio e não há Click).
2. **toda row de setting muda o mundo** — seam que CLICA: mexer na gravidade **muda a aceleração dos
   corpos**; mexer nos substeps/damping **muda a simulação**. Mutação: um arm que não chama
   `apply_ui_edit` (fio órfão) sangra. Varre **cada** row (não "o card mais cheio" —
   [[feedback_the_fullest_card_premise_rots]]).
3. **sem string hardcoded** — gate i18n; todo label resolve via chave `panel.physics.*`.
4. **botão dimmed recusa no `event.rs`** — dim é cosmético ([[feedback_disabled_button_still_dispatches]]);
   editar body sem seleção é no-op explícito (`debug_assert`/`warn`), não corpo vazio.

### Smoke
`PH2D_PHYSICS_SMOKE=3` (W2a, autoria no Inspector) · **`=4` (W2b, o painel de mundo)**.

### Fora
Joints, bake.

---

## W2c — Camadas de colisão · *quem colide com quem*

**Por que NÃO entrou no W2b:** a matriz é metade de uma feature. A outra metade é
**a camada de cada corpo**, que é campo de component (bump de `PROJECT_SCHEMA` +
`ComponentRegistry`) e UI do **Inspector** — a superfície do W2a, já fechada e smokada.
Uma matriz sem atribuição por-corpo é uma matriz 1×1: todo corpo na camada 0, uma única
célula viva, e as outras 255 são chrome que não faz nada. *"Botão que não faz nada é pior
que botão que falta."*

Duas fricções reais que a wave tem de resolver de propósito, não por acidente:
- o gate `architecture_panel_wiring_parity` **não enxerga registro dentro de laço**, e uma
  matriz É um laço — os ids são dinâmicos (`hash` por par de camadas). Precisa do gate
  irmão que o Painter/Flip já têm para ids dinâmicos (`*_dynamic_ids_dont_collide_*`).
- mudar a matriz muda o `InteractionGroups` de **todo collider vivo**, então ela entra no
  mesmo choke point das outras settings (`set_settings` → aplica + limpa o ring), e o
  `BodyDesc` ganha `memberships`/`filter`.

### Entregáveis
- `Collider.layer: u8` (append-only) + a linha no Inspector · `PhysicsSettings.layer_matrix`
  ([u16; 16], triangular na UI como Unity) · `BodyDesc.memberships/filter` → `ColliderBuilder::collision_groups`.
- Bump `PROJECT_SCHEMA` + a tripla-pin.

### Smoke
`PH2D_PHYSICS_SMOKE=5` — dois grupos, duas camadas, um chão.

### Gates
1. **dois corpos em camadas que não colidem se ATRAVESSAM** (oráculo de aparência: o de
   cima chega ao chão). Mutação: a matriz ignorada → colidem → vermelho.
2. **mudar a matriz alcança colliders que já existem** (o irmão do
   `the_defaults_reach_bodies_that_already_exist`).
3. **a matriz é simétrica** — A colide com B ⟺ B colide com A. Uma matriz que pode ficar
   assimétrica tem dois donos para um fato.

## W3 — Joints · *as articulações*

**Objetivo:** pino/mola/motor/distância entre corpos; pêndulo, corrente, ragdoll simples.

### Entregáveis
- Components de joint (registrados no `ComponentRegistry` — append-only), autoria no Inspector/canvas
  (gizmo de ancoragem), mapeamento para `ImpulseJointSet`/`MultibodyJointSet` do rapier (acesso cru via
  `bodies_mut`/`colliders_mut` do wrapper). Determinismo preservado (mesma proibição de simd/parallel).
- ~~Bump `PROJECT_SCHEMA` (**21 → 22**)~~ — **NÃO acontece, e a contagem é que decide** (*"o valor se CONTA, não se escolhe"*). O blob de um componente no snapshot é chaveado por `stable_type_id = blake3(nome_canônico)[..8]`, derivado do **NOME** e não de uma posição no registry: registrar `ph2d::physics::PhysicsJoint` cunha um id novo e **não move nada**. É o oposto do W2c, que apendou `layer` DENTRO do `Collider`, onde postcard é posicional e o bump era obrigatório.
  Bumpar assim mesmo não é neutro: um schema divergente **recusa o arquivo inteiro** (`project.rs`), então jogaria fora todo projeto já salvo — para melhorar a mensagem de erro na única direção que não funciona de qualquer jeito (um build ANTIGO lendo um arquivo com joints). O raciocínio está falsificável em `crates/ph2d-physics-ecs/tests/joint_persistence.rs`: se uma mudança futura de fato mover o layout, o 1º gate fica vermelho e o bump passa a ser devido. **`PROJECT_SCHEMA` segue em 21.**

### Gates (red-first, mutation-tested)
1. **pêndulo de 2 corpos determinístico** — hash estável cross-OS (estende `physics-ecs-c9` com uma cena de
   joint, ou um segundo hash). Mutação: trocar a ordem de inserção dos joints sangra.
2. **joint sobrevive save/load** — schema bump provado por round-trip (grava, carrega, re-simula, mesmo
   hash). Mutação: joint não-registrado no `ComponentRegistry` some do snapshot (a armadilha D3) — o
   round-trip sangra.
3. **mutação de um parâmetro de joint sangra o gate de repro** — mudar stiffness/rest-length muda a
   trajetória; o oráculo de aparência pega.

### Smoke
`PH2D_PHYSICS_SMOKE=6` — pêndulo/corrente auto-play.

### Fora
Bake.

---

## W4 — Bake-to-timeline · *runtime-truth vira animação*

**Objetivo:** o botão "Bake" amostra a sim sobre um range e escreve keys editáveis nas tracks da entidade
— a metade motion-graphics do framing (ADR D11).

### Entregáveis
- Amostragem determinística da pose por frame → **`ph2d-anim::fit_fcurve`/Schneider** (colunas alinhadas,
  pré-filtro passa-baixa se preciso), **1 passo de undo**, via a ponte da timeline/anim. A costura:
  `sim → amostra por tick → fit_fcurve_at → Track::simplify_range → 1 undo step`. Reusa a máquina do record
  da timeline (W5), não reinventa.
- Botão "Bake" no painel/Inspector, range = seleção da timeline.

### Gates (red-first, mutation-tested)
1. **curva assada reproduz a sim dentro da tolerância** — **oráculo de APARÊNCIA** (posição no tempo certo,
   não uma fórmula — [[reference_topic_oracle_discipline]]). Nasce vermelho sobre uma curva que não segue a
   trajetória. Mutação: amostrar no relógio errado (playhead cru vs tick) sangra.
2. **bake é determinístico** — mesma sim → mesma curva (D7). Mutação: um transcendental sem convenção única
   no fit sangra o hash da curva.
3. **1 undo step (não 1 por frame)** — [[feedback_capture_stroke_session_before_pen_up]] análogo: a sessão
   de bake é UM passo. Mutação: 1 key/frame sem simplify vira 1 undo/frame — o gate conta os passos.

### Smoke
`PH2D_PHYSICS_SMOKE=7` — rampa + bola que rola + duas caixas, relógio PAUSADO: seleciona, assa, e dá play.

⚠️ **O bake não "desliga a física": ele ENTREGA a pose.** O apply da timeline escreve o `Transform` e o
readback da física escreve **depois**, então um corpo dinâmico recém-assado é sobrescrito pelo solver todo
frame e o artista veria o botão Bake não fazer nada. Por isso o bake vira `BodyKind::Kinematic` — o corpo
continua no mundo e continua empurrando, mas o movimento vem da curva. É o que *runtime-truth vira
animação* quer dizer. Ver §W4 do tracker.

⚠️ **CORREÇÃO (2026-07-18, W4b):** este parágrafo dizia também *"o desligamento manual seria o desenho
errado de qualquer jeito"* e **isso passou do ponto**. Ele respondia *"o Bake deve desligar a física no
corpo assado?"* (não — ele entrega a pose, pelo motivo acima) e enunciou a resposta como verdade sobre
**qualquer** interruptor. São duas perguntas. A outra — *"o Play tem de dirigir o solver?"* — é do
TRANSPORTE, não do corpo, e a resposta é **sim, o artista escolhe**: o Enio reportou o conflito (*"os
controles de simulação e de animação parecem ser os mesmos … a simulação roda junto com a animação"*) e a
wave **W4b** pôs o toggle **Physics** na barra da timeline, **desmarcado por padrão**. Registro completo
(incluindo por que esta nota enganava) em [`BUGS_physics.md`](BUGS_physics.md) #1. As duas decisões
convivem sem se tocar: o toggle diz se o solver **roda**, o `Kinematic` diz quem **escreve a pose** quando
ele roda. Ver §W4b do tracker.

### Fora
Soft-body, fluidos, collider-gen vetorial, fratura (M13+ / linhas próprias).

---

## Convenções do módulo (valem em todas as waves)

- **Inner loop:** `cargo check -p ph2d-physics-ecs` (ou `ph2d-physics`). Teste/clippy/auditoria **1× no
  fechamento** da wave, sobre o diff acumulado. Workstation voa — rust-analyzer full como oráculo.
- **LOC cap (HR-18):** shells/foundational = 600 LOC/arquivo; campo/mod novo que estoura → **split em módulo
  irmão**, não allowlist. `cargo fmt` re-expande → fmt ANTES de medir.
- **Determinismo:** NUNCA ligar `parallel`/`simd-*` no rapier. Todo transcendental no código NOSSO com
  convenção única; 1 ulp já é bug.
- **Ids/consts/variants novos:** próximo livre, **anotados no tracker** (`HANDOFF_line_physics.md`) para o
  integrador grepar mesmo-símbolo (§1.5.9).
- **Fechamento de wave = gate batched verde + handoff de tracker atualizado. Então PARE.** Integração e ship
  só por ordem EXPLÍCITA do Enio, via agente integrador dedicado (regra E/F).
