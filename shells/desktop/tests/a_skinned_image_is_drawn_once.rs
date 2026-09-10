//! ⭐⭐⭐ **UMA IMAGEM PRESA AO ESQUELETO É DESENHADA UMA VEZ, E DEFORMADA.**
//!
//! ⛔⛔ **As duas metades andam juntas ou a arte aparece DUAS vezes.** O Vello desenha a imagem
//! deformada por cima; se o passe de sprites continuar a emitir a instância original, ela fica por
//! baixo, por deformar — e assim que o artista dobra o braço ela espreita por fora.
//!
//! ⚠️ **E a metade que se esquece é sempre a segunda**, porque a primeira é a que se vê a funcionar:
//! com a pose em repouso as duas coincidem ao pixel, e o defeito só aparece quando alguém dobra um
//! osso. *Uma cena que só mostra o defeito depois de um gesto não o mostra a quem só olha.*

//! # ⚠️⚠️ O que um gate que VARRE O FONTE não pode ver
//!
//! Estes três leem o texto do produto, e a prova de mutação desta wave expôs o limite deles: pôr
//! `if false &&` à frente da chamada do *Bind* **SOBREVIVEU** — o texto continua lá. ⇒ eles
//! apanham a chamada a **desaparecer**, a mudar de **selector** e a mudar de **ordem**; não
//! apanham uma chamada morta por uma condição.
//!
//! ⛔ **A cura NÃO é apertar a varredura** (procurar `if false` seria uma corrida contra o
//! próximo idioma que a desliga). A metade que falta é medida do outro lado: os gates de unidade
//! do [`crate::skeleton_skin_image`] provam que o bind e a deformação FAZEM o que dizem, e o smoke
//! é quem julga a costura inteira. *Um gate de texto responde «isto está escrito», nunca «isto
//! corre» — e escrever isso ao lado dele é o que impede o próximo leitor de lhe pedir a segunda
//! resposta.*

const EXTRACT: &str = include_str!("../src/render_loop/sim_extract.rs");
const LOOP: &str = include_str!("../src/render_loop/mod.rs");

/// **O fonte sem comentários** — sem isto, uma nota que cita a chamada conta como chamada.
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| {
            let t = l.trim_start();
            if t.starts_with("//") { "" } else { l }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **A SPRITE PRESA NÃO EMITE INSTÂNCIA** — e a guarda está no braço que emite.
///
/// ⚠️ **Ela tem de estar na MESMA condição que constrói a instância**, e não num `continue` ou num
/// filtro à parte: a travessia do extract também semeia a ORDEM, e um filtro por fora tiraria a
/// imagem da ordenação — ela deixaria de ocupar o lugar dela e abriria uma faixa que não desenha
/// nada, que é o defeito que o comentário da forma vectorial já nomeia oito linhas acima.
#[test]
fn a_bound_sprite_emits_no_render_instance() {
    let src = code_only(EXTRACT);
    let i = src
        .find("skinned_image(sim, sim_entity)")
        .expect("o extract deixou de perguntar se a sprite está presa");
    let j = src
        .find("let Some(spr) = sim.get::<Sprite>(sim_entity)")
        .expect("o braço que emite a instância da sprite deixou de existir");
    assert!(
        i < j && j - i < 200,
        "a guarda da imagem presa (byte {i}) não está na condição que emite a instância (byte \
         {j}) — fora dela, ou ela não guarda nada, ou tira a imagem da ORDEM"
    );
    // ⚠️ E a pergunta é DERIVADA: nada aqui escreve `Visibility`, que é o olho da Hierarquia.
    let porta = src
        .find("pub(super) fn skinned_image")
        .expect("a porta da pergunta deixou de existir");
    let corpo = &src[porta..porta + 400];
    assert!(
        corpo.contains("SkinBind") && corpo.contains("Sprite"),
        "a pergunta deixou de ser derivada dos dois componentes"
    );
    assert!(
        !corpo.contains("Visibility"),
        "o extract passou a escrever/ler `Visibility` para esconder a imagem — é o olho da \
         Hierarquia, e a shell não pode discutir com o artista por aquele bool"
    );
}

/// ⭐⭐ **E A OUTRA METADE: o Vello desenha-a**, no quadro, antes dos ossos.
///
/// ⚠️ **A ORDEM é a leitura**: a imagem é a ARTE e o rig é o chrome que se desenha por cima dela.
/// Invertê-la esconderia o esqueleto debaixo do desenho exactamente quando o artista o está a posar.
#[test]
fn the_frame_draws_the_deformed_image_before_the_bones() {
    let src = code_only(LOOP);
    let desenho = src
        .find("skeleton_skin_image::draw_skinned_images(")
        .expect("o quadro deixou de desenhar as imagens presas");
    let ossos = src
        .find("skeleton_render::draw_bones(")
        .expect("o quadro deixou de desenhar os ossos");
    assert!(
        desenho < ossos,
        "a imagem é desenhada DEPOIS dos ossos (byte {desenho} contra {ossos}) — o rig fica \
         debaixo da arte exactamente quando o artista o está a posar"
    );
}

/// ⭐⭐⭐ **O MESMO BOTÃO *Bind* alcança as duas mídias.**
///
/// ⚠️ **O sujeito de uma imagem é a SELECÇÃO DO GIZMO**, e não a lista de caminhos do pen — são
/// duas famílias com dois selectores, e ler o do vector daria sempre **zero** imagens. Este gate
/// mede que o dreno lê o selector certo.
#[test]
fn the_bind_verb_reaches_both_media() {
    let src = code_only(LOOP);
    let i = src
        .find("if pending_bone_bind {")
        .expect("o dreno do *Bind* deixou de existir");
    let corpo = &src[i..i + 2600];
    assert!(
        corpo.contains("skeleton_live::bind("),
        "o *Bind* deixou de prender FORMAS"
    );
    assert!(
        corpo.contains("skeleton_live::bind_image("),
        "o *Bind* não alcança as IMAGENS — o botão existe, o gesto existe, e a 2.ª mídia fica \
         inalcançável por dentro"
    );
    assert!(
        corpo.contains("selecao_bits"),
        "o *Bind* de imagens não lê a selecção do gizmo — lendo a lista do pen, ele acha sempre \
         zero imagens e a recusa é MUDA"
    );
}
