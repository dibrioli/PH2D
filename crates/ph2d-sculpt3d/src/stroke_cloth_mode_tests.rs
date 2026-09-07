//! ⭐⭐ **OS SELECTORES DO PINCEL DE TECIDO CHEGAM AO MOTOR** — o modo de
//! deformação e a área simulada.
//!
//! ⛔⛔ **É a pergunta que nenhum instrumento desta casa faz** (CLAUDE.md §5.0):
//! o censo de registo mede se o chip é focalizável e os `seam_*` provam que o
//! clique chega à ferramenta — **nenhum pergunta se o VALOR chega a um
//! consumidor**. Um selector que o adaptador lesse e a matemática descartasse
//! passaria em todos eles.
//!
//! Irmão do [`super::stroke_cloth_tests`], e o corte é *o que a LEI faz* (lá)
//! contra *o que o PAINEL alcança* (aqui).

use super::cloth_tests::{arrastar, pincel, plano};
use crate::{Brush, Dab, SculptStroke, Symmetry};

/// ⭐⭐⭐ **GATE — os OITO modos de deformação dão OITO panos diferentes.**
///
/// ⛔⛔ **É a pergunta que nenhum instrumento desta casa faz** (CLAUDE.md §5.0):
/// o `hit_indexed_ids_are_registered` mede se o chip é focalizável e os `seam_*`
/// provam que o clique chega à ferramenta — **nenhum pergunta se o VALOR chega a
/// um consumidor**. Um `cloth_mode` que o adaptador lesse e a matemática
/// descartasse passaria em todos eles.
///
/// ⚠️ **E o anti-vácuo é metade do gate:** sem o piso, oito modos que não
/// fizessem nada dariam oito panos idênticos ao repouso e a desigualdade
/// «todos diferentes» seria falsa — mas oito que fizessem a MESMA coisa também.
/// Por isso as duas metades: cada um move, e não há dois iguais.
#[test]
fn os_oito_modos_de_deformacao_dao_oito_panos_diferentes() {
    let antes = plano();
    let saidas: Vec<(crate::ClothMode, Vec<[f32; 3]>)> = crate::ClothMode::ALL
        .into_iter()
        .map(|modo| {
            let b = Brush {
                cloth_mode: modo,
                ..pincel()
            };
            let (m, _) = arrastar(8, &b);
            (modo, m.positions().to_vec())
        })
        .collect();
    for (modo, p) in &saidas {
        let pior = (0..antes.vert_count())
            .map(|v| {
                let (a, b) = (antes.positions()[v], p[v]);
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
            })
            .fold(0.0f32, f32::max);
        assert!(
            pior > 1e-4,
            "{modo:?} nao moveu NADA ({pior:.2e}) -- o chip escreve e ninguem le"
        );
    }
    for (i, (ma, pa)) in saidas.iter().enumerate() {
        for (mb, pb) in saidas.iter().skip(i + 1) {
            assert!(
                pa != pb,
                "{ma:?} e {mb:?} dao o MESMO pano ao bit -- um dos dois nao chega \
                 ao motor, ou a matematica descarta-o"
            );
        }
    }
}

/// ⭐⭐ **GATE — as TRÊS áreas simuladas dão três panos diferentes.**
///
/// A área decide o que entra na simulação, o centro da banda e — no *Local* — a
/// lista de restrições em duplicado (espec §2.1, §5.2-bis). Se ela não chegasse,
/// as três leriam igual.
#[test]
fn as_tres_areas_simuladas_dao_tres_panos_diferentes() {
    let saidas: Vec<(crate::ClothArea, Vec<[f32; 3]>)> = crate::ClothArea::ALL
        .into_iter()
        .map(|area| {
            let b = Brush {
                cloth_area: area,
                ..pincel()
            };
            let (m, _) = arrastar(8, &b);
            (area, m.positions().to_vec())
        })
        .collect();
    for (i, (aa, pa)) in saidas.iter().enumerate() {
        for (ab, pb) in saidas.iter().skip(i + 1) {
            assert!(pa != pb, "{aa:?} e {ab:?} dao o MESMO pano ao bit");
        }
    }
}

