//! **Os títulos do popover do Preset** (dono 22/10, depois do smoke: o título apagado por PASTA lia-se
//! como «nomes de pincéis apagados»): um TÍTULO por categoria, do tipo cabeçalho, e nenhum por pasta.

use super::*;

fn e(rel: &str, nome: &str, categoria: &str, favorito: bool) -> st::VistaPreset {
    st::VistaPreset {
        relativo: rel.to_owned(),
        nome: nome.to_owned(),
        categoria: categoria.to_owned(),
        favorito,
    }
}

/// Três categorias, cada uma com pincéis de PASTAS (e motores) diferentes — a fixtura em que um título
/// por pasta e um por categoria dão números diferentes.
fn vista(ilegiveis: usize) -> VistaPresets {
    VistaPresets {
        entradas: vec![
            e(
                "Digital/Pencil/Graphite/A.ph2d-brush.png",
                "A",
                "Pencil",
                false,
            ),
            e(
                "WetPainter/Pencil/Charcoal/B.ph2d-brush.png",
                "B",
                "Pencil",
                false,
            ),
            e(
                "WetPainter/Pencil/Graphite/C.ph2d-brush.png",
                "C",
                "Pencil",
                true,
            ),
            e("Digital/Ink/Ballpoint/D.ph2d-brush.png", "D", "Ink", false),
            e("WetPainter/Ink/Pen/E.ph2d-brush.png", "E", "Ink", false),
            e("Digital/User/User/F.ph2d-brush.png", "F", "Other", false),
        ],
        ativo: None,
        politica: 0,
        ilegiveis,
    }
}

fn titulos(o: &[DropdownOption<usize>]) -> Vec<&str> {
    o.iter()
        .filter(|o| o.header)
        .map(|o| o.label.as_str())
        .collect()
}

/// (i) Exactamente um título por categoria presente (mais os Favoritos), na ordem da vista, e nenhum por
/// pasta: as cinco pastas não dão cinco títulos.
#[test]
fn um_titulo_por_categoria_e_nenhum_por_pasta() {
    let (o, _) = opcoes_do_preset(&vista(0));
    assert_eq!(
        titulos(&o),
        [
            tr("panel.painter_layers.wetpaint.preset_favorites"),
            "Pencil",
            "Ink",
            "Other"
        ]
    );
    assert_eq!(
        o.iter().filter(|o| !o.header).count(),
        6,
        "cada pincel uma vez (o favorito só no topo)"
    );
}

/// (ii) Os títulos são linhas de CABEÇALHO e nunca opções apagadas; os pincéis são opções vivas. O aviso
/// dos ilegíveis continua um aviso apagado, não um título.
#[test]
fn os_titulos_sao_cabecalhos_e_os_pinceis_opcoes() {
    let (o, _) = opcoes_do_preset(&vista(2));
    for l in &o {
        if l.header {
            assert!(!l.disabled, "o título {:?} pinta-se apagado", l.label);
        }
    }
    let pinceis: Vec<_> = o.iter().filter(|l| !l.header && !l.disabled).collect();
    assert_eq!(pinceis.len(), 6);
    let aviso: Vec<_> = o.iter().filter(|l| l.disabled).collect();
    assert_eq!(aviso.len(), 1, "o aviso dos ilegíveis");
    assert!(!aviso[0].header);
    assert_eq!(o[0].label, aviso[0].label, "o aviso vem primeiro");
}
