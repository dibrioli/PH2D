//! Os gates da [`super`] — e **cada um traz o CONTROLO**: a mesma fixtura medida
//! pela régua que já existia ([`crate::shape`]), a provar que ela não a vê.
//!
//! ⚠️ *Sem o controlo, «a régua nova acusa» é uma afirmação sobre a régua nova. A
//! razão de este módulo existir é a outra metade — que a antiga fica **verde**.*

use super::{QuadKind, SQUARENESS_DEFECT, TIP_FRACTION, WARP_DEFECT_DEG, local_shape_of};
use ph2d_mesh::Face;

fn quad(p: [[f32; 3]; 4]) -> (Vec<[f32; 3]>, Vec<Face>) {
    (p.to_vec(), vec![Face::quad(0, 1, 2, 3)])
}

/// ⭐ O caso trivial, e ele é o **controlo negativo** de tudo o resto: se um
/// quadrado perfeito acusasse alguma coisa, nenhum número acima significaria nada.
#[test]
fn um_quadrado_perfeito_nao_acusa_nada() {
    let (pos, faces) = quad([
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ]);
    let (s, per) = local_shape_of(&pos, &faces);
    assert_eq!(per[0].kind, QuadKind::Convex);
    assert!(per[0].warp_deg < 1.0e-3, "torcao {}", per[0].warp_deg);
    assert!(
        (per[0].squareness - 1.0).abs() < 1.0e-5,
        "um quadrado tem de dar 1,0 e deu {}",
        per[0].squareness
    );
    assert!(!per[0].is_defect());
    assert_eq!(s.defects, 0);
}

/// ⭐⭐⭐ **A GRAVATA — e o CONTROLO é a parte que importa.**
///
/// ⛔ A fixtura é o quad com dois vértices trocados: ele auto-intersecta, as duas
/// metades cancelam-se e no ecrã é uma face embolada. ⚠️ **E a régua que já
/// existia fica VERDE nele:** os cantos medem `45°` de desvio, que está abaixo da
/// barra de `60°` do `QuadShape`, e o aspecto é `√2`. *Uma face que se dobra sobre
/// si mesma passa em aspecto, em enviesamento e no `>60°`.*
#[test]
fn uma_gravata_e_acusada_e_a_regua_antiga_fica_verde() {
    let (pos, faces) = quad([
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ]);
    let (s, per) = local_shape_of(&pos, &faces);
    assert_eq!(per[0].kind, QuadKind::Bowtie, "a gravata tem de ser vista");
    assert!(per[0].is_defect());
    assert_eq!(s.bowties, 1);

    // ⚠️ **O CONTROLO.** Se um dia isto ficar vermelho, a régua antiga passou a
    // ver a gravata — e este módulo perdeu metade da razão de existir.
    let old = crate::shape::quad_shape_of(&pos, &faces);
    assert_eq!(
        old.skew_over_60, 0,
        "o CONTROLO caiu: a regua antiga passou a acusar a gravata pelo enviesamento"
    );
    assert_eq!(
        old.aspect_over_4, 0,
        "o CONTROLO caiu: a regua antiga passou a acusar a gravata pelo aspecto"
    );
}

/// ⭐⭐ **UM CANTO REENTRANTE não é uma gravata** — e a distinção é load-bearing.
///
/// ⛔ Tratar as duas como a mesma coisa faria a régua acusar toda a malha de uma
/// peça côncava. *Um quad em «seta» é feio e legítimo; um que se atravessa não é
/// uma superfície.*
#[test]
fn um_canto_reentrante_e_concavo_e_nao_gravata() {
    let (pos, faces) = quad([
        [0.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [1.0, 0.5, 0.0],
        [2.0, 2.0, 0.0],
    ]);
    let (_, per) = local_shape_of(&pos, &faces);
    assert_eq!(per[0].kind, QuadKind::Concave);
}

/// ⭐⭐⭐ **A TORÇÃO, e outra vez com o controlo.**
///
/// ⚠️ A fixtura é plana em três vértices e levanta o quarto: as duas metades
/// deixam de partilhar um plano. ⛔ **Os quatro cantos continuam a medir bem** —
/// esta é a segunda forma de uma face «embolada» que o `QuadShape` não vê.
#[test]
fn uma_face_torcida_e_acusada_e_a_regua_antiga_fica_verde() {
    let (pos, faces) = quad([
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 1.4],
    ]);
    let (s, per) = local_shape_of(&pos, &faces);
    assert!(
        per[0].warp_deg > WARP_DEFECT_DEG,
        "a torcao mediu {} e a barra e' {WARP_DEFECT_DEG}",
        per[0].warp_deg
    );
    assert_eq!(s.warped, 1);
    assert!(per[0].is_defect());

    let old = crate::shape::quad_shape_of(&pos, &faces);
    assert_eq!(
        old.skew_over_60, 0,
        "o CONTROLO caiu: a regua antiga passou a acusar a torcao"
    );
}

