//! B44.1 (missão 4K N10): `set_brush_size_px` toma o RAIO (`brush.radius_px`). 19 sondas passavam
//! `raio * 2.0` e rotulavam a coluna com o raio: o pincel medido tinha 4× a área. Censo de texto:
//! nenhuma chamada dobra o argumento fora da lista abaixo (cada uma com o porquê).
//! B44.1 fecho: num ficheiro de sonda (`measure_*`/`sonda_*`/`medida_*`/`mede_*`) o raio entra por
//! `sonda_raio`, que imprime o `brush_size_px()` LIDO (o corte em `raio_maximo` deixava de ser mudo).

use std::path::Path;

/// Fixturas de teste (não réguas de custo): o `Caso::r` / `RAIO` delas é meio raio por construção, e a
/// cena aprovada é a do raio dobrado — mudar o nome muda o que o portão vê.
const PERMITIDOS: &[&str] = &[
    "src/tool/paint/tests/auditoria_l2_seam.rs",
    "src/tool/paint/dual_ferramenta_tests.rs",
];

fn varre(
    dir: &Path,
    raiz: &Path,
    out: &mut Vec<String>,
    mudas: &mut Vec<String>,
    vistos: &mut usize,
) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            varre(&p, raiz, out, mudas, vistos);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let s = std::fs::read_to_string(&p).unwrap();
            let rel = p
                .strip_prefix(raiz)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let nome = p.file_name().unwrap().to_string_lossy();
            let sonda = ["measure_", "sonda_", "medida_", "mede_"]
                .iter()
                .any(|x| nome.starts_with(x));
            for (i, l) in s.lines().enumerate() {
                if l.trim_start().starts_with("//") {
                    continue;
                }
                let direta = l.find("set_brush_size_px(");
                let Some(k) = direta.or_else(|| l.find(".sonda_raio(")) else {
                    continue;
                };
                *vistos += 1;
                if sonda && direta.is_some() {
                    mudas.push(format!("{rel}:{} {}", i + 1, l.trim()));
                }
                let arg = &l[k..];
                let arg = &arg[..arg.find(')').map_or(arg.len(), |f| f + 1)];
                let dobra = arg.contains("* 2.0") || arg.contains("2.0 *");
                if dobra && !PERMITIDOS.contains(&rel.as_str()) {
                    out.push(format!("{rel}:{} {}", i + 1, l.trim()));
                }
            }
        }
    }
}

#[test]
fn as_sondas_passam_o_raio_ao_set_brush_size_px() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (mut maus, mut mudas, mut vistos) = (Vec::new(), Vec::new(), 0);
    for d in ["src", "tests", "examples"] {
        varre(&raiz.join(d), raiz, &mut maus, &mut mudas, &mut vistos);
    }
    assert!(
        vistos >= 50,
        "o censo leu só {vistos} chamadas — o nome mudou?"
    );
    assert!(
        maus.is_empty(),
        "`set_brush_size_px` toma o RAIO; estas passam o diâmetro:\n{}",
        maus.join("\n")
    );
    assert!(
        mudas.is_empty(),
        "uma sonda põe o raio por `sonda_raio` (imprime o `brush_size_px()` lido); estas chamam direto:\n{}",
        mudas.join("\n")
    );
}
