//! ⭐⭐⭐ **A BORDA QUE FERVE — os portões** (`W7c`, report do dono de 2026-09-19).
//!
//! ⚠️ **Irmão do [`super::borda_sondas`] por RESPONSABILIDADE:** este **afirma** (barras, controlos
//! e mutações que os matam) e aquele **mede** (imprime tabelas e não afirma quase nada). *Uma sonda
//! e um gate têm leitores diferentes: a primeira responde «quanto», o segundo «ainda é verdade».*
//!
//! ⭐ **As RÉGUAS vivem aqui e a sonda consome-as** — a banda da silhueta e a luminância composta
//! são a mesma lei nos dois sítios, e duas cópias divergiriam no dia em que uma fosse afinada.

use super::device_tests::{FUNDO, LH, LW};

/// A cena dos portões: a **lâmina e a esfera**, que põem uma silhueta recta e uma curva na mesma
/// imagem — as duas formas em que a escada se lê de maneiras diferentes.
const CENA: u32 = 33;

/// A luminância de um pixel **pré-multiplicado** — que é o que o
/// [`ph2d_field_gpu::trace::Pintado`] entrega, logo isto já é a composição sobre preto.
pub(super) fn luz(px: &[u8]) -> f32 {
    0.2126 * f32::from(px[0]) + 0.7152 * f32::from(px[1]) + 0.0722 * f32::from(px[2])
}

/// ⭐ **A BANDA DA SILHUETA** — os pixels cuja vizinhança `3×3` atravessa a fronteira da peça.
///
/// ⚠️ **Ela é definida pelo ALFA e não pela cor**: uma aresta interna de um vinco também muda de
/// cor depressa, e a pergunta aqui é só sobre o contorno contra o fundo.
pub(super) fn banda(rgba: &[u8], w: usize, h: usize) -> Vec<usize> {
    let a = |i: usize| rgba[i * 4 + 3];
    let mut out = Vec::new();
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let (mut lo, mut hi) = (255u8, 0u8);
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
                    let j = (y as i32 + dy) as usize * w + (x as i32 + dx) as usize;
                    lo = lo.min(a(j));
                    hi = hi.max(a(j));
                }
            }
            if hi - lo >= 128 {
                out.push(y * w + x);
            }
        }
    }
    out
}

/// Que fracção da banda leva cobertura **entre** `0` e `255` — a grandeza que a segunda passagem
/// produz e que a sua ausência apaga por completo.
#[allow(clippy::cast_precision_loss)]
pub(super) fn parcial(rgba: &[u8], b: &[usize]) -> f32 {
    b.iter()
        .filter(|&&i| {
            let a = rgba[i * 4 + 3];
            a > 0 && a < 255
        })
        .count() as f32
        / b.len() as f32
}

/// Uma câmara de omissão rodada de `d` radianos em `yaw` — a porta da casa
/// ([`ph2d_field_render::Orbit::from_yaw_pitch`]), e não um quaternion escrito à mão.
pub(super) fn camara(d: f32) -> ph2d_field_render::Orbit {
    ph2d_field_render::Orbit {
        rotation: ph2d_field_render::Orbit::from_yaw_pitch(0.72 + d, 0.52).rotation,
        ..ph2d_field_render::Orbit::default()
    }
}

/// Um quadro pintado no dispositivo, com o documento, a bandeira e a sonda que o chamador escolhe.
pub(super) fn quadro(
    t: &crate::gpu_frame::SharedTracer,
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    assente: bool,
    sonda: crate::gpu_frame::Sonda,
) -> Option<ph2d_field_gpu::trace::Pintado> {
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    crate::gpu_frame::paint_com(
        t,
        doc,
        reg,
        cam,
        &[crate::gpu_frame::tests_lampada(cam)],
        &surfaces,
        &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
        FUNDO,
        None,
        LW,
        LH,
        assente,
        sonda,
    )
}

