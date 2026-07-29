//! **A cena pronta para o smoke da DUREZA** (`PH2D_FLIP_HARDNESS_SMOKE=1`, 03 §8.6).
//!
//! É a foto do Enio encenada: *"Tudo que quero é que tenha o aspecto do traço do nosso próprio
//! módulo painter digital"* (2026-07-28, 4ª rodada, com setas vermelhas sobre cunhas ESCURAS nas
//! quinas de um rabisco que cruza a si mesmo).
//!
//! ⚠️ **A cura foi UMA frase:** o Flip desenha um **TRAÇO**, então o perfil dele tem de ser o
//! perfil de **TRAÇO** do Painter (a fileira de dabs a `spacing × diâmetro` de arco, composta por
//! `over`), nunca o de um **DAB** dele. As duas rodadas anteriores igualaram a lei do dab — que é
//! muito mais RALA — e é isso que abria as cunhas.
//!
//! **Números MEDIDOS** (`ph2d-flip-render/tests/painter_look.rs`, contra o depósito de verdade,
//! numa estrela de um traço só):
//!
//! | | falta de tinta | px fora de 16 |
//! |---|---|---|
//! | lei do DAB (o que a foto mostra) | **−112 de 255** | 613 |
//! | lei do TRAÇO (agora) | **−4** | 166, TODOS de SOBRA |
//!
//! E num traço RETO o Flip virou o depósito do Painter ao **±1 de 255**.
//!
//! ⚠️ **`hardness = 1.0` é byte-idêntico nas duas leis** (disco duro), e é o default do Flip ⇒
//! o X da ESQUERDA é o CONTROLE: se ele mudou, quebrei outra coisa.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use ph2d_vec_scene::Xform;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

/// ⚠️ **A tinta tem de ser VISÍVEL no canvas do Flip, que é CLARO.** Esta cena nasceu com o
/// quase-branco `0.92,0.92,0.95` (copiado de smokes antigos) — sobre papel claro isso desenha
/// **fantasmas**, e julgar "o aspecto do traço" com tinta invisível é impossível. É o mesmo azul
/// que as duas cenas de Flip mais recentes (`airbrush`, `self_overlap`) já usam.
const INK: Rgba = Rgba::new(0.20, 0.55, 0.85, 1.0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_HARDNESS_SMOKE").is_some())
}

/// As durezas encenadas. A 1ª é o CONTROLE (byte-idêntica nas duas leis).
const HARDNESS: [f32; 3] = [1.0, 0.7, 0.4];
/// O `dn` de meia-tinta sob a lei de TRAÇO (a de hoje), medido — a metade VISÍVEL da largura.
const HALF_INK_NOW: [f32; 3] = [1.000, 0.899, 0.824];
/// O mesmo número sob a lei de DAB (a rodada anterior, que o Enio reprovou).
const HALF_INK_WAS: [f32; 3] = [1.000, 0.850, 0.700];

/// **UM cruzamento** — duas retas que se cortam no centro `(cx, 0)`, exatamente a figura da foto.
/// O X é a fixture certa porque é onde o defeito lia pior: a cauda macia de uma passagem sobre o
/// NÚCLEO da outra. Traço GROSSO de propósito: o perfil precisa de várias linhas para se ver.
fn crossing(cx: f32, hardness: f32) -> [FlipStroke; 2] {
    let ink = INK;
    let arm = 0.62_f32;
    let mut out = Vec::with_capacity(2);
    for (dx, dy) in [(arm, arm), (arm, -arm)] {
        let mut s = FlipStroke::new();
        for k in [-1.0_f32, 1.0] {
            s.push_point(Point {
                pos: Vec2::new(cx + k * dx, k * dy),
                width: 0.42,
                opacity: 1.0,
                color: ink,
            });
        }
        s.hardness = hardness;
        out.push(s);
    }
    let mut it = out.into_iter();
    [it.next().expect("perna 1"), it.next().expect("perna 2")]
}

