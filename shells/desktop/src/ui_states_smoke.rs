//! **A cena dos ESTADOS de UI** — `PH2D_BUILD_SMOKE=61` (plano UI/UX W7).
//!
//! # A pergunta desta cena é de olho, e ela é sobre O MEIO
//!
//! *Eu autorei DUAS pontas e o motor descobriu o caminho entre elas — a forma não salta, ela
//! anda; e o que eu não autorei não se mexe.*
//!
//! A cena monta três hospedeiros, cada um a responder uma pergunta diferente:
//!
//! 1. **Play** — um retângulo com um ponto dentro. Entre Default e Hover mudam **posição, escala
//!    e cor ao mesmo tempo**, e o filho anda **junto** com o pai. É a prova de que um estado é da
//!    SUB-ÁRVORE, não de uma forma.
//! 2. **Card** — só a **COR** muda. ⚠️ É o caso que o custo do motor decide: geometrias idênticas
//!    **não constroem `Plan`**, e o número aparece no anúncio. Vinte objetos numa troca só-de-cor
//!    pagariam 12,79 ms — 77% de um quadro de 60 fps — para não mover um vértice.
//! 3. **Plain** — o **CONTROLE**: nenhum estado gravado. A seção continua a ser oferecida (a face
//!    VAZIA é a que torna a feature alcançável), e nada nele se mexe.
//!
//! ⚠️ **E ela imprime os números que a tornam válida:** quantas poses gravou e quantos `Plan`s a
//! transição do Card custou. Se as poses forem zero, PARE — a autoria não correu, e o resto do
//! roteiro não diz nada.

use ph2d_ui_state::{StateRole, Transition};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, ellipse, rectangle};

/// Os retângulos: `(caixa, nome, é hospedeiro?)`.
///
/// ⚠️ O CONTROLE é o último e fica **longe** dos outros dois de propósito: numa foto, um objeto
/// que não se mexe ao lado de um que se mexe só é legível se ninguém duvidar de qual é qual.
const ART: [([f64; 4], &str); 5] = [
    ([-6.4, 0.2, -3.6, 1.6], "Play"),
    ([-6.0, 0.6, -5.4, 1.2], "Dot"),
    ([-2.8, 0.2, 0.0, 1.6], "Card"),
    ([1.0, 0.2, 3.8, 1.6], "Shape"),
    ([5.4, 0.2, 7.4, 1.6], "Plain"),
];

/// Índices no `ART`.
const PLAY: usize = 0;
const DOT: usize = 1;
const CARD: usize = 2;
const SHAPE: usize = 3;

/// As cores de repouso e as de hover, na ordem do `ART`.
///
/// ⚠️ O salto de cor é GRANDE de propósito: a interpolação é perceptual (OKLab, pela porta única
/// do Blend), e a diferença entre um caminho perceptual e um lerp de sRGB só aparece quando o
/// caminho é longo — num salto curto os dois passam pelo mesmo lugar e a cena não diria nada.
const REST: [[u8; 3]; 5] = [
    [58, 66, 92],
    [120, 132, 170],
    [96, 60, 64],
    [64, 88, 80],
    [72, 96, 76],
];
const HOVER: [[u8; 3]; 5] = [
    [88, 150, 232],
    [236, 244, 255],
    [232, 176, 96],
    [96, 196, 168],
    [72, 96, 76],
];

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        5 => name_them(app),
        7 => record_default(app),
        9 => pose_hover(app),
        11 => record_hover(app),
        13 => back_to_rest(app),
        15 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, (r, _)) in ART.iter().enumerate() {
        // O ponto é redondo — um retângulo dentro de outro leria como moldura, e o que a cena
        // precisa de mostrar é uma coisa a ANDAR dentro de outra.
        let mut p: VecPath = if i == DOT {
            ellipse([(r[0] + r[2]) * 0.5, (r[1] + r[3]) * 0.5], 0.3, 0.3)
        } else {
            rectangle([r[0], r[1]], [r[2], r[3]])
        };
        let c = REST[i];
        p.fill = Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
        // ⚠️ O Shape nasce COM traço: um perfil de largura modula a largura do traço, e sem
        // traço nenhum o Width Tool não teria o que animar — a fixture não conteria o fenômeno.
        if i == SHAPE {
            p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
                Rgba8::new(240, 244, 250, 255),
                0.14,
            ));
        }
        gfx.vec_scene.push_path(p);
    }
}

fn path_ids(app: &crate::App) -> Vec<VecPathId> {
    app.gfx
        .as_ref()
        .map(|g| g.vec_scene.paths().iter().map(|p| p.id).collect())
        .unwrap_or_default()
}

