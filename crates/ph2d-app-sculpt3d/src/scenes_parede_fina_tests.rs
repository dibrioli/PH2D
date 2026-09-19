//! Os gates da cena `=50`.

use super::*;

/// ⭐⭐⭐ **A CENA CONTÉM O FENÓMENO — e o gate mede-o pela porta do produto.**
///
/// ⚠️⚠️ *Uma cena que ensina o CONTRÁRIO do que acontece é pior que uma cena
/// ausente* (CLAUDE.md §5.0), e esta cena promete duas coisas ao dono no passo
/// (4) e no passo (5). As duas têm de ser verdade **nesta malha**, com **este**
/// pincel:
/// * com a máscara armada as costas ficam quietas;
/// * com ela desarmada elas mexem-se — senão o CONTROLO do roteiro manda-o
///   procurar uma diferença que não existe.
#[test]
fn a_cena_contem_o_defeito_e_a_cura() {
    use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};
    let repouso = barbatana();
    // ⚠️ O raio é o que o roteiro manda o dono pôr: **maior que a peça é
    // grossa**. Com um pincel pequeno o fenómeno não existe, e o gate ficaria
    // verde sobre uma cena que não ensina nada.
    let raio = 0.40;
    assert!(
        raio > 3.0 * ESPESSURA,
        "o raio do gate tem de ser bem maior que a espessura, senao a cena nao \
         contem o fenomeno"
    );
    // ⛔⛔ **A DISTÂNCIA À BEIRA É O GATE, e a 1.ª redacção estava fora da
    // banda.** Ela carimbava a meio caminho (`d = 0,50`) e ficava VERDE com a
    // lei da razão APAGADA — ali a superfície mede `1,06` contra o tecto
    // absoluto de `2,00 × R = 0,80`, logo **a lei antiga já cortava**, e o gate
    // media uma cura que existia antes desta wave.
    // ⇒ o carimbo mora em `d = 0,20`, o meio da banda MEDIDA
    // (`diag_onde_a_razao_e_a_unica_que_cura`: só a razão cura de `0,10` a
    // `0,35`), e a 4.ª metade abaixo afirma que ele lá está.
    const DIST_DA_BEIRA: f32 = 0.20;
    let centro = [MEIO - DIST_DA_BEIRA, 0.0, ESPESSURA * 0.5];
    let superficie = 2.0 * DIST_DA_BEIRA + ESPESSURA;
    assert!(
        superficie <= ph2d_sculpt3d::ALCANCE_TECTO * raio,
        "a fixtura saiu da banda: a superficie mede {superficie:.3} contra o tecto \
         ABSOLUTO de {:.3} -- ali a lei ANTIGA ja' cortava, e este gate passaria \
         a medir uma cura que ja' existia",
        ph2d_sculpt3d::ALCANCE_TECTO * raio
    );
    assert!(
        superficie / ESPESSURA > ph2d_sculpt3d::RAZAO_MAXIMA,
        "a fixtura esta' aquem da razao: {:.2} contra {:.2} -- aqui NENHUMA das \
         duas leis corta, e o passo (4) do roteiro seria falso",
        superficie / ESPESSURA,
        ph2d_sculpt3d::RAZAO_MAXIMA
    );

    let d = |a: [f32; 3], b: [f32; 3]| {
        let v = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
    };
    let perto = |p: [f32; 3]| -> usize {
        repouso
            .positions()
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                d(**a, p)
                    .partial_cmp(&d(**b, p))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map_or(0, |(i, _)| i)
    };
    let frente = perto(centro);
    let costas = perto([centro[0], centro[1], -ESPESSURA * 0.5]);

    let bate = |mascara: bool| -> (f32, f32) {
        let mut m = repouso.clone();
        let b = Brush {
            verb: Verb::Draw,
            radius: raio,
            strength: 1.0,
            surface_only: mascara,
            ..Brush::default()
        };
        let mut st = SculptStroke::default();
        st.begin(&m);
        st.dab(
            &mut m,
            &b,
            &Dab::at(centro, raio, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
        (
            d(repouso.positions()[frente], m.positions()[frente]),
            d(repouso.positions()[costas], m.positions()[costas]),
        )
    };

    let (f_com, c_com) = bate(true);
    let (_, c_sem) = bate(false);
    assert!(
        f_com > 0.5 * ESPESSURA,
        "o carimbo da cena mal move a FRENTE ({f_com:.4}) — o gate mede um no-op"
    );
    assert!(
        c_com <= 1e-6,
        "o passo (4) do roteiro promete as costas LISAS e elas moveram {c_com:.6}"
    );
    assert!(
        c_sem > 0.8 * f_com,
        "o passo (5) — o CONTROLO — promete que sem a mascara a bossa aparece dos \
         DOIS lados, e as costas so' moveram {c_sem:.4} contra {f_com:.4} da frente"
    );
}

/// ⚠️ **A peça é FINA, e a espessura é a única coisa que a cena precisa de ter.**
///
/// A lei é `(2d + t)/t > 3,5`, logo o fenómeno vive em peças cuja espessura é
/// pequena ao lado do lado delas. Este gate prende a proporção, para que ninguém
/// engrosse a barbatana a arrumar a cena e a deixe a ensinar nada.
#[test]
fn a_barbatana_e_fina_de_proposito() {
    let m = barbatana();
    let (mut zmin, mut zmax, mut xmax) = (f32::INFINITY, f32::NEG_INFINITY, 0.0f32);
    for p in m.positions() {
        zmin = zmin.min(p[2]);
        zmax = zmax.max(p[2]);
        xmax = xmax.max(p[0].abs());
    }
    let espessura = zmax - zmin;
    assert!(
        (espessura - ESPESSURA).abs() < 1e-5,
        "a espessura mudou: {espessura:.4}"
    );
    assert!(
        2.0 * xmax > 20.0 * espessura,
        "a peca deixou de ser FINA (lado {:.2} contra espessura {espessura:.3}) — \
         a cena deixa de conter o fenomeno",
        2.0 * xmax
    );
    // ⛔ E ela é FECHADA: sem a cinta da beira o caminho pela superfície não
    // existe, e a lei da razão leria `∞` em toda a face de trás — a cena
    // passaria por acidente.
    assert_eq!(
        ph2d_mesh::border_edges(&m),
        0,
        "a barbatana tem de ser fechada — sem a cinta da beira a lei da razao le' \
         `infinito` nas costas e a cena passaria por acidente"
    );
}

/// ⛔⛔ **E o gate acima chama a PORTA; este percorre a ROTA.**
///
/// ⚠️ *Um gate que chama a função em vez de percorrer o despacho afirma que a
/// peça certa EXISTE, nunca que a cena a usa* — a lição que o §24 desta linha
/// pagou com uma mutação sobrevivente (trocar o braço do selector para o default
/// deixava o gate verde). A metade que fecha isso lê o **despacho** por
/// [`include_str!`], que deixa de **compilar** se o irmão mudar de sítio.
#[test]
fn o_selector_de_malha_escolhe_a_barbatana() {
    let despacho = include_str!("scenes_mesh.rs");
    // ⚠️ A agulha é o **braço inteiro** e não os dois nomes soltos: uma mutação
    // que prefixa `false &&` deixa os dois presentes e o despacho morto, e ela
    // SOBREVIVEU à 1.ª redacção deste gate.
    assert!(
        despacho.contains(
            "    if parede_fina::parede_fina_scene() {\n        return parede_fina::barbatana();\n    }"
        ),
        "o selector de malha deixou de escolher a barbatana — a cena =50 abriria \
         na peca de fabrica, que e' GROSSA e nao contem o fenomeno"
    );
    // ⚠️ E o roteiro tem de ser ANUNCIADO: a cena nasceu uma vez com o texto
    // escrito e **nenhum chamador**, e quem o apanhou foi um aviso do
    // compilador — *um aviso mede visibilidade, nunca a lei*.
    let scripts = include_str!("scripts.rs");
    assert!(
        scripts.contains("scenes::parede_fina::announce()"),
        "o roteiro da =50 existe e ninguem o imprime"
    );
}

/// ⚠️ **SONDA** — onde é que a lei da RAZÃO de facto cura, nesta peça.
///
/// A lei antiga (só o tecto absoluto) corta quando `2d + t > 2,0 × R`; a nova
/// corta quando `(2d + t)/t > 3,5`. ⇒ **existe uma banda em que só a nova cura**,
/// e é ela que a cena tem de conter — senão o roteiro ensina uma cura que já
/// existia antes desta wave.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn diag_onde_a_razao_e_a_unica_que_cura() {
    use ph2d_sculpt3d::{ALCANCE_TECTO, Brush, Dab, RAZAO_MAXIMA, SculptStroke, Symmetry, Verb};
    let repouso = barbatana();
    let raio = 0.40f32;
    println!("\n== a banda em que só a RAZÃO cura ==");
    println!("   barbatana  lado 2,0 · espessura {ESPESSURA}  ·  pincel R = {raio:.2}");
    println!(
        "   tecto absoluto {ALCANCE_TECTO:.2} × R = {:.3}   ·   razão máxima {RAZAO_MAXIMA:.2}\n",
        ALCANCE_TECTO * raio
    );
    println!(
        "{:>7} {:>9} {:>8} | {:>9} {:>9} | {:>10}",
        "d", "superfic.", "sup/ar", "frente", "costas", "veredito"
    );
    println!("{}", "-".repeat(66));

    let d = |a: [f32; 3], b: [f32; 3]| {
        let v = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
    };
    for dist in [0.05f32, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40, 0.50] {
        let centro = [MEIO - dist, 0.0, ESPESSURA * 0.5];
        let mut m = repouso.clone();
        let b = Brush {
            verb: Verb::Draw,
            radius: raio,
            strength: 1.0,
            surface_only: true,
            ..Brush::default()
        };
        let mut st = SculptStroke::default();
        st.begin(&m);
        st.dab(
            &mut m,
            &b,
            &Dab::at(centro, raio, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
        // O ponto exactamente atrás do cursor.
        let atras = [centro[0], centro[1], -ESPESSURA * 0.5];
        let (mut i_f, mut i_t) = (0usize, 0usize);
        let (mut bf, mut bt) = (f32::INFINITY, f32::INFINITY);
        for (i, p) in repouso.positions().iter().enumerate() {
            let df = d(*p, centro);
            if df < bf {
                bf = df;
                i_f = i;
            }
            let dt = d(*p, atras);
            if dt < bt {
                bt = dt;
                i_t = i;
            }
        }
        let mov_f = d(repouso.positions()[i_f], m.positions()[i_f]);
        let mov_t = d(repouso.positions()[i_t], m.positions()[i_t]);
        let sup = 2.0 * dist + ESPESSURA;
        let razao = sup / ESPESSURA;
        let so_a_razao = sup <= ALCANCE_TECTO * raio && razao > RAZAO_MAXIMA;
        println!(
            "{dist:>7.2} {sup:>9.3} {razao:>8.2} | {mov_f:>9.4} {mov_t:>9.4} | {:>10}",
            if so_a_razao {
                "SO' A RAZAO"
            } else if razao <= RAZAO_MAXIMA {
                "nenhuma"
            } else {
                "as duas"
            }
        );
    }
}
