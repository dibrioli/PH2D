# Escada do `PROJECT_SCHEMA` — degraus **v2 a v59** (arquivo)

> ⚠️ **Isto é história, verbatim.** Os degraus VIVOS (v60 em diante) e o literal continuam
> **colados um ao outro** em [`shells/desktop/src/project_schema.rs`](../../shells/desktop/src/project_schema.rs)
> — é lá que se conta o próximo, e a razão está escrita no degrau v69: ele chegou ao `main`
> com a linha da escada AUSENTE, e *quem conta o próximo degrau lê a escada, não o literal*.
>
> Cortado em 2026-08-23, quando o ficheiro bateu o teto de 600 LOC do HR-18 com **589 das 608
> linhas em doc-comment**. A regra da casa é cortar para o irmão (ou para o arquivo), nunca
> declarar exceção — e um degrau de 2026-05 não é o que uma linha de hoje precisa de ler.

v2 (ADR-0114): `ProjectState` ganhou o campo `flip: FlipDoc` (3º) — postcard é
posicional, então um arquivo v1 não desserializa. Sem custo real: a
persistência ainda é stub (sem diálogo de arquivo), sem saves publicados.
v3: `ProjectFile` ganhou os **documentos do Painter** (3º campo). Sem eles o projeto salvava um
sprite apontando para uma textura de runtime que morre com o processo — pintar, salvar e reabrir
devolvia o quadro em branco. Ver [`crate::project_painter`].
v4: o `Layer` do Painter ganhou o **Impasto por camada** (`impasto_depth` / `impasto_composite` /
`has_relief`) — postcard é posicional, então um documento pintado v3 lê lixo nos campos seguintes.
Rejeitar é a única leitura honesta.
v5 (doc 56): `ProjectFile` ganhou `motion` (o grafo de Motion Nodes, em texto) — 4º campo. Pelo
mesmo motivo posicional, um v4 não desserializa aqui.
v6 (ADR-0114 W3): a `FlipDoc` — que vive DENTRO do `ProjectState` — mudou de forma: a camada
ganhou `cycle` + `use_onion` e o `OnionSettings` ganhou `kind_filter` (`FLIP_SCHEMA_VERSION` 1→2).
Não é campo novo no ARQUIVO, é o MESMO campo com outro layout — e posicional é posicional.
v7 (ADR-0114 W4): o `FlipStroke` ganhou `holes` + `hide_stroke` (o balde —
`FLIP_SCHEMA_VERSION` 2→3). Mesma regra: a forma mudou, então a versão sobe.
Sem o bump, um arquivo v6 passaria na checagem de versão e seria lido com o
layout NOVO — postcard não tem nomes de campo para reclamar, e o que sai é
geometria embaralhada em vez de um erro honesto.
v8 (ADR-0114 W6): o `FlipStroke` ganhou `selected` — a seleção é ATRIBUTO do traço
(o Edit Mode; `FLIP_SCHEMA_VERSION` 3→4), e não estado do shell. Idem: a forma do
`FlipDoc` mudou dentro do `ProjectState`, então o par sobe junto.
v9 (ADR-0114 W7.2): a CHAVE (`FlipFrame`) ganhou `offset` — a **pose do quadro**
(`FLIP_SCHEMA_VERSION` 4→5). É o que faz uma instância (duas chaves, um desenho) ser
mais que um hold: a arte é compartilhada e o lugar é de cada quadro.
v10 (ADR-0121): o `VecVertex` ganhou `corner_radius` (Live Corners —
`ph2d_vec_scene::corner_live`), e a `VecScene` vai embutida aqui
(`VEC_SCENE_SCHEMA_VERSION` 7→8). Mesma regra.
v11: o `PaintedDocument` ganhou `mats` — o MATERIAL do Impasto por camada
(Roughness/Metallic/Wax/Shine, por pixel). Sem o bump, um save anterior seria lido com o
layout novo e o material sairia dos bytes da COBERTURA. (O `Shine` deixou de ser global e
virou propriedade da TINTA — Enio, 2026-07-13.)
v12: o MESMO `mats` mudou de FORMA — 4 bytes → 7 (a **cor do Wax**, o filtro sobre a luz que
atravessa a tinta). Não é campo novo, é o mesmo campo com outro layout, e posicional é
posicional: sem o bump um v11 passaria na checagem e o material sairia dos bytes errados.
v13 (W4.T6/B5): `ProjectFile` ganhou `timeline` (o `TimelineDoc` em postcard) — 5º campo.
**A animação era perdida ao fechar o app**: nada a salvava (o "sidecar" que dizia salvá-la
era código morto — o Ctrl+S global já retornava antes). Os bytes trazem a própria versão
(`DOC_VERSION`), então um bump lá é RECUSADO com erro honesto e não obriga a bumpar aqui;
o campo NOVO, sim, obriga (posicional).
v14 (W7.5): a **pose da chave** do Flip virou AFIM (`FlipFrame.pose: Pose([f32;6])`, era
`offset: Vec2`) — o `ProjectState` embute o `FlipDoc`, e postcard é posicional: 8 bytes → 24
por chave posada. Sem o bump um v13 leria os coeficientes do afim como o `Vec2` + lixo.
v15 (W8): o traço do Flip ganhou `point_sel` (seleção no domínio Point, FLIP v6→7) —
campo novo no `FlipStroke`, layout posicional muda.
v16 (ADR-0131 W1): `RigidBody`/`Collider` foram REGISTRADOS no
`ComponentRegistry`, então uma cena com corpos físicos grava blobs novos
nas linhas do `WorldSnapshot` — um leitor v15 leria esses bytes na posição
errada. O `PhysicsBridge` em si NÃO é serializado (é derivado das
components no load); só os components viajam.
v17 (ADR-0131 W2): o `Collider` ganhou `restitution`/`friction` APENDADOS —
campo novo, layout posicional muda.
v18 (§4.C.6 do Flip): a **UNIDADE** do `Point.width` do Flip mudou (px de tela → MUNDO,
FLIP v7→8). ⚠️ **O layout NÃO mudou** — e é por isso que o bump é obrigatório: postcard lê
o `f32` antigo com sucesso e o interpreta na unidade nova, ~100× mais grosso, sem um
erro sequer. Todos os bumps anteriores quebravam LAYOUT (falham alto); este quebra
SIGNIFICADO (falharia calado). Arquivo v17 é recusado — ver o `load`.
v19 (ADR-0131 W2b): o arquivo carrega as **settings de MUNDO** da física
(`ProjectFile.physics`) — gravidade, solver, arrasto, sono. Campo novo,
layout posicional muda. Sem ele o painel do W2b seria um painel de knobs que
ESQUECEM: gravidade zero para um jogo top-down é uma decisão do projeto, e
perdê-la ao reabrir é o mesmo que não tê-la.
v20 (ADR-0131 W2b, pós-smoke): `PhysicsSettings` ganhou `air_drag` APENDADO —
o campo entra no layout de `ProjectFile.physics`. Nenhuma constante de esquema
mudou, então **nenhum gate podia ver isto**: postcard é posicional e um save
v19 lido como v20 devolveria lixo bem-formado.
v21 (ADR-0131 W2c): camadas de colisão — `Collider.layer` APENDADO ao
component (blob novo nas linhas do `WorldSnapshot`) **e**
`PhysicsSettings.layer_matrix` apendado. Duas quebras de layout no mesmo
bump, nenhuma visível a um gate de constante.
v22 (ADR-0132): `VecPath` ganhou a pilha de efeitos (Live Path Effects). v23: a entrada da
pilha virou `FxEntry` (o efeito + se está LIGADO). v24: a pilha ganhou os variants
`Repeat`/`Twist`/`Bloat` — apender variant não move os índices anteriores, então um arquivo
v23 continua a ser lido CERTO; o bump existe para que o caminho inverso (um v24 aberto por um
binário v23) morra como erro de versão em vez de como um postcard perdido.
⚠️ Estas entradas nasceram como v19..v21 na `line/Vector` e foram **renumeradas +3 na
integração de 2026-07-19**, porque a `line/physics` bumpou três vezes na mesma jornada e o
contador se **CONTA**, não se escolhe.
v27 (ADR-0131 W7): triggers — `Collider.is_sensor` APENDADO ao component (blob novo nas
linhas do `WorldSnapshot`), mesmo padrão do v21 (`layer`). Layout posicional muda: um save
v26 lido como v27 leria além do fim do blob do `Collider`; um v27 lido por um binário v26 é
recusado como erro de versão em vez de virar um postcard perdido.
v28 (ADR-0131, Weld): `JointKind` ganhou o variant `Weld` APENDADO (discriminante 3). Apender
variant NÃO move os índices anteriores, então um save v27 (Pin/Spring/Rope) continua a ser
lido CERTO; o bump existe pro caminho INVERSO — um save com um Weld, aberto por um binário v27,
morre como erro de VERSÃO em vez de como um postcard perdido no discriminante 3 desconhecido
(mesmo raciocínio do v24, os variants do vetor).
v30 (ADR-0131, gold-standard joint anchors): `PhysicsJoint` ganhou `local_a`/`local_b`/`anchored`
APENDADOS — a âncora deixou de ser um ponto de MUNDO re-derivado (o `Transform` do joint) e
passou a ser autorada BODY-LOCAL por corpo (a rep nativa do rapier), pra a âncora seguir o
corpo quando ele se move. Layout posicional muda (mesmo padrão do v27/`is_sensor`): um save v29
lido como v30 leria além do fim do blob do `PhysicsJoint`; um v30 lido por um binário v29 é
recusado como erro de versão.
v31 (Flip, 03 §8): o `FlipStroke` ganhou `tip` + `dot_spacing` (o pincel pontilhado) — campos
no MEIO do struct (após `hardness`), então o layout posicional do `FlipDoc` embutido muda. É o
mesmo motivo dos bumps anteriores do `FlipStroke` (v7 `holes`/`hide_stroke`, v8 `selected`):
`FLIP_SCHEMA_VERSION` 8→9, e `PROJECT_SCHEMA` acompanha porque o `FlipDoc` viaja DENTRO do
`ProjectState`.

