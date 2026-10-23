//! ⭐ **Duas pinceladas sobrepostas no tempo pintam ao mesmo tempo** (W16, `docs/Macros/14_W16_plano.md`
//! §5, G3–G7): a vermelha em cima, a azul em baixo (disjuntas); a azul deslocada para começar a meio da
//! vermelha. Nos quatro meios: a meio da sobreposição as duas regiões mudaram no MESMO quadro, nenhuma
//! acabada; no fim as duas estão inteiras e cada uma com a SUA cor; o ▷ repete-se ao byte. Em Digital o
//! fim é, ao byte, o das duas em série (a ordem não importa a regiões disjuntas).
//!
//! Medido antes da cura (22/10, um traço só no tool): o Down da azul tomava o traço da vermelha — uma
//! linha a ligar as duas (cima 4 911 texels contra 3 938) e a azul pintada de vermelho (R 756 712 · B
//! 287 704 em baixo); no Wet Paint, as duas no mesmo rasto do motor da água (a vermelha ficava roxa).

use super::etapas::{self, ErroDeEdicaoDoPainter, LeitorDoPainter};
use super::roteiro::{Pincelada, grava_nos_meios, grava_o_traco_sob_o_fill_e_por_cima};
use super::tocar::TocadorDoPainter;
use crate::PaintMedia;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_macro::{Track, codec};

const LADO: u32 = 128;
// LITERAL-COLOR-OK: as cores de pinceladas GRAVADAS, não da UI.
const VERMELHA: [u8; 3] = [220, 30, 30];
// LITERAL-COLOR-OK: idem.
const AZUL: [u8; 3] = [30, 40, 220];

const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

fn p(cor: [u8; 3], de: [f32; 2], ate: [f32; 2]) -> Pincelada {
    Pincelada {
        cor,
        tamanho_px: 18.0,
        de,
        ate,
        curva: 0.0,
    }
}

fn disjuntas() -> [Pincelada; 2] {
    [
        p(VERMELHA, [0.15, 0.25], [0.85, 0.25]),
        p(AZUL, [0.15, 0.75], [0.85, 0.75]),
    ]
}

fn gravada(ps: &[Pincelada], meios: &[PaintMedia]) -> Track {
    let (b, _) = grava_nos_meios(LADO, ps, meios).expect("grava");
    codec::read(&b).expect("lê-se").tracks.remove(0)
}

/// A etapa `k` deslocada para começar no instante `t` µs, pela porta do painel.
fn levada(g: &Track, k: usize, t: u64) -> Result<Track, ErroDeEdicaoDoPainter> {
    let e = etapas::etapas(g).expect("lê-se");
    let d = i64::try_from(t).expect("µs") - i64::try_from(e[k].t0.0).expect("µs");
    let mut tr = g.clone();
    etapas::desloca(&mut tr, k, d)?;
    Ok(tr)
}

/// A azul a começar a meio da vermelha, e um instante em que as duas estão a meio.
fn sobreposta(g: &Track) -> (Track, u64) {
    let e = etapas::etapas(g).expect("lê-se");
    let meio_da_1a = (e[0].t0.0 + e[0].t1.0) / 2;
    let s = levada(g, 1, meio_da_1a).expect("sobrepor duas pinceladas é aceite");
    (s, (meio_da_1a + e[0].t1.0) / 2)
}

/// A tela (RGBA straight) no instante `ate` µs da trilha que toca (`None` = no fim) e a do início.
fn telas(track: &Track, ate: Option<u64>) -> (Vec<u8>, Vec<u8>) {
    let t = track.tocavel(&LeitorDoPainter).expect("a edição cabe");
    let novo = || {
        TocadorDoPainter::novo(&t.initial, t.events.iter().map(|e| e.payload.as_slice()))
            .expect("tocador")
    };
    // `run_full` é o Aplicar: entrega a tela e esvazia o tool — a do início sai de outro tocador.
    let inicio = novo().tool_mut().run_full().0;
    let mut toc = novo();
    for e in t.events.iter().filter(|e| ate.is_none_or(|a| e.t.0 <= a)) {
        toc.aplica(&e.payload).expect("toca");
    }
    (toc.tool_mut().run_full().0, inicio)
}

/// Os texels que mudaram em cima (`false`) ou em baixo (`true`) e as somas de R e de B deles.
#[derive(Debug, Clone, Copy)]
struct Regiao {
    n: usize,
    r: i64,
    b: i64,
}

fn regiao(tela: &[u8], inicio: &[u8], baixo: bool) -> Regiao {
    let mut g = Regiao { n: 0, r: 0, b: 0 };
    let meia = (LADO / 2) as usize;
    for (i, (p, q)) in tela.chunks(4).zip(inicio.chunks(4)).enumerate() {
        if (i / LADO as usize >= meia) != baixo || p == q {
            continue;
        }
        g.n += 1;
        g.r += i64::from(p[0]);
        g.b += i64::from(p[2]);
    }
    g
}

