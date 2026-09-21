//! ⭐⭐⭐⭐ **O PLANO DE TINTA FINA SOBE À PLACA, pela rota do PRODUTO.**
//!
//! # ⛔⛔ Porque este ficheiro existe
//!
//! A bancada de paridade ([`super::tinta_paridade`]) prova que o gémeo em WGSL
//! calcula a mesma cor que a CPU — e fá-lo com um **arnês de compute próprio**,
//! que nunca chama a [`ph2d_mesh_render::MeshRenderer::upload_tinta_at`]. A
//! suíte de desenho, do outro lado, nunca **arma** um plano. ⇒ *a rota que o
//! produto de facto toma não tinha uma única régua*, e foi ali que o defeito
//! abaixo viveu a wave inteira.
//!
//! # ⛔⛔⛔ O defeito que este gate reproduz
//!
//! A capacidade do buffer de ÍNDICES era **derivada** da do buffer de origens
//! (`cap_tri * 3`) em vez de ser um campo. Na PRIMEIRA subida o `origem`
//! realoca e actualiza o `cap_tri`, logo `cap_tri * 3` já descreve o tamanho
//! NOVO enquanto o `idx` ainda é o buffer-dummy de `16` bytes — a escrita toma
//! o caminho rápido e despeja milhares de bytes lá dentro.
//!
//! ⚠️ **A sequência é o gate:** uma subida só não chega, porque o defeito nasce
//! da realocação. É preciso **subir, e subir outra vez maior** — que é
//! exactamente o que o artista faz quando o passe de topologia adensa a peça
//! debaixo do pincel.

use ph2d_mesh::shapes;
use ph2d_mesh_colors::Tinta;
use ph2d_mesh_render::MeshRenderer;

use super::device_de_teste::device;

/// ⭐⭐⭐ **GATE — subir o plano duas vezes, a segunda MAIOR, não estoura.**
///
/// ⚠️ **O veredito é o do `wgpu`, e é preciso ir buscá-lo:** uma escrita fora do
/// buffer é um erro de VALIDAÇÃO, e o `wgpu` entrega-o por um *callback* de erro
/// em vez de um `Result` — sem o `push_error_scope` o teste passaria com o
/// device a acumular erros em silêncio, que é o modo de falha que este ficheiro
/// existe para não ter.
///
/// ⚠️ **No `wgpu` 29 o escopo é um GUARDA** (`ErrorScopeGuard`) e o veredito sai
/// do `.pop()` DELE — não há `Device::pop_error_scope`. Largar o guarda sem o
/// consultar fecha o escopo em silêncio.
#[test]
#[ignore = "precisa de adaptador"]
fn o_plano_sobe_a_placa_e_cresce_sem_estourar() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adaptador nesta máquina — nada a afirmar");
        return;
    };
    let mut r = MeshRenderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    // A peça PEQUENA primeiro, e a GRANDE depois — a ordem é o gate.
    let pequena = shapes::uv_sphere(8, 12, 1.0);
    let grande = shapes::uv_sphere(24, 32, 1.0);
    assert!(
        grande.vert_count() > pequena.vert_count() * 4,
        "a fixtura tem de CRESCER de facto: {} contra {}",
        grande.vert_count(),
        pequena.vert_count()
    );

    for (passo, m) in [(1u32, &pequena), (2, &grande)] {
        let escopo = device.push_error_scope(wgpu::ErrorFilter::Validation);
        r.upload_at(&device, &queue, 0, m, &[]);
        let t = Tinta::nova(
            m.vert_count(),
            m.faces().iter().map(ph2d_mesh::Face::verts),
            2,
        );
        r.upload_tinta_at(&device, &queue, 0, m, Some(&t));
        device.poll(wgpu::PollType::wait_indefinitely()).ok();
        let erro = pollster::block_on(escopo.pop());
        assert!(
            erro.is_none(),
            "a subida {passo} ({} vértices, {} amostras) deu erro de validação: {erro:?}",
            m.vert_count(),
            t.amostras().len()
        );
    }
}

/// ⚠️ **GATE — desarmar não apaga os buffers, e também não estoura.**
///
/// ⛔ O caminho `None` escreve `armado = 0` e **não** liberta os recursos: o
/// layout é do PIPELINE, e um binding que aparece e desaparece obrigaria a
/// reconstruí-lo. Este gate é o que impede alguém de «optimizar» isso e
/// descobrir pelo primeiro quadro branco.
#[test]
#[ignore = "precisa de adaptador"]
fn desarmar_o_plano_nao_estoura_e_o_slot_continua_desenhavel() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adaptador nesta máquina — nada a afirmar");
        return;
    };
    let mut r = MeshRenderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);
    let m = shapes::uv_sphere(12, 16, 1.0);

    let escopo = device.push_error_scope(wgpu::ErrorFilter::Validation);
    r.upload_at(&device, &queue, 0, &m, &[]);
    let t = Tinta::nova(
        m.vert_count(),
        m.faces().iter().map(ph2d_mesh::Face::verts),
        1,
    );
    r.upload_tinta_at(&device, &queue, 0, &m, Some(&t));
    // E agora DESARMA — duas vezes, porque a segunda tem de ser um no-op.
    r.upload_tinta_at(&device, &queue, 0, &m, None);
    r.upload_tinta_at(&device, &queue, 0, &m, None);
    // E volta a armar, que é o que o artista faz ao mexer no chip.
    r.upload_tinta_at(&device, &queue, 0, &m, Some(&t));
    device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let erro = pollster::block_on(escopo.pop());
    assert!(
        erro.is_none(),
        "armar/desarmar deu erro de validação: {erro:?}"
    );
}

