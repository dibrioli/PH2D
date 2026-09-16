//! Gates da lei do corte.

use super::*;
use ph2d_mesh::shapes;

/// Raios de uma vista ORTOGRÁFICA a olhar `−z`: o ponto de ecrã é o `(x, y)`.
fn raios_orto(anel: &[[f32; 2]]) -> Vec<Ray> {
    anel.iter()
        .map(|p| Ray::new([p[0], p[1], 10.0], [0.0, 0.0, -1.0]))
        .collect()
}

/// Raios de uma vista em PERSPECTIVA, com o olho a `dist` em `+z`.
fn raios_persp(anel: &[[f32; 2]], dist: f32) -> Vec<Ray> {
    anel.iter()
        .map(|p| {
            let olho = [0.0, 0.0, dist];
            let d = [p[0] - olho[0], p[1] - olho[1], -olho[2]];
            Ray::new(olho, d)
        })
        .collect()
}

fn plano() -> Plano {
    Plano {
        origem: [0.0, 0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    }
}

/// Um **C** — côncavo de propósito (espec §7.3).
fn ce() -> Vec<[f32; 2]> {
    vec![
        [-0.8, -0.8],
        [0.8, -0.8],
        [0.8, -0.4],
        [-0.3, -0.4],
        [-0.3, 0.4],
        [0.8, 0.4],
        [0.8, 0.8],
        [-0.8, 0.8],
    ]
}

fn caixa() -> Vec<[f32; 2]> {
    vec![[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
}

/// **As contagens da espec §7.1, e a fórmula confere.**
#[test]
fn o_prisma_tem_2n_vertices_e_2n_menos_2_mais_2n_faces() {
    let bola = shapes::uv_sphere(12, 16, 1.0);
    for anel in [caixa(), ce()] {
        let n = anel.len();
        let p = prisma(
            &anel,
            &raios_orto(&anel),
            &plano(),
            &bola,
            Profundidade::DaPeca,
            Paredes::Fixas,
            Resolucao::Minima,
        )
        .expect("o prisma");
        assert_eq!(p.vert_count(), 2 * n, "n = {n}: vértices");
        assert_eq!(p.face_count(), 2 * (n - 2) + 2 * n, "n = {n}: faces");
    }
}

/// ⭐⭐⭐ **A COSTURA COM QUEM CORTA: o prisma sai FECHADO.**
///
/// ⚠️ Não é higiene — a [`ph2d_mesh_bool::corta`] **recusa** uma lâmina aberta,
/// em voz alta, porque o motor dela devolveria malha vazia a dizer «sem erro».
/// *Um prisma com bordo é um corte que nunca acontece, e o artista lê isso como
/// a ferramenta partida.*
#[test]
fn o_prisma_fecha_e_por_isso_serve_de_lamina() {
    let bola = shapes::uv_sphere(12, 16, 1.0);
    for anel in [caixa(), ce()] {
        for paredes in [Paredes::Fixas, Paredes::Projectadas] {
            let p = prisma(
                &anel,
                &raios_persp(&anel, 4.0),
                &plano(),
                &bola,
                Profundidade::DaPeca,
                paredes,
                Resolucao::Minima,
            )
            .expect("o prisma");
            assert_eq!(
                ph2d_mesh::border_edges(&p),
                0,
                "n = {}, {paredes:?}: o prisma tem BORDO — a porta do corte recusá-lo-ia",
                anel.len()
            );
        }
    }
}

/// ⭐⭐⭐ **O SENTIDO DO DESENHO É IRRELEVANTE — e a saída é idêntica AO BIT.**
///
/// ⚠️⚠️ É a armadilha mais cara desta lei (espec §8): sem isto, metade dos gestos
/// entrega um volume com o dentro e o fora trocados — e uma diferença com o
/// operando invertido **não falha**, ela devolve o **complemento**. A ferramenta
/// faria o contrário do pedido, de forma **intermitente**, conforme o sentido do
/// gesto.
///
/// ⚠️ E o anel é o **C côncavo**, que exercita a §7.3 ao mesmo tempo.
#[test]
fn o_mesmo_c_desenhado_nos_dois_sentidos_da_a_mesma_saida_ao_bit() {
    let bola = shapes::uv_sphere(12, 16, 1.0);
    let horario = ce();
    let mut anti = horario.clone();
    anti.reverse();

    let saida = |anel: &[[f32; 2]]| {
        let p = prisma(
            anel,
            &raios_orto(anel),
            &plano(),
            &bola,
            Profundidade::DaPeca,
            Paredes::Fixas,
            Resolucao::Minima,
        )
        .expect("o prisma");
        let mut bits: Vec<[u32; 3]> = p
            .positions()
            .iter()
            .map(|v| [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()])
            .collect();
        bits.sort_unstable();
        (bits, volume_com_sinal(&p))
    };

    let (bits_h, vol_h) = saida(&horario);
    let (bits_a, vol_a) = saida(&anti);
    assert_eq!(bits_h, bits_a, "os dois sentidos deram vértices diferentes");
    assert!(
        vol_h > 0.0,
        "o volume tem de ser POSITIVO (normais para fora), e deu {vol_h}"
    );
    assert!(
        (vol_h - vol_a).abs() < 1e-9,
        "os dois sentidos deram volumes diferentes: {vol_h} contra {vol_a}"
    );
}

/// **§7.2 — `Fixas` é independente da vista; `Projectadas` deixa a perspectiva
/// passar, e a divergência CRESCE com ela.**
///
/// ⚠️ **Um gate que compare os dois modos em ORTOGRÁFICA não afirma nada** — a
/// espec di-lo, e a primeira metade deste gate é exactamente esse controlo.
#[test]
fn as_paredes_fixas_ignoram_a_vista_e_as_projectadas_nao() {
    let bola = shapes::uv_sphere(12, 16, 1.0);
    let anel = caixa();
    let corre = |raios: &[Ray], paredes| {
        prisma(
            &anel,
            raios,
            &plano(),
            &bola,
            Profundidade::DaPeca,
            paredes,
            Resolucao::Minima,
        )
        .expect("o prisma")
        .positions()
        .to_vec()
    };

    let d = |a: &[[f32; 3]], b: &[[f32; 3]]| {
        a.iter()
            .zip(b)
            .map(|(x, y)| (0..3).map(|k| (x[k] - y[k]).abs()).fold(0.0f32, f32::max))
            .fold(0.0f32, f32::max)
    };

    // (1) O CONTROLO: em ortográfica os dois modos coincidem.
    //
    // ⚠️ **A barra não é igualdade AO BIT, e a razão é medida:** as duas rotas
    // chegam ao mesmo ponto por aritméticas diferentes — uma soma `frente +
    // eixo × (trás − frente)`, a outra resolve o cruzamento do raio com o plano
    // de trás —, e em `f32` isso deixa **1 ULP** de resíduo (`1,0210001` contra
    // `1,0209999`, medido). *Exigir o bit aqui mediria a ordem das operações e
    // não a lei;* a divergência que a §7.2 descreve é `10 000×` maior, e está
    // medida no passo (3).
    let orto = raios_orto(&anel);
    let residuo = d(
        &corre(&orto, Paredes::Fixas),
        &corre(&orto, Paredes::Projectadas),
    );
    assert!(
        residuo < 1e-6,
        "em ortográfica os dois modos TÊM de coincidir, e diferem {residuo:.3e}"
    );

    // (2) ⭐ O QUE `Fixas` GARANTE: o anel de TRÁS é o da frente TRANSLADADO
    // pelo eixo ⇒ as paredes são **paralelas** e a secção é constante ao longo
    // do varrimento.
    //
    // ⛔⛔ **A 1.ª redacção deste gate afirmava outra coisa — que a saída de
    // `Fixas` «não se mexe quando a câmara se aproxima» — e a medição
    // REFUTOU-A** (`0,5638` a `8,0` contra `0,6963` a `2,6`). E tinha de
    // refutar: num gesto de ECRÃ o anel da FRENTE é desprojectado, logo em
    // perspectiva ele segue o cone **nos dois modos**. *O que `Fixas` fixa são
    // as PAREDES, não o tamanho do prisma* — e é isso que este passo mede.
    let perto = raios_persp(&anel, 2.6);
    let p = prisma(
        &anel,
        &perto,
        &plano(),
        &bola,
        Profundidade::DaPeca,
        Paredes::Fixas,
        Resolucao::Minima,
    )
    .expect("o prisma");
    let v = p.positions();
    let n = anel.len();
    let desloc = [v[n][0] - v[0][0], v[n][1] - v[0][1], v[n][2] - v[0][2]];
    for i in 0..n {
        for k in 0..3 {
            assert!(
                (v[n + i][k] - v[i][k] - desloc[k]).abs() < 1e-5,
                "`Fixas`: o vértice {i} de trás não é o da frente TRANSLADADO — \
                 as paredes deixaram de ser paralelas"
            );
        }
    }

    // (3) `Projectadas` ALARGA com a distância — a secção de trás é maior que a
    // da frente, e é isso a conicidade.
    let q = prisma(
        &anel,
        &perto,
        &plano(),
        &bola,
        Profundidade::DaPeca,
        Paredes::Projectadas,
        Resolucao::Minima,
    )
    .expect("o prisma");
    let w = q.positions();
    let largura = |m: &[[f32; 3]], base: usize| {
        (m[base + 1][0] - m[base][0])
            .abs()
            .max((m[base + 1][1] - m[base][1]).abs())
    };
    // ⚠️ **`frente`/`trás` são o MÍNIMO e o MÁXIMO ao longo do EIXO** (espec
    // §6.1) — **não** «perto» e «longe» da câmara. Aqui o eixo aponta para o
    // olho, logo a `frente` é a ponta mais **distante** dele. ⇒ a propriedade
    // diz-se pela distância ao OLHO, que é o que a torna legível e independente
    // da escolha do eixo. *A 1.ª redacção deste passo escreveu `trás > frente` e
    // mediu `1,3927` contra `0,6073`: o cone estava certo e a minha leitura do
    // vocabulário é que não.*
    let olho = [0.0f32, 0.0, 2.6];
    let dist = |base: usize| {
        let c = w[base];
        ((c[0] - olho[0]).powi(2) + (c[1] - olho[1]).powi(2) + (c[2] - olho[2]).powi(2)).sqrt()
    };
    let (la, lb, da, db) = (largura(w, 0), largura(w, n), dist(0), dist(n));
    let (perto_l, longe_l) = if da < db { (la, lb) } else { (lb, la) };
    assert!(
        longe_l > perto_l * 1.05,
        "`Projectadas` tinha de ALARGAR com a distância ao olho: {perto_l:.4} perto, {longe_l:.4} longe"
    );

    // (4) E a divergência entre os dois modos CRESCE com a força da perspectiva.
    let longe = raios_persp(&anel, 8.0);
    let corre = |raios: &[Ray], paredes| {
        prisma(
            &anel,
            raios,
            &plano(),
            &bola,
            Profundidade::DaPeca,
            paredes,
            Resolucao::Minima,
        )
        .expect("o prisma")
        .positions()
        .to_vec()
    };
    let (dl, dp) = (
        d(
            &corre(&longe, Paredes::Projectadas),
            &corre(&longe, Paredes::Fixas),
        ),
        d(
            &corre(&perto, Paredes::Projectadas),
            &corre(&perto, Paredes::Fixas),
        ),
    );
    assert!(
        dp > dl && dl > 0.0,
        "a divergência tinha de CRESCER com a perspectiva: {dl:.4} a `8,0` contra {dp:.4} a `2,6`"
    );
}

/// **§6.1 — no regime de omissão o volume ATRAVESSA a peça, por construção.**
#[test]
fn no_regime_da_peca_o_volume_atravessa_sempre() {
    let bola = shapes::uv_sphere(12, 16, 1.0);
    let anel = caixa();
    let p = prisma(
        &anel,
        &raios_orto(&anel),
        &plano(),
        &bola,
        Profundidade::DaPeca,
        Paredes::Fixas,
        Resolucao::Minima,
    )
    .expect("o prisma");
    let z = |m: &Mesh| {
        m.positions()
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v[2]), hi.max(v[2]))
            })
    };
    let (pz0, pz1) = z(&p);
    let (bz0, bz1) = z(&bola);
    assert!(
        pz0 < bz0 && pz1 > bz1,
        "o prisma [{pz0:.4}; {pz1:.4}] tinha de conter a peça [{bz0:.4}; {bz1:.4}] no eixo"
    );
}

