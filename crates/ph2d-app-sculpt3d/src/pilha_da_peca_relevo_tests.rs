//! Gates do relevo por camada (`docs/3D/30` §5 e §15, W4).

use super::*;
use ph2d_tool_painter::{PainterTool, ReliefPlaneProbe};

const L: u32 = 16;
const H: u32 = 8;
const N: usize = (L * H) as usize;

/// Uma camada da fixtura: relevo (abaixo do joelho do tecto de vidro do 2D, `24`)
/// e corpo em bytes (`/255`, a cobertura do 2D), variados por amostra.
struct Camada {
    altura: Vec<f32>,
    corpo: Vec<u8>,
    depth: f32,
    modo: ReliefComposite,
    visivel: bool,
}

fn camada(s: u32, escala: f32, depth: f32, modo: ReliefComposite, visivel: bool) -> Camada {
    let hash = |i: u32| (i ^ s).wrapping_mul(2_654_435_761);
    Camada {
        altura: (0..N as u32)
            .map(|i| ((hash(i) >> 8) % 1000) as f32 / 1000.0 * escala - escala * 0.25)
            .collect(),
        // Zero, cheio e o meio, para o `Level` enterrar todo, nada e em parte.
        corpo: (0..N as u32)
            .map(|i| match i % 4 {
                0 => 0,
                1 => 255,
                _ => (hash(i) >> 16) as u8,
            })
            .collect(),
        depth,
        modo,
        visivel,
    }
}

/// A fixtura: Add cheia, `Level` a `0,8`, uma ESCONDIDA com relevo, e `Add` com
/// profundidade NEGATIVA no topo.
fn fixtura() -> Vec<Camada> {
    use ReliefComposite::{Add, Level};
    vec![
        camada(1, 6.0, 1.0, Add, true),
        camada(2, 4.0, 0.8, Level, true),
        camada(3, 9.0, 1.0, Add, false),
        camada(4, 3.0, -0.6, Add, true),
    ]
}

/// A pilha da peça com estas camadas, de baixo para cima.
fn pilha_de(camadas: &[Camada]) -> (PilhaDaPeca, Vec<LayerId>) {
    let mut p = PilhaDaPeca::de_partes(LayerStack::new(), BTreeMap::new(), N, Vec::new());
    let mut ids = Vec::new();
    for (k, c) in camadas.iter().enumerate() {
        let id = p.nova_camada(&format!("c{k}")).expect("camada");
        p.planos.get_mut(&id).expect("plano").relevo = Some(
            c.altura
                .iter()
                .zip(&c.corpo)
                .map(|(&a, &b)| [a, f32::from(b) / 255.0])
                .collect(),
        );
        p.marca_relevo(id);
        p.pilha.set_impasto_depth(id, c.depth);
        p.pilha.set_impasto_composite(id, c.modo);
        p.pilha.set_visible(id, c.visivel);
        ids.push(id);
    }
    (p, ids)
}

/// Um plano de `N` amostras (nível `0`: só os vértices).
fn peca_de_n() -> Tinta {
    let t = Tinta::nova(N, [[0u32, 1, 2]].iter().map(|f| &f[..]), 0);
    assert_eq!(t.amostras().len(), N, "a fixtura tem N amostras");
    t
}

fn dois_d(camadas: &[Camada]) -> Vec<f32> {
    let sonda: Vec<ReliefPlaneProbe> = camadas
        .iter()
        .map(|c| ReliefPlaneProbe {
            height: c.altura.clone(),
            cover: c.corpo.clone(),
            depth: c.depth,
            composite: c.modo,
            visible: c.visivel,
        })
        .collect();
    PainterTool::composed_relief_of_planes(L, H, &sonda).expect("planos do tamanho da tela")
}

