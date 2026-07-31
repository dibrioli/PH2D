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
        (46, 12, 13),
        "a forma do FlipDoc ou da VecScene mudou (ou o esquema do projeto): suba o \
         PROJECT_SCHEMA junto e atualize esta tripla. Postcard nao avisa - ele so le errado."
    );
}
