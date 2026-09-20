//! Os gates da lei do canvas — a tabela é a do ORÁCULO, e está no doc da crate.
//!
//! ⚠️ **A janela do oráculo é `720×450`**, logo a vista equivalente tem meia-janela `[360, 225]`.
//! Os números da direita saíram de `godot_hud_probe.gd` (bloco L2) e estão aqui **verbatim**.

// ⚠️ **Os zeros à direita são a precisão IMPRESSA pelo oráculo** (`%.6f`), e os números estão
// aqui verbatim de propósito: é o que torna a tabela conferível contra a saída da sonda sem
// ninguém ter de a reformatar de cabeça. O clippy quer `2.25`; nós queremos `2.250_000`.
#![allow(clippy::excessive_precision, reason = "a tabela é verbatim do oráculo")]

use super::{Canvas, Fit, View, bands, effective_box, place};

/// A vista que reproduz a janela do oráculo.
fn vista_do_oraculo() -> View {
    View {
        center: [0.0, 0.0],
        half: [360.0, 225.0],
    }
}

/// `ignore` — um factor por eixo, e **nenhum** deles é o do outro.
#[test]
fn a_lei_do_stretch_e_um_factor_por_eixo() {
    // (ref_w, ref_h, escala_x, escala_y) — verbatim do oráculo, `aspect=ignore`.
    const ORACULO: [(f32, f32, f32, f32); 5] = [
        (320.0, 180.0, 2.250_000, 2.500_000),
        (640.0, 360.0, 1.125_000, 1.250_000),
        (1280.0, 360.0, 0.562_500, 1.250_000),
        (320.0, 480.0, 2.250_000, 0.937_500),
        (500.0, 400.0, 1.440_000, 1.125_000),
    ];
    for (rw, rh, ex, ey) in ORACULO {
        let c = Canvas::new(rw, rh, Fit::Stretch).expect("caixa válida");
        let p = place(&c, vista_do_oraculo());
        assert!(
            (p.scale[0] - ex).abs() <= 1e-6 && (p.scale[1] - ey).abs() <= 1e-6,
            "ref {rw}×{rh}: nós {:?}, o oráculo ({ex}, {ey})",
            p.scale
        );
    }
}

/// `keep` — uniforme, e a BANDA é a que o oráculo imprime **menos o arredondamento dele**.
///
/// ⛔ As duas últimas linhas são a **divergência declarada**: ali o alvo arredonda a banda a
/// inteiro e encolhe a escala para caber, e nós ficamos com o valor exacto.
#[test]
fn a_lei_do_keep_e_a_do_oraculo_menos_o_arredondamento() {
    // (ref_w, ref_h, escala_do_oraculo_x, banda_do_oraculo_x, banda_do_oraculo_y)
    const ORACULO: [(f32, f32, f32, f32, f32); 5] = [
        (320.0, 180.0, 2.250_000, 0.0, 23.0),
        (640.0, 360.0, 1.125_000, 0.0, 23.0),
        (1280.0, 360.0, 0.562_500, 0.0, 124.0),
        (320.0, 480.0, 0.937_500, 210.0, 0.0),
        (500.0, 400.0, 1.124_000, 79.0, 0.0),
    ];
    for (rw, rh, es, bx, by) in ORACULO {
        let c = Canvas::new(rw, rh, Fit::Keep).expect("caixa válida");
        let v = vista_do_oraculo();
        let p = place(&c, v);
        assert!(
            (p.scale[0] - p.scale[1]).abs() <= f32::EPSILON,
            "ref {rw}×{rh}: o keep NÃO pode distorcer, e deu {:?}",
            p.scale
        );
        let esperado = (2.0 * v.half[0] / rw).min(2.0 * v.half[1] / rh);
        assert!(
            (p.scale[0] - esperado).abs() <= 1e-6,
            "ref {rw}×{rh}: a escala é o MÍNIMO dos dois factores"
        );
        // A escala do oráculo é a nossa a menos do arredondamento dele (< 1 px de banda).
        assert!(
            (p.scale[0] - es).abs() <= 3e-3,
            "ref {rw}×{rh}: nós {:.6}, o oráculo {es:.6}",
            p.scale[0]
        );
        let b = bands(&c, v);
        assert!(
            (b[0] - bx).abs() <= 1.0 && (b[1] - by).abs() <= 1.0,
            "ref {rw}×{rh}: bandas nossas {b:?}, do oráculo ({bx}, {by})"
        );
    }
}

