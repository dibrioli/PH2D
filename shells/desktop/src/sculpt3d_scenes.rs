//! **AS CENAS DO SMOKE** — com que malha cada uma abre, e o que ela declara.
//!
//! Filho (`#[path]`) de [`super`], e o corte é entre *o que a cena VIVA é* (lá:
//! a malha, a câmera, o pincel, o passe) e *que fixture cada cena de smoke
//! monta* (aqui). São assuntos diferentes: uma é o produto, a outra é o que se
//! põe na frente do Enio para ele julgar o produto — e a segunda cresce uma
//! entrada por wave.
//!
//! ⚠️ **Toda fixture aqui é construída com os VERBOS do produto**, nunca com
//! geometria fabricada à mão: um relevo escrito direto nos vértices seria uma
//! segunda resposta a *"como uma crista é feita"*, e ela divergiria da primeira
//! no dia em que o depósito mudasse.

use super::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// A cena está armada? (`PH2D_SCULPT3D_SMOKE` em `1`..`7`.)
pub(crate) fn smoke_armed() -> bool {
    matches!(
        std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref(),
        Some("1" | "2" | "3" | "4" | "5" | "6" | "7")
    )
}

/// `=7` — **A CENA**: mais de um objeto, cada um com a sua pose.
pub(crate) fn objects_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("7")
}

/// **As peças que a cena `=7` põe na mesa**, além da que a cena já abre.
///
/// ⚠️ Formas DIFERENTES de propósito, e não três esferas: o que o smoke julga é
/// *"o pincel caiu na peça que eu cliquei"*, e três cópias da mesma silhueta
/// tornariam a resposta certa indistinguível da errada. Tamanhos diferentes pelo
/// mesmo motivo — a escala é metade da pose, e um trio de peças do mesmo tamanho
/// deixaria essa metade sem oráculo nenhum na tela.
pub(crate) fn scene_objects() -> Vec<(ph2d_mesh::Mesh, ph2d_mesh::Pose)> {
    vec![
        // O CUBO, à esquerda e GRANDE: a peça em que a escala se vê.
        (
            ph2d_mesh::shapes::cube(1.0),
            ph2d_mesh::Pose::new([-2.6, 0.0, 0.0], 1.4),
        ),
        // O OCTAEDRO, à direita e pequeno.
        (
            ph2d_mesh::shapes::octahedron(1.0),
            ph2d_mesh::Pose::new([2.2, 0.0, 0.0], 0.6),
        ),
    ]
}

/// `=5` — a cena do **TWIST e do LOCAL SCALE**: uma esfera com CRISTAS.
pub(crate) fn turn_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("5")
}

/// A esfera com uma CRUZ de cristas — o modelo em que um giro se VÊ.
///
/// ⚠️ **Uma esfera lisa é a fixture errada para o Twist, e não por gosto:** ela é
/// invariante por rotação em torno de qualquer eixo que passe pelo centro, então
/// torcer a calota frontal a deixa quase idêntica a ela mesma. O gesto
/// funcionaria e o smoke não teria como dizer. As cristas dão ao redemoinho uma
/// forma que o olho segue — e ao inchaço do Local Scale uma referência de
/// tamanho.
///
/// ⚠️ **Elas são desenhadas com o VERBO do produto**: um relevo escrito à mão nos
/// vértices seria uma segunda resposta a *"como uma crista é feita"*, e ela
/// divergiria da primeira no dia em que o depósito mudasse.
///
/// ⚠️ **São TRÊS traços, e a contagem sai da lei do envelope.** Um traço deixa
/// exatamente um `reach` de altura por mais dabs que ele carimbe — é essa a
/// promessa —, e um `reach` aqui é `raio × 0,2 = 0,028`, cerca de 1,4% do
/// diâmetro: no enquadramento padrão isso são ~5 px, uma ondulação que o olho
/// não segue. Cada `begin` re-congela o `pre`, então os traços empilham; três é
/// o que a medição pôs em ~4% do diâmetro (gate abaixo).
fn ridged_sphere() -> ph2d_mesh::Mesh {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    let brush = Brush {
        verb: Verb::Draw,
        radius: 0.14,
        strength: 1.0,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    const STEPS: usize = 48;
    for _ in 0..3 {
        stroke.begin(&mesh);
        for i in 0..=STEPS {
            let u = -0.6 + 1.2 * i as f32 / STEPS as f32;
            for c in [
                [u, 0.0, (1.0 - u * u).max(1e-3).sqrt()],
                [0.0, u, (1.0 - u * u).max(1e-3).sqrt()],
            ] {
                stroke.dab(
                    &mut mesh,
                    &brush,
                    // O olho aponta da superfície para o centro: é o raio que
                    // teria produzido este acerto vindo de fora.
                    &Dab::at(c, brush.radius, [-c[0], -c[1], -c[2]]),
                    Symmetry::default(),
                );
            }
        }
    }
    mesh
}

/// `=6` — a cena do **REMESH**: uma esfera com um bico ESTICADO até o barro
/// acabar.
pub(crate) fn remesh_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("6")
}

