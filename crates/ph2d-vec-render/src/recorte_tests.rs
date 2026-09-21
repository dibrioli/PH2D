//! ⭐⭐⭐ **OS GATES DO RECORTE POR CÂMARA** — ordem do dono, 2026-09-21 (*«pode implementar a
//! cura»*), depois de ele medir que `72 900` estrelas arredondadas também não cabem num quadro.
//!
//! ⛔⛔ **A economia é INVISÍVEL a toda régua de imagem, por construção:** uma cópia recortada é
//! uma que não aparecia na tela de qualquer maneira, logo desenhar e não desenhar dão o MESMO
//! pixel. *Uma cura que não muda o que se vê só pode ser gateada pela CONTA* — e é por isso que
//! metade destes gates lê o contador e não o desenho.
//!
//! ⚠️ **A outra metade é o lado caro do erro.** Um recorte que come de menos é gratuito; um que
//! come de mais é um BURACO na tela, e é esse que os gates da borda e do traço gordo defendem.

use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecVertex};
use ph2d_vector::{Affine, Rect, VectorScene};

/// Um quadrado de lado `1` centrado na origem, preenchido.
fn quadrado() -> VecPath {
    VecPath {
        verts: vec![
            VecVertex::corner([-0.5, -0.5]),
            VecVertex::corner([0.5, -0.5]),
            VecVertex::corner([0.5, 0.5]),
            VecVertex::corner([-0.5, 0.5]),
        ],
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(200, 60, 60, 255))),
        ..VecPath::default()
    }
}

/// A janela de teste: `[0, 100] × [0, 100]`.
fn janela() -> Rect {
    Rect::new(0.0, 0.0, 100.0, 100.0)
}

/// Desenha `poses` cópias do `path` e devolve `(objectos de desenho, cópias recortadas)`.
fn desenha(path: &VecPath, poses: &[Affine], janela: Option<Rect>) -> (usize, u32) {
    let _ = crate::encode_cost_tests::take_recortadas();
    let mut cena = VectorScene::new();
    let itens: Vec<(u32, Affine, [f32; 4])> = poses.iter().map(|p| (1u32, *p, [1.0; 4])).collect();
    crate::draw_shared_instances(itens, |_| Some(path), janela, &mut cena);
    let objectos = cena.inner().encoding().draw_tags.len();
    (objectos, crate::encode_cost_tests::take_recortadas())
}

/// ⭐⭐⭐ **O QUE ESTÁ FORA NÃO É DESENHADO — e o que está dentro é.**
///
/// ⚠️ **As duas metades são obrigatórias.** Sem a segunda, um recorte que deitasse fora TUDO
/// passaria neste gate — e o defeito seria uma tela vazia, que é o pior modo de falha desta cura.
#[test]
fn o_que_a_camara_nao_ve_nao_vai_a_placa() {
    let p = quadrado();
    // Cinco dentro (espalhadas pela janela) e cinco muito longe.
    let mut poses: Vec<Affine> = (0..5)
        .map(|i| Affine::translate((10.0 + f64::from(i) * 15.0, 50.0)) * Affine::scale(10.0))
        .collect();
    poses
        .extend((0..5).map(|i| {
            Affine::translate((1000.0 + f64::from(i) * 15.0, 50.0)) * Affine::scale(10.0)
        }));

    let (com, recortadas) = desenha(&p, &poses, Some(janela()));
    assert_eq!(recortadas, 5, "as cinco de fora têm de ser recortadas");
    assert_eq!(com, 5, "e só as cinco de dentro chegam ao encode");

    // ⭐ O CONTROLO: sem janela, as dez desenham-se e nenhuma é recortada.
    let (sem, recortadas_sem) = desenha(&p, &poses, None);
    assert_eq!(
        (sem, recortadas_sem),
        (10, 0),
        "controlo: sem janela o lote é o de sempre"
    );
}

/// ⭐⭐⭐ **A FORMA QUE ENCOSTA NA BORDA FICA** — o lado caro do erro.
///
/// ⚠️ Varre a forma ATRAVÉS da borda esquerda, meio pixel de cada vez: enquanto qualquer parte
/// dela tocar a janela, ela tem de ser desenhada. *Um recorte que coma a forma meia-dentro deixa
/// um buraco exactamente onde o artista está a olhar.*
#[test]
fn a_forma_que_encosta_na_borda_nao_e_comida() {
    let p = quadrado();
    let lado = 10.0; // a escala ⇒ a forma mede 10 × 10
    // De `x = −lado` (encostada por fora) a `x = 0` (metade dentro).
    let mut vistas = 0;
    for passo in 0..=20 {
        let x = -lado / 2.0 + f64::from(passo) * (lado / 20.0);
        let pose = Affine::translate((x, 50.0)) * Affine::scale(lado);
        let (objectos, _) = desenha(&p, &[pose], Some(janela()));
        assert_eq!(
            objectos, 1,
            "a {x:.2} a forma ainda toca a janela e tem de ser desenhada"
        );
        vistas += 1;
    }
    assert_eq!(vistas, 21, "controlo: a varredura tem de ter corrido");

    // ⭐ E o CONTROLO do outro lado: bem longe, ela é recortada.
    let longe = Affine::translate((-500.0, 50.0)) * Affine::scale(lado);
    let (objectos, recortadas) = desenha(&p, &[longe], Some(janela()));
    assert_eq!(
        (objectos, recortadas),
        (0, 1),
        "controlo: longe da janela ela sai"
    );
}

