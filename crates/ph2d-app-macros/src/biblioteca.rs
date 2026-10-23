//! **A biblioteca do utilizador** (W18, `docs/Macros/16_W18_plano.md` §2.6): uma pasta, um
//! `<nome>.ph2dmacro` por macro — o molde de `ph2d-brush-presets/src/biblioteca.rs`. O nome é o do
//! ficheiro: nenhuma lista à parte que possa divergir do disco.

use ph2d_macro::ficheiro::{self, EXTENSAO, Ficheiro};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Erro {
    /// O disco recusou (o caminho e o porquê).
    Disco(String),
    /// O ficheiro não é uma macro que este build toque.
    Ficheiro(ph2d_macro::CodecError),
    /// O nome, depois de limpo, ficou vazio.
    NomeVazio,
    /// Já há uma macro com esse nome.
    JaExiste(String),
}

/// Uma macro da biblioteca.
#[derive(Clone, Debug, PartialEq)]
pub struct Entrada {
    pub nome: String,
    pub caminho: PathBuf,
    pub ficheiro: Ficheiro,
}

/// As macros da pasta, por nome, e os ficheiros que não se leram (nunca em silêncio).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Biblioteca {
    pub entradas: Vec<Entrada>,
    pub erros: Vec<(PathBuf, Erro)>,
}

/// A pasta: `PH2D_MACROS_DIR` (os testes, a cena de smoke) · `$XDG_DATA_HOME/ph2d/macros` ·
/// `$HOME/.local/share/ph2d/macros`.
#[must_use]
pub fn pasta_do_utilizador() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("PH2D_MACROS_DIR") {
        return Some(PathBuf::from(d));
    }
    if let Some(d) = std::env::var_os("XDG_DATA_HOME").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d).join("ph2d").join("macros"));
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share/ph2d/macros"))
}

/// **O nome que pode ser ficheiro**: sem separadores nem caracteres de controlo, sem espaços nas pontas.
#[must_use]
pub fn limpa(nome: &str) -> String {
    nome.chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect::<String>()
        .trim()
        .trim_start_matches('.')
        .to_owned()
}

fn caminho(raiz: &Path, nome: &str) -> PathBuf {
    raiz.join(format!("{nome}{EXTENSAO}"))
}

fn disco(p: &Path) -> impl Fn(std::io::Error) -> Erro + '_ {
    move |e| Erro::Disco(format!("{}: {e}", p.display()))
}

impl Biblioteca {
    /// Lê a pasta (ausente = vazia).
    #[must_use]
    pub fn carregar(raiz: &Path) -> Self {
        let mut b = Biblioteca::default();
        let Ok(rd) = std::fs::read_dir(raiz) else {
            return b;
        };
        let mut fs: Vec<PathBuf> = rd.filter_map(Result::ok).map(|e| e.path()).collect();
        fs.sort();
        for p in fs {
            let Some(nome) = p
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_suffix(EXTENSAO))
                .map(str::to_owned)
            else {
                continue;
            };
            match std::fs::read(&p)
                .map_err(disco(&p))
                .and_then(|b| ficheiro::le(&b).map_err(Erro::Ficheiro))
            {
                Ok(ficheiro) => b.entradas.push(Entrada {
                    nome,
                    caminho: p,
                    ficheiro,
                }),
                Err(e) => b.erros.push((p, e)),
            }
        }
        b
    }

    #[must_use]
    pub fn procura(&self, nome: &str) -> Option<&Entrada> {
        self.entradas.iter().find(|e| e.nome == nome)
    }

    /// `base`, ou `base 2`, `base 3`… — o primeiro que nenhuma entrada usa.
    #[must_use]
    pub fn nome_livre(&self, base: &str) -> String {
        let base = limpa(base);
        let base = if base.is_empty() { "Macro".to_owned() } else { base };
        if self.procura(&base).is_none() {
            return base;
        }
        (2..)
            .map(|n| format!("{base} {n}"))
            .find(|n| self.procura(n).is_none())
            .expect("há sempre um número livre")
    }
}

