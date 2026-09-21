//! Os gates da porta que é dona do plano de tinta fina.

use super::*;
use ph2d_mesh::Face;

/// Um par de triângulos, quatro posições.
fn dois_tris() -> Mesh {
    Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![Face::tri(0, 1, 2), Face::tri(0, 2, 3)],
    )
    .expect("a fixtura é uma malha válida")
}

/// ⭐⭐ **O plano NOVO nasce da cor que a peça JÁ TEM.**
///
/// O defeito que isto impede tem nome no doc da [`garante`] e é o mais caro que
/// esta porta pode ter: o artista pinta, pede mais resolução, e a tinta some.
///
/// ⚠️ **O CONTROLO é a metade que o torna uma medição:** uma peça que ninguém
/// pintou continua branca, senão o gate passaria com um `garante` que copiasse
/// qualquer coisa para todo lado.
#[test]
fn o_plano_novo_nasce_com_a_cor_que_a_peca_ja_tinha() {
    let mut pintada = dois_tris();
    for c in pintada.colors_mut() {
        *c = [0.2, 0.7, 0.1];
    }
    let mut t = None;
    assert!(garante(&pintada, &mut t, Some(2)), "o plano tem de nascer");
    let plano = t.expect("nasceu");
    for (i, a) in plano.amostras().iter().enumerate() {
        for k in 0..3 {
            assert!(
                (a[k] - [0.2, 0.7, 0.1][k]).abs() <= 1e-6,
                "a amostra {i} nasceu {a:?} e não a cor da peça"
            );
        }
    }

    let crua = dois_tris();
    let mut t2 = None;
    assert!(garante(&crua, &mut t2, Some(2)));
    let branco = t2.expect("nasceu");
    assert!(
        branco.amostras().iter().all(|a| *a == [1.0, 1.0, 1.0]),
        "CONTROLO: uma peça por pintar nasce branca"
    );
}

/// ⚠️ **Reconciliar é BARATO quando nada mudou** — e `false` é o que impede o
/// upload de `O(V + F)` de correr em todo quadro parado.
#[test]
fn reconciliar_o_mesmo_nivel_nao_reconstroi_nada() {
    let m = dois_tris();
    let mut t = None;
    assert!(garante(&m, &mut t, Some(1)), "a primeira vez constrói");
    assert!(!garante(&m, &mut t, Some(1)), "a segunda não mexe em nada");
    assert!(garante(&m, &mut t, Some(2)), "outro nível reconstrói");
    assert_eq!(t.as_ref().map(ph2d_mesh_colors::Tinta::nivel), Some(2));
    assert!(garante(&m, &mut t, None), "desarmar larga o plano");
    assert!(t.is_none());
    assert!(
        !garante(&m, &mut t, None),
        "desarmar duas vezes não é mudança"
    );
}

/// ⛔⛔ **A concordância tem DUAS metades e cada uma apanha uma mudança de
/// topologia que a outra não vê** — o doc da [`concorda_com`] diz o mecanismo,
/// e este gate constrói as duas malhas que o separam.
#[test]
fn a_concordancia_ve_os_vertices_e_as_faces() {
    let m = dois_tris();
    let mut t = None;
    garante(&m, &mut t, Some(1));
    let plano = t.expect("nasceu");
    assert!(concorda_com(&plano, &m), "ela concorda consigo mesma");

    // Mesmas FACES, mais um vértice (órfão): só a 1.ª metade acusa.
    let mais_um = Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [2.0, 2.0, 0.0],
        ],
        vec![Face::tri(0, 1, 2), Face::tri(0, 2, 3)],
    )
    .expect("válida");
    assert_eq!(mais_um.faces().len(), m.faces().len());
    assert!(
        !concorda_com(&plano, &mais_um),
        "um vértice a mais é outra malha"
    );

    // Mesmos VÉRTICES, uma face a menos: só a 2.ª metade acusa.
    let menos_uma = Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        vec![Face::tri(0, 1, 2)],
    )
    .expect("válida");
    assert_eq!(menos_uma.vert_count(), m.vert_count());
    assert!(
        !concorda_com(&plano, &menos_uma),
        "uma face a menos é outra malha"
    );
}