/// ⭐⭐⭐ **GATE — o gesto que o tecido lê é o CAMINHO PROJECTADO no plano do
/// ecrã, e é o OLHO que define esse plano.**
///
/// ⛔⛔ **Esta costura não tinha régua nenhuma, e a razão é a fixtura**: as
/// grelhas deste ficheiro são planas e vistas de frente, e ali a projecção é um
/// **no-op** — o adaptador podia entregar o caminho 3D cru que nenhum gate desta
/// crate mudava de cor. *Uma lei que só é exercida por uma vista que a fixtura
/// não tem é uma lei sem gate.* A cura é inclinar o OLHO, não a malha.
///
/// A propriedade é observável pela porta pública, e tem DUAS metades:
/// - **inclinar o olho MUDA** o pano nos sete modos que lêem `δ`, porque o plano
///   do ecrã roda e a projecção do mesmo caminho passa a ser outra;
/// - **e NÃO muda** no arrasto, que é o único modo cuja direcção sai da
///   diferença dos dois pontos 3D (espec §4.2/§4.3).
///
/// ⚠️ **Sem a segunda metade o gate ficaria verde sobre um adaptador que
/// projectasse TUDO**, arrasto incluído.
#[test]
fn o_olho_define_o_plano_em_que_o_tecido_le_o_gesto() {
    let de_frente = [0.0f32, 0.0, -1.0];
    let inclinado = {
        // ⚠️ A inclinação tem de ser no eixo do TRAÇO: com ela no eixo `y` o
        // caminho, que corre em `x`, já é perpendicular ao olho, a projecção não
        // tira nada — a primeira redacção deste gate era VAZIA por isso.
        let v = [0.6f32, 0.0, -1.0];
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / l, v[1] / l, v[2] / l]
    };
    let correr = |olho: [f32; 3], modo: crate::ClothMode| -> Vec<[f32; 3]> {
        let mut mesh = plano();
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        let b = Brush {
            cloth_mode: modo,
            ..pincel()
        };
        for k in 0..8 {
            // ⚠️ O caminho é derivado dos CENTROS pelo traço, não do construtor
            // — e ele é o mesmo nas duas corridas. O que muda é só o olho.
            let c = [0.02 * k as f32, 0.0, 0.0];
            s.dab(
                &mut mesh,
                &b,
                &Dab::at(c, b.radius, olho),
                Symmetry::default(),
            );
        }
        mesh.positions().to_vec()
    };
    let antes = plano();
    let frente_grab = correr(de_frente, crate::ClothMode::Grab);
    let movidos = frente_grab
        .iter()
        .zip(antes.positions())
        .filter(|(a, b)| a != b)
        .count();
    assert!(movidos > 100, "so' {movidos} movidos -- vacuo");
    assert_ne!(
        frente_grab,
        correr(inclinado, crate::ClothMode::Grab),
        "inclinar o OLHO nao mudou o pano no Grab -- o adaptador esta a entregar o \
         caminho 3D cru onde a lei pede a projeccao no plano do ecra (espec §4.3)"
    );
    assert_eq!(
        correr(de_frente, crate::ClothMode::Drag),
        correr(inclinado, crate::ClothMode::Drag),
        "inclinar o OLHO mudou o ARRASTO -- ele e' o unico modo cuja direccao sai da \
         diferenca dos dois pontos 3D, e nao de `δ`"
    );
}

