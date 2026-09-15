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

/// ⛔⛔ **A GEOMETRIA DO REPORT DO DONO, guardada como FIXTURA.**
///
/// Era a altura com que a `=45` shipou em 2026-09-14, e a cena de fábrica
/// mudou-a para `+0,40` depois de a régua medir que ali o barro viaja `5,7`–`7,9`
/// raios de pincel (ver [`super::ALTURA_DA_PLACA`]).
///
/// ⚠️⚠️ **Ela FICA, e não como história:** a lei do pick é sobre o **pincel** e
/// não sobre esta cena — nada impede o artista de pôr o alvo longe, e é
/// exactamente aí que o cursor persegue o barro que ele próprio mandou embora.
/// *Uma cena corrigida deixa de conter o fenómeno, e um gate cuja fixtura
/// deixou de o conter não afirma nada* — foi isto que a cura da cena provocou:
/// o controlo deste gate passou a ler `0,24` onde antes lia `1,22`, e a
/// medição disse-o em voz alta.
const ALTURA_DO_REPORT: f32 = -1.25;

/// A cena `=45` montada como o produto a monta: a peça de fábrica mais a placa.
fn cena_45(device: &wgpu::Device) -> Sculpt3dScene {
    cena(device, super::ALTURA_DA_PLACA)
}