/// ⭐⭐ **O TRAÇO GORDO SEGURA A FORMA NA CENA** — a metade que prova que a lei do transbordo
/// entra no recorte, e não só a caixa do preenchimento.
///
/// ⚠️ A fixtura é construída para que o PREENCHIMENTO caia fora e o TRAÇO alcance a janela: sem o
/// transbordo na conta, a forma sai e o artista vê o contorno dela desaparecer ao aproximar-se da
/// borda. *É o mesmo sintoma da ponta CEIFADA que a `inflate_for_stroke` já documenta.*
#[test]
fn o_traco_gordo_segura_a_forma_na_cena() {
    let mut p = quadrado();
    p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 1.0));
    // Escala `10` ⇒ a forma mede `10` e o traço `10` de largura (`5` para cada lado).
    // Pousada a `x = −9`, o preenchimento vive em `[−14, −4]` (FORA) e o traço chega a `+1`.
    let pose = Affine::translate((-9.0, 50.0)) * Affine::scale(10.0);
    let (objectos, recortadas) = desenha(&p, &[pose], Some(janela()));
    // ⚠️ **`2` e não `1`**: uma forma com preenchimento E traço emite DOIS objectos de desenho, e
    // a 1.ª redacção deste gate assumiu um. *A régua estava errada, não o produto* — o que se
    // afirma é que ela DESENHOU e não foi recortada.
    assert!(
        objectos >= 1 && recortadas == 0,
        "o traço alcança a janela ⇒ a forma fica (leu {objectos} objectos, {recortadas} recortadas)"
    );

    // ⭐ O CONTROLO: a MESMA pose sem traço sai — é isso que prova que foi o traço a segurá-la.
    let sem_traco = quadrado();
    let (objectos, recortadas) = desenha(&sem_traco, &[pose], Some(janela()));
    assert_eq!(
        (objectos, recortadas),
        (0, 1),
        "controlo: sem traço, a mesma pose é recortada"
    );
}

/// ⭐⭐⭐ **A CAIXA DO RECORTE CONTÉM A CAIXA EXACTA** — o que liga a metade barata à lei.
///
/// ⚠️ O recorte julga com uma caixa CONSERVADORA (transformar o rectângulo envolvente e não a
/// geometria), e o que a torna segura não é essa frase: é esta medição, sobre formas e afins que
/// incluem rotação, escala não-uniforme e traço. *Se a caixa barata alguma vez não contiver a
/// exacta, o recorte pode comer uma forma visível.*
#[test]
fn a_caixa_do_recorte_contem_a_caixa_exacta() {
    let mut com_traco = quadrado();
    com_traco.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 0.2));
    let formas = [quadrado(), com_traco];
    let afins = [
        Affine::IDENTITY,
        Affine::scale(7.0),
        Affine::rotate(0.7) * Affine::scale(3.0),
        Affine::scale_non_uniform(5.0, 0.5) * Affine::rotate(0.4),
        Affine::translate((30.0, -12.0)) * Affine::rotate(-1.1) * Affine::scale(9.0),
    ];
    let mut medidas = 0;
    for f in &formas {
        let tess = crate::instance::tessellate_shape_instance(f);
        let local = tess.caixa_local.expect("a forma tem geometria");
        for xf in afins {
            let barata = crate::bounds_from_local(&tess.transbordo, local, xf);
            let exacta = crate::path_bounds_under(f, xf).expect("a forma tem geometria");
            assert!(
                barata.x0 <= exacta.x0
                    && barata.y0 <= exacta.y0
                    && barata.x1 >= exacta.x1
                    && barata.y1 >= exacta.y1,
                "a caixa barata {barata:?} tem de CONTER a exacta {exacta:?}"
            );
            medidas += 1;
        }
    }
    assert_eq!(medidas, 10, "controlo: as dez células têm de ter corrido");
}

