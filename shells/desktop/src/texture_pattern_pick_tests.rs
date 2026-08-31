//! Os gates do NASCIMENTO de um padrão (plano 33; report do Enio de 2026-08-27).

use super::*;
use ph2d_asset::AssetDb;
use ph2d_vec_scene::{VecPath, VecVertex};

/// ⛔⛔ **UM PADRÃO NASCE SOBRE A FORMA, NÃO NA ORIGEM DO MUNDO.**
///
/// Com `Tile`/`Mirror` a diferença é invisível (o padrão repete-se por toda a parte); com `Clamp` é
/// catastrófica: o `Extend::Pad` devolve a **borda** da arte esticada, e o artista vê um borrão
/// chapado. Medido na cena de smoke antes da cura: as seis formas caíam em `uv.x` de **−331 a +331**
/// com o ladrilho a cobrir `0..32`.
///
/// ⚠️ É a metade que faltava da lei que o plano se gabava de honrar: eu evitei a ancoragem à origem
/// da régua do Illustrator na metade do TRANSFORM (o padrão anda com a forma) e reproduzi-a na
/// metade do NASCIMENTO.
#[test]
fn a_new_pattern_is_born_over_the_shape_not_at_the_world_origin() {
    let db = AssetDb::new();
    let art = db.insert_image_rgba8(4, 2, vec![9u8; 4 * 2 * 4]);
    let source = PatternSource::Image(art);

    let mut scene = VecScene::default();
    // Uma forma LONGE da origem — é aí que o defeito se vê.
    let id = scene.push_path(VecPath {
        verts: [[100.0, 50.0], [106.0, 50.0], [106.0, 56.0], [100.0, 56.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    // ⚠️ A arte mede-se pela porta REAL (`art_dims`), não por um `[4, 2]` escrito à mão: assim este
    // gate também prova que ela continua a saber medir uma IMAGEM depois de aprender a medir uma
    // FORMA. Os mapas vazios são legítimos — a rota da imagem não os lê.
    let arte = art_dims(
        &db,
        &scene,
        &ph2d_vec_scene::VecXforms::default(),
        &ph2d_vec_render::LiveGeometry::default(),
        id,
        &source,
        &|id| vec![id],
    );
    assert_eq!(
        arte,
        Some([4, 2]),
        "a porta deixou de saber medir uma imagem"
    );
    let (size, origin) = default_placement(&scene, id, arte);
    assert_eq!(origin, [100.0, 50.0], "o padrao nasceu na origem do MUNDO");
    // E o tamanho continua a preservar o aspecto 2:1 da arte.
    assert!(
        (size[0] / size[1] - 2.0).abs() < 1e-9,
        "o aspecto 2:1 da arte nao sobreviveu: {size:?}"
    );
    // ⚠️ Controlo: uma forma NOUTRO sítio nasce noutro canto — senão este gate estaria a medir uma
    // constante.
    let id2 = scene.push_path(VecPath {
        verts: [[-7.0, -3.0], [-5.0, -3.0], [-5.0, -1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let (_, o2) = default_placement(&scene, id2, arte);
    assert_eq!(o2, [-7.0, -3.0]);
}

/// ⭐⭐⭐ **O CHIP *Pattern* NÃO ESCOLHE A ARTE PELO ARTISTA — NAS DUAS TINTAS** (report do Enio,
/// 2026-08-30: *"ao apertar pattern o usuário é obrigado a selecionar uma img no dialog"*, e depois
/// *"e para Stroke?"*).
///
/// # A régua, e porque ela não é o valor devolvido
///
/// O defeito era um **efeito colateral**: a porta abria um diálogo de ficheiro. Um gate que só
/// olhasse o valor devolvido ficaria verde com o diálogo ainda lá — e num arnês sem ecrã o diálogo
/// **bloqueia ou devolve `None`**, o que se leria como "a função não fez nada".
///
/// ⇒ o que se afirma é a coisa que só é verdade **sem** diálogo: a porta é PURA. Ela deixou de
/// receber o `AssetDb` (não há o que descodificar nem inserir), e sem ele **não há forma de ela
/// produzir uma `PatternSource::Image`** — o tipo diz o que o comentário prometia. Este gate corre
/// num teste normal, o que por si só prova que ela não abre nada: um `rfd::FileDialog` num arnês
/// headless não voltaria daqui.
///
/// ⚠️⚠️ **O SLOT é a metade que a 1.ª redacção não tinha, e foi por isso que o traço ficou de fora
/// da 1.ª cura.** Ele nem chamava esta porta: ia direto ao `pick_source`. *Um gate escrito de uma
/// tinta deixa a outra sem cobertura, e as duas têm chips idênticos na mesma janela.*
///
/// ⚠️ **E a segunda metade importa tanto como a primeira:** o chip não deita fora a arte que a
/// tinta já tem. Sem ela, a cura seria "o chip apaga o que estava lá".
#[test]
fn choosing_pattern_on_a_bare_shape_does_not_pick_the_art_for_the_artist() {
    use ph2d_vec_render::PatternSlot;
    let arte = |src| {
        ph2d_vec_scene::PatternFill::new(src, [2.0, 2.0], ph2d_vec_scene::Rgba8::new(1, 2, 3, 255))
    };
    for slot in [PatternSlot::Fill, PatternSlot::Stroke] {
        let mut scene = VecScene::default();
        let verts = || {
            [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0]]
                .map(VecVertex::corner)
                .to_vec()
        };
        // ⚠️ A forma NUA tem contorno: sem ele o `set_kind` do traço recusa por desenho, e a metade
        // do traço mediria uma forma que a ferramenta nunca produz.
        let nua = scene.push_path(VecPath {
            verts: verts(),
            closed: true,
            stroke: Some(ph2d_vec_scene::StrokeSpec::new(
                ph2d_vec_scene::Rgba8::new(9, 9, 9, 255),
                1.0,
            )),
            ..VecPath::default()
        });
        assert_eq!(
            source_for(&scene, nua, slot),
            Some(PatternSource::None),
            "{slot:?}: o chip Pattern numa tinta sem padrao tem de nascer SEM arte escolhida - se \
             ele escolher, escolhe sempre a mesma, e a outra arte fica atras dela"
        );
        // ⚠️ CONTROLO: numa tinta que JÁ tem padrão, a porta devolve a arte que lá está.
        let mut vestida = VecPath {
            verts: verts(),
            closed: true,
            ..VecPath::default()
        };
        match slot {
            PatternSlot::Fill => {
                vestida.fill = Some(ph2d_vec_scene::Paint::Pattern(Box::new(arte(
                    PatternSource::Shape(nua),
                ))));
            }
            PatternSlot::Stroke => {
                let mut s =
                    ph2d_vec_scene::StrokeSpec::new(ph2d_vec_scene::Rgba8::new(9, 9, 9, 255), 1.0);
                s.paint =
                    ph2d_vec_scene::StrokePaint::Pattern(Box::new(arte(PatternSource::Shape(nua))));
                vestida.stroke = Some(s);
            }
        }
        let vestida = scene.push_path(vestida);
        assert_eq!(
            source_for(&scene, vestida, slot),
            Some(PatternSource::Shape(nua)),
            "{slot:?}: o chip deitou fora a arte que a tinta ja' tinha"
        );
        // ⭐ E as tintas são INDEPENDENTES: a que não foi vestida continua sem arte escolhida.
        let outra = match slot {
            PatternSlot::Fill => PatternSlot::Stroke,
            PatternSlot::Stroke => PatternSlot::Fill,
        };
        assert_eq!(
            source_for(&scene, vestida, outra),
            Some(PatternSource::None),
            "{slot:?}: vestir uma tinta respondeu pela OUTRA - a porta esta' a ler o slot errado"
        );
    }
}

/// ⭐⭐⭐ **UM GRUPO ALTO NÃO NASCE ACHATADO** (report do Enio, 2026-08-30).
///
/// # O defeito, e porque ele sobreviveu a tudo o resto desta linha
///
/// A porta que mede a arte **só sabia medir imagens**: uma fonte-FORMA caía no `unwrap_or([1, 1])`
/// do [`default_placement`], que é um **quadrado**. ⇒ toda estampa vestida por uma forma ou por um
/// grupo nascia com o aspecto errado, e nenhum gate o via porque todos usavam arte de imagem.
///
/// ⚠️ **E a nota que o adiava estava errada no ponto que decidia o preço:** ela dizia que as
/// dimensões de um grupo *"exigem o assado de GPU"*. A caixa é `path_screen_bounds` — **geometria em
/// CPU** —, e o assado já a calculava e deitava fora.
///
/// # A régua é a RAZÃO, não os pixels
///
/// Os pixels dependem do DPI do assado, que é detalhe interno; o que o artista vê é a **proporção**.
/// A fixtura é um grupo `1 x 3` (dois membros empilhados), e um quadrado seria `1,00`.
///
/// ⚠️ **CONTROLO da fixtura**: os dois membros ficam em sítios diferentes, senão a união seria a
/// caixa de um deles e o gate mediria um caminho só.
#[test]
fn the_art_of_a_group_is_measured_and_a_tall_group_is_not_born_square() {
    let db = AssetDb::new();
    let mut scene = VecScene::default();
    let caixa = |x0: f64, y0: f64, x1: f64, y1: f64| VecPath {
        verts: [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    // Um grupo de DOIS membros empilhados: 1 de largura, 3 de altura ao todo.
    let a = scene.push_path(caixa(0.0, 0.0, 1.0, 1.0));
    let b = scene.push_path(caixa(0.0, 2.0, 1.0, 3.0));
    // A forma que vai VESTIR o padrão — criada antes de a cena ser emprestada à sonda.
    let alvo_id = scene.push_path(caixa(10.0, 10.0, 20.0, 20.0));
    let grupo = move |id: ph2d_vec_scene::VecPathId| {
        if id == a || id == b {
            vec![a, b]
        } else {
            vec![id]
        }
    };
    let medir =
        |src: &PatternSource,
         obj: &dyn Fn(ph2d_vec_scene::VecPathId) -> Vec<ph2d_vec_scene::VecPathId>| {
            art_dims(
                &db,
                &scene,
                &ph2d_vec_scene::VecXforms::default(),
                &ph2d_vec_render::LiveGeometry::default(),
                alvo_id,
                src,
                obj,
            )
        };

    let d = medir(&PatternSource::Shape(a), &grupo).expect("a porta nao mediu a arte-FORMA");
    let razao = f64::from(d[1]) / f64::from(d[0]);
    assert!(
        (razao - 3.0).abs() < 0.05,
        "o grupo 1x3 mediu {d:?} (razao {razao:.3}) - um quadrado da' 1,00, e e' isso que o padrao \
         herdava"
    );
    // ⚠️ CONTROLO: UM membro sozinho é 1:1 — sem isto, uma porta que devolvesse sempre 1:3 passaria.
    let solo = medir(&PatternSource::Shape(a), &|id| vec![id]).expect("mediu um membro so'");
    assert!(
        (f64::from(solo[1]) / f64::from(solo[0]) - 1.0).abs() < 0.05,
        "o membro sozinho nao e' quadrado: {solo:?} - a fixtura nao contem o fenomeno"
    );
    // ⭐ E a colocação HERDA a razão medida: é isso que o artista vê.
    let alvo = alvo_id;
    let (size, _) = default_placement(&scene, alvo, Some(d));
    assert!(
        (size[1] / size[0] - razao).abs() < 1e-9,
        "a colocacao deitou fora o aspecto medido: {size:?}"
    );
    // ⛔ E sem arte escolhida ainda não há o que medir — o quadrado é o marcador, e é legítimo.
    assert_eq!(medir(&PatternSource::None, &grupo), None);
}

/// ⛔⛔ **MEDIR UMA ARTE QUE O DESENHO VAI RECUSAR GRAVA UM TAMANHO ERRADO *PARA SEMPRE*** —
/// achado da auditoria desta wave (2026-08-30).
///
/// # A cadeia, e porque o dano não é só uma medida errada
///
/// O guarda do canvas só barra `guide == host`; clicar um **irmão do mesmo grupo** passa por ele.
/// A partir daí: a medição via o grupo INTEIRO (anfitrião incluído) → o `set_source` adoptava esse
/// tamanho **e consumia** a lei *"adopta só quando não havia arte"* → o assado recusava o ciclo e a
/// forma passava a pintar a `fallback` → e a arte SEGUINTE, já válida, **não** voltava a re-derivar,
/// porque a fonte já não era `None`.
///
/// ⇒ *Uma recusa que um dos leitores não vê é uma recusa que queima a decisão do outro.*
///
/// ⚠️ **A régua é a PORTA PARTILHADA**, não um `if` escrito aqui: a medição e o assado perguntam
/// `art_members`, e por isso não podem discordar.
#[test]
fn measuring_the_art_obeys_the_same_cycle_refusal_as_the_bake() {
    let db = AssetDb::new();
    let mut scene = VecScene::default();
    let caixa = |x0: f64, y0: f64, x1: f64, y1: f64| VecPath {
        verts: [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    let anfitriao = scene.push_path(caixa(0.0, 0.0, 1.0, 1.0));
    let irmao = scene.push_path(caixa(0.0, 2.0, 1.0, 3.0));
    // Criada ANTES de a cena ser emprestada à sonda: uma forma de FORA do grupo, para o controlo.
    let de_fora = scene.push_path(caixa(10.0, 10.0, 11.0, 11.0));
    // O grupo contém o ANFITRIÃO — é o ciclo que o guarda do canvas não apanha.
    let grupo = move |id: ph2d_vec_scene::VecPathId| {
        if id == anfitriao || id == irmao {
            vec![anfitriao, irmao]
        } else {
            vec![id]
        }
    };
    let medir = |host, art| {
        art_dims(
            &db,
            &scene,
            &ph2d_vec_scene::VecXforms::default(),
            &ph2d_vec_render::LiveGeometry::default(),
            host,
            &PatternSource::Shape(art),
            &grupo,
        )
    };
    assert_eq!(
        medir(anfitriao, irmao),
        None,
        "a medicao aceitou uma arte que a contem - o tamanho errado ficaria gravado para sempre, \
         porque a arte seguinte ja' nao encontra a fonte em `None`"
    );
    // ⚠️ CONTROLO: uma forma de FORA do grupo mede-se normalmente — senão este gate estaria a
    // afirmar que a porta recusa tudo.
    assert!(
        medir(de_fora, irmao).is_some(),
        "a porta recusou uma arte legitima - o controlo nao passa"
    );
}

/// ⛔⛔⛔ **O TECTO NÃO PODE ACHATAR A ARTE** — achado da auditoria desta wave (2026-08-30), e é o
/// report original a reaparecer por outra porta.
///
/// # O que estava lá, com os números
///
/// O `MAX_TILE_SIDE` era aplicado **em cada eixo, independentemente**, e o afim do assado era uma
/// translação pura. ⇒ acima do tecto o artista via **um canto da arte**, sem mensagem nenhuma — e,
/// pior, os dois eixos saturavam no mesmo número. Medido num grupo de razão geométrica `3,000`:
///
/// | grupo (mundo) | razão medida |
/// |---|---|
/// | `1 x 3` · `2 x 6` | `3,000` |
/// | `4 x 12` | `2,000` |
/// | **`8 x 24`** e acima | **`1,000` ← o quadrado do report** |
///
/// ⚠️ E chega-se lá depressa: a caixa de um traço é inflada por `2 x width` por lado
/// (`miter_limit = 4`), então uma caixa `1 x 1` com traço de largura `1` já mede **5** das 8
/// unidades.
///
/// ⛔ E o doc do [`MAX_TILE_SIDE`] prometia o contrário — *"clamped to it (a coarser effective
/// DPI)"*. **Não havia DPI nenhum mais grosso.** *Uma afirmação que descreve o que se queria, e não
/// o que o código faz, impede a próxima pessoa de olhar.*
///
/// ⇒ hoje a escala é **uniforme**, tirada do lado maior: o aspecto sobrevive por construção, e o
/// que se perde é resolução — que é o que a nota sempre prometeu.
#[test]
fn the_tile_ceiling_costs_resolution_never_the_aspect() {
    let db = AssetDb::new();
    let mut scene = VecScene::default();
    let alto = |k: f64| VecPath {
        verts: [[0.0, 0.0], [k, 0.0], [k, 3.0 * k], [0.0, 3.0 * k]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    // ⚠️ O anfitrião é uma forma REAL e de fora — um `VecPathId::default()` pode colidir com o
    // primeiro id da cena e disparar a recusa de ciclo, e o gate leria isso como "não mediu".
    let anfitriao = scene.push_path(alto(0.5));
    // Uma escada que ATRAVESSA o tecto: a `k = 32` a caixa mede 32 x 96 unidades de mundo.
    let mut razoes = Vec::new();
    for k in [1.0, 4.0, 8.0, 32.0] {
        let id = scene.push_path(alto(k));
        let d = art_dims(
            &db,
            &scene,
            &ph2d_vec_scene::VecXforms::default(),
            &ph2d_vec_render::LiveGeometry::default(),
            anfitriao,
            &PatternSource::Shape(id),
            &|i| vec![i],
        )
        .expect("mediu");
        razoes.push((k, d, f64::from(d[1]) / f64::from(d[0])));
    }
    for (k, d, r) in &razoes {
        assert!(
            (r - 3.0).abs() < 0.02,
            "a k={k} o grupo 3:1 mediu {d:?} (razao {r:.3}) - o tecto achatou a arte, e a `1,000` \
             isso e' literalmente o quadrado do report de 30/08 a voltar"
        );
    }
    // ⚠️ CONTROLO: a fixtura ATRAVESSA mesmo o tecto — senão este gate mede quatro casos abaixo
    // dele e não afirma nada sobre o tecto. O lado maior tem de saturar.
    let maior = razoes.last().expect("ha' casos").1;
    assert_eq!(
        maior[1],
        crate::motion_object_bake::MAX_TILE_SIDE,
        "a fixtura nao chega ao tecto ({maior:?}) - este gate estaria a medir o caso comum"
    );
    // E o caso pequeno NÃO satura, senão a escada não é uma escada.
    assert!(razoes[0].1[1] < crate::motion_object_bake::MAX_TILE_SIDE);
}