/// A lasca — área muito menor do que as arestas prometem.
#[test]
fn uma_lasca_e_acusada_pela_area() {
    let (pos, faces) = quad([
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.01, 0.0],
        [0.0, 0.01, 0.0],
    ]);
    let (s, per) = local_shape_of(&pos, &faces);
    assert!(
        per[0].squareness < SQUARENESS_DEFECT,
        "quadratura {}",
        per[0].squareness
    );
    assert_eq!(s.slivers, 1);
}

/// ⭐⭐⭐ **A LOCALIZAÇÃO — a metade que responde ao report do artista.**
///
/// ⛔ **Este gate é o que separa esta régua de um contador global.** A fixtura tem
/// **uma** face má no fim de um braço e **muitas** boas no corpo: a contagem
/// global vê `1` defeito em `N` faces, um número que se lê como ruído — e a coluna
/// radial diz que ele está **na ponta**, que é a frase que o artista usou.
#[test]
fn o_defeito_na_ponta_e_contado_como_da_ponta() {
    // Um tapete de quads bons à volta da origem…
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    for i in 0..6i32 {
        for j in 0..6i32 {
            let b = u32::try_from(pos.len()).expect("cabe");
            #[allow(clippy::cast_precision_loss)]
            let (x, y) = (i as f32, j as f32);
            pos.extend_from_slice(&[
                [x, y, 0.0],
                [x + 1.0, y, 0.0],
                [x + 1.0, y + 1.0, 0.0],
                [x, y + 1.0, 0.0],
            ]);
            faces.push(Face::quad(b, b + 1, b + 2, b + 3));
        }
    }
    let bons = faces.len();
    // …e UMA gravata lá longe, no fim de um braço.
    let b = u32::try_from(pos.len()).expect("cabe");
    pos.extend_from_slice(&[
        [40.0, 0.0, 0.0],
        [41.0, 0.0, 0.0],
        [40.0, 1.0, 0.0],
        [41.0, 1.0, 0.0],
    ]);
    faces.push(Face::quad(b, b + 1, b + 2, b + 3));

    let (s, per) = local_shape_of(&pos, &faces);
    assert_eq!(s.defects, 1, "so' a gravata e' defeito");
    assert_eq!(
        s.defects_at_tip, 1,
        "o defeito esta' no raio {} e a barra e' {TIP_FRACTION}",
        per[bons].radial
    );
    assert!(
        per[bons].radial >= TIP_FRACTION,
        "a face do braco tem de ser da ponta, e mediu {}",
        per[bons].radial
    );
    assert!(
        s.faces_at_tip <= 2,
        "o DENOMINADOR tem de ser pequeno: se o corpo inteiro contasse como ponta, \
         a coluna nao distinguiria nada (mediu {})",
        s.faces_at_tip
    );

    // ⚠️ **O CONTROLO da localização:** a mesma gravata no MEIO do tapete não pode
    // ser contada como da ponta — senão a coluna mede a existência do defeito, e
    // não o sítio dele.
    let mut pos2 = pos.clone();
    let n = pos2.len();
    pos2[n - 4] = [2.0, 2.0, 1.0];
    pos2[n - 3] = [3.0, 2.0, 1.0];
    pos2[n - 2] = [2.0, 3.0, 1.0];
    pos2[n - 1] = [3.0, 3.0, 1.0];
    let (s2, _) = local_shape_of(&pos2, &faces);
    assert_eq!(s2.defects, 1);
    assert_eq!(
        s2.defects_at_tip, 0,
        "uma gravata no MEIO nao e' um defeito de ponta"
    );
}

