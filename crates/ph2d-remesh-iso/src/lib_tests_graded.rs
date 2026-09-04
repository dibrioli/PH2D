//! **OS GATES DA GRELHA GRADUADA E DA FASE ZERO** — irmão de [`super::tests`].
//!
//! ⚠️ O corte é por PERGUNTA, não por tamanho: o irmão mede a propriedade que o passe
//! isotrópico existe para dar (densidade uniforme, forma, manifold, género, rebordo);
//! este mede o que a **grelha por sítio** faz por cima dela — o orçamento, a banda que
//! engrossa onde a forma é chapada, e as invariâncias da fase zero.

use ph2d_mesh::shapes;

use super::{ALPHA, remesh_isotropic, remesh_isotropic_graded, target_edge};

/// ⭐⭐⭐ **A reprojecção que respeita a normal nasce DESLIGADA** — ver
/// [`super::facing_on`], que traz a tabela da medição.
///
/// ⛔ Ela **cura** a fase zero (o alcance que a peça do artista perde cai de `−15,9 %` para
/// `−5,7 %`) e **parte** a cadeia a jusante (`χ` de `1` para `−16`, bordo de `4` para `250`,
/// `5` ilhas, o dobro do relógio). ⚠️ *Uma fase medida sozinha pode melhorar e piorar o
/// produto* — e é por isso que a decisão vive numa função com gate em vez de num comentário.
#[test]
fn a_reprojeccao_que_respeita_a_normal_nasce_desligada() {
    assert!(
        !super::facing_on(),
        "⛔ sem a env ela tem de estar DESLIGADA -- ligada, a peca do artista sai com 250 \
         arestas de bordo e cinco ilhas"
    );
}

/// ⭐⭐⭐ **A cerca por sítio nasce DESLIGADA** — ver [`super::adaptive_on`], que traz as duas
/// tabelas.
///
/// ⛔ Ela **cura** a agulha (o alcance perdido na fase zero vai de `−15,8 %` para `−0,8 %`, com
/// a topologia da malha de trabalho perfeita) e **parte** a cadeia (`χ` de `1` para `−7`, bordo
/// de `4` para `62`, `6×` o relógio). ⚠️ *É a segunda vez que uma cura de fase zero mede assim
/// — e é por isso que a decisão vive numa função com gate.*
#[test]
fn a_cerca_por_sitio_nasce_ligada() {
    assert!(
        super::adaptive_on(),
        "⛔ ela passou a nascer LIGADA em 2026-08-31 -- ver a tabela no doc de `adaptive_on`"
    );
}

