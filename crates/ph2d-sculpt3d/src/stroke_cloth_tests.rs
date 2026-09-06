//! ⭐⭐⭐ **O PINCEL DE TECIDO** — e estes gates existem porque ele SAIU de dois
//! censos.
//!
//! ⚠️ **Tirar um verbo de um censo é uma dívida até ele ter os próprios gates.**
//! O tecido não escreve `accum` nem `target` (ele desvia antes do laço), então
//! os censos do `stroke_apply` afirmariam vácuo sobre ele — a exclusão está feita
//! por LEI ([`Verb::writes_through_applicator`]) e o que se cobra dele está aqui.

use super::*;
use ph2d_mesh::Mesh;

fn plano() -> Mesh {
    // Uma grade PLANA, e não uma esfera: um pano tem de poder fazer prega, e numa
    // esfera de raio 1 a curvatura do repouso domina o que a régua vê.
    const N: usize = 24;
    let s = 2.0 / N as f32;
    let mut pos = Vec::new();
    for j in 0..=N {
        for i in 0..=N {
            pos.push([i as f32 * s - 1.0, j as f32 * s - 1.0, 0.0]);
        }
    }
    let id = |i: usize, j: usize| u32::try_from(j * (N + 1) + i).unwrap_or(u32::MAX);
    let mut faces = Vec::new();
    for j in 0..N {
        for i in 0..N {
            faces.push(ph2d_mesh::Face::quad(
                id(i, j),
                id(i + 1, j),
                id(i + 1, j + 1),
                id(i, j + 1),
            ));
        }
    }
    Mesh::from_parts(pos, faces).expect("grade plana")
}

fn pincel() -> Brush {
    Brush {
        verb: Verb::Cloth,
        radius: 0.30,
        strength: 1.0,
        ..Brush::default()
    }
}

/// Um dab olhando de `+z` para o plano.
///
/// ⚠️ **`Dab::hooking`, que é EXATAMENTE o que o shell manda** (`hook_step` ⇒
/// `Dab::hooking(center, radius, eye, step)`). A 1.ª redação usava `Dab::at`, e
/// a diferença não muda a lei do tecido — mas uma fixtura que manda outro dab
/// que o produto é uma fixtura que pode ficar verde sobre um caminho que
/// ninguém percorre.
fn dab_em(c: [f32; 3], r: f32, passo: [f32; 3]) -> Dab {
    Dab::hooking(c, r, [0.0, 0.0, -1.0], passo)
}

/// Arrasta a mão por `passos` eventos, devolvendo a malha e o traço.
fn arrastar(passos: usize, brush: &Brush) -> (Mesh, SculptStroke) {
    let mut mesh = plano();
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..passos {
        let c = [0.02 * k as f32, 0.0, 0.0];
        let passo = if k == 0 { [0.0; 3] } else { [0.02, 0.0, 0.0] };
        s.dab(
            &mut mesh,
            brush,
            &dab_em(c, brush.radius, passo),
            Symmetry::default(),
        );
    }
    (mesh, s)
}

fn desloc(a: &Mesh, b: &Mesh, v: usize) -> f32 {
    let (p, q) = (a.positions()[v], b.positions()[v]);
    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
}

/// ⭐⭐⭐ **GATE — encostar sem mover NÃO deforma.**
///
/// ⚠️ **É a razão de o tecido ter saído dos dois censos do aplicador**, e é
/// também produto correto: o pano responde à VELOCIDADE da mão, então pousar o
/// dedo e não andar não tem por que fazer prega. Os outros 23 verbos carimbam no
/// primeiro dab; este não, e a diferença é a lei, não um buraco.
#[test]
fn encostar_sem_mover_nao_deforma() {
    let antes = plano();
    let (depois, _) = arrastar(1, &pincel());
    let pior = (0..antes.vert_count())
        .map(|v| desloc(&antes, &depois, v))
        .fold(0.0f32, f32::max);
    assert_eq!(pior, 0.0, "um dab parado deformou o pano");
}