/// ⭐⭐⭐⭐ **GATE — A dobra do relevo é UMA**: a peça e o Painter 2D (pelo
/// amostrador da luz dele) sobre os MESMOS planos — `Add`, `Level`, profundidade
/// negativa e uma camada escondida —, AO BIT em cada amostra. CONTROLO: trocar a
/// ordem de duas camadas na peça reprova em quase todas (a régua vê a ordem).
#[test]
fn a_dobra_do_relevo_da_peca_e_a_do_2d_ao_bit() {
    let camadas = fixtura();
    let esperado = dois_d(&camadas);
    let (p, ids) = pilha_de(&camadas);
    let relevo = p.relevo_composto().expect("há relevo");
    let diferem = |r: &[[f32; 2]]| {
        r.iter()
            .zip(&esperado)
            .filter(|(a, b)| a[0].to_bits() != b.to_bits())
            .count()
    };
    assert_eq!(diferem(&relevo), 0, "a peça e o 2D, ao bit");
    assert!(
        esperado.iter().all(|h| h.abs() < 24.0),
        "a fixtura fica abaixo do joelho do tecto do 2D"
    );
    let corpo_max = (0..N)
        .map(|i| {
            [0, 1, 3]
                .iter()
                .map(|&k| f32::from(camadas[k].corpo[i]) / 255.0)
                .fold(f32::NEG_INFINITY, f32::max)
        })
        .collect::<Vec<_>>();
    assert!(
        relevo
            .iter()
            .zip(&corpo_max)
            .all(|(r, c)| r[1].to_bits() == c.to_bits()),
        "o corpo é o máximo das visíveis (a cobertura do 2D)"
    );

    // Duas subidas: a 1.ª só a passa pela escondida, que não entra (e não muda nada).
    let mut trocada = p.clone();
    trocada.pilha.move_up(ids[1]);
    assert_eq!(
        diferem(&trocada.relevo_composto().expect("r")),
        0,
        "passar a escondida"
    );
    trocada.pilha.move_up(ids[1]);
    let r = trocada.relevo_composto().expect("há relevo");
    assert!(
        diferem(&r) > N / 2,
        "CONTROLO: a ordem trocada difere em {} de {N}",
        diferem(&r)
    );
}

/// ⭐⭐⭐ **GATE — O neutro é um no-op ao bit**: uma camada só, `Add` a `1`, é o
/// relevo dela (o `-0,0` incluído); uma camada a profundidade `0` dá, ao bit, a
/// peça em que o relevo dela é ZERO (o relevo não conta; a tinta dela continua a
/// ser tinta — o corpo, e a parte que ela tira à de baixo, §18); `Level` sem
/// corpo não enterra nada. CONTROLO: a profundidade `0,5` muda.
#[test]
fn profundidade_zero_e_o_neutro_nao_mudam_um_bit() {
    use ReliefComposite::{Add, Level};
    let mut so = [camada(1, 6.0, 1.0, Add, true)];
    so[0].altura[7] = -0.0;
    let (p, _) = pilha_de(&so);
    let bits = |r: &[[f32; 2]]| r.iter().map(|x| x.map(f32::to_bits)).collect::<Vec<_>>();
    let uma = p.relevo_composto().expect("relevo");
    let plano = p
        .plano(p.base().expect("base"))
        .expect("plano")
        .relevo()
        .expect("relevo");
    assert_eq!(bits(&uma), bits(plano), "uma camada neutra é o relevo dela");

    let mut muda = camada(5, 5.0, 0.0, Add, true);
    let mut zerada = camada(5, 5.0, 1.0, Add, true);
    zerada.altura.fill(0.0);
    let (p0, _) = pilha_de(&[camada(1, 6.0, 1.0, Add, true), muda]);
    let (pz, _) = pilha_de(&[camada(1, 6.0, 1.0, Add, true), zerada]);
    let r0 = p0.relevo_composto().expect("r");
    assert_eq!(
        bits(&r0),
        bits(&pz.relevo_composto().expect("r")),
        "profundidade 0 = o relevo dela a zero, ao bit"
    );
    let alturas = |r: &[[f32; 2]]| r.iter().map(|x| x[0].to_bits()).collect::<Vec<_>>();
    let base_so = pilha_de(&[camada(1, 6.0, 1.0, Add, true)])
        .0
        .relevo_composto()
        .expect("r");
    let mut sem_corpo = camada(5, 5.0, 1.0, Level, true);
    sem_corpo.corpo.fill(0);
    let (pl, _) = pilha_de(&[camada(1, 6.0, 1.0, Add, true), sem_corpo]);
    let rl = pl.relevo_composto().expect("r");
    assert_eq!(
        alturas(&rl),
        alturas(&base_so),
        "Level sem corpo não enterra nada"
    );

    muda = camada(5, 5.0, 0.5, Add, true);
    let (ph, _) = pilha_de(&[camada(1, 6.0, 1.0, Add, true), muda]);
    assert_ne!(
        alturas(&ph.relevo_composto().expect("r")),
        alturas(&r0),
        "CONTROLO: a 0,5 o relevo dela conta"
    );
}