/// Escreve `bytes` em `destino` por um ficheiro irmão e renomeia: um corte a meio nunca deixa uma macro
/// truncada no lugar da boa.
fn escreve_atomico(destino: &Path, bytes: &[u8]) -> Result<(), Erro> {
    if let Some(pai) = destino.parent() {
        std::fs::create_dir_all(pai).map_err(disco(destino))?;
    }
    let tmp = destino.with_extension("ph2dmacro.tmp");
    std::fs::write(&tmp, bytes).map_err(disco(destino))?;
    std::fs::rename(&tmp, destino).map_err(disco(destino))
}

/// **Guarda** `f` na biblioteca com o nome `nome` (limpo). Recusa um nome que já existe.
///
/// # Errors
/// [`Erro`].
pub fn grava(raiz: &Path, nome: &str, f: &Ficheiro) -> Result<PathBuf, Erro> {
    let nome = limpa(nome);
    if nome.is_empty() {
        return Err(Erro::NomeVazio);
    }
    let destino = caminho(raiz, &nome);
    if destino.exists() {
        return Err(Erro::JaExiste(nome));
    }
    let mut f = f.clone();
    f.nome.clone_from(&nome);
    escreve_atomico(&destino, &ficheiro::escreve(&f))?;
    Ok(destino)
}

/// **Renomeia** a macro `de` para `para` (o nome dentro do ficheiro acompanha o do ficheiro).
///
/// # Errors
/// [`Erro`].
pub fn renomeia(raiz: &Path, de: &str, para: &str) -> Result<PathBuf, Erro> {
    let para = limpa(para);
    if para.is_empty() {
        return Err(Erro::NomeVazio);
    }
    let origem = caminho(raiz, de);
    let destino = caminho(raiz, &para);
    if para == de {
        return Ok(destino);
    }
    if destino.exists() {
        return Err(Erro::JaExiste(para));
    }
    let bytes = std::fs::read(&origem).map_err(disco(&origem))?;
    let mut f = ficheiro::le(&bytes).map_err(Erro::Ficheiro)?;
    f.nome.clone_from(&para);
    escreve_atomico(&destino, &ficheiro::escreve(&f))?;
    std::fs::remove_file(&origem).map_err(disco(&origem))?;
    Ok(destino)
}

/// **Apaga** a macro `nome`. Ausente não é erro.
///
/// # Errors
/// [`Erro::Disco`].
pub fn apaga(raiz: &Path, nome: &str) -> Result<(), Erro> {
    let p = caminho(raiz, nome);
    match std::fs::remove_file(&p) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(disco(&p)(e)),
    }
}

/// **Importa** o ficheiro `origem` (de qualquer pasta) para a biblioteca: lê-o primeiro — um ficheiro
/// que este build não toca recusa-se aqui, não na hora de aplicar. O nome é o de dentro do ficheiro (ou
/// o do ficheiro), livre na biblioteca. Devolve o nome.
///
/// # Errors
/// [`Erro`].
pub fn importa(raiz: &Path, origem: &Path) -> Result<String, Erro> {
    let bytes = std::fs::read(origem).map_err(disco(origem))?;
    let f = ficheiro::le(&bytes).map_err(Erro::Ficheiro)?;
    let base = if f.nome.trim().is_empty() {
        origem
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.strip_suffix(EXTENSAO).unwrap_or(n).to_owned())
            .unwrap_or_default()
    } else {
        f.nome.clone()
    };
    let nome = Biblioteca::carregar(raiz).nome_livre(&base);
    grava(raiz, &nome, &f)?;
    Ok(nome)
}

/// **Exporta** a macro para `destino` (o caminho que o artista escolheu; a extensão acrescenta-se se
/// faltar). Devolve o caminho escrito.
///
/// # Errors
/// [`Erro::Disco`].
pub fn exporta(destino: &Path, f: &Ficheiro) -> Result<PathBuf, Erro> {
    let destino = if destino
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.ends_with(EXTENSAO))
    {
        destino.to_path_buf()
    } else {
        let mut s = destino.as_os_str().to_owned();
        s.push(EXTENSAO);
        PathBuf::from(s)
    };
    escreve_atomico(&destino, &ficheiro::escreve(f))?;
    Ok(destino)
}
