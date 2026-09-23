//! ⭐⭐⭐⭐ **OS GATES DO TORNO POR FÓRMULA** — a fidelidade, a cerca que a defende, e o eixo.
//!
//! Desde 2026-09-23 (ordem do dono) um torno desce por **fórmula** quando a silhueta é a região
//! entre duas funções da altura: `931 → 124` linhas de WGSL na cena `5`, `2×` no quadro, e o divisor
//! do prévio de `2` para `1`. A rota é **aproximada por construção**, logo ela precisa de três
//! afirmações que a rota exacta não precisava.

use crate::profile_formula::{FIDELIDADE, GRAU, sd_revolve_por_formula};
use ph2d_field::{FillRule, Profile};

/// O copo do [`crate::tests::the_seam_of_a_lathe_lies_on_the_axis_and_is_not_a_wall`] — parede
/// externa, lábio, parede interna e a costura a fechar pelo eixo.
fn copo() -> Profile {
    Profile::new(
        vec![vec![
            [0.00, -0.45],
            [0.30, -0.45],
            [0.30, 0.30],
            [0.24, 0.30],
            [0.24, -0.25],
            [0.00, -0.25],
        ]],
        FillRule::NonZero,
        1.0e-4,
    )
    .expect("o copo é um perfil válido")
}

/// ⭐⭐⭐ **Uma parede ONDULADA — a fixtura que discrimina o GRAU.**
///
/// ⛔⛔ **O copo não a discrimina, e foi uma mutação que o disse:** baixar o [`GRAU`] de `16` para
/// `4` deixava o gate da fidelidade VERDE, porque as paredes de um copo são rectas e um polinómio de
/// grau `4` di-las ao micrómetro. ⇒ *uma fixtura lisa não testa o grau.* Esta tem **três períodos**
/// de ondulação, que um grau `4` não consegue dizer e um `16` consegue.
fn parede_ondulada() -> Profile {
    const N: usize = 64;
    let mut pts = vec![[0.0f32, -0.40]];
    for i in 0..=N {
        #[allow(clippy::cast_precision_loss)]
        let t = i as f32 / N as f32;
        let v = -0.40 + 0.80 * t;
        let u = 0.05f32.mul_add((6.0 * std::f32::consts::PI * t).sin(), 0.30);
        pts.push([u, v]);
    }
    pts.push([0.0, 0.40]);
    Profile::new(vec![pts], FillRule::NonZero, 1.0e-4).expect("a parede ondulada é válida")
}

/// Uma roldana com ESCALÕES — paredes verticais que um polinómio da altura não diz.
fn roldana() -> Profile {
    Profile::new(
        vec![vec![
            [0.00, -0.30],
            [0.40, -0.30],
            [0.40, -0.10],
            [0.12, -0.10],
            [0.12, 0.10],
            [0.40, 0.10],
            [0.40, 0.30],
            [0.00, 0.30],
        ]],
        FillRule::NonZero,
        1.0e-4,
    )
    .expect("a roldana é um perfil válido")
}

/// O raio da parede EXTERNA à altura `v` — a última travessia, varrendo de FORA para dentro.
///
/// ⚠️⚠️ **A 1.ª redacção varria de dentro para fora e exigia o eixo DENTRO do sólido** — o que num
/// copo só vale na base: o piso de população do gate leu `17` de `64` alturas e reprovou. *Foi o
/// controlo a funcionar, e a cura é a régua.*
fn raio_externo(f: &crate::Field, v: f64, u_max: f64) -> Option<f64> {
    let dentro = |u: f64| f.at(u, v, 0.0) < 0.0;
    let passo = u_max / 512.0;
    let mut u = u_max;
    while u > 0.0 && !dentro(u) {
        u -= passo;
    }
    if u <= 0.0 {
        return None;
    }
    let (mut a, mut b) = (u, u + passo);
    for _ in 0..40 {
        let m = 0.5 * (a + b);
        if dentro(m) { a = m } else { b = m }
    }
    Some(0.5 * (a + b))
}

