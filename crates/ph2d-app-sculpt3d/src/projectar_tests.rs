//! **O PINCEL DE PROJECTAR, no GESTO e na CENA** — o que a bancada do kernel não
//! pode ver: a câmara de fábrica, a pose das peças e o que o artista carrega.
//!
//! ```text
//! cargo test -p ph2d-app-sculpt3d --lib projectar -- --ignored --nocapture
//! ```
//!
//! # ⛔⛔⛔ O REPORT DO DONO (`=45`, 2026-09-14): *«resultado bem bizarro»*
//!
//! A peça saiu **rasgada** — golpes longos e escuros a atravessar a superfície.
//! Esta sonda reproduziu-o e isolou o mecanismo em quatro medições:
//!
//! | o que se varia | salto entre VIZINHOS |
//! |---|---|
//! | o traço inteiro (6 dabs) | **`0,78`** — a aresta média é `0,008` ⇒ **97×** |
//! | **UM** dab só | `0,038` |
//! | a máscara de alcance desligada | `0,603` (era `0,607`) ⇒ **não é ela** |
//! | o pincel 4× mais pequeno | **`0,92`** ⇒ **piora** |
//! | a força a `0,1` | `0,11` |
//!
//! ⭐⭐⭐ **O MECANISMO, e ele está nos pontos onde cada dab ATERRA:**
//!
//! ```text
//! dab 0 -> [ 0,59,  0,38,  0,74]   a frente da bola
//! dab 1 -> [ 0,00, -0,03, -0,24]   ⛔ o MIOLO — nove pixels depois
//! dab 2 -> [ 0,08, -0,02, -0,24]
//! dab 3 -> [ 0,41,  0,17,  0,16]   e volta
//! ```
//!
//! **Um dab deste pincel move a superfície MAIS do que o raio do próprio
//! pincel** (`1,15` com um raio de `0,35`), logo *a superfície foge de debaixo
//! do cursor*: o raio do evento seguinte passa pelo buraco onde ela estava e
//! acerta no outro lado da peça. Seis dabs = seis crateras em sítios sem
//! relação, e as fronteiras entre elas **são** os golpes da foto.
//!
//! ⚠️⚠️ **E o corpus do oráculo NÃO PODE responder a isto:** as `24` fixturas
//! correm sobre um **plano chato visto de frente**, onde a superfície se afasta
//! **ao longo do próprio raio** — ali o cursor nunca a perde. *Uma paridade
//! medida numa fixtura plana não afirma nada sobre uma peça curva.*
//!
//! ⚠️ É a MESMA família que o polegar já pagou nesta casa (*um gesto que
//! desloca o barro quase um raio leva os próprios vértices para fora da
//! consulta*) — mas um nível acima: ali era a PEGADA que fugia, aqui é o
//! **PICK**.

use ph2d_sculpt3d::Verb;

use crate::Sculpt3dScene;

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

const CENTRE: (f32, f32) = (450.0, 350.0);

/// A cena `=45` montada como o produto a monta: a peça de fábrica mais a placa.
fn cena_45(device: &wgpu::Device) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, ph2d_mesh::shapes::sculpt_sphere(1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    // ⚠️ A placa é montada pela MESMA porta da cena, mas sem passar pela env:
    // um teste que arma uma variável de ambiente corre contra os vizinhos.
    s.push_object(
        super::placa(),
        ph2d_mesh::Pose::new([0.0, 0.0, super::ALTURA_DA_PLACA], 1.0),
    );
    s.frame_all(900.0 / 700.0);
    s.brush.verb = Verb::SceneProject;
    if std::env::var("SEM_MASCARA").is_ok() {
        s.brush.surface_only = false;
    }
    if let Some(f) = std::env::var("FORCA").ok().and_then(|v| v.parse().ok()) {
        s.brush.strength = f;
    }
    if let Some(r) = std::env::var("RAIO_PX").ok().and_then(|v| v.parse().ok()) {
        s.radius_px = r;
    }
    s
}