/// ⛔ A DIVERGÊNCIA tem de EXISTIR, senão o tecto acima vira licença.
///
/// Nas duas referências em que o alvo arredonda, o número dele **não** é o exacto — e é isso que
/// esta metade afirma. *Um gate que tolera uma diferença sem nunca a exigir deixa de a descrever.*
#[test]
fn onde_o_alvo_arredonda_nos_ficamos_com_o_exacto() {
    let v = vista_do_oraculo();
    // ref 1280×360: banda exacta 123,75 — o alvo imprimiu 124.
    let c = Canvas::new(1280.0, 360.0, Fit::Keep).expect("caixa válida");
    let b = bands(&c, v);
    assert!(
        (b[1] - 123.75).abs() <= 1e-4,
        "a banda exacta é 123,75 e deu {}",
        b[1]
    );
    assert!(
        (b[1] - 124.0).abs() > 1e-3,
        "se a nossa banda fosse a arredondada do alvo, a divergência não existia"
    );
    // ref 500×400: banda exacta 78,75 — o alvo imprimiu 79 e encolheu a escala para 1,124.
    let c = Canvas::new(500.0, 400.0, Fit::Keep).expect("caixa válida");
    assert!((bands(&c, v)[0] - 78.75).abs() <= 1e-4);
    assert!(
        (place(&c, v).scale[0] - 1.125).abs() <= 1e-6,
        "a nossa escala é o mínimo exacto (1,125), não o 1,124 que o arredondamento dele produz"
    );
}

/// A translação é o centro da vista — nos dois modos, e com a vista fora da origem.
#[test]
fn a_translacao_e_sempre_o_centro_da_vista() {
    let v = View {
        center: [12.5, -7.25],
        half: [16.0, 9.0],
    };
    for fit in [Fit::Keep, Fit::Stretch] {
        let c = Canvas::new(32.0, 18.0, fit).expect("caixa válida");
        assert_eq!(place(&c, v).translate, [12.5, -7.25], "modo {fit:?}");
    }
}

/// O neutro é EXACTO: caixa do tamanho da vista ⇒ escala `1` e bandas `0`, por divisão de iguais.
#[test]
fn o_neutro_e_exacto_nos_dois_modos() {
    let v = View {
        center: [0.0, 0.0],
        half: [16.0, 9.0],
    };
    for fit in [Fit::Keep, Fit::Stretch] {
        let c = Canvas::new(32.0, 18.0, fit).expect("caixa válida");
        let p = place(&c, v);
        assert_eq!(p.scale, [1.0, 1.0], "modo {fit:?}");
        assert_eq!(bands(&c, v), [0.0, 0.0], "modo {fit:?}");
    }
}

/// O CONTROLO dos dois modos: com aspectos diferentes, um distorce e o outro não.
///
/// ⚠️ Sem esta metade, um `Stretch` que por engano calculasse o mínimo passaria em tudo o que está
/// acima — as tabelas do oráculo têm linhas em que os dois modos coincidem.
#[test]
fn o_stretch_distorce_onde_o_keep_nao_distorce() {
    // ⚠️ meia-janela `[48, 9]` e não `[32, 9]`: com esta a diferença dá `2,0` e com aquela dá
    // EXACTAMENTE `1,0`, que era a barra — *uma fixtura que aterra em cima da barra não a testa.*
    let v = View {
        center: [0.0, 0.0],
        half: [48.0, 9.0],
    };
    let k = place(&Canvas::new(32.0, 18.0, Fit::Keep).expect("k"), v);
    let s = place(&Canvas::new(32.0, 18.0, Fit::Stretch).expect("s"), v);
    assert!(
        (k.scale[0] - k.scale[1]).abs() <= f32::EPSILON,
        "o keep não distorce"
    );
    assert!(
        (s.scale[0] - s.scale[1]).abs() > 1.0,
        "o stretch TEM de distorcer aqui, e deu {:?}",
        s.scale
    );
}