/// ⭐⭐⭐ **GATE — arrastar move o MIOLO, e o anel pregado NÃO se mexe.**
///
/// ⛔ O anel é a feature: é ele que faz a transição para o resto da escultura
/// não estourar. Um pregado que escorrega escorrega visivelmente num traço de
/// mil eventos.
#[test]
fn arrastar_move_o_miolo_e_o_anel_fica() {
    let antes = plano();
    let (depois, _) = arrastar(12, &pincel());
    // ⚠️ **A cerca sai da LEI, não de um número.** A área simulada da lei da
    // referência acaba em `R(1 + L)` com `L` = *Simulation Limit* — e não em
    // `2R`, que era a cerca da lei VBD e mordeu quando esta passou a ser a de
    // omissão. Fora de `R(1+L)` o peso de banda é ZERO, logo o pano não se
    // pode mexer ali, e é isso que se afirma.
    let alcance =
        f64::from(pincel().radius) * (1.0 + ph2d_cloth::verlet_gesto::Pincel::default().limite);
    #[allow(clippy::cast_possible_truncation)]
    let fora = alcance as f32;
    let mut miolo = 0.0f32;
    let mut longe = 0.0f32;
    let mut contados = 0usize;
    for v in 0..antes.vert_count() {
        let p = antes.positions()[v];
        let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
        // ⚠️ **A distância é ao CAMINHO, não à origem** — a área anda com o
        // cursor, e medir da origem daria por «fora» pontos que o pincel visitou
        // no fim do traço. O caminho é o mesmo que [`arrastar`] percorre.
        let ao_caminho = (0..12)
            .map(|k| {
                let c = [0.02 * k as f32, 0.0, 0.0];
                ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt()
            })
            .fold(f32::MAX, f32::min);
        let d = desloc(&antes, &depois, v);
        if r < 0.15 {
            miolo = miolo.max(d);
        } else if ao_caminho > fora {
            contados += 1;
            longe = longe.max(d);
        }
    }
    assert!(miolo > 1e-3, "o pano nao respondeu ao arrasto: {miolo:.3e}");
    // Anti-vácuo: tem de haver vértices ALÉM da cerca, senão «nada se moveu lá»
    // é verdade por não haver «lá».
    assert!(
        contados > 20,
        "so' {contados} vertices alem de {fora:.2} -- a fixtura nao alcanca a cerca"
    );
    assert_eq!(longe, 0.0, "o pano moveu fora da regiao ({fora:.2}): {longe:.3e}");
}

/// ⭐⭐⭐ **GATE — o pano responde FORA da pegada, e é isso que o separa de um Grab.**
///
/// ⚠️⚠️ **É a propriedade que justifica a feature inteira.** Um Grab move
/// exatamente quem está sob o dedo e mais nada; um tecido é arrastado pela
/// MEMBRANA e pela DOBRA, então a vizinhança anda junto. Sem isto não há prega —
/// há um Grab com outro nome.
#[test]
fn o_pano_responde_fora_da_pegada() {
    let antes = plano();
    let b = pincel();
    let (depois, _) = arrastar(12, &b);
    let mut fora = 0.0f32;
    for v in 0..antes.vert_count() {
        let p = antes.positions()[v];
        let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
        // Fora da pegada (`radius`), dentro da região (`radius × 2`) e antes do
        // anel pregado (`× 0,7`).
        if r > b.radius * 1.05 && r < b.radius * 2.0 * 0.65 {
            fora = fora.max(desloc(&antes, &depois, v));
        }
    }
    assert!(
        fora > 1e-4,
        "so' a pegada se moveu -- isto e' um Grab, nao um tecido: {fora:.3e}"
    );
}

/// ⭐⭐⭐ **GATE — a MÁSCARA protege o que o artista protegeu.**
///
/// ⛔ Um pincel de tecido que ignorasse a máscara destruiria a região que o
/// artista mascarou — e ele a lê pela MESMA porta dos outros verbos
/// (`mask_ops::free_weight`, no `pre` congelado).
#[test]
fn a_mascara_protege() {
    let mut mesh = plano();
    // Mascara TUDO: nenhum vértice pode sentir força.
    let n = mesh.vert_count();
    mesh.masks_mut()[..n].fill(1.0);
    let antes = mesh.clone();
    let b = pincel();
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..12 {
        let c = [0.02 * k as f32, 0.0, 0.0];
        let passo = if k == 0 { [0.0; 3] } else { [0.02, 0.0, 0.0] };
        s.dab(
            &mut mesh,
            &b,
            &dab_em(c, b.radius, passo),
            Symmetry::default(),
        );
    }
    let pior = (0..n)
        .map(|v| desloc(&antes, &mesh, v))
        .fold(0.0f32, f32::max);
    assert_eq!(pior, 0.0, "a mascara nao segurou o tecido: {pior:.3e}");
    // Controle: SEM máscara o mesmo gesto move.
    let (livre, _) = arrastar(12, &b);
    let base = plano();
    let move_ = (0..n)
        .map(|v| desloc(&base, &livre, v))
        .fold(0.0f32, f32::max);
    assert!(
        move_ > 1e-3,
        "o controle nao se mexeu: o gate mediria vacuo"
    );
}