/// O documento como o quadro de MOVIMENTO o vê — com o contorno engrossado, se ele morder.
fn doc_a_mexer() -> ph2d_field::FieldDoc {
    let real = crate::smoke::scene(CENA);
    crate::preview::coarse_doc(&real, true).unwrap_or(real)
}

/// ⭐⭐⭐ **O QUADRO QUE A MÃO ARRASTA LEVA COBERTURA PARCIAL NA SILHUETA** — a `W7c` inteira, num
/// número.
///
/// # ⛔⛔ O CONTROLO vem primeiro, e aqui ele é o defeito de 19/09
///
/// Sem ele este gate passaria com a segunda passagem **partida**: `x > 0,2` sobre uma banda cheia de
/// cobertura parcial vinda de outro sítio leria igual. O controlo é o produto **de ontem** — a
/// mesma cena, a mesma câmara, com a [`crate::gpu_frame::Sonda::bordas`] em baixo —, e ele tem de
/// medir **zero**: *a silhueta que a mão arrastava era uma escada binária, sem um único pixel no
/// meio.*
#[test]
#[ignore = "precisa de GPU"]
fn o_quadro_que_a_mao_arrasta_leva_cobertura_parcial() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (doc, reg) = (doc_a_mexer(), crate::smoke::sampled_registry());
    let (w, h) = (LW as usize, LH as usize);
    let cam = camara(0.0);
    let sem = crate::gpu_frame::Sonda {
        bordas: false,
        ..crate::gpu_frame::Sonda::default()
    };
    let Some(antes) = quadro(t, &doc, &reg, &cam, false, sem) else {
        println!("a placa recusa esta peça — saltado");
        return;
    };
    let b = banda(&antes.rgba, w, h);
    assert!(
        b.len() > 1_000,
        "a banda da silhueta tem {} pixels — a fixtura não tem contorno, e as barras abaixo não \
         estariam a afirmar nada",
        b.len()
    );
    let antes_pct = parcial(&antes.rgba, &b);
    assert!(
        antes_pct == 0.0,
        "o CONTROLO mediu {:.1} % de cobertura parcial sem a segunda passagem — ou a régua deixou \
         de medir a silhueta, ou a cobertura passou a vir de outro sítio",
        antes_pct * 100.0
    );

    let agora = quadro(
        t,
        &doc,
        &reg,
        &cam,
        false,
        crate::gpu_frame::Sonda::default(),
    )
    .expect("a mesma peça tem de pintar com a segunda passagem");
    let agora_pct = parcial(&agora.rgba, &banda(&agora.rgba, w, h));
    // A barra sai do MEDIDO (`28`–`39 %` em cinco cenas) com a folga de uma cena que tenha menos
    // contorno curvo — ⛔ e não de um número confortável.
    assert!(
        agora_pct > 0.20,
        "o quadro de MOVIMENTO devolveu {:.1} % de cobertura parcial na silhueta (medido: 28–39 %) \
         — a segunda passagem não está a chegar ao quadro que a mão arrasta, e a peça ferve",
        agora_pct * 100.0
    );
}