/// Uma caixa que não é um rectângulo utilizável é RECUSADA — e a recusa é a lei.
#[test]
fn uma_caixa_impossivel_e_recusada() {
    for (w, h) in [
        (0.0, 18.0),
        (32.0, 0.0),
        (-1.0, 18.0),
        (f32::NAN, 18.0),
        (32.0, f32::INFINITY),
    ] {
        assert!(
            Canvas::new(w, h, Fit::Keep).is_none(),
            "caixa {w}×{h} tinha de ser recusada"
        );
    }
    assert!(
        Canvas::new(32.0, 18.0, Fit::Keep).is_some(),
        "o CONTROLO positivo"
    );
}

/// O texto de um número: a contagem crua, e o tempo com uma casa e **nunca negativo**.
#[test]
fn o_tempo_que_ja_passou_mostra_zero_e_nunca_um_negativo() {
    use super::{Valor, formata};
    assert_eq!(formata(Valor::Inteiro(0)), "0");
    assert_eq!(
        formata(Valor::Inteiro(-3)),
        "-3",
        "uma CONTAGEM pode ser negativa (dívida, vidas a menos)"
    );
    assert_eq!(formata(Valor::Segundos(3.25)), "3.2", "uma casa decimal");
    assert_eq!(formata(Valor::Segundos(0.0)), "0.0");
    // ⭐ o caso que a lei existe para cobrir: o relógio passou do fim.
    assert_eq!(formata(Valor::Segundos(-1.3)), "0.0");
}

/// As QUATRO células do bloco L3 do oráculo, uma a uma.
#[test]
fn um_botao_dispara_ao_largar_e_so_se_os_dois_toques_forem_dentro() {
    use super::{Gesto, clique};

    // (a) carregar DENTRO e largar DENTRO ⇒ publica, UMA vez.
    let (mem, pub_) = clique(Gesto::Baixo, None, Some(7));
    assert_eq!((mem, pub_), (Some(7), false), "o carregar nunca publica");
    let (mem, pub_) = clique(Gesto::Cima, mem, Some(7));
    assert_eq!((mem, pub_), (None, true));
    // e a memória ficou limpa ⇒ um segundo largar não repete.
    assert_eq!(clique(Gesto::Cima, mem, Some(7)), (None, false));

    // (b) carregar DENTRO, largar FORA ⇒ não publica.
    let (mem, _) = clique(Gesto::Baixo, None, Some(7));
    assert_eq!(clique(Gesto::Cima, mem, None), (None, false));

    // (c) carregar FORA, largar DENTRO ⇒ não publica.
    let (mem, _) = clique(Gesto::Baixo, None, None::<u8>);
    assert_eq!(clique(Gesto::Cima, mem, Some(7)), (None, false));

    // (d) carregar num botão e largar noutro ⇒ não publica.
    let (mem, _) = clique(Gesto::Baixo, None, Some(7));
    assert_eq!(clique(Gesto::Cima, mem, Some(9)), (None, false));

    // ⚠️ E um `Baixo` fora LIMPA a memória — senão o largar seguinte publicaria um botão em que o
    // dedo nunca pousou.
    let (mem, _) = clique(Gesto::Baixo, Some(7), None);
    assert_eq!(mem, None);
}

// ─────────────────────────────────────────────────────────────────────────────
// A CAIXA EFECTIVA — a grandeza que a âncora precisava e não tinha
// ─────────────────────────────────────────────────────────────────────────────