/// ⭐⭐⭐⭐ **A SUPERFÍCIE DA FÓRMULA FICA ONDE A DO CONTORNO DESENHADO ESTÁ**, dentro da
/// [`FIDELIDADE`].
///
/// ⚠️ **A régua é a POSIÇÃO da superfície e não o VALOR do campo**, e isso é obrigatório: a fórmula
/// é normalizada por um majorante da inclinação, logo os valores dela são **conservadores por
/// desenho** (mais rasos) e compará-los mediria a normalização em vez da forma. ⇒ as duas leis são
/// cruzadas por **bissecção**, em unidades da peça.
///
/// ⭐ **E o oráculo é INDEPENDENTE:** a lei exacta é o contorno desenhado, que não passa por nenhum
/// passo da fórmula.
#[test]
fn o_torno_por_formula_fica_dentro_da_fidelidade() {
    // ⚠️ **A fixtura é a ONDULADA e não o copo** — ver [`parede_ondulada`]: uma mutação que baixou o
    // `GRAU` de `16` para `4` sobrevivia ao copo.
    let _ = GRAU;
    let p = parede_ondulada();
    let exacto = crate::Field::from_tree(&crate::profile::probe_sd_revolve_exacto(&p));
    let formula = crate::Field::from_tree(&sd_revolve_por_formula(&p).expect(
        "a parede ondulada é a região entre duas funções da altura, e o grau tem de a dizer \
             dentro da fidelidade",
    ));
    let (plo, phi) = p.bounds();
    let raio = f64::from(phi[0].max(plo[0].abs()));
    let barra = FIDELIDADE * raio;
    let (mut pior, mut medidas) = (0.0f64, 0usize);
    for i in 0..64 {
        let v = f64::from(plo[1])
            + (f64::from(phi[1]) - f64::from(plo[1])) * (f64::from(i) + 0.5) / 64.0;
        let (Some(a), Some(b)) = (
            raio_externo(&exacto, v, raio * 2.0),
            raio_externo(&formula, v, raio * 2.0),
        ) else {
            continue;
        };
        pior = pior.max((a - b).abs());
        medidas += 1;
    }
    // ⚠️ **O piso de população**: sem ele, uma bissecção que deixasse de achar a superfície devolvia
    // `0` alturas medidas e o gate ficava verde a afirmar nada.
    assert!(
        medidas >= 48,
        "CONTROLO: a bissecção achou a superfície em {medidas} de 64 alturas — o gate está a medir \
         o nada"
    );
    assert!(
        pior <= barra,
        "a superfície da fórmula está a {pior:.5} da do contorno desenhado, contra a barra de \
         {barra:.5} ({} do raio) — a rota deixou de ser fiel",
        FIDELIDADE
    );
}

/// ⭐⭐⭐ **A CERCA MORDE: um perfil com ESCALÕES volta ao contorno desenhado.**
///
/// ⛔ Sem ela a fórmula arredondaria uma parede vertical e entregaria outra peça, rápida. ⭐ E o
/// CONTROLO é o copo: sem ele, uma cerca que recusasse TUDO passaria esta metade.
#[test]
fn a_cerca_da_fidelidade_recusa_um_perfil_de_escaloes() {
    assert!(
        sd_revolve_por_formula(&roldana()).is_none(),
        "a roldana de escalões passou a cerca — uma parede vertical não é uma função da altura, e a \
         fórmula entregaria outra peça"
    );
    assert!(
        sd_revolve_por_formula(&copo()).is_some(),
        "CONTROLO: o copo foi recusado — uma cerca que recusa tudo não é uma cerca"
    );
}

/// ⭐⭐⭐ **NA ROTA DA FÓRMULA O EIXO TAMBÉM NÃO É UMA PAREDE** — a terceira metade da lei da costura.
///
/// O irmão [`crate::tests::the_seam_of_a_lathe_lies_on_the_axis_and_is_not_a_wall`] mede a lei
/// EXACTA, onde a costura existe e o campo tem de valer a distância com `‖∇f‖ = 1`. ⚠️ Aqui a
/// costura **não existe** (a fórmula não tem arestas) e o campo é um minorante conservador ⇒ a
/// afirmação que resta é a que importa: *dentro do sólido, sobre o eixo, o campo é francamente
/// negativo — não há nível zero fantasma para a extracção malhar*.
#[test]
fn na_formula_o_eixo_tambem_nao_e_uma_parede() {
    let p = copo();
    let f = crate::Field::from_tree(&sd_revolve_por_formula(&p).expect("o copo desce por fórmula"));
    for y in [-0.43, -0.40, -0.28] {
        let d = f.at(0.0, y, 0.0);
        let parede = -f64::min(y + 0.45, -0.25 - y);
        // ⭐ A barra é METADE da distância verdadeira: um minorante conservador é legítimo, um nível
        // zero no eixo não é. ⚠️ E ela não pode ser `d < 0`: um campo que lesse `−1e-9` seria um
        // nível zero para a esfera-marcha e passaria.
        assert!(
            d <= 0.5 * parede,
            "no eixo, y = {y}: a fórmula leu {d:.4} contra uma parede a {parede:.4} — metade da \
             distância é o mínimo para isto não ser um nível zero fantasma"
        );
    }
}

