//! Save/load de PROJETO em disco (Ctrl+S / Ctrl+O globais).
//!
//! O projeto é a MESMA captura do undo — `ProjectState = {WorldSnapshot + VecScene}`
//! — mais os **bytes das imagens** dos sprites (`SavedAsset`), que o undo não guarda
//! (são estáveis, não mudam a cada ação). Um arquivo é `(PROJECT_SCHEMA, ProjectFile)`
//! em postcard.
//!
//! Fase 2a (esta): estado + geometria. Formas vetoriais voltam 100%; sprites voltam
//! com pose/estrutura, e a imagem se o `AssetDb` ainda a tiver (mesma sessão).
//! Fase 2b: `collect_assets`/`materialize_assets` embutem e re-materializam os pixels,
//! fechando o cross-sessão.

use crate::undo::{ProjectState, ProjectUndo};

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
const PROJECT_SCHEMA: u32 = 68;

/// O conteúdo de um arquivo de projeto.
#[derive(serde::Serialize, serde::Deserialize)]
struct ProjectFile {
    /// Mundo (ECS) + geometria vetorial — a unidade do undo.
    state: ProjectState,
    /// Pixels dos sprites, para re-materializar o atlas noutra sessão (Fase 2b).
    /// Vazio na Fase 2a.
    assets: Vec<SavedAsset>,
    /// Os **documentos do Painter** (camadas + pixels + relevo), por identidade estável
    /// (`ph2d_ecs::PaintedDoc`). Vazio quando nada foi pintado. Ver [`crate::project_painter`].
    painted: Vec<ph2d_tool_painter::PaintedDocument>,
    /// O documento de **Motion Nodes**, na forma textual canônica do `ph2d-motion-doc`
    /// (linha-a-linha, com `[layout]` e `[backdrop]` — ADR-0032 §6).
    ///
    /// Campo do ARQUIVO, deliberadamente **fora do `ProjectState`**: o `ProjectState` é a
    /// unidade do undo GLOBAL, e o Motion tem undo próprio (`MotionHistory`) — o Enio já
    /// separou os dois escopos. Enfiar o grafo ali dentro faria cada Ctrl+Z do canvas
    /// rebobinar o grafo junto, e vice-versa.
    ///
    /// É **texto**, não postcard, porque esse já é o formato canônico do documento: é
    /// diffável e mergeável por linha (o requisito multiagente que descartou JSON/RON).
    /// Um projeto sem grafo carrega `""`.
    motion: String,
    /// O **`TimelineDoc`** (clips, faixas, tracks, keys) em postcard — a animação inteira.
    ///
    /// Fora do `ProjectState` pelo mesmo motivo do `motion`: o `ProjectState` é a unidade do
    /// undo GLOBAL, e a timeline tem undo próprio. Enfiá-la ali faria cada Ctrl+Z do canvas
    /// rebobinar a animação junto.
    ///
    /// As bindings viajam com o **`wire_id`** (hash do `Name` do objeto) carimbado no save, e
    /// NÃO com os bits de entidade — que o load recicla. Quem as recola é o `upkeep` do frame,
    /// a mesma função que cura delete+undo (ver [`crate::timeline_persist`]). Um projeto sem
    /// animação carrega `vec![]`.
    timeline: Vec<u8>,
    /// As **settings de MUNDO** da física (ADR-0131 D8 / W2b).
    ///
    /// Fora do `ProjectState` de propósito: o `ProjectState` é a unidade do undo
    /// GLOBAL, e um Ctrl+Z do canvas não deve rebobinar a gravidade da cena —
    /// o mesmo motivo que mantém `motion` e `timeline` aqui fora.
    ///
    /// O mundo rapier em si **não** viaja (D2: ele é derivado); o que viaja é o
    /// que o artista autorou.
    physics: ph2d_physics_ecs::PhysicsSettings,
    /// **A tabela de COR autorada pelo artista** (plano UI/UX W6, degrau 1).
    ///
    /// ⚠️ **Esparsa e FORA do `ProjectState`**, pelas duas razões de sempre: só o que difere da
    /// fábrica viaja (um projeto que nunca abriu o painel guarda um vetor vazio), e um Ctrl+Z do
    /// canvas não deve rebobinar a cara do editor — o mesmo motivo que mantém `physics`,
    /// `motion` e `timeline` aqui fora.
    ///
    /// ⚠️ O que viaja é o par `(modo, chave-do-token)` e a cor. A **CHAVE**, nunca o índice do
    /// variant: guardar o índice amarraria todo projeto salvo à ORDEM da lista, e acrescentar um
    /// token no meio da tabela re-pintaria o app com as cores trocadas. É a mesma lei do `W4a`.
    tokens: Vec<crate::project_tokens::SavedToken>,
    /// **A ESCULTURA** (ADR-0150 W8.3) — a lista de peças, cada uma com a pilha de
    /// níveis e a pose, em postcard. Ver [`crate::sculpt3d`] (`sculpt3d_doc.rs`).
    ///
    /// Fora do `ProjectState` pelo mesmo motivo de `motion`/`timeline`/`physics`: o
    /// `ProjectState` é a unidade do undo GLOBAL, e a escultura tem fila própria —
    /// um Ctrl+Z do canvas não pode rebobinar uma pincelada de barro.
    ///
    /// ⚠️ **`Vec<u8>` opaco e SEM `cfg`**, e é isso que sustenta a promessa de
    /// removibilidade do `docs/3D/02.3`: o campo existe com o módulo desligado (o
    /// postcard é posicional — um campo condicional daria DUAS formas de arquivo com o
    /// mesmo número de schema), e um binário sem escultura **carrega os bytes adiante**
    /// em vez de os triturar. Ele carrega a própria versão lá dentro.
    sculpt: Vec<u8>,
    /// **OS CANAIS ASSADOS** (ADR-0150 W8.7) — por objeto: os pixels antes da luz, o G-buffer que
    /// uma malha doou, e o rig com que aquilo foi aceso. Ver [`crate::project_baked_form`].
    ///
    /// ⚠️ **Campo de SPRITE, e não parte do blob `sculpt` acima**, embora aquele já guarde as
    /// malhas. O parser da escultura é `#[cfg(feature = "sculpt3d")]`; guardar os canais lá os
    /// tornaria legíveis só com o módulo 3D no build — o oposto exato do que a *rota A* promete
    /// (`docs/3D/02.2`: a malha some do build, o objeto continua reluminável). Ele fica ao lado do
    /// `painted`, que resolve o mesmo problema para o outro produtor de `SpriteSource::Individual`.
    ///
    /// Vazio quando nada foi assado.
    baked_forms: Vec<crate::project_baked_form::BakedFormDocument>,
    /// **A CORRIDA GRAVADA** (ADR-0131 W17) — o que o dedo do jogador fez, tique
    /// a tique, na forma de arquivo da fita (`ph2d_physics_ecs::TapeWire`).
    ///
    /// ⚠️ **Ela é AUTORIA, e é o bake da W16 que o prova:** a fita é a entrada que
    /// o bake replaya para escrever as curvas, então perdê-la ao fechar o app é
    /// perder a corrida que o artista jogou — reabrir e apertar Bake devolvê-la é
    /// a razão inteira deste campo.
    ///
    /// ⚠️ **Fora do `ProjectState`**, pelo mesmo motivo de `motion`/`timeline`/
    /// `physics`: aquele é a unidade do undo GLOBAL, e um Ctrl+Z do canvas não
    /// deve rebobinar a gravação.
    ///
    /// Vazia num projeto onde ninguém correu — e ⚠️ **é a correção da W17 que
    /// torna essa frase verdadeira**: antes dela a fita gravava todo tique que o
    /// relógio andasse, então TODO projeto do app carregaria uma corrida de
    /// ninguém. Ver `render_loop::physics_bridge::dispatch`.
    player_tape: ph2d_physics_ecs::TapeWire,
}