/// A referência de fábrica desta casa.
fn canvas16x9(fit: Fit) -> Canvas {
    Canvas::new(32.0, 18.0, fit).expect("canvas")
}

fn vista(hw: f32, hh: f32) -> View {
    View {
        center: [0.0, 0.0],
        half: [hw, hh],
    }
}

/// ⭐⭐ **O NEUTRO é EXACTO** — no aspecto da própria caixa, a efectiva **É** a de referência, ao
/// bit.
///
/// ⚠️ É esta metade que torna a porta barata: com ela, um filho ancorado desenha **byte-idêntico**
/// ao que desenhava antes de a porta existir. Sem ela, toda cena de HUD já autorada mudaria de
/// imagem no dia em que isto shipasse.
///
/// **Mutação que deve sangrar:** somar um epsilon a `hx`.
#[test]
fn no_aspecto_da_caixa_a_efectiva_e_a_de_referencia_ao_bit() {
    let c = canvas16x9(Fit::Expand);
    // 16:9 exacto, em três tamanhos — a escala muda, a caixa LOCAL não.
    for (hw, hh) in [(16.0, 9.0), (32.0, 18.0), (8.0, 4.5)] {
        assert_eq!(
            effective_box(&c, vista(hw, hh)),
            [-16.0, -9.0, 16.0, 9.0],
            "a efectiva mexeu-se num aspecto SEM banda ({hw}x{hh})"
        );
    }
}

/// ⭐ **A banda entra em unidades LOCAIS** — e a prova é que a caixa cresce do valor que o
/// `bands` devolve, dividido pela escala da raiz.
///
/// ⚠️ A asserção é uma IDENTIDADE entre as duas portas e não um número escrito à mão: um número
/// pinaria a aritmética; a identidade afirma que as duas respondem sobre a mesma banda.
///
/// **Mutação que deve sangrar:** tirar a divisão pela escala.
#[test]
fn a_banda_atravessa_a_escala_para_virar_local() {
    let c = canvas16x9(Fit::Expand);
    for (hw, hh) in [(21.0, 9.0), (16.0, 12.0), (40.0, 9.0), (5.0, 9.0)] {
        let v = vista(hw, hh);
        let b = bands(&c, v);
        let p = place(&c, v);
        let e = effective_box(&c, v);
        let esperado_hx = 16.0 + b[0] / p.scale[0];
        let esperado_hy = 9.0 + b[1] / p.scale[1];
        assert!(
            (e[2] - esperado_hx).abs() < 1.0e-5 && (e[3] - esperado_hy).abs() < 1.0e-5,
            "{hw}x{hh}: efectiva {e:?} nao e' a de referencia mais a banda local"
        );
    }
}

/// ⭐⭐⭐ **A caixa efectiva CHEGA à borda real da vista** — que é a frase inteira desta porta.
///
/// ⚠️ A régua é o canto levado ao MUNDO pela pose da raiz, comparado com a meia-extensão da vista:
/// é o que o artista vê, e não a aritmética interna.
///
/// **Mutação que deve sangrar:** usar `ref_w / 2.0` em vez da efectiva.
#[test]
fn o_canto_da_efectiva_pousa_na_borda_da_vista() {
    for fit in [Fit::Expand, Fit::Stretch] {
        let c = canvas16x9(fit);
        for (hw, hh) in [(21.0, 9.0), (16.0, 12.0), (16.0, 9.0), (40.0, 4.0)] {
            let v = vista(hw, hh);
            let p = place(&c, v);
            let e = effective_box(&c, v);
            let canto_x = e[2] * p.scale[0] + p.translate[0];
            let canto_y = e[3] * p.scale[1] + p.translate[1];
            assert!(
                (canto_x - v.half[0]).abs() < 1.0e-4 && (canto_y - v.half[1]).abs() < 1.0e-4,
                "{fit:?} {hw}x{hh}: o canto pousou em ({canto_x}, {canto_y}) e a borda e' {:?}",
                v.half
            );
        }
    }
}