/// ⭐⭐⭐ **GATE — O traço escreve o relevo SÓ da camada activa e o desfazer
/// tira-o**: a cópia de trabalho leva o relevo da activa (não o composto), ele
/// desce só a ela, a peça incremental é a inteira ao bit, e a troca do desfazer
/// devolve a peça de antes ao bit.
#[test]
fn o_traco_escreve_o_relevo_so_da_activa_e_o_desfazer_o_tira() {
    use ReliefComposite::Add;
    let (mut p, ids) = pilha_de(&[camada(1, 6.0, 1.0, Add, true)]);
    let cima = p.nova_camada("cima").expect("cima");
    let mut peca = peca_de_n();
    p.pinta_tinta(&mut peca, || vec![[1.0; 3]; N]);
    let peca_antes = peca.clone();
    let base_antes = p.plano(ids[0]).expect("base").relevo().map(<[_]>::to_vec);

    let (id, mut w) = p.trabalho_da_activa(&peca).expect("activa");
    assert_eq!(id, cima);
    assert!(
        w.relevo().is_none(),
        "a cópia leva o relevo DA activa (nenhum)"
    );
    let idx = [3u32, 4, 5, 60];
    for &i in &idx {
        w.relevo_mut()[i as usize] = [2.5, 1.0];
    }
    p.recebe_do_traco(id, &w, &idx);
    p.compoe_amostras(&idx, &mut peca, || vec![[1.0; 3]; N]);
    assert_eq!(
        p.plano(ids[0]).expect("base").relevo().map(<[_]>::to_vec),
        base_antes,
        "a base fica"
    );
    assert!(
        p.pilha().get(cima).is_some_and(|c| c.has_relief),
        "o painel sabe"
    );
    let r = p.plano(cima).expect("cima").relevo().expect("relevo");
    assert_eq!(r[4], [2.5, 1.0]);
    assert_eq!(
        r.iter().filter(|x| **x != [0.0; 2]).count(),
        idx.len(),
        "só as sujas"
    );
    let bits = |r: &[[f32; 2]]| r.iter().map(|x| x.map(f32::to_bits)).collect::<Vec<_>>();
    let inteira = p.relevo_composto().expect("relevo");
    assert_eq!(
        bits(peca.relevo().expect("r")),
        bits(&inteira),
        "incremental = inteira"
    );
    assert_ne!(peca.relevo(), peca_antes.relevo(), "CONTROLO: a peça mudou");

    p.fim_do_traco();
    p.troca_relevo(cima, &idx, &[[0.0; 2]; 4]).expect("desfaz");
    p.compoe_amostras(&idx, &mut peca, || vec![[1.0; 3]; N]);
    assert_eq!(
        bits(peca.relevo().expect("r")),
        bits(peca_antes.relevo().expect("r")),
        "o desfazer devolve a peça ao bit"
    );
}

/// ⭐⭐ **GATE — A cópia leva o relevo** (com a profundidade e o modo) e a base
/// continua permanente (a lei do 2D).
#[test]
fn duplicar_leva_o_relevo_e_a_base_fica() {
    let (mut p, ids) = pilha_de(&fixtura()[..2]);
    let copia = p.duplica(ids[1]).expect("duplica");
    assert_eq!(
        p.plano(copia).and_then(|c| c.relevo().map(<[_]>::to_vec)),
        p.plano(ids[1]).and_then(|c| c.relevo().map(<[_]>::to_vec))
    );
    let meta = |id| {
        p.pilha()
            .get(id)
            .map(|c| (c.has_relief, c.impasto_depth, c.impasto_composite))
    };
    assert_eq!(meta(copia), meta(ids[1]));
    assert!(p.sincronizada());
    assert_eq!(p.apaga(ids[0]), Err(RecusaDaPilha::ABase));
}

/// ⭐⭐ **GATE — A recomposição só redobra o relevo quando a FORMA da dobra
/// muda**: a opacidade não; a profundidade, o modo e esconder uma camada com
/// relevo sim — e o relevo da peça passa a ser o composto.
#[test]
fn so_a_forma_da_dobra_redobra_o_relevo() {
    let (mut p, ids) = pilha_de(&fixtura());
    let mut peca = peca_de_n();
    assert!(p.redobra_o_relevo(&mut peca), "a 1.ª dobra");
    assert!(!p.redobra_o_relevo(&mut peca), "nada mudou");
    p.pilha.set_opacity(ids[3], 0.3);
    assert!(
        !p.redobra_o_relevo(&mut peca),
        "a opacidade não mexe no relevo"
    );
    for muda in [
        |s: &mut LayerStack, id| s.set_impasto_depth(id, 0.25),
        |s: &mut LayerStack, id| s.toggle_impasto_composite(id),
        |s: &mut LayerStack, id| s.set_visible(id, false),
    ] {
        muda(&mut p.pilha, ids[3]);
        assert!(p.redobra_o_relevo(&mut peca));
        assert_eq!(peca.relevo().map(<[_]>::to_vec), p.relevo_composto());
    }
}

