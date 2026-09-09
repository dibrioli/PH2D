//! **O GESTO DO FILTRO DE TECIDO, medido no produto.**
//!
//! Irmão do [`super::tests`] pelo tecto de LOC, e o corte é por **sujeito**: lá
//! ficam os gates do filtro em geral (o sinal do arrasto, a exclusão contra o
//! transform, o alcance da lei, o undo) e aqui os do **tecido** — o alvo do
//! aperto, a taxa de amostragem, e o «baixo» da gravidade.
//!
//! ⚠️ **O arnês é uma cópia local**, como o dos irmãos: um macro exportado entre
//! módulos de teste seria acoplamento por conveniência.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --release --bins sculpt3d::filter::cloth_tests -- --ignored --nocapture
//! ```

use ph2d_mesh::shapes::uv_sphere;
use ph2d_sculpt3d::{ClothFilterKind, FilterLaw, Verb};

use super::super::Sculpt3dScene;

/// Abre a GPU, ou diz que não há nada a afirmar.
macro_rules! gpu_or_skip {
    () => {
        match ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) {
            Ok(g) => g,
            Err(_) => {
                eprintln!("no GPU adapter on this machine — nothing to assert");
                return;
            }
        }
    };
}

/// Uma cena com uma esfera e o verbo pedido em mãos.
fn scene(device: &wgpu::Device, verb: Verb) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, uv_sphere(24, 36, 1.0), 1.0);
    s.note_canvas(ph2d_editor::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = verb;
    s
}