/// **§6.1 — o enchimento tem DOIS termos, e o ABSOLUTO cobre a peça degenerada.**
///
/// ⚠️ Sem ele, uma peça achatada no eixo (extensão zero) daria um prisma de
/// espessura zero — um corte que não corta. *O relativo escala com a peça; o
/// absoluto existe para o caso em que não há peça que escale.*
#[test]
fn o_enchimento_salva_a_peca_achatada_no_eixo() {
    // Uma peça com extensão ZERO ao longo de `z`.
    let chata = Mesh::from_parts(
        vec![[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]],
        vec![Face::tri(0, 1, 2)],
    )
    .expect("o triângulo chato");
    let anel = caixa();
    let p = prisma(
        &anel,
        &raios_orto(&anel),
        &plano(),
        &chata,
        Profundidade::DaPeca,
        Paredes::Fixas,
        Resolucao::Minima,
    )
    .expect("o prisma tinha de existir — o termo absoluto do enchimento é para isto");
    let esp = p
        .positions()
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(v[2]), hi.max(v[2]))
        });
    assert!(
        esp.1 - esp.0 > 1e-4,
        "a espessura ficou em {} — o termo absoluto não entrou",
        esp.1 - esp.0
    );
}

/// **§6.2 — o regime do cursor é uma FATIA, e com raio pequeno ela não atravessa.**
///
/// ⚠️ E **não há recusa**: sai um bolso. A espec é explícita — *nada de especial
/// acontece*.
#[test]
fn o_regime_do_cursor_e_uma_fatia_que_pode_nao_atravessar() {
    let bola = shapes::uv_sphere(12, 16, 1.0);
    let anel = caixa();
    let p = prisma(
        &anel,
        &raios_orto(&anel),
        &plano(),
        &bola,
        Profundidade::DoCursor {
            medio: 0.0,
            raio: 0.15,
        },
        Paredes::Fixas,
        Resolucao::Minima,
    )
    .expect("o prisma");
    let (lo, hi) = p
        .positions()
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(v[2]), hi.max(v[2]))
        });
    assert!(
        (hi - lo - 0.30).abs() < 1e-5,
        "a fatia tinha de ter `2 × 0,15` de espessura, e tem {}",
        hi - lo
    );
    assert!(
        lo > -1.0 && hi < 1.0,
        "a fatia [{lo:.3}; {hi:.3}] tinha de ficar DENTRO da peça — é o bolso"
    );
}