/// ⭐⭐⭐ **GATE (report do dono, 04/10: a tinta vermelha com uma orla de relevo à volta) — a encosta
/// que uma camada de cima tem FORA da sua tinta não conta sobre a tinta da de baixo**; dentro da tinta
/// dela conta inteira; e numa camada SÓ a encosta fica, ao bit (a luz da peça já a apaga pelo corpo).
#[test]
fn a_encosta_fora_da_tinta_de_uma_camada_de_cima_nao_conta() {
    use ReliefComposite::Add;
    let mut base = camada(1, 6.0, 1.0, Add, true);
    base.corpo.fill(255);
    let mut cima = camada(2, 4.0, 1.0, Add, true);
    for (i, c) in cima.corpo.iter_mut().enumerate() {
        *c = if i % 2 == 0 { 0 } else { 255 };
    }
    let (p, _) = pilha_de(&[base, cima]);
    let r = p.relevo_composto().expect("relevo");
    let base_so = {
        let mut b = camada(1, 6.0, 1.0, Add, true);
        b.corpo.fill(255);
        pilha_de(&[b]).0.relevo_composto().expect("r")
    };
    let cima_altura = camada(2, 4.0, 1.0, Add, true).altura;
    for i in 0..N {
        if i % 2 == 0 {
            assert_eq!(
                r[i][0].to_bits(),
                base_so[i][0].to_bits(),
                "{i}: fora da tinta de cima, só a base"
            );
        } else {
            assert_eq!(
                r[i][0],
                base_so[i][0] + cima_altura[i],
                "{i}: dentro, as duas somam"
            );
        }
    }
    let mut so = camada(2, 4.0, 1.0, Add, true);
    for (i, c) in so.corpo.iter_mut().enumerate() {
        *c = if i % 2 == 0 { 0 } else { 255 };
    }
    let alturas = so.altura.clone();
    let (q, _) = pilha_de(&[so]);
    let rs = q.relevo_composto().expect("r");
    assert!(
        rs.iter()
            .zip(&alturas)
            .all(|(a, b)| a[0].to_bits() == b.to_bits()),
        "CONTROLO: numa camada só a encosta fica"
    );
}

/// ⭐⭐⭐ **GATE (a foto de 04/10: o «fantasma») — esconder uma camada que ganhou relevo por um TRAÇO
/// tira o relevo dela da peça.** O traço actualiza a peça aos bocados; sem renovar a assinatura da
/// dobra, esconder a camada voltava à assinatura guardada antes do traço e a redobra dizia «nada
/// mudou». CONTROLO: com ela visível o relevo dela está na peça.
#[test]
fn esconder_uma_camada_que_ganhou_relevo_num_traco_tira_o_relevo_dela() {
    use ReliefComposite::Add;
    let mut base = camada(1, 6.0, 1.0, Add, true);
    base.corpo.fill(255);
    let (mut p, ids) = pilha_de(&[base]);
    let mut peca = peca_de_n();
    p.pinta_tinta(&mut peca, || vec![[1.0; 3]; N]);
    assert!(!p.redobra_o_relevo(&mut peca), "a peça já é a dobra");
    let cima = p.nova_camada("cima").expect("cima");
    let (id, mut w) = p.trabalho_da_activa(&peca).expect("activa");
    let idx: Vec<u32> = (10..40).collect();
    for &i in &idx {
        w.relevo_mut()[i as usize] = [0.05, 1.0];
    }
    p.recebe_do_traco(id, &w, &idx);
    p.compoe_amostras(&idx, &mut peca, || vec![[1.0; 3]; N]);
    p.fim_do_traco();
    let so_base = p
        .plano(ids[0])
        .and_then(|b| b.relevo().map(<[_]>::to_vec))
        .expect("base");
    assert_ne!(
        peca.relevo().map(<[_]>::to_vec),
        Some(so_base.clone()),
        "CONTROLO: visível conta"
    );
    let mut escondida = p.pilha().clone();
    escondida.set_visible(cima, false);
    p.troca_metadado(escondida).expect("esconde");
    assert!(
        p.redobra_o_relevo(&mut peca),
        "esconder mudou a forma da dobra"
    );
    let bits = |r: &[[f32; 2]]| r.iter().map(|x| x.map(f32::to_bits)).collect::<Vec<_>>();
    assert_eq!(
        bits(peca.relevo().expect("r")),
        bits(&so_base),
        "escondida, a peça é a base"
    );
}
