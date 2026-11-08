//! Os nomes das peças da folha.

use super::OpcoesFolha;

/// Tokens do `--filename-format`. `{frame}` conta na saída; `{frameNN}`: largura pelos dígitos, e o
/// valor deles é o primeiro número (`{frame1}` começa em 1, `{frame01}` tem 2 dígitos).
#[must_use]
pub fn nome_do_quadro(
    fmt: &str,
    titulo: &str,
    camada: &str,
    tag: &str,
    n: usize,
    ext: &str,
) -> String {
    let mut out = String::new();
    let mut resto = fmt;
    while let Some(a) = resto.find('{') {
        let Some(b) = resto[a..].find('}').map(|b| a + b) else {
            break;
        };
        out.push_str(&resto[..a]);
        let t = &resto[a + 1..b];
        match t {
            "title" | "name" => out.push_str(titulo),
            "layer" => out.push_str(camada),
            "tag" => out.push_str(tag),
            "extension" => out.push_str(ext),
            _ => match t.strip_prefix("frame") {
                Some(d) if d.bytes().all(|c| c.is_ascii_digit()) => {
                    let n = n + d.parse::<usize>().unwrap_or(0);
                    out.push_str(&format!("{n:0w$}", w = d.len()));
                }
                _ => {
                    out.push('{');
                    out.push_str(t);
                    out.push('}');
                }
            },
        }
        resto = &resto[b + 1..];
    }
    out.push_str(resto);
    out
}

pub(super) fn padrao(
    o: &OpcoesFolha,
    com_camada: bool,
    com_tag: bool,
    com_slice: bool,
    varios: bool,
) -> String {
    let mut f = String::from("{title}");
    if com_camada {
        f.push_str(" ({layer})");
    }
    if com_slice {
        f.push_str(" [{slice}]");
    }
    if com_tag {
        f.push_str(" #{tag}");
    }
    if varios {
        f.push_str(" {frame}");
    }
    f.push_str(".{extension}");
    let Some(dele) = o.nome.clone() else { return f };
    if !com_slice || dele.contains("{slice}") {
        return dele;
    }
    // Sem `{slice}` todas as slices teriam um nome: entra onde o por omissão o põe, depois do
    // `{title}`; sem ele, antes da extensão; sem ela, no fim.
    let at = dele
        .find("{title}")
        .map(|i| i + "{title}".len())
        .or_else(|| dele.rfind(".{extension}"))
        .unwrap_or(dele.len());
    let mut dele = dele;
    dele.insert_str(at, " [{slice}]");
    dele
}
