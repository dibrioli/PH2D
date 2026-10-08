//! A biblioteca: duas raízes (fábrica, utilizador) com a MESMA árvore `<motor>/<meio>/<grupo>/<nome>`.
//! O ficheiro do utilizador no mesmo caminho relativo SOBREPÕE o de fábrica («Save as Default»);
//! apagá-lo é o «Reset». Grupos = pastas; nenhuma lista à parte que possa divergir do disco.

use crate::modelo::{Erro, Origem, Preset};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A extensão de um preset (o `.png` no fim: qualquer visualizador o abre).
pub const EXTENSAO: &str = ".ph2d-brush.png";

/// Um preset visível na biblioteca.
#[derive(Clone, Debug, PartialEq)]
pub struct Entrada {
    /// `<motor>/<meio>/<grupo>/<nome>.ph2d-brush.png`, sempre com `/` — a chave de favoritos e recentes.
    pub relativo: String,
    /// O ficheiro que vale (o do utilizador quando sobrepõe).
    pub caminho: PathBuf,
    /// [`Origem::Fabrica`] ou [`Origem::Utilizador`] — de qual raiz veio o ficheiro que vale.
    pub origem: Origem,
    /// Há um de fábrica neste caminho (o «Reset» de um sobreposto volta a ele).
    pub tem_fabrica: bool,
    pub preset: Preset,
}

impl Entrada {
    /// As três pastas e o nome, pela ordem.
    #[must_use]
    pub fn partes(&self) -> [&str; 4] {
        partes(&self.relativo)
    }
}

fn partes(relativo: &str) -> [&str; 4] {
    let mut it = relativo.splitn(4, '/');
    let mut p = [""; 4];
    for s in &mut p {
        *s = it.next().unwrap_or("");
    }
    p[3] = p[3].strip_suffix(EXTENSAO).unwrap_or(p[3]);
    p
}

/// O caminho relativo de um preset.
#[must_use]
pub fn caminho_relativo(motor: &str, meio: &str, grupo: &str, nome: &str) -> String {
    format!("{motor}/{meio}/{grupo}/{nome}{EXTENSAO}")
}

/// A raiz do utilizador: `PH2D_BRUSHES_DIR` (os testes) · `$XDG_DATA_HOME/ph2d/brushes` ·
/// `$HOME/.local/share/ph2d/brushes`.
#[must_use]
pub fn pasta_do_utilizador() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("PH2D_BRUSHES_DIR") {
        return Some(PathBuf::from(d));
    }
    if let Some(d) = std::env::var_os("XDG_DATA_HOME").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d).join("ph2d").join("brushes"));
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share/ph2d/brushes"))
}

/// Os presets das duas raízes, por caminho relativo, e os ficheiros que não se leram (NUNCA em
/// silêncio: o painel diz quantos e o terminal diz quais e porquê).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Biblioteca {
    pub entradas: Vec<Entrada>,
    pub erros: Vec<(PathBuf, Erro)>,
}

fn varrer(raiz: &Path, achados: &mut BTreeMap<String, PathBuf>, erros: &mut Vec<(PathBuf, Erro)>) {
    fn pastas(p: &Path) -> Vec<PathBuf> {
        let Ok(rd) = std::fs::read_dir(p) else {
            return Vec::new();
        };
        let mut v: Vec<PathBuf> = rd.filter_map(Result::ok).map(|e| e.path()).collect();
        v.sort();
        v
    }
    for motor in pastas(raiz).into_iter().filter(|p| p.is_dir()) {
        for meio in pastas(&motor).into_iter().filter(|p| p.is_dir()) {
            for grupo in pastas(&meio).into_iter().filter(|p| p.is_dir()) {
                for f in pastas(&grupo) {
                    let nome = f.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !nome.ends_with(EXTENSAO) {
                        continue;
                    }
                    let rel = [&motor, &meio, &grupo]
                        .iter()
                        .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
                        .chain([nome])
                        .collect::<Vec<_>>()
                        .join("/");
                    if rel.split('/').count() == 4 {
                        achados.insert(rel, f);
                    } else {
                        erros.push((f, Erro::Disco("nome de pasta não é UTF-8".to_owned())));
                    }
                }
            }
        }
    }
}