/// ⭐⭐⭐ **Devolver o plano REESCREVE o canal por vértice** — a metade que
/// mantém UMA cor entre as duas resoluções.
///
/// ⚠️ **O CONTROLO é o que separa a lei do acaso:** a cor que o plano carrega
/// é diferente da que a malha tinha, logo o `assert` só passa se a cópia de
/// facto corre.
#[test]
fn devolver_o_plano_reescreve_a_cor_por_vertice() {
    let mut m = dois_tris();
    for c in m.colors_mut() {
        *c = [1.0, 0.0, 0.0];
    }
    let mut t = None;
    garante(&m, &mut t, Some(1));
    // Pinta o PLANO de verde, sem tocar na malha.
    for a in t.as_mut().expect("nasceu").amostras_mut() {
        *a = [0.0, 1.0, 0.0];
    }
    assert_eq!(
        m.colors().expect("pintada")[0],
        [1.0, 0.0, 0.0],
        "a malha ainda é vermelha"
    );

    let emprestado = empresta(&mut t);
    assert!(t.is_none(), "empréstimo é um TAKE: a peça fica sem plano");
    devolve(&mut m, &mut t, emprestado);
    assert!(t.is_some(), "e ele volta");
    for (v, c) in m.colors().expect("pintada").iter().enumerate() {
        assert_eq!(
            *c,
            [0.0, 1.0, 0.0],
            "o vértice {v} não recebeu a cor do plano"
        );
    }
}

/// ⚠️ **Devolver NADA não escreve nada** — o caminho de um traço que correu com
/// o plano desarmado, que é o caminho de omissão do produto.
#[test]
fn devolver_nada_deixa_a_peca_como_estava() {
    let mut m = dois_tris();
    for c in m.colors_mut() {
        *c = [0.3, 0.3, 0.3];
    }
    let mut t = None;
    devolve(&mut m, &mut t, None);
    assert!(t.is_none());
    assert!(
        m.colors()
            .expect("pintada")
            .iter()
            .all(|c| *c == [0.3, 0.3, 0.3])
    );
}

/// ⛔ **Um plano que já não descreve a malha NÃO reescreve a cor dela.**
///
/// Este é o ramo que impede a armadilha muda do cabeçalho do módulo de chegar
/// ao canal por vértice: se a topologia mudou a meio do traço, os `V` primeiros
/// samples já não são os `V` vértices de agora.
#[test]
fn um_plano_desactualizado_nao_escreve_no_canal_por_vertice() {
    let m = dois_tris();
    let mut t = None;
    garante(&m, &mut t, Some(1));
    for a in t.as_mut().expect("nasceu").amostras_mut() {
        *a = [0.0, 0.0, 1.0];
    }
    let emprestado = empresta(&mut t);

    let mut outra = Mesh::from_parts(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
        vec![Face::tri(0, 1, 2)],
    )
    .expect("válida");
    for c in outra.colors_mut() {
        *c = [0.9, 0.9, 0.9];
    }
    devolve(&mut outra, &mut t, emprestado);
    assert!(
        outra
            .colors()
            .expect("pintada")
            .iter()
            .all(|c| *c == [0.9, 0.9, 0.9]),
        "o plano de outra topologia não pode escrever aqui"
    );
    assert!(
        t.is_some(),
        "e ele volta à peça na mesma — quem o mata é a reconciliação"
    );
}

