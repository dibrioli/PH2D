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

use super::fixtures::{hooked_sphere, punctured_sphere, ridged_sphere};

/// A cena está armada? (`PH2D_SCULPT3D_SMOKE` em `1`..`11`.)
pub(crate) fn smoke_armed() -> bool {
    matches!(
        std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref(),
        Some("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" | "11")
    )
}

/// `=9` — a cena do **IMPORT**: um arquivo para o artista soltar na janela.
pub(crate) fn import_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("9")
}

/// **Escreve o OBJ-fixture da cena `=9`** e devolve o caminho.
///
/// ⚠️ **A cena FABRICA o arquivo em vez de pedir um ao artista**, e o motivo é
/// que ela precisa de um que CONTENHA o fenômeno: dois objetos (`o`), longe da
/// origem e enormes. Um `.obj` qualquer que estivesse à mão poderia já vir
/// centrado e do tamanho certo — e o smoke ficaria verde sem exercitar nada.
fn write_import_fixture() -> std::path::PathBuf {
    // Duas pirâmides: a "cabeça" pequena acima da "corpo" grande, as duas a 400
    // unidades da origem e medindo centenas de unidades. É o arquivo que sai de
    // um software de modelagem com o modelo onde o autor o deixou.
    let mut obj = String::from("# fixture do smoke =9 -- 2 objetos, longe do zero, enorme\n");
    let piece = |obj: &mut String, name: &str, at: [f32; 3], s: f32, base: usize| {
        obj.push_str(&format!("o {name}\n"));
        for (dx, dy, dz) in [
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.5, 0.0, 1.0),
            (0.5, 1.0, 0.5),
        ] {
            obj.push_str(&format!(
                "v {} {} {}\n",
                at[0] + dx * s,
                at[1] + dy * s,
                at[2] + dz * s
            ));
        }
        for (a, b, c) in [(1, 2, 4), (2, 3, 4), (3, 1, 4), (1, 3, 2)] {
            obj.push_str(&format!("f {} {} {}\n", base + a, base + b, base + c));
        }
    };
    piece(&mut obj, "corpo", [400.0, 400.0, 400.0], 300.0, 0);
    piece(&mut obj, "cabeca", [500.0, 750.0, 500.0], 120.0, 4);

    let path = std::env::temp_dir().join("ph2d_smoke_import.obj");
    if let Err(e) = std::fs::write(&path, obj) {
        eprintln!("[sculpt3d] =9 NAO consegui escrever o fixture: {e}");
    }
    path
}

/// `=7` — **A CENA**: mais de um objeto, cada um com a sua pose.
///
/// ⚠️ Privada: o bootstrap não pergunta mais *qual cena é esta*, ele pergunta
/// *quais peças eu ponho* ([`scene_objects`]).
fn objects_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("7")
}

/// `=8` — a cena do **DOCUMENTO**: a escultura que tem de sobreviver a fechar o app.
pub(crate) fn document_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("8")
}

