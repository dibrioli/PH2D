//! **AS CENAS DOS CANAIS DE SOMBREAMENTO** — as que julgam como a forma é LIDA,
//! e não o que ela É.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE, e a linha é clara:** o irmão [`super`]
//! monta cenas que respondem *"a topologia sobreviveu?"*, *"o documento voltou?"*,
//! *"o remesh fechou?"* — perguntas sobre a MALHA, cujo oráculo é geometria. Estas
//! respondem *"este canal desenha o que ele diz desenhar?"*, e o oráculo delas é a
//! **APARÊNCIA**: a fresta escura, a borda macia, a escada entre três raios.
//!
//! É por isso que as duas viajaram juntas quando o pai cruzou o teto de 600 LOC:
//! elas compartilham a fixture (esferas LISAS, dispostas para o efeito ser
//! inequívoco) e o mesmo tipo de instrução no roteiro impresso.
//!
//! ⚠️ **E o corte consertou um defeito que nenhum gate via:** os dois
//! doc-comments estavam **TROCADOS** no pai. Ao inserir a `sss_scene` eu a pus
//! entre o doc do `=18` e a função dele, e o doc órfão adotou a minha — a prosa
//! passou a descrever a outra cena, com os números ainda certos. É a mesma classe
//! que a §5 já registra (*"minhas linhas `mod` orfanaram doc-comments"*), e ela
//! não levanta erro nenhum: só uma leitura pega.

use ph2d_mesh::Pose;

/// `=18` — a cena do **AO DE TELA**: duas peças ENCOSTADAS.
///
/// ⚠️ **Duas peças, e é a fixture inteira.** A oclusão que uma lança sobre a
/// outra é exatamente o que o AO ASSADO **não consegue medir** — o bake marcha
/// cones contra o campo SDF de UM corpo, e aquele corpo é convexo e não vê a
/// vizinha. Numa peça só, as duas fontes mediriam a mesma coisa e o smoke não
/// conseguiria distinguir a feature de um slider redundante.
pub(crate) fn screen_ao_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("18")
}

/// `=19` — a cena da **TINTA QUE A LUZ ATRAVESSA** (`docs/3D/05.1` §2a e §2b).
///
/// ⚠️ **A fixture é uma ESCADA, e é ela que separa o efeito de um slider de
/// brilho.** Os dois canais são medidos em quanta MATÉRIA a luz tem de vencer,
/// então uma peça só não distingue *"a luz atravessa formas finas"* de *"a peça
/// clareou"*. Três esferas de raios 1,0 / 0,45 / 0,2 **sob o mesmo material** —
/// e o que o smoke tem de ver é a MENOR acendendo enquanto a maior mal se mexe.
///
/// ⚠️ **E uma CHAPA, que as esferas não sabem mostrar.** O §2b existe para a
/// folha, a orelha e a mão contra a lanterna, e nenhuma delas é uma bola: a
/// chapa é a forma em que a luz de fato sai pelo outro lado. Ela também é o que
/// impede a fixture de validar um atalho errado — o proxy `2/|κ|` acerta a
/// esfera ao dígito e erra a chapa em **511%**.
///
/// ⚠️ **A cena nasce ASSADA**, e é por isso que ela mostra o canal no primeiro
/// frame: a espessura é MEDIDA da forma, e sem o bake a peça lê como opaca (que
/// é a leitura certa de *ninguém mediu*). Numa peça do artista, quem assa é o
/// botão **Bake Occlusion + Thickness**.
pub(crate) fn sss_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("19")
}

/// `=20` — a cena da **OCLUSÃO QUE CHEGA À TINTA** (`docs/3D/05.2`, W10.7).
///
/// ⚠️ **Cena própria e não um passo da `=2`**, pela regra que o próprio
/// `donation_scene` escreve: *a doação* e *a oclusão da doação* são duas
/// perguntas. A `=2` pergunta **a forma acende a tinta?** (a normal); esta
/// pergunta **a fresta da escultura ESCURECE a tinta?**, e o oráculo é outro —
/// lá é o realce que aparece, aqui é a sombra que aparece onde a forma tem um
/// vinco.
///
/// ⚠️ **A peça nasce SULCADA, e é a fixture inteira.** Sobre uma esfera lisa a
/// curvatura é quase constante (medido: `k ≈ −0,037`, a faixa da oclusão fica
/// em `1,108 .. 1,155`) e *"a oclusão chegou"* seria indistinguível de *"a peça
/// clareou um pouco"*. Com os sulcos ela abre para `0,000 .. 1,781`, e é desse
/// fosso que o olho decide.
pub(crate) fn occlusion_donation_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("20")
}