impl Biblioteca {
    /// Lê as duas raízes. Uma raiz ausente é uma biblioteca vazia, não um erro.
    #[must_use]
    pub fn carregar(fabrica: &Path, utilizador: Option<&Path>) -> Self {
        let mut erros = Vec::new();
        let mut de_fabrica = BTreeMap::new();
        varrer(fabrica, &mut de_fabrica, &mut erros);
        let mut do_utilizador = BTreeMap::new();
        if let Some(u) = utilizador {
            varrer(u, &mut do_utilizador, &mut erros);
        }
        let mut rels: Vec<&String> = de_fabrica.keys().chain(do_utilizador.keys()).collect();
        rels.sort();
        rels.dedup();
        let mut entradas = Vec::with_capacity(rels.len());
        for rel in rels {
            let (caminho, origem) = match do_utilizador.get(rel) {
                Some(p) => (p.clone(), Origem::Utilizador),
                None => (de_fabrica[rel].clone(), Origem::Fabrica),
            };
            match std::fs::read(&caminho)
                .map_err(|e| Erro::Disco(e.to_string()))
                .and_then(|b| crate::ler(&b))
            {
                Ok(preset) => entradas.push(Entrada {
                    relativo: rel.clone(),
                    caminho,
                    origem,
                    tem_fabrica: de_fabrica.contains_key(rel),
                    preset,
                }),
                Err(e) => erros.push((caminho, e)),
            }
        }
        Self { entradas, erros }
    }

    #[must_use]
    pub fn procurar(&self, relativo: &str) -> Option<&Entrada> {
        self.entradas.iter().find(|e| e.relativo == relativo)
    }

    /// Um nome que ainda não existe no grupo: `base`, `base 2`, `base 3`… («Save as New»).
    #[must_use]
    pub fn nome_livre(&self, motor: &str, meio: &str, grupo: &str, base: &str) -> String {
        let livre = |n: &str| {
            self.procurar(&caminho_relativo(motor, meio, grupo, n))
                .is_none()
        };
        if livre(base) {
            return base.to_owned();
        }
        (2u32..)
            .map(|k| format!("{base} {k}"))
            .find(|n| livre(n))
            .expect("um inteiro livre existe")
    }
}

/// Grava um preset na raiz do utilizador, no caminho relativo (cria as pastas; escreve num ficheiro
/// irmão e renomeia, para um corte a meio nunca deixar um preset truncado no lugar do bom).
///
/// # Errors
/// [`Erro::Disco`] se o disco recusar; os de [`crate::escrever`].
pub fn gravar(raiz: &Path, relativo: &str, preset: &Preset) -> Result<PathBuf, Erro> {
    let bytes = crate::escrever(preset)?;
    let destino = raiz.join(relativo);
    let disco = |e: std::io::Error| Erro::Disco(format!("{}: {e}", destino.display()));
    if let Some(pai) = destino.parent() {
        std::fs::create_dir_all(pai).map_err(disco)?;
    }
    let tmp = destino.with_extension("png.tmp");
    std::fs::write(&tmp, &bytes).map_err(disco)?;
    std::fs::rename(&tmp, &destino).map_err(disco)?;
    Ok(destino)
}

/// Apaga o preset do utilizador nesse caminho (o «Reset» de um sobreposto). Ausente não é erro.
///
/// # Errors
/// [`Erro::Disco`] se o disco recusar.
pub fn apagar(raiz: &Path, relativo: &str) -> Result<(), Erro> {
    match std::fs::remove_file(raiz.join(relativo)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Erro::Disco(e.to_string())),
    }
}
