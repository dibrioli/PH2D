//! ⭐⭐⭐ **Os gates das CAMADAS no documento** (v6, `docs/3D/30` §5). Filho
//! (`#[path]`) do [`super`]; correm sem janela, pelo par [`encode`]/[`decode`].

use super::doc_camadas::CamadasDoc;
use super::*;
use crate::pilha_da_peca::{PilhaDaPeca, nome_da_base};
use ph2d_mesh::{Multires, shapes};
use ph2d_tool_painter::{AdjustmentKind, AdjustmentParams, BlendMode, HsbParams};

/// Uma peça com plano SEMEADO da cor por vértice, uma mancha de outra cor e,
/// por baixo da mancha, uma lomba de relevo.
fn peca() -> (Multires, Pose, Tinta) {
    let mut stack = Multires::new(shapes::octahedron(1.0));
    for i in 0..stack.mesh().vert_count() {
        stack.mesh_mut().colors_mut()[i] = [0.2 + 0.1 * i as f32, 0.4, 0.6];
    }
    let m = stack.mesh();
    let faces = || m.faces().iter().map(ph2d_mesh::Face::verts);
    let mut t = Tinta::semeada(m.colors().expect("cor"), faces(), 3);
    let n = t.amostras().len();
    for i in (n / 4)..(n / 3) {
        t.amostras_mut()[i] = [0.9, 0.1, 0.05 + i as f32 * 1e-5];
        t.relevo_mut()[i] = [0.01 + i as f32 * 1e-6, 0.5];
    }
    (stack, Pose::new([1.0, 0.0, 0.0], 1.5), t)
}

fn bits3(a: &[[f32; 3]]) -> Vec<[u32; 3]> {
    a.iter().map(|c| c.map(f32::to_bits)).collect()
}

fn bits2(a: &[[f32; 2]]) -> Vec<[u32; 2]> {
    a.iter().map(|c| c.map(f32::to_bits)).collect()
}

fn pior(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .flat_map(|(x, y)| (0..3).map(move |c| (x[c] - y[c]).abs()))
        .fold(0.0, f32::max)
}

/// ⭐⭐⭐⭐ **GATE — UM v5 ABRE IGUAL, e regrava-se a meio degrau.**
///
/// (1) O v5 abre ao BIT — a leitura dele não passa pelas camadas. (2) Gravado
/// de novo (v6), ele é UMA camada opaca com o nome da base, e volta a no
/// máximo meio degrau de sRGB8 na cor e ao bit no relevo.
#[test]
fn um_v5_abre_igual_e_regrava_a_meio_degrau() {
    let (stack, pose, t) = peca();
    let (lidas, _) =
        decode(&super::tests::encode_v5(&stack, &pose, &t)).expect("um v5 tem de abrir");
    let aberto = lidas[0].tinta.as_ref().expect("o plano do v5");
    assert_eq!(
        bits3(aberto.amostras()),
        bits3(t.amostras()),
        "o v5 abriu ao bit"
    );
    assert_eq!(
        bits2(aberto.relevo().expect("relevo")),
        bits2(t.relevo().expect("relevo"))
    );

    let v6 = encode(&[(stack.to_data(), pose.to_data(), Some(aberto))], 0);
    let doc: SculptDoc = postcard::from_bytes(&v6).expect("re-lê");
    let camadas = &doc.objects[0].tinta.as_ref().expect("plano").camadas;
    assert_eq!(camadas.pilha.len(), 1, "UMA camada");
    let base = camadas.pilha.root()[0];
    assert_eq!(
        camadas.pilha.get(base).map(|c| c.name.as_str()),
        Some(nome_da_base())
    );

    let (lidas, _) = decode(&v6).expect("o v6 abre");
    let volta = lidas[0].tinta.as_ref().expect("o plano");
    let p = pior(volta.amostras(), t.amostras());
    assert!(p <= 0.5 / 255.0 + 1e-6, "a cor voltou a {p}");
    assert!(
        p > 0.0,
        "o CONTROLO: a fixtura tem cor que não cai em degrau"
    );
    assert_eq!(
        bits2(volta.relevo().expect("relevo")),
        bits2(t.relevo().expect("relevo")),
        "o relevo ao bit"
    );
}