/// ⭐⭐⭐ **A TORÇÃO NÃO DEPENDE DA DIAGONAL ESCOLHIDA** — e este gate nasceu de uma
/// mutação que SOBREVIVEU.
///
/// ⛔⛔ **A razão que o doc dava era falsa.** Ele afirmava que uma sela é plana ao
/// longo de uma diagonal e torcida ao longo da outra; medido, as duas leituras
/// são `109,47 / 109,47` na sela, `63,20 / 70,25` no canto levantado e
/// `68,67 / 60,19` no assimétrico — **quatro pontos ou são coplanares ou não
/// são**, e nenhuma diagonal fica cega.
///
/// ⭐ **A lei que sobra é outra, e é esta que o gate afirma:** as duas leituras
/// DIFEREM, então quem medisse só uma daria um número diferente conforme a
/// diagonal que o renderizador triangulou — e o `max` é a leitura conservadora.
///
/// ⚠️ *A desigualdade só é observável pela porta [`super::warp_splits`]: uma lei
/// que só se vê no resultado já colapsado não tem gate.*
/// ⛔⛔ **E são precisas DUAS fixturas, uma de cada ORDEM.** A primeira versão
/// deste gate tinha só a assimétrica, onde a diagonal `0–2` é a maior — e a
/// mutação *«olha só a `0–2`»* **SOBREVIVEU**, porque ali `max(a, b) == a`. *Uma
/// fixtura em que o máximo calha ser o primeiro argumento não distingue `max` de
/// «o primeiro».*
#[test]
fn a_torcao_nao_depende_da_diagonal_escolhida() {
    // `0–2` maior…                                    …e `1–3` maior.
    for (rotulo, p, primeira_maior) in [
        (
            "assimetrico",
            [
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [5.0, 4.0, 0.0],
                [0.0, 1.0, 2.0],
            ],
            true,
        ),
        (
            "canto levantado",
            [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 1.4],
            ],
            false,
        ),
    ] {
        let (pos, faces) = quad(p);
        let v = faces[0].verts();
        let (a, b) = super::warp_splits(&pos, v);
        assert_eq!(
            a > b,
            primeira_maior,
            "{rotulo}: a fixtura tem de ter a ORDEM declarada ({a:.2} / {b:.2}) -- as duas \
             ordens juntas sao o que mata «olha so' a primeira»"
        );
        assert!(
            (a - b).abs() > 5.0,
            "{rotulo}: as duas diagonais tem de DISCORDAR, e mediram {a:.2} / {b:.2}"
        );
        let (_, per) = local_shape_of(&pos, &faces);
        assert!(
            (per[0].warp_deg - a.max(b)).abs() < 1.0e-4,
            "{rotulo}: a torcao publicada ({}) tem de ser a MAIOR das duas ({a:.2} / {b:.2})",
            per[0].warp_deg
        );
        // ⛔ **O CONTROLO da própria premissa:** nenhuma das duas pode ser ~0, senão
        // a razão antiga («uma diagonal fica cega») estaria certa e o doc, errado.
        assert!(
            a > 20.0 && b > 20.0,
            "{rotulo}: as DUAS diagonais tem de ver a torcao ({a:.2} / {b:.2}) -- se uma \
             ficar perto de zero, a premissa refutada em 30/08 volta a valer"
        );
    }
}