/// ⛔⛔ **GATE — o PREÇO do tecto é MEDIDO, e a 1.ª redacção do doc estava
/// errada por 2×.**
///
/// Ela dizia *«~3,1 M amostras»* para a peça de fábrica porque eu contei os
/// interiores de cada quad e **esqueci as arestas**, que levam `L−1` amostras
/// cada. *Um número escrito de cabeça ao lado de um tecto é o palpite que o
/// §0.0 proíbe* — e a cura não é corrigir o número, é o gate medi-lo.
///
/// ⚠️ **A fixtura é uma esfera de QUADS**, que é a forma da peça de fábrica do
/// módulo e a única em que o `L²` por vértice vale: numa malha de triângulos o
/// custo por vértice é outro (há `2` faces por vértice e o interior de um
/// triângulo tem `(L−1)(L−2)/2`), e medir ali daria um multiplicador que não
/// descreve o que o artista paga.
#[test]
fn o_custo_do_tecto_por_vertice_e_o_que_a_constante_diz() {
    // ⚠️ **Um TORO e não uma esfera UV**, e a 1.ª redacção usava a esfera e
    //    reprovou na primeira corrida: os PÓLOS dela são leques de TRIÂNGULOS,
    //    e este multiplicador só descreve a família dos quads. *A fixtura tem
    //    de conter o regime que a constante descreve.*
    let m = ph2d_mesh::shapes::torus(48, 24, 1.0, 0.35);
    assert!(
        m.faces().iter().all(|f| !f.is_tri()),
        "a fixtura tem de ser de QUADS: a constante só descreve essa família"
    );
    let mut t = None;
    garante(&m, &mut t, Some(NIVEL_MAX));
    let amostras = t.expect("nasceu").amostras().len();
    let por_vertice = amostras as f64 / m.vert_count() as f64;
    let alvo = CUSTO_POR_VERTICE_NO_TECTO as f64;
    // ⚠️ A banda é de `1 %`: num toro fechado de quads a contagem é EXACTA
    //    (`V` faces, `2V` arestas), e a folga cobre só o arredondamento da
    //    divisão — não é margem de conforto.
    assert!(
        (por_vertice - alvo).abs() / alvo < 0.01,
        "o tecto custa {por_vertice:.1} amostras por vértice e a constante diz          {alvo:.0} -- a aritmetica do doc deixou de descrever o que se aloca          ({amostras} amostras sobre {} vertices)",
        m.vert_count()
    );
}

/// ⚠️ **O tecto é um CLAMP e não um pânico** — um nível vindo de um documento
/// mais novo tem de aterrar no mais fino que este produto sabe alocar.
#[test]
fn o_nivel_acima_do_tecto_e_cortado_no_tecto() {
    let m = dois_tris();
    let mut t = None;
    garante(&m, &mut t, Some(NIVEL_MAX + 4));
    assert_eq!(
        t.as_ref().map(ph2d_mesh_colors::Tinta::nivel),
        Some(NIVEL_MAX)
    );
}

/// ⛔⛔ **GATE — o tecto que o PAINEL oferece é o tecto que o MOTOR aloca.**
///
/// ⚠️ **Ele atravessa duas crates de propósito**, e é o único sítio de onde se
/// pode fazer a pergunta: o painel declara os chips e a porta declara o
/// `NIVEL_MAX`, e as duas listas não se conhecem. Sem isto, um chip `16×` novo
/// nasceria a pedir um nível que a [`garante`] **corta em silêncio** — o
/// *«aceita e mente»* que esta casa já pagou no `lattice`, no `kaleidoscope` e
/// no `iterations` do colisor.
///
/// ⚠️ **As duas metades:** o mais fino que o painel oferece é EXACTAMENTE o
/// tecto (senão o produto oferece menos do que aloca, e o `NIVEL_MAX` deixa de
/// descrever o que quer que seja), e nenhum chip pede acima dele.
#[test]
fn o_tecto_do_painel_e_o_tecto_do_motor() {
    use ph2d_panel_sculpt3d::state::DetalheDaTinta;
    let mais_fino = DetalheDaTinta::ALL
        .into_iter()
        .filter_map(DetalheDaTinta::nivel)
        .max()
        .expect("o painel oferece pelo menos um plano");
    assert_eq!(
        mais_fino, NIVEL_MAX,
        "o painel oferece ate' `{mais_fino}` e a porta aloca ate' `{NIVEL_MAX}` -- oferecer \
         menos deixa o tecto a descrever nada, e oferecer mais e' um chip que o `garante` corta \
         em silencio"
    );
}