fn diferentes(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

/// Mutações que sangram: o tocador sem faixas (o passo da faixa aplicado no traço do tool) · a troca sem
/// o pincel · o tique só do traço do tool · o fecho partilhado no 1.º pen-up · as lanes da água sem
/// índices próprios.
#[test]
fn duas_pinceladas_sobrepostas_pintam_ao_mesmo_tempo_nos_quatro_meios() {
    for meio in MEIOS {
        let g = gravada(&disjuntas(), &[meio]);
        let (s, a_meio) = sobreposta(&g);
        let (serie, ini) = telas(&g, None);
        let (fim, _) = telas(&s, None);
        let (meio_t, _) = telas(&s, Some(a_meio));
        let [sc, sb] = [false, true].map(|b| regiao(&serie, &ini, b));
        let [fc, fb] = [false, true].map(|b| regiao(&fim, &ini, b));
        let [mc, mb] = [false, true].map(|b| regiao(&meio_t, &ini, b));
        eprintln!("{meio:?}: série {sc:?} {sb:?} · sobreposta {fc:?} {fb:?} · a meio {mc:?} {mb:?}");
        for (a_meio, fim, serie, onde) in [(mc, fc, sc, "a vermelha"), (mb, fb, sb, "a azul")] {
            assert!(
                a_meio.n * 4 > serie.n && a_meio.n < fim.n,
                "{meio:?}: {onde} está a meio no mesmo quadro ({} de {})",
                a_meio.n,
                fim.n
            );
            assert!(
                fim.n.abs_diff(serie.n) * 50 <= serie.n,
                "{meio:?}: {onde} inteira ({} contra {} em série)",
                fim.n,
                serie.n
            );
        }
        assert!(fc.r > fc.b, "{meio:?}: a de cima é vermelha {fc:?}");
        assert!(fb.b > fb.r, "{meio:?}: a de baixo é azul — o pincel viaja com ela {fb:?}");
        assert_eq!(
            diferentes(&fim, &telas(&s, None).0),
            0,
            "{meio:?}: o ▷ repete-se ao byte"
        );
        if meio == PaintMedia::Digital {
            assert_eq!(diferentes(&fim, &serie), 0, "Digital: a das duas em série");
        }
    }
}

/// Trocar a ordem de duas que se CRUZAM dá, ao byte, a gravação feita na ordem trocada — e a original não.
#[test]
fn passar_a_frente_de_outra_e_grava_la_antes() {
    let a = p(VERMELHA, [0.2, 0.2], [0.8, 0.8]);
    let b = p(AZUL, [0.2, 0.8], [0.8, 0.2]);
    let g = gravada(&[a, b], &[PaintMedia::Digital]);
    let e = etapas::etapas(&g).expect("lê-se");
    let trocada = levada(&g, 0, e[1].t1.0 + 200_000).expect("passar à frente é aceite");
    let verdade = telas(&gravada(&[b, a], &[PaintMedia::Digital]), None).0;
    let (fim, _) = telas(&trocada, None);
    assert!(
        diferentes(&telas(&g, None).0, &verdade) > 0,
        "controlo: a ordem muda o cruzamento"
    );
    assert_eq!(diferentes(&fim, &verdade), 0);
    let vistas = etapas::etapas(&trocada).expect("lê-se");
    assert_eq!(
        (vistas[0].cor, vistas[1].cor),
        (VERMELHA, AZUL),
        "o painel lista pela gravação, cada uma com o seu pincel"
    );
    assert!(vistas[0].t0 > vistas[1].t1, "e cada uma no instante dela");
}

/// Só pinceladas simples do mesmo meio se sobrepõem; a recusa deixa a trilha como estava.
#[test]
fn sobrepor_um_preenchimento_ou_outro_meio_recusa_se() {
    let (b, _) = grava_o_traco_sob_o_fill_e_por_cima(LADO).expect("grava");
    let g = codec::read(&b).expect("lê-se").tracks.remove(0);
    let e = etapas::etapas(&g).expect("lê-se");
    assert_eq!(
        levada(&g, 2, e[1].t0.0).err(),
        Some(ErroDeEdicaoDoPainter::SoPinceladasSeSobrepoem(2)),
        "uma pincelada em cima do ColorDrop"
    );
    let mistas = gravada(&disjuntas(), &[PaintMedia::Digital, PaintMedia::Watercolor]);
    let e = etapas::etapas(&mistas).expect("lê-se");
    let mut tr = mistas.clone();
    assert_eq!(
        etapas::desloca(&mut tr, 1, -i64::try_from(e[1].t0.0 - e[0].t0.0).expect("µs")),
        Err(ErroDeEdicaoDoPainter::OutroMeio(1)),
        "Watercolor em cima de Digital"
    );
    assert_eq!(tr.edicoes, mistas.edicoes, "a recusa não grava");
    let antes = levada(&mistas, 1, 0).err();
    assert!(
        antes.is_some(),
        "sobrepor no início também é sobrepor (a vermelha começa em 0)"
    );
    let depois = levada(&mistas, 0, e[1].t1.0 + 100_000).expect("sem sobrepor, o meio viaja no pincel");
    let v = etapas::etapas(&depois).expect("lê-se");
    assert_eq!(
        (v[0].meio, v[1].meio),
        (PaintMedia::Digital, PaintMedia::Watercolor)
    );
    let molhada = gravada(&disjuntas(), &[PaintMedia::Digital, PaintMedia::WetPaint]);
    let e = etapas::etapas(&molhada).expect("lê-se");
    assert_eq!(
        levada(&molhada, 1, e[0].t1.0 + 50_000).err(),
        Some(ErroDeEdicaoDoPainter::OutroMeio(1)),
        "a água do Wet Paint é da tela: antes de a armar não há Wet Paint"
    );
}

/// Levar a etapa de volta ao sítio dela não deixa edição vazia: o ficheiro volta a ser o de antes.
#[test]
fn deslocar_para_o_sitio_dela_nao_guarda_edicao() {
    let g = gravada(&disjuntas(), &[PaintMedia::Digital]);
    let e = etapas::etapas(&g).expect("lê-se");
    let mut tr = levada(&g, 1, e[1].t0.0 + 100_000).expect("aceite");
    assert_eq!(tr.edicoes.len(), 1);
    etapas::desloca(&mut tr, 1, 0).expect("de volta");
    assert_eq!(tr.edicoes, g.edicoes, "sem edição vazia");
}