/// **SONDA** — o que um traço de projectar faz à peça de fábrica.
#[test]
#[ignore]
fn diag_o_traco_do_dono() {
    let gpu = gpu_or_skip!();
    let mut s = cena_45(&gpu.device);
    let antes: Vec<[f32; 3]> = s.mesh().positions().to_vec();
    eprintln!(
        "peca: {} verts · pecas na cena: {} · activa: {}",
        antes.len(),
        s.objects.len(),
        s.active
    );
    eprintln!(
        "caixa do mundo: {:?} .. {:?}",
        s.world_bounds().min,
        s.world_bounds().max
    );
    let raio = s.ray_at(CENTRE.0, CENTRE.1);
    eprintln!("olho (mundo): {:?}", raio.dir());
    assert!(s.aim(CENTRE.0, CENTRE.1), "o raio errou a peca");
    let b = s.armed_brush([0.0, 0.0, 0.0]);
    eprintln!(
        "RAIO do pincel em espaco de OBJECTO: {:.4}  (a peca tem raio 1,0)",
        b.radius
    );
    s.stroke.begin(s.objects[s.active].stack.mesh());
    s.open_dyntopo_stroke();
    s.open_reference_stroke();
    // ⚠️⚠️ **AS DUAS LINHAS DO PEN-DOWN que este arnês tem de repetir** — no
    // produto quem as corre é o `input_down`, que precisa de um `AppHost`.
    // ⛔ Sem elas o pincel é inerte **e a razão é a lei**, não um defeito: a
    // primeira corrida desta sonda leu `0` vértices movidos e eu quase o
    // registei como defeito de produto. *Um arnês a que falta um passo do
    // pen-down mede outro programa* — a terceira vez que este módulo o paga.
    s.stroke.pecas_da_cena.clear();
    if s.brush.precisa_das_pecas_da_cena() {
        let activo = s.active;
        for (i, o) in s.objects.iter().enumerate() {
            if i != activo {
                s.stroke
                    .pecas_da_cena
                    .push((o.stack.mesh().clone(), o.pose));
            }
        }
    }
    s.stroke.pose_activa = s.objects[s.active].pose;
    eprintln!(
        "alvos fotografados: {} · pose activa {:?} escala {}",
        s.stroke.pecas_da_cena.len(),
        s.stroke.pose_activa.translation,
        s.stroke.pose_activa.scale()
    );
    let dabs: u32 = std::env::var("DABS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    for k in 0..dabs {
        let x = CENTRE.0 + 9.0 * k as f32;
        let onde = s.pick(x, CENTRE.1).map(|(i, h)| (i, h.point));
        s.sculpt_at(x, CENTRE.1);
        eprintln!(
            "  dab {k} em px ({x:.0},{:.0}) -> acerto {onde:?}",
            CENTRE.1
        );
    }
    eprintln!("dabs: {dabs}");
    s.close_stroke();

    let depois = s.mesh().positions();
    let mut movidos = 0usize;
    let mut pior = 0.0f32;
    let mut soma = 0.0f64;
    for (a, b) in antes.iter().zip(depois) {
        let d = (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0f32, f32::max);
        if d > 1e-6 {
            movidos += 1;
            soma += f64::from(d);
            pior = pior.max(d);
        }
    }
    eprintln!(
        "movidos {movidos} · deslocamento PIOR {pior:.4} · MEDIO {:.4}",
        if movidos > 0 {
            soma / movidos as f64
        } else {
            0.0
        }
    );
    // O rasgo: a maior diferenca de deslocamento entre VIZINHOS.
    let viz = s.mesh().adjacency().vert_verts.clone();
    let desloc: Vec<f32> = antes
        .iter()
        .zip(depois)
        .map(|(a, b)| (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0f32, f32::max))
        .collect();
    let (mut salto, mut par) = (0.0f32, (0usize, 0usize));
    for v in 0..desloc.len() {
        for &u in viz.neighbours(v) {
            let d = (desloc[v] - desloc[u as usize]).abs();
            if d > salto {
                salto = d;
                par = (v, u as usize);
            }
        }
    }
    eprintln!("SALTO maximo entre vizinhos: {salto:.4}  (a aresta media e' ~0,008)");
    let (v, u) = par;
    eprintln!(
        "  o par: v{v} em {:?} andou {:.4}; v{u} em {:?} andou {:.4}",
        antes[v], desloc[v], antes[u], desloc[u]
    );
    // O que a lei diz para cada um deles, isolada.
    for (nome, i) in [("v", v), ("u", u)] {
        let d = ph2d_sculpt3d::distancia_de_projeccao_para_teste(
            antes[i],
            raio.dir(),
            s.stroke.pose_activa,
            &s.stroke.pecas_da_cena,
            s.brush.project_bidirectional,
            s.brush.project_min_distance,
        );
        eprintln!("  distancia crua de {nome}{i}: {d:?}");
    }
}