/// Uma imagem de sprite embutida no projeto: os pixels RGBA + a célula de atlas que
/// o `Sprite.source` referencia. (Fase 2b.)
#[derive(serde::Serialize, serde::Deserialize)]
struct SavedAsset {
    /// A célula de atlas (`SpriteSource::Atlas { key }`) que estes pixels ocupam.
    key: u32,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

/// **O que a troca de documento faz os produtores VIVOS esquecerem** — irmão pelo teto de LOC
/// (HR-18), e o corte é por responsabilidade: aqui vive *o que não sobrevive a um load*.
#[path = "project_forget.rs"]
mod forget;

impl crate::App {
    /// Caminho do arquivo de projeto (env `PH2D_PROJECT_PATH`, default no CWD).
    fn project_path() -> String {
        std::env::var("PH2D_PROJECT_PATH").unwrap_or_else(|_| "ph2d_project.postcard".to_string())
    }

    /// **Os bytes da escultura que este save vai gravar.**
    ///
    /// A cena VIVA é a verdade sempre que ela existe; quando não existe, a verdade são
    /// os bytes como vieram do arquivo — o que cobre os dois casos honestos: um projeto
    /// aberto antes de a GPU aparecer (o `pending` ainda não instalou) e um binário
    /// construído **sem** o módulo, que carrega adiante o que não sabe ler.
    fn sculpt_bytes_for_save(&self) -> Vec<u8> {
        #[cfg(feature = "sculpt3d")]
        if let Some(gfx) = self.gfx.as_ref()
            && let Some(scene) = gfx.sculpt3d.as_ref()
        {
            return scene.to_doc_bytes();
        }
        self.sculpt_doc.clone()
    }