/// ⭐⭐⭐ **GATE — QUEM SEGURA O PLANO, e o que se reconcilia.**
///
/// ⛔⛔ **As duas leis desta porta têm modo de falha caro e INVISÍVEL a toda
/// régua de cor**, e é por isso que ela é pura:
///
/// 1. **durante um traço não se reconcilia nada** — ali o `Option` da peça está
///    VAZIO (o empréstimo é um `take`), logo um `garante` construiria um plano
///    BRANCO novo em cada quadro, que o `close_stroke` sobrescreveria. *Um
///    alocador de dezenas de MB a 60 Hz que nenhuma imagem acusa.*
/// 2. **só a peça ACTIVA ganha um plano novo** — armar o knob não pode
///    multiplicar `64` amostras por vértice por toda a cena.
///
/// ⚠️ **E a terceira metade é a que o artista vê:** com o traço a segurar, o
/// upload lê o GESTO. Lendo a peça ele subiria `armado = 0` e *a tinta fina
/// desapareceria no instante em que o artista começasse a pintar*.
#[test]
fn a_rota_diz_quem_segura_o_plano_e_o_que_se_reconcilia() {
    // (1) A activa com o traço a segurar: não se reconcilia, e lê-se o gesto.
    assert_eq!(rota(true, true, false, Some(2)), Rota::Emprestado);
    // ⚠️ **Mesmo que a peça AINDA tenha um plano** — o caso do quadro em que o
    // pen-down já correu mas a peça não foi tocada: a resposta é a mesma.
    assert_eq!(rota(true, true, true, Some(2)), Rota::Emprestado);

    // (2) Uma peça que NÃO é a activa não ganha plano, mesmo com o knob armado.
    assert_eq!(
        rota(false, false, false, Some(3)),
        Rota::DaPeca { pedir: None }
    );
    // ⛔ **O CONTROLO:** se ela já tem um, mantém-se no nível do knob — largá-lo
    //    faria a tinta de uma peça sumir por o artista ter escolhido outra.
    assert_eq!(
        rota(false, false, true, Some(3)),
        Rota::DaPeca { pedir: Some(3) }
    );

    // (3) A activa sem traço: reconcilia com o que o knob diz, nos dois lados.
    assert_eq!(
        rota(true, false, false, Some(1)),
        Rota::DaPeca { pedir: Some(1) }
    );
    assert_eq!(rota(true, false, true, None), Rota::DaPeca { pedir: None });

    // ⛔⛔ **E um traço NOUTRA peça não empresta nada a esta** — sem a cerca do
    //    `e_a_activa` o laço leria o plano do gesto para TODA peça do quadro, e
    //    o device desenharia a tinta da peça activa nas vizinhas.
    assert_eq!(
        rota(false, true, true, Some(2)),
        Rota::DaPeca { pedir: Some(2) }
    );
}

/// ⚠️⚠️ **GATE — o LAÇO DE UPLOAD percorre a porta, e não uma cópia da lei.**
///
/// ⛔ *Um gate que chama a função em vez de percorrer a rota afirma que a lei
/// existe, nunca que o quadro a usa* — a frase que o §24 desta linha pagou e
/// que ela própria violou uma wave depois. O laço pede um `wgpu::Device`, logo
/// a única régua que corre no CI é o TEXTO; ele é lido por [`include_str!`],
/// que **deixa de compilar** se o irmão mudar de ficheiro.
#[test]
fn o_laco_de_upload_pergunta_a_porta_de_onde_ler_o_plano() {
    const SLOTS: &str = include_str!("slots.rs");
    let codigo: Vec<&str> = SLOTS
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect();
    assert!(
        codigo.iter().any(|l| l.contains("fn sync_mesh(")),
        "a extracção de código partiu-se: o laço não está nas {} linhas colhidas",
        codigo.len()
    );
    for agulha in [
        "tinta_da_peca::rota(",
        "tinta_da_peca::Rota::DaPeca { pedir }",
        "tinta_da_peca::Rota::Emprestado",
        "upload_tinta_at(",
    ] {
        assert!(
            codigo.iter().any(|l| l.contains(agulha)),
            "o laço de upload deixou de conter `{agulha}` -- ou ele parou de \
             perguntar a' porta, ou parou de subir o plano"
        );
    }
    // ⛔ **E a metade NEGATIVA:** o laço não pode voltar a escrever a decisão à
    //    mão. Com a lei em dois sítios, o gate acima passa a medir a porta
    //    enquanto o produto corre a cópia.
    assert!(
        !codigo
            .iter()
            .any(|l| l.contains("i == self.active && self.stroke.tinta_fina")),
        "a decisão de quem segura o plano voltou a ser escrita dentro do laço: \
         uma lei escrita em dois sítios ainda não é uma lei"
    );
}