/// **A peça com que a cena ABRE**, para as cenas que têm elenco próprio.
///
/// ⚠️ **Ela existe por causa de um defeito que a aritmética achou e nenhum gate via.** O
/// [`super::smoke_mesh`] devolve a peça PRIMÁRIA e o [`scene_objects`] só acrescenta **extras** —
/// então uma cena que declarasse o elenco inteiro em `scene_objects` abria com a esfera lisa de
/// 96×144 **por cima**, não convidada. Medido na `=19`: a bola de raio `0,45` em `x = 0,35` fica
/// `0,35 + 0,45 = 0,80 < 1,0` — **inteiramente enterrada** dentro da primária —, e a de raio `1,0`
/// em `x = −1,7` se funde com ela. *A escada que o roteiro chama de oráculo tinha um degrau
/// invisível e um quarto corpo que ninguém pediu.*
///
/// A cura é estrutural: uma cena de sombreamento é dona do **elenco inteiro**, e o gate
/// `a_shading_scene_owns_its_whole_cast` afirma que as duas portas respondem juntas.
pub(crate) fn primary_mesh() -> Option<ph2d_mesh::Mesh> {
    if screen_ao_scene() {
        // ⚠️ **Uma das DUAS**, e o gate achou isto: a primária acidental era uma terceira esfera de
        // raio 1 na ORIGEM — exatamente dentro da fresta de `0,04` que esta cena existe para
        // mostrar. O artista olhava a fresta e via barro.
        let mut a = ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
        a.triangulate();
        return Some(a);
    }
    if sss_scene() {
        // A maior da escada — a peça que o artista enquadra primeiro.
        return Some(baked(ph2d_mesh::shapes::uv_sphere(48, 72, 1.0)));
    }
    if occlusion_donation_scene() {
        return Some(grooved_sphere());
    }
    None
}

/// **Os dois canais assados**, pela mesma sequência que o botão usa (voxeliza → flood fill → mede).
///
/// Sem isso a cena abriria opaca e o smoke julgaria um canal que não está ligado.
fn baked(mut m: ph2d_mesh::Mesh) -> ph2d_mesh::Mesh {
    m.triangulate();
    let mut field = ph2d_sdf::VoxelField::for_bounds(m.bounds(), ph2d_sdf::DEFAULT_RESOLUTION);
    field.voxelize(&m);
    field.flood_fill();
    let thickness = ph2d_sdf::bake_thickness(&field, &m);
    let ao = ph2d_sdf::bake_ao(
        &field,
        m.positions(),
        m.normals(),
        ph2d_sdf::AoParams::for_bounds(m.bounds()),
    );
    m.set_thickness(thickness);
    m.set_ao(ao);
    m
}

/// A esfera de **três sulcos** da `=20` — faixas de latitude recuadas ao longo da própria normal.
///
/// ⚠️ É a MESMA fixture dos gates de GPU desta wave, e reusá-la é o que faz o smoke julgar o que o
/// gate mede: numa esfera lisa a oclusão fica em `1,108 .. 1,155` e *"a oclusão chegou"* seria
/// indistinguível de *"a peça clareou"*; com os sulcos ela abre para `0,000 .. 1,781`.
fn grooved_sphere() -> ph2d_mesh::Mesh {
    let mut m = ph2d_mesh::shapes::uv_sphere(72, 108, 1.0);
    for band in [0.45f32, 0.0, -0.45] {
        let moved: Vec<u32> = (0..m.vert_count() as u32)
            .filter(|&v| (m.positions()[v as usize][1] - band).abs() < 0.045)
            .collect();
        for &v in &moved {
            let n = m.normals()[v as usize];
            let p = &mut m.positions_mut()[v as usize];
            for k in 0..3 {
                p[k] -= n[k] * 0.07;
            }
        }
    }
    m.rebuild();
    m.triangulate();
    m
}

