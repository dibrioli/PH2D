//! ⭐⭐⭐⭐ **AS SONDAS DA TINTA FINA** — as medições que produziram as leis do
//! irmão [`super::tinta_no_produto_tests`], cortadas dele quando o par cruzou o
//! tecto de LOC.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e não por tamanho:** um gate AFIRMA e
//! uma sonda MEDE. As cinco daqui imprimem tabelas e não têm barra nenhuma — é
//! delas que saíram o mapa em ilhas do report de 21/09, o A/B que mostrou a
//! máscara inerte para a cor fina, e o `1010 → 1010` que mede o `Ctrl+Z` a não
//! desfazer a tinta.
//!
//! ⛔ **Elas ficam VERSIONADAS de propósito:** cada uma é a régua que decidiu
//! uma cura, e a próxima janela que duvidar de um número corre-a em vez de o
//! re-derivar. *Uma medição apagada é uma medição que se volta a pagar.*

use ph2d_sculpt3d::Verb;

use super::{amostras, cena_52, gesto, topologia, traco};
use crate::Sculpt3dScene;

/// Sonda: ONDE mora o plano em cada passo de dois traços de cor.
#[test]
#[ignore = "precisa de adaptador"]
fn diag_onde_mora_o_plano_entre_dois_tracos() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    let (on, _) = s.toggle_dyntopo();
    eprintln!("[diag] interruptor armado = {on}");
    s.sync_mesh(&gpu.device, &gpu.queue);
    fn onde(s: &Sculpt3dScene, quando: &str) {
        eprintln!(
            "[diag] {quando:<28} peca={:?} traco={:?} nivel={:?} topo={:?}",
            s.objects[s.active]
                .tinta
                .as_ref()
                .map(|t| t.amostras().len()),
            s.stroke.tinta_fina.as_ref().map(|t| t.tocadas().len()),
            s.tinta_nivel,
            topologia(s),
        );
    }
    onde(&s, "inicio");
    let a = traco(&mut s, 400.0);
    onde(&s, "A: pen-up, pre-sync");
    s.sync_mesh(&gpu.device, &gpu.queue);
    onde(&s, "A: pos-sync");
    let b = traco(&mut s, 250.0);
    onde(&s, "B: pen-up, pre-sync");
    s.sync_mesh(&gpu.device, &gpu.queue);
    onde(&s, "B: pos-sync");
    eprintln!("[diag] pen-down aceite: A={a} B={b}");
}

/// Sonda do report de 21/09: *«se comecar a pintar sem tocar um vertex acontece
/// mais vezes de sumir a pintura»*.
#[test]
#[ignore = "precisa de adaptador"]
fn diag_o_gesto_que_comeca_fora_da_peca() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let nasceu = amostras(&s).len();
    fn conta(s: &Sculpt3dScene) -> usize {
        amostras(s)
            .iter()
            .filter(|c| **c != [1.0, 1.0, 1.0])
            .count()
    }
    let estado = |s: &Sculpt3dScene, q: &str| {
        eprintln!(
            "[diag] {q:<34} plano={:<7?} traco={:<8?} pintadas={:<6} drag_aberto={}",
            amostras(s).len(),
            s.stroke.tinta_fina.as_ref().map(|t| t.tocadas().len()),
            conta(s),
            s.drag.is_some(),
        );
    };
    eprintln!("[diag] o plano nasceu com {nasceu} amostras; a peca ocupa x em [280, 610]");
    estado(&s, "inicio");

    // (A) um traco NORMAL, todo dentro da peca.
    gesto(&mut s, 380.0, 460.0, 10);
    s.sync_mesh(&gpu.device, &gpu.queue);
    estado(&s, "A: dentro da peca");

    // (B) o gesto do report: COMECA FORA e entra na peca.
    gesto(&mut s, 150.0, 480.0, 20);
    s.sync_mesh(&gpu.device, &gpu.queue);
    estado(&s, "B: comecou FORA, entrou");

    // (C) e um traco normal outra vez, para ver o que sobrou.
    gesto(&mut s, 380.0, 460.0, 10);
    s.sync_mesh(&gpu.device, &gpu.queue);
    estado(&s, "C: dentro outra vez");
}

