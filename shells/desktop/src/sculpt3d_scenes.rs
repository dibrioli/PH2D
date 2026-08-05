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

use super::fixtures::{hooked_sphere, punctured_sphere, ridged_sphere, wrinkled_sphere};

/// A cena está armada? — **qualquer nível ≥ 1**.
///
/// ⚠️ **Aqui havia uma ENUMERAÇÃO (`"1" | "2" | … | "13"`), e ela apodreceu no
/// dia previsível:** a cena `=14` nasceu com predicado próprio, script próprio e
/// malha própria, e o app abriu com **o canvas em branco** — o módulo inteiro
/// nunca armou, porque ninguém acrescentou o `"14"` a esta lista. Nenhum gate
/// via: cada peça da cena existia e estava certa.
///
/// A pergunta certa não é *"este número está na lista?"* e sim *"o artista pediu
/// uma cena?"*. Um nível que não existe passa a abrir a esfera padrão — uma
/// degradação visível e honesta, contra uma tela preta que se lê como crash. E a
/// lista de cenas pode crescer para sempre sem ninguém ter de lembrar deste
/// arquivo.
pub(crate) fn smoke_armed() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
        .is_some_and(|n| n >= 1)
}

/// `=13` — a cena da **FUSÃO e do ISOLAMENTO**: quatro peças de formas
/// DIFERENTES.
///
/// ⚠️ **As formas têm de ser distinguíveis, e isso é o oráculo de três coisas de
/// uma vez.** A fusão não muda a silhueta da cena (as peças ficam onde estavam),
/// o isolamento tira peças da tela, e o slot do device pode passar a descrever
/// **outra** peça — com quatro esferas iguais os três acertos e os três erros
/// desenham a mesma imagem.
pub(crate) fn fuse_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("13")
}