/// A mesma cena com a placa a uma altura DADA — ver [`ALTURA_DO_REPORT`].
fn cena(device: &wgpu::Device, altura: f32) -> Sculpt3dScene {
    let mut s = Sculpt3dScene::new(device, ph2d_mesh::shapes::sculpt_sphere(1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    // ⚠️ A placa é montada pela MESMA porta da cena, mas sem passar pela env:
    // um teste que arma uma variável de ambiente corre contra os vizinhos.
    s.push_object(
        super::placa(),
        ph2d_mesh::Pose::new([0.0, 0.0, altura], 1.0),
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

/// ⭐⭐ **A FOTOGRAFIA É DECIDIDA PELO VERBO, e a decisão é do PRODUTO** — as
/// duas metades de
/// [`Sculpt3dScene::fotografa_a_superficie_do_pen_down`](crate::Sculpt3dScene).
///
/// ⛔ **O lado NEGATIVO é metade do gate:** congelar o pick de um verbo que não
/// foge trocaria o cursor por um fantasma sem comprar nada, e uma fotografia
/// da malha inteira por traço é o preço mais caro deste módulo.
#[test]
#[ignore]
fn so_o_projectar_fotografa_a_superficie_do_pen_down() {
    let gpu = gpu_or_skip!();
    let mut s = cena_45(&gpu.device);
    s.fotografa_a_superficie_do_pen_down();
    assert!(
        s.superficie_do_pen_down.is_some(),
        "o projectar tem de picar contra a superfície do pen-down — sem ela o \
         cursor persegue o barro que ele próprio mandou embora"
    );
    // ⭐ O CONTROLO, e com ele o `None` que o pen-down tem de escrever SEMPRE:
    // sem esta escrita a fotografia de um traço sobreviveria ao seguinte.
    s.brush.verb = Verb::Draw;
    s.fotografa_a_superficie_do_pen_down();
    assert!(
        s.superficie_do_pen_down.is_none(),
        "um verbo que desloca uma FRACÇÃO do raio não paga uma cópia da malha \
         por traço — e uma fotografia que fica viva envenena o traço seguinte"
    );
    // ⭐ **E a fotografia MORRE COM O TRAÇO** — ela é uma cópia da malha
    // inteira, e o `close_stroke` é a porta que a larga. ⚠️ A linha vive ANTES
    // dos dois `return` dele, e é isso que este gate prende: um traço que
    // devolve cedo (a topologia mudou · a janela está vazia) larga-a na mesma.
    s.brush.verb = Verb::SceneProject;
    s.fotografa_a_superficie_do_pen_down();
    s.close_stroke();
    assert!(
        s.superficie_do_pen_down.is_none(),
        "o fecho do traço tem de largar a cópia da malha — segurá-la fora do \
         gesto é uma segunda peça em memória por nada"
    );
}

/// ⛔⛔ **O PEN-DOWN DO PRODUTO CHAMA A PORTA** — a metade que nenhum teste
/// pode exercitar, porque o `input_down` pede um `AppHost`.
///
/// ⚠️ **`include_str!` e não `read_to_string`:** o gémeo em runtime só falha
/// **quando o teste corre**, e um `#[ignore]` ou um filtro deixam-no mudo para
/// sempre; assim, se o irmão mudar de sítio isto **deixa de compilar**
/// (`HOWTO §2.6`, a espécie que fica VERDE ao mover código).
#[test]
fn o_pen_down_fotografa_a_superficie() {
    const PEN_DOWN: &str = include_str!("input_down.rs");
    assert!(
        PEN_DOWN.contains("scene.fotografa_a_superficie_do_pen_down();"),
        "o pen-down deixou de fotografar a superfície: o projectar volta a picar \
         contra a malha que ele próprio acabou de mover"
    );
}

/// **SONDA — O PREÇO DA FOTOGRAFIA**, que é o único custo novo desta cura.
///
/// ⚠️ **Ela corre UMA vez por traço, no pen-down** — ao lado da cópia que a
/// [`ph2d_sculpt3d::SculptStroke::pecas_da_cena`] já paga no mesmo instante —,
/// logo o orçamento dela não é o do dab (`8 ms`): é o do **gesto que começa**.
/// ⛔ Corra em `--release`: em debug a cópia lê-se ~20× mais lenta e o número
/// descreveria outro programa.
#[test]
#[ignore]
fn diag_o_preco_da_fotografia() {
    let gpu = gpu_or_skip!();
    let mut s = cena_45(&gpu.device);
    for passo in 0..3 {
        // A mediana de cinco: a primeira cópia paga o aquecimento do alocador.
        let mut t: Vec<f64> = (0..5)
            .map(|_| {
                let t0 = std::time::Instant::now();
                s.fotografa_a_superficie_do_pen_down();
                t0.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        t.sort_by(f64::total_cmp);
        eprintln!(
            "  nivel {passo} · {:>8} verts · {:>8} faces -> {:.3} ms (min {:.3})",
            s.mesh().vert_count(),
            s.mesh().face_count(),
            t[2],
            t[0]
        );
        if !s.subdivide() {
            break;
        }
    }
}

/// **SONDA — A RÉGUA DA CENA: quantos raios de pincel o barro tem de viajar?**
///
/// ⭐⭐⭐ **É a grandeza que decide se esta cena ENSINA o pincel ou o difama.** O
/// deslocamento de um dab é `d · peso · força²`, e o `d` é uma distância da
/// **CENA** — nada na lei o compara com o pincel. Com `d ≫ R` o barro sai do
/// alcance do próprio carimbo e o que o artista vê é uma língua de barro a
/// atravessar a peça; com `d ≈ R` ele vê a superfície **encostar** no alvo, que
/// é o que a ferramenta faz.
///
/// Ela varre a coluna do meio do canvas e imprime, por pixel, onde o raio
/// aterra e o `d/R` dali.
#[test]
#[ignore]
fn diag_a_regua_da_cena() {
    let gpu = gpu_or_skip!();
    let mut s = cena_45(&gpu.device);
    assert!(s.aim(CENTRE.0, CENTRE.1), "o raio errou a peca");
    s.stroke.pecas_da_cena.clear();
    let activo = s.active;
    for (i, o) in s.objects.iter().enumerate() {
        if i != activo {
            s.stroke
                .pecas_da_cena
                .push((o.stack.mesh().clone(), o.pose));
        }
    }
    s.stroke.pose_activa = s.objects[s.active].pose;
    let raio_pincel = s.armed_brush([0.0, 0.0, 0.0]).radius;
    eprintln!("raio do pincel em espaco de objecto: {raio_pincel:.4}");
    eprintln!("   y px | acerto na peca              |      d | d/R");
    for k in 0..13 {
        let y = 120.0 + 40.0 * k as f32;
        let Some(h) = s.pick_do_dab(CENTRE.0, y) else {
            eprintln!("  {y:6.0} | (fora da peca)");
            continue;
        };
        let raio = s.ray_at(CENTRE.0, y);
        let d = ph2d_sculpt3d::distancia_de_projeccao_para_teste(
            h.point,
            raio.dir(),
            s.stroke.pose_activa,
            &s.stroke.pecas_da_cena,
            s.brush.project_bidirectional,
            s.brush.project_min_distance,
        );
        match d {
            Some(d) => eprintln!(
                "  {y:6.0} | [{:6.2},{:6.2},{:6.2}] | {d:6.3} | {:5.2}",
                h.point[0],
                h.point[1],
                h.point[2],
                d / raio_pincel
            ),
            None => eprintln!(
                "  {y:6.0} | [{:6.2},{:6.2},{:6.2}] | (o raio nao acerta na placa)",
                h.point[0], h.point[1], h.point[2]
            ),
        }
    }
}

/// **O QUE UM TRAÇO DEIXOU** — a saída do arnês, para os dois lados poderem ser
/// comparados pelo mesmo código.
struct Traco {
    /// Onde cada dab ATERROU, na ordem em que foram carimbados.
    centros: Vec<[f32; 3]>,
    /// Quantos vértices se mexeram.
    movidos: usize,
    /// O maior deslocamento de um vértice.
    pior: f32,
    /// ⭐ **O RASGO** — a maior diferença de deslocamento entre dois vértices
    /// VIZINHOS. É esta a grandeza da foto do dono: um degrau entre vizinhos é
    /// uma parede onde devia haver superfície.
    salto: f32,
}

impl Traco {
    /// O maior passo entre dois dabs CONSECUTIVOS.
    ///
    /// ⭐⭐ **É a régua do mecanismo, e não do sintoma:** o traço anda `9` px por
    /// evento, logo dois dabs seguidos têm de cair **ao lado um do outro**. Um
    /// passo de mais de um raio de pincel quer dizer que o cursor perdeu a
    /// superfície e foi parar a outro sítio da peça.
    fn passo_maximo(&self) -> f32 {
        self.centros
            .windows(2)
            .map(|w| {
                let d = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
            })
            .fold(0.0f32, f32::max)
    }
}

/// **UM TRAÇO DE PROJECTAR NA `=45`**, pelo caminho do produto, com o pen-down
/// reproduzido passo a passo.
///
/// ⚠️⚠️ **O `congelar` é o CONTRAFACTUAL e não um knob de produto:** no app quem
/// responde é [`ph2d_sculpt3d::Verb::pica_na_superficie_do_pen_down`], e para
/// este verbo ela é `true` sempre. Aqui ele existe para o gate poder medir os
/// **dois** lados — *uma cura sem o lado de antes não é uma medição, é uma
/// afirmação*.
fn traco(device: &wgpu::Device, congelar: bool, dabs: u32, altura: f32) -> Traco {
    let mut s = cena(device, altura);
    let antes: Vec<[f32; 3]> = s.mesh().positions().to_vec();
    assert!(s.aim(CENTRE.0, CENTRE.1), "o raio errou a peca");
    s.stroke.begin(s.objects[s.active].stack.mesh());
    s.open_dyntopo_stroke();
    s.open_reference_stroke();
    // ⚠️⚠️ **AS TRÊS LINHAS DO PEN-DOWN que este arnês tem de repetir** — no
    // produto quem as corre é o `input_down`, que precisa de um `AppHost`.
    // ⛔ Sem as duas primeiras o pincel é inerte **e a razão é a lei**, não um
    // defeito: a primeira corrida desta sonda leu `0` vértices movidos e eu
    // quase o registei como defeito de produto. *Um arnês a que falta um passo
    // do pen-down mede outro programa* — a terceira vez que este módulo o paga.
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
    // ⭐ **A TERCEIRA: a superfície contra a qual este traço PICA — e ela vai
    // pela PORTA DO PRODUTO**, senão uma mutação no predicado do verbo deixava
    // este gate verde (*um arnês que monta o estado à mão mede outro programa*).
    s.fotografa_a_superficie_do_pen_down();
    if !congelar {
        s.superficie_do_pen_down = None;
    }

    let mut centros = Vec::new();
    for k in 0..dabs {
        let x = CENTRE.0 + 9.0 * k as f32;
        if let Some(h) = s.pick_do_dab(x, CENTRE.1) {
            centros.push(h.point);
        }
        s.sculpt_at(x, CENTRE.1);
    }
    s.close_stroke();

    let depois = s.mesh().positions();
    // ⚠️ **Só os vértices que já existiam**: com a topologia dinâmica armada a
    // malha cresce, e comparar por índice para além do fim do `antes` mediria
    // um vértice contra outro.
    let desloc: Vec<f32> = antes
        .iter()
        .zip(depois)
        .map(|(a, b)| (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0f32, f32::max))
        .collect();
    let movidos = desloc.iter().filter(|d| **d > 1e-6).count();
    let pior = desloc.iter().copied().fold(0.0f32, f32::max);
    let viz = s.mesh().adjacency().vert_verts.clone();
    let mut salto = 0.0f32;
    for v in 0..desloc.len() {
        for &u in viz.neighbours(v) {
            let u = u as usize;
            if u < desloc.len() {
                salto = salto.max((desloc[v] - desloc[u]).abs());
            }
        }
    }
    Traco {
        centros,
        movidos,
        pior,
        salto,
    }
}

/// ⭐⭐⭐ **A CURA DO REPORT DO DONO: um traço trabalha UMA superfície — a que
/// ele viu quando a caneta encostou.**
///
/// As duas metades, e nenhuma basta sozinha:
///
/// 1. **o mecanismo** — com a fotografia armada, dois dabs consecutivos caem ao
///    lado um do outro; sem ela, o cursor atravessa a peça e aterra no outro
///    lado (é isto que produz as crateras sem relação);
/// 2. **o sintoma** — o RASGO entre vizinhos desaba.
///
/// ⛔⛔ **O lado SEM a cura é o controlo, e é ele que torna este gate honesto:**
/// sem essa metade, uma cura que não fizesse nada leria verde — as duas colunas
/// seriam a mesma. *Uma barra calibrada sem o lado reprovado mede a nossa
/// imaginação.*
///
/// ⚠️ **As barras saem da MEDIÇÃO desta cena** (impressa pela sonda irmã), com
/// margem, e são RAZÕES contra o raio do pincel — nunca comprimentos absolutos,
/// que mudariam com o enquadramento.
#[test]
#[ignore]
fn o_traco_do_projectar_fica_numa_superficie_so() {
    let gpu = gpu_or_skip!();
    let vivo = traco(&gpu.device, false, 6, ALTURA_DO_REPORT);
    let congelado = traco(&gpu.device, true, 6, ALTURA_DO_REPORT);
    // O raio do pincel em espaço de objecto, para as barras serem razões.
    let raio = {
        let mut s = cena(&gpu.device, ALTURA_DO_REPORT);
        assert!(s.aim(CENTRE.0, CENTRE.1));
        s.armed_brush([0.0, 0.0, 0.0]).radius
    };
    eprintln!(
        "raio {raio:.4} · passo vivo {:.4} congelado {:.4} · salto vivo {:.4} \
         congelado {:.4} · pior vivo {:.4} congelado {:.4}",
        vivo.passo_maximo(),
        congelado.passo_maximo(),
        vivo.salto,
        congelado.salto,
        vivo.pior,
        congelado.pior
    );

    // (1) O MECANISMO, com o controlo primeiro.
    assert!(
        vivo.passo_maximo() > 2.0 * raio,
        "o CONTROLO não reproduz o defeito: sem a fotografia o cursor tinha de \
         saltar mais de dois raios ({raio:.4}) entre dabs, e saltou {:.4} — sem \
         isto a metade de baixo deste gate não afirma nada",
        vivo.passo_maximo()
    );
    assert!(
        congelado.passo_maximo() < 0.5 * raio,
        "com a superfície do pen-down fotografada, dois dabs a 9 px um do outro \
         têm de cair ao lado um do outro: passo {:.4} contra um raio de {raio:.4}",
        congelado.passo_maximo()
    );

    // (2) O SINTOMA — o rasgo.
    assert!(
        congelado.salto < vivo.salto / 4.0,
        "o rasgo entre vizinhos tinha de desabar: {:.4} contra {:.4}",
        congelado.salto,
        vivo.salto
    );

    // (3) ⭐ **E o pincel continua a PROJECTAR** — sem esta linha, um pick que
    // não acertasse em nada passaria as duas de cima com folga.
    assert!(
        congelado.movidos > 1000 && congelado.pior > raio,
        "o traço curado tem de continuar a levar barro até à placa: {} vértices, \
         pior {:.4}",
        congelado.movidos,
        congelado.pior
    );
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
    s.fotografa_a_superficie_do_pen_down();
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