/// ⭐ **A LEI DA PORTA DE BISSECÇÃO, pura** — as quatro células.
///
/// ⚠️ Ela é uma função de um `Option<&str>` e não uma leitura do ambiente, pela lei da casa: *um
/// gate que lê o ambiente mede a MÁQUINA em que corre*.
#[test]
fn a_porta_do_recorte_so_desliga_com_zero() {
    assert!(crate::instance::recorte_por(None), "ausente ⇒ LIGADO");
    assert!(!crate::instance::recorte_por(Some("0")), "`0` ⇒ desligado");
    assert!(
        !crate::instance::recorte_por(Some(" 0 ")),
        "com espaços ⇒ desligado"
    );
    assert!(crate::instance::recorte_por(Some("1")), "`1` ⇒ ligado");
    assert!(
        crate::instance::recorte_por(Some("banana")),
        "lixo ⇒ LIGADO (o caminho de omissão é o do produto)"
    );
}

/// ⭐⭐⭐ **O TRANSBORDO HOISTADO É A LEI DE REFERÊNCIA, AO BIT.**
///
/// ⛔⛔ Ele existe porque a cura do recorte reescreveu a [`standalone::inflate_for_stroke`] por
/// PERFORMANCE — os coeficientes subiram para fora do laço, para o teste do recorte deixar de
/// custar mais do que o recorte poupa. *Uma reescrita de performance que se diz byte-idêntica sem
/// a referência ao lado é uma promessa*, e a referência vive sob `cfg(test)` exactamente para isto.
///
/// ⚠️ A comparação é `==` sobre `f64` **de propósito**: a ordem das operações foi preservada à
/// mão (`((meia · smax) · alcance)`), e um epsilon aqui esconderia precisamente o defeito que este
/// gate existe para apanhar — reassociar três factores.
///
/// ⛔⛔ **E há um MUTANTE EQUIVALENTE aqui, NOMEADO com a medição em vez de forçado a sangrar:**
/// reassociar para `meia · (smax · alcance)` **não muda um bit**, e não por sorte — o `alcance` é
/// `1,0` (junta não-miter) ou o `miter_limit` da kurbo, que o [`crate::kurbo_stroke`] **nunca
/// escreve** e cujo default é `4,0` (`kurbo-0.13.0/src/stroke.rs:98`; o `with_miter_limit` não tem
/// chamador nesta crate). *Multiplicar por uma potência de dois é exacto em IEEE-754*, logo
/// nenhuma fixtura pode distinguir as duas associações enquanto a junta não for configurável.
///
/// ⚠️ **A ordem fica preservada na mesma**, porque ela custa zero e porque o dia em que alguém
/// expuser o `miter_limit` é o dia em que este mutante deixa de ser equivalente — e ninguém vai
/// lembrar-se de voltar aqui.
#[test]
fn o_transbordo_hoistado_e_a_lei_de_referencia_ao_bit() {
    let mut fino = quadrado();
    fino.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 0.03));
    let mut gordo = quadrado();
    gordo.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 3.7));
    let formas = [quadrado(), fino, gordo];
    let afins = [
        Affine::IDENTITY,
        Affine::scale(11.3),
        Affine::rotate(0.73) * Affine::scale(2.9),
        Affine::scale_non_uniform(6.1, 0.37) * Affine::rotate(0.41),
        Affine::translate((17.0, -3.5)) * Affine::rotate(-1.13) * Affine::scale(0.019),
    ];
    let caixa = Rect::new(-1.5, -2.25, 3.125, 0.75);
    let mut medidas = 0;
    for f in &formas {
        let t = crate::standalone::transbordo_do_caminho(f);
        for xf in afins {
            let hoje = crate::standalone::inflate_com(&t, xf, caixa);
            let antes = crate::standalone::inflate_for_stroke_referencia(f, xf, caixa);
            assert_eq!(
                (hoje.x0, hoje.y0, hoje.x1, hoje.y1),
                (antes.x0, antes.y0, antes.x1, antes.y1),
                "o transbordo hoistado divergiu da referência"
            );
            medidas += 1;
        }
    }
    assert_eq!(
        medidas, 15,
        "controlo: as quinze células têm de ter corrido"
    );

    // ⭐ O CONTROLO da própria régua: a lei tem de MOVER a caixa em alguma destas células, senão
    // este gate compara dois no-ops. (Sem traço ela é a identidade; com traço gordo não é.)
    let t = crate::standalone::transbordo_do_caminho(&formas[2]);
    let inflada = crate::standalone::inflate_com(&t, Affine::scale(11.3), caixa);
    assert!(
        inflada.x0 < caixa.x0 && inflada.x1 > caixa.x1,
        "controlo: o traço gordo tem de inflar a caixa"
    );
    assert!(
        crate::standalone::transbordo_do_caminho(&formas[0]).e_nulo(),
        "controlo: uma forma sem traço não transborda — é o que salta as raízes por cópia"
    );
}