/// **A ESTRELA DE UM TRAÇO** — a figura da foto: quinas de 36° em cada ponta e cinco
/// auto-cruzamentos no miolo, desenhada **sem levantar a caneta**.
///
/// ⚠️ **É esta a cena que faltava.** As rodadas anteriores encenavam cada cruzamento como DOIS
/// traços, e dois traços cruzados nunca tiveram o defeito (o depth deles difere e o mais novo
/// pinta por cima, ou seja **já compõe**). O caso do Enio — um traço só, com quina — exigia
/// desenhar à mão para aparecer.
/// A mesma estrela, com a opção de encená-la como uma **MÃO LENTA**: amostras densas com o
/// tremor que uma mão real carrega.
///
/// ⚠️ **É esta a variante que contém o fenômeno da wave de 2026-07-28.** O RDP tem tolerância
/// `0,05 × espessura = 0,1·r` e a reamostragem só ACRESCENTA pontos, então desenhar devagar
/// entrega passo abaixo de `0,1875·r` — a cerca onde a lista de vizinhos (então capeada por
/// CONTAGEM) truncava e a tinta SUMIA (−184 de 255 em `0,10·r`; −255 em `0,05·r`, medidos
/// contra o depósito real do Painter). A cena anterior só tinha a versão de mão RÁPIDA, que cai
/// do lado seguro da cerca — ela não podia mostrar o defeito nem a cura.
fn one_stroke_star_sampled(cx: f32, hardness: f32, slow_hand: bool) -> FlipStroke {
    let outer = 0.80_f32;
    let mut pontas = Vec::new();
    for k in 0..5 {
        // Passo de 2/5 de volta = a estrela de um traço só.
        let a = -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
        pontas.push(Vec2::new(cx + outer * a.cos(), outer * a.sin()));
    }
    pontas.push(pontas[0]);
    let mut corners = vec![pontas[0]];
    if slow_hand {
        // Amostras densas COM TREMOR — sem o tremor o RDP colapsaria a perna numa reta e a
        // densidade voltaria à da mão rápida (a fixture não conteria o fenômeno).
        for w in pontas.windows(2) {
            let (a, b) = (w[0], w[1]);
            // ⚠️ **A amplitude do tremor tem de PASSAR a tolerância do RDP** (`0,05 × largura`
            // = `0,1·r`), senão o simplificador o apaga e a polilinha volta à densidade da mão
            // rápida — a fixture pareceria conter o fenômeno e não conteria. É o mesmo tremor
            // que uma mão real carrega quando desenha devagar.
            let n = 160usize;
            for k in 1..=n {
                let t = k as f32 / n as f32;
                let h = ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
                let g = ((k as u64).wrapping_mul(40_503) % 977) as f32 / 977.0 - 0.5;
                corners.push(Vec2::new(
                    a.x + (b.x - a.x) * t + h * 0.05,
                    a.y + (b.y - a.y) * t + g * 0.05,
                ));
            }
        }
    } else {
        corners = pontas;
    }

    // ⚠️ **PELO PIPELINE DE VERDADE** (`stroke_from_samples`: smoothing → RDP → reamostragem
    // suave → `build_stroke`), NÃO por `push_point` cru. A versão anterior desta cena empurrava
    // os 6 cantos direto no `FlipStroke`, então ela **pulava exatamente o estágio onde a
    // densidade da polilinha é decidida** — e é a densidade que governa o orçamento de vizinhos
    // (`MAX_RIBBON_EXTRAS`; penhasco MEDIDO em passo `< 0,1875·r`). Um smoke que arma o estado
    // por baixo do pano pula a costura que ele existe para provar.
    let style = ph2d_tool_flip::FlipStyleSnapshot {
        stroke: [51, 140, 217, 255],
        width_px: 0.42 * 100.0,
        hardness,
        ..Default::default()
    };
    let pressures = vec![1.0_f32; corners.len()];
    let mut st =
        crate::flip_draw::stroke_from_samples(&style, &corners, &pressures, &Xform::IDENTITY);
    st.hardness = hardness;
    st
}

/// O passo MÍNIMO da polilinha que o pipeline entregou, em raios — o número que diz de que lado
/// da (hoje extinta) cerca de `0,1875·r` a figura caiu. É o MÍNIMO, não a média: a média de uma
/// estrela é dominada pelas retas longas e escondia justamente os trechos densos.
fn min_step_in_radii(st: &FlipStroke) -> f32 {
    let r = 0.42_f32 * 0.5;
    (1..st.len())
        .filter_map(|i| Some((st.point(i)?.pos - st.point(i - 1)?.pos).length() / r))
        .fold(f32::MAX, f32::min)
}