/// ⭐⭐⭐ **GATE — OS SETE KNOBS QUE FALTAVAM CHEGAM AO MOTOR, e o neutro é o de
/// antes AO BIT.**
///
/// ⛔⛔ **Eles existiam na LEI e não existiam no painel até 07/09:** a tradução
/// `Brush → Pincel` escrevia `Radial` literal na forma de queda e caía no
/// `Pincel::default()` para os outros seis, e o corpus do oráculo tem fixture
/// para cada um (`plano_*_plano_local` × 4 · `massa2` · `amort05` · `amort1` ·
/// `plast05` · `pino` · `preset` com limite `5`). *Uma lei medida na bancada e
/// não ligada no produto é uma lei que o artista não tem.*
///
/// ⚠️ **As duas metades, e nenhuma basta sozinha:** (a) mexer cada knob muda o
/// pano — senão ele é um controlo morto; (b) com os sete nas omissões a saída é
/// **byte-idêntica** à do pincel de antes desta wave — senão a wave mudou o que
/// o artista já tinha.
///
/// ⚠️ **O pino corre em área *Local*** de propósito: a lei recusa-o nas outras
/// (`ClothArea::offers_pin`), e no `Dynamic` de omissão ele seria um knob que o
/// gate declara vivo e o motor descarta — que é exactamente o defeito que este
/// ficheiro existe para apanhar.
#[test]
fn os_sete_knobs_do_tecido_chegam_ao_motor() {
    let neutro = pincel();
    let (base, _) = arrastar(8, &neutro);
    let base = base.positions().to_vec();
    // (b) o neutro é o de antes: a malha de repouso mexeu-se, e mexeu-se para
    // onde o pincel pré-wave a levava.
    let antes = plano();
    let moveu = base
        .iter()
        .zip(antes.positions())
        .map(|(a, b)| {
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max);
    assert!(moveu > 0.01, "o pincel neutro nao moveu nada ({moveu:.5})");

    // (a) cada knob, fora da omissão, muda o pano.
    let casos: [(&str, Brush); 7] = [
        (
            "force falloff",
            Brush {
                cloth_force_falloff: crate::ClothForceFalloff::Plane,
                ..neutro.clone()
            },
        ),
        (
            "simulation limit",
            Brush {
                cloth_limit: 5.0,
                ..neutro.clone()
            },
        ),
        (
            "simulation falloff",
            Brush {
                cloth_falloff: 0.2,
                ..neutro.clone()
            },
        ),
        (
            "pin",
            Brush {
                cloth_area: crate::ClothArea::Local,
                cloth_pin: true,
                ..neutro.clone()
            },
        ),
        (
            "mass",
            Brush {
                cloth_mass: 2.0,
                ..neutro.clone()
            },
        ),
        (
            "damping",
            Brush {
                cloth_damping: 1.0,
                ..neutro.clone()
            },
        ),
        (
            "plasticity",
            Brush {
                cloth_plasticity: 0.5,
                ..neutro.clone()
            },
        ),
    ];
    for (nome, b) in casos {
        // ⚠️ O caso do PINO troca a área junto, logo o controlo dele é a mesma
        // área SEM o pino — senão ele mediria a troca de área.
        let referencia = if nome == "pin" {
            let (m, _) = arrastar(
                8,
                &Brush {
                    cloth_area: crate::ClothArea::Local,
                    ..neutro.clone()
                },
            );
            m.positions().to_vec()
        } else {
            base.clone()
        };
        let (m, _) = arrastar(8, &b);
        let d = m
            .positions()
            .iter()
            .zip(&referencia)
            .map(|(a, c)| {
                ((a[0] - c[0]).powi(2) + (a[1] - c[1]).powi(2) + (a[2] - c[2]).powi(2)).sqrt()
            })
            .fold(0.0f32, f32::max);
        assert!(
            d > 1e-4,
            "`{nome}` fora da omissao move o pano {d:.6} -- ele nao chega ao motor"
        );
    }
}

/// ⭐ **GATE — o PINO só existe onde a lei o aceita, e a porta é UMA.**
///
/// ⚠️ O painel não pinta a caixa fora da área *Local* e a tradução
/// `Brush → Pincel` não a honra — as duas perguntam a
/// [`crate::ClothArea::offers_pin`]. ⛔ Duas cópias divergiriam num interruptor
/// que aparece e não muda um vértice.
#[test]
fn o_pino_e_inerte_fora_da_area_local() {
    for area in crate::ClothArea::ALL {
        let sem = Brush {
            cloth_area: area,
            cloth_pin: false,
            ..pincel()
        };
        let com = Brush {
            cloth_pin: true,
            ..sem.clone()
        };
        let (a, _) = arrastar(8, &sem);
        let (b, _) = arrastar(8, &com);
        let igual = a.positions() == b.positions();
        assert_eq!(
            igual,
            !area.offers_pin(),
            "area {}: o pino {} a malha, e `offers_pin` diz {}",
            area.label(),
            if igual { "NAO muda" } else { "muda" },
            area.offers_pin()
        );
    }
}

/// ⭐⭐ **GATE — o NEUTRO dos sete knobs é o pincel que a bancada corre**, e a
/// ÚNICA diferença é a que o `f32` do painel não sabe representar.
///
/// ⛔⛔ **A 1.ª redacção desta wave afirmou «byte-idêntico» e a medição
/// desmentiu-a:** o amortecimento de omissão é `0,01`, que **não existe em
/// `f32`** — o knob do artista guarda `0,009999999776482582`, e antes desta wave
/// a tradução caía no `Solver::default()`, que é `0,01` exacto em `f64`. ⇒ o
/// neutro mudou por `2,2·10⁻¹⁰` relativos.
///
/// ⚠️ **E a cura não é fazer a porta «voltar» ao `f64` quando o knob está na
/// omissão:** isso seria um caminho escondido em que o mesmo número na tela
/// significa duas coisas conforme alguém lhe tenha tocado. *Um knob de `f32`
/// exprime o que um `f32` exprime*, e é isso que este gate fixa — com o número
/// dentro, para que a próxima leitura não tenha de o redescobrir.
#[test]
fn o_neutro_dos_sete_knobs_e_o_pincel_da_bancada() {
    let b = pincel();
    let p = crate::stroke::stroke_cloth_ref::pincel_de_para_sonda(&b, 1);
    let d = ph2d_cloth::verlet_gesto::Pincel::default();
    // Os que o `f32` representa EXACTAMENTE — aqui a igualdade é dura.
    assert_eq!(p.limite, d.limite, "simulation limit");
    assert_eq!(p.banda, d.banda, "simulation falloff");
    assert_eq!(p.pino, d.pino, "pin");
    assert_eq!(p.solver.massa, d.solver.massa, "cloth mass");
    assert_eq!(p.solver.plasticidade, d.solver.plasticidade, "plasticity");
    assert_eq!(p.solver.varreduras, d.solver.varreduras, "varreduras");
    assert_eq!(
        p.falloff_forca,
        ph2d_cloth::verlet_gesto::FalloffForca::Radial,
        "force falloff"
    );
    // ⚠️ **O amortecimento é o um que não cabe**, e o valor está aqui por
    // extenso: ele é `f64::from(0.01_f32)`, e nada mais.
    assert_eq!(
        p.solver.amortecimento,
        f64::from(0.01_f32),
        "cloth damping: o `f32` do painel nao representa 0,01"
    );
    let desvio = (p.solver.amortecimento - d.solver.amortecimento).abs() / d.solver.amortecimento;
    assert!(
        desvio < f64::from(f32::EPSILON),
        "o amortecimento desviou {desvio:.3e} do que a bancada corre -- acima do \
         que o epsilon do `f32` explica ({:.3e}) -- alguem mexeu na omissao",
        f64::from(f32::EPSILON)
    );
}

/// **SONDA — o neutro dos sete knobs é o pincel de ANTES, ao bit.**/// **SONDA — o neutro dos sete knobs é o pincel de ANTES, ao bit.**
///
/// ⚠️ Ela existe porque a afirmação «o mundo pré-wave é byte-idêntico» não é
/// demonstrável de dentro do produto de hoje: o pincel de antes já não existe.
/// O que ela mede é o mesmo por outra via — os sete nas omissões entregam o
/// `Pincel::default()` da bancada, e é ele que o corpus do oráculo corre.
#[test]
#[ignore = "sonda"]
fn sonda_do_neutro_dos_sete_knobs() {
    let b = pincel();
    let p = crate::stroke::stroke_cloth_ref::pincel_de_para_sonda(&b, 1);
    let d = ph2d_cloth::verlet_gesto::Pincel::default();
    println!("limite {} vs {}", p.limite, d.limite);
    println!("banda {} vs {}", p.banda, d.banda);
    println!("pino {} vs {}", p.pino, d.pino);
    println!("massa {} vs {}", p.solver.massa, d.solver.massa);
    println!(
        "amort {} vs {}",
        p.solver.amortecimento, d.solver.amortecimento
    );
    println!(
        "plast {} vs {}",
        p.solver.plasticidade, d.solver.plasticidade
    );
    println!(
        "varreduras {} vs {}",
        p.solver.varreduras, d.solver.varreduras
    );
}

/// **SONDA — a GEOMETRIA do oráculo, corrida pelo caminho do PRODUTO.**
///
/// ⚠️⚠️ **Existe porque o gate de artefacto do produto corre num regime que o
/// corpus inteiro do oráculo não alcança:** ali o deslocamento máximo vale
/// `4,8 · R`, e o traço mais fundo de 78 fixtures — incluindo o de **36 passos**,
/// gravado de propósito para o alcançar — satura em `0,76 · R`. *Ou o produto
/// entrega outra coisa que a lei, ou a fixtura dele é outro gesto.* Esta sonda
/// põe as duas geometrias lado a lado e imprime o número.
#[test]
#[ignore = "sonda"]
fn sonda_da_geometria_do_oraculo_pelo_produto() {
    use crate::Verb;
    use crate::stroke::cloth_artefatos_tests::plano_n;
    use crate::stroke::cloth_tests::dab_em;
    for (n, raio, passos, avanco, area) in [
        // A do oráculo: grelha 64², raio `0,35`, caminho `0,6` em 11 avanços.
        (
            64usize,
            0.35f32,
            12usize,
            0.6f32 / 11.0,
            crate::ClothArea::Local,
        ),
        // A mesma, em 36 passos — o traço longo.
        (64, 0.35, 36, 0.6 / 35.0, crate::ClothArea::Local),
        // A mesma, na área DINÂMICA — que é a omissão do produto.
        (64, 0.35, 12, 0.6 / 11.0, crate::ClothArea::Dynamic),
        // A do gate de artefacto do produto, nas duas áreas.
        (144, 0.30, 35, 0.02, crate::ClothArea::Dynamic),
        (144, 0.30, 35, 0.02, crate::ClothArea::Local),
    ] {
        let antes = plano_n(n);
        let mut mesh = plano_n(n);
        let b = Brush {
            verb: Verb::Cloth,
            radius: raio,
            strength: 1.0,
            cloth_area: area,
            ..Brush::default()
        };
        let pc = crate::stroke::stroke_cloth_ref::pincel_de_para_sonda(&b, 1);
        println!(
            "   pincel: curva={:?} dureza={} forca={} area={:?} queda={:?} limite={} banda={} massa={} amort={}",
            pc.curva,
            pc.dureza,
            pc.forca,
            pc.area,
            pc.falloff_forca,
            pc.limite,
            pc.banda,
            pc.solver.massa,
            pc.solver.amortecimento
        );
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        for k in 0..passos {
            let c = [avanco * k as f32, 0.0, 0.0];
            let passo = if k == 0 { [0.0; 3] } else { [avanco, 0.0, 0.0] };
            s.dab(
                &mut mesh,
                &b,
                &dab_em(c, b.radius, passo),
                Symmetry::default(),
            );
        }
        let max = (0..antes.vert_count())
            .map(|v| {
                let (p, q) = (antes.positions()[v], mesh.positions()[v]);
                ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
            })
            .fold(0.0f32, f32::max);
        println!(
            "n={n:>4} R={raio:.2} passos={passos:>3} area={area:?} caminho={:.3}R  ->  max={max:.4} = {:.2}R",
            avanco * (passos - 1) as f32 / raio,
            max / raio
        );
    }
}

/// **SONDA — O QUE UM DAB DE TECIDO CUSTA, e onde ele encosta no quadro.**
///
/// ⛔⛔ **Ela existe porque não há tecto nenhum no caminho do tecido, e nunca
/// houve medição.** Nem a `ph2d-cloth` nem o adaptador escrevem um `MAX_*`, um
/// «por ora» ou uma cerca de densidade — o que está certo pela §0.0 do CLAUDE.md
/// (*meça antes de limitar*), e deixa a outra metade por fazer: **ninguém sabe
/// onde o pincel deixa de caber num quadro.** *Um limite que ninguém escreveu e
/// um limite que ninguém mediu leem-se igual até o artista abrir uma peça densa.*
///
/// Colunas: `1.º` é o dab que CONSTRÓI a lista de restrições (uma vez por traço)
/// e `regime` é a mediana dos seguintes. O orçamento de um quadro a 60 fps é
/// `16,7 ms`. ⚠️ **Não asserta nada** — é relógio, e o `/proc/loadavg` do momento
/// manda mais que o código (CLAUDE.md §5).
#[test]
#[ignore = "sonda"]
fn sonda_do_custo_de_um_dab_de_tecido() {
    use crate::Verb;
    use crate::stroke::cloth_artefatos_tests::plano_n;
    use crate::stroke::cloth_tests::dab_em;
    use std::time::Instant;
    println!(
        "{:>6} {:>9} | {:>10} {:>10} {:>10} | {:>8}",
        "n", "vertices", "1.º (ms)", "regime(ms)", "pior (ms)", "% quadro"
    );
    for (n, area) in [
        (64usize, crate::ClothArea::Dynamic),
        (128, crate::ClothArea::Dynamic),
        (160, crate::ClothArea::Dynamic),
        (160, crate::ClothArea::Global),
    ] {
        let mut mesh = plano_n(n);
        let b = Brush {
            verb: Verb::Cloth,
            radius: 0.30,
            strength: 1.0,
            cloth_area: area,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        let mut ms: Vec<f64> = Vec::new();
        for k in 0..20 {
            let c = [0.02 * k as f32, 0.0, 0.0];
            let passo = if k == 0 { [0.0; 3] } else { [0.02, 0.0, 0.0] };
            let t0 = Instant::now();
            s.dab(
                &mut mesh,
                &b,
                &dab_em(c, b.radius, passo),
                Symmetry::default(),
            );
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        let primeiro = ms[0];
        let mut resto: Vec<f64> = ms[1..].to_vec();
        resto.sort_by(f64::total_cmp);
        let mediana = resto[resto.len() / 2];
        let pior = resto[resto.len() - 1];
        println!(
            "{n:>6} {:>9} | {primeiro:>10.2} {mediana:>10.2} {pior:>10.2} | {:>7.1}% {area:?}",
            mesh.vert_count(),
            100.0 * mediana / 16.7
        );
    }
}

/// ⭐⭐⭐ **GATE — A BASE PERSISTENTE CHEGA AO MOTOR PELA PORTA DO ARTISTA, e ela
/// SATURA** (espec §6.4).
///
/// ⛔⛔ **É o gate que separa «a lei existe» de «o artista alcança-a»** — a
/// `ph2d-cloth` reproduz as fixtures do oráculo, e entre ela e a mão há o
/// interruptor, o botão que congela a base e a validação por comprimento.
///
/// ⚠️ **A ORDEM é a lei, e as duas metades estão aqui:** gravar a base **antes**
/// dos traços faz a deformação saturar; gravá-la **depois** de um traço é um
/// **no-op exacto**, porque ali ela É o repouso do traço seguinte. *A experiência
/// que ocorre primeiro a quem a desenha é a segunda, e ela não mede nada.*
#[test]
fn a_base_persistente_chega_ao_motor_e_satura() {
    use crate::Verb;
    use crate::stroke::cloth_artefatos_tests::plano_n;
    use crate::stroke::cloth_tests::dab_em;
    let antes = plano_n(48);
    let pico = |m: &ph2d_mesh::Mesh| {
        (0..antes.vert_count())
            .map(|v| {
                let (p, q) = (antes.positions()[v], m.positions()[v]);
                ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
            })
            .fold(0.0f32, f32::max)
    };
    // `quando`: `None` = sem base · `Some(0)` = base ANTES do 1.º traço ·
    // `Some(1)` = base DEPOIS do 1.º (a armadilha de autoria).
    let correr = |tracos: usize, quando: Option<usize>| -> f32 {
        let mut mesh = plano_n(48);
        let b = Brush {
            verb: Verb::Cloth,
            cloth_mode: crate::ClothMode::Grab,
            cloth_area: crate::ClothArea::Local,
            radius: 0.35,
            strength: 1.0,
            cloth_persistent: quando.is_some(),
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        for t in 0..tracos {
            if quando == Some(t) {
                s.set_persistent_base(&mesh);
            }
            // ⚠️ O `begin` mata a sessão do traço anterior (`cloth_ref`), que é
            // o que faz cada traço nascer com um pen-down novo.
            s.begin(&mesh);
            for k in 0..12 {
                let c = [0.05 * k as f32, 0.0, 0.0];
                let passo = if k == 0 { [0.0; 3] } else { [0.05, 0.0, 0.0] };
                s.dab(
                    &mut mesh,
                    &b,
                    &dab_em(c, b.radius, passo),
                    Symmetry::default(),
                );
            }
        }
        pico(&mesh)
    };
    let (um, dois, tres) = (correr(1, None), correr(2, None), correr(3, None));
    let (p1, p2, p3) = (correr(1, Some(0)), correr(2, Some(0)), correr(3, Some(0)));
    // (1) SEM base, acumula.
    assert!(
        dois > um * 1.4 && tres > dois * 1.2,
        "sem base tem de ACUMULAR: {um:.4} -> {dois:.4} -> {tres:.4}"
    );
    // (2) COM a base gravada ANTES, satura — e o 1.º traço é o mesmo dos dois
    // (ali a base É o repouso do traço, logo a opção é um no-op exacto).
    assert_eq!(um, p1, "o 1.º traco tem de ser identico com e sem base");
    assert!(
        p2 < p1 * 1.15 && p3 < p1 * 1.25,
        "com base tem de SATURAR: {p1:.4} -> {p2:.4} -> {p3:.4}"
    );
    // (3) ⛔ **A ARMADILHA:** a base gravada DEPOIS do 1.º traço é um no-op —
    // ela é o repouso do 2.º, e as quatro leituras da §6.4 não mudam.
    assert_eq!(
        correr(2, Some(1)),
        dois,
        "gravar a base DEPOIS do 1.º traco tem de ser um NO-OP exacto -- se \
         mudar alguma coisa, a base esta' a ser lida onde a espec diz que nao e'"
    );
}