fn entity(app: &crate::App, id: VecPathId) -> Option<ph2d_ecs::Entity> {
    app.vec_entities
        .get(&id)
        .map(|&bits| ph2d_ecs::Entity::from_bits(bits))
}

/// Dá o NOME a cada forma e pendura o ponto no Play.
///
/// ⚠️ Num frame POSTERIOR ao `build`, e é obrigatório: a entidade de uma forma nasce no
/// `vec_entities::sync`, que corre no frame do desenho. Nomear antes seria escrever num objeto que
/// ainda não existe — a mesma ordem que o `widget_skin_smoke` já documenta.
fn name_them(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() < ART.len() {
        return;
    }
    let ents: Vec<_> = ids.iter().map(|&id| entity(app, id)).collect();
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, (_, name)) in ART.iter().enumerate() {
        let Some(e) = ents[i] else { continue };
        let Ok(mut ent) = gfx.sim.world_mut().get_entity_mut(e) else {
            continue;
        };
        ent.insert(ph2d_ecs::Name::new(*name));
    }
    // O ponto é FILHO do Play: é isso que faz um estado do Play carregar os dois.
    if let (Some(play), Some(dot)) = (ents[PLAY], ents[DOT]) {
        crate::vec_transform::reparent_keeping_world(&mut gfx.sim, dot, play);
    }
}

/// Grava a pose de repouso dos dois hospedeiros.
fn record(app: &mut crate::App, host: usize, role: StateRole) {
    let ids = path_ids(app);
    if ids.len() < ART.len() {
        return;
    }
    let map = &app.vec_entities;
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    // ⚠️ Pela porta do PRODUTO (`vec_ui_state_edit::apply`), e não escrevendo a tabela à mão: uma
    // cena que semeia estado por baixo pula exactamente a costura que ela existe para provar.
    crate::vec_ui_state_edit::apply(
        &mut gfx.sim,
        &mut gfx.vec_scene,
        map,
        &[ids[host]],
        &mut gfx.ui_states,
        crate::vec_ui_state_edit::UiStateEdit::Record(role),
    );
}

fn record_default(app: &mut crate::App) {
    for h in [PLAY, CARD, SHAPE] {
        record(app, h, StateRole::Default);
    }
}

/// Põe a cena na pose de HOVER — exactamente o que o artista faria com a mão.
fn pose_hover(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() < ART.len() {
        return;
    }
    let ents: Vec<_> = ids.iter().map(|&id| entity(app, id)).collect();
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    // O Play cresce; o ponto anda para a direita DENTRO dele.
    if let Some(e) = ents[PLAY]
        && let Some(mut t) = gfx.sim.world_mut().get_mut::<ph2d_ecs::Transform>(e)
    {
        t.scale.x = 1.12;
        t.scale.y = 1.12;
    }
    if let Some(e) = ents[DOT]
        && let Some(mut t) = gfx.sim.world_mut().get_mut::<ph2d_ecs::Transform>(e)
    {
        t.translation.x += 0.9;
    }
    // **O SHAPE muda de FORMA**, e pelas três ferramentas que o report nomeia: um nó puxado
    // (modo Node), as quinas arredondadas (Fillet) e um perfil de largura (Width Tool).
    if let Some(p) = gfx.vec_scene.path_mut(ids[SHAPE]) {
        p.verts[1].anchor[1] += 1.4;
        for v in &mut p.verts {
            v.corner_radius = 0.35;
        }
    }
    if let Some(e) = ents[SHAPE] {
        gfx.sim
            .world_mut()
            .entity_mut(e)
            .insert(ph2d_ecs::VecStrokeProfile {
                stops: ph2d_vec_scene::WidthProfile {
                    start: 0.2,
                    mid: 2.4,
                    end: 0.2,
                    position: 0.5,
                }
                .to_stops(),
            });
    }
    // As tintas de hover (o CONTROLE mantém a dele: `HOVER[4] == REST[4]`).
    for i in [PLAY, DOT, CARD, SHAPE] {
        if let Some(p) = gfx.vec_scene.path_mut(ids[i]) {
            let c = HOVER[i];
            p.fill = Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
        }
    }
}

fn record_hover(app: &mut crate::App) {
    for h in [PLAY, CARD, SHAPE] {
        record(app, h, StateRole::Hover);
    }
}