/// ⛔⛔⛔ **A FÓRMULA NÃO ATRAVESSA A PEÇA — `‖∇f‖ ≤ 1` em toda parte.**
///
/// # Porque este gate existe, e ele nasceu de uma mutação SOBREVIVENTE
///
/// A normalização pela inclinação (`k = 1/√(1 + L²)`) é o que torna o campo um **minorante** da
/// distância: `u − fora(v)` é MAIOR do que a distância à curva quando a parede é inclinada, e uma
/// esfera-marcha que acredite num valor maior do que a distância **dá um passo para dentro do
/// sólido** — a peça sai com buracos.
///
/// ⛔⛔ **Eu escrevi a guarda e não a gateei.** Apagá-la (`k = 1`) passou as `8` paridades de imagem
/// do `ph2d-field-render` e os três gates desta família — *a mutação sobreviveu*. É a forma que esta
/// casa já registou por nome, e a régua que a mata é a directa: `‖∇f‖` é `1` num campo de distância
/// e **maior** num que promete demasiado.
///
/// ⭐ **E o CONTROLO é o majorante estar APERTADO**: uma normalização que dividisse por `100`
/// passaria esta metade e faria toda marcha rastejar. A segunda asserção prende-o por baixo.
#[test]
fn a_formula_nao_atravessa_a_peca() {
    for (nome, p) in [("copo", copo()), ("ondulada", parede_ondulada())] {
        let f = crate::Field::from_tree(
            &sd_revolve_por_formula(&p).unwrap_or_else(|| panic!("{nome} desce por fórmula")),
        );
        let (plo, phi) = p.bounds();
        let raio = f64::from(phi[0].max(plo[0].abs())) * 1.4;
        let (vb, vt) = (f64::from(plo[1]) * 1.4, f64::from(phi[1]) * 1.4);
        const N: i32 = 12;
        let (mut pior, mut melhor) = (0.0f64, f64::MAX);
        for i in -N..=N {
            for j in -N..=N {
                for k in -N..=N {
                    let x = f64::from(i) / f64::from(N) * raio;
                    let z = f64::from(k) / f64::from(N) * raio;
                    let y = vb + (vt - vb) * (f64::from(j + N) / f64::from(2 * N));
                    let g = f.gradient_norm(x, y, z, 1.0e-4);
                    if g.is_finite() && g > 1.0e-3 {
                        pior = pior.max(g);
                        melhor = melhor.min(g);
                    }
                }
            }
        }
        // ⚠️ A folga é de `f32` sobre uma diferença central com `ε = 1e-4`, não um número escolhido.
        assert!(
            pior <= 1.0 + 1.0e-2,
            "{nome}: ‖∇f‖ chegou a {pior:.4} — o campo promete mais do que a distância e a \
             esfera-marcha atravessa a peça"
        );
        // ⭐ **O CONTROLO**: se a normalização fosse grosseira, `‖∇f‖` seria minúsculo em toda
        // parte e a marcha rastejaria. `0,2` é uma ordem de grandeza abaixo de `1` — folga larga
        // para a inclinação real das paredes destas duas peças (`3,4` no vaso ⇒ `k ≈ 0,28`).
        assert!(
            pior >= 0.2,
            "CONTROLO: {nome}: o maior ‖∇f‖ do domínio foi {pior:.4} — uma normalização grosseira \
             passa a metade de cima e faz toda marcha rastejar"
        );
    }
}