/// ⛔ **Com `Stretch` ela é a de referência ao bit** — e isso é a LEI, não um caso por cobrir: ali
/// a caixa já preenche a vista nos dois eixos, logo não há banda nenhuma para crescer.
///
/// ⚠️ Metade NEGATIVA: sem ela, uma implementação que crescesse a caixa nos dois modos passaria o
/// gate de cima (o canto continuaria a pousar na borda) e daria ao `Stretch` **o dobro** do
/// alcance que ele tem.
#[test]
fn com_stretch_a_efectiva_e_a_de_referencia() {
    let c = canvas16x9(Fit::Stretch);
    for (hw, hh) in [(21.0, 9.0), (16.0, 12.0), (16.0, 9.0)] {
        assert_eq!(
            effective_box(&c, vista(hw, hh)),
            [-16.0, -9.0, 16.0, 9.0],
            "o Stretch cresceu a caixa, e ali nao ha' banda"
        );
    }
}

/// ⛔ **Uma vista degenerada não devolve `inf` nem `NaN`.**
///
/// ⚠️ Um `NaN` aqui não fica aqui: ele viaja até à pose de **todo** filho ancorado, e uma pose
/// `NaN` desenha-se como o objecto a desaparecer — o modo de falha mais caro de diagnosticar.
#[test]
fn uma_vista_degenerada_nao_devolve_infinito() {
    let c = canvas16x9(Fit::Expand);
    for (hw, hh) in [(0.0, 0.0), (0.0, 9.0), (16.0, 0.0)] {
        let e = effective_box(&c, vista(hw, hh));
        assert!(e.iter().all(|v| v.is_finite()), "{hw}x{hh} devolveu {e:?}");
    }
}

/// ⛔⛔⛔ **O `Keep` CONFINA — a caixa efectiva dele é a de REFERÊNCIA, em toda janela.**
///
/// ⚠️⚠️ **Este gate nasceu de um defeito que shipou:** a 1.ª redacção do `effective_box` crescia a
/// caixa no `Keep`, e isso fazia o `Keep` comportar-se como o **`expand`** do alvo — uma
/// divergência **silenciosa** que retirava a capacidade de confinar o HUD à área segura.
///
/// O oráculo (Godot 4.7.2 MIT, bloco L4, janela `720×450`) é quem decide, e não por pouco: com
/// `keep` um filho preso ao canto lê a caixa `(640, 360)` — **a referência** — e o canto dele
/// aterra a `22 px` da borda; com `expand` ele lê `(640, 400)` e aterra **na** borda.
///
/// **Mutação que deve sangrar:** tirar a guarda `if canvas.fit != Fit::Expand`.
#[test]
fn o_keep_confina_e_a_efectiva_dele_e_a_de_referencia() {
    let c = canvas16x9(Fit::Keep);
    for (hw, hh) in [(21.0, 9.0), (16.0, 12.0), (16.0, 9.0), (40.0, 4.0)] {
        assert_eq!(
            effective_box(&c, vista(hw, hh)),
            [-16.0, -9.0, 16.0, 9.0],
            "o `Keep` cresceu a caixa em {hw}x{hh} — ele passa a ser o `expand` do alvo, e o HUD \
             deixa de poder ficar na area segura"
        );
    }
}

/// ⭐⭐ **A POSE do `Expand` é a do `Keep`, AO BIT** — a diferença entre os dois é só até onde uma
/// âncora pode ir.
///
/// ⚠️ Sem esta metade, alguém «consertaria» o `Expand` mexendo na escala, e a imagem do que está
/// dentro da caixa de referência mudaria — que é exactamente o que o alvo NÃO faz.
#[test]
fn a_pose_do_expand_e_a_do_keep_ao_bit() {
    let k = canvas16x9(Fit::Keep);
    let e = canvas16x9(Fit::Expand);
    for (hw, hh) in [(21.0, 9.0), (16.0, 12.0), (16.0, 9.0), (40.0, 4.0)] {
        let v = vista(hw, hh);
        assert_eq!(place(&k, v), place(&e, v), "{hw}x{hh}: as poses divergiram");
    }
}