/// ⭐⭐⭐ **O ALVO DO APERTO É UM VÉRTICE DA MALHA, e não um ponto da face.**
///
/// ⚠️ **A diferença foi medida contra o oráculo e vale `65×`**: apertar para o
/// ponto solto onde o raio bateu dá erro `0,696942` na bancada
/// (`ph2d-cloth/tests/oraculo_do_filtro.rs`); para o vértice, `0,010755`. ⛔ E
/// nenhum gate da LEI a via: a bancada corre sobre fixtures, e quem escolhe o
/// ponto no produto é esta função. *Um gate que mede a lei é cego a quem lhe
/// entrega os argumentos.*
///
/// ⚠️ **O controlo está dentro:** o ponto do acerto tem de ser DIFERENTE do
/// vértice devolvido nalgum clique, senão a asserção é vácua sobre uma malha em
/// que os dois coincidem por acaso.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_pinch_anchor_lands_on_a_vertex_of_the_mesh() {
    let gpu = gpu_or_skip!();
    let s = scene(&gpu.device, Verb::Draw);
    let verts: Vec<[f32; 3]> = s.mesh().positions().to_vec();
    // ⚠️ **O CENTRO do viewport**, que é onde a esfera está: sem acerto a porta
    // devolve o centro da caixa (a resposta honesta para «carregou fora do
    // barro»), e o gate mediria o fallback em vez da lei — foi o que ele fez na
    // 1.ª redacção, quando lia `self.last` em vez de receber o ponto.
    let (cx, cy) = (s.viewport().0 as f32 * 0.5, s.viewport().1 as f32 * 0.5);
    let ancora = s.filter_pinch_anchor(cx, cy);
    let mais_perto = verts
        .iter()
        .map(|p| {
            let d = [p[0] - ancora[0], p[1] - ancora[1], p[2] - ancora[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(f32::INFINITY, f32::min);
    assert_eq!(
        mais_perto, 0.0,
        "a ancora do aperto ({ancora:?}) nao e' um vertice da malha -- o mais proximo esta' a \
         {mais_perto:.6}, e o alvo aperta para o VERTICE activo"
    );
}

/// ⭐⭐⭐ **QUANTOS PASSOS A SIMULAÇÃO AVANÇA É FUNÇÃO DOS QUADROS, NUNCA DOS
/// EVENTOS DO RATO.**
///
/// ⛔⛔ **É a lei que esta casa pagou SEIS vezes no relevo do Painter** — *o traço
/// é fato do CAMINHO, nunca de quão fino o motor amostrou o caminho* — e o filtro
/// de tecido nasceu a violá-la: ele corria um passo de solver por evento do
/// sistema, e um rato de `1000 Hz` entrega dezasseis por quadro contra os dois de
/// um de `125 Hz`. ⇒ *dois artistas com ratos diferentes obtinham panos
/// diferentes do mesmo gesto.*
///
/// ⚠️ **Ela é de CORRECÇÃO antes de ser de relógio.** Que o report de performance
/// caia junto é consequência.
///
/// O gate tem as **três** metades:
/// 1. um evento sozinho **não** mexe a peça;
/// 2. o quadro mexe;
/// 3. **dez eventos e um quadro dão o MESMO que um evento e um quadro**, ao bit,
///    para o mesmo `x` final.
#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn the_cloth_filter_advances_per_frame_and_not_per_pointer_event() {
    let gpu = gpu_or_skip!();
    let fotografar = |s: &Sculpt3dScene| -> Vec<[f32; 3]> { s.mesh().positions().to_vec() };
    let pior = |a: &[[f32; 3]], b: &[[f32; 3]]| -> f32 {
        a.iter()
            .zip(b)
            .map(|(p, q)| {
                ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
            })
            .fold(0.0, f32::max)
    };

    // (1) e (2) — o evento regista, o quadro corre.
    let mut s = scene(&gpu.device, Verb::Draw);
    s.filter_law = FilterLaw::Cloth(ClothFilterKind::Gravity);
    assert!(s.arm_filter() && s.begin_filter(400.0, 300.0));
    let ao_carregar = fotografar(&s);
    for k in 1..=10 {
        s.filter_at(400.0 + 30.0 * k as f32);
    }
    assert_eq!(
        pior(&ao_carregar, &fotografar(&s)),
        0.0,
        "dez eventos de ponteiro moveram a peca SEM um quadro -- o filtro de tecido voltou a \
         avancar por evento, e quantos passos ele da' passa a ser funcao da taxa do rato"
    );
    s.flush_cloth_filter();
    let dez_eventos = fotografar(&s);
    assert!(
        pior(&ao_carregar, &dez_eventos) > 0.0,
        "o quadro nao correu um passo -- o dreno esta' partido, e sem ele o filtro fica MUDO"
    );

    // (3) — a mesma mão, amostrada uma vez só.
    let mut t = scene(&gpu.device, Verb::Draw);
    t.filter_law = FilterLaw::Cloth(ClothFilterKind::Gravity);
    assert!(t.arm_filter() && t.begin_filter(400.0, 300.0));
    t.filter_at(400.0 + 30.0 * 10.0);
    t.flush_cloth_filter();
    let um_evento = fotografar(&t);
    let desvio = pior(&dez_eventos, &um_evento);
    println!("dez eventos contra um, no mesmo x final: desvio {desvio:.9}");
    assert_eq!(
        desvio, 0.0,
        "a peca depende de QUANTOS eventos o rato mandou (desvio {desvio:.9}) -- \
         o resultado tem de ser fato do CAMINHO, nao da amostragem"
    );
}

/// ⭐⭐⭐ **A GRAVIDADE CAI PARA BAIXO NA TELA** — o report do Enio, 2026-09-08:
/// *«parece que a gravidade está em z mas neste app deve ser em y»*.
///
/// # ⚠️ A régua NÃO pergunta qual é a constante
///
/// Um gate que comparasse `gravity_axis` com `−Camera3d::UP` seria a mesma
/// linha escrita duas vezes: ele passaria por construção, e passaria na mesma
/// se as duas estivessem erradas. *Um controlo que depende da cura que mede não
/// é um controlo* — esta casa já pagou isso na wave anterior deste mesmo
/// filtro, com o refresco das normais.
///
/// ⇒ a régua é **onde o pano vai parar no ECRÃ**: corre a lei de verdade sobre
/// uma malha, projecta o centróide antes e depois pela câmera do produto, e
/// exige que o `y` da tela **cresça** (a convenção de janela: `y` cresce para
/// baixo). *Gravidade é uma afirmação sobre o que o artista vê, e é isso que se
/// mede.*
///
/// # ⭐ E o CONTROLO é o valor que shipava
///
/// Com `[0, 0, −1]` — a convenção do alvo da espec, que é `Z` para cima — a
/// mesma medição dá o sinal **oposto** no enquadramento de omissão: o pano
/// subia. Sem esta metade o gate não distinguiria a cura de uma malha que
/// simplesmente não se mexe.
#[test]
fn a_gravidade_do_filtro_cai_para_baixo_na_tela() {
    use ph2d_mesh_render::Camera3d;
    use ph2d_sculpt3d::{ClothFilterOrientation, ClothFilterStep, SculptStroke};

    /// O centróide da malha — a régua de *para onde a peça inteira foi*.
    fn centroide(m: &ph2d_mesh::Mesh) -> [f32; 3] {
        let p = m.positions();
        let n = p.len().max(1) as f32;
        let mut c = [0.0f32; 3];
        for q in p {
            for k in 0..3 {
                c[k] += q[k];
            }
        }
        c.map(|v| v / n)
    }

    /// Quantos pixels o pano desceu na tela sob este «baixo». Positivo = desceu.
    fn queda_na_tela(gravity: [f32; 3]) -> f32 {
        const SIZE: (u32, u32) = (1280, 720);
        let mut mesh = uv_sphere(16, 24, 1.0);
        let antes = centroide(&mesh);
        let mut st = SculptStroke::default();
        st.cloth_filter_begin(
            &mesh,
            ph2d_sculpt3d::ClothFilterProps::default(),
            ClothFilterKind::Gravity,
            [0.0; 3],
        );
        st.cloth_filter_step(
            &mut mesh,
            ClothFilterKind::Gravity,
            &ClothFilterStep {
                s: 1.0,
                gravity_axis: gravity,
                frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                axes: [true; 3],
                eye: [0.0, 0.0, 1.0],
            },
        );
        let depois = centroide(&mesh);
        // ⚠️ A câmera de OMISSÃO, que é o enquadramento em que a peça nasce —
        // é nele que o report foi escrito.
        let cam = Camera3d::default();
        let a = cam
            .project(antes, SIZE)
            .expect("o centroide esta' na frente");
        let d = cam
            .project(depois, SIZE)
            .expect("o centroide esta' na frente");
        d.1 - a.1
    }

    // O «baixo» que o PRODUTO monta, lido pela porta e não escrito à mão.
    let (_, produto) = super::referencial_e_gravidade(
        ClothFilterOrientation::Local,
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    );
    let desceu = queda_na_tela(produto);
    // ⚠️ **O CONTROLO**: o eixo que shipava até 2026-09-08.
    let antes_da_cura = queda_na_tela([0.0, 0.0, -1.0]);
    println!(
        "queda na tela: produto {produto:?} -> {desceu:+.4} px | pre-cura [0,0,-1] -> \
         {antes_da_cura:+.4} px"
    );
    assert!(
        desceu > 0.0,
        "o filtro de GRAVIDADE nao leva o pano para BAIXO na tela (Delta y = {desceu:+.4} px) -- \
         o «baixo» tem de sair do CIMA da camera (`Camera3d::UP`), e esta casa e' Y para cima"
    );
    assert!(
        antes_da_cura < 0.0,
        "o controlo nao reproduz o report: com o eixo do ALVO ([0,0,-1], que e' Z para cima) o \
         pano tinha de SUBIR na tela, e mediu {antes_da_cura:+.4} px -- sem isto este gate nao \
         distingue a cura de uma malha parada"
    );
}

/// ⭐⭐⭐ **NA ORIENTAÇÃO *VIEW* O «BAIXO» É O VERTICAL DO ECRÃ — NÃO A PROFUNDIDADE.**
///
/// ⚠️⚠️ **Esta é a metade que o [`a_gravidade_do_filtro_cai_para_baixo_na_tela`]
/// não pode ver.** Aquele dirige o braço *Local* com a base de ecrã
/// **IDENTIDADE**, e ali as duas respostas coincidem **ao bit**: `−cima_do_ecrã`
/// e `−Camera3d::UP` são o mesmo vector. *Um enquadramento em que os dois braços
/// devolvem o mesmo número não testa nenhum dos dois* — é o verde por vácuo que
/// o irmão [`super::super::ref_mode_tests`] já pagou nas pontas que coincidem.
///
/// ⇒ a fixture **INCLINA** o ecrã 90° em torno do `x`: o cima da tela passa a
/// ser o `+z` do mundo e a profundidade passa a ser o `−y`. Nesse enquadramento
/// a resposta certa é `[0, 0, −1]`, e a resposta «profundidade» é `[0, ±1, 0]` —
/// que é **exactamente** o que o braço *Local* devolve, porque o cima do MUNDO
/// caiu sobre o eixo do olho.
///
/// ⭐ É isso que dá o discriminador: aqui *usar o cima do mundo* e *usar a
/// profundidade* são a **mesma** resposta errada, e só a vertical do ecrã se
/// separa das duas.
#[test]
fn na_vista_o_baixo_e_o_vertical_do_ecra_e_nao_a_profundidade() {
    use ph2d_mesh_render::Camera3d;
    use ph2d_sculpt3d::ClothFilterOrientation;

    /// Dois eixos são o mesmo? (evita comparar `f32` por `==`).
    fn mesmo(a: [f32; 3], b: [f32; 3]) -> bool {
        (0..3).all(|k| (a[k] - b[k]).abs() < 1e-6)
    }

    // O ecrã inclinado 90° em torno do x: direita · cima · para-o-olho.
    const DIREITA: [f32; 3] = [1.0, 0.0, 0.0];
    const CIMA: [f32; 3] = [0.0, 0.0, 1.0];
    const PROFUNDIDADE: [f32; 3] = [0.0, -1.0, 0.0];
    let ecra = [DIREITA, CIMA, PROFUNDIDADE];

    let (frame_vista, vista) = super::referencial_e_gravidade(ClothFilterOrientation::View, ecra);
    let (_, local) = super::referencial_e_gravidade(ClothFilterOrientation::Local, ecra);
    println!("ecra inclinado: vista {vista:?} | local {local:?} | profundidade {PROFUNDIDADE:?}");

    // (1) Na vista, o referencial É a base do ecrã.
    assert!(
        (0..3).all(|r| mesmo(frame_vista[r], ecra[r])),
        "na orientacao View o referencial tem de ser a base do ECRA; veio {frame_vista:?}"
    );

    // (2) A lei: o baixo é o −cima do ECRÃ.
    let esperado = [-CIMA[0], -CIMA[1], -CIMA[2]];
    assert!(
        mesmo(vista, esperado),
        "na orientacao View o «baixo» tem de ser o VERTICAL do ecra ({esperado:?}); veio {vista:?}"
    );

    // (3) ⛔ E NÃO a profundidade — nos dois sinais.
    assert!(
        !mesmo(vista, PROFUNDIDADE)
            && !mesmo(
                vista,
                [-PROFUNDIDADE[0], -PROFUNDIDADE[1], -PROFUNDIDADE[2]]
            ),
        "o «baixo» da vista virou a PROFUNDIDADE ({PROFUNDIDADE:?}); veio {vista:?}"
    );

    // (4) ⚠️ O CONTROLO DE NÃO-VACUIDADE: neste enquadramento os dois braços TÊM
    // de discordar. Com o ecrã alinhado ao mundo eles coincidem, e então este
    // gate passaria mesmo com o braço da vista apagado — que é o defeito que
    // ele existe para apanhar.
    assert!(
        !mesmo(vista, local),
        "a fixture NAO separa os dois bracos (vista {vista:?} == local {local:?}) -- sem isto o \
         gate e' verde por vacuo"
    );

    // E aqui o braço *Local* cai sobre o eixo do OLHO, que é a resposta errada.
    let up = Camera3d::UP;
    assert!(
        mesmo(local, [-up.x, -up.y, -up.z]),
        "o braco Local mudou de lei; veio {local:?}"
    );
}