/// ⭐⭐⭐ **UMA FORMA SEM CAIXA NÃO É RECORTADA** — o valor conservador, e uma mutação sobrevivente
/// é que o pediu.
///
/// ⛔⛔ O corpo do recorte diz por escrito *«sem caixa, a forma SEGUE»*, e **nenhuma fixtura tinha
/// uma forma sem caixa**: trocar o `if let (Some, Some)` por um `unwrap_or(Rect::ZERO)` — que
/// RECORTA tudo o que não se soube medir — passava os seis gates. *Uma cerca que o corpus não
/// exercita é um comentário com sintaxe de código.*
///
/// ⚠️ **O observável é o CONTADOR e não o desenho:** uma forma sem geometria não emite nada de
/// qualquer maneira, logo o encode não distingue as duas leis. É exactamente a razão de o contador
/// existir.
#[test]
fn uma_forma_sem_caixa_nao_e_recortada() {
    let vazia = VecPath {
        verts: Vec::new(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(10, 10, 10, 255))),
        ..VecPath::default()
    };
    let tess = crate::instance::tessellate_shape_instance(&vazia);
    assert!(
        tess.caixa_local.is_none(),
        "controlo: esta fixtura tem de ser a que não se sabe medir"
    );
    // Bem longe da janela — onde uma forma COM caixa seria recortada.
    let longe = Affine::translate((-9_000.0, -9_000.0)) * Affine::scale(3.0);
    let (_, recortadas) = desenha(&vazia, &[longe], Some(janela()));
    assert_eq!(
        recortadas, 0,
        "sem caixa não se decide: a forma segue, e não conta como recortada"
    );

    // ⭐ O CONTROLO: a MESMA pose com uma forma que TEM caixa é recortada — é isso que prova que a
    // fixtura está no regime em que o recorte de facto corta.
    let (_, recortadas) = desenha(&quadrado(), &[longe], Some(janela()));
    assert_eq!(recortadas, 1, "controlo: com caixa, esta pose é recortada");
}

/// ⭐⭐⭐ **A CAIXA É A DO CONTORNO MAIS GORDO DA PILHA** — a lei que a `inflate_for_stroke` já
/// escrevia e que nenhuma fixtura exercitava.
///
/// ⛔⛔ Mutação sobrevivente: trocar *«o mais gordo»* por *«o primeiro»* passava os seis gates,
/// porque **todas as fixturas tinham um traço só**. A prosa daquela função chama a isto «dez vezes
/// mais gorda do que o `stroke.width` diz», e o sintoma é a forma a desaparecer da tela ao
/// aproximar-se da borda.
#[test]
fn a_caixa_e_a_do_contorno_mais_gordo_da_pilha() {
    use ph2d_vec_scene::{PaintEntry, PaintKind};
    let mut p = quadrado();
    // O traço BASE é fino; a camada de cima é DEZ vezes mais gorda.
    p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 0.1));
    p.paints = vec![PaintEntry::new(PaintKind::Stroke(StrokeSpec::new(
        Rgba8::new(0, 0, 0, 255),
        1.0,
    )))];
    // ⚠️ **A ARITMÉTICA da fixtura, escrita por extenso porque a 1.ª redacção caiu EM CIMA da
    // folga da janela** (o controlo leu `x1 = −2` contra uma borda em `−2`, e `toca` usa `>=` de
    // propósito). Com escala `10` e junta MITER (alcance `4`): o traço fino infla
    // `0,5 · 0,1 · 10 · 4 = 2` e o gordo `0,5 · 1,0 · 10 · 4 = 20`; o preenchimento mede `±5`.
    // ⇒ a `x = −12` o fino chega a `−5` (FORA da janela inflada, que começa em `−2`) e o gordo
    // chega a `+13` (DENTRO). *É a única posição em que os dois lados do gate afirmam algo.*
    let pose = Affine::translate((-12.0, 50.0)) * Affine::scale(10.0);
    let (objectos, recortadas) = desenha(&p, &[pose], Some(janela()));
    assert!(
        objectos >= 1 && recortadas == 0,
        "o contorno mais gordo alcança a janela ⇒ a forma fica (leu {objectos}, {recortadas})"
    );

    // ⭐ O CONTROLO: com a camada gorda FORA da pilha, a mesma pose é recortada — é isso que prova
    // que foi ela a segurar a forma, e não o traço base.
    let mut so_fino = p.clone();
    so_fino.paints.clear();
    let (_, recortadas) = desenha(&so_fino, &[pose], Some(janela()));
    assert_eq!(
        recortadas, 1,
        "controlo: só com o traço fino, a mesma pose sai"
    );
}