/// Uma pilha com o que o ficheiro tem de levar: duas camadas, modo, opacidade,
/// recorte, máscara, um ajuste com parâmetros e relevo na base.
fn pilha_rica(t: &Tinta) -> PilhaDaPeca {
    let n = t.amostras().len();
    let mut p = PilhaDaPeca::de_tinta(t);
    let base = p.pilha().root()[0];
    let cima = p.nova_camada("cima").expect("camada");
    let px: Vec<[u8; 4]> = (0..n)
        .map(|i| [i as u8, 7, (i / 3) as u8, (i * 5) as u8])
        .collect();
    p.plano_mut(cima).expect("plano").escreve(&px, None);
    p.define_modo(cima, BlendMode::Screen);
    p.define_opacidade(cima, 0.35);
    p.define_recorte(cima, true);
    let mascara = p.nova_mascara(cima).expect("máscara");
    let cinza: Vec<[u8; 4]> = (0..n).map(|i| [(i * 3) as u8; 4]).collect();
    p.plano_mut(mascara).expect("plano").escreve(&cinza, None);
    let hsb = p
        .novo_ajuste(AdjustmentKind::HueSaturationBrightness)
        .expect("ajuste");
    p.define_parametros(
        hsb,
        AdjustmentParams::HueSaturationBrightness(HsbParams {
            h: -40.0,
            s: 0.3,
            b: 0.0,
        }),
    )
    .expect("parâmetros");
    p.define_visivel(base, true);
    assert!(p.sincronizada());
    p
}

/// ⭐⭐⭐ **GATE — A PILHA ATRAVESSA O FICHEIRO** igual (metadado e planos).
#[test]
fn a_pilha_atravessa_o_ficheiro() {
    let (_, _, t) = peca();
    let p = pilha_rica(&t);
    let bytes = postcard::to_allocvec(&CamadasDoc::da_pilha(&p)).expect("serializa");
    let lida: CamadasDoc = postcard::from_bytes(&bytes).expect("re-lê");
    // ⭐ O FUNDO viaja com ela (v7) — é o gravado que volta, não um derivado.
    assert_eq!(lida.fundo.as_slice(), p.fundo());
    let fundo = lida.fundo.clone();
    assert_eq!(lida.pilha(t.amostras().len(), fundo).as_ref(), Some(&p));
}

/// ⛔⛔ **GATE — Uma pilha que não descreve o plano RECUSA o load**, e diz qual
/// peça: um plano a menos, a mais, repetido, ou um relevo da contagem errada.
#[test]
fn uma_pilha_que_nao_descreve_o_plano_recusa_o_load() {
    let (stack, pose, t) = peca();
    let bytes = encode(&[(stack.to_data(), pose.to_data(), Some(&t))], 0);
    assert!(decode(&bytes).is_ok(), "o CONTROLO: assim ele abre");
    let forja = |f: &dyn Fn(&mut CamadasDoc)| {
        let mut doc: SculptDoc = postcard::from_bytes(&bytes).expect("re-lê");
        f(&mut doc.objects[0].tinta.as_mut().expect("plano").camadas);
        decode(&postcard::to_allocvec(&doc).expect("serializa"))
    };
    type Caso<'a> = (&'a str, &'a dyn Fn(&mut CamadasDoc));
    let casos: [Caso; 4] = [
        ("uma camada sem plano", &|c| {
            c.planos.clear();
        }),
        // ⚠️ A cópia é EXACTA (contagem certa): uma cópia com outra contagem era
        //    recusada pela contagem e o caso não chegava à repetição (a mutação
        //    M10 sobreviveu assim).
        ("um plano repetido", &|c| {
            let copia = postcard::from_bytes::<super::doc_camadas::PlanoDoc>(
                &postcard::to_allocvec(&c.planos[0]).expect("serializa"),
            )
            .expect("re-lê");
            c.planos.push(copia);
        }),
        ("um plano sem camada", &|c| {
            let mut p = postcard::from_bytes::<super::doc_camadas::PlanoDoc>(
                &postcard::to_allocvec(&c.planos[0]).expect("serializa"),
            )
            .expect("re-lê");
            p.id = ph2d_tool_painter::LayerId(4_242);
            c.planos.push(p);
        }),
        ("um relevo com uma amostra a mais", &|c| {
            if let Some(super::doc_tinta::Forma::Corridas(r)) = &mut c.planos[0].relevo {
                r.push((1, [0.0, 0.0]));
            } else if let Some(super::doc_tinta::Forma::Cruas(r)) = &mut c.planos[0].relevo {
                r.push([0.0, 0.0]);
            } else {
                panic!("a fixtura tem relevo");
            }
        }),
    ];
    for (nome, f) in casos {
        match forja(f) {
            Err(SculptDocError::Tinta { peca, .. }) => assert_eq!(peca, 0, "{nome}: nomeia a peça"),
            other => panic!("{nome}: tinha de ser recusado, e veio {other:?}"),
        }
    }
}