/// ⛔⛔ **SONDA do 3.º report de 21/09:** *«a tinta só é depositada se o pincel
/// está sobre um vertex»*.
///
/// Ela varre o ecrã de 5 em 5 píxeis com UM clique em cada sítio e imprime um
/// `#` onde alguma amostra foi pintada e um `.` onde nada caiu — com quatro
/// raios de pincel, do mais fino ao mais gordo. *A régua é a AMOSTRA, que é a
/// unidade que a tinta fina existe para dar; contar vértices aqui mediria
/// exactamente a grandeza que o report acusa.*
#[test]
#[ignore = "precisa de adaptador"]
fn diag_a_tinta_so_cai_onde_ha_vertice() {
    let gpu = gpu_or_skip!();
    {
        let s = cena_52(&gpu.device);
        let m = s.mesh();
        let mut soma = 0.0f32;
        let mut n = 0usize;
        for f in m.faces() {
            let vs = f.verts();
            for k in 0..vs.len() {
                let a = m.positions()[vs[k] as usize];
                let b = m.positions()[vs[(k + 1) % vs.len()] as usize];
                let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                soma += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                n += 1;
            }
        }
        eprintln!(
            "[diag] a peca tem {} vertices e {} faces; aresta media {:.4}",
            m.vert_count(),
            m.faces().len(),
            soma / n as f32
        );
    }
    for raio in [6.0f32, 12.0, 24.0, 48.0] {
        let mut s = cena_52(&gpu.device);
        s.sync_mesh(&gpu.device, &gpu.queue);
        s.radius_px = raio;
        let pintadas = |s: &Sculpt3dScene| {
            amostras(s)
                .iter()
                .filter(|c| **c != [1.0, 1.0, 1.0])
                .count()
        };
        let mut mapa = String::new();
        let mut acertos = 0;
        let mut sitios = 0;
        let mut antes = pintadas(&s);
        let mut x = 300.0f32;
        while x <= 600.0 {
            gesto(&mut s, x, x, 1);
            s.sync_mesh(&gpu.device, &gpu.queue);
            let agora = pintadas(&s);
            sitios += 1;
            if agora > antes {
                acertos += 1;
                mapa.push('#');
            } else {
                mapa.push('.');
            }
            antes = agora;
            x += 5.0;
        }
        eprintln!(
            "[diag] raio {raio:>4.0} px  mundo {:.4}  pintou em {acertos}/{sitios}  {mapa}",
            s.brush.radius
        );
    }
}

/// ⛔ **SONDA: o `Ctrl+Z` desfaz a tinta FINA?** — a pergunta que decide se a
/// cura do report de 21/09 precisa de uma segunda metade (a entrada de desfazer
/// é escrita a partir da janela de VÉRTICES tocados, e um dab que só toca
/// AMOSTRAS deixa essa janela vazia).
#[test]
#[ignore = "precisa de adaptador"]
fn diag_o_ctrl_z_desfaz_a_tinta_fina() {
    use winit::keyboard::KeyCode as K;
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let pintadas = |s: &Sculpt3dScene| {
        amostras(s)
            .iter()
            .filter(|c| **c != [1.0, 1.0, 1.0])
            .count()
    };
    eprintln!("[diag] antes do traco: {} amostras pintadas", pintadas(&s));
    assert!(
        gesto(&mut s, 380.0, 470.0, 10),
        "o pen-down nao foi da cena"
    );
    s.sync_mesh(&gpu.device, &gpu.queue);
    let depois = pintadas(&s);
    eprintln!("[diag] depois do traco: {depois} amostras pintadas");

    let mut req = crate::Sculpt3dRequests::default();
    let factos = crate::keys::keys_delete::DeleteFacts {
        clay_on_screen: true,
        text_focused: false,
        over_panel: false,
        vector_has_selection: false,
    };
    let consumiu = crate::keys::key(
        &mut s,
        &mut req,
        None,
        crate::keys::KeyPress {
            code: K::KeyZ,
            ctrl: true,
            shift: false,
        },
        &factos,
        "",
    );
    s.sync_mesh(&gpu.device, &gpu.queue);
    eprintln!(
        "[diag] Ctrl+Z consumido={consumiu}; depois do desfazer: {} amostras pintadas",
        pintadas(&s)
    );
}

/// ⛔⛔ **SONDA: a máscara de alcance ainda DECIDE alguma coisa na tinta fina?**
///
/// A máscara (`Connected Only`) corta VÉRTICES e a tinta fina escreve
/// AMOSTRAS. O A/B é o que responde sem inventar uma porta: o MESMO traço
/// sobre a MESMA parede fina, com a máscara armada e desarmada. Contagens
/// iguais ⇒ *ela já é inerte para a cor fina*, e transplantar a lei dela para
/// a amostra é uma CURA; contagens diferentes ⇒ ela decide, e abrir a cerca
/// dela seria mudar em silêncio um caminho aprovado.
#[test]
#[ignore = "precisa de adaptador"]
fn diag_a_mascara_ainda_decide_na_tinta_fina() {
    let gpu = gpu_or_skip!();
    let mut linha = Vec::new();
    for (rotulo, mascara) in [("armada", true), ("desarmada", false)] {
        for raio in [64.0f32, 24.0, 10.0] {
            let mut s =
                Sculpt3dScene::new(&gpu.device, crate::scenes::parede_fina::barbatana(), 1.0);
            s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
            ph2d_panel_sculpt3d::state::switch_verb_parts(
                &mut s.verb_slots,
                &mut s.brush,
                &mut s.radius_px,
                Verb::Paint,
            );
            s.brush.color = [1.0, 0.0, 0.0];
            s.brush.surface_only = mascara;
            s.tinta_nivel = crate::scenes::tinta_fina::DEGRAU_DA_LICAO.nivel();
            s.radius_px = raio;
            s.sync_mesh(&gpu.device, &gpu.queue);
            gesto(&mut s, 420.0, 470.0, 8);
            s.sync_mesh(&gpu.device, &gpu.queue);
            let n = amostras(&s)
                .iter()
                .filter(|c| **c != [1.0, 1.0, 1.0])
                .count();
            linha.push(format!("{rotulo} r={raio:.0}: {n}"));
        }
    }
    eprintln!("[diag] amostras pintadas -> {}", linha.join("  |  "));
}