/// `=14` — a cena da **TOPOLOGIA DINÂMICA**.
pub(crate) fn dyntopo_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("14")
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
    if fuse_scene() {
        // ⚠️ **TRÊS formas diferentes ao lado da esfera que a cena abre**, e a
        // razão é o gate do olho: o defeito que esta wave curou desenhava uma
        // peça com a geometria de OUTRA, e um quarteto de esferas iguais o
        // esconde por completo. Um cubo, um octaedro e um cubo pequeno são
        // distinguíveis a qualquer distância e em qualquer ângulo.
        return vec![
            (
                ph2d_mesh::shapes::cube(1.0),
                ph2d_mesh::Pose::new([-2.8, 0.0, 0.0], 1.2),
            ),
            (
                ph2d_mesh::shapes::octahedron(1.0),
                ph2d_mesh::Pose::new([2.6, 0.2, 0.0], 0.9),
            ),
            (
                ph2d_mesh::shapes::cube(1.0),
                ph2d_mesh::Pose::new([0.2, 2.4, 0.0], 0.5),
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

/// `=15` — a cena da **CAVIDADE**: uma esfera com rugas EM ESCADA.
pub(crate) fn cavity_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("15")
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
    // ⚠️ A `=15` abre com as RUGAS EM ESCADA, e a escada é o oráculo: a cavidade
    // entrega *ver o que a luz sozinha não mostra*, então a cena tem de conter
    // sulcos que a luz já mostra e sulcos que ela quase não mostra. Com uma
    // profundidade só, ligar o canal daria *uma imagem diferente* — e diferente
    // não é a pergunta.
    if cavity_scene() {
        return wrinkled_sphere();
    }
    if turn_scene() || document_scene() || export_scene() || bake_scene() || reopen_scene() {
        return ridged_sphere();
    }
    if remesh_scene() {
        return hooked_sphere();
    }
    if holes_scene() {
        return punctured_sphere();
    }
    if dyntopo_scene() {
        // ⚠️ **GROSSA de propósito, e é a metade do smoke que o número prova.**
        // A esfera de 96×144 que o resto do módulo abre já tem arestas menores
        // que o alvo de qualquer pincel razoável — ligar a topologia dinâmica
        // sobre ela não partiria nada, e a cena ficaria verde mostrando NADA.
        // Com 10×14 as facetas são visíveis a olho nu, e é contra elas que o
        // detalhe nascendo se vê.
        return ph2d_mesh::shapes::uv_sphere(10, 14, 1.0);
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

/// `=12` — a cena do **OBJETO ASSADO QUE VOLTA** (`docs/3D/02.2`, rota A): a mesma mesa da `=11`,
/// e um passo que só um ARQUIVO responde.
///
/// ⚠️ Cena própria pela regra que já separou a `=11` da `=2`: o passo desta é **fechar o app**, e
/// ele é destrutivo para a anterior — quem estivesse no meio do roteiro da `=11` perderia a sessão
/// para julgar esta. E a pergunta é outra: lá é *o objeto sobrevive à MALHA?*, aqui é *o objeto
/// sobrevive ao PROCESSO?*.
pub(crate) fn reopen_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("12")
}

/// **Esta cena quer uma TELA na mesa?** A pergunta é feita UMA vez, e as duas cenas que respondem
/// sim ([`donation_scene`] e [`bake_scene`]) precisam da mesma superfície branca pelo mesmo motivo:
/// a luz da forma é o que se vê, e sobre branco não há cor competindo.
pub(crate) fn wants_canvas() -> bool {
    donation_scene() || bake_scene() || reopen_scene()
}
/// **O roteiro de cada cena** — módulo filho, separado por ASSUNTO (e pelo teto de LOC).
#[path = "sculpt3d_scripts.rs"]
mod scripts;

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
         [sculpt3d] o espelho nasce DESLIGADO; PH2D_SCULPT3D_DIAG=1 mede se o pincel cai sob o cursor\n\
         [sculpt3d] --- O PAINEL (W12) ---\n\
         [sculpt3d] ele abre com a cena, e a CRASE (`) o fecha e o reabre\n\
         [sculpt3d] TOOL (os 16 verbos) · BRUSH (raio, forca, falloff, mascara) · SYMMETRY\n\
         [sculpt3d] TOPOLOGY (dyntopo, detalhe, niveis, remesh, tapar) · SHADING · SCENE\n\
         [sculpt3d] a CAVIDADE e' o slider da secao SHADING: 0 e' o barro liso, 1 o teto\n\
         [sculpt3d] MATERIAL (SHADING): 'Rig' e' a luz do DOCUMENTO; os outros seis sao MATCAPS --\n\
         [sculpt3d]     luz do OLHO, que nao gira com o modelo. Sob um matcap as duas pistas de\n\
         [sculpt3d]     lampada SOMEM, porque ele nao le o rig -- e isso e' o certo, nao um bug\n\
         [sculpt3d] ACCUMULATE (BRUSH): desarmado, cruzar o proprio traco NAO intensifica --\n\
         [sculpt3d]     e' a lei do envelope, e uma pincelada deposita no maximo a forca do\n\
         [sculpt3d]     pincel. Armado, passar duas vezes soma duas vezes. Ele so' aparece nos\n\
         [sculpt3d]     verbos de CARIMBO: quem tem ancora (G/H/T/S) carrega o gesto TOTAL\n\
         [sculpt3d]     desde o pen-down, e somar totais nao significa nada\n\
         [sculpt3d]     ATENCAO: a PRIMEIRA passada acumulada e' mais FRACA (a lei entrega a\n\
         [sculpt3d]     media do falloff, nao o pico); e' da segunda em diante que ela paga\n\
         [sculpt3d] WIREFRAME (SHADING): a malha por cima da forma -- e' o que mostra onde o remesh\n\
         [sculpt3d]     pos os aneis e ate' onde o refino chegou; ela some e volta sem custo com\n\
         [sculpt3d]     a caixa desmarcada (a lista de arestas so' existe com ela armada)\n\
         [sculpt3d] o ANEL do cursor e' desenhado NO PONTO DE ACERTO -- se ele nao estiver\n\
         [sculpt3d]     debaixo do mouse sobre o barro, o pick esta' errado e da' para VER",
        mesh.vert_count(),
        mesh.face_count(),
        mesh.triangle_count()
    );
    scripts::for_scene(mesh);
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