/// ⭐⭐⭐ **GATE — Onde a pilha não é opaca, a peça mostra a SEMENTE** (a cor
/// por vértice), e não branco nem preto. O CONTROLO é a mesma pilha visível.
#[test]
fn uma_pilha_transparente_assenta_na_semente() {
    let (stack, pose, t) = peca();
    let bytes = encode(&[(stack.to_data(), pose.to_data(), Some(&t))], 0);
    let mut doc: SculptDoc = postcard::from_bytes(&bytes).expect("re-lê");
    let camadas = &mut doc.objects[0].tinta.as_mut().expect("plano").camadas;
    let base = camadas.pilha.root()[0];
    camadas.pilha.set_visible(base, false);
    let (lidas, _) = decode(&postcard::to_allocvec(&doc).expect("serializa")).expect("abre");
    let volta = lidas[0].tinta.as_ref().expect("plano");
    let semente = crate::tinta_da_peca::semente(stack.mesh(), 3);
    let p = pior(volta.amostras(), semente.amostras());
    assert!(p < 1e-5, "a peça escondida mostra a semente (desvio {p})");
    assert!(
        pior(t.amostras(), semente.amostras()) > 0.5,
        "o CONTROLO: a mancha não é a semente"
    );
}

/// ⭐⭐⭐ **HR-14 — A FORMA DA PILHA GRAVADA É PINADA.**
///
/// ⛔ O metadado viaja como o Painter 2D o serializa (`Layer`, `LayerKind`,
/// `AdjustmentLayer` são da `ph2d-tool-painter`), e o postcard é posicional:
/// um campo novo lá muda o blob da escultura sem tocar no
/// `SCULPT_DOC_VERSION`. Este gate é o vermelho.
///
/// ⚠️ Duas fixturas: UMA camada (a conta fecha à mão) e a pilha RICA (máscara,
/// recorte, ajuste com parâmetros — o que a mão não conta, MEDIDO).
///
/// Quebrou? **Suba o `SCULPT_DOC_VERSION`** com o degrau de leitura, e só
/// então re-pine.
#[test]
fn a_forma_da_pilha_gravada_e_pinada() {
    let faces: Vec<[u32; 3]> = vec![[0, 1, 2]];
    let t = Tinta::nova(3, faces.iter().map(|f| &f[..]), 0);
    assert_eq!(t.amostras().len(), 3, "a fixtura: um triângulo no degrau 0");
    let uma = postcard::to_allocvec(&CamadasDoc::da_pilha(&PilhaDaPeca::de_tinta(&t)))
        .expect("serializa")
        .len();
    let (_, _, rica_t) = peca();
    let rica = postcard::to_allocvec(&CamadasDoc::da_pilha(&pilha_rica(&rica_t)))
        .expect("serializa")
        .len();
    // ⭐ A conta da camada única FECHA à mão (MEDIDO em 2026-10-02):
    //
    // | pedaço | bytes |
    // |---|---:|
    // | `arena`: comprimento + o `Layer` (id 1 · nome `7+1` · `Raster` 1 + `1024` 2 + `1` 1 · modo 1 · opacidade 4 · 5 flags · máscara `None` 1 · profundidade 4 · composição 1 · relevo 1) | 1 + 30 |
    // | `root` (1 id) · `active` (`Some` + id) · `next_id` | 2 + 2 + 1 |
    // | `planos`: comprimento + id + `Corridas` (variante + comprimento + `(3, [u8; 4])`) + relevo `None` | 1 + 1 + 7 + 1 |
    // | `fundo` (v7, 03/10): comprimento + `3 × [f32; 3]` (a rica: `1 + 6 × 12 = 73`) | 1 + 36 |
    // | **total** | **83** |
    assert_eq!(
        (uma, rica),
        (83, 3590),
        "a forma da pilha gravada mudou — suba SCULPT_DOC_VERSION, não re-pine este número"
    );
}