/// ⭐⭐⭐ **GATE — a grelha por sítio PRESERVA O ORÇAMENTO.**
///
/// ⛔⛔⛔ **É a lei que a wave de 31/08 comprou.** Até essa data o campo só afinava (o tecto
/// era `1`), logo ele **acrescentava** trabalho: a malha de trabalho da peça do artista ia de
/// `3 982` para `33 156` faces (`8,3×`), e era essa inflação — e não a graduação — que a
/// jusante não digeria. *A adaptação move os quads; ela não os cria.*
///
/// A régua é a contagem que o campo prevê, `N = Σ_face área / h²`, com o `h` lido **pela
/// própria grelha** (o `at()` leva o mínimo dos 27 vizinhos).
#[test]
fn a_grelha_por_sitio_preserva_o_orcamento() {
    // Uma esfera com uma banda de curvatura muito maior que a mediana: sem contraste de
    // curvatura a grelha e' constante e o gate nao mede nada.
    let mut mesh = ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
    {
        let pos = mesh.positions_mut();
        for p in pos.iter_mut() {
            // Uma crista fina em torno do equador.
            let d = p[1].abs();
            if d < 0.06 {
                let k = 1.0 + 0.35 * (1.0 - d / 0.06);
                p[0] *= k;
                p[2] *= k;
            }
        }
    }
    mesh.rebuild();
    let target = super::target_edge(&mesh, super::ALPHA);
    let grid = super::SizingGrid::build(&mesh, target, &[]).expect("a fixtura tem curvatura");

    // ⚠️ **O CONTROLE:** a grelha tem de VARIAR, senao o que este gate mede e' o campo
    // uniforme e a renormalizacao seria trivialmente `1`.
    let (mut lo, mut hi) = (f32::INFINITY, 0.0f32);
    for f in mesh.faces() {
        let v = f.verts();
        let pos = mesh.positions();
        let mid = [
            (pos[v[0] as usize][0] + pos[v[1] as usize][0] + pos[v[2] as usize][0]) / 3.0,
            (pos[v[0] as usize][1] + pos[v[1] as usize][1] + pos[v[2] as usize][1]) / 3.0,
            (pos[v[0] as usize][2] + pos[v[1] as usize][2] + pos[v[2] as usize][2]) / 3.0,
        ];
        let h = grid.at(mid);
        lo = lo.min(h);
        hi = hi.max(h);
    }
    assert!(
        hi / lo > 1.5,
        "⛔ a fixtura nao produz contraste de tamanho ({lo:.5}..{hi:.5}) -- o gate nao mede nada"
    );

    // ⭐ A contagem prevista contra a que o alvo escalar pede.
    let (mut pred, mut area) = (0.0f64, 0.0f64);
    let pos = mesh.positions();
    for f in mesh.faces() {
        let v = f.verts();
        let (a, b, c) = (pos[v[0] as usize], pos[v[1] as usize], pos[v[2] as usize]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            u[1] * w[2] - u[2] * w[1],
            u[2] * w[0] - u[0] * w[2],
            u[0] * w[1] - u[1] * w[0],
        ];
        let tri = f64::from((n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt()) * 0.5;
        let mid = [
            (a[0] + b[0] + c[0]) / 3.0,
            (a[1] + b[1] + c[1]) / 3.0,
            (a[2] + b[2] + c[2]) / 3.0,
        ];
        let h = f64::from(grid.at(mid).max(1.0e-9));
        pred += tri / (h * h);
        area += tri;
    }
    let want = area / f64::from(target).powi(2);
    let razao = pred / want;
    assert!(
        (razao - 1.0).abs() < 0.05,
        "⛔ a grelha nao preserva o orcamento: previstos {pred:.0} para {want:.0} pedidos \
         (razao {razao:.3}) -- a renormalizacao caiu?"
    );
}

/// ⭐⭐ **GATE — a grelha AFINA e ENGROSSA, e o «engrossa» vem da RENORMALIZAÇÃO.**
///
/// ⛔ A lei por vértice tem tecto `1` (*«nunca mais grossa que o alvo»*) — é o factor
/// `√(N_previsto/N_pedido)`, que sai `> 1`, que empurra as regiões chapadas para cima do alvo.
/// *Sem ele o campo só afina, e um campo que só afina não redistribui um orçamento: aumenta-o.*
///
/// ⚠️ **Este gate e o [`a_grelha_por_sitio_preserva_o_orcamento`] morrem da MESMA mutação**
/// (apagar o `*= s`) e são mantidos os dois de propósito: um mede a **contagem**, o outro o
/// **intervalo**, e uma renormalização errada pode acertar num e falhar no outro.
///
/// ⛔⛔ **Uma banda simétrica (`[alvo/√R, alvo·√R]`) foi construída e REVERTIDA** — a mutação
/// que a apagava sobrevivia a este gate, e o A/B ponta a ponta deu-lhe a resposta pela régua
/// por ponta (ver o doc de [`super::adaptive_on`]).
#[test]
fn a_banda_da_grelha_engrossa_onde_a_forma_e_chapada() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
    {
        let pos = mesh.positions_mut();
        for p in pos.iter_mut() {
            let d = p[1].abs();
            if d < 0.06 {
                let k = 1.0 + 0.35 * (1.0 - d / 0.06);
                p[0] *= k;
                p[2] *= k;
            }
        }
    }
    mesh.rebuild();
    let target = super::target_edge(&mesh, super::ALPHA);
    let grid = super::SizingGrid::build(&mesh, target, &[]).expect("a fixtura tem curvatura");
    let mut mais_grosso = false;
    let mut mais_fino = false;
    for p in mesh.positions() {
        let h = grid.at(*p);
        if h > target * 1.05 {
            mais_grosso = true;
        }
        if h < target * 0.95 {
            mais_fino = true;
        }
    }
    assert!(
        mais_fino,
        "⛔ a grelha tem de AFINAR onde a forma aperta -- e' a razao de ela existir"
    );
    assert!(
        mais_grosso,
        "⛔ e tem de ENGROSSAR onde a forma e' chapada, senao ela acrescenta trabalho em vez \
         de o mover (o tecto `1` foi o que a fez inflar 8,3x)"
    );
}

