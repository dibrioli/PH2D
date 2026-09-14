//! Os gates do **INDICADOR** da pose — ver [`crate::pose_previa`].
//!
//! ⚠️ **O que só aqui se pode afirmar** é o que separa este indicador do do
//! alvo: que ele **não reconstrói** quando nada mudou, que a chave dele traz
//! exactamente o que a construção lê, e — o mais importante — que o osso que
//! ele desenha é o **mesmo** que o pen-down vai construir. *Um indicador que
//! mostra outra cadeia é pior que nenhum: o artista aprende o gesto errado.*

use ph2d_mesh::Mesh;

use crate::{Brush, Dab, SculptStroke, Symmetry, Verb};

fn esfera() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(32, 48, 1.0)
}

/// ⚠️ **O raio é GRANDE de propósito.** Numa esfera lisa com raio pequeno a
/// franja é um anel quase simétrico à volta do cursor e o primeiro segmento
/// nasce com comprimento de ruído (§11.1) — *um gate sobre uma cadeia
/// degenerada mede o nada e fica verde*. Com o raio a `0,8` da peça a franja
/// está longe, o pivô cai fundo, e há osso para comparar. O
/// [`o_osso_do_arnes_nao_e_degenerado`] é o controlo positivo disto.
fn pincel() -> Brush {
    Brush {
        verb: Verb::Pose,
        radius: 0.8,
        strength: 1.0,
        ..Brush::default()
    }
}

const CURSOR: [f32; 3] = [0.0, 0.0, 1.0];

