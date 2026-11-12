//! ⭐ **A MEMÓRIA da faixa, por utilizador** (line/BarraDeMenus, onda 3): os recentes da busca,
//! quantas vezes cada comando foi escolhido PELO MENU, os atalhos que o artista já aprendeu e os
//! projetos abertos por último. É do artista e da máquina dele, nunca do documento: vive em
//! `~/.ph2d/menu-memory.txt` (irmão do `prefs.txt`), texto `chave=valor`, std-only, best-effort.
//! ⚠️ Sem número de schema pela mesma razão do `prefs.txt`: a linha que não se entende salta-se.

use ph2d_a11y::NodeId;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Quantos recentes a busca guarda — mais do que a lista mostra, porque um recente que o modelo de
/// agora não oferece não aparece. Escolha de desenho, não tecto de recurso.
const RECENT_KEPT: usize = 20;
/// Quantos projetos o *Open Recent* lembra (o número do Blender, do Godot e do VS Code).
pub const PROJECTS_KEPT: usize = 10;
/// Na 3.ª escolha pelo menu o chip ensina o atalho (dono 12/11).
pub const TEACH_AT: u32 = 3;

/// O que a faixa lembra entre sessões.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuMemory {
    recent: Vec<NodeId>,
    picks: BTreeMap<NodeId, u32>,
    learned: BTreeSet<NodeId>,
    projects: Vec<String>,
    /// Mudou desde a última gravação.
    dirty: bool,
}

impl MenuMemory {
    /// Os últimos comandos escolhidos na busca, o mais novo primeiro, sem repetir.
    #[must_use]
    pub fn recent(&self) -> &[NodeId] {
        &self.recent
    }

    /// `id` foi escolhido na busca: sobe ao topo.
    pub fn remember(&mut self, id: NodeId) {
        self.recent.retain(|r| *r != id);
        self.recent.insert(0, id);
        self.recent.truncate(RECENT_KEPT);
        self.dirty = true;
    }

    /// `id` foi escolhido pelo MENU: devolve quantas vezes, contando esta.
    pub fn picked_from_menu(&mut self, id: NodeId) -> u32 {
        let n = self.picks.entry(id).or_insert(0);
        *n = n.saturating_add(1);
        self.dirty = true;
        *n
    }

    /// Quantas vezes `id` foi escolhido pelo menu.
    #[must_use]
    pub fn menu_picks(&self, id: NodeId) -> u32 {
        self.picks.get(&id).copied().unwrap_or(0)
    }

    /// O artista já sabe o atalho de `id` (usou a tecla, ou fechou o chip).
    #[must_use]
    pub fn knows(&self, id: NodeId) -> bool {
        self.learned.contains(&id)
    }

    /// `id` passa a ser sabido: o chip dele nunca volta.
    pub fn learn(&mut self, id: NodeId) {
        self.dirty |= self.learned.insert(id);
    }

    /// Os projetos abertos ou gravados por último, o mais novo primeiro.
    #[must_use]
    pub fn projects(&self) -> &[String] {
        &self.projects
    }

    /// `path` foi aberto ou gravado: sobe ao topo do *Open Recent*.
    pub fn note_project(&mut self, path: &str) {
        if self.projects.first().is_some_and(|p| p == path) {
            return;
        }
        self.projects.retain(|p| p != path);
        self.projects.insert(0, path.to_owned());
        self.projects.truncate(PROJECTS_KEPT);
        self.dirty = true;
    }

    /// `path` já não abre (apagado, movido): sai da lista.
    pub fn forget_project(&mut self, path: &str) {
        let n = self.projects.len();
        self.projects.retain(|p| p != path);
        self.dirty |= self.projects.len() != n;
    }

    /// Esvazia o *Open Recent*.
    pub fn clear_projects(&mut self) {
        self.dirty |= !self.projects.is_empty();
        self.projects.clear();
    }

    /// O texto do ficheiro. Inverso de [`Self::parse`].
    #[must_use]
    pub fn serialize(&self) -> String {
        use std::fmt::Write as _;
        let mut s = String::from("# PH2D menu memory\n");
        for id in &self.recent {
            let _ = writeln!(s, "recent={:x}", id.0);
        }
        for (id, n) in &self.picks {
            let _ = writeln!(s, "pick={:x} {n}", id.0);
        }
        for id in &self.learned {
            let _ = writeln!(s, "learned={:x}", id.0);
        }
        for p in &self.projects {
            let _ = writeln!(s, "project={p}");
        }
        s
    }