/// Uma linha da tabela do oráculo: `(ref_w, ref_h, fit, meia-janela, a caixa que ele devolveu)`.
///
/// ⚠️ Um `type` porque o clippy recusa a tupla escrita à mão (*«very complex type»*) — e a forma é
/// o ponto: são cinco colunas, e o dia em que faltar uma o gate deixa de compilar.
type Caso = (f32, f32, Fit, [f32; 2], [f32; 2]);

/// ⭐⭐⭐ **A tabela do ORÁCULO, verbatim** (bloco L4, janela `720×450`).
///
/// ⚠️ A régua é a caixa que o filho ancorado LÊ, em unidades locais — que é o `rect_fim` que o alvo
/// imprime. Os números estão aqui **verbatim** de propósito: é o que torna a tabela conferível
/// contra a saída da sonda sem ninguém a reformatar de cabeça.
#[test]
fn a_caixa_efectiva_bate_o_oraculo_ao_numero() {
    // (ref_w, ref_h, fit, meia-janela, a caixa que o alvo devolveu)
    let casos: [Caso; 6] = [
        (640.0, 360.0, Fit::Keep, [360.0, 225.0], [640.0, 360.0]),
        (1280.0, 360.0, Fit::Keep, [360.0, 225.0], [1280.0, 360.0]),
        (320.0, 480.0, Fit::Keep, [360.0, 225.0], [320.0, 480.0]),
        (640.0, 360.0, Fit::Expand, [360.0, 225.0], [640.0, 400.0]),
        (1280.0, 360.0, Fit::Expand, [360.0, 225.0], [1280.0, 800.0]),
        (320.0, 480.0, Fit::Expand, [360.0, 225.0], [768.0, 480.0]),
    ];
    for (rw, rh, fit, half, esperado) in casos {
        let c = Canvas::new(rw, rh, fit).expect("canvas");
        let e = effective_box(&c, vista(half[0], half[1]));
        // a caixa é centrada ⇒ a LARGURA é `2 × hx`
        let (w, h) = (e[2] * 2.0, e[3] * 2.0);
        assert!(
            (w - esperado[0]).abs() < 0.01 && (h - esperado[1]).abs() < 0.01,
            "{fit:?} ref {rw}x{rh}: demos ({w}, {h}) e o alvo deu ({}, {})",
            esperado[0],
            esperado[1]
        );
    }
}

/// ⭐ **Todo modo é alcançável pelo selector, tem rótulo, e o índice é uma involução.**
///
/// ⚠️ A metade que importa é a CONTAGEM: um modo novo fora do `ALL` existe, tem lei, tem gates — e
/// o artista não lhe chega.
#[test]
fn todo_modo_e_alcancavel_e_o_indice_volta() {
    assert_eq!(Fit::ALL.len(), 3);
    for f in Fit::ALL {
        assert!(!f.label().is_empty());
        assert_eq!(Fit::from_index(f.index()), f, "o indice de {f:?} nao volta");
    }
    assert_eq!(Fit::default(), Fit::Keep);
    // ⛔ Dois modos com o mesmo rótulo dariam duas linhas indistinguíveis no menu.
    let mut rotulos: Vec<&str> = Fit::ALL.iter().map(|f| f.label()).collect();
    rotulos.sort_unstable();
    rotulos.dedup();
    assert_eq!(rotulos.len(), 3, "dois modos partilham rotulo");
    // ⚠️ Fora de alcance cai no de fábrica, e não em pânico.
    assert_eq!(Fit::from_index(99), Fit::Keep);
}