/// **As peças que uma cena põe na mesa**, além da que ela já abre — vazio nas
/// que abrem com uma peça só.
///
/// ⚠️ **UMA porta para duas cenas, e não um `if` por cena no bootstrap.** A
/// pergunta que o `sculpt3d_smoke` faz é *"esta cena tem mais peças?"*, e ela é
/// a mesma para a `=7` e para a `=8`; um segundo ramo lá seria a lista de cenas
/// escrita num lugar que não é o das cenas, e ela apodrece na nona.
///
/// ⚠️ Formas DIFERENTES de propósito, e não três esferas: o que a `=7` julga é
/// *"o pincel caiu na peça que eu cliquei"*, e três cópias da mesma silhueta
/// tornariam a resposta certa indistinguível da errada. Tamanhos diferentes pelo
/// mesmo motivo — a escala é metade da pose, e um trio de peças do mesmo tamanho
/// deixaria essa metade sem oráculo nenhum na tela.
pub(crate) fn scene_objects() -> Vec<(ph2d_mesh::Mesh, ph2d_mesh::Pose)> {
    if import_scene() {
        // ⚠️ **A cena DECLARA o caminho do arquivo que escreveu.** Um smoke de
        // import sem um arquivo para soltar é indistinguível da feature
        // quebrada — e um arquivo já centrado não exercitaria nada, então este
        // vem a 400 unidades da origem e medindo centenas.
        let path = write_import_fixture();
        eprintln!(
            "[sculpt3d] =9 O IMPORT: escrevi um OBJ de DOIS objetos em\n\
             [sculpt3d]    {}\n\
             [sculpt3d]    Ele esta' a 400 unidades da origem e mede ~450 -- que e' como um\n\
             [sculpt3d]    arquivo de verdade chega. Se a linha acima nao aparecer, PARE.\n\
             [sculpt3d]    1) Aperte Ctrl+SHIFT+O e escolha esse arquivo.\n\
             [sculpt3d]       Duas piramides tem de aparecer, do tamanho da esfera e AO LADO\n\
             [sculpt3d]       dela -- nao por cima, e nao fora do quadro.\n\
             [sculpt3d]       (ARRASTAR o arquivo faz o mesmo -- em X11, macOS e Windows. No\n\
             [sculpt3d]       WAYLAND o winit 0.30 nao entrega arquivo soltado, entao o cursor\n\
             [sculpt3d]       para na beirada da janela: e' a plataforma, nao esta feature, e\n\
             [sculpt3d]       vale para o drop de IMAGEM tambem.)\n\
             [sculpt3d]    2) A cabeca tem de estar ACIMA do corpo: o arranjo do arquivo\n\
             [sculpt3d]       sobrevive, e a cabeca continua menor que o corpo.\n\
             [sculpt3d]    3) Clique numa delas e aperte X (espelho), depois esculpa:\n\
             [sculpt3d]       a copia espelhada tem de sair DENTRO da peca. Se ela sair longe,\n\
             [sculpt3d]       o plano do espelho ficou fora do modelo -- e' a divida desta wave.\n\
             [sculpt3d]    4) Ctrl+Z desfaz o import peca por peca.\n\
             [sculpt3d]    5) Aperte Ctrl+O (sem shift): ele tem de continuar sendo o LOAD de\n\
             [sculpt3d]       projeto -- o import nao pode ter comido o atalho do vizinho.",
            path.display()
        );
    }
    if export_scene() {
        // ⚠️ **A fixture TEM de conter as três coisas que um formato pode
        // perder**, senão o smoke não distingue um export honesto de um que
        // joga fora metade: peças SEPARADAS (só o OBJ as guarda), COR pintada
        // (o STL não a tem) e POSES diferentes (sem elas, *local* e *mundo*
        // coincidem e o gate mais importante fica verde por vácuo).
        let mut a = ph2d_mesh::shapes::cube(1.0);
        for (i, c) in a.colors_mut().iter_mut().enumerate() {
            *c = if i % 2 == 0 {
                [0.95, 0.25, 0.15]
            } else {
                [0.15, 0.35, 0.95]
            };
        }
        let mut b = ph2d_mesh::shapes::octahedron(1.0);
        for c in b.colors_mut() {
            *c = [0.2, 0.85, 0.3];
        }
        eprintln!(
            "[sculpt3d] =10 A PORTA DE SAIDA: tres pecas, COLORIDAS, em poses diferentes.\n\
             [sculpt3d]    O oraculo e' a IDA E VOLTA, e ela nao precisa de outro programa.\n\
             [sculpt3d]    1) Ctrl+Shift+E e salve como  volta.obj  -- o toast diz quantas\n\
             [sculpt3d]       pecas sairam e o que o formato NAO leva.\n\
             [sculpt3d]    2) Ctrl+Shift+O e escolha esse mesmo arquivo. As tres pecas voltam\n\
             [sculpt3d]       AO LADO das originais, na mesma disposicao e COM as cores.\n\
             [sculpt3d]       Se voltarem empilhadas na origem, a pose nao viajou.\n\
             [sculpt3d]    3) Repita com  volta.ply : as cores voltam, mas as tres viram UMA\n\
             [sculpt3d]       peca so' -- e o toast tinha avisado (pieces merged).\n\
             [sculpt3d]    4) Repita com  volta.stl : a forma volta e a COR nao (tudo branco).\n\
             [sculpt3d]       O toast tinha avisado. E a peca tem de continuar ESCULPIVEL:\n\
             [sculpt3d]       clique nela e passe o pincel -- se ela for de triangulos soltos,\n\
             [sculpt3d]       nada acontece.\n\
             [sculpt3d]    5) Salve como  volta.xyz : ele tem de RECUSAR com o nome, nunca\n\
             [sculpt3d]       escrever um OBJ disfarcado."
        );
        return vec![
            (a, ph2d_mesh::Pose::new([-2.8, 0.6, 0.0], 1.0)),
            (b, ph2d_mesh::Pose::new([2.6, -0.4, 0.0], 0.8)),
        ];
    }
    if document_scene() {
        // ⚠️ **Um CUBO e um OCTAEDRO, cada um com pose própria** — e a peça que
        // a cena abre é a esfera com CRISTAS. As três escolhas são o oráculo: o
        // que este smoke pergunta é *"o que eu salvei é o que eu abro?"*, e uma
        // esfera lisa reaberta é indistinguível de uma esfera lisa recém-nascida.
        // A pose entra pelo mesmo motivo — sem ela, "a lista voltou" e "a lista
        // voltou na ordem certa, no lugar certo" seriam a mesma imagem.
        return vec![
            (
                ph2d_mesh::shapes::cube(1.0),
                ph2d_mesh::Pose::new([-2.6, 0.4, 0.0], 1.1),
            ),
            (
                ph2d_mesh::shapes::octahedron(1.0),
                ph2d_mesh::Pose::new([2.4, -0.5, 0.0], 0.7),
            ),
        ];
    }
    if !objects_scene() {
        return Vec::new();
    }
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

/// `=10` — a cena da **PORTA DE SAÍDA**: exportar, e trazer de volta.
///
/// ⚠️ **O oráculo é o ROUND-TRIP, e ele mora DENTRO do app** — é por isso que
/// esta wave trouxe os leitores de STL e PLY junto com os escritores. Sem eles o
/// smoke dependeria de o artista abrir o Blender para julgar, e um smoke que
/// precisa de outro programa não é um smoke: é uma tarefa.
pub(crate) fn export_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("10")
}