/// ⛔⛔ **GATE — o PLANO conta no que a peça pesa, porque uma peça apagada leva
/// o plano para a fila de desfazer.**
///
/// ⚠️ **A redacção anterior do doc do `footprint_bytes` enumerava o que pesava
/// («a pilha inteira, que é tudo o que tem tamanho aqui») e ficou FALSA no dia
/// em que o campo `tinta` nasceu.** O braço `StrokeUndo::RemovedObject` guarda
/// um `SceneObject` INTEIRO, logo apagar uma peça com tinta fina armada punha
/// até `75 MB` na fila **invisíveis ao tecto que existe para os impedir**.
///
/// ⭐ **O CONTROLO é a metade que o torna uma medição:** sem plano o número não
/// pode mudar — senão este gate passaria com um `footprint_bytes` que somasse
/// qualquer coisa.
#[test]
fn o_plano_conta_no_que_a_peca_pesa() {
    use crate::objects::{ObjectId, SceneObject};
    let m = dois_tris();
    let mut peca = SceneObject::new(ObjectId(0), m.clone(), ph2d_mesh::Pose::default());
    let sem = peca.footprint_bytes();

    // CONTROLO: reconciliar para `None` não muda um byte.
    garante(&m, &mut peca.tinta, None);
    assert_eq!(
        peca.footprint_bytes(),
        sem,
        "sem plano o peso da peça não pode mudar"
    );

    garante(&m, &mut peca.tinta, Some(NIVEL_MAX));
    let com = peca.footprint_bytes();
    let plano = peca
        .tinta
        .as_ref()
        .expect("o plano nasceu")
        .footprint_bytes();
    assert!(
        plano > 0,
        "o plano diz pesar zero: a porta da `ph2d-mesh-colors` deixou de medir"
    );
    assert_eq!(
        com - sem,
        plano,
        "a peça cresceu {} bytes e o plano pesa {plano} -- o `footprint_bytes` \
         deixou de o somar, e uma peça apagada leva-o para a fila de desfazer \
         invisível ao tecto",
        com - sem
    );
}

/// ⛔⛔⛔ **GATE — a VOZ é ARMADA pela peça, e não por um literal.**
///
/// ⚠️⚠️ **Ele nasceu de uma MUTAÇÃO SOBREVIVENTE** (2026-09-20): trocar
/// `tinta_fina_armada: o.tinta.is_some()` por `false` no
/// [`super::Sculpt3dScene::diz_a_recusa_do_pen_down`] passava a suíte inteira
/// — *a lei estava certa e o produto nunca a armava*, que é a família que esta
/// casa já pagou meia dúzia de vezes (o defeito lê-se como *«o app não me
/// avisou»*, nunca como *«a regra está errada»*).
///
/// ⛔ **A régua é o TEXTO, e é o tecto honesto:** o
/// `diz_a_recusa_do_pen_down` pede uma [`super::Sculpt3dScene`], que pede um
/// `wgpu::Device` — um gate sobre ele nasceria `#[ignore]` e o CI nunca o
/// correria. O `include_str!` **deixa de compilar** se o ficheiro mudar de
/// sítio, em vez de ficar verde a medir menos.
///
/// ⚠️ **As DUAS metades, e cada uma sozinha mente:** a positiva prova que o
/// campo é alimentado pela peça; a negativa proíbe o literal, que é a forma
/// exacta que a mutação instalou.
#[test]
fn a_voz_da_tinta_fina_e_armada_pela_peca() {
    const RECUSA: &str = include_str!("recusa.rs");
    let codigo: Vec<&str> = RECUSA
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect();
    assert!(
        codigo
            .iter()
            .any(|l| l.contains("fn diz_a_recusa_do_pen_down")),
        "a extracção de código partiu-se: o construtor não está nas {} linhas colhidas",
        codigo.len()
    );
    assert!(
        codigo
            .iter()
            .any(|l| l.contains("tinta_fina_armada: o.tinta.is_some()")),
        "o produto deixou de armar a voz a partir da PEÇA -- a regra fica certa \
         e o aviso nunca soa, que é indistinguível de não existir"
    );
    for morto in ["tinta_fina_armada: false", "tinta_fina_armada: true"] {
        assert!(
            !codigo.iter().any(|l| l.contains(morto)),
            "o produto arma a voz com um LITERAL (`{morto}`): ou ela nunca soa, \
             ou soa sempre -- e as duas leem-se como a regra estar partida"
        );
    }
}