/// ⛔ **Raio zero é EspessuraNula** — a condição de fronteira da espec §6.2.
///
/// ⚠️ Ela existe porque o raio do cursor **só está definido** quando o gesto
/// começa sobre a superfície: começando fora, quem chama tem de o derivar do
/// pincel. *Sem esta recusa, um `0` esquecido dá um corte que não corta e o
/// artista não sabe porquê.*
#[test]
fn raio_zero_e_recusado_e_diz_a_cura() {
    let bola = shapes::uv_sphere(12, 16, 1.0);
    let anel = caixa();
    let r = prisma(
        &anel,
        &raios_orto(&anel),
        &plano(),
        &bola,
        Profundidade::DoCursor {
            medio: 0.0,
            raio: 0.0,
        },
        Paredes::Fixas,
        Resolucao::Minima,
    );
    assert_eq!(r.err(), Some(Recusa::EspessuraNula));
    assert!(Recusa::EspessuraNula.porque().contains("pincel"));
}

/// **As recusas de chamada, e o gesto degenerado.**
#[test]
fn um_gesto_sem_area_e_um_anel_curto_sao_recusados() {
    let bola = shapes::uv_sphere(12, 16, 1.0);
    let curto = vec![[0.0, 0.0], [1.0, 0.0]];
    assert_eq!(
        prisma(
            &curto,
            &raios_orto(&curto),
            &plano(),
            &bola,
            Profundidade::DaPeca,
            Paredes::Fixas,
            Resolucao::Minima
        )
        .err(),
        Some(Recusa::GestoDegenerado)
    );

    // Um anel de área nula: três pontos colineares.
    let risco = vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
    assert_eq!(
        prisma(
            &risco,
            &raios_orto(&risco),
            &plano(),
            &bola,
            Profundidade::DaPeca,
            Paredes::Fixas,
            Resolucao::Minima
        )
        .err(),
        Some(Recusa::GestoDegenerado)
    );

    // Raios a menos: defeito de chamada, e ele é NOMEADO à parte.
    let anel = caixa();
    assert_eq!(
        prisma(
            &anel,
            &raios_orto(&anel)[..3],
            &plano(),
            &bola,
            Profundidade::DaPeca,
            Paredes::Fixas,
            Resolucao::Minima
        )
        .err(),
        Some(Recusa::RaiosNaoBatem)
    );
}

