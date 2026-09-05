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
    let mut miolo = 0.0f32;
    let mut longe = 0.0f32;
    for v in 0..antes.vert_count() {
        let p = antes.positions()[v];
        let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
        let d = desloc(&antes, &depois, v);
        if r < 0.15 {
            miolo = miolo.max(d);
        } else if r > 0.80 {
            // Bem fora da região de simulação (`0,30 × 2 = 0,60`).
            longe = longe.max(d);
        }
    }
    assert!(miolo > 1e-3, "o pano nao respondeu ao arrasto: {miolo:.3e}");
    assert_eq!(longe, 0.0, "o pano moveu fora da regiao: {longe:.3e}");
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
/// ⛔⛔ **As barras saem do defeito REPRODUZIDO, não de um número escolhido.** O
/// report (*«zero física! muitos artefatos»*) foi reproduzido nesta mesma
/// fixtura, e cada coluna tem os dois lados:
///
/// | grandeza | o defeito | hoje | barra |
/// |---|---|---|---|
/// | espinho (maior deslocamento) | `0,690` | `0,052` | `0,20` |
/// | rasgo (salto entre vizinhos) | `0,387` | `0,018` | `0,10` |
/// | estica (aresta / repouso) | `2,98×` | `1,14×` | `1,50×` |
///
/// ⚠️ **E o CONTROLE é metade do gate:** sem o piso, um pincel que não fizesse
/// nada passaria em todas as três — que é literalmente o estado em que ele
/// esteve no report ANTERIOR (*«nada aconteceu ao pintar»*). *Os dois reports do
/// mesmo dia são os dois lados desta assertiva.*
#[test]
fn um_traco_longo_nao_deixa_artefatos() {
    let (espinho, rasgo, estica) = artefatos(None);
    assert!(espinho > 0.01, "o pincel nao fez NADA: {espinho:.4}");
    assert!(espinho < 0.20, "espinho: {espinho:.4} (defeito: 0,690)");
    assert!(rasgo < 0.10, "rasgo: {rasgo:.4} (defeito: 0,387)");
    assert!(estica < 1.50, "estica: {estica:.2}x (defeito: 2,98x)");
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
#[test]
fn o_gesto_nao_depende_do_orcamento_do_solver() {
    // ⚠️ **A barra é RELATIVA, e não `1e-6`:** dobrar os sub-passos muda a
    // CONVERGÊNCIA do solver, e isso é legítimo — o que não pode mudar é a
    // MAGNITUDE do que o gesto propõe. A 1.ª redação exigia igualdade ao bit e
    // reprovava sobre ruído numérico.
    let base = artefatos(Some(4));
    let mut pior = 0.0f32;
    for sub in [2u32, 8, 16] {
        let (e, r, st) = artefatos(Some(sub));
        for (a, b) in [(e, base.0), (r, base.1), (st, base.2)] {
            pior = pior.max((a - b).abs() / b.abs().max(1e-6));
        }
    }
    assert!(
        pior < 0.05,
        "o orcamento mudou a resposta em {:.2} % -- ha' um termo que depende dele \
         (o defeito de 05/09 mudava 430 %)",
        pior * 100.0
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