/// ⛔⛔⛔⛔ **A FÓRMULA É AJUSTADA UMA VEZ POR PEÇA, NUNCA POR REGIÃO.**
///
/// # O defeito que este gate mede, e ele era meu
///
/// O [`crate::RegionCompiler`] chama o `specialised_profile` **por ladrilho × fatia de
/// profundidade**, e a 1.ª redacção do torno por fórmula ajustava ali: extracção da silhueta (`128`
/// alturas) e **dois** ajustes de mínimos quadrados `17×17`, **por região**.
///
/// **Medido:** ajustar custa `0,0748 ms` (contra `0,0280 ms` que montar a árvore EXACTA da peça
/// inteira custa) e um quadro a `1920×1080` pede `750` regiões com ladrilho `64` e `39 406` com
/// ladrilho `8` ⇒ **`56` a `2 948 ms` por quadro só a ajustar**.
///
/// ⛔⛔ **E o A/B de relógio NÃO o viu:** o traçado de CPU leu `90,17 ms` pela lei exacta e
/// `88,23 ms` pela fórmula — *dois números grandes a cancelarem-se*, a poupança da marcha contra o
/// gasto da montagem. ⇒ **a régua tem de ser a CONTA**, porque a árvore que a região devolve é a
/// MESMA ao bit nos dois caminhos e nenhuma régua de valor os distingue.
///
/// ⭐ **E o CONTROLO é o número de regiões**: sem ele, uma compilação que não especializasse nada
/// leria `1` ajuste e passaria a afirmar o contrário do que mede.
#[test]
fn a_formula_e_ajustada_uma_vez_por_peca() {
    use crate::profile_formula::AJUSTES;
    use std::sync::atomic::Ordering;
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field::Node {
            xform: ph2d_field::Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile: copo() }),
            mods: Vec::new(),
            verb: None,
        }],
        ph2d_field::NodeId(0),
    )
    .expect("o copo girado é um documento válido");
    let reg = crate::hybrid::Registry::default();

    AJUSTES.store(0, Ordering::Relaxed);
    let rc = crate::RegionCompiler::new(&doc);
    let no_arranque = AJUSTES.load(Ordering::Relaxed);
    assert!(
        no_arranque >= 1,
        "CONTROLO: a construção do compilador de regiões não ajustou nada ({no_arranque}) — ou o \
         copo deixou de descer por fórmula, e então este gate mede o nada"
    );

    // ⭐ Uma grelha de regiões, como um quadro pede.
    const LADO: usize = 6;
    let bola = crate::bounds::bounding_ball(&doc, &reg).expect("a bola da peça");
    let (lo, hi) = crate::bounds_clip::march_clip(bola);
    AJUSTES.store(0, Ordering::Relaxed);
    let mut regioes = 0usize;
    for iz in 0..LADO {
        for iy in 0..LADO {
            for ix in 0..LADO {
                let passo = [0, 1, 2].map(|k| (hi[k] - lo[k]) / LADO as f32);
                #[allow(clippy::cast_precision_loss)]
                let rlo = [
                    lo[0] + ix as f32 * passo[0],
                    lo[1] + iy as f32 * passo[1],
                    lo[2] + iz as f32 * passo[2],
                ];
                let rhi = [rlo[0] + passo[0], rlo[1] + passo[1], rlo[2] + passo[2]];
                let _ = rc.compile(&doc, rlo, rhi);
                regioes += 1;
            }
        }
    }
    let por_regiao = AJUSTES.load(Ordering::Relaxed);
    assert_eq!(
        regioes,
        LADO * LADO * LADO,
        "CONTROLO: o laço compilou {regioes} regiões — o gate está a medir o nada"
    );
    assert_eq!(
        por_regiao, 0,
        "a fórmula foi ajustada {por_regiao} vezes ao compilar {regioes} regiões — ela voltou a ser \
         ajustada POR REGIÃO, e isso custa 0,0748 ms cada (56 ms por quadro a 1920×1080)"
    );
}

/// ⭐⭐⭐ **UMA PEÇA FEITA SÓ DE TORNOS POR FÓRMULA NÃO PEDE LADRILHOS.**
///
/// ⚠️ A árvore de uma fórmula não tem arestas para cortar, logo a região dela é a **identidade**.
/// Ladrilhar por causa dela faz o quadro pagar a montagem por região **sem poupar um passo** — e o
/// [`crate::RegionCompiler::is_worth_it`] é a porta que o consumidor lê para decidir se ladrilha.
///
/// ⭐ **E o CONTROLO é um EXTRUDE**, que continua a cortar arestas: sem ele, um `is_worth_it` que
/// devolvesse sempre `false` passaria a metade de cima e desligaria a especialização de toda a casa.
#[test]
fn uma_peca_so_de_formula_nao_pede_ladrilhos() {
    let torno = ph2d_field::FieldDoc::new(
        vec![ph2d_field::Node {
            xform: ph2d_field::Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Revolve { profile: copo() }),
            mods: Vec::new(),
            verb: None,
        }],
        ph2d_field::NodeId(0),
    )
    .expect("o torno");
    let extrude = ph2d_field::FieldDoc::new(
        vec![ph2d_field::Node {
            xform: ph2d_field::Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Extrude {
                profile: copo(),
                half_height: 0.3,
                round: 0.0,
                chamfer: 0.0,
            }),
            mods: Vec::new(),
            verb: None,
        }],
        ph2d_field::NodeId(0),
    )
    .expect("o extrude");
    assert!(
        !crate::RegionCompiler::new(&torno).is_worth_it(),
        "um torno por fórmula pediu ladrilhos — o quadro paga a montagem por região sem poupar um \
         passo"
    );
    assert!(
        crate::RegionCompiler::new(&extrude).is_worth_it(),
        "CONTROLO: o extrude deixou de pedir ladrilhos — um `is_worth_it` sempre falso desliga a \
         especialização de toda a casa"
    );
}