/// **§6.1 — o termo RELATIVO do enchimento escala com a peça.**
///
/// ⚠️ **O gate irmão cobre o termo absoluto e este cobre o outro**, e só os dois
/// juntos afirmam a frase da espec (*«os dois termos são necessários»*). A razão
/// do relativo é que a margem tem de continuar a **significar** alguma coisa
/// numa peça grande: `0,001` sobre uma peça de `1000` é ruído de `f32`, e a
/// margem existe para afastar as tampas das faces — a configuração em que um
/// solucionador exacto é mais frágil.
#[test]
fn o_enchimento_e_proporcional_a_peca() {
    let anel = caixa();
    let margem = |raio: f32| {
        let bola = shapes::uv_sphere(12, 16, raio);
        let p = prisma(
            &anel,
            &raios_orto(&anel),
            &plano(),
            &bola,
            Profundidade::DaPeca,
            Paredes::Fixas,
            Resolucao::Minima,
        )
        .expect("o prisma");
        let (lo, hi) = p
            .positions()
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v[2]), hi.max(v[2]))
            });
        // O que o prisma tem ALÉM da peça, de cada lado.
        (hi - lo - 2.0 * raio) * 0.5
    };
    let (pequena, grande) = (margem(1.0), margem(100.0));
    assert!(
        grande > pequena * 50.0,
        "a margem tinha de ESCALAR com a peça: {pequena:.5} a raio 1 contra \
         {grande:.5} a raio 100 — o termo relativo não entrou"
    );
}