/// **Monta a cena** — porta única (a mensagem encena por aqui). Devolve os x dos QUATRO grupos.
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> [f32; 4] {
    obj.fps = 12.0;
    obj.onion.enabled = false; // um quadro só; o onion sujaria a leitura do perfil.

    let xs = [-2.6_f32, -0.9, 0.8, 2.5];
    let layer = obj.add_layer("Hardness");
    if let Some(d) = obj.insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe) {
        let strokes = &mut obj.drawing_mut(d).expect("desenho").strokes;
        // Dois X de DOIS traços: o controle duro e o macio.
        strokes.extend(crossing(xs[0], HARDNESS[0]));
        strokes.extend(crossing(xs[1], HARDNESS[2]));
        // E as DUAS estrelas de UM traço: mão RÁPIDA e mão LENTA.
        //
        // ⚠️ A DENSIDADE é o que governava o orçamento de vizinhos, e é por isso que as duas
        // estão na cena: a rápida cai do lado seguro da (hoje extinta) cerca de `0,1875·r`, a
        // lenta cai do lado que perdia tinta. Imprimir o passo MÍNIMO real é a única forma de
        // saber de que lado cada uma caiu — a média escondia (é dominada pelas retas longas).
        for (i, slow) in [(2usize, false), (3, true)].into_iter() {
            let star = one_stroke_star_sampled(xs[i], HARDNESS[2], slow);
            eprintln!(
                "[hardness-smoke] estrela {} pelo pipeline REAL: {} pontos, passo MINIMO \
                 {:.3} x raio (a cerca do orcamento antigo ficava em 0.1875)",
                if slow { "MAO LENTA" } else { "mao rapida" },
                star.len(),
                min_step_in_radii(&star)
            );
            strokes.push(star);
        }
    }
    xs
}

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_hardness_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let tool_ok = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Hardness Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recem-criado");
        let xs = stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[hardness-smoke] cena montada: 2 cruzamentos + 2 ESTRELAS DE UM TRACO em \
             x={:?}, hardness {:?}. Ferramenta flip ativa: {}.",
            xs,
            [HARDNESS[0], HARDNESS[2], HARDNESS[2], HARDNESS[2]],
            if tool_ok {
                "sim"
            } else {
                "NAO (PARE: sem ela o traco nao e dirigido pela tool Flip)"
            }
        );

        let mut tabela = String::new();
        for i in [0usize, 1, 2] {
            tabela.push_str(&format!(
                "                 {:>4.1}         {:>5.3}          {:>5.3}\n",
                HARDNESS[i], HALF_INK_WAS[i], HALF_INK_NOW[i]
            ));
        }

        eprintln!(
            "\n\
             ============================================================\n\
             ANTES DE TUDO: este terminal imprimiu, logo acima, a linha\n\
             comecando com '[hardness-smoke] cena montada'? Se NAO, PARE:\n\
             o smoke nao rodou (arvore ou variavel de ambiente errada).\n\
             ============================================================\n\
             \n\
             A CENA, da esquerda para a direita:\n\
               1. X duro (hardness 1.0) -- o CONTROLE. As duas leis sao\n\
                  byte-identicas aqui, e este e o default do Flip. Se ele\n\
                  mudou, algo mais quebrou.\n\
               2. X macio (hardness 0.4), DOIS tracos cruzados.\n\
               3. ESTRELA de UM traco, MAO RAPIDA (hardness 0.4).\n\
               4. ESTRELA de UM traco, MAO LENTA -- **A SUA FOTO**.\n\
                  Mesma figura, amostrada densa com tremor, que e o que\n\
                  acontece quando voce desenha devagar. Veja no terminal\n\
                  acima o passo MINIMO de cada uma.\n\
             \n\
             O QUE OLHAR -- e e so isso:\n\
               1. **AS DUAS ESTRELAS TEM DE SER A MESMA FIGURA.** Era\n\
                  exatamente aqui que o defeito vivia: a da direita\n\
                  (mao lenta) PERDIA tinta nas quinas e nos cruzamentos,\n\
                  e o buraco lia como uma dobra 3D. Se as duas estao\n\
                  iguais, a wave fez o que prometeu.\n\
               2. Desenhe voce mesmo, DEVAGAR, cruzando o proprio traco\n\
                  sem levantar a caneta -- o gesto do seu report.\n\
               3. Abra o PAINTER, pincel digital normal, MESMA hardness, e\n\
                  rabisque uma estrela sem levantar a caneta. O aspecto\n\
                  tem de ser o MESMO -- e a razao desta wave existir.\n\
               4. O X da esquerda nao pode ter mudado.\n\
             \n\
             ------------------------------------------------------------\n\
             A CURA DESTA RODADA, numa frase: a lista de vizinhos que o\n\
             fragment recebe era capeada por CONTAGEM (16 segmentos), mas\n\
             o que ela precisa cobrir e um ALCANCE (3 x raio). Contagem =\n\
             alcance / passo, entao desenhar DEVAGAR atravessava o teto,\n\
             a lista truncava e a tinta SUMIA -- medido contra o deposito\n\
             real do Painter: -184 de 255 com passo 0,10 x raio e -255\n\
             (tinta NENHUMA) com 0,05. Agora a lista conta CAPSULAS, e\n\
             uma capsula cobre um PEDACO DE CAMINHO: 1 numa reta, ~6 numa\n\
             curva do tamanho do pincel -- em qualquer densidade. O\n\
             desvio virou CONSTANTE (-3 de 255) de 0,80 ate 0,04 x raio.\n\
             \n\
             ------------------------------------------------------------\n\
             A CURA DA RODADA ANTERIOR, numa frase: o Flip desenha um\n\
             TRACO, entao o perfil\n\
             dele e o perfil de TRACO do Painter (a fileira de dabs\n\
             composta por `over`), nunca o de um DAB dele. As duas\n\
             rodadas anteriores igualaram a lei do DAB, que e muito mais\n\
             rala -- em hardness 0.4 e dn 0.70 um dab pesa 0.500 e o\n\
             traco pesa 0.916.\n\
             \n\
             Medido: o dn onde a tinta cruza meia-tinta (= a metade\n\
             VISIVEL da largura pedida).\n\
             \n\
             hardness      lei do DAB      lei do TRACO\n\
             {tabela}\
             ------------------------------------------------------------\n\
             \n\
             Contra o deposito REAL do Painter (sonda `painter_look.rs`,\n\
             a mesma estrela): ZERO pixel com MENOS tinta que o Painter,\n\
             em toda a faixa de hardness, e num traco RETO o Flip virou\n\
             o deposito dele ao +-1 de 255.\n\
             \n\
             ⚠️ RESIDUO NOMEADO: na PONTA de uma quina muito afiada os\n\
             dabs do Painter RECUAM em vez de correr paralelos, e o Flip\n\
             pinta ali um pouco mais cheio (+122 de 255 no vertice de\n\
             36 graus; some conforme a hardness sobe). E a direcao\n\
             OPOSTA a queixa -- a ponta fica mais redonda, nao mordida.\n\
             Se ISSO incomodar, reporte: e outra wave.\n"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{min_step_in_radii, one_stroke_star_sampled};

    /// 🔴 **A CENA CONTÉM O FENÔMENO — e isso é conferido, não afirmado.**
    ///
    /// A estrela de mão RÁPIDA cai do lado seguro da cerca que o orçamento de vizinhos tinha
    /// (`3·r / 16 = 0,1875·r`); a de mão LENTA cai do lado que perdia tinta. Sem as duas, o
    /// smoke não pode mostrar nem o defeito nem a cura — foi exatamente por isso que quatro
    /// rodadas de smoke passaram por cima do defeito reportado.
    ///
    /// ⚠️ O gate vigia a FIXTURE, não o produto: se um dia a reamostragem mudar e a estrela
    /// lenta subir acima da cerca, esta cena para de provar o que a mensagem dela promete — e
    /// é melhor ficar vermelho aqui do que verde numa tela que não contém o caso.
    const OLD_FENCE: f32 = 0.1875;

    #[test]
    fn the_slow_hand_star_is_denser_than_the_old_neighbour_fence() {
        let rapida = min_step_in_radii(&one_stroke_star_sampled(0.0, 0.4, false));
        let lenta = min_step_in_radii(&one_stroke_star_sampled(0.0, 0.4, true));
        println!("  passo MINIMO: mao rapida {rapida:.3} x raio | mao lenta {lenta:.3} x raio");
        assert!(
            lenta < OLD_FENCE,
            "a estrela de MAO LENTA tem de cair abaixo da cerca ({lenta:.3} x raio): sem isso a \
             cena não contém o defeito que ela existe para mostrar"
        );
        assert!(
            rapida > OLD_FENCE,
            "a de mão RÁPIDA tem de ficar ACIMA ({rapida:.3} x raio): ela é o controle, e duas \
             fixtures do mesmo lado da cerca não comparam nada"
        );
    }
}
