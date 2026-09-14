//! **O ESFREGÃO DE DESLOCAMENTO, na CENA** — o que só uma pilha de
//! multiresolução a sério pode afirmar.
//!
//! Irmão (`#[path]`) do [`super`], e o corte é o mesmo do vizinho
//! [`super::apagador`]: a **LEI** dele (a média a montante, a força ao
//! quadrado, a degenerescência do arrasto parado) é medida sem GPU na
//! `ph2d-sculpt3d`, sobre uma fixtura onde ela se calcula à mão; aqui fica o que
//! depende da **pilha** — a recusa quando não há, e a promessa que dá nome à
//! ferramenta: *a forma de baixo não se mexe*.
//!
//! ⚠️⚠️ **O traço destes gates ANDA**, e é obrigatório: o [`Dab::path`] sai da
//! diferença entre centros consecutivos, logo um arnês de UM dab deixa o
//! [`SmearMode::Drag`] **inerte por lei** (espec §5.3) e mediria um pincel
//! parado sobre uma implementação correcta.

use super::*;

/// Uma peça com `níveis` níveis.
fn cena_com_pilha(device: &wgpu::Device, niveis: usize) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, ph2d_mesh::shapes::uv_sphere(16, 24, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.radius_px = 160.0;
    for _ in 1..niveis {
        assert!(s.subdivide(), "a fixtura não conseguiu subdividir");
    }
    s
}

/// **UM TRAÇO QUE ANDA** — `n` dabs em linha, pela sequência do pen-down.
fn um_traco(s: &mut Sculpt3dScene, n: usize) {
    let (x0, y) = (450.0f32, 350.0f32);
    assert!(s.aim(x0, y), "o raio errou a peça enquadrada");
    s.stroke.begin(s.objects[s.active].stack.mesh());
    s.open_dyntopo_stroke();
    s.open_reference_stroke();
    let mut pegou = 0usize;
    for k in 0..n {
        if s.sculpt_at(x0 + 18.0 * k as f32, y) {
            pegou += 1;
        }
    }
    s.close_stroke();
    assert!(
        pegou >= n - 1,
        "o traço só pegou a malha em {pegou} de {n} pontos: a fixtura não \
         contém o fenómeno"
    );
}