/// ⭐⭐⭐ **GATE — a janela de upload não sai vazia, nem herdada.**
///
/// ⛔⛔ **O segundo modo de *«a malha mudou e a tela não»*.** O
/// [`SculptStroke::last_gpu_dirty`] escolhe entre duas listas pela bandeira
/// `last_paints_mask`, e o `begin` **não a reinicia**: um traço de tecido logo
/// depois de um traço de MÁSCARA leria a bandeira do outro. Aqui o gate encena
/// exatamente essa ordem.
#[test]
fn a_janela_de_upload_nao_e_herdada_do_traco_anterior() {
    let mut mesh = plano();
    let mut s = SculptStroke::default();
    // Um traço de MÁSCARA primeiro — é ele que arma a bandeira do outro lado.
    let mascara = Brush {
        verb: Verb::Mask,
        radius: 0.30,
        strength: 1.0,
        ..Brush::default()
    };
    s.begin(&mesh);
    for k in 0..4 {
        let c = [0.02 * k as f32, 0.0, 0.0];
        let passo = if k == 0 { [0.0; 3] } else { [0.02, 0.0, 0.0] };
        s.dab(
            &mut mesh,
            &mascara,
            &dab_em(c, mascara.radius, passo),
            Symmetry::default(),
        );
    }
    // Agora o tecido, no MESMO traço-objeto.
    let b = pincel();
    s.begin(&mesh);
    for k in 0..12 {
        let c = [0.02 * k as f32, 0.0, 0.0];
        let passo = if k == 0 { [0.0; 3] } else { [0.02, 0.0, 0.0] };
        s.dab(
            &mut mesh,
            &b,
            &dab_em(c, b.radius, passo),
            Symmetry::default(),
        );
    }
    assert!(
        !s.last_gpu_dirty().is_empty(),
        "a janela de upload do tecido saiu VAZIA -- a malha muda e a tela nao"
    );
}

/// ⭐⭐⭐ **GATE — todo vértice que o pano moveu foi CAPTURADO.**
///
/// ⛔⛔ É o contrato do undo. O `Ctrl+Z` repõe o `base_pos` dos vértices
/// capturados; um vértice que a simulação move sem ter sido capturado é um
/// vértice que o desfazer **não sabe repor** — e o defeito só apareceria depois,
/// como uma mossa que não sai.
#[test]
fn todo_vertice_movido_foi_capturado() {
    let antes = plano();
    let (depois, s) = arrastar(12, &pincel());
    let mut moveu = 0usize;
    for v in 0..antes.vert_count() {
        if desloc(&antes, &depois, v) > 0.0 {
            moveu += 1;
            assert!(
                s.touched().contains(&u32::try_from(v).unwrap_or(u32::MAX)),
                "o vertice {v} moveu e nao foi capturado -- o undo nao o repoe"
            );
        }
    }
    assert!(moveu > 10, "o gate mediria vacuo: so' {moveu} vertices");
}

/// ⭐⭐⭐ **GATE — um traço novo não herda a região do anterior.**
///
/// ⚠️ A região é medida no pen-down e morre no pen-up. Herdá-la faria o traço
/// seguinte simular onde o artista já não está — e, pior, sobre um repouso que
/// descreve a peça de antes do primeiro traço.
#[test]
fn um_traco_novo_nao_herda_a_regiao() {
    let mut mesh = plano();
    let b = pincel();
    let mut s = SculptStroke::default();
    // Primeiro traço, num canto.
    s.begin(&mesh);
    for k in 0..8 {
        let c = [-0.6 + 0.02 * k as f32, -0.6, 0.0];
        let passo = if k == 0 { [0.0; 3] } else { [0.02, 0.0, 0.0] };
        s.dab(
            &mut mesh,
            &b,
            &dab_em(c, b.radius, passo),
            Symmetry::default(),
        );
    }
    let entre = mesh.clone();
    // Segundo traço, no canto OPOSTO.
    s.begin(&mesh);
    for k in 0..8 {
        let c = [0.6 + 0.02 * k as f32, 0.6, 0.0];
        let passo = if k == 0 { [0.0; 3] } else { [0.02, 0.0, 0.0] };
        s.dab(
            &mut mesh,
            &b,
            &dab_em(c, b.radius, passo),
            Symmetry::default(),
        );
    }
    // O canto do PRIMEIRO traço não pode ter andado no segundo.
    let mut velho = 0.0f32;
    for v in 0..mesh.vert_count() {
        let p = entre.positions()[v];
        if p[0] < -0.3 && p[1] < -0.3 {
            velho = velho.max(desloc(&entre, &mesh, v));
        }
    }
    assert_eq!(velho, 0.0, "o 2o traco mexeu na regiao do 1o: {velho:.3e}");
}