/// ⭐⭐⭐⭐ **GATE — O PÂNICO DO DONO, pela rota do PRODUTO: um plano da malha
/// de ANTES DESARMA, e não estoura.**
///
/// ⛔⛔ **Report de 2026-09-21, quadro `10216`:**
/// `index out of bounds: the len is 196608 but the index is 196608` em
/// `ph2d-mesh-colors/src/topo.rs:239`. O número é `4 × 49 152` — a conta
/// `4 * f + s` com `f` a valer a contagem de faces do PLANO, ou seja a lista de
/// faces que chegou ao registo tinha mais faces do que ele.
///
/// **O caminho:** o pen-down de um gesto de FORMA com o plano armado empresta o
/// plano ao traço e **logo a seguir** tritura a malha
/// (`history_dyntopo::open_dyntopo_stroke`, os dois motores de topologia
/// recusam quads) — e a rota do plano emprestado é a única que não reconcilia.
///
/// ⚠️ **O cabeçalho da porta já escrevia o perigo** (*«um plano da malha de
/// antes é tinta no vértice errado, e nenhuma contagem o vê»*) e a única régua
/// era um `debug_assert`, que no perfil `smoke` **não existe**.
///
/// ⭐ **O discriminador é a CONTAGEM DE AMOSTRAS IGUAL dos dois lados.** A
/// `upload_tinta_amostras_at` recusa por duas razões — o slot não estar armado
/// *ou* o device ter outro número de amostras —, e ao nível `0` um plano vale
/// `verts` amostras **triangulado ou não**. Com as contagens presas, o `false`
/// dela só pode vir do `armado`, que é a propriedade que este gate afirma.
#[test]
#[ignore = "precisa de adaptador"]
fn um_plano_da_malha_de_antes_desarma_em_vez_de_estourar() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adaptador nesta máquina — nada a afirmar");
        return;
    };
    let mut r = MeshRenderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb);

    // A peça de QUADS, e o plano que nasce dela.
    let quads = shapes::cube(1.0);
    let plano = Tinta::nova(
        quads.vert_count(),
        quads.faces().iter().map(ph2d_mesh::Face::verts),
        0,
    );

    // A MESMA peça depois do que o pen-down faz.
    let mut tris = quads.clone();
    let partidas = tris.triangulate();
    assert!(
        partidas > 0,
        "a fixtura tem de CONTER o fenómeno: a peça já era de triângulos"
    );
    assert_eq!(
        tris.vert_count(),
        quads.vert_count(),
        "triangular não cria vértices — é isso que prende a contagem de amostras"
    );
    assert!(
        tris.faces().len() > quads.faces().len(),
        "o gate precisa de MAIS faces: {} contra {}",
        tris.faces().len(),
        quads.faces().len()
    );

    let escopo = device.push_error_scope(wgpu::ErrorFilter::Validation);
    // (1) o CONTROLO: a malha de que o plano nasceu ARMA.
    r.upload_at(&device, &queue, 0, &quads, &[]);
    r.upload_tinta_at(&device, &queue, 0, &quads, Some(&plano));
    let mut sujas = vec![0u32];
    assert!(
        r.upload_tinta_amostras_at(&queue, 0, &plano, &mut sujas),
        "o CONTROLO falhou: o plano da própria malha tinha de deixar o slot armado"
    );

    // (2) e a malha de DEPOIS, com o plano de ANTES: desarma.
    r.upload_at(&device, &queue, 0, &tris, &[]);
    r.upload_tinta_at(&device, &queue, 0, &tris, Some(&plano));
    let mut sujas = vec![0u32];
    assert!(
        !r.upload_tinta_amostras_at(&queue, 0, &plano, &mut sujas),
        "o plano descreve {} faces e a malha tem {} — o slot tinha de ficar DESARMADO",
        plano.topologia().faces(),
        tris.faces().len()
    );

    // (3) ⭐⭐⭐⭐ **E a metade que o PAYLOAD não pode ver: os VÉRTICES.**
    // Um registo é feito de FACES, logo um `payload` que aceite a lista inteira
    // não afirma nada sobre a contagem de vértices — e a `pos` e as `amostras`
    // que sobem ao lado dele são indexadas por vértice. A malha aqui tem as
    // MESMAS faces e um vértice ÓRFÃO a mais, que é a única maneira de pôr as
    // duas réguas a discordar.
    let mut posicoes = quads.positions().to_vec();
    posicoes.push([9.0, 9.0, 9.0]);
    let com_orfao = ph2d_mesh::Mesh::from_parts(posicoes, quads.faces().to_vec())
        .expect("um vértice a mais não torna as faces inválidas");
    assert_eq!(
        com_orfao.faces().len(),
        quads.faces().len(),
        "o CONTROLO: só os VÉRTICES podem diferir, senão o payload já decidia"
    );
    r.upload_at(&device, &queue, 0, &com_orfao, &[]);
    r.upload_tinta_at(&device, &queue, 0, &com_orfao, Some(&plano));
    let mut sujas = vec![0u32];
    assert!(
        !r.upload_tinta_amostras_at(&queue, 0, &plano, &mut sujas),
        "o plano descreve {} vértices e a malha tem {} — o slot tinha de ficar DESARMADO",
        plano.topologia().verts(),
        com_orfao.vert_count()
    );

    device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let erro = pollster::block_on(escopo.pop());
    assert!(erro.is_none(), "a subida deu erro de validação: {erro:?}");
}