/// A cena de um destes canais, **se alguma estiver armada**.
///
/// ⚠️ **`Option` e não um `Vec` vazio:** *"nenhuma destas está armada"* e *"esta
/// cena não tem peça nenhuma"* são respostas diferentes, e colapsá-las faria o
/// roteador do pai sair do caminho normal em silêncio.
pub(crate) fn scene_objects() -> Option<Vec<(ph2d_mesh::Mesh, Pose)>> {
    if sss_scene() {
        // ⚠️ **Esferas LISAS, e de propósito.** O que este canal desenha é o
        // TERMINADOR — a fronteira entre o lado aceso e o escuro —, e num barro
        // vincado ele passaria por cima de curvaturas locais que competem com a
        // da própria peça. O oráculo do smoke é a ESCADA entre as três, e ela só
        // é legível se cada uma tiver uma curvatura só.
        // ⚠️ **Esferas LISAS, e de propósito.** O que o §2a desenha é o
        // TERMINADOR — a fronteira entre o lado aceso e o escuro —, e num barro
        // vincado ele passaria por cima de curvaturas locais que competem com a
        // da própria peça.
        //
        // ⚠️ **E cada peça nasce com os dois canais assados** — ver o [`baked`], a porta que a
        // [`primary_mesh`] também atravessa. Sem isso a cena abriria opaca e o smoke julgaria um
        // canal que não está ligado.
        let ball = |segs: usize, r: f32| baked(ph2d_mesh::shapes::uv_sphere(segs * 2 / 3, segs, r));
        eprintln!(
            "[sculpt3d] =19 A TINTA QUE A LUZ ATRAVESSA: tres esferas (raios 1.00 / 0.45 /\n\
             [sculpt3d]    0.20) e uma CHAPA, todas com o MESMO material, todas ja' assadas.\n\
             [sculpt3d]    Os dois canais medem quanta MATERIA a luz vence -- entao a peca fina\n\
             [sculpt3d]    acende e a grossa nao. A ESCADA e' o oraculo.\n\
             [sculpt3d]    1) Aperte Shift+S ate 1.00 (ou arraste 'Subsurface'). Duas coisas tem\n\
             [sculpt3d]       de acontecer: a borda da sombra AMOLECE, e o lado ESCURO das pecas\n\
             [sculpt3d]       finas ACENDE -- medido, a esfera pequena ganha 4,3x o que a grande\n\
             [sculpt3d]       ganha. Se as quatro mudarem igual, o canal virou um brilho global\n\
             [sculpt3d]       e nao um material -- PARE.\n\
             [sculpt3d]    2) A CHAPA e' onde a cera mora: ela e' fina em toda parte, entao ela\n\
             [sculpt3d]       acende inteira. Olhe a COR do que acende -- tem de puxar para o\n\
             [sculpt3d]       VERMELHO (o azul se apaga 7,5x mais depressa, e e' isso que faz\n\
             [sculpt3d]       uma mao contra a lanterna parecer uma mao).\n\
             [sculpt3d]    3) Arraste 'Scatter': ele e' o alcance dentro do material, e e' ele\n\
             [sculpt3d]       que decide QUAO GROSSA uma peca pode ser e ainda atravessar.\n\
             [sculpt3d]    4) 'Subsurface' de volta a 0.00: o barro tem de voltar EXATAMENTE ao\n\
             [sculpt3d]       que era (ha gate de byte-identidade).\n\
             [sculpt3d]    5) ESCULPA a chapa e aperte 'Bake Occlusion + Thickness': a espessura\n\
             [sculpt3d]       e' MEDIDA da forma, entao ela envelhece com o dab -- o painel diz\n\
             [sculpt3d]       'Baked channels describe the previous shape' ate' voce reassar."
        );
        // A CHAPA em pé: um disco de 0,12 de espessura, que é a forma do §2b.
        let slab = baked(ph2d_mesh::shapes::cylinder(48, 0.7, 0.12));
        // ⚠️ **A de raio 1,0 saiu daqui e virou a PRIMÁRIA** — ver o doc do [`primary_mesh`]. E as
        // posições foram RE-MEDIDAS para que nenhuma peça toque a outra: a primária ocupa
        // `[−1, +1]`, então a de `0,45` começa em `1,0 + 0,15 + 0,45` e assim por diante. A escada
        // só é oráculo se as três forem VISÍVEIS.
        return Some(vec![
            (ball(48, 0.45), ph2d_mesh::Pose::at([1.60, 0.0, 0.0])),
            (ball(36, 0.2), ph2d_mesh::Pose::at([2.45, 0.0, 0.0])),
            (slab, ph2d_mesh::Pose::at([3.45, 0.0, 0.0])),
        ]);
    }
    if screen_ao_scene() {
        // ⚠️ **Duas esferas ENCOSTADAS, e a distância é o oráculo.** A `2,04` de
        // separação entre centros de raio 1 deixa uma fresta de 0,04 — funda o
        // bastante para a oclusão ser inequívoca, e estreita o bastante para o
        // AO ASSADO **não conseguir vê-la** (o bake marcha contra o campo de UM
        // corpo, e cada esfera é convexa). É essa diferença que o smoke julga.
        let mut b = ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
        b.triangulate();
        eprintln!(
            "[sculpt3d] =18 O AO DE TELA: duas esferas ENCOSTADAS.\n\
             [sculpt3d]    A oclusao que UMA lanca sobre a OUTRA e' o que so' este passe mede --\n\
             [sculpt3d]    o bake marcha contra o campo de um corpo so', e cada esfera e' convexa.\n\
             [sculpt3d]    1) Olhe a FRESTA entre elas: ela tem de estar escura, e o flanco de\n\
             [sculpt3d]       FORA claro. Medido: a fresta escurece 46,6% e o flanco 0,5%.\n\
             [sculpt3d]    2) Arraste 'Screen Occlusion' de 1 a 0 e de volta: a fresta clareia e\n\
             [sculpt3d]       escurece, o flanco quase nao se move. Em ZERO a peca e' byte-\n\
             [sculpt3d]       identica ao barro de sempre.\n\
             [sculpt3d]    3) GIRE a cena: a oclusao tem de ficar COLADA na fresta, nunca na\n\
             [sculpt3d]       tela. Se ela grudar na tela, o passe esta lendo a vista errada.\n\
             [sculpt3d]    4) ESCULPA uma cratera funda: ela escurece NA HORA, sem botao. Este e'\n\
             [sculpt3d]       o ponto inteiro -- o AO assado so' mudaria depois de 'Bake AO'.\n\
             [sculpt3d]    5) Agora aperte 'Bake AO' e suba tambem 'Ambient Occlusion': as duas\n\
             [sculpt3d]       fontes compoem pela MAIS ESCURA, nunca multiplicando. A peca nao\n\
             [sculpt3d]       pode ficar preta na fresta ao ligar a segunda."
        );
        // ⚠️ **A primeira é a PRIMÁRIA** (ver [`primary_mesh`]), e ela abre na origem — então a
        // segunda mora a `2,04`, que é a MESMA separação de antes entre as duas superfícies: a
        // fresta continua medindo `0,04`, que é o número que o roteiro cita.
        return Some(vec![(b, ph2d_mesh::Pose::at([2.04, 0.0, 0.0]))]);
    }
    if occlusion_donation_scene() {
        eprintln!(
            "[sculpt3d] =20 A FRESTA CHEGA A TINTA: uma esfera com TRES SULCOS, e a tela branca.\n\
             [sculpt3d]    Ate' agora a doacao levava so' a NORMAL: a tinta acendia pela forma e a\n\
             [sculpt3d]    fresta que a escultura desenha ficava no viewport. Agora ela viaja.\n\
             [sculpt3d]    ⚠️ D CICLA TRES POSICOES, e em LUZ o BARRO SAI DA TELA -- DE PROPOSITO.\n\
             [sculpt3d]       O que voce olha ali e' a TINTA acesa pela forma; se a esfera ainda\n\
             [sculpt3d]       estivesse la' ela taparia justamente o que ha' para julgar.\n\
             [sculpt3d]    1) Em BARRO, olhe os tres sulcos e suba 'Cavity' ate' 1: eles escurecem.\n\
             [sculpt3d]       Este e' o seu CONTROLE -- guarde como eles ficaram.\n\
             [sculpt3d]    2) Aperte D ate' 'LUZ' (a esfera sai), pegue o Painter e pinte CHAPADO\n\
             [sculpt3d]       sobre a tela. A tinta tem de sair com os TRES SULCOS ESCUROS -- nao\n\
             [sculpt3d]       so' acesa de um lado, que e' o que a normal ja' fazia sozinha.\n\
             [sculpt3d]    3) Arraste 'Cavity' de 1 a 0 com a tinta na tela: a fresta NA TINTA tem\n\
             [sculpt3d]       de clarear junto. Em 0.00 a tinta volta ao que era -- os tres canais\n\
             [sculpt3d]       em zero dao oclusao 1 em toda parte, e 1 e' a identidade exata.\n\
             [sculpt3d]    4) GIRE a cena e pinte de novo: a sombra tem de seguir a FORMA, nunca\n\
             [sculpt3d]       ficar colada na tela.\n\
             [sculpt3d]    5) O TESTE QUE UMA MUTACAO PEDIU: em 'LUZ', suba 'Cavity' e pinte. A\n\
             [sculpt3d]       doacao tem de carregar o valor NOVO do knob -- em LUZ o barro nem e'\n\
             [sculpt3d]       desenhado, e o passe que doa precisa ser informado por conta propria."
        );
        // ⚠️ **A peça é a PRIMÁRIA** (ver [`primary_mesh`]), então esta cena não tem extras — e o
        // `Some(vec![])` é deliberado: ele diz *"o elenco é meu e está completo"*, que é o que o
        // gate `a_shading_scene_owns_its_whole_cast` afirma.
        return Some(Vec::new());
    }
    None
}
