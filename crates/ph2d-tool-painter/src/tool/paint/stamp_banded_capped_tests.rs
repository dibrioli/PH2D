//! O cap de accumulate atravessa a divisão em bandas (2026-08-04).

use super::*;

// ---------------------------------------------------------------------------------------------
// O CAP DE ACCUMULATE ATRAVESSA A DIVISÃO (2026-08-04)
// ---------------------------------------------------------------------------------------------
//
// ⚠️ **A premissa que estes gates enterram:** o cap era excluído do lote em banda porque tem *"estado
// compartilhado"* — e a máscara é compartilhada entre DABS, nunca entre LINHAS. Ela é lida e escrita
// por-texel, no índice do próprio pixel; bandas são linhas disjuntas. O que muda é que agora existem
// DUAS fatias a manter em passo, e é isso que o gate absoluto abaixo vigia.

/// Um pincel cujo cap é **observável**: `strength < 1` faz `stroke_cover_wanted` disparar, e o teto por
/// texel passa a ser exatamente `strength` — então o segundo dab sobre o mesmo lugar tem o que LER.
fn capped_brush() -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        color: [0.1, 0.2, 0.8],
        strength: 0.5,
        ..BrushSpec::default()
    }
}

/// `(canvas, mask)` depois do lote, com o piso dado.
fn capped_batch(dabs: &[Dab], min_area: usize) -> (Vec<u8>, Vec<u8>) {
    let mut buf = canvas();
    let mut mask = vec![0u8; (W as usize) * (H as usize)];
    let _ = stamp_plain_dabs_banded_with(
        &mut buf,
        W,
        H,
        dabs,
        &capped_brush(),
        false,
        Some(&mut mask),
        min_area,
    );
    (buf, mask)
}

/// **Dividir as linhas não move um byte — nem da tinta, nem da COBERTURA.**
///
/// ⚠️ A comparação é contra `min_area = usize::MAX`, que é o laço `for d in dabs` do produto: as duas
/// rotas aqui são *a de antes desta wave* e *a de agora*, nunca uma segunda implementação escrita para
/// o teste.
///
/// ⚠️ **A fixture TEM de conter o fenômeno em duas frentes** — o lote precisa cruzar o piso (asserido,
/// não presumido) e os dabs precisam **se sobrepor**, porque a razão de o cap existir é ler a cobertura
/// que um dab anterior deixou. Sobre uma máscara que ninguém escreveu duas vezes, um erro de fatia pode
/// passar sem deixar marca.
#[test]
fn the_capped_batch_is_byte_identical_whether_its_rows_are_split_or_not() {
    // ⚠️ **O `n = 2` virou `6`, e quem mandou foi a asserção de vácuo logo abaixo.** Enquanto a
    // contagem de bandas era `available_parallelism()`, `min_area = 0` bastava para forçar a divisão;
    // desde que ela sai do TRABALHO, um lote de 2 dabs de raio 12 (1 458 visitas) pede **uma** banda —
    // e as duas chamadas viravam a mesma rota. Seis dabs no MESMO anel de raio 40 continuam **sem se
    // tocar** (o espaçamento é 42 px contra 24 de diâmetro), então o controle *"a rota dividida não
    // depende de haver sobreposição"* sobrevive intacto; o que muda é que agora ele de fato divide.
    for n in [6usize, 17, 200] {
        let dabs = arc(n, 40.0);
        // ⚠️ **A premissa é que as duas rotas DIFIRAM, e ela se afirma pela porta do produto.** O
        // `wants_bands` DEVOLVE a decisão real (quantas bandas, não *"passou do piso"*), então esta
        // linha falha alto quando a fixture deixa de conter o fenômeno — foi ela que pegou o `n = 2`.
        assert!(
            wants_bands(batch_work(&caixas_de(&dabs)), &caixas_de(&dabs), 0),
            "n={n}: sem divisão as duas chamadas são o MESMO código e o verde é vácuo"
        );
        let (par_buf, par_mask) = capped_batch(&dabs, 0);
        let (ser_buf, ser_mask) = capped_batch(&dabs, usize::MAX);

        let written = ser_mask.iter().filter(|&&m| m > 0).count();
        assert!(
            written > 500,
            "n={n}: a máscara tem de estar escrita para haver o que comparar (got {written})"
        );
        // ⚠️ **A premissa que faltava, e as DUAS versões erradas que eu escrevi antes dela.** A razão
        // de o cap existir é LER a cobertura que um dab anterior deixou, então a máscara tem de ser
        // load-bearing NESTA fixture — senão o gate compara duas rotas sobre um buffer inerte. Tentei
        // afirmá-lo contando texels *"no teto"*, e o teto não é o que eu supus: medido, a máscara
        // satura em **76**, que é `coverage 0,6 × strength 0,5 × 255` — e um dab SOZINHO já o alcança
        // no centro, então contar texels saturados não distingue sobreposição de dab único. O que
        // distingue é o efeito: **com o cap a tinta tem de sair diferente de sem ele**.
        //
        // ⚠️ `n = 6` fica de fora: seis dabs num arco de raio 40 ficam a 42 px e, com raio 12, não se
        // tocam — ali o cap é honestamente inerte, e ele é o controle de que a rota dividida não
        // depende de haver sobreposição para estar certa.
        if n > 6 {
            let mut plain_buf = canvas();
            let _ = stamp_plain_dabs_banded_with(
                &mut plain_buf,
                W,
                H,
                &dabs,
                &capped_brush(),
                false,
                None,
                usize::MAX,
            );
            let cap_effect = plain_buf
                .iter()
                .zip(&ser_buf)
                .filter(|(a, b)| a != b)
                .count();
            assert!(
                cap_effect > 1000,
                "n={n}: o cap não mudou a tinta ({cap_effect} bytes) — a máscara está inerte \
                 nesta fixture e a comparação entre rotas seria vácuo"
            );
        }

        let cd = par_buf.iter().zip(&ser_buf).filter(|(a, b)| a != b).count();
        assert_eq!(
            cd, 0,
            "n={n}: a TINTA divergiu entre banda e serial ({cd} bytes)"
        );
        let md = par_mask
            .iter()
            .zip(&ser_mask)
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(md, 0, "n={n}: a COBERTURA divergiu ({md} texels)");
    }
}

