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

use super::fixtures::{
    eared_sphere, hooked_sphere, punctured_sphere, ridged_sphere, wrinkled_sphere,
};

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
    // AS CENAS DOS CANAIS DE SOMBREAMENTO — ver o módulo irmão.
    if let Some(v) = shading::scene_objects() {
        return v;
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
    if transform_scene() {
        // A da direita é a MASCARADA; a esquerda é a nua que o `smoke_mesh` abre
        // — a mesma divisão da `=22`, e pelo mesmo motivo.
        return vec![(
            soft_masked_sphere(),
            ph2d_mesh::Pose::new([2.6, 0.0, 0.0], 1.0),
        )];
    }
    if extract_scene() {
        // ⚠️ **A da direita é a MASCARADA**, e a esquerda é a que a cena já abre
        // (nua, pelo `smoke_mesh`). Sem a nua ao lado, o smoke provaria só que
        // o botão lê uma máscara que ELE não viu ser pintada.
        return vec![(masked_dome(), ph2d_mesh::Pose::new([2.6, 0.0, 0.0], 1.0))];
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

/// `=17` — a cena do **AO ASSADO**: um TORO, onde a oclusão é inequívoca.
///
/// ⚠️ **A forma é o oráculo, e é a mesma dos gates.** O aro INTERNO enxerga a
/// parede oposta através do furo e o EXTERNO enxerga o céu — não há como um bake
/// correto inverter isso, e não há como o artista confundir *"o AO chegou"* com
/// *"a luz mudou"*. Numa esfera lisa não haveria nada a ocluir, e o smoke não
/// conseguiria distinguir a feature de um slider inerte.
pub(crate) fn ao_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("17")
}

/// `=16` — a cena do **ALPHA**: uma esfera DENSA o bastante para o padrão ser
/// amostrado como padrão.
pub(crate) fn alpha_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("16")
}

/// `=21` — a cena do **EIXO**: a família DIRECIONAL do alpha.
///
/// ⚠️ **Cena própria e não um passo da `=16`**, pela mesma regra que separou a
/// `=20` da `=2`: a `=16` pergunta *o padrão sobrevive à lei do traço?* e o
/// oráculo dela é o padrão APARECER; esta pergunta *o eixo aponta o padrão?*, e
/// o oráculo é ele GIRAR. Um passo a mais na `=16` faria o artista julgar a
/// segunda coisa com a fixture da primeira.
///
/// ⚠️ **Ela abre com a MESMA malha densa da `=16`**, e não por comodidade: a lei
/// das dez arestas vale igual, e um estrato picado por uma malha grossa lê como
/// chuvisco — o artista veria o eixo girar um ruído.
///
/// ⚠️ **E ela NÃO arma o padrão.** O doc do `impasto_smoke` do Painter pregou o
/// preço disso em letra — *"a cena que arma estado por baixo do pano pula
/// justamente a costura que ela devia provar"* —, e aqui a costura É o gesto: os
/// chips novos existem, estão registrados, respondem ao mouse e levam a algum
/// lugar. O roteiro manda escolher; a cena entrega o barro.
pub(crate) fn directional_alpha_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("21")
}

/// **A CENA DO AGARRE ELÁSTICO** (`=28`) — irmã da [`masked`] pelo mesmo teto de
/// LOC e pela mesma linha de corte: cada arquivo é a história de uma wave.
#[path = "sculpt3d_scenes_elastic.rs"]
pub(crate) mod elastic;

/// **A CENA DA FAIXA** (`=29`) — irmã da [`elastic`] pela mesma linha de corte:
/// cada arquivo é a história de uma wave.
#[path = "sculpt3d_scenes_strip.rs"]
pub(crate) mod strip;

/// **A CENA DO FILTRO** (`=34`) — irmã da [`layer`] pela mesma linha de corte.
#[path = "sculpt3d_scenes_filter.rs"]
pub(crate) mod filter;

/// **A CENA DA RETOPOLOGIA** (`=35`) — irmã da [`filter`] pela mesma linha de
/// corte: cada arquivo é a história de uma wave.
#[path = "sculpt3d_scenes_quad.rs"]
pub(crate) mod quad;

/// **A CENA DA ORELHA** (`=36`) — a retopologia sobre um vinco CÔNCAVO fundo, que
/// é a feição que nenhuma outra fixtura tinha. Irmã da [`quad`].
#[path = "sculpt3d_scenes_ear.rs"]
pub(crate) mod ear;