// ─────────────────────────────────────────────────────────────────────────────
// ⛔⛔⛔ A SONDA DO REPORT DE 2026-09-05 («zero física! muitos artefatos», foto)
//
// A foto tem TRÊS coisas distintas, e uma régua que medisse só «moveu?» via as
// três como sucesso: **arcos escuros** (rachaduras), **bicos** nas pontas do
// traço, e **um espinho** disparado para fora da peça. Esta sonda mede cada uma.
// ─────────────────────────────────────────────────────────────────────────────

/// Uma esfera, que é o que o dono tem na cena `=1`.
fn esfera() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(48, 64, 1.0)
}

/// O traço do report: a mão atravessando a peça, evento a evento.
fn traco_longo_em(mesh: &mut Mesh, b: &Brush, passos: usize, sub: Option<u32>) -> SculptStroke {
    let mut s = SculptStroke {
        cloth_substeps_override: sub,
        ..SculptStroke::default()
    };
    s.begin(mesh);
    let d = 0.03;
    for k in 0..passos {
        let x = -0.5 + d * k as f32;
        // O dedo desce na superfície da esfera, olhando de +z.
        let z = (1.0 - x * x).max(0.0).sqrt();
        let c = [x, 0.0, z];
        let passo = if k == 0 { [0.0; 3] } else { [d, 0.0, 0.0] };
        s.dab(
            mesh,
            b,
            &Dab::hooking(c, b.radius, [0.0, 0.0, -1.0], passo),
            Symmetry::default(),
        );
    }
    s
}

/// As três grandezas da foto, medidas sobre um traço longo.
fn artefatos(sub: Option<u32>) -> (f32, f32, f32) {
    let antes = esfera();
    let mut mesh = esfera();
    let b = pincel();
    traco_longo_em(&mut mesh, &b, 35, sub);

    // (1) O ESPINHO: o maior deslocamento de um vértice.
    let mut pior = 0.0f32;
    // (2) A RACHADURA: a maior DIFERENÇA de deslocamento entre vizinhos de aresta.
    let mut rasgo = 0.0f32;
    let adj = antes.adjacency();
    let d: Vec<f32> = (0..antes.vert_count())
        .map(|v| desloc(&antes, &mesh, v))
        .collect();
    for v in 0..antes.vert_count() {
        pior = pior.max(d[v]);
        for n in adj.vert_verts.neighbours(v) {
            rasgo = rasgo.max((d[v] - d[*n as usize]).abs());
        }
    }
    // (3) A ARESTA ESTICADA: quanto a maior aresta cresceu.
    let mut estica = 1.0f32;
    for v in 0..antes.vert_count() {
        for n in adj.vert_verts.neighbours(v) {
            let l0 = {
                let (p, q) = (antes.positions()[v], antes.positions()[*n as usize]);
                ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
            };
            let l1 = {
                let (p, q) = (mesh.positions()[v], mesh.positions()[*n as usize]);
                ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
            };
            if l0 > 1e-9 {
                estica = estica.max(l1 / l0);
            }
        }
    }
    (pior, rasgo, estica)
}

/// ⛔ **A SONDA** — imprime as três grandezas da foto.
#[test]
#[ignore = "sonda: imprime, nao julga"]
fn sonda_dos_artefatos() {
    let (pior, rasgo, estica) = artefatos(None);
    eprintln!(
        "  espinho (maior deslocamento) : {pior:.4}\n  \
           rasgo   (salto entre vizinhos): {rasgo:.4}\n  \
           estica  (aresta / repouso)    : {estica:.2}x"
    );
}