/// ⭐⭐ **O volume sai POSITIVO com o eixo nos DOIS sentidos.**
///
/// ⚠️ É o gate da inversão pelo sinal do volume. O enrolamento do ANEL já é
/// ordenado antes do varrimento, mas isso não chega: **virar o EIXO** também
/// inverte o prisma, e aí não há anel nenhum para reordenar. *São duas causas
/// para o mesmo defeito, e cada uma tem a sua cura.*
///
/// ⛔ E o defeito que elas impedem não é geometria visivelmente errada: uma
/// diferença com o operando invertido devolve o **complemento** — apaga tudo
/// *menos* o que se queria apagar.
#[test]
fn o_volume_sai_positivo_com_o_eixo_nos_dois_sentidos() {
    let bola = shapes::uv_sphere(12, 16, 1.0);
    let anel = caixa();
    for normal in [[0.0, 0.0, 1.0], [0.0, 0.0, -1.0]] {
        let p = prisma(
            &anel,
            &raios_orto(&anel),
            &Plano {
                origem: [0.0, 0.0, 0.0],
                normal,
            },
            &bola,
            Profundidade::DaPeca,
            Paredes::Fixas,
            Resolucao::Minima,
        )
        .expect("o prisma");
        let v = volume_com_sinal(&p);
        assert!(
            v > 0.0,
            "com o eixo {normal:?} o volume saiu {v} — o prisma está do avesso, \
             e a diferença devolveria o COMPLEMENTO"
        );
    }
}

/// **A RESOLUÇÃO da malha do prisma** — assunto próprio, ficheiro próprio.
///
/// ⚠️ O corte saiu do tecto de LOC deste ficheiro e foi **por
/// responsabilidade**, nunca por uma entrada de isenção: aqui em cima mede-se a
/// **FORMA** do volume (contagens, enrolamento, faixa, paredes, tampas), e ali
/// a **DENSIDADE** da malha que ele entrega.
#[path = "lib_resolucao_tests.rs"]
mod resolucao;
