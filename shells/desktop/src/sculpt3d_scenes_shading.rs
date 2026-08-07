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

/// `=19` — a cena do **ESPALHAMENTO SUB-SUPERFICIAL** (`docs/3D/05.1` §2a).
///
/// ⚠️ **A fixture é uma ESCADA de raios, e é ela que separa o efeito de um
/// slider de brilho.** O eixo da tabela é `t = scatter/R`: o espalhamento é
/// medido em **raios de curvatura**, então uma peça só não distingue *"a luz
/// atravessa formas finas"* de *"a peça clareou"*. Três esferas de raios 1,0 /
/// 0,45 / 0,2 sob o MESMO material dão `t` crescente — e o que o smoke tem de
/// ver é a MENOR ficando translúcida enquanto a maior mal se mexe.
pub(crate) fn sss_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("19")
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
        let ball = |segs: usize, r: f32| {
            let mut m = ph2d_mesh::shapes::uv_sphere(segs * 2 / 3, segs, r);
            m.triangulate();
            m
        };
        eprintln!(
            "[sculpt3d] =19 O ESPALHAMENTO SUB-SUPERFICIAL: tres esferas, MESMO material,
             [sculpt3d]    raios 1.00 / 0.45 / 0.20. O eixo do efeito e' `scatter / RAIO`, entao
             [sculpt3d]    a MENOR e' a que a luz atravessa -- e a escada e' o oraculo.
             [sculpt3d]    1) Aperte Shift+S ate 1.00. A borda da sombra tem de AMOLECER, e
             [sculpt3d]       amolecer MAIS na esfera pequena que na grande. Se as tres
             [sculpt3d]       mudarem igual, o canal virou um brilho global -- PARE.
             [sculpt3d]    2) Olhe a COR na borda da sombra: ela puxa para o VERMELHO. Medido,
             [sculpt3d]       o vermelho vaza 2x o azul. Se sair cinza, o perfil por canal
             [sculpt3d]       virou uma media e o efeito perdeu a unica coisa que faz carne
             [sculpt3d]       parecer carne.
             [sculpt3d]    3) 'Subsurface' de volta a 0.00: o barro tem de voltar EXATAMENTE ao que
             [sculpt3d]       era (ha gate de byte-identidade).
             [sculpt3d]    4) ESCULPA a esfera grande com um pincel pequeno: os vincos novos sao
             [sculpt3d]       finos, entao ELES passam a espalhar enquanto o corpo nao.
             [sculpt3d]    5) Experimente com o matcap 'Skin' (tecla M): la o `rim` FINGE
             [sculpt3d]       translucidez e aqui ela e' medida -- os dois convivem."
        );
        return Some(vec![
            (ball(72, 1.0), ph2d_mesh::Pose::at([-1.7, 0.0, 0.0])),
            (ball(48, 0.45), ph2d_mesh::Pose::at([0.35, 0.0, 0.0])),
            (ball(36, 0.2), ph2d_mesh::Pose::at([1.35, 0.0, 0.0])),
        ]);
    }
    if screen_ao_scene() {
        // ⚠️ **Duas esferas ENCOSTADAS, e a distância é o oráculo.** A `2,04` de
        // separação entre centros de raio 1 deixa uma fresta de 0,04 — funda o
        // bastante para a oclusão ser inequívoca, e estreita o bastante para o
        // AO ASSADO **não conseguir vê-la** (o bake marcha contra o campo de UM
        // corpo, e cada esfera é convexa). É essa diferença que o smoke julga.
        let mut a = ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
        a.triangulate();
        let b = a.clone();
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
        return Some(vec![
            (a, ph2d_mesh::Pose::at([-1.02, 0.0, 0.0])),
            (b, ph2d_mesh::Pose::at([1.02, 0.0, 0.0])),
        ]);
    }
    None
}