/// ⭐⭐⭐ **GATE — um traço longo não deixa os artefatos da foto de 2026-09-05.**
///
/// ⛔⛔⛔ **DUAS das três barras foram RETIRADAS em 2026-09-06, porque elas
/// reprovavam a saída do próprio alvo.** Elas tinham sido calibradas sobre a lei
/// VBD — a que o dono reprovou três vezes com foto — e essa lei era TÍMIDA: o
/// que elas liam como saúde era ela mal deformar o pano. Medido na saída do
/// oráculo, sobre os 53 traços do clean-room:
///
/// | grandeza | o defeito (05/09) | a lei VBD | **o ALVO** | a nossa lei | veredito |
/// |---|---|---|---|---|---|
/// | espinho (maior deslocamento) | `0,690` | `0,052` | **`0,900`** | `0,767` | ⛔ não discrimina |
/// | rasgo (salto entre vizinhos) | `0,387` | `0,018` | **`0,219`** (arrasto) | `0,122` | ✅ discrimina |
/// | estica (aresta / repouso) | `2,98×` | `1,14×` | **`3,72×`** (arrasto) | `1,17×` | ⛔ não discrimina |
///
/// ⚠️ *O alvo deforma MAIS do que o defeito deformava em duas das três colunas.*
/// Uma barra que o reprovasse proibiria o comportamento correcto — e é
/// exactamente a armadilha que esta casa já registou: **uma barra calibrada sem
/// o lado aprovado mede os nossos próprios defeitos.**
///
/// ⭐ **O que SOBRA é o rasgo, e ele é o discriminador certo por mecanismo:** uma
/// agulha é um vértice que anda longe enquanto os vizinhos dele ficam, que é
/// literalmente um salto de deslocamento entre vizinhos de aresta. A barra
/// `0,30` fica na banda medida entre o pior arrasto do alvo (`0,219`) e o
/// defeito reproduzido (`0,387`).
///
/// ⚠️ **E o CONTROLE é metade do gate:** sem o piso, um pincel que não fizesse
/// nada passaria — que é literalmente o estado do report ANTERIOR (*«nada
/// aconteceu ao pintar»*). *Os dois reports do mesmo dia são os dois lados
/// desta assertiva.*
#[test]
fn um_traco_longo_nao_deixa_artefatos() {
    let (espinho, rasgo, estica) = artefatos(None);
    assert!(espinho > 0.01, "o pincel nao fez NADA: {espinho:.4}");
    assert!(
        rasgo < 0.30,
        "rasgo: {rasgo:.4} -- o defeito de 05/09 leu 0,387 e o pior arrasto do \
         alvo le 0,219 (espinho {espinho:.4}, estica {estica:.2}x)"
    );
}

/// ⭐⭐⭐ **GATE — a lei do gesto NÃO depende do orçamento do solver.**
///
/// ⛔⛔ **É a propriedade que eu quebrei sem ver.** A 3.ª versão do drive
/// dividia a aceleração pelo número de sub-passos, e o resultado é que **mais
/// orçamento deixava o pano PIOR**: `4 → 8 → 16` sub-passos levavam o esticão de
/// `2,3×` a `5,7×` e a `10,7×`. *Um solver que piora com mais orçamento não está
/// a convergir — há um termo que depende do orçamento*, e ali era o MOMENTO: uma
/// aceleração aplicada durante o evento inteiro injeta `a·dt` de velocidade, e a
/// derivação só tinha somado os `h²·a` de posição.
///
/// ⇒ com a cinemática completa (`Δx = ½·a·dt²`), as quatro corridas dão o MESMO
/// resultado, e é isso que este gate prende.
/// ⚠️⚠️ **A BARRA ESCOLHIDA MORREU EM 05/09, E A CURA CERTA É QUE A MATOU.** As
/// duas primeiras redações comparavam tudo contra `subs = 4` e exigiam `< 5 %`.
/// Quando a Hessiana da membrana passou a ser projetada
/// ([`ph2d_cloth::membrane`]), o solver começou de facto a **convergir** — e a
/// deriva de convergência subiu para `5,47 %`, reprovando o gate **sobre a
/// cura**. ⛔ Subir a barra para `6 %` seria o defeito que a memória desta casa
/// já registou (*«um tecto em graus SOBE quando a cura correta piora o número»*).
///
/// ⭐⭐⭐ **A troca é por uma PROPRIEDADE ANALÍTICA, que não tem barra para subir:**
/// *deriva de convergência ENCOLHE quando se dobra o orçamento; um termo que
/// depende do orçamento NÃO encolhe.* Com a resposta a convergir, as diferenças
/// entre orçamentos sucessivos caem geometricamente; com o defeito de 05/09 (a
/// aceleração dividida pelos sub-passos) cada duplicação **dobrava** a resposta,
/// logo a diferença relativa entre consecutivos ficava **constante em ~100 %** —
/// e nenhuma tolerância precisa de ser escolhida para as separar.
#[test]
fn o_gesto_nao_depende_do_orcamento_do_solver() {
    const ORCAMENTOS: [u32; 5] = [4, 8, 16, 32, 64];
    let lidos: Vec<(f32, f32, f32)> = ORCAMENTOS.iter().map(|s| artefatos(Some(*s))).collect();

    // Controlo anti-vácuo comum às duas leis: o traço tem de ter deformado.
    assert!(
        lidos[0].0 > 0.01,
        "o pincel nao fez NADA: {:.4} -- o gate ficaria verde por vacuo",
        lidos[0].0
    );

    // A deriva relativa entre dois orçamentos CONSECUTIVOS (que dobram).
    let deriva: Vec<f32> = lidos
        .windows(2)
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            [(a.0, b.0), (a.1, b.1), (a.2, b.2)]
                .iter()
                .fold(0.0f32, |m, (x, y)| m.max((y - x).abs() / x.abs().max(1e-6)))
        })
        .collect();

    // ⭐⭐ **A lei da REFERÊNCIA satisfaz a propriedade na forma mais forte que
    // existe: o orçamento não é um botão dela.** Ela relaxa um número fixo de
    // vezes por passo, então as cinco leituras são IDÊNTICAS e toda deriva é
    // exactamente zero. ⛔ *Isto não é o vácuo que a 2.ª redacção deste gate
    // deixou passar* — o piso acima já provou que o traço deformou; o que se
    // afirma aqui é que a deformação não mudou com o orçamento.
    if deriva.iter().all(|d| *d == 0.0) {
        return;
    }

    // ⭐ E a lei VBD (alcançável por `PH2D_CLOTH_LAW=vbd`) satisfaz a mesma
    // propriedade na forma fraca: a deriva de convergência ENCOLHE quando se
    // dobra o orçamento. Um termo que dependa do orçamento — o defeito de
    // 05/09, a aceleração dividida pelos sub-passos — mantém a deriva constante
    // em ~100 % e reprova, sem tolerância nenhuma escolhida.
    let (primeira, ultima) = (deriva[0], deriva[deriva.len() - 1]);
    assert!(
        ultima <= primeira / 2.0,
        "a deriva NAO encolhe ao dobrar o orcamento: {primeira:.4} -> {ultima:.4} \
         (derivas {deriva:?}) -- um termo proporcional ao orcamento le' ~100 % constante"
    );
}