    /// Ctrl+S: serializa o projeto inteiro (mundo + geometria + pixels) para o disco.
    pub(crate) fn project_save(&mut self) {
        let assets = self.collect_assets();
        // Os documentos pintados carimbam a identidade estável no mundo, então isto tem de rodar ANTES
        // da captura — senão o `PaintedDoc` recém-inserido ficaria de fora do snapshot e o load não
        // teria a quem devolver o documento.
        let painted = self.collect_painted_docs();
        // A animação. O `serialize` carimba em cada binding o hash do NOME do objeto — é por
        // ele que a track reencontra o objeto do outro lado do arquivo (os bits de entidade
        // não sobrevivem a um respawn). Precisa do mundo, então vem antes da captura.
        let timeline = match self.gfx.as_ref() {
            Some(gfx) => {
                let world = gfx.sim.world();
                match crate::timeline_persist::serialize(&mut self.timeline, world) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("[proj] timeline nao serializou, projeto salvo SEM ela: {e}");
                        Vec::new()
                    }
                }
            }
            None => Vec::new(),
        };
        let Some(state) = self.capture_project() else {
            return;
        };
        let file = ProjectFile {
            state,
            assets,
            painted,
            motion: self
                .gfx
                .as_ref()
                .map(|g| g.motion.doc.to_text())
                .unwrap_or_default(),
            timeline,
            physics: self
                .gfx
                .as_ref()
                .map(|g| g.physics.settings())
                .unwrap_or_default(),
            tokens: crate::project_tokens::collect(),
            sculpt: self.sculpt_bytes_for_save(),
            baked_forms: self.collect_baked_forms(),
            // A corrida que o artista jogou (W17). O `to_wire` é a única tradução
            // — o `PlayerInput` da crate da LEI não conhece serde de propósito.
            player_tape: self.player_tape.to_wire(),
        };
        let bytes = match postcard::to_allocvec(&(PROJECT_SCHEMA, &file)) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[proj] falha ao serializar: {e}");
                return;
            }
        };
        let path = Self::project_path();
        match std::fs::write(&path, &bytes) {
            Ok(()) => {
                eprintln!("[proj] salvo: {path} ({} bytes)", bytes.len());
                let n = self.timeline.doc.bindings().len();
                self.toast(format!(
                    "Project saved · {} KB · {n} animation track(s)",
                    bytes.len() / 1024
                ));
            }
            Err(e) => {
                eprintln!("[proj] erro ao gravar {path}: {e}");
                self.toast(format!("Project save FAILED: {e}"));
            }
        }
    }

    /// Um aviso na tela — não só no terminal. O Ctrl+O é destrutivo (troca a cena, zera o undo)
    /// e o Ctrl+S é silencioso; um `eprintln!` num app de janela é uma mensagem para ninguém.
    fn toast(&mut self, msg: String) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(ph2d_editor::Toast::info(msg));
        }
    }
}

/// **O lado da LEITURA** — irmão pelo teto de LOC (HR-18); o corte é por
/// responsabilidade: aqui fica *o que um arquivo É e como ele é escrito*, lá *como
/// ele é lido e a sessão esquece o documento anterior*.
#[path = "project_load.rs"]
mod load;

/// **Os pixels que o undo não guarda** — irmão pelo teto de LOC (HR-18); o corte é por
/// responsabilidade: aqui fica *o que um arquivo É*, lá *como os pixels vão e voltam do atlas*.
#[path = "project_assets.rs"]
mod assets;

#[cfg(test)]
#[path = "project_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "project_schema_tests.rs"]
mod schema_tests;
