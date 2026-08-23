//! **A ESCADA do `PROJECT_SCHEMA`** — o número do formato de arquivo, e como
//! ele chegou onde está.
//!
//! ⚠️ **Corte por RESPONSABILIDADE, e por LOC:** o irmão [`super::project`]
//! responde *"o que um arquivo de projeto contém, e como ele vai e volta do
//! disco"*; este responde *"que versão ele é, e por que"*. O `project.rs`
//! cruzou o teto de 600 do HR-18 com o degrau v77, e a escada é a metade que
//! cresce **um parágrafo por wave** — separá-la é o corte que não volta.
//!
//! ⚠️ **A escada mora COLADA à constante de propósito**, e a razão está escrita
//! no degrau v69: ele chegou ao `main` com a linha da escada AUSENTE, e *quem
//! conta o próximo degrau lê a escada, não o literal*. Mover as duas juntas
//! preserva isso; mover só o literal seria o defeito outra vez.
//!
//! ⚠️ **E o valor se CONTA contra o `main` do dia, nunca se escolhe** — esta
//! colisão passa **muda** quando duas linhas escrevem o MESMO número, porque o
//! git não sabe o que ele significa.

/// Versão do formato de arquivo de projeto. Bump ⇒ migração ou hard-break.
/// v2 (ADR-0114): `ProjectState` ganhou o campo `flip: FlipDoc` (3º) — postcard é
/// posicional, então um arquivo v1 não desserializa. Sem custo real: a
/// persistência ainda é stub (sem diálogo de arquivo), sem saves publicados.
/// v3: `ProjectFile` ganhou os **documentos do Painter** (3º campo). Sem eles o projeto salvava um
/// sprite apontando para uma textura de runtime que morre com o processo — pintar, salvar e reabrir
/// devolvia o quadro em branco. Ver [`crate::project_painter`].
/// v4: o `Layer` do Painter ganhou o **Impasto por camada** (`impasto_depth` / `impasto_composite` /
/// `has_relief`) — postcard é posicional, então um documento pintado v3 lê lixo nos campos seguintes.
/// Rejeitar é a única leitura honesta.
/// v5 (doc 56): `ProjectFile` ganhou `motion` (o grafo de Motion Nodes, em texto) — 4º campo. Pelo
/// mesmo motivo posicional, um v4 não desserializa aqui.
/// v6 (ADR-0114 W3): a `FlipDoc` — que vive DENTRO do `ProjectState` — mudou de forma: a camada
/// ganhou `cycle` + `use_onion` e o `OnionSettings` ganhou `kind_filter` (`FLIP_SCHEMA_VERSION` 1→2).
/// Não é campo novo no ARQUIVO, é o MESMO campo com outro layout — e posicional é posicional.
/// v7 (ADR-0114 W4): o `FlipStroke` ganhou `holes` + `hide_stroke` (o balde —
/// `FLIP_SCHEMA_VERSION` 2→3). Mesma regra: a forma mudou, então a versão sobe.
/// Sem o bump, um arquivo v6 passaria na checagem de versão e seria lido com o
/// layout NOVO — postcard não tem nomes de campo para reclamar, e o que sai é
/// geometria embaralhada em vez de um erro honesto.
/// v8 (ADR-0114 W6): o `FlipStroke` ganhou `selected` — a seleção é ATRIBUTO do traço
/// (o Edit Mode; `FLIP_SCHEMA_VERSION` 3→4), e não estado do shell. Idem: a forma do
/// `FlipDoc` mudou dentro do `ProjectState`, então o par sobe junto.
/// v9 (ADR-0114 W7.2): a CHAVE (`FlipFrame`) ganhou `offset` — a **pose do quadro**
/// (`FLIP_SCHEMA_VERSION` 4→5). É o que faz uma instância (duas chaves, um desenho) ser
/// mais que um hold: a arte é compartilhada e o lugar é de cada quadro.
/// v10 (ADR-0121): o `VecVertex` ganhou `corner_radius` (Live Corners —
/// `ph2d_vec_scene::corner_live`), e a `VecScene` vai embutida aqui
/// (`VEC_SCENE_SCHEMA_VERSION` 7→8). Mesma regra.
/// v11: o `PaintedDocument` ganhou `mats` — o MATERIAL do Impasto por camada
/// (Roughness/Metallic/Wax/Shine, por pixel). Sem o bump, um save anterior seria lido com o
/// layout novo e o material sairia dos bytes da COBERTURA. (O `Shine` deixou de ser global e
/// virou propriedade da TINTA — Enio, 2026-07-13.)
/// v12: o MESMO `mats` mudou de FORMA — 4 bytes → 7 (a **cor do Wax**, o filtro sobre a luz que
/// atravessa a tinta). Não é campo novo, é o mesmo campo com outro layout, e posicional é
/// posicional: sem o bump um v11 passaria na checagem e o material sairia dos bytes errados.
/// v13 (W4.T6/B5): `ProjectFile` ganhou `timeline` (o `TimelineDoc` em postcard) — 5º campo.
/// **A animação era perdida ao fechar o app**: nada a salvava (o "sidecar" que dizia salvá-la
/// era código morto — o Ctrl+S global já retornava antes). Os bytes trazem a própria versão
/// (`DOC_VERSION`), então um bump lá é RECUSADO com erro honesto e não obriga a bumpar aqui;
/// o campo NOVO, sim, obriga (posicional).
/// v14 (W7.5): a **pose da chave** do Flip virou AFIM (`FlipFrame.pose: Pose([f32;6])`, era
/// `offset: Vec2`) — o `ProjectState` embute o `FlipDoc`, e postcard é posicional: 8 bytes → 24
/// por chave posada. Sem o bump um v13 leria os coeficientes do afim como o `Vec2` + lixo.
/// v15 (W8): o traço do Flip ganhou `point_sel` (seleção no domínio Point, FLIP v6→7) —
/// campo novo no `FlipStroke`, layout posicional muda.
/// v16 (ADR-0131 W1): `RigidBody`/`Collider` foram REGISTRADOS no
/// `ComponentRegistry`, então uma cena com corpos físicos grava blobs novos
/// nas linhas do `WorldSnapshot` — um leitor v15 leria esses bytes na posição
/// errada. O `PhysicsBridge` em si NÃO é serializado (é derivado das
/// components no load); só os components viajam.
/// v17 (ADR-0131 W2): o `Collider` ganhou `restitution`/`friction` APENDADOS —
/// campo novo, layout posicional muda.
/// v18 (§4.C.6 do Flip): a **UNIDADE** do `Point.width` do Flip mudou (px de tela → MUNDO,
/// FLIP v7→8). ⚠️ **O layout NÃO mudou** — e é por isso que o bump é obrigatório: postcard lê
/// o `f32` antigo com sucesso e o interpreta na unidade nova, ~100× mais grosso, sem um
/// erro sequer. Todos os bumps anteriores quebravam LAYOUT (falham alto); este quebra
/// SIGNIFICADO (falharia calado). Arquivo v17 é recusado — ver o `load`.
/// v19 (ADR-0131 W2b): o arquivo carrega as **settings de MUNDO** da física
/// (`ProjectFile.physics`) — gravidade, solver, arrasto, sono. Campo novo,
/// layout posicional muda. Sem ele o painel do W2b seria um painel de knobs que
/// ESQUECEM: gravidade zero para um jogo top-down é uma decisão do projeto, e
/// perdê-la ao reabrir é o mesmo que não tê-la.
/// v20 (ADR-0131 W2b, pós-smoke): `PhysicsSettings` ganhou `air_drag` APENDADO —
/// o campo entra no layout de `ProjectFile.physics`. Nenhuma constante de esquema
/// mudou, então **nenhum gate podia ver isto**: postcard é posicional e um save
/// v19 lido como v20 devolveria lixo bem-formado.
/// v21 (ADR-0131 W2c): camadas de colisão — `Collider.layer` APENDADO ao
/// component (blob novo nas linhas do `WorldSnapshot`) **e**
/// `PhysicsSettings.layer_matrix` apendado. Duas quebras de layout no mesmo
/// bump, nenhuma visível a um gate de constante.
/// v22 (ADR-0132): `VecPath` ganhou a pilha de efeitos (Live Path Effects). v23: a entrada da
/// pilha virou `FxEntry` (o efeito + se está LIGADO). v24: a pilha ganhou os variants
/// `Repeat`/`Twist`/`Bloat` — apender variant não move os índices anteriores, então um arquivo
/// v23 continua a ser lido CERTO; o bump existe para que o caminho inverso (um v24 aberto por um
/// binário v23) morra como erro de versão em vez de como um postcard perdido.
/// ⚠️ Estas entradas nasceram como v19..v21 na `line/Vector` e foram **renumeradas +3 na
/// integração de 2026-07-19**, porque a `line/physics` bumpou três vezes na mesma jornada e o
/// contador se **CONTA**, não se escolhe.
/// v27 (ADR-0131 W7): triggers — `Collider.is_sensor` APENDADO ao component (blob novo nas
/// linhas do `WorldSnapshot`), mesmo padrão do v21 (`layer`). Layout posicional muda: um save
/// v26 lido como v27 leria além do fim do blob do `Collider`; um v27 lido por um binário v26 é
/// recusado como erro de versão em vez de virar um postcard perdido.
/// v28 (ADR-0131, Weld): `JointKind` ganhou o variant `Weld` APENDADO (discriminante 3). Apender
/// variant NÃO move os índices anteriores, então um save v27 (Pin/Spring/Rope) continua a ser
/// lido CERTO; o bump existe pro caminho INVERSO — um save com um Weld, aberto por um binário v27,
/// morre como erro de VERSÃO em vez de como um postcard perdido no discriminante 3 desconhecido
/// (mesmo raciocínio do v24, os variants do vetor).
/// v30 (ADR-0131, gold-standard joint anchors): `PhysicsJoint` ganhou `local_a`/`local_b`/`anchored`
/// APENDADOS — a âncora deixou de ser um ponto de MUNDO re-derivado (o `Transform` do joint) e
/// passou a ser autorada BODY-LOCAL por corpo (a rep nativa do rapier), pra a âncora seguir o
/// corpo quando ele se move. Layout posicional muda (mesmo padrão do v27/`is_sensor`): um save v29
/// lido como v30 leria além do fim do blob do `PhysicsJoint`; um v30 lido por um binário v29 é
/// recusado como erro de versão.
/// v31 (Flip, 03 §8): o `FlipStroke` ganhou `tip` + `dot_spacing` (o pincel pontilhado) — campos
/// no MEIO do struct (após `hardness`), então o layout posicional do `FlipDoc` embutido muda. É o
/// mesmo motivo dos bumps anteriores do `FlipStroke` (v7 `holes`/`hide_stroke`, v8 `selected`):
/// `FLIP_SCHEMA_VERSION` 8→9, e `PROJECT_SCHEMA` acompanha porque o `FlipDoc` viaja DENTRO do
/// `ProjectState`.
///
/// ⚠️ Este bump nasceu `30` na `line/FLIP` e virou **31** na integração de 2026-07-25: a
/// `line/physics` reivindicou o mesmo 30 na MESMA janela, por outro motivo (a âncora body-local
/// do joint, o parágrafo acima). O valor certo se CONTA, não se escolhe — ele não estava em
/// nenhum dos dois lados do conflito ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
/// v32 (ADR-0131, W-J6 servo + guincho): `PhysicsJoint` ganhou `motor_mode` +
/// `motor_target` APENDADOS — o motor deixou de ser só uma TAXA e passou a poder
/// mirar um LUGAR, e passou a existir também no Slider e na Rope. Mesmo padrão
/// posicional do v30: dois campos a mais no fim do blob, então um save v31 lido
/// como v32 leria além do fim dele, e um v32 lido por um binário v31 é recusado
/// como erro de versão em vez de virar um postcard perdido.
/// v33 (ADR-0131, W-J7 break force): `PhysicsJoint` ganhou `break_enabled` +
/// `break_force` + `break_torque` APENDADOS — um joint pode ser autorado para
/// ROMPER sob carga. Mesmo padrão posicional do v30/v32: três campos a mais no
/// fim do blob. ⚠️ O `∞ = off` NÃO é serializado: o componente guarda um
/// booleano e dois números finitos, e a ponte é quem os resolve na infinidade
/// que o solver quer — guardar `f32::INFINITY` faria o painel ter de mostrar
/// "inf" numa row numérica.
/// v34 (ADR-0131, W-J8 higiene do par): `PhysicsJoint` ganhou `active` +
/// `collide_connected` APENDADOS — desligar a restrição sem apagar o objeto, e
/// escolher se os dois corpos que ela une ainda se batem. Mesmo padrão
/// posicional do v30/v32/v33: dois campos a mais no fim do blob. ⚠️ O **Swap
/// A↔B** da mesma wave NÃO move nada aqui — ele reescreve campos que já
/// existem (as duas pontas, as duas âncoras, e os sinais medidos entre elas),
/// que é exatamente por que um bump se CONTA em vez de acompanhar a wave.
/// v35 (Flip, 2.5D multiplane, ADR-0114 §Decisão 3): a `FlipLayer` ganhou `depth` (a fração de
/// paralaxe da câmera) APENDADO — o `FlipDoc` viaja no `ProjectState`, então o layout posicional
/// muda e um save v34 lido como v35 leria `depth` além do fim do buffer. `FLIP_SCHEMA_VERSION`
/// 9→10, e o `PROJECT_SCHEMA` acompanha. ⚠️ A `line/FLIP` escreveu **32** aqui e a
/// `line/physics` reivindicou o MESMO 32 na mesma janela (o servo do W-J6) — a SEGUNDA vez
/// que estas duas linhas colidem no mesmo número, depois do 30 de 25/07. O valor certo se
/// CONTA a partir do `main` do dia (34 + 1), e não estava em nenhum dos dois lados.
/// v36 (Flip, Self Overlap, 03 §8): o `FlipStroke` ganhou `self_overlap` (auto-sobreposição com
/// acúmulo) no MEIO do struct (após `dot_spacing`) ⇒ layout posicional muda, um save v35 leria os
/// campos seguintes deslocados. `FLIP_SCHEMA_VERSION` 10→11.
/// v37 (Flip, Airbrush, 03 §8): o `FlipStroke` ganhou `airbrush` (falloff físico Beer-Lambert por
/// dab esférico) no MEIO do struct (após `self_overlap`) ⇒ mesmo raciocínio posicional.
/// `FLIP_SCHEMA_VERSION` 11→12.
/// v38 (Vector, plano 24 W6 — a LEI DE MISTURA por degrau): o `ph2d_ecs::FxOp` ganhou `blend`
/// APENDADO — um degrau da pilha de FX raster passa a dizer *como a cor dele encosta na que já
/// está ali* (Inner Shadow em Multiply escurece em vez de lavar; Color Overlay em Color troca a
/// matiz preservando a luminosidade). O `VecFilter` é componente registado, e postcard é
/// POSICIONAL, então um save v37 lido como v38 leria `blend` além do fim de cada degrau.
/// ⚠️ Não há como evitar o bump com `serde(default)`: o postcard não tem NOMES de campo, e um
/// buffer que acaba cedo é erro de decode, não um default.
/// ⚠️ **E o 38 carrega TAMBÉM a turbulência (plano 24 W6b), o Grow / Shrink (W7), o Color Adjust
/// (W8) e o Duotone (W9), de propósito:** o mesmo `FxOp` ganhou `scale`/`detail`/`seed`, depois
/// `grow`, depois `hue`/`sat`/`bright` e por fim `color_b` na MESMA janela, e um save v37 já é
/// recusado pelo 38 — pôr cada leva num número próprio jogaria fora exatamente os mesmos
/// arquivos. **Uma linha, um bump**; o que não pode acontecer é o número não subir.
/// v39 (physics, W-Rod): `JointKind` ganhou a variante **`Rod`** (a barra rígida). Apender
/// variante NÃO move os índices das existentes — o bump é para o caminho INVERSO: um build
/// antigo lendo um arquivo novo veria o discriminante 5 e devolveria lixo bem-formado em vez
/// de recusar. É o mesmo raciocínio do Weld (v27→28) e do Slider, e é por isso que a recusa
/// tem de ser ALTA. `FLIP_SCHEMA_VERSION` fica em 12.
/// v42 (physics, W-Pulley W1): o `PhysicsJoint` **PERDEU** `wheel_a`/`wheel_b`/`ratio` — uma
/// roldana virou ENTIDADE (`PulleyWheel` + `Transform`), o que remove o teto de duas e dá ao
/// artista contar/posicionar/dimensionar cada uma. Componente NOVO não custa bump (blob-key
/// própria); o que custa é a REMOÇÃO dos três campos, porque postcard é posicional e um blob
/// v41 traz três a mais. E o `ratio` saiu por ser FÍSICA ERRADA: numa corda única sobre
/// roldanas livres a tensão é uniforme, então não há vantagem mecânica a ganhar de diâmetro
/// nenhum. `FLIP_SCHEMA_VERSION` fica em 12.
/// v40 (physics, W-Wheel): `JointKind` ganhou a variante **`Wheel`** (o cubo que gira E cavalga
/// uma suspensão). Mesmo raciocínio do v39, um degrau adiante: apender variante não move
/// índice nenhum, e o bump existe para o build antigo RECUSAR em vez de ler o discriminante 6
/// como lixo bem-formado. `FLIP_SCHEMA_VERSION` fica em 12.
/// v43 (physics, W-Pulley W2): a `PulleyWheel` ganhou **`motor_speed`** — a roldana
/// dirigida, o guincho. Componente NOVO não custa bump (blob-key própria), mas
/// APENDAR campo a um componente que já existe custa: postcard é **posicional**, e
/// um blob v42 tem um `f32` a menos, então o load leria lixo bem-formado no lugar
/// de recusar. Mesmo raciocínio do `is_sensor` (v27) e do `offset` (v29).
/// v44 (physics, W-Pulley W2): a `PulleyWheel` ganhou **`break_enabled`** e
/// **`break_force`** — o eixo que cede sob carga. Dois campos apendados, mesmo
/// raciocínio posicional do v43.
/// v45 (physics, W-Pulley W3): a `PulleyWheel` ganhou **`body`**, **`local`** e
/// **`mounted`** — a roldana montada num corpo que se move, a *cadernal móvel* de
/// uma talha, e com ela a vantagem mecânica que o `ratio` prometia e não
/// entregava. Três campos apendados, mesmo raciocínio posicional do v43/v44. O
/// par `local`/`mounted` é o do W-AnchorFollow: o eixo é guardado no frame do
/// CORPO e convertido uma vez, senão mover o bloco o faz deslizar por ele.
/// v46 (physics, W-Pulley W4): a `PulleyWheel` ganhou **`radius_out`** — o SEGUNDO
/// diâmetro do eixo, que faz dela um **tambor diferencial** e devolve a vantagem
/// mecânica CONTÍNUA que o `ratio` do W-Pulley prometia sem ter peça na cena. Um
/// campo apendado, mesmo raciocínio posicional do v43/v44/v45.
/// v47 (physics, W-SoftWeld): o `PhysicsJoint` ganhou **`soft`** — a solda que
/// CEDE, o vão que este conjunto tinha entre segurar um ângulo *absolutamente* e
/// deixá-lo *inteiramente livre*. Um campo apendado, mesmo raciocínio posicional
/// do v43..v46; a dureza reusa a `stiffness`/`damping` que a mola já carregava,
/// então é UM bool e não três campos.
/// v48 (FLIP, as PONTAS do traço): o `Cap` ganhou a variante **`Square`**, e o
/// `FlipDoc` viaja DENTRO do `ProjectFile` ⇒ o `FLIP_SCHEMA_VERSION` 12→13 arrasta
/// este junto (a escada dele está no `ph2d_flip::FLIP_SCHEMA_VERSION`).
/// ⚠️ **As duas linhas escreveram 47 na mesma janela** — a `line/FLIP` e a
/// `line/physics` (v47 acima) —, e o valor se **CONTA** a partir do `main` do dia:
/// a física ficou com o 47 e o Flip foi contado para o 48. É a 3ª vez entre estas
/// duas ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]), e desta vez
/// ela quase passou MUDA: os dois lados escreveram o mesmo literal, então o
/// `project.rs` **não conflitou** — quem denunciou foi o gate da tripla ao lado.
/// v49 (vector, W6.2 — as guias e a régua): o `ProjectState` ganhou **`guides`**, a
/// lista de linhas de referência que o artista arrasta da régua. Campo apendado ao
/// `ProjectState`, que viaja DENTRO do `ProjectFile` — o mesmo raciocínio posicional
/// do `flip`, e o mesmo motivo de estar ali e não num campo de arquivo próprio: o
/// `ProjectState` é a unidade do UNDO, e uma guia arrastada tem de desfazer.
/// v50 (vector, W6.4 — o alinhamento do traço): o `StrokeSpec` ganhou **`align`**
/// (Centre/Inner/Outer), e ele mora dentro do `VecScene` — que viaja no `ProjectState`. O bump
/// é obrigatório nos DOIS sentidos e pelo motivo medido em `VEC_SCENE_SCHEMA_VERSION` v14: o
/// postcard não sinaliza ausência, então um save antigo lido pelo novo chega ao fim dos bytes
/// no campo novo e o novo lido pelo antigo traz um byte a mais. O número transforma os dois num
/// erro de VERSÃO em vez de num postcard a falhar longe da causa.
/// v51 (plano UI/UX W6): o arquivo carrega a **tabela de COR autorada** (`tokens`), campo
/// apendado ao `ProjectFile`. Postcard é posicional ⇒ o bump é obrigatório nos dois sentidos,
/// pelo mesmo motivo do v50 logo acima.
/// v52 (3D, W8.3 — o documento da escultura): campo de ARQUIVO novo, `sculpt`, um blob
/// opaco que carrega a própria versão (`SCULPT_DOC_VERSION`) — o precedente do
/// `TimelineDoc`, e é ele que deixa o módulo evoluir muitas waves sem tocar este número
/// de novo (docs/3D/02.3 previu exatamente isto). O bump é obrigatório porque o postcard
/// é POSICIONAL: um campo novo no fim faz o leitor velho chegar ao fim dos bytes.
/// ⚠️ O número se CONTA contra o `main` do dia, não se escolhe — este 52 era
/// PROVISÓRIO até a integração ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
/// v53 (3D, W8.7 — os canais assados): campo de ARQUIVO novo, `baked_forms`, com o `base`, a
/// `form` e o RIG de cada objeto que uma malha acendeu (`docs/3D/02.2`, rota A). Bump
/// obrigatório pela mesma razão de sempre — postcard é POSICIONAL, e um campo novo no fim faz o
/// leitor velho chegar ao fim dos bytes. ⚠️ Ele NÃO entrou no blob `sculpt` acima, embora
/// aquele já guarde as malhas: o parser dele é `#[cfg(feature = "sculpt3d")]`, e um objeto
/// assado tem de ser legível **sem o módulo 3D no build** — é isso que a rota A promete.
/// ⚠️ Este 53 é PROVISÓRIO pelo mesmo motivo que o 52 era.
/// v54 (physics, W-JointCustom — o joint que o artista descreve por EIXO): o
/// `PhysicsJoint` ganhou **`custom`**, a configuração por grau de liberdade
/// (*Free / Limited / Locked*, o modelo do Unreal) que expõe o `GenericJoint` do
/// rapier. Campo apendado ao componente, mesmo raciocínio posicional dos
/// v32/v33/v34: um save v53 lido por v54 chega ao fim dos bytes no campo novo.
/// v55 (physics, W10 — as duas assistências que faltavam): o `PlatformPlayer`
/// ganhou **`corner_reach`** e **`lift_momentum`**, os dois campos que o W8 tinha
/// nomeado e não construído. Dois campos apendados ao componente, mesmo
/// raciocínio posicional — e o mesmo motivo de estarem NELE e não num componente
/// próprio: são knobs da mesma lei que os outros dezenove, e o custo de um
/// componente novo por knob é uma lista que ninguém lê.
/// ⚠️ A linha escreveu **52**; o valor CONTADO contra o `main` da integração é 55.
/// v56 (plano UI/UX W7 — os ESTADOS de UI): o `ProjectState` ganhou **`ui_states`**, a tabela
/// de idle/hover/press por hospedeiro. Campo apendado ao `ProjectState`, que viaja DENTRO do
/// `ProjectFile` — o mesmo raciocínio posicional do `guides` (v49), e o mesmo motivo de estar
/// ali e não num campo de arquivo próprio: o `ProjectState` é a unidade do UNDO, e **gravar um
/// estado tem de desfazer**.
/// ⚠️ **Nenhum gate viu este bump, e é por isso que ele está escrito à mão:** um campo APENDADO
/// não move constante nenhuma, então a suíte inteira fica verde com o arquivo já incompatível —
/// o postcard é posicional e devolveria lixo bem-formado. Quem apende, bumpa, no MESMO commit.
/// ⚠️ E o valor é **PROVISÓRIO**: ele se CONTA contra o `main` do dia da integração, não se
/// escolhe ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
/// v57 (plano UI/UX W4b — o ALIAS de token): um token autorado passa a valer uma cor **ou** o
/// nome de outro token, então o `SavedToken` troca o campo `rgba: [u8; 4]` pelo enum
/// `SavedValue`. ⚠️ Um enum e não um campo `alias` ao lado do `rgba`: os dois seriam mutuamente
/// exclusivos e nada no formato o diria. Postcard é POSICIONAL e a forma do registro mudou ⇒ o
/// bump é obrigatório nos dois sentidos, o mesmo raciocínio do v50/v51.
/// ⚠️ **PROVISÓRIO** pelo mesmo motivo que o v56.
/// v58 (plano UI/UX W4c.1 — a camada NUMÉRICA): a escala (`spacing.*`, `radius.*`, `stroke.*`)
/// passa a ser autorável, e o valor autorado viaja na **MESMA lista** `tokens` — o `SavedValue`
/// ganha a variante **`Number(f32)`**, e a chave (`"spacing.md"`) é quem diz de que família a
/// entrada é. ⚠️ Uma lista só, e não um campo `num_tokens` ao lado: o que o arquivo guarda é *"que
/// tokens o artista autorou"*, e duas listas para isso seriam duas respostas à mesma pergunta que
/// o import/export DTCG (W4c.5) teria de juntar de novo.
/// ⚠️ **Apendar variante NÃO move `Literal`(0) nem `Alias`(1)**, então todo arquivo já salvo
/// continua a ler — o bump é pelo caminho **INVERSO**: um build antigo a ler um arquivo novo
/// bateria num índice de variante que ele não tem, e o número transforma isso num erro de VERSÃO
/// em vez de num postcard a falhar longe da causa (o raciocínio do `JointKind::Weld`/`Cap::Square`).
/// ⚠️ **PROVISÓRIO** pelo mesmo motivo que o v56.
/// v59 (plano UI/UX W4c.3 — a MATH): um token numérico passa a poder valer uma **fórmula**
/// (`{spacing.md} * 2`), e o `SavedValue` ganha a variante **`Formula(String)`**. ⚠️ TEXTO, e não
/// o IR parseado: é o texto que o artista reabre e edita, e serializar a árvore faria o formato do
/// arquivo depender da forma de um tipo do parser — a decisão que a `motion.expression` já tomou.
/// ⚠️ **Apendar variante NÃO move `Literal`(0)/`Alias`(1)/`Number`(2)**, então todo arquivo já
/// salvo continua a ler; o bump é pelo caminho INVERSO, o mesmo raciocínio do v58 logo acima.
/// ⚠️ **PROVISÓRIO** pelo mesmo motivo que o v56.
/// v60 (plano UI/UX W4c.4 — os tokens de ESCALA no DOCUMENTO): o `ph2d_ecs::BoundProp` ganha
/// **`StrokeWidth`(2)**, **`LayoutGapMain`(3)** e **`LayoutGapCross`(4)** — a espessura de um traço
/// e o vão de um auto layout passam a poder SEGUIR um token numérico, como a cor já seguia.
/// ⚠️ **Apendar variantes NÃO move `Fill`(0) nem `StrokeColor`(1)**, então todo binding já salvo
/// continua a ler; o bump é pelo caminho INVERSO, o mesmo raciocínio do v58/v59 acima.
/// ⚠️ **PROVISÓRIO** pelo mesmo motivo que o v56.
/// v61 (plano UI/UX **W2a** — o texto sabe medir-se): o `ph2d_ecs::VecTextParams` ganha
/// **`wrap_width: Option<f64>`**, a largura da caixa a que o texto REFLUI.
/// ⚠️ **Este é o único bump que o plano UI/UX previu e NOMEOU o preço** (§6.3): campo apendado
/// a componente EXISTENTE, e o blob de um componente é postcard **posicional** ⇒ um arquivo já
/// salvo lido pelo build novo bate no fim dos bytes. Um componente NOVO teria custado zero (o
/// precedente do `VecStrokeProfile`/ADR-0148 e dos overrides da física) — e foi recusado com
/// motivo: `wrap_width` é um número de layout ao lado do `align`/`tracking`/`line_height`, e
/// pô-lo noutro componente partiria a porta única `layout_of_params` em duas, com todo
/// consumidor que esquecesse a segunda a desenhar um texto sem refluxo **em silêncio**.
/// ⚠️ A dívida que este bump paga é a que a **F1.W1 da `line/runtime`** (*uma versão por
/// `ComponentBlob`*) apagaria — ela não existe, então o preço é este, escrito.
/// ⚠️ **PROVISÓRIO** pelo mesmo motivo que o v56.
/// v62 (plano UI/UX **W7m** — a MOLA como opção): o `ph2d_ui_state::HostStates` ganha
/// **`spring: Option<Spring>`**, a alternativa ao par *duração + curva*.
/// ⚠️ **Campo apendado a um struct SERIALIZADO** ⇒ postcard posicional ⇒ quebra dura, a mesma
/// classe do v61. Um `#[serde(default)]` **não salva**: o postcard não sinaliza ausência, então
/// um arquivo antigo bate no fim dos bytes de qualquer maneira.
/// ⚠️ **O sistema de easing fica INTACTO** — `duration_s` e `easing` continuam onde estavam e um
/// hospedeiro sem mola percorre o MESMO caminho de antes, byte a byte. A mola é uma `Option`
/// porque *ter mola* e *que mola* são a mesma decisão (o desenho do `wrap_width` do v61), e
/// desligá-la **não apaga** o que o artista afinou nas outras duas.
/// ⚠️ **PROVISÓRIO** pelo mesmo motivo que o v56.
/// v63 (3D, W10.7 — a oclusão que a doação carrega): o `BakedFormDocument` ganhou
/// **`form_occ`**, a oclusão de forma de um objeto assado (cavidade × os dois AOs),
/// um byte por texel. Bump obrigatório pela razão de sempre — postcard é POSICIONAL.
/// ⚠️ **E ela viaja em vez de ser assada no `base`**, o que seria de graça em disco: um
/// re-bake REUSA o `base` (`sculpt3d_bake::bake_one`), então pré-multiplicar a oclusão
/// ali a comporia a cada gesto e o objeto escureceria sozinho — o defeito exato que o
/// `base` existe para impedir. ⚠️ **VAZIO num documento anterior**, e é a leitura
/// honesta: o neutro da oclusão é `1.0`, e um plano de zeros pintaria de preto toda arte
/// já assada.
/// ⚠️ **A linha escreveu 56; o valor CONTADO contra o `main` da integração é 63** — a
/// `line/Vector` trouxe os SETE degraus v56..v62 na mesma janela, e o número se CONTA,
/// não se escolhe ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
/// v64 (physics, W13 — AS PAREDES): o `PlatformPlayer` ganhou **`wall_slide_speed`**,
/// **`wall_jump_height`**, **`wall_jump_push`** e **`wall_reach`** — a capacidade de
/// escorregar por uma parede e pular dela. Quatro campos apendados ao componente,
/// mesmo raciocínio posicional dos v32/v33/v34/v54/v55: um save v63 lido por v64
/// chega ao fim dos bytes no primeiro campo novo, e é o número que transforma isso
/// num erro de VERSÃO em vez de num postcard a falhar longe da causa.
/// ⚠️ **A linha escreveu 56; o valor CONTADO contra o `main` da integração é 64** — a
/// `line/Vector` (v56..v62) e a `line/sculpt3d` (v63) pousaram antes na mesma janela.
/// O número se CONTA, nunca se escolhe; três linhas já colidiram nele por o terem
/// escolhido ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
/// v65 (physics, W14 — O ARRANQUE): o `PlatformPlayer` ganhou **`dash_speed`**,
/// **`dash_time`** e **`dash_cooldown`**. Três campos apendados ao componente, pelo
/// mesmo raciocínio posicional de todos os degraus acima.
/// v66 (physics, W15 — O AGACHAR): o `PlatformPlayer` ganhou **`crouch_height`** e
/// **`crouch_speed`**. Dois campos apendados, e o motivo do bump é o de sempre —
/// postcard é posicional. ⚠️ Note o que este degrau **não** traz: nenhuma forma de
/// collider muda, porque agachar aqui é uma perna mais CURTA e não um corpo menor.
/// v67 (physics, W17 — A CORRIDA SOBREVIVE AO ARQUIVO): campo de ARQUIVO novo,
/// `player_tape`, com o que o dedo do jogador fez tique a tique. ⚠️ Ele fecha o
/// último item aberto do §4 do plano 06, e é o **bake da W16** que o torna útil:
/// a fita é a entrada que o bake replaya, então reabrir um projeto e apertar Bake
/// devolve a corrida de ontem. Medido: 60 s de corrida pesam **28,1 kB**.
/// v68 (physics, W23 — O AGARRAR-SE): o `PlatformPlayer` ganhou
/// **`wall_grab_stamina`**. Um campo apendado ao componente, pelo mesmo
/// raciocínio posicional de todos os degraus acima.
/// ⚠️ **Note o que este degrau NÃO traz:** o botão novo (`PlayerInput::grab`)
/// **não** move o formato da fita — ela guarda os botões num BITMASK (`(f32,
/// u8)`), e um bit livre não muda um byte do postcard. Uma corrida gravada antes
/// desta wave volta com o agarrar em zero, que é o que ela de facto tinha; um
/// campo novo na tupla teria custado o bump por si só e recusado todo arquivo já
/// salvo.
/// v69 (`line/motion-value`, doc 88 D3 — AS SETTINGS DO PROJETO viajam no arquivo):
/// campo de ARQUIVO novo, `settings` (`SavedSettings`) — a escala do mundo
/// (`pixels_per_meter`), a unidade que o artista LÊ, os dois snaps do gizmo e o modo
/// de filtragem. Fora do `ProjectState` pela razão dos irmãos `physics`/`motion`/
/// `timeline`: o `ProjectState` é a unidade do undo GLOBAL, e um Ctrl+Z do canvas
/// não deve rebobinar a escala do mundo.
/// ⚠️ **A linha escreveu 56 e o valor CONTADO é 69** — a `line/Vector` (v56..v62), a
/// `line/sculpt3d` (v63) e a `line/physics` (v64..v68) pousaram antes na mesma janela
/// ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
/// ⚠️ **E esta linha do degrau nasceu AUSENTE:** a integração renumerou o literal de
/// 56 para 69 e não escreveu a entrada, então a escada ficou documentando até v68 sob
/// uma const que dizia 69 — o buraco que faz o próximo bump nascer mal-numerado, pois
/// quem conta o próximo degrau lê a escada e não o literal. Escrita na varredura da
/// §5 do CLAUDE.md, no fim da mesma jornada.
/// v70 (physics, W-KinPush — O EMPURRÃO): o `PlatformPlayer` ganhou
/// **`reaction_push`**, o terceiro escalar da 3ª lei — quanto de um bloqueio
/// LATERAL volta para o corpo que o causou. Um campo do componente, pelo mesmo
/// raciocínio posicional de todos os degraus acima: o postcard grava na ordem de
/// declaração, então um leitor velho leria os campos seguintes deslocados.
/// ⚠️ **PROVISÓRIO até a integração** — o valor se CONTA contra o `main` do dia,
/// e nesta janela há outras linhas vivas
/// ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
/// v71 (physics, W-Swim — NADAR): o `PlatformPlayer` ganhou **três** campos
/// apendados — `swim_speed` (a capacidade; `0` desliga, e é assim que ela
/// nasce), `swim_acceleration` (a autoridade do servo contra o empuxo) e
/// `swim_enter` (**quantos PESOS o fluido tem de carregar** para o regime
/// armar). Os três num degrau só porque são uma capacidade só: quem escreve o
/// primeiro recebe um nado que funciona.
/// ⚠️ **O limiar não é uma altura, e não podia ser** — a mesma altura significa
/// coisas diferentes em cada poça; `1.0` é *a água sozinha me sustenta*, que é
/// por construção a linha de flutuação em qualquer densidade (a tabela está em
/// `measure_the_swim_threshold`).
/// ⚠️ **A FITA não se move**: o eixo vertical do nado sai dos botões `jump`/
/// `down`, que já viajam no BITMASK — um eixo novo teria mudado a forma de
/// `(f32, u8)` e recusado toda corrida já salva.
/// ⚠️ **PROVISÓRIO, como o degrau acima** — contado contra o `main` de hoje (70).
///
/// v72 (physics, W-Probes2 — OS SENSORES FICAM EDITÁVEIS): o `PlatformPlayer`
/// ganhou **quatro** campos apendados — `corner_samples` e `corner_lookahead`
/// (quantas amostras o perfil da quina varre, e quantos tiques de antecedência
/// ele olha), `wall_samples` e `wall_spread` (quantos raios o flanco casta, e
/// onde os de fora se sentam).
/// ⚠️ **Eram `const` e passam a ser AUTORADOS, com os defaults iguais às consts**
/// — todo player já salvo fica byte-idêntico, e o que muda é só quem pode mexer
/// neles. O report do Enio: *"não temos inputs para ajustes dos tamanhos e
/// posições dos sensores nem a quantidade de sensores"*.
/// ⚠️ **Um degrau só porque é um assunto só**: a `W-Probes` fechou a metade (b)
/// da §4.55 medindo que *cada NÚMERO tem row* — e não fez a pergunta que
/// faltava, que é sobre a GEOMETRIA das amostras.
/// ⚠️ **PROVISÓRIO** — contado contra o `main` do dia na integração.
///
/// v73 (physics, W-Probes2 — A PERNA VIRA UM LEQUE): o `PlatformPlayer` ganhou
/// `foot_samples` + `foot_spread`. ⚠️ **Este degrau MOVE FÍSICA**, ao contrário
/// do v72: o default de `foot_samples` é **3, não 1**, porque uma perna de um
/// raio só afunda **0,411 m — 46% do `float_height`** parada sobre uma fenda de
/// 10 cm num corpo de 40 cm que as bordas suportam
/// (`measure_what_a_single_ground_ray_costs_over_a_gap`). Um projeto salvo em
/// v72 reabre com a perna em leque, que é a correção.
///
/// v74 (physics, W-MultiJump — O PULO MÚLTIPLO): o `PlatformPlayer` ganhou
/// `air_jumps` + `air_jump_height`, no MEIO do struct (logo depois do
/// `jump_height`, que é onde eles se leem), e o postcard é posicional ⇒ quebra
/// dura. ⚠️ **Este degrau NÃO move física:** a contagem nasce em `0`, que é a
/// capacidade DESLIGADA — o precedente do wall slide e do wall jump —, então um
/// projeto salvo em v73 reabre com o pulo exatamente como estava.
///
/// v75 (physics, W-Ledge — A BEIRADA): o `PlatformPlayer` ganhou `ledge_grab` +
/// `ledge_speed`, apendados ao FIM, e o postcard é posicional ⇒ quebra dura.
/// ⚠️ **Este degrau NÃO move física:** o alcance nasce em `0`, que é a
/// capacidade DESLIGADA (o idioma de `coyote_time`/`corner_reach`/`air_jumps`),
/// então um projeto salvo em v74 reabre exatamente como estava — e o sensor
/// novo nem sequer é castado.
///
/// v76 (physics, W-Glide — PLANAR): o `PlatformPlayer` ganhou
/// `glide_fall_speed`, apendado ao FIM, e o postcard é posicional ⇒ quebra
/// dura. ⚠️ **Este degrau também NÃO move física:** o teto nasce em `0`, que é
/// a capacidade DESLIGADA, então um projeto salvo em v75 reabre a cair
/// exatamente como caía — e o `physics_ecs_c9` sai byte-idêntico, que é a prova
/// executável.
///
/// v77 (physics, W-LedgeSensor — O SENSOR DA BEIRADA): o `PlatformPlayer` ganhou
/// `ledge_reach_y` + `ledge_span`, apendados ao FIM, e o postcard é posicional ⇒
/// quebra dura. ⚠️ **Este degrau NÃO move física, e a razão é a redução
/// literal:** o `span` nasce em `0`, onde o leque tem **uma** amostra na posição
/// exacta do raio de antes, e o `reach_y` reproduz a janela `2·grab` de antes
/// quando vale o mesmo que o `grab` — as cenas de smoke autoram os dois em 0,60,
/// então elas reabrem idênticas e o `physics_ecs_c9` sai byte-idêntico.
///
/// ⚠️ **E o degrau é o único preço da wave**: os dois campos existem porque a
/// varredura da referência (GDevelop `Grab tolerance` + `Grab offset`, Corgi
/// *origem e comprimento*, os 5 traços do Unreal) refutou a frase *"o alcance é
/// uma grandeza só"* que o `grab` carregava.
/// v78 (physics, W-LedgeSensor — O OFFSET VERTICAL): o `PlatformPlayer` ganhou
/// `ledge_offset_y`, apendado ao FIM ⇒ quebra dura. ⚠️ **Ele existe porque o
/// `reach_y` é TAMANHO e não POSIÇÃO**, e o degrau v77 tinha mapeado os dois no
/// mesmo número (report do Enio: *"não temos como mover os sensores na
/// vertical"*). ⚠️ **Também NÃO move física:** nasce em `0`, onde a janela fica
/// centrada no topo do corpo como antes, e o `physics_ecs_c9` sai byte-idêntico.
/// v79 (physics, W-Brake — FREAR NÃO É ACELERAR): o `PlatformPlayer` ganhou
/// `brake_scale`, apendado ao FIM ⇒ quebra dura. ⚠️ **Ele existe porque a lei
/// gastava `acceleration` nos DOIS sentidos** — o fator de viragem cobre
/// *inverter* e não cobre *largar o direcional*, então um personagem que arranca
/// rápido era obrigado a parar rápido. ⚠️ **E o degrau é o ÚNICO preço da wave:**
/// o campo nasce em `1`, onde a lei reduz LITERALMENTE (`x * 1.0` é `x` em
/// IEEE-754) ⇒ todo projeto salvo em v78 reabre a andar e a parar exactamente
/// como estava, e o `physics_ecs_c9` sai byte-idêntico.
/// v80 (physics, W-Fall — O TETO DE QUEDA): o `PlatformPlayer` ganhou
/// `max_fall_speed`, apendado ao FIM ⇒ quebra dura. ⚠️ **Ele existe porque NÃO
/// havia velocidade terminal, e o número é desta wave:** largando de mil metros
/// a descida chega a **142,57 m/s aos 8 s** e continua a crescer, nos DOIS
/// modos — um personagem que caia de alto o bastante atravessa o cenário a
/// velocidades que nenhum colisor discreto resolve. ⚠️ **E o degrau é o ÚNICO
/// preço da wave:** o campo nasce em `0`, que **desliga** a lei (a porta devolve
/// `None` e o motor é `Motor::default()`) ⇒ todo projeto salvo em v79 reabre a
/// cair exactamente como caía, e o `physics_ecs_c9` sai byte-idêntico.
/// v81 (physics, W-Leave — O QUE A PLATAFORMA DA AO PULO): o `PlatformPlayer`
/// ganhou `platform_lift`, apendado ao FIM ⇒ quebra dura. ⚠️ **A altura autorada
/// era medida contra a PLATAFORMA, e ninguem tinha escolhido isso:** o pulo leva
/// a subida RELATIVA ao chao ao `v0`, o que e' o `ADD_VELOCITY` do Godot — e num
/// elevador a descer a 4 m/s o pico medido cai de **1,865 para 0,016 m**, nos
/// tres modos (`measure_platform_leave`). ⚠️ **E o degrau e' o UNICO preco:** o
/// campo nasce em `Full`, onde a porta devolve `rel_up` VERBATIM ⇒ todo projeto
/// salvo em v80 reabre a pular exactamente como pulava, e o `physics_ecs_c9` sai
/// byte-identico.
/// v82 (physics, W-Brink — A TRAVA DE BEIRADA): o `PlatformPlayer` ganhou
/// `walk_off_ledges` e `crouch_walk_off_ledges`, apendados ao FIM ⇒ quebra dura.
/// O `bCanWalkOffLedges` do Unreal, que ele serve a IA e ao *andar com cuidado*.
/// ⚠️ **Os campos guardam a CAPACIDADE, nunca a trava**, e a razao e' o postcard:
/// num `stop_at_ledges` o `false` que todo arquivo antigo traz num campo novo
/// significaria *trava armada*, e a capacidade nasceria ligada em toda arte ja'
/// autorada. ⚠️ **E o degrau e' o UNICO preco:** os dois nascem em `true`, onde
/// a lei devolve o alvo VERBATIM e o sensor nem sequer casta ⇒ todo projeto
/// salvo em v81 reabre a andar exactamente como andava, e o `physics_ecs_c9` sai
/// byte-identico. ⚠️ **O alcance NAO e' um degrau:** ele e' DERIVADO
/// (`v²/2a` + meia-largura) porque o knob que ele substituiu tinha o valor certo
/// em funcao de outros dois — medido, a 8 m/s um `0,30` deixava o personagem
/// CAIR e um `0,60` o segurava, com a fronteira exactamente em `0,533`.
/// v83 (`line/Vector`, item 4 do estudo dos contêineres — A TABELA SINAL → AÇÃO): o
/// `HostStates` ganhou **`on_signal`**, a lista de ligações *nome de sinal → papel*
/// (`ph2d_ui_state::SignalBinding`). Ele mora DENTRO do `HostStates` — e não numa
/// tabela própria — porque o `retain_hosts` já corre por frame: uma forma apagada leva
/// as ligações dela sem uma linha a mais, no mesmo frame e no mesmo passo de undo. É o
/// degrau irmão do `spring` (v62), no mesmo struct e pelo mesmo motivo posicional: o
/// postcard grava na ordem de declaração, então um leitor velho leria lixo bem-formado.
/// v84 (`line/Vector`, item 5 do estudo dos contêineres — A GRADE): o `LayoutDir` ganhou
/// **`Grid`** (variante APENDADA, logo `Row`/`Column`/`RowWrap` continuam em 0/1/2 e todo
/// arquivo já salvo segue legível) e o `VecLayout` ganhou **`columns`**.
/// ⚠️ O bump é pelo caminho **INVERSO**, e é o campo que o obriga: o postcard é
/// posicional, então um leitor velho leria os bytes do `columns` como o começo do que
/// vem a seguir. O número transforma isso num erro de VERSÃO — o raciocínio do
/// `Cap::Square` do Flip e do `JointKind::Weld` da física.
/// ⚠️ A contagem mora no `VecLayout` e **não** dentro do variante, para sobreviver a uma
/// troca de direção: ir a `Row` e voltar devolve a grade intacta, como o vão e o recuo já
/// fazem.
/// v85 (`line/Sprite`, plano [`docs/Sprite_projeto/17`] §3 — OS PIXELS GANHAM NOME): o
/// `ProjectFile` ganhou **`sprite_pixels`**, o documento do `ph2d-sprite-sheet`. Campo
/// apendado ⇒ bump pelo motivo posicional de sempre.
/// ⚠️ **O que este degrau CONSERTA é perda de dados em produção, não uma capacidade nova.**
/// `SpriteSource::Individual { texture_id }` guarda um id de alocação da GPU, e o
/// `IndividualTextureStore` recomeça a numerar em `1` a cada processo: um sprite tocado por
/// QUALQUER ferramenta de imagem (trim · bgremoval · make-square · padding · upscale ·
/// rasterize · equalize) reabria **invisível**, ou a exibir os pixels de outro sprite que
/// ficou com aquele id no restore. O Painter (v3) e o bake 3D (`baked_forms`) já tinham sido
/// resgatados um a um; este degrau é o **chão** que faltava debaixo dos dois.
/// ⚠️ **E ele bumpa UMA vez.** O blob carrega a própria versão (`SHEET_DOC_VERSION`), como o
/// `TimelineDoc` e o `sculpt` — então as regiões do hand-packed, que entram neste MESMO
/// documento, não voltarão a recusar projeto salvo nenhum.
/// v86 (`line/Sprite`, §5 9-Slice — A FORMA DO `SliceNine` MUDOU TRÊS VEZES NUM DIA): o
/// componente `SliceNine`, registado em `register_ecs_components`, perdeu o campo
/// **`stretch_value`** (o slider `Stretch` do `Adaptive`, retirado porque o mecanismo dele não
/// podia funcionar) e o `SliceDrawMode` perdeu a variante **`Tiled`** (ela era o `Sliced` menos a
/// capacidade de esticar uma região).
/// ⚠️ **O componente é name-keyed, mas o BLOB dele é posicional.** Um projeto gravado hoje de
/// manhã tem a mesma chave `"ph2d::ecs::SliceNine"` com o layout velho: sem este degrau o leitor
/// novo lê o `bool` do `fill_center` onde estavam os quatro bytes do `stretch_value`. É a lei que
/// os degraus v6/v7/v8 já escreveram — *não é campo novo no arquivo, é o MESMO campo com outro
/// layout, e posicional é posicional*.
/// ⚠️ **Um bump para as três mudanças, não três.** Elas caem no mesmo dia e no mesmo componente,
/// e nenhuma chegou ao `main`: o que o número tem de separar é o formato de ontem do de hoje.
/// ⚠️ Adicionar o `SliceNine` e o `NamedAnchorList` ao registo (2026-08-21) **não** pediu degrau —
/// isso é aditivo numa tabela por NOME, e um arquivo velho apenas não os tem. O que pede degrau é
/// **mudar a forma de uma chave que já existe**.
/// v87 (`line/Vector` — O RECORTE SAIU DA MOLDURA): o `VecFrame` perdeu o campo `clip` e virou
/// **marcador**, e o recorte passou a ser o componente próprio `ph2d_ecs::VecClipContent`, que
/// qualquer forma FECHADA pode carregar (Enio: *"coloque a feature Clip Content para qualquer
/// forma vetorial fechada"*).
/// ⚠️ **Quem obriga o bump é o campo REMOVIDO**, e é o caso inverso ao do v84: o postcard é
/// posicional, então um leitor novo sobre bytes velhos leria o `bool` do `clip` como o começo do
/// componente seguinte — e ao contrário de um campo apendado, aqui nem o tamanho bate. O número
/// transforma isso num erro de VERSÃO honesto.
/// ⚠️ E o registro de componentes subiu junto (63 → 64): um componente que não passa por
/// `register_ecs_components` é descartado em silêncio pelo snapshot, e o recorte evaporaria no
/// primeiro Ctrl+Z com a arte toda no lugar — o modo de falha mais enganoso da lista.
/// ⚠️ **Este degrau nasceu como v85 na `line/Vector` e foi RECONTADO na integração de
/// 2026-08-22**: a `line/Sprite` entrou antes com v85/v86, e *número que soma entre linhas se
/// CONTA, nunca se escolhe* (CLAUDE.md §5.0) — o handoff da linha ainda diz 85.
/// v88 (`line/Vector` — OS FILTROS NOS ESTADOS DE UI): o `ph2d_ui_state::ObjectPose` ganhou
/// **`filters`**, a pilha de FX raster daquele estado (`ph2d_fx_op::FxOp`), apendada ao fim.
/// ⚠️ **É o irmão exacto do `width`**, e pela mesma razão que aquele campo existe: os dois são
/// canais que NÃO vivem no `VecPath` — são componentes ECS (`VecStrokeProfile`, `VecFilter`) —,
/// então a pose tem de os carregar por si. Sem ele um blur era o único efeito do editor incapaz
/// de diferir entre *Default* e *Hover* (Enio, 2026-08-21).
/// ⚠️ O bump é posicional como sempre: o campo entra dentro de cada `ObjectPose`, que viaja no
/// `HostStates` (v83), que viaja no `ProjectFile`.
/// ⚠️ E o degrau MUDOU DE CASA na mesma wave — `FxOp` saiu do `ph2d-ecs` para a folha
/// `ph2d-fx-op`, para a `ph2d-ui-state` (que deliberadamente não vê ECS) o poder carregar. A
/// forma serializada é a MESMA; o que mudou foi de que crate o tipo vem.
/// ⚠️ Nasceu como v86 na `line/Vector`; RECONTADO para v88 na integração de 2026-08-22 (ver v87).
/// v89 (`line/Vector` — UM VERBO POR FORMA na booleana viva): o componente novo
/// `ph2d_ecs::VecBoolOp` guarda, **por forma**, a operação com que ela dobra sobre o resultado das
/// anteriores (Enio, 2026-08-22: *"o modo do boolean é escolhido por shape e na ordem em que
/// aparece na hierarquia atua sobre o resultante das operações pregressas"*).
/// ⚠️ **Quem obriga o bump é o REGISTRO, não um campo.** O componente não muda a forma de nenhum
/// tipo já serializado — ele é uma entrada NOVA no `ComponentRegistry` (64 → 65). E o modo de
/// falha de esquecer o registro é o mais enganoso desta escada inteira: um componente que não
/// passa por `register_ecs_components` é **descartado em silêncio** pelo snapshot, então o
/// primeiro Ctrl+Z devolveria a arte toda no lugar, a combinação intacta — e a receita achatada de
/// volta no `op` do grupo, desenhando outra coisa **sem nada em falta na tela a denunciá-lo**.
/// ⚠️ Ausência do componente continua a ser **herança** do `op` do grupo, e é isso que faz todo
/// arquivo ≤ v88 desenhar byte-idêntico: nenhuma forma o tem, todas herdam, e herdar é o que o
/// grupo já fazia.
/// ⚠️ Nasceu como v87 na `line/Vector` (o handoff dela diz «86 → 87»); RECONTADO para v89 na
/// integração de 2026-08-22 — a `line/Sprite` entrou antes com v85/v86 (ver v87).
/// v90 (`line/Sprite` — QUEM MONTA numa âncora): o componente novo `ph2d_ecs::AnchorMount`
/// guarda, na entidade FILHA, o **nome** da âncora do pai de que ela parte (ADR-0072 §2.6 — o
/// consumidor que o ADR declarou em 2026-05 e que nunca existiu).
/// ⚠️ **Quem obriga o bump é o REGISTRO, não um campo** — irmão exacto do v89, e com o mesmo modo
/// de falha enganoso: um componente fora do `ComponentRegistry` é **descartado em silêncio** pelo
/// snapshot, e então reabrir o projeto devolveria a espada como filha comum do personagem —
/// no sítio certo, parada. Nada some, nada avisa, e o defeito só aparece quando o braço se mexe.
/// ⚠️ Ausência do componente é **não montar**, que é o que toda entidade fazia até v89: por isso
/// todo arquivo ≤ v89 desenha byte-idêntico.
/// ⚠️ O componente guarda o NOME, nunca o índice na lista nem os bits da entidade — apagar a
/// âncora `0` faria toda a gente descer uma casa em silêncio, e *o undo respawna tudo com bits
/// novos*.
pub(crate) const PROJECT_SCHEMA: u32 = 90;