/// ⭐⭐⭐ **GATE — a resposta do pano NÃO depende de quão fina é a malha.**
///
/// ⛔⛔⛔ **Ela quebrou DUAS vezes, e a segunda foi o report *«não acontece
/// nada»*.** O dono esculpe numa peça de `50 000` faces; as minhas fixturas
/// tinham `3 000` vértices, e o pincel que respondia `17 %` do raio ali
/// respondia **`4 %`** na peça dele — invisível.
///
/// | âncora da mão | 362 v | 3 010 v | 12 162 v |
/// |---|---|---|---|
/// | aceleração (`F = m·a`) | `151 %` | `17 %` | **`4 %`** ⛔ |
/// | rigidez da INÉRCIA | `52 %` | `16 %` | **`4 %`** ⛔ |
/// | **rigidez ABSOLUTA** | `15 %` | `24 %` | **`30 %`** ✅ |
///
/// ⚠️ *Um material de contínuo não pode depender de como se o malha* — e a causa
/// das duas quebras é a mesma: tudo o que escala com a MASSA de um vértice
/// desaparece ao refinar, enquanto a Hessiana elástica por vértice é `O(μ)` e
/// não muda. A mão tem de viver na escala do elástico, sem ser feita dele.
///
/// ⚠️ **A barra é uma RAZÃO entre os extremos, e não uma tolerância apertada:**
/// a resposta ainda cresce `~40 %` de `1 490` a `12 162` vértices, o que é
/// deriva de discretização honesta — contra os **`13×` de COLAPSO** que as duas
/// versões anteriores tinham.
#[test]
fn a_resposta_nao_depende_da_densidade_da_malha() {
    let mut lidos = Vec::new();
    for (r, s) in [(32usize, 48usize), (48, 64), (72, 96), (96, 128)] {
        let antes = ph2d_mesh::shapes::uv_sphere(r, s, 1.0);
        let mut mesh = antes.clone();
        let b = pincel();
        traco_longo_em(&mut mesh, &b, 35, None);
        let pior = (0..antes.vert_count())
            .map(|v| desloc(&antes, &mesh, v))
            .fold(0.0f32, f32::max);
        lidos.push(pior / b.radius);
    }
    let (lo, hi) = lidos
        .iter()
        .fold((f32::MAX, 0.0f32), |(a, b), v| (a.min(*v), b.max(*v)));
    assert!(lo > 0.05, "o pincel some na malha fina: {lo:.3} do raio");
    assert!(
        hi / lo < 2.0,
        "a resposta varia {:.1}x com a densidade ({lidos:?}) -- as versoes \
         anteriores colapsavam 13x",
        hi / lo
    );
}