/// A esfera com um BICO puxado longe demais — o modelo que pede um remesh.
///
/// ⚠️ **É a fixture certa porque ela nomeia o problema que o botão resolve**, e
/// uma esfera lisa não nomeia nenhum. Um SnakeHook longo arrasta os mesmos
/// vértices por uma distância grande: a densidade do bico despenca, os
/// triângulos ficam compridos e finos, e a partir de certo ponto **não há mais
/// barro ali para esculpir**. O remesh devolve densidade uniforme, e é isso que
/// o artista vai julgar — não a forma, que tem de sobreviver.
///
/// ⚠️ **Puxado com o VERBO do produto**, como toda fixture deste arquivo: um bico
/// escrito à mão nos vértices seria uma segunda resposta a *"o que um SnakeHook
/// faz"*, e ela divergiria da primeira no dia em que o gancho mudasse.
fn hooked_sphere() -> ph2d_mesh::Mesh {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
    let brush = Brush {
        verb: Verb::SnakeHook,
        radius: 0.35,
        strength: 1.0,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    // O gancho AGARRA e leva: cada passo puxa o que está sob ele mais um pouco,
    // e é a soma deles que estica o barro além do que a malha representa.
    const STEPS: usize = 40;
    let mut tip = [0.0f32, 0.0, 1.0];
    for _ in 0..STEPS {
        let step = [0.0f32, 0.028, 0.028];
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::hooking(tip, brush.radius, [0.0, 0.0, -1.0], step),
            Symmetry::default(),
        );
        for k in 0..3 {
            tip[k] += step[k];
        }
    }
    mesh
}

/// `=3` — a cena da **REVERSÃO**: um modelo denso que É uma subdivisão.
pub(crate) fn reversion_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("3")
}

/// `=4` — a cena de **FECHAR BURACO**: uma esfera com um pedaço arrancado.
pub(crate) fn holes_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("4")
}

/// A esfera com um FURO — o modelo que chega quebrado.
///
/// ⚠️ **O furo é feito ARRANCANDO faces, não desenhando uma beira**, e é isso que
/// o torna a fixture certa: a beira resultante é a que uma malha de verdade tem
/// (um contorno irregular, seguindo a grade da esfera), e não uma que eu
/// escolhi. Os vértices que ficam sem face nenhuma sobrevivem soltos, como
/// sobreviveriam num OBJ de terceiro.
fn punctured_sphere() -> ph2d_mesh::Mesh {
    let sphere = ph2d_mesh::shapes::uv_sphere(24, 32, 1.0);
    let keep: Vec<ph2d_mesh::Face> = sphere
        .faces()
        .iter()
        .copied()
        .filter(|f| {
            let vs = f.verts();
            let n = vs.len() as f32;
            let mut c = [0.0f32; 3];
            for &v in vs {
                let p = sphere.positions()[v as usize];
                for k in 0..3 {
                    c[k] += p[k] / n;
                }
            }
            // Uma calota lateral, bem visível no enquadramento padrão.
            !(c[0] > 0.45 && c[1] > 0.15)
        })
        .collect();
    ph2d_mesh::Mesh::from_parts(sphere.positions().to_vec(), keep)
        .expect("arrancar face não inventa índice")
}

/// A malha com que cada cena abre.
///
/// ⚠️ **Porta única, e ela existe para o gate.** A cena `=3` só significa alguma
/// coisa se a malha dela de fato reverter, e isso é um fato sobre a GEOMETRIA
/// que nenhum arch-gate de fonte enxerga. Um gate que reconstruísse a malha por
/// conta própria estaria medindo outra malha no dia em que esta mudasse.
#[must_use]
pub(crate) fn smoke_mesh() -> ph2d_mesh::Mesh {
    if turn_scene() {
        return ridged_sphere();
    }
    if remesh_scene() {
        return hooked_sphere();
    }
    if holes_scene() {
        return punctured_sphere();
    }
    if reversion_scene() {
        // ⚠️ **Ela é DUAS vezes subdividida de propósito**: um modelo denso que
        // chega pronto não tem um nível embaixo, e a cena só demonstra a
        // reversão se houver mais de um para reconstruir. A esfera UV mistura
        // quads no corpo com triângulos nos polos, que é o caso que exercita os
        // dois ramos do reconhecedor de uma vez.
        let coarse = ph2d_mesh::shapes::uv_sphere(12, 18, 1.0);
        ph2d_mesh::subdivide(&ph2d_mesh::subdivide(&coarse))
    } else {
        ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)
    }
}