/// ⭐⭐⭐ **GATE — a fase zero não depende de ONDE a peça está na cena.**
///
/// ⛔⛔⛔ **Ordem do dono, 2026-08-31: *«o remesh deve funcionar perfeitamente em qualquer
/// lugar»*.** Ela nasceu de um report com foto: a MESMA escultura, o mesmo `Detail`, a mesma
/// `Curvature`, dava `0` de `4` pontas cortadas na origem e `2` de `4` onde o importador a
/// põe (`sculpt3d_import::IMPORT_SPAN` ancora toda peça importada fora da origem).
///
/// ⭐ **A causa era esta crate:** a [`super::sizing::SizingGrid`] indexava por coordenada de
/// **mundo** (`p / cell`), logo mover a peça movia as fronteiras dos baldes — e como cada
/// balde guarda o **mínimo** e o `at` lê o mínimo de 27, um deslocamento muda que região
/// herda a finura de uma agulha. Hoje a grelha é ancorada no canto da caixa da **peça**.
///
/// ⚠️ **Medido na `uv_sphere(96, 144)`, `x ∈ {0, ½, 1, 2}`:**
///
/// | | antes | **depois** |
/// |---|---|---|
/// | vértices | `2 633` · `2 712` · `2 679` · `2 586` | ⭐ **`2 687` nas quatro** |
/// | dispersão | `4,9 %` | **`0,0 %`** |
///
/// ⚠️ **O CONTROLO é o caminho SEM graduação**, que tem de ser **exactamente** igual nas
/// quatro: ele não tem campo, logo não tem fronteiras — *é ele que prova que o remalhador
/// em si já era invariante e que o defeito era do campo.*
///
/// ⛔ **`x = 16` fica FORA da barra de propósito:** a `16` unidades de distância com feições
/// de `0,03`, a subtracção `p − origem` perde bits e o remalhador é iterativo — um bit muda
/// uma decisão de corte e a diferença cascateia. *A cerca é honesta: esta crate garante
/// invariância na escala em que uma cena vive, não bit-exactidão a qualquer distância.*
#[test]
fn a_fase_zero_nao_depende_de_onde_a_peca_esta() {
    let base = shapes::uv_sphere(96, 144, 1.0);
    let corrida = |graded: bool| -> Vec<usize> {
        [0.0f32, 0.5, 1.0, 2.0]
            .iter()
            .map(|d| {
                let mut m = base.clone();
                for p in m.positions_mut() {
                    p[0] += d;
                }
                if graded {
                    remesh_isotropic_graded(&mut m, ALPHA).verts_after
                } else {
                    remesh_isotropic(&mut m, ALPHA).verts_after
                }
            })
            .collect()
    };
    let liso = corrida(false);
    assert!(
        liso.iter().all(|v| *v == liso[0]),
        "CONTROLO: sem graduacao o remalhador ja' era invariante, e deu {liso:?}"
    );
    let grad = corrida(true);
    // ⛔⛔⛔ **A METADE QUE FALTAVA, e duas mutações sobreviveram sem ela:** desligar a
    // ancoragem só na CONSULTA (ou só na CONSTRUÇÃO) faz as chaves nunca casarem, o `at`
    // cai no `fallback` constante, o campo **morre** — e um campo morto é perfeitamente
    // invariante. *«Invariante porque ancorada» e «invariante porque não existe» lêem-se
    // igual em qualquer régua que só meça dispersão.*
    assert_ne!(
        grad[0], liso[0],
        "a graduacao tem de MUDAR a malha, senao a invariancia acima e' a de um campo morto"
    );
    let (lo, hi) = (
        *grad.iter().min().expect("quatro corridas"),
        *grad.iter().max().expect("quatro corridas"),
    );
    #[expect(clippy::cast_precision_loss, reason = "milhares de vertices")]
    let dispersao = (hi - lo) as f32 / lo.max(1) as f32;
    assert!(
        dispersao <= 0.01,
        "a graduacao tem de ser invariante a' translacao: {grad:?} -> dispersao {:.1} %          (antes da ancoragem da grelha eram 4,9 %)",
        100.0 * dispersao
    );
}