/// `=5` — a cena do **TWIST e do LOCAL SCALE**: uma esfera com CRISTAS.
pub(crate) fn turn_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("5")
}

/// `=6` — a cena do **REMESH**: uma esfera com um bico ESTICADO até o barro
/// acabar.
pub(crate) fn remesh_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("6")
}

/// `=3` — a cena da **REVERSÃO**: um modelo denso que É uma subdivisão.
pub(crate) fn reversion_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("3")
}

/// `=4` — a cena de **FECHAR BURACO**: uma esfera com um pedaço arrancado.
pub(crate) fn holes_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("4")
}

/// A malha com que cada cena abre.
///
/// ⚠️ **Porta única, e ela existe para o gate.** A cena `=3` só significa alguma
/// coisa se a malha dela de fato reverter, e isso é um fato sobre a GEOMETRIA
/// que nenhum arch-gate de fonte enxerga. Um gate que reconstruísse a malha por
/// conta própria estaria medindo outra malha no dia em que esta mudasse.
#[must_use]
pub(crate) fn smoke_mesh() -> ph2d_mesh::Mesh {
    // ⚠️ A `=8` abre com as CRISTAS pelo motivo que o `scene_objects` explica:
    // uma esfera lisa reaberta é indistinguível de uma recém-nascida, e o smoke
    // do documento pergunta exatamente *o que eu salvei é o que eu abro?*.
    // ⚠️ A `=10` abre com as CRISTAS pelo mesmo motivo da `=8`: uma esfera lisa
    // que volta de um arquivo é indistinguível de uma recém-nascida, e o que
    // este smoke pergunta é *a FORMA atravessou?*.
    // ⚠️ A `=11` abre com as CRISTAS porque o que ela julga é a LUZ: sobre uma esfera lisa a
    // iluminação de uma normal quase constante lê como um degradê chapado, e o artista não teria
    // como separar *o objeto ficou aceso pela forma* de *alguém escureceu o sprite*.
    if turn_scene() || document_scene() || export_scene() || bake_scene() {
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

/// `=11` — a cena do **OBJETO MISTO** (`docs/3D/02.2`): a esfera com cristas E um sprite para
/// acender.
///
/// ⚠️ Cena própria, e não um passo da `=2`, pela mesma razão que separou a `=2` da `=1`: a doação
/// pergunta *a forma acende a TINTA que eu estou pintando?* e esta pergunta *o OBJETO fica aceso
/// depois que a malha sai?*. A segunda tem um passo que a primeira não tem — apagar a escultura — e
/// misturá-las faria o artista destruir a cena da doação para julgar esta.
pub(crate) fn bake_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("11")
}

/// **Esta cena quer uma TELA na mesa?** A pergunta é feita UMA vez, e as duas cenas que respondem
/// sim ([`donation_scene`] e [`bake_scene`]) precisam da mesma superfície branca pelo mesmo motivo:
/// a luz da forma é o que se vê, e sobre branco não há cor competindo.
pub(crate) fn wants_canvas() -> bool {
    donation_scene() || bake_scene()
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
    if document_scene() {
        // ⚠️ **A cena DECLARA quantas peças montou e com que forma.** Um smoke
        // de persistência sobre uma esfera LISA é indistinguível da feature
        // quebrada — reabrir e ver uma esfera não diz se ela VOLTOU ou se
        // NASCEU. Por isso a peça central tem cristas, e por isso o número aqui
        // é a contagem de peças: se ele não for 3, PARE.
        eprintln!(
            "[sculpt3d] =8 O DOCUMENTO: 3 pecas na mesa -- a esfera com CRISTAS no centro,\n             [sculpt3d]    um CUBO grande a` esquerda e um OCTAEDRO pequeno a` direita, cada um\n             [sculpt3d]    com pose propria. Se voce nao ver TRES formas diferentes, PARE.\n             [sculpt3d]    1) Esculpa: marque a esfera com um traco que voce reconheca depois.\n             [sculpt3d]    2) Aperte K (subdividir), esculpa um detalhe FINO, e aperte , (descer ao 0).\n             [sculpt3d]       -- e' esta a janela em que o detalhe fino so' existe no documento.\n             [sculpt3d]    3) Ctrl+S. FECHE o app. Abra de novo com a MESMA variavel e Ctrl+O.\n             [sculpt3d]    A escultura tem de voltar INTEIRA: as tres pecas, nas mesmas poses,\n             [sculpt3d]    com o seu traco -- e no nivel 0, que e' onde voce estava.\n             [sculpt3d]    4) Aperte . (subir): o detalhe fino do passo 2 tem de estar la'.\n             [sculpt3d]    Ctrl+Z depois de abrir NAO pode desfazer nada da sessao anterior."
        );
    }
    if objects_scene() {
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
    if bake_scene() {
        // ⚠️ **Os passos (4) e (5) são a wave**, e nenhum dos dois é sobre o momento do bake: o que
        // separa isto de um carimbo é o objeto continuar RESPONDENDO à luz depois, e continuar
        // aceso depois de a escultura sair. Um roteiro que parasse no (3) deixaria o artista
        // aprovar um efeito que qualquer filtro de imagem entrega.
        eprintln!(
            "[sculpt3d] =11 O OBJETO MISTO: ha' um SPRITE branco na mesa (ja' SELECIONADO), e a\n\
             [sculpt3d]    forma da esfera vai acender ELE.\n\
             [sculpt3d]    (1) A esfera chega com CRISTAS -- e' delas que a luz toma a forma.\n\
             [sculpt3d]        Gire (botao direito) ate' a vista que voce quer assar: o bake usa a\n\
             [sculpt3d]        camera do ESCULTOR, entao o que voce ve' e' o que ele grava.\n\
             [sculpt3d]    (2) Shift+B ASSA. O log diz o tamanho.\n\
             [sculpt3d]    (3) Aperte D UMA vez: o barro sai da tela e o SPRITE aparece. Ele tem de\n\
             [sculpt3d]        estar com o RELEVO DA ESFERA desenhado em luz e sombra. Se ele so'\n\
             [sculpt3d]        escureceu por igual, reprove.\n\
             [sculpt3d]        (com o barro na tela o sprite fica ATRAS dele -- por isso o D.)\n\
             [sculpt3d]    (4) O TESTE DA WAVE: Q/E giram a lampada, R/F a sobem. O sprite tem de\n\
             [sculpt3d]        RE-ACENDER a cada toque -- as sombras ANDAM. Se ele ficar congelado,\n\
             [sculpt3d]        isto e' um carimbo e nao um objeto, e a wave falhou.\n\
             [sculpt3d]    (5) O SEGUNDO TESTE: aperte D ate' voltar ao BARRO e Delete ate' a\n\
             [sculpt3d]        escultura sumir; volte ao D. O sprite tem de continuar aceso E as\n\
             [sculpt3d]        teclas de luz tem de continuar movendo as sombras dele -- sem malha.\n\
             [sculpt3d]    (6) Um objeto assado e' FOSCO de proposito (o barro ainda nao tem\n\
             [sculpt3d]        material): procure FORMA, nao brilho especular.\n\
             [sculpt3d]    (7) Assar DE NOVO por outro angulo tem de substituir a luz, nao somar --\n\
             [sculpt3d]        gire, Shift+B, e o sprite nao pode ficar mais escuro a cada bake.\n\
             [sculpt3d]    ⚠️ O bake NAO sobrevive a fechar o app: os canais no arquivo sao a\n\
             [sculpt3d]        proxima fatia, e a ausencia esta' nomeada no `sculpt3d.rs`."
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