/// Devolve a cena ao repouso — a pose que o artista vê ao abrir.
///
/// ⚠️ Pelo **Show**, e não desfazendo as escritas à mão: é a porta do produto, e usá-la aqui é o
/// que faz a cena provar que ela funciona antes de o artista tocar em nada.
fn back_to_rest(app: &mut crate::App) {
    let ids = path_ids(app);
    if ids.len() < ART.len() {
        return;
    }
    for host in [PLAY, CARD, SHAPE] {
        let Some(gfx) = app.gfx.as_mut() else { return };
        crate::render_loop::ui_state_bridge::request(
            &mut gfx.ui_machines,
            &gfx.ui_states,
            ids[host],
            StateRole::Default,
        );
    }
}

fn announce(app: &mut crate::App) {
    let ids = path_ids(app);
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    if ids.len() < ART.len() {
        eprintln!("[ui-states] ⚠️ a cena nao montou — PARE");
        return;
    }
    let poses: usize = [PLAY, CARD, SHAPE]
        .iter()
        .map(|&h| gfx.ui_states.get(ids[h]).len())
        .sum();
    // ⚠️ O custo do PAR, medido aqui e não afirmado: um par só-de-cor tem de dizer ZERO. É esse
    // zero que vale 12,79 ms numa cena de vinte objetos.
    let plans = match (
        gfx.ui_states.role(ids[CARD], StateRole::Default),
        gfx.ui_states.role(ids[CARD], StateRole::Hover),
    ) {
        (Some(a), Some(b)) => Transition::new(&a.objects, &b.objects).plans_built(),
        _ => usize::MAX,
    };
    // ⚠️ E o do SHAPE tem de dizer **1**: se a forma mudou e ninguem a casou, o Show vai
    // TROCAR a forma no fim em vez de a fazer viajar — que era o defeito reportado.
    let shape_plans = match (
        gfx.ui_states.role(ids[SHAPE], StateRole::Default),
        gfx.ui_states.role(ids[SHAPE], StateRole::Hover),
    ) {
        (Some(a), Some(b)) => Transition::new(&a.objects, &b.objects).plans_built(),
        _ => usize::MAX,
    };
    eprintln!(
        "[ui-states] {poses} poses gravadas (Play, Card e Shape, Default+Hover); a transicao do \
         Card custou {plans} Plan(s) e a do Shape {shape_plans}."
    );
    if shape_plans != 1 {
        eprintln!(
            "[ui-states] ⚠️ **PARE**: a transicao do Shape tinha de custar 1 Plan. A forma nao \
             foi gravada, e o Show vai troca-la em vez de a animar."
        );
        return;
    }
    if poses < 6 {
        eprintln!("[ui-states] ⚠️ **PARE**: eram para ser 6 poses. A autoria nao correu.");
        return;
    }
    eprintln!("[ui-states] o roteiro:");
    eprintln!("  1. Selecione o **Play** -> secao **States**. Default e Hover ja' tem pose (Show");
    eprintln!("     e Clear aparecem neles); Pressed e Disabled so' oferecem **Rec**.");
    eprintln!("  2. ⚠️ **A PROVA DA WAVE**: aperte **Show** no Hover. A forma NAO salta — ela");
    eprintln!("     anda: cresce, o ponto desliza para a direita e a cor atravessa. Voce autorou");
    eprintln!("     duas pontas e o motor descobriu o meio.");
    eprintln!("  3. ⚠️ **O ESTADO E' DA SUB-ARVORE**: o ponto anda JUNTO com o pai, e ninguem o");
    eprintln!("     gravou separadamente. Um estado que so' guardasse o hospedeiro deixaria de");
    eprintln!("     fora justamente o que se move num hover.");
    eprintln!("  4. **Show** no Default devolve. Interrompa no meio (Show/Show/Show depressa): a");
    eprintln!("     forma inverte **de onde esta'**, nunca da ponta — a maquina parte da pose");
    eprintln!("     VIVA.");
    eprintln!("  5. ⚠️ **A CHEGADA E' EXATA**: va' e volte dez vezes. O botao termina onde voce o");
    eprintln!("     desenhou, ao bit. Sem isso a cena DERIVA e ninguem ve' de onde.");
    eprintln!("  6. **Duration**: arraste para ~1 s e repita o Show. A mesma animacao, mais");
    eprintln!("     lenta. Em 0 ela e' instantanea — e passa pela MESMA porta de chegada.");
    eprintln!("  7. ⚠️ **UM Ctrl+Z desfaz um Show**, nao nove. A transicao inteira e' um passo:");
    eprintln!("     o undo espera a maquina chegar, e so' entao ve' um estado do mundo.");
    eprintln!(
        "  8. **O CARD** so' muda de COR, e a transicao dele custa **0 Plan** (linha acima)."
    );
    eprintln!("     A cor atravessa pelo caminho perceptual, sem passar pelo cinza.");
    eprintln!("  9. ⚠️ **O CONTROLE**: selecione o **Plain**. A secao States EXISTE e esta' vazia");
    eprintln!("     — os quatro papeis so' oferecem Rec. E' a face vazia que torna a feature");
    eprintln!("     alcancavel; sem ela, gravar so' seria possivel onde ja' se gravou.");
    eprintln!(
        " 10. Pose o Plain como quiser e aperte **Rec** no Hover: a partir dai' ele responde"
    );
    eprintln!("     como os outros. **Clear** o esquece, e a linha volta a oferecer so' o Rec.");
    eprintln!(" 11. ⚠️ **O SHAPE E' A WAVE NOVA**: selecione-o e aperte **Show** no Hover. A");
    eprintln!("     forma MORFA — o no' sobe, as quinas ARREDONDAM e o traco engrossa no meio —,");
    eprintln!("     e as tres coisas sao as ferramentas do report: modo Node, Fillet e Width.");
    eprintln!(" 12. ⚠️ **E o que ele NAO perde**: volte ao Default e entre no modo **Node**. As");
    eprintln!("     alcas de quina continuam la', com o raio que voce autorou. A transicao passa");
    eprintln!("     geometria COZIDA pelo documento e a chegada devolve a FONTE — se as alcas");
    eprintln!("     sumissem, o Show teria assado o seu desenho.");
    eprintln!(" 13. ⚠️ **O MODO DE PREVIEW (W7r)** — a metade de RUNTIME. No topo da secao States");
    eprintln!("     ha' um botao **Preview**. Aperte. Ele ACENDE, a linha diz como sair, e a");
    eprintln!("     autoria inteira FECHA (nem Rec, nem Show, nem Clear, nem a duracao).");
    eprintln!(" 14. ⚠️ **Agora passe o rato por cima do Play e do Card, SEM clicar.** Eles");
    eprintln!("     reagem — e com o mesmo tween que voce autorou. Saia de um para o outro: o");
    eprintln!("     que voce DEIXA volta ao Default no mesmo gesto (se ficasse aceso, seria o");
    eprintln!("     defeito que um botao so' nunca mostra).");
    eprintln!(" 15. ⚠️ **APERTE e SEGURE** sobre o Play: ele vai para **Pressed** se voce gravou");
    eprintln!("     esse papel, e volta ao Hover ao soltar. Apertar no VAZIO nao prende ninguem.");
    eprintln!(" 16. ⚠️ **O clique nao pinta, nao seleciona e nao arrasta** dentro do modo — mas");
    eprintln!("     **pan e zoom continuam vivos** (o Figma faz igual: olhar de perto nao e'");
    eprintln!("     editar). E o painel continua clicavel: o botao Preview e' a porta de saida.");
    eprintln!(" 17. ⚠️ **A PROVA da wave: mova o Play com o gizmo ANTES de entrar** (ele fica");
    eprintln!("     longe do Default que voce gravou). Entre na preview, passe o rato por cima,");
    eprintln!("     saia por **Esc**. Ele tem de voltar para ONDE VOCE O DEIXOU, e nao para o");
    eprintln!("     Default gravado — sair para o Default MOVERIA o seu desenho. E o **Ctrl+Z**");
    eprintln!("     seguinte tem de desfazer o SEU move, nunca um passo que a preview inventou.");
    eprintln!(" 18. ⚠️ **RELOCAR O WIDGET (Enio 2026-08-07)** — o outro report. Saia da preview e");
    eprintln!("     grave um **Pressed** no Play com a forma LONGE da posicao inicial. Agora");
    eprintln!("     arraste o Play para outro canto e aperte **Show** no Pressed: ele volta para");
    eprintln!("     o lugar ANTIGO. E' o defeito: a translacao do hospedeiro esta' congelada em");
    eprintln!("     cada estado.");
    eprintln!(" 19. ⚠️ Marque **Move All States** e arraste o Play de novo. Agora **todos** os");
    eprintln!("     estados acompanham: Show no Pressed mostra a mesma animacao, no lugar novo.");
    eprintln!("     ⚠️ E o **DOT** (o filho) nao se desloca duas vezes — a pose dele e' local ao");
    eprintln!("     pai, entao ela ja' viaja junto: a coreografia interna fica intacta.");
    eprintln!(" 20. ⚠️ **E a caixa DESMARCADA tem de continuar a servir** — arraste com ela off e");
    eprintln!("     so' a pose de agora e' re-autorada. E' o que se quer quando a intencao e'");
    eprintln!("     corrigir UM estado, e por isso ela e' opt-in em vez de lei.");
    eprintln!();
    eprintln!("  --- O SELETOR DE CURVA (W7c) ---");
    eprintln!(" 21. Abaixo de **Duration** ha' agora **Curve** (onze familias) e **Direction**");
    eprintln!("     (In / Out / In-Out). O aceso e' o que o documento guarda: de fabrica,");
    eprintln!("     **Cubic** + **Out**. Ate' hoje esse par era o UNICO alcancavel — o campo ja'");
    eprintln!("     viajava no arquivo e nao havia gesto nenhum que o escrevesse.");
    eprintln!(" 22. Escolha **Elastic**, entre em Preview e passe o rato pelo Play. A forma passa");
    eprintln!("     do alvo e volta. Escolha **Bounce**: ela quica. E' a MESMA transicao — o que");
    eprintln!("     mudou foi a curva, e o Duration continua a dizer quanto tempo ela leva.");
    eprintln!(" 23. ⚠️ **Escolha Linear: a fileira Direction DESAPARECE.** Nao e' esquecimento —");
    eprintln!(
        "     Linear ignora a direcao (a curva e' a mesma nas tres), entao oferece-la seriam"
    );
    eprintln!(
        "     tres botoes a desenhar a mesma coisa. Volte a **Quad** e ela reaparece **com a"
    );
    eprintln!("     direcao que voce tinha escolhido**: passar por Linear nao apaga a escolha.");
    eprintln!(" 24. ⚠️ **O que ESPERAR de mau, porque esta' medido e e' decisao sua:** com");
    eprintln!("     **In-Out** escolhido, interromper um hover no meio faz a forma **parar e");
    eprintln!(
        "     recomecar** (a volta arranca do repouso — 0,00x da velocidade que trazia); com"
    );
    eprintln!("     **Elastic**, ela arranca a 7,02x. A POSE nunca salta, so' a velocidade. E'");
    eprintln!("     inerente a animacao por CURVA — a cura chama-se mola, e ela e' outra wave.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O CONTROLE é mesmo um controle.**
    ///
    /// ⚠️ A quarta forma existe para não se mexer, e a única coisa que garante isso é as duas
    /// tintas dela serem a MESMA. Alguém que "clareie as cores de hover" da lista apaga o
    /// controle em silêncio — a cena continua bonita e deixa de provar que gravar um estado é a
    /// única coisa que faz uma forma responder.
    #[test]
    fn the_control_shape_has_no_hover_of_its_own() {
        let last = ART.len() - 1;
        assert_eq!(
            REST[last], HOVER[last],
            "a forma de CONTROLE ganhou uma cor de hover — ela deixou de ser controle"
        );
        for i in [PLAY, DOT, CARD, SHAPE] {
            assert_ne!(
                REST[i], HOVER[i],
                "o objeto {i} nao muda de cor entre os dois estados — a cena nao mostraria a \
                 travessia perceptual"
            );
        }
    }

    /// **As cinco formas não se sobrepõem** — e o Shape precisa de espaço para CRESCER.
    ///
    /// ⚠️ A premissa que se apaga em silêncio: o hover do Shape puxa um nó **para cima** e
    /// arredonda as quinas, então uma caixa colada na do vizinho faria a prova da wave parecer
    /// uma colisão. O gate mede a folga horizontal entre caixas irmãs.
    #[test]
    fn the_shapes_do_not_touch_each_other() {
        for w in ART.windows(2) {
            let (a, b) = (w[0].0, w[1].0);
            // O ponto vive DENTRO do Play — o par que ele forma é o único isento.
            if b[0] > a[0] && b[2] < a[2] {
                continue;
            }
            assert!(
                b[0] > a[2] || b[2] < a[0],
                "as caixas {a:?} e {b:?} se tocam: a cena leria como colisao"
            );
        }
    }

    /// **O ponto é FILHO do Play**, e é isso que a cena prova sobre a sub-árvore.
    ///
    /// ⚠️ O gate é sobre a GEOMETRIA da fixture: o ponto tem de caber DENTRO do retângulo do Play,
    /// senão a foto mostra duas coisas lado a lado e a frase *"o filho anda junto"* fica sem
    /// sujeito visível.
    #[test]
    fn the_dot_sits_inside_the_play_button() {
        let (p, d) = (ART[PLAY].0, ART[DOT].0);
        assert!(
            d[0] > p[0] && d[2] < p[2] && d[1] > p[1] && d[3] < p[3],
            "o ponto saiu de dentro do Play: {d:?} contra {p:?}"
        );
    }
}