/// **A cobertura é escrita onde a tinta caiu** — o oráculo ABSOLUTO, e ele não é redundante.
///
/// ⚠️ O gate acima compara duas ROTAS, e isso tem um limite conhecido: o recorte `y_top * mrow` é
/// computado no corpo compartilhado, antes do ramo, então um erro ali move as duas rotas igual e a
/// comparação fica verde — *razão entre dois doentes*
/// ([[feedback_an_identity_gate_cannot_see_a_defect_in_the_shared_body]]). Este mede contra o mundo:
/// as linhas escritas na máscara têm de cair dentro da pegada do lote.
///
/// ⚠️ **E o lote nasce longe da linha 0 de propósito:** com `y_top == 0` um deslocamento esquecido é
/// indistinguível do recorte certo.
#[test]
fn the_cap_is_written_where_the_batch_painted_not_displaced() {
    let dabs = arc(60, 40.0);
    let mut buf = canvas();
    let mut mask = vec![0u8; (W as usize) * (H as usize)];
    let r = stamp_plain_dabs_banded_with(
        &mut buf,
        W,
        H,
        &dabs,
        &capped_brush(),
        false,
        Some(&mut mask),
        0,
    )
    .expect("o lote pintou");
    assert!(r.y > 0, "a fixture TEM de ter o lote longe da linha 0");

    let rows: Vec<u32> = (0..H)
        .filter(|y| (0..W).any(|x| mask[(y * W + x) as usize] > 0))
        .collect();
    let (first, last) = (rows[0], rows[rows.len() - 1]);
    assert!(
        first >= r.y && last < r.y + r.h,
        "a cobertura foi escrita nas linhas {first}..={last}, fora da pegada do lote ({}..{}) \
         — a fatia da máscara está deslocada em relação à da tinta",
        r.y,
        r.y + r.h
    );
}

/// **Um pincel COM o cap chega ao lote em banda** — a metade que prova que o predicado partiu.
///
/// ⚠️ Este gate é sobre o `stamp_dabs_per_pixel`, não sobre o motor: a identidade acima podia estar
/// perfeita com o produto ainda mandando todo lote capeado para o laço serial, porque a decisão mora
/// numa linha de predicado uma camada acima. Enquanto `plain` respondia às duas perguntas de uma vez
/// (*a banda consegue carregar isto?* e *o WGSL transcreve todas as leis?*), a resposta do device
/// vetava a CPU.
///
/// ⚠️ **O alcance é maior que o impasto e é por isso que a fixture usa `strength`:** `stroke_cover_wanted`
/// dispara em `strength < 1`, ajuste comum de pincel digital — não é um caso de canto do relevo.
///
/// ⚠️ **O controle positivo NÃO passa pelo instrumento sob teste** (a lição do gate irmão): quem prova
/// que a fixture pintou é o CANVAS, porque `dabs` conta só quem chega a este módulo e zeraria nas duas
/// hipóteses.
#[test]
fn a_capped_brush_reaches_the_banded_batch_too() {
    use ph2d_painter_brush::StrokeMethod;
    let mut t = crate::tool::PainterTool::default();
    t.set_source(vec![255u8; 1024 * 1024 * 4], 1024, 1024);
    t.set_brush_size_px(40.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    // O cap, pela porta que o artista de fato mexe.
    t.paint.brush.strength = 0.5;
    for slot in &mut t.paint.brush_by_mode {
        slot.strength = 0.5;
        slot.stroke_method = StrokeMethod::Ellipse;
    }
    assert!(
        t.stroke_cover_wanted(&t.paint.brush),
        "a fixture TEM de ligar o cap, senão ela testa o mundo de antes"
    );

    let _ = super::super::stamp_banded::diag::take();
    t.on_canvas_pointer(cpx([100.0, 412.0], PointerPhase::Down));
    t.on_canvas_pointer(cpx([900.0, 612.0], PointerPhase::Move));
    // ⚠️ **O carimbo agora acontece em REPOUSO** (`super::shape_draft`): sob a mão o gizmo é o
    // preview e nenhum lote é produzido, então a fixture tem de SOLTAR — senão ela mede o silêncio.
    t.on_canvas_pointer(cpx([900.0, 612.0], PointerPhase::Up));
    let d = super::super::stamp_banded::diag::take();
    assert!(
        t.canvas_rgba.iter().any(|&b| b != 255),
        "a fixture não pintou um pixel — ela não contém o fenômeno"
    );
    assert!(
        d.banded > 0,
        "um pincel com o cap de Accumulate NÃO alcança o lote em banda: {} em banda x {} serial(is), \
         {} dabs no módulo — o predicado voltou a colapsar as duas perguntas numa só",
        d.banded,
        d.serial,
        d.dabs
    );
}