    /// Lê o texto; toda linha que não se entende fica de fora.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let hex = |v: &str| u64::from_str_radix(v.trim(), 16).ok().map(NodeId);
        let mut m = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "recent" => m.recent.extend(hex(value)),
                "pick" => {
                    if let Some((id, n)) = value.split_once(' ')
                        && let (Some(id), Ok(n)) = (hex(id), n.trim().parse())
                    {
                        m.picks.insert(id, n);
                    }
                }
                "learned" => m.learned.extend(hex(value)),
                "project" if !value.is_empty() => m.projects.push(value.to_owned()),
                _ => {}
            }
        }
        m.recent.truncate(RECENT_KEPT);
        m.projects.truncate(PROJECTS_KEPT);
        m
    }

    /// Lê `path`; sem ficheiro (ou ilegível), a memória nasce vazia.
    #[must_use]
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .map(|t| Self::parse(&t))
            .unwrap_or_default()
    }

    /// Grava em `path` se mudou desde a última vez (uma vez por quadro, na shell). Um erro de disco
    /// avisa e não volta a tentar até à próxima mudança.
    pub fn save_if_changed(&mut self, path: &Path) {
        if !std::mem::take(&mut self.dirty) {
            return;
        }
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(path, self.serialize()) {
            eprintln!("[ph2d] menu memory save: {e}");
        }
    }
}

impl MenuMemory {
    /// A memória do utilizador ([`default_path`]); vazia sem `$HOME` ou sem ficheiro.
    #[must_use]
    pub fn load_home() -> Self {
        default_path().map_or_else(Self::default, |p| Self::load(&p))
    }

    /// [`Self::save_if_changed`] no ficheiro do utilizador (o caminho só se pede se há o que gravar).
    pub fn save_home_if_changed(&mut self) {
        if self.dirty
            && let Some(p) = default_path()
        {
            self.save_if_changed(&p);
        }
    }
}

/// `~/.ph2d/menu-memory.txt`, ou `None` com `$HOME` por definir (a memória fica só na sessão).
#[must_use]
pub fn default_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".ph2d").join("menu-memory.txt"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_memoria_volta_igual_do_texto() {
        let mut m = MenuMemory::default();
        m.remember(NodeId(0xAB));
        m.remember(NodeId(7));
        for _ in 0..3 {
            m.picked_from_menu(NodeId(0xCAFE));
        }
        m.learn(NodeId(0xCAFE));
        m.note_project("/tmp/a b.ph2dproj");
        m.note_project("/tmp/c.ph2dproj");
        let back = MenuMemory::parse(&m.serialize());
        assert_eq!(back.recent(), &[NodeId(7), NodeId(0xAB)]);
        assert_eq!(back.menu_picks(NodeId(0xCAFE)), 3);
        assert!(back.knows(NodeId(0xCAFE)));
        assert_eq!(back.projects(), &["/tmp/c.ph2dproj", "/tmp/a b.ph2dproj"]);
        assert!(!back.dirty, "o lido não está por gravar");
    }

    #[test]
    fn lixo_no_ficheiro_nao_estraga_o_resto() {
        let m = MenuMemory::parse("# x\nrecent=zz\npick=10\npick=10 x\nrecent=1f\nnova=3\n");
        assert_eq!(m.recent(), &[NodeId(0x1f)]);
        assert_eq!(m.menu_picks(NodeId(0x10)), 0);
    }

    #[test]
    fn o_open_recent_sobe_o_ultimo_e_guarda_dez() {
        let mut m = MenuMemory::default();
        for i in 0..12 {
            m.note_project(&format!("/p/{i}"));
        }
        m.note_project("/p/5");
        assert_eq!(m.projects().len(), PROJECTS_KEPT);
        assert_eq!(m.projects()[0], "/p/5");
        assert_eq!(m.projects().iter().filter(|p| *p == "/p/5").count(), 1);
        m.forget_project("/p/5");
        assert_eq!(m.projects()[0], "/p/11");
    }

    #[test]
    fn grava_so_quando_muda() {
        let dir = std::env::temp_dir().join(format!("ph2d-menu-memory-{}", std::process::id()));
        let path = dir.join("menu-memory.txt");
        let mut m = MenuMemory::default();
        m.remember(NodeId(3));
        m.save_if_changed(&path);
        assert_eq!(MenuMemory::load(&path).recent(), &[NodeId(3)]);
        std::fs::write(&path, "").expect("escreve");
        m.save_if_changed(&path);
        assert_eq!(
            std::fs::read_to_string(&path).expect("lê"),
            "",
            "regravou sem mudar"
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