/// ⭐⭐⭐ **SONDA — a fase zero é invariante a uma TRANSLAÇÃO?**
///
/// ⛔ Ordem do dono, 2026-08-31: *«o remesh deve funcionar perfeitamente em qualquer
/// lugar»*. Esta é a primeira pergunta da cadeia: se o F1 já muda, tudo a jusante muda.
///
/// ```text
/// cargo test -p ph2d-remesh-iso --release a_fase_zero_e_invariante_a_translacao -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- invariancia a translacao da fase zero"]
fn a_fase_zero_e_invariante_a_translacao() {
    for (nome, base) in [
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0)),
        ("esfera 96x144", shapes::uv_sphere(96, 144, 1.0)),
    ] {
        for graded in [false, true] {
            let mut linha = Vec::new();
            for d in [0.0f32, 0.5, 1.0, 2.0, 16.0] {
                let mut m = base.clone();
                for p in m.positions_mut() {
                    p[0] += d;
                }
                let r = if graded {
                    remesh_isotropic_graded(&mut m, ALPHA)
                } else {
                    remesh_isotropic(&mut m, ALPHA)
                };
                linha.push((d, r.verts_after, m.face_count()));
            }
            eprintln!(
                "   {nome} graded={graded}: {}",
                linha
                    .iter()
                    .map(|(d, v, f)| format!("x={d} -> {v}v {f}f"))
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        }
    }
}

/// ⭐⭐⭐ **SONDA — o CAMPO é invariante, medido nos mesmos sítios da superfície?**
///
/// ⚠️ Ela mede a [`super::sizing::SizingGrid`] **sozinha**, sem o remalhador pelo meio: a
/// grelha é construída na peça em `x = 0` e na mesma peça em `x = d`, e o `at` é lido nos
/// vértices correspondentes. *Se o campo concorda e a saída não, o defeito é do laço; se o
/// campo já discorda, é dele.*
#[test]
#[ignore = "sonda -- invariancia do CAMPO"]
fn o_campo_e_invariante_nos_mesmos_sitios() {
    for (nome, base) in [
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0)),
        ("esfera 96x144", shapes::uv_sphere(96, 144, 1.0)),
    ] {
        let alvo = target_edge(&base, ALPHA);
        let g0 = super::sizing::SizingGrid::build(&base, alvo, &[]).expect("grelha");
        for d in [0.5f32, 1.0, 2.0, 16.0] {
            let mut m = base.clone();
            for p in m.positions_mut() {
                p[0] += d;
            }
            let g1 = super::sizing::SizingGrid::build(&m, alvo, &[]).expect("grelha");
            let mut pior = 0.0f32;
            let mut n_dif = 0usize;
            for (a, b) in base.positions().iter().zip(m.positions()) {
                let (v0, v1) = (g0.at(*a), g1.at(*b));
                let rel = (v0 - v1).abs() / v0.max(1.0e-9);
                if rel > 1.0e-6 {
                    n_dif += 1;
                }
                pior = pior.max(rel);
            }
            eprintln!(
                "   {nome} x={d}: pior desvio relativo {:.3e} | {n_dif} de {} sitios diferem",
                pior,
                base.positions().len()
            );
        }
    }
}