/// ⭐⭐⭐ **AUDITORIA — a régua tem de dizer `1,0` numa peça SEM pontas.**
///
/// ⛔⛔ **É a lente que decide se o número que eu reporto é medição ou ruído.** Numa
/// esfera lisa todos os pontos estão à mesma distância do centro, logo caem todos na
/// casca de fora: o «corpo» e a «ponta» passam a ser a MESMA população, e a razão
/// tem de ser exactamente `1`. *Se ela desviasse aqui, todo desvio numa peça com
/// pontas seria indistinguível do artefacto da própria régua.*
#[test]
fn a_regua_diz_um_numa_peca_sem_pontas() {
    let mesh = ph2d_mesh::shapes::uv_sphere(48, 32, 1.0);
    let cent: Vec<[f32; 3]> = mesh
        .faces()
        .iter()
        .map(|f| {
            let v = f.verts();
            let mut c = [0.0f32; 3];
            for &i in v {
                let p = mesh.positions()[i as usize];
                for k in 0..3 {
                    c[k] += p[k];
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let n = v.len() as f32;
            [c[0] / n, c[1] / n, c[2] / n]
        })
        .collect();
    let uns = vec![1.0f32; cent.len()];
    let (r, amostra) = super::tip_body_ratio(&cent, &uns);
    assert!(
        (r - 1.0).abs() < 1.0e-6,
        "um campo CONSTANTE tem de dar razao 1,0 e deu {r} -- se a regua desvia com o \
         campo constante, ela mede a forma da peca e nao a densidade"
    );
    assert!(amostra > 0, "a casca da ponta nao pode ficar vazia");
}

/// ⚠️ **AUDITORIA — entradas degeneradas não podem inventar um número.**
///
/// ⛔ Uma razão devolvida sobre lista vazia ou desemparelhada seria lida como
/// medição pelo consumidor, que a imprime ao lado do alvo `0,59`.
#[test]
fn a_regua_recusa_entrada_degenerada_em_vez_de_inventar() {
    assert_eq!(super::tip_body_ratio(&[], &[]), (0.0, 0));
    let p = vec![[0.0f32, 0.0, 0.0]];
    assert_eq!(
        super::tip_body_ratio(&p, &[]),
        (0.0, 0),
        "pontos e valores desemparelhados tem de dar ZERO, nao um numero plausivel"
    );
    // ⛔⛔ **Um ponto só: a casca da PONTA fica VAZIA**, e é isso que a auditoria de
    // 30/08 achou. Todas as distâncias são `0`, logo todos os pontos caem na casca
    // `0` e a razão sai `0,0` — *que se lê como o melhor resultado possível*.
    // ⇒ o contrato é: **`n == 0` quer dizer NÃO MEDIDO**, e quem imprime olha a
    // contagem antes do número.
    let (r, n) = super::tip_body_ratio(&p, &[0.5]);
    assert!(r.is_finite(), "um ponto so' deu {r}, que nao e' finito");
    assert_eq!(
        n, 0,
        "⛔ com todos os pontos ao mesmo raio a casca da ponta e' VAZIA, e a contagem \
         tem de o dizer -- senao a razao 0,0 passa por excelente"
    );
}

/// ⭐⭐⭐ **UMA PONTA CORTADA E UMA REAMOSTRADA TÊM DE SER DISTINGUÍVEIS.**
///
/// ⛔⛔ **É a barra [`super::TIP_CUT_PCT`] que faz a distinção, e sem este gate ela não
/// é exercitada por nada** — a mutação que a punha em `0` (⇒ *toda* ponta conta como
/// cortada) **sobreviveu** ao gate da linha do relatório, que constrói a contagem à mão.
///
/// ⚠️ **As duas populações estão separadas por uma ordem de grandeza**, e é aí que a
/// barra vive: medido na peça do artista, as pontas intactas dão `−0,0 %` a `−0,4 %` e
/// as cortadas `−5 %` a `−22 %`.
#[test]
fn uma_ponta_cortada_distingue_se_de_uma_reamostrada() {
    use ph2d_mesh::{Face, Mesh};
    // Uma tenda: anel de quatro na base, um ápice em cima.
    let tent = |apex_z: f32| -> Mesh {
        Mesh::from_parts(
            vec![
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, apex_z],
            ],
            vec![
                Face::tri(0, 1, 4),
                Face::tri(1, 2, 4),
                Face::tri(2, 3, 4),
                Face::tri(3, 0, 4),
                Face::quad(3, 2, 1, 0),
            ],
        )
        .expect("a fixtura e' construida aqui")
    };
    let entrada = tent(3.0);

    let cortada = super::tip_survival(&entrada, &tent(2.4));
    assert_eq!(cortada.total, 1, "a tenda tem UM apice");
    assert_eq!(
        cortada.cut, 1,
        "uma ponta 20 % mais curta tem de contar como CORTADA (mediu {:.1} %)",
        cortada.worst_pct
    );

    // ⚠️ **O CONTROLO, e é ele que a mutação da barra mata:** uma perda de reamostragem
    // (a saída é poliédrica e os vértices não se correspondem) **não** é amputação.
    let quase = super::tip_survival(&entrada, &tent(2.985));
    assert_eq!(
        quase.total, 1,
        "o controlo tem de medir a mesma ponta, senao ele nao e' controlo"
    );
    assert_eq!(
        quase.cut, 0,
        "uma perda de {:.2} % e' reamostragem, nao amputacao -- se ela contar, a coluna \
         acusa toda peca e o artista deixa de a ler",
        quase.worst_pct
    );
}
