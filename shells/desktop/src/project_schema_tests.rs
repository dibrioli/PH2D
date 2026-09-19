//! **O gate da TRIPLA de schema** — `PROJECT_SCHEMA` × `FLIP_SCHEMA_VERSION` ×
//! `VEC_SCENE_SCHEMA_VERSION`.
//!
//! Irmão do [`super::tests`], separado por assunto quando o arquivo bateu o cap de
//! 600 LOC: ali ficam os gates de *o que um load FAZ* (o relógio, o histórico, a
//! timeline, as settings); aqui, o único que fala de *que NÚMERO o arquivo
//! carrega* — e ele cresce um parágrafo por wave, porque a narrativa da escada é
//! o valor dele.

use super::*;

/// **Estopim de esquema.** O `ProjectState` embute o `FlipDoc` E a `VecScene` inteiros, e o
/// postcard é POSICIONAL: qualquer campo novo em qualquer struct deles muda o layout do
/// arquivo de projeto. Sem bump, o loader aceita o arquivo velho (a versão bate) e o lê com
/// o layout novo — sai geometria embaralhada, não um erro. Foi o que quase aconteceu na W4
/// (`holes`/`hide_stroke`).
///
/// Esta tripla existe para que bumpar UM sem pensar nos OUTROS fique vermelho. E o pin só
/// protege quem ele NOMEIA: enquanto ele era um par (só o Flip), um campo novo no
/// `VecVertex` teria bumpado o `VEC_SCENE_SCHEMA_VERSION`, deixado o `PROJECT_SCHEMA` para
/// trás, e **este teste teria passado**.
///
/// O `PROJECT_SCHEMA` é **14** — e não o 8 que esta linha trazia sozinha, nem o 9 que outras
/// duas traziam. Ele conta TODAS as quebras de layout do arquivo, de TODOS os módulos:
/// v3/v4 do Painter (documentos + impasto) · v5 do Motion (o grafo) · v6/v7 e v8/v9 do Flip
/// (o balde; depois `selected` + `offset`) · v10 do Vector (o `corner_radius` do `VecVertex`)
/// · v11/v12 do Painter (o `mats` do impasto, e o `mats` mudando de FORMA: 4 → 7 bytes) ·
/// v13 a timeline (5º campo do `ProjectFile`) · v14 a pose AFIM do Flip (W7.5:
/// `FlipFrame.offset: Vec2` → `pose: Pose([f32; 6])`, FLIP v5→6) · v15 a seleção no
/// domínio Point do Flip (W8: `FlipStroke.point_sel`, FLIP v6→7) · v16 os corpos de
/// física (ADR-0131 W1: `RigidBody`/`Collider` registrados → blobs novos nas linhas do
/// `WorldSnapshot`; nem o FlipDoc nem a VecScene mudaram, mas o layout do arquivo sim) ·
/// **v17** os campos `restitution`/`friction` APENDADOS ao `Collider` (ADR-0131 W2, a autoria
/// no Inspector). Nenhuma constante de esquema mudou, então **nenhum gate podia ver isto** —
/// postcard é posicional, e um save v16 lido como v17 devolveria lixo bem-formado. · **v18** a
/// UNIDADE do `Point.width` do Flip (§4.C.6, `cb42c9a2`) — o caso que o PONTO CEGO abaixo
/// narra, e que ninguém tinha acrescentado a esta lista · **v19** as settings de MUNDO da
/// física (ADR-0131 W2b: 6º campo do `ProjectFile`) · **v20** o `air_drag` APENDADO ao
/// `PhysicsSettings` (o smoke do W2b mostrou que o damping uniforme não é ar; o modelo de
/// arrasto real é campo novo) · **v21** a camada + a matriz de colisão (ADR-0131 W2c) ·
/// **v22** a PILHA de Live Path Effects (ADR-0132: `VecPath.effects`,
/// `VEC_SCENE_SCHEMA_VERSION` 8→9) · **v23** a entrada da pilha virou `FxEntry` (o efeito +
/// se está LIGADO — o olho desarma sem perder os parâmetros), `VEC_SCENE_SCHEMA_VERSION` 9→10 ·
/// **v24** os variants `Repeat`/`Twist`/`Bloat` na pilha (`VEC_SCENE_SCHEMA_VERSION` 10→11).
/// (v27 triggers, v28 Weld, v29 offset do collider — ver `project.rs`.) · **v30** a âncora
/// body-local do joint (ADR-0131 padrão-ouro): `PhysicsJoint` ganhou
/// `local_a`/`local_b`/`anchored` APENDADOS, pra a âncora seguir o corpo em vez de deslizar.
///
/// ⚠️ As entradas do Vector nasceram em **v19..v23** na linha dela e foram **renumeradas para
/// v22..v26 na integração de 2026-07-19**: a `line/physics` bumpou três vezes na MESMA jornada,
/// e o contador se **CONTA** — 18 (base) + 3 (física) + 5 (Vector) = 26. Escolher um dos lados
/// faria os saves do outro passarem na checagem de versão e serem lidos com o layout errado.
///
/// Na integração de 2026-07-13, QUATRO linhas bumparam este contador ao mesmo tempo, cada uma
/// a partir do 7, cada uma por um motivo diferente. **O valor certo não existia em nenhum lado
/// do conflito: ele se CONTA.** Escolher um dos lados faria os saves das outras passarem na
/// checagem de versão e serem lidos com o layout errado — e postcard não tem nome de campo
/// para reclamar; ele devolve lixo bem-formado.
/// ⚠️ **PONTO CEGO deste gate — ele já deixou passar um, leia antes de confiar.**
///
/// Ele pina CONSTANTES, então só acorda quando alguém mexe numa. Uma mudança de **UNIDADE**
/// (ou de significado) num campo cujo **layout não muda** atravessa este gate inteira e
/// VERDE — foi o que o §4.C.6 fez, ao trocar o `Point.width` do Flip de px de TELA para
/// unidade de MUNDO. O campo continuou um `f32`, o postcard lia o arquivo antigo **com
/// sucesso**, e a arte saía ~100× mais grossa sem um erro sequer.
///
/// **A regra é mais larga do que este gate consegue verificar:** bumpe o schema quando um
/// arquivo antigo passar a ser lido **ERRADO** — não só quando deixar de ser lido. Quebra
/// de LAYOUT falha alto e o gate a pega; quebra de SIGNIFICADO falha calada, e só quem faz
/// a mudança pode pegá-la.
#[test]
fn a_schema_bump_anywhere_must_bump_the_project_schema() {
    assert_eq!(
        (
            PROJECT_SCHEMA,
            ph2d_flip::FLIP_SCHEMA_VERSION,
            ph2d_vec_scene::VEC_SCENE_SCHEMA_VERSION,
        ),
        // ⚠️⚠️ **A NARRATIVA dos degraus `30`..`119` está ARQUIVADA verbatim** em
        // `docs/archive/project-schema-tripla-v119.md` desde 2026-09-19: este ficheiro é UM gate
        // cujo corpo é a história, logo o tecto de LOC dele mede o TAMANHO DA HISTÓRIA e não um
        // autor. ⛔ A cura foi CORTE, como a escada do `PROJECT_SCHEMA` já fez quatro vezes.
        // ⛔⛔ E ela foi para `docs/`, e não para um ficheiro IRMÃO: a 1.ª tentativa fez isso e a
        // catraca `the_shell_only_shrinks` reprovou — *mover narrativa de um ficheiro da shell
        // para outro ficheiro da shell não é um corte, é uma mudança de endereço*.
        // ⚠️ **`128` e nao `124` nem `127`** — integracao de 2026-09-10: a `line/Vector` subiu o
        // PROJECT_SCHEMA `+4` e esta linha `+1`, no mesmo dia. O valor certo nao estava em nenhum
        // dos dois lados, e e' esta a linha que as duas editaram, logo o git conflitou em vez de
        // fundir mudo. Os dois numeros ao lado NAO se mexem: nenhum dos cinco degraus mudou a
        // forma da `VecScene` nem do `FlipDoc`.
        // ⚠️ **`129` desde 2026-09-13** — as TAGS (TOP-20 #9): a árvore entrou no `ProjectState` (um
        // campo no MEIO do fluxo de bytes, logo com tipo congelado e migração) e o `SignalAction`
        // ganhou o alvo por TAG. Os dois números ao lado NÃO se mexem: nada na forma do `FlipDoc`
        // nem da `VecScene` mudou — é a décima terceira vez que esta tripla é cega a um degrau.
        // ⚠️ **`130` desde 2026-09-14** — a FÁBRICA e o CICLO DE VIDA (TOP-20 #11 e #12): três
        // componentes registados novos (`Factory`, `Lifetime`, `DestroyOutside`). Os dois números
        // ao lado NÃO se mexem, e é a décima quarta vez. ⛔ O `Spawned` — o quarto tipo da wave —
        // **não é registado**, e é por isso que o degrau vale `+1`: ele marca o que uma corrida
        // pôs na cena, e o `world_to_snapshot` poda essas subárvores.
        // ⚠️ **`131` desde 2026-09-15** — o MOVER DE VISTA DE CIMA (TOP-20 #13): UM componente
        // registado novo (`ph2d::physics::TopDownPlayer`). Os dois números ao lado NÃO se mexem, e
        // é a décima quinta vez. ⛔ O `TopDownState` — a memória da velocidade — **não é
        // componente**: ela vive na ponte, dentro do anel de checkpoints, porque um campo que muda
        // por tique num componente registado faria o undo ver cada quadro como um passo.
        // ⭐⭐⭐ **132 → 133 — o CÉREBRO AUTORÁVEL** (TOP-20 #15, `line/components`, 2026-09-15):
        // UM componente registado novo (`ph2d::ecs::StateMachine`). O Flip e a VecScene não se
        // mexem, e **a tripla não vê este degrau** — é a **décima sétima** vez. ⛔ O
        // `StateMachineRuntime` — em que estado se está agora — **não é componente registado**: a
        // cerca é o TIPO (ele não deriva `Serialize`), e registá-lo faria **cada transição** virar
        // um passo de `Ctrl+Z`. Ele é reposto ao rebobinar pela porta do `ph2d_ecs::rewind_runtime`.
        // ⭐⭐⭐ **133 → 134 — o SCRIPT DO ARTISTA passa a ser GRAVADO** (TOP-20 #16, 2026-09-16):
        // nenhum tipo novo — o `LuauScript` já existia e o registador dele não corria no boot, logo
        // o snapshot o descartava. A tripla não vê este degrau: a **décima oitava** vez.
        // PROJECT 135→136: o `Bone` ganhou `segments` + `curve` (o *bendy bone*, F8). Dois
        // campos apendados a um componente ⇒ o postcard, que e' posicional, leria um ficheiro de
        // dois campos como tendo quatro. ⚠️ **A tripla NAO ve^ este degrau** -- os bytes mudaram
        // dentro de um `ComponentBlob`, opaco para ela. E' a DECIMA TERCEIRA vez (ver a escada).
        // PROJECT 136→137: a malha de uma IMAGEM presa passou a levar os pesos do padrao-ouro
        // dentro (`SkinnedMesh` no lugar de `Mesh2d`); o postcard e' posicional. ⚠️ **A tripla NAO
        // ve^ este degrau** (14.a vez): os bytes estao DENTRO de um `ComponentBlob`.
        // PROJECT 137→138: a forma VECTORIAL presa passou a levar os pesos do padrao-ouro dentro
        // (`SkinnedPath` no lugar de `VecPath`) — fecha a divergencia que o 137 abriu, e o postcard
        // e' posicional. ⚠️ A tripla NAO ve^ (15.a vez): bytes dentro de um `ComponentBlob`.
        // PROJECT 138→139: o `Bone` ganhou `handles` (de onde ve^m as duas alcas de curvatura —
        // autoradas ou derivadas da corrente). Um campo APENDADO a um componente ⇒ o postcard, que
        // e' posicional, leria um ficheiro velho com um campo a mais e comeria os bytes do vizinho.
        // ⚠️ **A tripla NAO ve^ este degrau** (16.a vez): os bytes estao dentro de um
        // `ComponentBlob`.
        // PROJECT 139→140: o `Bone` ganhou `curve_tip` (QUEM manda na ponta da curva — a corrente,
        // ninguem, ou um filho escolhido pelo `StableId` dele). Outro campo APENDADO ⇒ a mesma
        // razao do 139. ⚠️ **A tripla NAO ve^ este degrau** (17.a vez): bytes dentro de um
        // `ComponentBlob`.
        // PROJECT 140→141: o `ph2d::field::FieldMaterial` ganhou `emission` + `emission_color`
        // (o brilho proprio, `docs/Render3d/05` §20) -- quatro `f32` APENDADOS a um componente
        // registado, e o postcard e' posicional E sem comprimento: um blob v128 tem 20 bytes onde
        // este binario pede 36.
        // ⚠️ **A tripla NAO ve^ este degrau** — a 18.a vez (a escada conta-as), e pela
        // razao de sempre: os bytes vivem dentro de um `ComponentBlob`, que para ela e' opaco.
        // ⚠️ E o `FIELD_DOC_VERSION` NAO se mexe, apesar de o degrau falar de material: o documento
        // do campo e' GEOMETRIA, e uma cor nao muda uma distancia.
        // PROJECT 141→142: o mesmo `FieldMaterial` ganhou os CINCO numeros do verniz (§21) —
        // `coat`, `coat_roughness`, `coat_color`, `coat_ior`, `coat_darkening`. Mesmo mecanismo do
        // degrau anterior, um dia depois: 36 bytes contra 64.
        // ⚠️ **A tripla NAO ve^ este degrau** — a SEXTA vez.
        // PROJECT 142→143: o mesmo `FieldMaterial` fechou com as ultimas CINCO entradas do
        // OpenPBR (§22) e os campos foram RE-ORDENADOS para a ordem da nodedef — uma quebra de
        // layout mais severa do que apendar. ⭐ E' o ultimo degrau que o material pede.
        // ⭐ **PROJECT 131→132** (2026-09-16): o `ph2d_field::Profile` ganhou os `arcs` — a
        // decomposição exacta em rectas e ARCOS, `FIELD_DOC_VERSION` 22→23. Ele viaja dentro de
        // uma `Primitive`, que viaja posicionalmente no blob do `FieldNode`: é a regra dos degraus
        // 109/110 (campo novo numa struct já gravada). ⚠️ O `FIELD_DOC_VERSION` NÃO está nesta
        // tripla e continua a subir à mão — o instrumento que avisa é o
        // `the_shape_of_a_saved_profile_is_pinned` da `ph2d-field` (90 → 92 bytes).
        // ⭐ **PROJECT 144→145** (2026-09-17): o HUD (TOP-20 #20) — quatro componentes novos no
        // registo (`UiCanvas`/`UiLabel`/`UiButton`/`Counter`) e uma variante APENDADA no fim do
        // `SignalVerb`. ⚠️ **A tripla NÃO vê este degrau** — a SÉTIMA vez: nem o `FlipDoc` nem a
        // `VecScene` mudam de forma, e quem muda é a POPULAÇÃO do registo de componentes, que é
        // medida noutro sítio (`registry_tests`, 91 → 95, e os dois espelhos 92 → 96).
        // ⭐ **PROJECT 145→146** (2026-09-17): a CUTSCENE (TOP-20 #19) — UM componente novo no
        // registo (`SequencePlayer`). ⚠️ **A tripla NÃO vê este degrau** — a OITAVA vez, e pela
        // mesma razão: nem o `FlipDoc` nem a `VecScene` mudam de forma, e quem muda é a POPULAÇÃO
        // do registo (`registry_tests`, 95 → 96, e os dois espelhos 96 → 97).
        // ⭐ **PROJECT 146→147** (2026-09-17): a VIGIA DO CONTADOR — UM componente novo no registo
        // (`CounterWatch`). ⚠️ **A tripla NÃO vê este degrau** — a NONA vez, e pela mesma razão:
        // nem o `FlipDoc` nem a `VecScene` mudam de forma, e quem muda é a POPULAÇÃO do registo
        // (`registry_tests`, 96 → 97, e os dois espelhos 97 → 98).
        // ⭐ **PROJECT 147→148** (2026-09-18): O GATILHO (suplente #24) — UM componente novo no
        // registo (`SignalOnAction`). ⚠️ **A tripla NÃO vê este degrau** — a DÉCIMA vez, e pela
        // mesma razão: nem o `FlipDoc` nem a `VecScene` mudam de forma, e quem muda é a POPULAÇÃO
        // do registo (`registry_tests`, 97 → 98, e os dois espelhos 98 → 99).
        // ⭐ **PROJECT 148→149** (2026-09-19): O SINAL SABE QUEM (suplente #24) — e este degrau é
        // de OUTRA espécie: **zero** componentes novos, logo os três contadores do registo NÃO se
        // mexem. O que muda é a FORMA de um blob já gravado — o `SignalAction` ganha o campo
        // `from`, e o postcard é posicional. ⚠️ **A tripla também NÃO vê este degrau**, mas por uma
        // razão nova: ela mede o `FlipDoc` e a `VecScene`, e quem mudou foi um blob de COMPONENTE.
        // *É a primeira vez em onze waves desta linha que nem a tripla nem o registo o veem* — quem
        // o vê é o `signal_actions_tests`, que fixa os bytes do v128 reescrito.
        // ⭐ **PROJECT 149→150** (2026-09-19): O RAIO (suplente #21) — **DOIS** componentes novos
        // no registo (`RaySensor` e `RaySignals`) e **UM** degrau, porque o número mede o que o
        // FICHEIRO passa a conter e não quantos tipos nasceram (a lei do degrau `130`).
        // ⚠️ **A tripla NÃO vê este degrau** — a DÉCIMA PRIMEIRA vez, e pela razão de sempre: os
        // componentes viajam em `ComponentBlob`s, que para ela são opacos.
        // ⭐ **PROJECT 150→151** (2026-09-19): O TWEEN (suplente #22) — **UM** componente novo no
        // registo (`Tweens`), e ⛔ **nenhum runtime ao lado dele**: o tween é função pura do
        // relógio do `Timers`, logo não há estado vivo para excluir do ficheiro.
        // ⚠️ **A tripla NÃO vê este degrau** — a DÉCIMA SEGUNDA vez, e pela razão de sempre.
        // ⭐ **PROJECT 151→152** (2026-09-19): o CICLO do tween (o *ping-pong* que o dono pediu) —
        // ⛔ **ZERO componentes novos**: é um CAMPO novo no `ph2d_tween::Tween`, que já viajava.
        // ⚠️ **A tripla NÃO vê este degrau** — a DÉCIMA TERCEIRA vez, e por OUTRA razão que as
        // doze anteriores: ali o tipo era novo, aqui o que mudou foi o CONTEÚDO de um blob.
        // ⭐ **PROJECT 152→153** (2026-09-19): O SEGUIDOR DE CAMINHO (suplente #23) — **UM**
        // componente novo no registo (`PathFollow`), e ⛔ **nenhum runtime ao lado dele** nem
        // geometria dentro dele: o que viaja é o NOME da forma desenhada.
        // ⚠️ **A tripla NÃO vê este degrau** — a DÉCIMA QUARTA vez, e pela razão de sempre.
        // ⭐ **PROJECT 153→154** (2026-09-19): O ABANÃO DA VISTA (suplente #25) — **DOIS**
        // componentes novos no registo (`CameraShake` na câmera, `ShakeEmitter` em quem explode), e
        // ⛔ **um TERCEIRO tipo que NÃO se regista**: o `CameraShakeRuntime` guarda o trauma vivo,
        // não deriva `Serialize` (a cerca é o TIPO) e a entrada dele é no `rewind_runtime`.
        // ⚠️ **A tripla NÃO vê este degrau** — a DÉCIMA QUINTA vez, e pela razão de sempre.
        // ⭐ **PROJECT 154→155** (2026-09-19): A ARMA DO JOGADOR — **UM** componente novo no registo
        // (`WeaponFire`) **e** um campo novo na `Factory` (`spread_deg`), e qualquer um dos dois
        // sozinho ja' obrigava o degrau (o postcard e' POSICIONAL). ⛔ **A MUNICAO nao viaja aqui**:
        // ela e' um `Counter`, que ja' se gravava — e e' isso que a poe no HUD de graca.
        // ⚠️ **A tripla NÃO vê este degrau** — a DÉCIMA SEXTA vez, e pela razão de sempre.
        // ⚠️ **PROJECT 144→145** (2026-09-19): o `SkinBind` ganhou `law: SkinLaw` — por que lei
        // cada DESENHO se deforma (ordem do dono). Campo novo numa struct já gravada ⇒ regra dos
        // degraus 109/110. ⚠️ **A tripla NÃO vê este degrau** (a SÉTIMA vez): ele viaja no
        // `WorldSnapshot`, não no `FlipDoc` nem na `VecScene`.
        // ⚠️ **PROJECT 145→146** (2026-09-19): o `IkGoal` ganhou `offset: f64` — o desvio do
        // APONTAR (o `additional_rotation` do `SkeletonModification2DLookAt` do Godot). Campo novo
        // numa struct já gravada ⇒ a mesma regra dos degraus 109/110. ⚠️ **A tripla NÃO vê este
        // degrau** (a OITAVA vez): o `IkGoal` viaja no `WorldSnapshot`.
        (157, 13, 22),
        "a forma do FlipDoc ou da VecScene mudou (ou o esquema do projeto): suba o \
         PROJECT_SCHEMA junto e atualize esta tripla. Postcard nao avisa - ele so le errado."
    );
}