⚠️ Este bump nasceu `30` na `line/FLIP` e virou **31** na integração de 2026-07-25: a
`line/physics` reivindicou o mesmo 30 na MESMA janela, por outro motivo (a âncora body-local
do joint, o parágrafo acima). O valor certo se CONTA, não se escolhe — ele não estava em
nenhum dos dois lados do conflito ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
v32 (ADR-0131, W-J6 servo + guincho): `PhysicsJoint` ganhou `motor_mode` +
`motor_target` APENDADOS — o motor deixou de ser só uma TAXA e passou a poder
mirar um LUGAR, e passou a existir também no Slider e na Rope. Mesmo padrão
posicional do v30: dois campos a mais no fim do blob, então um save v31 lido
como v32 leria além do fim dele, e um v32 lido por um binário v31 é recusado
como erro de versão em vez de virar um postcard perdido.
v33 (ADR-0131, W-J7 break force): `PhysicsJoint` ganhou `break_enabled` +
`break_force` + `break_torque` APENDADOS — um joint pode ser autorado para
ROMPER sob carga. Mesmo padrão posicional do v30/v32: três campos a mais no
fim do blob. ⚠️ O `∞ = off` NÃO é serializado: o componente guarda um
booleano e dois números finitos, e a ponte é quem os resolve na infinidade
que o solver quer — guardar `f32::INFINITY` faria o painel ter de mostrar
"inf" numa row numérica.
v34 (ADR-0131, W-J8 higiene do par): `PhysicsJoint` ganhou `active` +
`collide_connected` APENDADOS — desligar a restrição sem apagar o objeto, e
escolher se os dois corpos que ela une ainda se batem. Mesmo padrão
posicional do v30/v32/v33: dois campos a mais no fim do blob. ⚠️ O **Swap
A↔B** da mesma wave NÃO move nada aqui — ele reescreve campos que já
existem (as duas pontas, as duas âncoras, e os sinais medidos entre elas),
que é exatamente por que um bump se CONTA em vez de acompanhar a wave.
v35 (Flip, 2.5D multiplane, ADR-0114 §Decisão 3): a `FlipLayer` ganhou `depth` (a fração de
paralaxe da câmera) APENDADO — o `FlipDoc` viaja no `ProjectState`, então o layout posicional
muda e um save v34 lido como v35 leria `depth` além do fim do buffer. `FLIP_SCHEMA_VERSION`
9→10, e o `PROJECT_SCHEMA` acompanha. ⚠️ A `line/FLIP` escreveu **32** aqui e a
`line/physics` reivindicou o MESMO 32 na mesma janela (o servo do W-J6) — a SEGUNDA vez
que estas duas linhas colidem no mesmo número, depois do 30 de 25/07. O valor certo se
CONTA a partir do `main` do dia (34 + 1), e não estava em nenhum dos dois lados.
v36 (Flip, Self Overlap, 03 §8): o `FlipStroke` ganhou `self_overlap` (auto-sobreposição com
acúmulo) no MEIO do struct (após `dot_spacing`) ⇒ layout posicional muda, um save v35 leria os
campos seguintes deslocados. `FLIP_SCHEMA_VERSION` 10→11.
v37 (Flip, Airbrush, 03 §8): o `FlipStroke` ganhou `airbrush` (falloff físico Beer-Lambert por
dab esférico) no MEIO do struct (após `self_overlap`) ⇒ mesmo raciocínio posicional.
`FLIP_SCHEMA_VERSION` 11→12.
v38 (Vector, plano 24 W6 — a LEI DE MISTURA por degrau): o `ph2d_ecs::FxOp` ganhou `blend`
APENDADO — um degrau da pilha de FX raster passa a dizer *como a cor dele encosta na que já
está ali* (Inner Shadow em Multiply escurece em vez de lavar; Color Overlay em Color troca a
matiz preservando a luminosidade). O `VecFilter` é componente registado, e postcard é
POSICIONAL, então um save v37 lido como v38 leria `blend` além do fim de cada degrau.
⚠️ Não há como evitar o bump com `serde(default)`: o postcard não tem NOMES de campo, e um
buffer que acaba cedo é erro de decode, não um default.
⚠️ **E o 38 carrega TAMBÉM a turbulência (plano 24 W6b), o Grow / Shrink (W7), o Color Adjust
(W8) e o Duotone (W9), de propósito:** o mesmo `FxOp` ganhou `scale`/`detail`/`seed`, depois
`grow`, depois `hue`/`sat`/`bright` e por fim `color_b` na MESMA janela, e um save v37 já é
recusado pelo 38 — pôr cada leva num número próprio jogaria fora exatamente os mesmos
arquivos. **Uma linha, um bump**; o que não pode acontecer é o número não subir.
v39 (physics, W-Rod): `JointKind` ganhou a variante **`Rod`** (a barra rígida). Apender
variante NÃO move os índices das existentes — o bump é para o caminho INVERSO: um build
antigo lendo um arquivo novo veria o discriminante 5 e devolveria lixo bem-formado em vez
de recusar. É o mesmo raciocínio do Weld (v27→28) e do Slider, e é por isso que a recusa
tem de ser ALTA. `FLIP_SCHEMA_VERSION` fica em 12.
v42 (physics, W-Pulley W1): o `PhysicsJoint` **PERDEU** `wheel_a`/`wheel_b`/`ratio` — uma
roldana virou ENTIDADE (`PulleyWheel` + `Transform`), o que remove o teto de duas e dá ao
artista contar/posicionar/dimensionar cada uma. Componente NOVO não custa bump (blob-key
própria); o que custa é a REMOÇÃO dos três campos, porque postcard é posicional e um blob
v41 traz três a mais. E o `ratio` saiu por ser FÍSICA ERRADA: numa corda única sobre
roldanas livres a tensão é uniforme, então não há vantagem mecânica a ganhar de diâmetro
nenhum. `FLIP_SCHEMA_VERSION` fica em 12.
v40 (physics, W-Wheel): `JointKind` ganhou a variante **`Wheel`** (o cubo que gira E cavalga
uma suspensão). Mesmo raciocínio do v39, um degrau adiante: apender variante não move
índice nenhum, e o bump existe para o build antigo RECUSAR em vez de ler o discriminante 6
como lixo bem-formado. `FLIP_SCHEMA_VERSION` fica em 12.
v43 (physics, W-Pulley W2): a `PulleyWheel` ganhou **`motor_speed`** — a roldana
dirigida, o guincho. Componente NOVO não custa bump (blob-key própria), mas
APENDAR campo a um componente que já existe custa: postcard é **posicional**, e
um blob v42 tem um `f32` a menos, então o load leria lixo bem-formado no lugar
de recusar. Mesmo raciocínio do `is_sensor` (v27) e do `offset` (v29).
v44 (physics, W-Pulley W2): a `PulleyWheel` ganhou **`break_enabled`** e
**`break_force`** — o eixo que cede sob carga. Dois campos apendados, mesmo
raciocínio posicional do v43.
v45 (physics, W-Pulley W3): a `PulleyWheel` ganhou **`body`**, **`local`** e
**`mounted`** — a roldana montada num corpo que se move, a *cadernal móvel* de
uma talha, e com ela a vantagem mecânica que o `ratio` prometia e não
entregava. Três campos apendados, mesmo raciocínio posicional do v43/v44. O
par `local`/`mounted` é o do W-AnchorFollow: o eixo é guardado no frame do
CORPO e convertido uma vez, senão mover o bloco o faz deslizar por ele.
v46 (physics, W-Pulley W4): a `PulleyWheel` ganhou **`radius_out`** — o SEGUNDO
diâmetro do eixo, que faz dela um **tambor diferencial** e devolve a vantagem
mecânica CONTÍNUA que o `ratio` do W-Pulley prometia sem ter peça na cena. Um
campo apendado, mesmo raciocínio posicional do v43/v44/v45.
v47 (physics, W-SoftWeld): o `PhysicsJoint` ganhou **`soft`** — a solda que
CEDE, o vão que este conjunto tinha entre segurar um ângulo *absolutamente* e
deixá-lo *inteiramente livre*. Um campo apendado, mesmo raciocínio posicional
do v43..v46; a dureza reusa a `stiffness`/`damping` que a mola já carregava,
então é UM bool e não três campos.
v48 (FLIP, as PONTAS do traço): o `Cap` ganhou a variante **`Square`**, e o
`FlipDoc` viaja DENTRO do `ProjectFile` ⇒ o `FLIP_SCHEMA_VERSION` 12→13 arrasta
este junto (a escada dele está no `ph2d_flip::FLIP_SCHEMA_VERSION`).
⚠️ **As duas linhas escreveram 47 na mesma janela** — a `line/FLIP` e a
`line/physics` (v47 acima) —, e o valor se **CONTA** a partir do `main` do dia:
a física ficou com o 47 e o Flip foi contado para o 48. É a 3ª vez entre estas
duas ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]), e desta vez
ela quase passou MUDA: os dois lados escreveram o mesmo literal, então o
`project.rs` **não conflitou** — quem denunciou foi o gate da tripla ao lado.
v49 (vector, W6.2 — as guias e a régua): o `ProjectState` ganhou **`guides`**, a
lista de linhas de referência que o artista arrasta da régua. Campo apendado ao
`ProjectState`, que viaja DENTRO do `ProjectFile` — o mesmo raciocínio posicional
do `flip`, e o mesmo motivo de estar ali e não num campo de arquivo próprio: o
`ProjectState` é a unidade do UNDO, e uma guia arrastada tem de desfazer.
v50 (vector, W6.4 — o alinhamento do traço): o `StrokeSpec` ganhou **`align`**
(Centre/Inner/Outer), e ele mora dentro do `VecScene` — que viaja no `ProjectState`. O bump
é obrigatório nos DOIS sentidos e pelo motivo medido em `VEC_SCENE_SCHEMA_VERSION` v14: o
postcard não sinaliza ausência, então um save antigo lido pelo novo chega ao fim dos bytes
no campo novo e o novo lido pelo antigo traz um byte a mais. O número transforma os dois num
erro de VERSÃO em vez de num postcard a falhar longe da causa.
v51 (plano UI/UX W6): o arquivo carrega a **tabela de COR autorada** (`tokens`), campo
apendado ao `ProjectFile`. Postcard é posicional ⇒ o bump é obrigatório nos dois sentidos,
pelo mesmo motivo do v50 logo acima.
v52 (3D, W8.3 — o documento da escultura): campo de ARQUIVO novo, `sculpt`, um blob
opaco que carrega a própria versão (`SCULPT_DOC_VERSION`) — o precedente do
`TimelineDoc`, e é ele que deixa o módulo evoluir muitas waves sem tocar este número
de novo (docs/3D/02.3 previu exatamente isto). O bump é obrigatório porque o postcard
é POSICIONAL: um campo novo no fim faz o leitor velho chegar ao fim dos bytes.
⚠️ O número se CONTA contra o `main` do dia, não se escolhe — este 52 era
PROVISÓRIO até a integração ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
v53 (3D, W8.7 — os canais assados): campo de ARQUIVO novo, `baked_forms`, com o `base`, a
`form` e o RIG de cada objeto que uma malha acendeu (`docs/3D/02.2`, rota A). Bump
obrigatório pela mesma razão de sempre — postcard é POSICIONAL, e um campo novo no fim faz o
leitor velho chegar ao fim dos bytes. ⚠️ Ele NÃO entrou no blob `sculpt` acima, embora
aquele já guarde as malhas: o parser dele é `#[cfg(feature = "sculpt3d")]`, e um objeto
assado tem de ser legível **sem o módulo 3D no build** — é isso que a rota A promete.
⚠️ Este 53 é PROVISÓRIO pelo mesmo motivo que o 52 era.
v54 (physics, W-JointCustom — o joint que o artista descreve por EIXO): o
`PhysicsJoint` ganhou **`custom`**, a configuração por grau de liberdade
(*Free / Limited / Locked*, o modelo do Unreal) que expõe o `GenericJoint` do
rapier. Campo apendado ao componente, mesmo raciocínio posicional dos
v32/v33/v34: um save v53 lido por v54 chega ao fim dos bytes no campo novo.
v55 (physics, W10 — as duas assistências que faltavam): o `PlatformPlayer`
ganhou **`corner_reach`** e **`lift_momentum`**, os dois campos que o W8 tinha
nomeado e não construído. Dois campos apendados ao componente, mesmo
raciocínio posicional — e o mesmo motivo de estarem NELE e não num componente
próprio: são knobs da mesma lei que os outros dezenove, e o custo de um
componente novo por knob é uma lista que ninguém lê.
⚠️ A linha escreveu **52**; o valor CONTADO contra o `main` da integração é 55.
v56 (plano UI/UX W7 — os ESTADOS de UI): o `ProjectState` ganhou **`ui_states`**, a tabela
de idle/hover/press por hospedeiro. Campo apendado ao `ProjectState`, que viaja DENTRO do
`ProjectFile` — o mesmo raciocínio posicional do `guides` (v49), e o mesmo motivo de estar
ali e não num campo de arquivo próprio: o `ProjectState` é a unidade do UNDO, e **gravar um
estado tem de desfazer**.
⚠️ **Nenhum gate viu este bump, e é por isso que ele está escrito à mão:** um campo APENDADO
não move constante nenhuma, então a suíte inteira fica verde com o arquivo já incompatível —
o postcard é posicional e devolveria lixo bem-formado. Quem apende, bumpa, no MESMO commit.
⚠️ E o valor é **PROVISÓRIO**: ele se CONTA contra o `main` do dia da integração, não se
escolhe ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
v57 (plano UI/UX W4b — o ALIAS de token): um token autorado passa a valer uma cor **ou** o
nome de outro token, então o `SavedToken` troca o campo `rgba: [u8; 4]` pelo enum
`SavedValue`. ⚠️ Um enum e não um campo `alias` ao lado do `rgba`: os dois seriam mutuamente
exclusivos e nada no formato o diria. Postcard é POSICIONAL e a forma do registro mudou ⇒ o
bump é obrigatório nos dois sentidos, o mesmo raciocínio do v50/v51.
⚠️ **PROVISÓRIO** pelo mesmo motivo que o v56.
v58 (plano UI/UX W4c.1 — a camada NUMÉRICA): a escala (`spacing.*`, `radius.*`, `stroke.*`)
passa a ser autorável, e o valor autorado viaja na **MESMA lista** `tokens` — o `SavedValue`
ganha a variante **`Number(f32)`**, e a chave (`"spacing.md"`) é quem diz de que família a
entrada é. ⚠️ Uma lista só, e não um campo `num_tokens` ao lado: o que o arquivo guarda é *"que
tokens o artista autorou"*, e duas listas para isso seriam duas respostas à mesma pergunta que
o import/export DTCG (W4c.5) teria de juntar de novo.
⚠️ **Apendar variante NÃO move `Literal`(0) nem `Alias`(1)**, então todo arquivo já salvo
continua a ler — o bump é pelo caminho **INVERSO**: um build antigo a ler um arquivo novo
bateria num índice de variante que ele não tem, e o número transforma isso num erro de VERSÃO
em vez de num postcard a falhar longe da causa (o raciocínio do `JointKind::Weld`/`Cap::Square`).
⚠️ **PROVISÓRIO** pelo mesmo motivo que o v56.
v59 (plano UI/UX W4c.3 — a MATH): um token numérico passa a poder valer uma **fórmula**
(`{spacing.md} * 2`), e o `SavedValue` ganha a variante **`Formula(String)`**. ⚠️ TEXTO, e não
o IR parseado: é o texto que o artista reabre e edita, e serializar a árvore faria o formato do
arquivo depender da forma de um tipo do parser — a decisão que a `motion.expression` já tomou.
⚠️ **Apendar variante NÃO move `Literal`(0)/`Alias`(1)/`Number`(2)**, então todo arquivo já
salvo continua a ler; o bump é pelo caminho INVERSO, o mesmo raciocínio do v58 logo acima.
⚠️ **PROVISÓRIO** pelo mesmo motivo que o v56.