/// ⛔ **A SONDA DA RESOLUÇÃO** — a resposta do pano contra a densidade da malha.
#[test]
#[ignore = "sonda: imprime, nao julga"]
fn sonda_da_resolucao() {
    eprintln!("  verts     aresta    espinho   % do raio");
    for (r, s) in [(16usize, 24usize), (32, 48), (48, 64), (72, 96), (96, 128)] {
        let antes = ph2d_mesh::shapes::uv_sphere(r, s, 1.0);
        let mut mesh = antes.clone();
        let b = pincel();
        traco_longo_em(&mut mesh, &b, 35, None);
        let pior = (0..antes.vert_count())
            .map(|v| desloc(&antes, &mesh, v))
            .fold(0.0f32, f32::max);
        let adj = antes.adjacency();
        let aresta = {
            let p = antes.positions();
            let n = adj.vert_verts.neighbours(antes.vert_count() / 2)[0] as usize;
            let (a, c) = (p[antes.vert_count() / 2], p[n]);
            ((a[0] - c[0]).powi(2) + (a[1] - c[1]).powi(2) + (a[2] - c[2]).powi(2)).sqrt()
        };
        eprintln!(
            "  {:<9} {aresta:<9.4} {pior:<9.4} {:.0}%",
            antes.vert_count(),
            100.0 * pior / b.radius
        );
    }
}

/// Uma grade plana de `n × n` células sobre `[-1, 1]` — a mesma forma do
/// [`plano`], com a densidade como parâmetro.
fn plano_n(n: usize) -> Mesh {
    let s = 2.0 / n as f32;
    let mut pos = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            pos.push([i as f32 * s - 1.0, j as f32 * s - 1.0, 0.0]);
        }
    }
    let id = |i: usize, j: usize| u32::try_from(j * (n + 1) + i).unwrap_or(u32::MAX);
    let mut faces = Vec::new();
    for j in 0..n {
        for i in 0..n {
            faces.push(ph2d_mesh::Face::quad(
                id(i, j),
                id(i + 1, j),
                id(i + 1, j + 1),
                id(i, j + 1),
            ));
        }
    }
    Mesh::from_parts(pos, faces).expect("grade plana")
}

/// **A RÉGUA LOCAL** — o pior resíduo de um vértice contra a média dos quatro
/// vizinhos da grade, em unidades do **chão da discretização**.
///
/// ⚠️ Num perfil liso esse resíduo vale `~(h²/4)·∇²f`, logo ele cai com `(h/R)²`:
/// uma barra fixa leria `0,093` numa grade de 24 e `0,002` numa de 144 **sobre o
/// mesmo produto**. Dividir pelo piso torna a grandeza comparável entre
/// densidades — e é o que separa *perfil inclinado* de *vértice que voou*.
fn residuo_local(antes: &Mesh, depois: &Mesh, n: usize, raio: f32) -> f32 {
    let id = |i: usize, j: usize| j * (n + 1) + i;
    let max = (0..antes.vert_count())
        .map(|v| desloc(antes, depois, v))
        .fold(0.0f32, f32::max);
    let piso = ((2.0 / n as f32) / raio).powi(2);
    let mut pior = 0.0f32;
    for j in 1..n {
        for i in 1..n {
            let d = desloc(antes, depois, id(i, j));
            let m = [id(i - 1, j), id(i + 1, j), id(i, j - 1), id(i, j + 1)]
                .iter()
                .map(|v| desloc(antes, depois, *v))
                .sum::<f32>()
                / 4.0;
            pior = pior.max((d - m).abs() / max.max(1e-9) / piso);
        }
    }
    pior
}

/// **SONDA — o orçamento do solver contra o passo da mão, com o PERCURSO fixo.**
///
/// ⚠️ Esta é a célula que a sonda de integração (`probe_cloth_front`) **não
/// consegue correr**: variar os sub-passos só é alcançável de dentro da crate, e
/// sem ela «passo menor» e «mais varreduras» ficam confundidos — as duas mudam
/// juntas quando se entrega o mesmo percurso em mais eventos.
#[test]
#[ignore = "sonda"]
fn sonda_do_orcamento_contra_o_passo() {
    println!(
        "{:>5} {:>7} {:>6} {:>9} {:>9}",
        "n", "passo/h", "subs", "residuo", "max/R"
    );
    for n in [96usize, 144, 192] {
        for sub in [4u32, 8, 16, 32, 64] {
            let antes = plano_n(n);
            let mut mesh = plano_n(n);
            let b = pincel();
            let mut s = SculptStroke {
                cloth_substeps_override: Some(sub),
                ..SculptStroke::default()
            };
            s.begin(&mesh);
            for k in 0..35 {
                let c = [0.02 * k as f32, 0.0, 0.0];
                let passo = if k == 0 { [0.0; 3] } else { [0.02, 0.0, 0.0] };
                s.dab(
                    &mut mesh,
                    &b,
                    &dab_em(c, b.radius, passo),
                    Symmetry::default(),
                );
            }
            let max = (0..antes.vert_count())
                .map(|v| desloc(&antes, &mesh, v))
                .fold(0.0f32, f32::max);
            println!(
                "{:>5} {:>7.2} {:>6} {:>9.1} {:>9.3}",
                n,
                0.02 / (2.0 / n as f32),
                sub,
                residuo_local(&antes, &mesh, n, b.radius),
                max / b.radius
            );
        }
    }
}