/// ⭐⭐⭐ **GATE — O escritor grava a PILHA da peça** (e não a de UMA camada do
/// plano), e o leitor instala-a — sem placa.
#[test]
fn o_escritor_grava_a_pilha_da_peca_e_o_leitor_a_instala() {
    let (stack, pose, t) = peca();
    let p = pilha_rica(&t);
    let mut composto = t.clone();
    p.pinta_tinta(&mut composto, || panic!("a base é opaca"));
    let bytes = super::doc_camadas::encode(
        &[(stack.to_data(), pose.to_data(), Some(&composto), Some(&p))],
        0,
    );
    let (lidas, _) = decode(&bytes).expect("abre");
    assert_eq!(
        lidas[0].pilha.as_ref(),
        Some(&p),
        "a pilha atravessou o ficheiro"
    );
    assert_eq!(
        lidas[0].tinta.as_ref().map(|x| x.amostras().to_vec()),
        Some(composto.amostras().to_vec())
    );
}

/// ⭐⭐ **GATE — Um v6 abre com o FUNDO da cor por vértice gravada** (`docs/3D/30`
/// §13): o v6 não guardava o fundo e lia-o da cor por vértice — o degrau dá-lhe
/// essa, a peça é a composição sobre ela, e regravar leva-a no v7. CONTROLO: a
/// cor por vértice gravada NÃO é o fundo com que a pilha nasceu.
#[test]
fn um_v6_abre_com_o_fundo_da_cor_por_vertice() {
    let (mut stack, pose, t) = peca();
    let mut p = pilha_rica(&t);
    let base = p.pilha().root().last().copied().expect("base");
    p.define_opacidade(base, 0.5);
    stack.mesh_mut().colors_mut()[0] = [0.95, 0.9, 0.1];
    let gravada = stack.mesh().colors().expect("cor").to_vec();
    assert_ne!(
        gravada.as_slice(),
        p.fundo(),
        "CONTROLO: a cor gravada não é o fundo de nascença"
    );
    let c = CamadasDoc::da_pilha(&p);
    let v6 = super::migracao::SculptDocV6 {
        version: super::migracao::V_ANTES_DO_FUNDO,
        objects: vec![super::migracao::ObjectDocV6 {
            stack: stack.to_data(),
            pose: pose.to_data(),
            tinta: Some(super::migracao::TintaDocV6 {
                nivel: t.nivel(),
                niveis: Vec::new(),
                camadas: super::migracao::CamadasDocV6 {
                    pilha: c.pilha,
                    planos: c.planos,
                },
            }),
        }],
        active: 0,
    };
    let (lidas, _) = decode(&postcard::to_allocvec(&v6).expect("serializa")).expect("um v6 abre");
    let lida = lidas[0].pilha.as_ref().expect("a pilha do v6");
    assert_eq!(
        lida.fundo(),
        gravada.as_slice(),
        "o fundo de um v6 é a cor por vértice gravada"
    );
    let plano = lidas[0].tinta.as_ref().expect("o plano");
    let mut esperado = plano.clone();
    lida.pinta_tinta(&mut esperado, || {
        lida.fundo_semeado(lidas[0].stack.mesh(), t.nivel())
    });
    assert_eq!(
        bits3(plano.amostras()),
        bits3(esperado.amostras()),
        "a peça é a composição sobre ele"
    );
    let v7 = super::doc_camadas::encode(
        &[(stack.to_data(), pose.to_data(), Some(plano), Some(lida))],
        0,
    );
    let doc: SculptDoc = postcard::from_bytes(&v7).expect("re-lê");
    assert_eq!(doc.version, SCULPT_DOC_VERSION);
    assert_eq!(
        doc.objects[0].tinta.as_ref().expect("plano").camadas.fundo,
        gravada,
        "regravado, o v7 leva o fundo"
    );
}