/// O maior deslocamento entre duas nuvens.
fn maior(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| {
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// Esculpe umas bossas com o `Draw`, que é o que o esfregão tem para
/// transportar.
fn bossas(s: &mut Sculpt3dScene) {
    let verbo = s.brush.verb;
    let forca = s.brush.strength;
    s.brush.verb = Verb::Draw;
    s.brush.strength = 1.0;
    for _ in 0..4 {
        um_traco(s, 2);
    }
    s.brush.verb = verbo;
    s.brush.strength = forca;
}

/// ⛔⛔ **SEM PILHA ELE RECUSA — e a recusa é o produto** (espec §5.6).
///
/// ⭐ **O controlo positivo está dentro:** o MESMO gesto, com pilha e com
/// relevo esculpido, move. *Sem ele um `assert_eq!` de igualdade ficaria verde
/// sobre um pincel que nunca funciona.*
#[test]
#[ignore]
fn sem_pilha_o_esfregao_nao_move_e_com_pilha_move() {
    let gpu = gpu_or_skip!();

    // (1) UM nível só: não há deslocamento nenhum para transportar.
    let mut s = cena_com_pilha(&gpu.device, 1);
    s.brush.verb = Verb::SmearMultires;
    s.brush.strength = 1.0;
    let antes = s.mesh().positions().to_vec();
    um_traco(&mut s, 4);
    assert_eq!(
        s.mesh().positions(),
        &antes[..],
        "o esfregão moveu barro numa peça sem pilha — ali não existe \
         deslocamento nenhum"
    );

    // ⭐ (2) O CONTROLO: com pilha e com relevo, ele move.
    let mut s = cena_com_pilha(&gpu.device, 3);
    bossas(&mut s);
    s.brush.verb = Verb::SmearMultires;
    s.brush.strength = 1.0;
    let antes = s.mesh().positions().to_vec();
    um_traco(&mut s, 4);
    let d = maior(&antes, s.mesh().positions());
    assert!(
        d > 1e-3,
        "com pilha e com relevo o esfregão mediu {d:.3e} — a metade de cima \
         deste gate não estaria a afirmar nada"
    );
}

/// ⭐⭐⭐ **ELE LEVA A PELE E NÃO TOCA NA FORMA DE BAIXO** — a promessa que dá
/// nome à ferramenta (espec §5.1).
///
/// ⚠️ **A régua é o NÍVEL DE BAIXO, byte a byte**, e não a caixa da peça: uma
/// caixa é um extremo global, e um pincel que deformasse a base sem lhe mudar a
/// extensão passaria. *A pergunta é «ele tocou na forma?», e a resposta exacta
/// é o nível que ele nunca devia ter aberto.*
///
/// ⭐ **E o controlo positivo é o nível de CIMA**, que tem de se mexer — senão
/// a metade de cima seria a afirmação trivial de que um pincel inerte não toca
/// em nada.
#[test]
#[ignore]
fn o_esfregao_leva_a_pele_e_a_forma_de_baixo_fica_intacta() {
    let gpu = gpu_or_skip!();
    let mut s = cena_com_pilha(&gpu.device, 3);
    bossas(&mut s);

    let base_antes = s.objects[s.active]
        .stack
        .level_mesh(0)
        .expect("o nível de baixo existe")
        .positions()
        .to_vec();
    let topo_antes = s.mesh().positions().to_vec();

    s.brush.verb = Verb::SmearMultires;
    s.brush.strength = 1.0;
    um_traco(&mut s, 5);

    let base_depois = s.objects[s.active]
        .stack
        .level_mesh(0)
        .expect("o nível de baixo continua a existir")
        .positions()
        .to_vec();
    assert_eq!(
        base_antes, base_depois,
        "o esfregão mexeu na FORMA de baixo — ele só pode mover a pele"
    );
    let d = maior(&topo_antes, s.mesh().positions());
    assert!(
        d > 1e-3,
        "o nível de cima não se mexeu ({d:.3e}): sem isto a metade de cima não \
         afirma nada"
    );
}

/// ⭐⭐ **O ARRASTO PARADO É INERTE NO CAMINHO DO PRODUTO, e o APERTO não**
/// (espec §5.3).
///
/// ⚠️ **A lei já é medida sem GPU**; o que este gate acrescenta é que ela
/// sobrevive à cadeia inteira — o pick, a câmera, a pilha. *Uma degenerescência
/// que existe na lei e desaparece no produto é um pincel que o artista vê
/// mexer-se quando a mão está parada.*
#[test]
#[ignore]
fn com_a_mao_parada_so_o_arrasto_fica_inerte() {
    let gpu = gpu_or_skip!();
    let parado = |modo: ph2d_sculpt3d::SmearMode| {
        let mut s = cena_com_pilha(&gpu.device, 3);
        bossas(&mut s);
        s.brush.verb = Verb::SmearMultires;
        s.brush.strength = 1.0;
        s.brush.smear_mode = modo;
        let antes = s.mesh().positions().to_vec();
        // ⚠️ **UM dab**, que é o que deixa o `Dab::path` a zero.
        um_traco(&mut s, 1);
        maior(&antes, s.mesh().positions())
    };
    assert_eq!(
        parado(ph2d_sculpt3d::SmearMode::Drag),
        0.0,
        "com a mão parada o arrasto tem de ser inerte AO BIT"
    );
    let d = parado(ph2d_sculpt3d::SmearMode::Pinch);
    assert!(
        d > 1e-4,
        "o aperto parado mediu {d:.3e} — a direcção dele é da GEOMETRIA e não \
         podia ter degenerado com o cursor"
    );
}

/// ⭐⭐⭐ **A PEÇA QUE AS DUAS CENAS DE MULTIRRESOLUÇÃO ABREM CABE NO ORÇAMENTO
/// DEPOIS DOS DOIS `K` QUE O ROTEIRO MANDA** — a cura do report do dono
/// (*«meio travado, até na hora de rotacionar o canvas»*, 2026-09-14).
///
/// ⛔⛔ **Elas caíam no default do módulo (`98 306` vértices) e o roteiro manda
/// apertar `K` duas vezes ⇒ `1 572 866`.** Ali um dab de **`Draw`** custa
/// `9,7 ms` contra o *kill* de `8` — *toda* ferramenta estoura, e a câmera
/// engasga. *A cena ensinava que o pincel é lento quando quem é pesada é a peça
/// que ela própria mandou construir.*
///
/// ⚠️ **A régua é a CONTAGEM e não o relógio**, de propósito: um gate de tempo
/// aqui seria mais um membro da família de flakes sob fan-out que o
/// `CLAUDE.md` §5.0 lista. A contagem é determinística, e o tecto sai da
/// medição que está no doc do [`crate::scenes_mesh`].
#[test]
fn a_peca_das_cenas_de_multirresolucao_aguenta_os_dois_k_do_roteiro() {
    // ⚠️ **O piso e o tecto**: com menos de `2 000` o relevo não tem onde se
    // ler, e acima de `~10 000` o dab sai do orçamento na peça do artista.
    const PISO: usize = 2_000;
    const TECTO: usize = 10_000;
    let mut m = crate::scenes::mesh::peca_de_multirresolucao();
    let abre = m.vert_count();
    for _ in 0..2 {
        m = ph2d_mesh::subdivide(&m);
    }
    assert!(
        (PISO..=TECTO).contains(&m.vert_count()),
        "a peça abre com {abre} vértices e os dois `K` do roteiro levam-na a \
         {} — fora da faixa [{PISO}, {TECTO}] em que o dab cabe no orçamento e \
         o relevo ainda se lê",
        m.vert_count()
    );

    // ⛔⛔ **A SEGUNDA METADE, e ela nasceu de uma mutação SOBREVIVENTE:** a de
    // cima mede a PORTA e é cega ao FIO. Trocar o despacho para devolver o
    // default do módulo deixava-a **verde** — *um gate que chama a função em vez
    // de percorrer a rota afirma que a peça certa existe, nunca que a cena a
    // usa*. É o ponto cego que o `CLAUDE.md` §5.0 nomeia: nenhuma sonda deste
    // repo pergunta se o VALOR chega a um consumidor.
    //
    // ⚠️ **`include_str!` e não `read_to_string`**, pela lição do HOWTO §2: o
    // gémeo em runtime só falha **quando o teste corre**, e este ficheiro tem de
    // deixar de COMPILAR no dia em que o irmão mudar de sítio.
    let despacho = include_str!("scenes_mesh.rs");
    let rota = despacho
        .lines()
        .skip_while(|l| !l.contains("erase_scene() || crate::scenes::smear::smear_scene()"))
        .take(3)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        rota.contains("peca_de_multirresolucao()"),
        "as duas cenas de multirresolução deixaram de rotear para a peça \
         grossa — elas voltaram a cair no default do módulo, e o roteiro delas \
         fabrica 1 572 866 vértices outra vez.\n\nrota lida:\n{rota}"
    );
}