fn comprimento(o: &crate::PoseOsso) -> f32 {
    let d = [
        o.cabeca[0] - o.origem[0],
        o.cabeca[1] - o.origem[1],
        o.cabeca[2] - o.origem[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// ⭐ **O controlo positivo do arnês** — sem ele todo gate deste ficheiro pode
/// estar a medir uma cadeia degenerada e a passar por isso.
#[test]
fn o_osso_do_arnes_nao_e_degenerado() {
    let malha = esfera();
    let mut s = SculptStroke::default();
    let ossos = s.pose_ossos(&malha, &pincel(), Symmetry::default(), CURSOR);
    assert_eq!(ossos.len(), 1, "um segmento por omissão");
    let c = comprimento(&ossos[0]);
    assert!(
        c > 0.1,
        "o osso do arnes mede {c:e} — a cadeia nasceu degenerada e os gates \
         deste ficheiro mediriam o nada"
    );
    assert!(!s.pose_previa_inerte(), "o arnes nasceu inerte");
}

/// ⭐⭐⭐ **A vantagem sobre o alvo, GATEADA: sobrevoar não reconstrói nada.**
///
/// O alvo reconstrói a cadeia inteira **a cada movimento do rato**, mesmo sem
/// traço nenhum, só para desenhar este indicador — é a causa registada em
/// quatro relatos públicos de o editor engasgar em malha densa (espec §10).
///
/// ⚠️ **Sem contador isto não é observável:** mover a construção para dentro do
/// laço de quadros dá exactamente a mesma figura, só mais lenta. *Uma vantagem
/// escrita num cabeçalho é promessa; uma com contador é propriedade.*
#[test]
fn o_indicador_nao_reconstroi_quando_nada_muda() {
    let malha = esfera();
    let b = pincel();
    let mut s = SculptStroke::default();
    for _ in 0..30 {
        let ossos = s.pose_ossos(&malha, &b, Symmetry::default(), CURSOR);
        assert!(!ossos.is_empty(), "o indicador devolveu vazio");
    }
    assert_eq!(
        s.pose_previa.construcoes, 1,
        "30 quadros de sobrevoo parado construiram {} cadeias",
        s.pose_previa.construcoes
    );
    assert_eq!(
        s.pose_previa.adjacencias, 1,
        "30 quadros construiram {} adjacencias — o `O(V²)` das pecas soltas \
         seria pago outra vez em cada uma",
        s.pose_previa.adjacencias
    );
    // ⭐ **O controlo negativo:** mover o cursor TEM de reconstruir, senão este
    // gate estaria a premiar um indicador congelado.
    //
    // ⚠️⚠️ **Sessenta chamadas, e a contagem é o gate.** O orçamento mede-se em
    // QUADROS e uma construção cara compra silêncio: com uma chamada só, este
    // controlo mediria o orçamento em vez da chave — e reprovaria sobre produto
    // correcto no dia em que a máquina estivesse sob carga. Sessenta chamadas
    // são um segundo a 60 fps, que cobre uma construção de `100 ms`; e como a
    // chave não volta a mudar, a conta certa continua a ser **exactamente uma**
    // construção nova.
    let outro = [0.2, 0.1, 0.95];
    for _ in 0..60 {
        s.pose_ossos(&malha, &b, Symmetry::default(), outro);
    }
    assert_eq!(
        s.pose_previa.construcoes, 2,
        "o cursor andou e o indicador construiu {} cadeias — ou ficou \
         congelado, ou reconstruiu-se em cada quadro",
        s.pose_previa.construcoes
    );
}

/// ⭐⭐ **A chave traz o que a construção LÊ, e nada mais.**
///
/// As suavizações mexem nos **pesos** (§4), e um osso não tem peso: mudá-las
/// não pode mover a figura. ⚠️ **As duas metades são precisas** — a primeira
/// mede que a saída de facto não muda (senão excluí-la da chave seria um
/// defeito), a segunda que a chave o sabe (senão pagávamos uma construção por
/// nada).
#[test]
fn a_suavizacao_do_peso_nao_move_o_osso_e_a_chave_sabe_disso() {
    let malha = esfera();
    let mut base = pincel();
    base.pose.suavizacoes_do_peso = 2;
    let mut outra = base.clone();
    outra.pose.suavizacoes_do_peso = 40;

    // (a) a SAÍDA não muda — duas construções frescas, não a cache.
    let mut s = SculptStroke::default();
    let a = s
        .pose_ossos(&malha, &base, Symmetry::default(), CURSOR)
        .to_vec();
    s.pose_previa.esquecer();
    let b = s
        .pose_ossos(&malha, &outra, Symmetry::default(), CURSOR)
        .to_vec();
    assert_eq!(
        a, b,
        "a suavizacao moveu o osso — ela entra na chave, entao"
    );

    // (b) e a CHAVE sabe: trocá-la não paga uma construção.
    let mut t = SculptStroke::default();
    t.pose_ossos(&malha, &base, Symmetry::default(), CURSOR);
    for _ in 0..60 {
        t.pose_ossos(&malha, &outra, Symmetry::default(), CURSOR);
    }
    assert_eq!(
        t.pose_previa.construcoes, 1,
        "trocar as suavizacoes pagou uma construcao que nao muda nada"
    );
    // ⭐ O controlo: um knob que a construção LÊ tem de pagar. (As 60 chamadas
    // são o orçamento, ver o irmão acima.)
    let mut com_segmentos = base.clone();
    com_segmentos.pose.segmentos = 3;
    for _ in 0..60 {
        t.pose_ossos(&malha, &com_segmentos, Symmetry::default(), CURSOR);
    }
    assert_eq!(
        t.pose_previa.construcoes, 2,
        "mudar os SEGMENTOS deu {} construcoes — a chave estaria cega ao que a \
         §3 de facto le",
        t.pose_previa.construcoes
    );
}

/// ⭐⭐⭐ **O INDICADOR NÃO MENTE: o osso que ele desenha é o que o pen-down
/// constrói.**
///
/// ⚠️ **É a única propriedade que justifica desenhá-lo.** Ele e o gesto entram
/// pela mesma porta ([`SculptStroke::pose_ossos`]) e pela mesma lei
/// ([`ph2d_pose::Cadeia::ossos`]) — este gate mede que a costura entre as duas
/// metades (a cache do sobrevoo e a sessão do traço) não as separa.
///
/// ⚠️ A barra não é o bit: o traço fotografa as posições no pen-down e resolve
/// a cadeia com arrasto nulo, o que passa a origem por `alvo − d·comprimento`.
#[test]
fn o_indicador_da_o_mesmo_osso_que_o_traco_constroi() {
    let mut malha = esfera();
    let b = pincel();
    let mut s = SculptStroke::default();
    let previsto = s
        .pose_ossos(&malha, &b, Symmetry::default(), CURSOR)
        .to_vec();

    // O pen-down no MESMO sítio, com arrasto nulo: a cadeia é construída pelo
    // traço, e o indicador passa a devolver a viva.
    s.begin(&malha);
    let olho = [0.0, 0.0, -1.0];
    s.dab(
        &mut malha,
        &b,
        &Dab::pulling(CURSOR, b.radius, olho, [0.0, 0.0, 0.0]),
        Symmetry::default(),
    );
    let vivo = s
        .pose_ossos(&malha, &b, Symmetry::default(), CURSOR)
        .to_vec();

    assert_eq!(
        previsto.len(),
        vivo.len(),
        "o indicador prometeu {} segmentos e o traco construiu {}",
        previsto.len(),
        vivo.len()
    );
    for (k, (p, v)) in previsto.iter().zip(&vivo).enumerate() {
        for (a, b) in [(p.origem, v.origem), (p.cabeca, v.cabeca)] {
            let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
            assert!(
                d < 1e-5,
                "segmento {k}: o indicador desenhou {a:?} e o traco construiu \
                 {b:?} (desvio {d:e}) — o artista aprenderia o gesto errado"
            );
        }
    }
}

/// ⚠️ **Um traço novo faz o indicador esquecer TUDO**, adjacência incluída — é
/// o único momento em que a malha pode ter mudado de uma forma que a chave não
/// vê (as ligações entre peças dependem das POSIÇÕES).
#[test]
fn um_traco_novo_faz_o_indicador_esquecer_a_adjacencia() {
    let malha = esfera();
    let b = pincel();
    let mut s = SculptStroke::default();
    s.pose_ossos(&malha, &b, Symmetry::default(), CURSOR);
    assert_eq!(s.pose_previa.adjacencias, 1);
    s.begin(&malha);
    s.pose_ossos(&malha, &b, Symmetry::default(), CURSOR);
    assert_eq!(
        s.pose_previa.adjacencias, 2,
        "o `begin` nao deitou fora a adjacencia — depois de um traco que \
         deforma a malha, o emparelhamento entre pecas descreveria as posicoes \
         de antes"
    );
}

/// ⛔ **O indicador é do verbo de POSE e de mais nenhum.**
///
/// Sem isto ele construiria uma cadeia — `O(V)` na malha inteira — a cada
/// quadro de sobrevoo de **todos** os outros 26 verbos, que é literalmente o
/// custo que este módulo existe para não pagar.
#[test]
fn nenhum_outro_verbo_paga_o_indicador() {
    let malha = esfera();
    let mut s = SculptStroke::default();
    for verbo in Verb::ALL {
        if verbo == Verb::Pose {
            continue;
        }
        let b = Brush {
            verb: verbo,
            ..pincel()
        };
        let ossos = s.pose_ossos(&malha, &b, Symmetry::default(), CURSOR);
        assert!(ossos.is_empty(), "{verbo:?} devolveu osso");
    }
    assert_eq!(
        s.pose_previa.construcoes, 0,
        "os outros verbos pagaram {} construcoes",
        s.pose_previa.construcoes
    );
}

/// ⭐⭐ **Durante o traço o osso é o VIVO — ele dobra com a mão.**
///
/// ⚠️⚠️ **Este gate existe porque o irmão acima não o pode dar:** com arrasto
/// nulo a cadeia viva e uma cadeia reconstruída dão a **mesma** figura, logo a
/// mutação que trocasse a sessão por uma construção nova sobreviveria a ele.
/// Com arrasto a sério as duas separam-se por construção — uma cadeia
/// reconstruída sob um cursor parado volta ao repouso, e a viva está dobrada.
///
/// *É a lição do corpus: uma propriedade que duas implementações partilham no
/// ponto neutro só se mede fora dele.*
#[test]
fn o_osso_vivo_segue_o_arrasto() {
    let mut malha = esfera();
    let b = pincel();
    let mut s = SculptStroke::default();
    let repouso = s
        .pose_ossos(&malha, &b, Symmetry::default(), CURSOR)
        .to_vec();

    s.begin(&malha);
    let olho = [0.0, 0.0, -1.0];
    let arrasto = [0.0, 0.30, 0.0];
    for k in 1..=4 {
        let t = f32::from(u8::try_from(k).unwrap_or(1)) / 4.0;
        s.dab(
            &mut malha,
            &b,
            &Dab::pulling(
                CURSOR,
                b.radius,
                olho,
                [arrasto[0] * t, arrasto[1] * t, arrasto[2] * t],
            ),
            Symmetry::default(),
        );
    }
    // O cursor **não anda** — só o arrasto. Uma cadeia reconstruída aqui
    // devolveria o repouso.
    let vivo = s
        .pose_ossos(&malha, &b, Symmetry::default(), CURSOR)
        .to_vec();
    let d = (0..3)
        .map(|i| (vivo[0].cabeca[i] - repouso[0].cabeca[i]).powi(2))
        .sum::<f32>()
        .sqrt();
    assert!(
        d > 0.1,
        "a cabeca do osso andou {d:e} com um arrasto de 0,30 — o indicador \
         esta' a reconstruir a cadeia em vez de ler a do traco"
    );
    // ⭐ E a dobradiça **fica**: é isso que o pincel promete.
    let pivo = (0..3)
        .map(|i| (vivo[0].origem[i] - repouso[0].origem[i]).powi(2))
        .sum::<f32>()
        .sqrt();
    assert!(
        pivo < d * 0.5,
        "a dobradica andou {pivo:e} contra {d:e} da cabeca — ela tem de ficar \
         mais parada que a mao, senao a figura nao mostra uma articulacao"
    );
}