/// `=2` — a cena da **DOAÇÃO**: a esfera E uma tela branca para pintar.
///
/// ⚠️ Cena própria, e não um passo a mais na `=1`: julgar a escultura e julgar a
/// doação são duas perguntas, e a segunda precisa de uma tela que a primeira não
/// quer ver. Misturá-las faria o smoke do barro abrir com um retângulo branco
/// atrás dele sem nada explicando por quê.
pub(crate) fn donation_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("2")
}

/// **A cena DECLARA o que montou** — o banner e as instruções de cada uma.
///
/// ⚠️ Mora aqui, ao lado da fixture, e não no arquivo do gesto: *que malha esta
/// cena monta* e *o que ela pede ao artista para julgar* são a mesma pergunta,
/// e mantê-las separadas foi o que deixou uma cena declarar um número que a
/// outra metade não produzia. O gesto ficou com o gesto.
///
/// ⚠️ E declarar não é cortesia: um smoke que não diz o que montou é
/// indistinguível da feature quebrada — a lição que o smoke do Colorize pagou, e
/// que as cenas `=4` e `=6` pagam de novo com um NÚMERO (a beira, a aresta).
pub(crate) fn announce(mesh: &ph2d_mesh::Mesh) {
    // A cena IMPRIME o que montou. Um smoke que não se declara deixa o
    // artista sem saber se está vendo a feature ou o app vazio — a lição
    // que o smoke do Colorize pagou.
    eprintln!(
        "[sculpt3d] esfera com {} vértices / {} faces / {} triângulos\n\
         [sculpt3d] ESQUERDO esculpe (fora do modelo, gira) · DIREITO gira · MEIO desloca · RODA aproxima\n\
         [sculpt3d] Shift = Smooth enquanto segurar · Ctrl inverte Draw/Inflate/Clay/Crease e limpa a mascara\n\
         [sculpt3d] 1..9,0 escolhem o verbo · A alarga (magnify) · M mascara · [ ] tamanho · X/Y/Z espelho · Ctrl+Z desfaz\n\
         [sculpt3d] o pincel mede PIXELS DE TELA: aproxime com a roda e ele continua do mesmo tamanho\n\
         [sculpt3d] a MASCARA (M) protege o que ela pinta e se VE (azul frio): C limpa · I inverte · B borra · N afia\n\
         [sculpt3d] K = SUBDIVIDIR: 4 faces onde havia 1, e a forma ALISA (Catmull-Clark/Loop)\n\
         [sculpt3d]     o log diz a contagem nova a cada toque -- ela quadruplica; Ctrl+Z desfaz\n\
         [sculpt3d] , e . DESCEM e SOBEM na pilha de niveis: esculpa fino em cima, volte ao 0\n\
         [sculpt3d]     para mover a FORMA GRANDE, e suba -- o detalhe fino continua la'\n\
         [sculpt3d] J = DES-SUBDIVIDIR: reconstroi um nivel ABAIXO da base (o par do K)\n\
         [sculpt3d]     so' funciona se a malha JA' for uma subdivisao -- o log diz quando nao e'\n\
         [sculpt3d] O = TAPAR BURACO: todo contorno aberto ganha uma tampa (e o log diz quantos)\n\
         [sculpt3d] V = RECONSTRUIR (voxel remesh): a malha vira um campo e volta com densidade\n\
         [sculpt3d]     UNIFORME -- e' o que devolve barro onde um estica'o o gastou; a forma fica\n\
         [sculpt3d] G = PEGAR o barro (grab): segure e arraste, e ele vem com o dedo\n\
         [sculpt3d] H = ESTICAR (snake hook): a pegada ANDA com o cursor e sai um espinho\n\
         [sculpt3d]     o G volta ao lugar quando voce volta; o H deixa a ponta la' -- essa e' a diferenca\n\
         [sculpt3d] T = TORCER (twist): segure e VARRA um circulo em volta do ponto que voce pegou\n\
         [sculpt3d] S = INFLAR/ENCOLHER (local scale): segure e arraste na HORIZONTAL\n\
         [sculpt3d]     os dois voltam ao lugar quando voce varre de volta -- o gesto e' o TOTAL, nao a soma\n\
         [sculpt3d] A LUZ e o rig do artista (o mesmo que acende a tinta): Q/E giram a lampada, R/F a sobem\n\
         [sculpt3d] o espelho nasce DESLIGADO; PH2D_SCULPT3D_DIAG=1 mede se o pincel cai sob o cursor",
        mesh.vert_count(),
        mesh.face_count(),
        mesh.triangle_count()
    );
    if crate::sculpt3d::holes_scene() {
        // ⚠️ **A cena DECLARA o furo que montou.** Um smoke de fechar buraco
        // sobre uma malha sem buraco é indistinguível da feature quebrada —
        // a lição que o smoke do Colorize pagou, e aqui o número é a beira.
        let edges = mesh.edges();
        let border = (0..edges.len())
            .filter(|&e| edges.valence(u32::try_from(e).unwrap_or(u32::MAX)) == 1)
            .count();
        eprintln!(
            "[sculpt3d] =4 FECHAR BURACO: a malha abre com {border} arestas de BEIRA -- se este\n\
             [sculpt3d]    numero for zero, PARE: nao ha' buraco e o resto do smoke nao diz nada.\n\
             [sculpt3d]    Esta esfera CHEGOU QUEBRADA -- gire com o botao direito\n\
             [sculpt3d]    ate' o furo, e olhe POR DENTRO dela (nao ha' culling: o interior aparece).\n\
             [sculpt3d]    Aperte O: o log diz quantos buracos tapou, e o furo vira uma TAMPA.\n\
             [sculpt3d]    A tampa e' um leque a partir do centro do contorno, entao ela AFUNDA --\n\
             [sculpt3d]    passe o Smooth (3) nela e ela vira superficie. Ctrl+Z desfaz.\n\
             [sculpt3d]    Depois de tapada, K subdivide e o modelo fica solido de verdade."
        );
    }
    if crate::sculpt3d::turn_scene() {
        // ⚠️ **A cena DECLARA que trouxe cristas.** Numa esfera LISA um
        // Twist em torno do eixo da vista é quase invisível — ela é
        // invariante por rotação —, e o smoke não teria como separar a
        // feature funcionando da feature morta.
        eprintln!(
            "[sculpt3d] =5 TORCER e INFLAR: esta esfera tem uma CRUZ de cristas, e ela existe\n\
             [sculpt3d]    porque numa esfera LISA um giro em torno do eixo da vista nao se ve.\n\
             [sculpt3d]    Aperte T, pegue o CRUZAMENTO das cristas e VARRA um circulo em volta\n\
             [sculpt3d]    dele: os bracos entortam em redemoinho. Varra de VOLTA ao comeco --\n\
             [sculpt3d]    a cruz tem de voltar reta (o gesto e' o TOTAL varrido, nao a soma dos passos).\n\
             [sculpt3d]    Perto do ponto que voce pegou ha' uma ZONA MORTA de 30 px: ali a direcao\n\
             [sculpt3d]    e' ruido, e nada gira ate' voce sair dela.\n\
             [sculpt3d]    Aperte S e arraste na HORIZONTAL: para a direita o cruzamento incha,\n\
             [sculpt3d]    para a esquerda ele encolhe -- e volta ao lugar no caminho de volta.\n\
             [sculpt3d]    Aperte X (espelho) e repita o T: as duas metades tem de girar para\n\
             [sculpt3d]    lados OPOSTOS (um redemoinho no espelho gira ao contrario); com o S\n\
             [sculpt3d]    as duas metades incham JUNTAS."
        );
    }
    if crate::sculpt3d::remesh_scene() {
        // ⚠️ **A cena DECLARA o esticamento que montou.** Um smoke de remesh
        // sobre uma malha saudável é indistinguível da feature quebrada: a
        // forma sobrevive nos dois casos, e é só a DENSIDADE que muda. O
        // número aqui é a maior aresta — a mesma lição da cena `=4`.
        let pos = mesh.positions();
        let mut longest = 0.0f32;
        let mut tris = Vec::new();
        mesh.triangle_indices(&mut tris);
        for t in &tris {
            for k in 0..3 {
                let a = pos[t[k] as usize];
                let b = pos[t[(k + 1) % 3] as usize];
                let d =
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
                longest = longest.max(d);
            }
        }
        eprintln!(
            "[sculpt3d] =6 O REMESH: a maior aresta desta malha mede {longest:.3} -- se este numero\n\
             [sculpt3d]    nao passar de ~0.15, PARE: o bico nao esticou e o resto nao diz nada.\n\
             [sculpt3d]    Esta esfera foi PUXADA por um snake hook ate' o barro acabar: gire e olhe\n\
             [sculpt3d]    o bico -- ele esta' FACETADO, feito de poucos triangulos compridos.\n\
             [sculpt3d]    (1) Tente esculpir na PONTA dele (Draw, tecla 1): quase nada acontece,\n\
             [sculpt3d]        porque nao ha' vertices ali. Essa e' a doenca.\n\
             [sculpt3d]    (2) Aperte V: o log diz vertices ANTES -> DEPOIS. A FORMA tem de\n\
             [sculpt3d]        sobreviver -- se o bico sumir ou a esfera virar outra coisa, reprove.\n\
             [sculpt3d]    (3) Esculpa na MESMA ponta de novo: agora ela responde. Esse e' o botao.\n\
             [sculpt3d]    (4) Ctrl+Z devolve a malha esticada, inteira; Ctrl+Shift+Z refaz.\n\
             [sculpt3d]    (5) Aperte K (subdividir) e depois V: ele RECUSA, e o log diz por que --\n\
             [sculpt3d]        um remesh troca a topologia, e os niveis de cima sao subdivisao dela."
        );
    }
    if crate::sculpt3d::objects_scene() {
        eprintln!(
            "[sculpt3d] =7 A CENA E' UMA LISTA: tres pecas, cada uma no SEU lugar e no SEU tamanho.\n\
             [sculpt3d]    Uma esfera no meio, um CUBO grande a' esquerda, um OCTAEDRO pequeno a' direita.\n\
             [sculpt3d]    (1) Gire (botao direito) e olhe: as tres tem de estar la', separadas, e a\n\
             [sculpt3d]        perspectiva tem de ser coerente -- nenhuma pode nadar em relacao as outras.\n\
             [sculpt3d]    (2) Esculpa no CUBO (esquerdo). O barro tem de ceder EXATAMENTE sob o cursor,\n\
             [sculpt3d]        e a pegada tem de ter o mesmo tamanho APARENTE que na esfera do meio --\n\
             [sculpt3d]        e' isso que prova que o pincel atravessou a escala da peca.\n\
             [sculpt3d]    (3) Esculpa no OCTAEDRO (pequeno, a' direita): mesma coisa. Se a pegada dele\n\
             [sculpt3d]        parecer MAIOR ou MENOR que a das outras, reprove.\n\
             [sculpt3d]    (4) Esculpa uma peca, depois OUTRA, e aperte Ctrl+Z duas vezes: cada undo tem\n\
             [sculpt3d]        de desfazer NA PECA CERTA. Se a segunda peca 'consertar' a primeira, reprove.\n\
             [sculpt3d]    (5) Aproxime com a roda ate' o cubo ocupar a tela e esculpa: o pincel continua\n\
             [sculpt3d]        do mesmo tamanho em PIXELS, como sempre foi.\n\
             [sculpt3d]    (6) Onde as pecas se cruzam na tela, clicar tem de pegar a que esta' NA FRENTE.\n\
             [sculpt3d]    (7) OS VERBOS DA LISTA: Shift+1 esfera, Shift+2 cubo, Shift+3 cilindro,\n\
             [sculpt3d]        Shift+4 toro. A peca nova nasce ONDE VOCE ESTA' OLHANDO e ja' vem ativa --\n\
             [sculpt3d]        esculpa nela sem clicar em mais nada.\n\
             [sculpt3d]    (8) Shift+D DUPLICA a ativa: a copia nasce AO LADO na tela (gire e confira que\n\
             [sculpt3d]        ela continua ao lado do ponto de vista NOVO, nao presa a um eixo de mundo).\n\
             [sculpt3d]    (9) Delete APAGA a ativa, e Ctrl+Z tem de devolve-la INTEIRA -- com o que voce\n\
             [sculpt3d]        esculpiu nela. Tente apagar ate' sobrar UMA: a ultima o log RECUSA.\n\
             [sculpt3d]   (10) O teste duro do undo: esculpa a peca A, acrescente B, esculpa B, apague B,\n\
             [sculpt3d]        e va' desfazendo. Cada passo tem de voltar NA PECA CERTA, na ordem inversa."
        );
    }
    if crate::sculpt3d::reversion_scene() {
        eprintln!(
            "[sculpt3d] =3 A REVERSAO: esta malha densa CHEGOU PRONTA -- um nivel so', e por isso\n\
             [sculpt3d]    o ',' nao leva a lugar nenhum. Aperte J: a malha NAO muda de forma\n\
             [sculpt3d]    (e' esse o ponto), e nasce um nivel ABAIXO dela. Aperte J de novo.\n\
             [sculpt3d]    Agora ',' desce ate' a base grossa: mova UM vertice la' e suba com '.'\n\
             [sculpt3d]    -- a forma grande andou e a pele fina continua onde estava.\n\
             [sculpt3d]    Ctrl+Z desfaz cada J; Ctrl+Shift+Z refaz."
        );
    }
    if crate::sculpt3d::donation_scene() {
        eprintln!(
            "[sculpt3d] =2 A DOACAO: ha uma TELA BRANCA embaixo, e a tecla D alterna\n\
             [sculpt3d]    BARRO (esculpir) -> LUZ (a forma acende a tinta) -> DESLIGADA (o A/B)\n\
             [sculpt3d]    esculpa, aperte D, pegue o Painter e pinte CHAPADO: a tinta tem de sair ACESA\n\
             [sculpt3d]    aperte D de novo e a MESMA tinta fica plana -- e essa diferenca e a wave"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ **A cena `=6` só significa alguma coisa se o bico dela estiver
    /// ESTICADO** — e a forma sobrevive ao remesh nos dois casos, então a
    /// densidade é a única coisa que separa a feature funcionando da morta. O
    /// oráculo é a maior ARESTA, que é a medida do esticamento.
    #[test]
    fn the_remesh_scene_opens_with_a_stretched_spike() {
        let mesh = hooked_sphere();
        let pos = mesh.positions();
        let mut tris = Vec::new();
        mesh.triangle_indices(&mut tris);
        let mut longest = 0.0f32;
        for t in &tris {
            for k in 0..3 {
                let a = pos[t[k] as usize];
                let b = pos[t[(k + 1) % 3] as usize];
                longest = longest.max(
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt(),
                );
            }
        }
        // A esfera de 48×72 tem aresta ~0.09 em repouso; o gancho tem de
        // multiplicar isso, senão não há barro gasto a demonstrar.
        assert!(
            longest > 0.15,
            "a maior aresta mede {longest:.4}: o gancho nao esticou nada"
        );
        // E a ponta tem de ter SAÍDO da esfera — um bico que não anda é um
        // esticamento que o olho não encontra.
        let far = mesh
            .positions()
            .iter()
            .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
            .fold(0.0f32, f32::max);
        assert!(far > 1.5, "a ponta chegou so' a {far:.3} de raio");
    }

    /// ⚠️ **A cena `=5` só significa alguma coisa se a esfera dela TIVER cristas**,
    /// e isso é um fato sobre geometria que nenhum arch-gate de fonte enxerga —
    /// o mesmo argumento do gate da cena `=3`, que pina que ela é construída
    /// subdividindo.
    ///
    /// ⚠️ **O oráculo tem duas metades, e a segunda é a que importa:** a crista
    /// tem de subir E a região LISA tem de ficar lisa. Só a primeira ficaria
    /// verde se o traço vazasse pela esfera inteira — e aí a fixture não teria
    /// forma a seguir, que é exatamente o que ela existe para dar.
    #[test]
    fn the_turn_scene_opens_with_a_sphere_that_has_ridges() {
        let mesh = ridged_sphere();
        let (mut on, mut off) = (0.0f32, 0.0f32);
        for p in mesh.positions() {
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            // A cruz vive na calota `+Z`, ao longo dos planos `y = 0` e `x = 0`.
            if p[2] < 0.7 {
                continue;
            }
            if p[0].abs() < 0.05 || p[1].abs() < 0.05 {
                on = on.max(r - 1.0);
            } else if p[0].abs() > 0.3 && p[1].abs() > 0.3 {
                off = off.max((r - 1.0).abs());
            }
        }
        assert!(
            on > 0.04,
            "a crista subiu só {on:.4} do raio — numa esfera de diâmetro 2 isso não se segue com o olho"
        );
        assert!(
            off < 0.005,
            "a região LISA subiu {off:.4}: o traço vazou, e a fixture perdeu a forma que ela existe para dar"
        );
    }
}