/// ⭐⭐⭐ **E ELA É A BORDA DE PARAR** — o alfa do quadro de movimento é o do quadro assente, ao BYTE.
///
/// ⚠️ **É esta a frase que o dono vai ler na tela:** *a peça que a mão arrasta deixa de ter uma
/// silhueta pior do que a da peça parada.* Os dois quadros continuam a diferir na COR (o ricochete
/// e a cor que a peça devolve ao chão são do assente, e custam `+284 ms` na cena `5`), e é por isso
/// que a régua é o canal do ALFA e não a imagem.
///
/// ⛔ **O contorno engrossado não entra nesta igualdade por acaso:** medido em 2026-09-19
/// (`borda_sondas::o_contorno_grosso_muda_alguma_coisa`), o [`crate::preview::coarse_doc`] **não
/// morde em nenhuma** das cenas do smoke — os perfis são feitos de ARCOS, e um arco já tem a normal
/// exacta, logo engrossá-lo nunca fica mais barato. *Se algum dia morder, este gate reprova e diz
/// que a silhueta voltou a ter duas versões.*
#[test]
#[ignore = "precisa de GPU"]
fn a_borda_que_a_mao_arrasta_e_a_de_parar() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let reg = crate::smoke::sampled_registry();
    let cam = camara(0.0);
    let real = crate::smoke::scene(CENA);
    let mexe = crate::preview::coarse_doc(&real, true).unwrap_or_else(|| real.clone());
    let para = crate::preview::coarse_doc(&real, false).unwrap_or(real);
    let Some(a) = quadro(
        t,
        &mexe,
        &reg,
        &cam,
        false,
        crate::gpu_frame::Sonda::default(),
    ) else {
        println!("a placa recusa esta peça — saltado");
        return;
    };
    let b = quadro(
        t,
        &para,
        &reg,
        &cam,
        true,
        crate::gpu_frame::Sonda::default(),
    )
    .expect("a mesma peça tem de pintar o quadro assente");
    let alfa = |p: &ph2d_field_gpu::trace::Pintado| -> Vec<u8> {
        p.rgba.as_chunks::<4>().0.iter().map(|px| px[3]).collect()
    };
    let (ca, cb) = (alfa(&a), alfa(&b));
    // ⭐ O controlo: as duas imagens TÊM de diferir nalgum sítio, senão a igualdade abaixo seria
    // trivialmente verdadeira sobre dois quadros idênticos e o gate não afirmaria nada.
    assert!(
        a.rgba != b.rgba,
        "o quadro de movimento e o assente saíram IDÊNTICOS — o ricochete e o campo do chão \
         deixaram de correr no assente, e a igualdade de alfa abaixo deixou de dizer alguma coisa"
    );
    let difere = ca.iter().zip(&cb).filter(|(x, y)| x != y).count();
    assert_eq!(
        difere, 0,
        "{difere} pixels têm cobertura diferente entre o quadro que a mão arrasta e o de parar — \
         a silhueta voltou a ter duas versões, e é isso que o olho lê como a peça a mudar ao largar"
    );
}

/// ⭐⭐ **OS TRÊS CAMINHOS DE UM QUADRO LÊEM A MESMA PORTA** — o pintor, o recuo pelo `march` e o
/// recuo pela CPU.
///
/// ⚠️ **Dois deles não são alcançáveis de um teste**: eles só correm quando a placa recusa a peça
/// ou quando não há adaptador nenhum. ⇒ o censo é do TEXTO, e o que ele mede é que ninguém escreveu
/// um `true` à mão ao lado da porta — *uma lei escrita em três sítios viaja para os dois de que
/// alguém se lembrou.*
#[test]
fn os_tres_caminhos_de_um_quadro_leem_a_mesma_porta() {
    let thread = include_str!("smoke_draw_thread.rs");
    let gpu = include_str!("gpu_frame.rs");
    let leitores = thread
        .matches("crate::preview::re_amostra_a_silhueta()")
        .count()
        + gpu
            .matches("crate::preview::re_amostra_a_silhueta()")
            .count();
    assert_eq!(
        leitores, 3,
        "a porta `re_amostra_a_silhueta` tem {leitores} leitores e os caminhos de um quadro são \
         TRÊS (o pintor pela `Sonda::default`, o recuo pelo `march` e o recuo pela CPU) — um \
         caminho que não a leia é uma silhueta que volta a ferver sem nada acusar"
    );
    let preview = include_str!("preview.rs");
    assert!(
        preview.contains(r#"std::env::var("PH2D_FIELD_BORDA").as_deref() != Ok("0")"#),
        "a porta de bissecção da `W7c` mudou de forma — sem ela um report de «piorou» não diz QUAL \
         mudança o causou, e o passo do smoke que mostra a fervura de volta deixa de existir"
    );
    assert!(
        !gpu.contains("antialias,\n"),
        "o `gpu_frame` voltou a encaminhar uma bandeira chamada `antialias` — ela saiu em 2026-09-19 \
         porque descrevia a bandeira do quadro assente pelo passageiro mais barato dos quatro"
    );
}
