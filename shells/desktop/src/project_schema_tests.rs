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
        // FLIP 8→9 + PROJECT 30→31: o `FlipStroke` ganhou `tip`+`dot_spacing` (o pincel
        // pontilhado, 03 §8) — campos no MEIO do struct, layout posicional muda.
        // ⚠️ A `line/FLIP` escreveu `30` aqui; a `line/physics` reivindicou o MESMO 30 na
        // mesma janela (âncora body-local do joint), então o valor certo é 31 — e ele não
        // estava em nenhum dos dois lados. O número se CONTA, não se escolhe.
        // PROJECT 31→32: `PhysicsJoint` ganhou `motor_mode`+`motor_target` (W-J6 —
        // o servo, e o motor no Slider/Rope). Campos APENDADOS, o mesmo padrão do
        // v30; `FLIP`/`VEC_SCENE` não se movem porque nada fora da física mudou.
        // PROJECT 32→33: `PhysicsJoint` ganhou `break_enabled`+`break_force`+
        // `break_torque` (W-J7 — o joint que rompe sob carga). Três campos
        // apendados, mesmo padrão.
        // PROJECT 33→34: `PhysicsJoint` ganhou `active`+`collide_connected`
        // (W-J8 — a higiene do par). Dois campos apendados; o Swap A↔B da mesma
        // wave não move schema nenhum, porque só reescreve campos existentes.
        // FLIP 9→10 + PROJECT 34→35: a `FlipLayer` ganhou `depth` (a paralaxe multiplano,
        // ADR-0114 §Decisão 3) — campo apendado, mas postcard é posicional ⇒ v9 lê errado.
        // ⚠️ A `line/FLIP` escreveu 32 aqui e a `line/physics` reivindicou o MESMO 32 (o
        // servo do W-J6) — a 2ª colisão entre estas duas linhas, depois do 30 de 25/07.
        // O valor certo se CONTA a partir do main do dia, e não está em nenhum dos lados.
        // FLIP 10→11 + PROJECT 35→36: o `FlipStroke` ganhou `self_overlap` (auto-sobreposição
        // com acúmulo, 03 §8) — campo no MEIO do struct (após `dot_spacing`), layout posicional
        // muda ⇒ v10 lê os campos seguintes deslocados.
        // FLIP 11→12 + PROJECT 36→37: o `FlipStroke` ganhou `airbrush` (falloff físico
        // Beer-Lambert por dab esférico, 03 §8) — campo no MEIO do struct (após `self_overlap`),
        // mesmo raciocínio posicional.
        // PROJECT 37→38: o `ph2d_ecs::FxOp` ganhou `blend` (a LEI DE MISTURA por degrau da pilha
        // de FX raster, plano 24 W6) — campo APENDADO ao componente `VecFilter`, e postcard é
        // posicional ⇒ um save v37 leria `blend` além do fim de cada degrau. ⚠️ `FLIP` e
        // `VEC_SCENE` NÃO se movem: a lei é do componente ECS, não da `VecScene`.
        // PROJECT 38→39: `JointKind` ganhou a variante `Rod` (W-Rod). Apender variante não
        // move índice; o bump é para o build ANTIGO recusar em vez de ler o discriminante 5
        // como lixo bem-formado. FLIP/VEC_SCENE ficam.
        // PROJECT 39→40: `JointKind` ganhou a variante `Wheel` (W-Wheel — o cubo que gira E
        // cavalga uma suspensão). Mesmo raciocínio, um degrau adiante.
        // PROJECT 40→41: o `PhysicsJoint` ganhou `wheel_a`/`wheel_b`/`ratio` (W-Pulley — a
        // corda por duas roldanas). ⚠️ Aqui o bump NÃO é cortesia como nos dois acima: são
        // CAMPOS apendados a um struct que o postcard codifica POSICIONALMENTE, então um
        // blob v40 tem o comprimento errado e todo joint de todo projeto salvo decodificaria
        // como outra coisa. A variante `Pulley` viaja junto e seria só cortesia sozinha.
        // PROJECT 41→42: os MESMOS três campos SAÍRAM (W-Pulley W1). Uma roldana virou
        // ENTIDADE (`PulleyWheel`), e um componente novo não custaria bump nenhum — o que
        // custa é a REMOÇÃO: postcard é posicional, então um blob v41 tem três campos a
        // mais e todo joint salvo leria os seguintes deslocados. Bump por remover, pelo
        // mesmo motivo que se bumpa por apendar.
        // PROJECT 42→43: a `PulleyWheel` ganhou `motor_speed` (W-Pulley W2 — a roldana
        // dirigida, o guincho). Componente NOVO não custaria bump; APENDAR campo a um
        // que já existe custa, porque postcard é posicional e um blob v42 tem um `f32`
        // a menos — o load leria lixo bem-formado em vez de recusar.
        // PROJECT 43→44: a `PulleyWheel` ganhou `break_enabled`+`break_force` (W2 —
        // o eixo que cede). Dois campos apendados, mesmo raciocínio posicional.
        // PROJECT 44→45: a `PulleyWheel` ganhou `body`+`local`+`mounted` (W-Pulley W3 — a
        // roldana montada num corpo que se move, e com ela a vantagem mecânica). Três
        // campos apendados, mesmo raciocínio posicional; o par `local`/`mounted` é o do
        // W-AnchorFollow, para o eixo não deslizar pelo bloco quando o bloco se move.
        // PROJECT 45→46: a `PulleyWheel` ganhou `radius_out` (W-Pulley W4 — o tambor
        // DIFERENCIAL: uma roldana com dois raios, e a vantagem mecânica contínua que
        // cai do quociente deles). Um campo apendado, mesmo raciocínio posicional.
        // PROJECT 46→47: o `PhysicsJoint` ganhou `soft` (W-SoftWeld — a solda que cede;
        // um bool apendado, e a dureza reusa a `stiffness`/`damping` da mola).
        // PROJECT 47→48 + FLIP 12→13: o `Cap` ganhou a variante `Square` (a 3ª ponta do
        // padrão — o traço estendido por meia-espessura e cortado reto). Apender variante
        // NÃO move os índices de `Round`/`Flat`, então todo arquivo já salvo segue legível;
        // o bump é pelo caminho INVERSO (um arquivo novo aberto por um leitor velho leria
        // `Square` como lixo), o mesmo raciocínio do `JointKind::Weld`.
        // ⚠️ A `line/FLIP` escreveu **47** e a `line/physics` reivindicou o MESMO 47 na
        // mesma janela — a TERCEIRA colisão entre estas duas linhas (30 em 25/07, 32/33/34
        // em 27/07). E aqui ela quase passou MUDA: o `project.rs` não conflitou, porque os
        // dois lados escreveram o mesmo literal e o git não tem opinião sobre o que o número
        // SIGNIFICA — o bump da FLIP teria evaporado com a suíte verde. O valor se CONTA a
        // partir do `main` do dia; ele não estava em nenhum dos dois lados.
        // PROJECT 48→49 (vector, W6.2 — as guias e a régua): o `ProjectState` ganhou
        // `guides`, a lista de linhas de referência que o artista arrasta da régua. Campo
        // apendado ao `ProjectState`, que viaja DENTRO do `ProjectFile` — o mesmo raciocínio
        // posicional do `flip`. ⚠️ O 49 é PROVISÓRIO: ele se CONTA contra o `main` do dia da
        // integração, não se escolhe, e esta linha o escreveu contra o `main` de 2026-08-01.
        // PROJECT 49→50 + VEC_SCENE 13→14 (vector, W6.4 — o ALINHAMENTO do traço): o
        // `StrokeSpec` ganhou `align` (Centre/Inner/Outer). Campo APENDADO, e o bump é
        // obrigatório nos DOIS sentidos — o postcard **não sinaliza ausência**, então um save
        // v13 lido por v14 chega ao fim dos bytes no campo novo (`Hit the end of buffer`,
        // MEDIDO numa sonda em 2026-08-01) e um v14 lido por v13 traz um byte a mais.
        // ⚠️ E isto corrigiu uma afirmação FALSA que vivia no `stroke_style.rs`: o
        // doc-comment do `marker_start` dizia que *"o postcard é posicional, então um save
        // anterior a este campo segue legível"* — as duas metades não se seguem. Posicional é
        // justamente o que IMPEDE a leitura; o `#[serde(default)]` serve a formatos
        // auto-descritivos, e quem protege o arquivo é este número.
        // ⚠️ O 50 é PROVISÓRIO pela mesma razão do 49 acima.
        // PROJECT 50→51: o `ProjectFile` ganhou `tokens` — a tabela de COR autorada pelo artista
        // (plano UI/UX W6, degrau 1). Campo APENDADO ao arquivo, e postcard é posicional ⇒ um
        // save v50 chega ao fim dos bytes onde o campo novo começa. ⚠️ `FLIP` e `VEC_SCENE` NÃO
        // se movem: a tabela é do ARQUIVO, não da cena — ela vive fora do `ProjectState` pelo
        // mesmo motivo que `physics`/`motion`/`timeline` (um Ctrl+Z do canvas não rebobina a
        // cara do editor). O 51 é PROVISÓRIO pela mesma razão dos dois acima.
        // PROJECT 51→52 (3D, W8.3 — o DOCUMENTO da escultura): o `ProjectFile` ganhou
        // `sculpt`, um blob que carrega a própria versão (`SCULPT_DOC_VERSION`) — o
        // precedente EXATO do `timeline`. Campo apendado ao `ProjectFile`, e o postcard é
        // posicional ⇒ o bump é obrigatório. ⚠️ Ele bumpa **UMA vez, agora**: daqui em
        // diante o módulo 3D pode evoluir muitas waves sem tocar este número, porque a
        // versão vive DENTRO do blob (é por isso que o `TimelineDoc` foi de v9 a v17 com
        // este schema quieto). E o campo é `Vec<u8>` **incondicional**, sem `cfg`: um
        // campo condicional daria DUAS formas de arquivo sob UM número de schema, e é o
        // que torna um build sem a feature um **passa-adiante** em vez de um triturador.
        // ⚠️ O 52 é PROVISÓRIO pela mesma razão do 49/50/51 acima: ele se CONTA contra o
        // `main` do dia da integração.
        // PROJECT 52→53: o `ProjectFile` ganhou `baked_forms` (ADR-0150 W8.7 — os canais que
        // uma malha doou a um sprite: `base`, `form` e o RIG). Campo apendado ao ARQUIVO, e
        // postcard é posicional ⇒ o leitor v52 chega ao fim dos bytes. ⚠️ `FLIP`/`VEC_SCENE`
        // não se movem: os canais são campo de sprite, e nem sequer entraram no blob `sculpt`
        // — o parser dele é `cfg(feature = "sculpt3d")`, e um objeto assado tem de ser legível
        // SEM o módulo 3D no build (é o que a *rota A* do `docs/3D/02.2` promete).
        // PROJECT 53→54: `PhysicsJoint` ganhou `custom` (W-JointCustom — a configuração de
        // eixos autorada de um `JointKind::Custom`). UM campo apendado, o mesmo padrão dos
        // v32/v33/v34, e o postcard é posicional ⇒ um save v53 lido por v54 chega ao fim dos
        // bytes no campo novo. ⚠️ A linha escreveu **51** aqui, contra o `main` em que ela
        // nasceu (50); o `main` do dia da integração dizia **53** — a tabela de cor do vector
        // e os dois degraus do 3D entraram no meio ⇒ o valor CONTADO é 54, e ele não estava
        // em nenhum dos dois lados do conflito
        // ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
        // PROJECT 54→55: `PlatformPlayer` ganhou `corner_reach` e
        // `lift_momentum` (W10 — a correção de quina e a memória do referencial
        // da plataforma). Dois campos apendados ao componente, e o postcard é
        // posicional ⇒ um save v54 lido por v55 chega ao fim dos bytes neles.
        // ⚠️ A linha escreveu 52; o valor CONTADO é 55 — ela trouxe DOIS degraus
        // (o `custom` do W-JointCustom e este), e o handoff dela contou UM só.
        // PROJECT 55→56: o `ProjectState` ganhou `ui_states`, a tabela de estados de UI
        // (plano UI/UX W7). Campo apendado ao estado que viaja DENTRO do `ProjectFile` — o
        // mesmo raciocínio posicional do `guides` (v49), e o mesmo motivo de morar ali: o
        // `ProjectState` é a unidade do UNDO, e gravar um estado tem de desfazer.
        // ⚠️ Nenhum gate VÊ um campo apendado — nenhuma constante se move —, então este
        // degrau existe porque foi escrito à mão. Quem apende, bumpa, no MESMO commit.
        // PROJECT 56→57: um token de cor autorado passa a valer uma cor **ou o nome de outro
        // token** (o ALIAS, plano UI/UX W4b), então o `SavedToken` troca o campo `rgba: [u8; 4]`
        // pelo enum `SavedValue`. A FORMA do registro mudou ⇒ o postcard, que é posicional, leria
        // um arquivo v56 com o layout errado; o número transforma isso num erro de VERSÃO.
        // ⚠️ Um enum e não um `rgba` com um `alias` ao lado: os dois seriam mutuamente exclusivos
        // e nada no formato o diria — a representação apaga o estado que ninguém especificou.
        // PROJECT 57→58: a ESCALA (`spacing.*`, `radius.*`, `stroke.*`) passa a ser autorável
        // (plano UI/UX W4c.1), e o valor autorado viaja na MESMA lista `tokens` — o `SavedValue`
        // ganha a variante `Number(f32)`, e a CHAVE (`"spacing.md"`) é quem diz de que família a
        // entrada é. ⚠️ **Uma lista só, e não um campo `num_tokens` ao lado**: o que o arquivo
        // guarda é *"que tokens o artista autorou"*, e duas listas para isso seriam duas respostas
        // à mesma pergunta que o import/export DTCG (W4c.5) teria de juntar de novo. Isso só é
        // seguro porque as duas famílias são provavelmente disjuntas nas chaves — há gate a
        // afirmá-lo (`no_key_is_claimed_by_both_families`).
        // ⚠️ Aqui o bump é o **caminho INVERSO**, e é a única razão: apender variante NÃO move
        // `Literal`(0) nem `Alias`(1), então todo arquivo já salvo continua a ler — mas um build
        // ANTIGO a ler um arquivo novo bateria num índice de variante que ele não tem, e o número
        // transforma isso num erro de VERSÃO em vez de num postcard a falhar longe da causa. É o
        // mesmo raciocínio do `JointKind::Weld` (v28) e do `Cap::Square` (v48).
        // ⚠️ `FLIP`/`VEC_SCENE` NÃO se movem: a tabela é do ARQUIVO, não da cena.
        // PROJECT 58→59: um token numérico passa a poder valer uma FÓRMULA (W4c.3), e o
        // `SavedValue` ganha `Formula(String)` — variante APENDADA, então `Literal`(0)/`Alias`(1)/
        // `Number`(2) não se movem e todo arquivo salvo continua a ler; o bump é pelo caminho
        // INVERSO, o mesmo raciocínio do v58 acima e do `JointKind::Weld` (v28).
        // ⚠️ `FLIP`/`VEC_SCENE` NÃO se movem: a tabela é do ARQUIVO, não da cena.
        // PROJECT 59→60: os tokens de ESCALA chegam ao DOCUMENTO (W4c.4) — o `ph2d_ecs::BoundProp`
        // ganha `StrokeWidth`(2), `LayoutGapMain`(3) e `LayoutGapCross`(4), então a espessura de um
        // traço e o vão de um auto layout podem SEGUIR um token numérico. Variantes APENDADAS:
        // `Fill`(0) e `StrokeColor`(1) não se movem e todo binding salvo continua a ler; o bump é
        // pelo caminho INVERSO, o mesmo raciocínio do v58/v59 acima.
        // ⚠️ `FLIP`/`VEC_SCENE` NÃO se movem: o binding é uma tabela LATERAL no ECS, e nenhum campo
        // foi apendado a `Paint`, a `StrokeSpec` ou a `VecShape` — que é a decisão inteira do
        // `vec_bindings` e a razão de o `VEC_SCENE_SCHEMA` ficar quieto numa feature de estilo.
        // ⚠️ O 60 é PROVISÓRIO pela mesma razão de todos os acima — ele se CONTA contra o `main`
        // do dia da integração ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
        // PROJECT 60→61: o texto ganha uma CAIXA (W2a) — o `ph2d_ecs::VecTextParams` ganha
        // `wrap_width: Option<f64>`, a largura a que ele reflui.
        // ⚠️ **Este bump é de outra CLASSE que os cinco acima, e é a diferença que importa:**
        // v57..v60 apendaram VARIANTES (o índice 0 não se move ⇒ o arquivo velho continua a ler,
        // e o número serve só ao caminho inverso). Aqui é um CAMPO num componente existente, e o
        // blob é postcard POSICIONAL ⇒ **todo arquivo já salvo bate no fim dos bytes**. O bump
        // não é cortesia com o build antigo: é o que transforma lixo bem-formado num erro.
        // ⚠️ Um componente NOVO teria custado zero (`VecStrokeProfile`/ADR-0148 é o precedente
        // desta própria linha) e foi recusado com motivo — a largura é um número de layout ao
        // lado do `align`/`tracking`, e um segundo componente partiria a porta `layout_of_params`
        // em duas. O `project.rs` guarda o argumento inteiro.
        // ⚠️ `FLIP`/`VEC_SCENE` NÃO se movem: o texto é um COMPONENTE do ECS, e a `VecScene` só
        // guarda a geometria já cozida — nenhum campo foi apendado a `VecPath` nem a `VecShape`.
        // ⚠️ O 61 é PROVISÓRIO pela mesma razão de todos os acima.
        //
        // PROJECT 61→62: a MOLA como OPÇÃO (W7m) — o `ph2d_ui_state::HostStates` ganha
        // `spring: Option<Spring>`, a alternativa ao par *duração + curva*.
        // ⚠️ **Mesma classe do v61** (campo apendado a struct serializado ⇒ postcard posicional ⇒
        // quebra dura), e um `#[serde(default)]` **não salva**: o postcard não sinaliza ausência.
        // ⚠️ **O easing fica INTACTO** — `duration_s` e `easing` continuam onde estavam, e um
        // hospedeiro sem mola percorre o mesmo caminho byte a byte. É por isso que a mola é uma
        // `Option` e não uma substituição.
        // ⚠️ `FLIP`/`VEC_SCENE` NÃO se movem: os estados de UI viajam no `ProjectFile` ao lado da
        // cena, não dentro dela.
        // ⚠️ O 62 é PROVISÓRIO pela mesma razão de todos os acima.
        //
        // PROJECT 62→63: o `BakedFormDocument` ganhou `form_occ` (3D, W10.7 — a oclusão de
        // forma de um objeto assado). UM campo apendado, postcard posicional ⇒ um save anterior
        // lido por este chega ao fim dos bytes nele. ⚠️ Ela viaja em vez de ser assada no
        // `base` porque um re-bake REUSA o `base`, e pré-multiplicá-la ali a comporia a cada
        // gesto. ⚠️ `FLIP`/`VEC_SCENE` NÃO se movem: o documento assado viaja no
        // `ProjectFile` ao lado da cena, não dentro dela.
        // ⚠️ **A linha escreveu 56; o valor CONTADO é 63** — a `line/Vector` trouxe os SETE
        // degraus v56..v62 na mesma janela de integração, e o número se CONTA, não se escolhe
        // ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
        // PROJECT 63→64: `PlatformPlayer` ganhou `wall_slide_speed`,
        // `wall_jump_height`, `wall_jump_push` e `wall_reach` (W13 — AS PAREDES:
        // escorregar por uma e pular dela). Quatro campos apendados ao
        // componente, e o postcard é posicional ⇒ um save v63 lido por v64 chega
        // ao fim dos bytes no primeiro deles.
        // PROJECT 64→65: `PlatformPlayer` ganhou `dash_speed`, `dash_time` e
        // `dash_cooldown` (W14 — O ARRANQUE). Três campos apendados, mesmo
        // raciocínio posicional.
        // PROJECT 65→66: `PlatformPlayer` ganhou `crouch_height` e
        // `crouch_speed` (W15 — O AGACHAR). Dois campos apendados; e note o que
        // ESTE degrau nao traz — nenhuma forma de collider muda, porque agachar
        // aqui e' uma perna mais CURTA e nao um corpo menor.
        // PROJECT 66→67: campo de ARQUIVO novo, `player_tape` (W17 — a CORRIDA
        // sobrevive ao arquivo). Nao e' um campo de componente: e' a gravacao do
        // dedo do jogador, tique a tique, que o bake da W16 replaya. Fora do
        // `ProjectState` pelo motivo de `motion`/`timeline`/`physics` — aquele e'
        // a unidade do undo GLOBAL.
        // PROJECT 67→68: `PlatformPlayer` ganhou `wall_grab_stamina` (W23 — O
        // AGARRAR-SE). Um campo apendado; ⚠️ e o botao novo (`PlayerInput::grab`)
        // NAO move o formato da fita — ela guarda os botoes num BITMASK, e um bit
        // livre nao muda um byte do postcard.
        // PROJECT 70→71: `PlatformPlayer` ganhou `swim_speed`,
        // `swim_acceleration` e `swim_enter` (W-Swim — NADAR). Tres campos
        // apendados num degrau so', porque sao UMA capacidade. ⚠️ A FITA nao se
        // move: o eixo vertical do nado sai dos botoes que ja' viajam no bitmask.
        // PROJECT 71→72: `PlatformPlayer` ganhou `corner_samples`,
        // `corner_lookahead`, `wall_samples` e `wall_spread` (W-Probes2 — OS
        // SENSORES FICAM EDITAVEIS). Quatro campos apendados num degrau so',
        // porque sao UM assunto: a geometria das amostras dos sensores, que era
        // `const` e passa a ser autorada. ⚠️ Os defaults sao as consts de sempre,
        // entao todo player ja' salvo fica byte-identico.
        // PROJECT 73→74: `PlatformPlayer` ganhou `air_jumps` + `air_jump_height`
        // (W-MultiJump — O PULO MULTIPLO), no MEIO do struct. ⚠️ A contagem
        // nasce em 0 (capacidade DESLIGADA), entao nenhum player ja' salvo muda
        // de comportamento — o degrau e' so' o layout.
        // PROJECT 74→75: `PlatformPlayer` ganhou `ledge_grab` + `ledge_speed`
        // (W-Ledge — A BEIRADA), apendados ao FIM. ⚠️ O alcance nasce em 0
        // (capacidade DESLIGADA), entao o degrau e' so' o layout — e o sensor
        // novo nem sequer e' castado num player ja' salvo.
        // ⚠️ **PROVISÓRIO:** o valor se CONTA contra o `main` do dia da
        // integração — três linhas já colidiram neste número por o terem
        // escolhido, e a última vez o certo não estava em nenhum dos dois lados.
        // PROJECT 78→79: `PlatformPlayer` ganhou `brake_scale` (W-Brake — FREAR
        // NAO E' ACELERAR), apendado ao FIM. ⚠️ Nasce em `1`, onde a lei reduz
        // LITERALMENTE — o degrau e' so' o layout, e nenhum player ja' salvo
        // muda de comportamento.
        (80, 13, 14),
        "a forma do FlipDoc ou da VecScene mudou (ou o esquema do projeto): suba o \
         PROJECT_SCHEMA junto e atualize esta tripla. Postcard nao avisa - ele so le errado."
    );
}