/// ⭐⭐⭐ **GATE — um traço numa malha da DENSIDADE DO DONO não deixa agulha.**
///
/// ⛔⛔⛔ **Este gate existe porque uma mutação ATRAVESSOU a suíte inteira.** Em
/// 2026-09-05 a projeção PSD da Hessiana da membrana
/// ([`ph2d_cloth::membrane`]) curou o report da agulha — e desfazê-la deixava os
/// **23 gates do solver e os 10 do pincel VERDES**. *Eu escrevi a guarda certa e
/// não a gateei*, que é uma família já registada na memória desta casa.
///
/// A razão de nenhum deles a ver é a **fixtura**: a [`esfera`] tem `3 010`
/// vértices e a [`plano`] tem `625`, e o defeito **não existe nessa densidade**.
/// O dono esculpe a `~25 000`.
///
/// # ⚠️ A régua é LOCAL, e a normalização é o chão da discretização
///
/// As três colunas de [`artefatos`] são extremos **globais**: `espinho = max‖u‖`
/// não distingue *«o pano todo andou `0,10`»* de *«um vértice voou `0,10` e os
/// vizinhos `0,001`»* — e uma agulha **aumenta** o mesmo número que serve de piso
/// anti-vácuo daquele gate. A régua que separa é o resíduo de um vértice contra a
/// **mediana da própria vizinhança**, e ela tem de ser dividida por `(h/R)²`
/// senão acusa a malha grossa (ver [`residuo_local`]).
///
/// # ⚠️ A barra saiu do VAZIO entre os dois lados, e os dois foram medidos
///
/// | regime | resíduo local |
/// |---|---|
/// | são (`96`–`192` células, com a projeção) | **`1,2` – `2,1`** |
/// | partido (o mesmo, sem a projeção) | **`118` – `483`** |
///
/// `20` fica no meio do vazio, com `~10×` de margem para cada lado. ⛔ Não é um
/// número escolhido a dedo: é a única banda em que nenhuma das duas populações
/// medidas cai.
///
/// # ⚠️ E o passo da mão é o do PRODUTO
///
/// O gatilho medido é o passo entre eventos em **arestas de malha**: acima de
/// `~1` aresta o elemento entra em compressão profunda, que é onde o StVK é
/// não-convexo. A `144` células a aresta é `0,0139` e o passo é `0,02` =
/// **`1,44` arestas** — e o `walk` do produto emite a `0,15 · raio`
/// ([`crate::MIN_SPACING_FRACTION`]), que numa peça de 25 mil vértices dá
/// **`1,3` arestas**. *A fixtura corre o regime que o artista corre.*
#[test]
fn um_traco_na_densidade_do_dono_nao_deixa_agulha() {
    const N: usize = 144;
    let antes = plano_n(N);
    let mut mesh = plano_n(N);
    let b = pincel();
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..35 {
        let c = [0.02 * k as f32, 0.0, 0.0];
        let passo = if k == 0 { [0.0; 3] } else { [0.02, 0.0, 0.0] };
        s.dab(
            &mut mesh,
            &b,
            &dab_em(c, b.radius, passo),
            Symmetry::default(),
        );
    }

    // ⚠️ **CONTROLE ANTI-VÁCUO:** sem ele, um traço que não movesse nada daria
    // resíduo `0` e o gate leria aprovado sobre um pincel morto.
    let max = (0..antes.vert_count())
        .map(|v| desloc(&antes, &mesh, v))
        .fold(0.0f32, f32::max);
    assert!(
        max > 0.05 * b.radius,
        "o pincel nao moveu nada na malha fina ({max:.4}) -- este gate estaria \
         verde por vacuo"
    );

    let residuo = residuo_local(&antes, &mesh, N, b.radius);
    assert!(
        residuo < 20.0,
        "AGULHA: um vertice esta' a {residuo:.1} unidades do chao da discretizacao \
         fora da propria vizinhanca (barra 20; sao 1,2-2,1; o report de 05/09 dava \
         118-483). Sem a projecao PSD da Hessiana da membrana o passo de Newton e' \
         um POLO em compressao"
    );
}