/// **A CENA DO FILTRO DE TECIDO** (`=37`) — irmã da [`thumb`]; ⚠️ **não** é parte
/// da [`filter`], e o cabeçalho dela diz porquê.
#[path = "sculpt3d_scenes_cloth_filter.rs"]
pub(crate) mod cloth_filter;

/// **A CENA DA DEMÃO** (`=33`) — irmã da [`surface`] pela mesma linha de corte.
#[path = "sculpt3d_scenes_layer.rs"]
pub(crate) mod layer;
/// `=22` — a cena do **EXTRACT**: a máscara vira uma PEÇA.
///
/// ⚠️ **Duas esferas, e a assimetria é o smoke inteiro.** A da DIREITA já vem
/// mascarada: o artista aperta o botão e julga a GEOMETRIA — a espessura, a
/// costura, o lado para onde a casca cresce — sem o confundidor de ter acabado
/// de pintar uma máscara à mão. A da ESQUERDA vem NUA: ele pinta a máscara dele
/// e extrai de novo, e é essa metade que prova a costura *pincel → botão*.
///
/// ⚠️ **Só a da direita é armada por baixo do pano**, e a lei que o doc do
/// `impasto_smoke` do Painter pregou continua honrada — *a cena que arma estado
/// pula justamente a costura que ela devia provar*. Ela não a pula: ela a põe na
/// peça ao lado.
#[path = "sculpt3d_scenes_masked.rs"]
pub(crate) mod masked;
/// **A CENA DA LÂMINA EM V** (`=31`) — irmã da [`thumb`] pela mesma linha de corte.
#[path = "sculpt3d_scenes_scrape.rs"]
pub(crate) mod scrape;
/// **A CENA DA SUPERFÍCIE LOCAL** (`=32`) — irmã da [`scrape`] pela mesma linha
/// de corte.
#[path = "sculpt3d_scenes_surface.rs"]
pub(crate) mod surface;
/// **A CENA DO POLEGAR** (`=30`) — irmã da [`strip`] pela mesma linha de corte.
#[path = "sculpt3d_scenes_thumb.rs"]
pub(crate) mod thumb;
/// **A CENA DOS QUATRO VIEWPORTS** (`=38`) — irmã da [`cloth_filter`] pela mesma
/// linha de corte: cada arquivo é a história de uma wave.
#[path = "sculpt3d_scenes_viewports.rs"]
pub(crate) mod viewports;
pub(crate) use masked::{
    flatten_scene, flatten_scene_counts, mask_channel_numbers, mask_channel_scene,
    masked_dome_counts, soft_masked_counts, transform_scene,
};
use masked::{masked_dome, soft_masked_sphere};

pub(crate) fn extract_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("22")
}

/// **COM QUE MALHA CADA CENA ABRE** — ver [`mesh`].
#[path = "sculpt3d_scenes_mesh.rs"]
mod mesh;
pub(crate) use mesh::smoke_mesh;

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

/// `=25` — a cena do **ALPHA POR IMAGEM**: um sprite na mesa e uma peça para
/// carimbar com ele.
///
/// ⚠️ **Ela precisa de um SPRITE, e é por isso que reusa a encenação da tela** —
/// o gesto inteiro é *selecione um sprite e aperte o botão*, e uma cena sem
/// sprite mostraria um painel sem o botão, que é o oposto do que ela existe para
/// provar. A peça é a esfera SULCADA das irmãs: um padrão sobre uma bola lisa é
/// difícil de separar de *"a peça ficou texturizada"*.
pub(crate) fn alpha_image_scene() -> bool {
    std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some("25")
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
    donation_scene()
        || bake_scene()
        || reopen_scene()
        || alpha_image_scene()
        || shading::occlusion_donation_scene()
}
/// **O roteiro de cada cena** — módulo filho, separado por ASSUNTO (e pelo teto de LOC).
/// AS CENAS DOS CANAIS DE SOMBREAMENTO — ver o módulo.
#[path = "sculpt3d_scenes_shading.rs"]
pub(crate) mod shading;

#[path = "sculpt3d_scripts.rs"]
pub(super) mod scripts;

/// **OS GATES DAS CENAS** — ver o irmão.
///
/// ⚠️ **FILHO e não irmão** (`#[cfg(test)] #[path]`), e a diferença é
/// load-bearing: dois deles montam a fixture com `hooked_sphere` e
/// `ridged_sphere`, que são **privadas** deste módulo, e um `use super::*` de um
/// módulo FILHO as alcança sem que nada precise ficar mais público só para o
/// teste. É o precedente exato do `physics_overlay_tests.rs` e do
/// `undo_delta_tests.rs`.
#[cfg(test)]
#[path = "sculpt3d_scenes_tests.rs"]
mod tests;
