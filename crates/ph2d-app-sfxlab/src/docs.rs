//! As abas da família, e a ponte delas com o mundo e o som (`service`, uma vez por imagem).

use std::sync::Arc;

use bevy_ecs::world::World;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr_with;
use ph2d_sfx_lab::aprovar::EstadoDoAprovado;
use ph2d_sfx_lab::medida::Medida;
use ph2d_sfx_recipe::{LayerId, Recipe, SfxLibrary, Takes};

use crate::aba::{Aba, Alvo};
use crate::escuta::Escuta;

#[derive(Default)]
pub struct SfxLabDocs {
    pub(crate) abas: Vec<Aba>,
    pub(crate) focus: Option<u64>,
    proximo: u64,
    /// As receitas do mundo, da última imagem.
    pub(crate) nomes: Vec<String>,
    pub(crate) takes: Arc<Takes>,
    pub(crate) abrir: Vec<String>,
    pub(crate) over: Option<Rect>,
    pub(crate) area: Option<Rect>,
    parar: bool,
    cena_aberta: bool,
}

impl SfxLabDocs {
    /// «New Sound»: nome único, e o som já toca.
    pub fn novo(&mut self) -> u64 {
        let nome = self.nome_livre();
        let r = ph2d_sfx_lab::novo::som_novo(&nome);
        self.abre_aba(nome, r, None)
    }

    /// O Create sound: o som leva o nome do objeto, único como o de qualquer som novo.
    pub(crate) fn novo_para(&mut self, objeto: &str) -> u64 {
        let base = tr_with("sfxlab.object_sound_name", &[("name", &objeto)]);
        let nome = self.primeiro_livre(
            std::iter::once(base.clone()).chain((2u32..).map(|n| format!("{base} {n}"))),
        );
        let r = ph2d_sfx_lab::novo::som_novo(&nome);
        self.abre_aba(nome, r, None)
    }

    fn nome_livre(&self) -> String {
        self.primeiro_livre((1u32..).map(|n| tr_with("sfxlab.default_name", &[("n", &n)])))
    }

    fn primeiro_livre(&self, mut candidatos: impl Iterator<Item = String>) -> String {
        let usado =
            |n: &str| self.nomes.iter().any(|x| x == n) || self.abas.iter().any(|a| a.nome == n);
        candidatos.find(|n| !usado(n)).unwrap_or_default()
    }

    fn abre_aba(&mut self, nome: String, r: Recipe, escrita: Option<Recipe>) -> u64 {
        self.proximo += 1;
        let id = self.proximo;
        self.abas
            .push(Aba::new(id, nome, r, escrita, self.takes.clone()));
        self.focus = Some(id);
        id
    }

    /// `true` só na primeira pergunta: a cena de smoke abre uma vez.
    pub(crate) fn cena_por_abrir(&mut self) -> bool {
        !std::mem::replace(&mut self.cena_aberta, true)
    }

    pub(crate) fn aba_mut(&mut self, id: u64) -> Option<&mut Aba> {
        self.abas.iter_mut().find(|a| a.id == id)
    }

    fn aba(&self, id: u64) -> Option<&Aba> {
        self.abas.iter().find(|a| a.id == id)
    }

    pub(crate) fn pede_abrir(&mut self, nome: String) {
        self.abrir.push(nome);
    }

    pub(crate) fn fecha(&mut self, id: u64) {
        self.abas.retain(|a| a.id != id);
        self.parar = true;
    }

    /// Uma vez por imagem: o mundo primeiro (quem abriu um projeto ganha), depois as edições e o som.
    pub fn servir(&mut self, world: &mut World, mut escuta: Option<&mut dyn Escuta>) {
        self.abrir_pedidos(world);
        self.adota_do_mundo(world);
        self.escreve_no_mundo(world);
        self.atende_o_jogo(world);
        for a in &mut self.abas {
            a.recebe();
            if a.quer_parar {
                a.quer_parar = false;
                self.parar = true;
            }
            if a.quer_tocar
                && !a.em_voo()
                && let Some(f) = &a.pronto
            {
                a.quer_tocar = false;
                if let Some(e) = escuta.as_deref_mut() {
                    e.toca(crate::escuta::som(&f.escuta, a.cursor));
                }
            }
        }
        if std::mem::take(&mut self.parar)
            && let Some(e) = escuta
        {
            e.para();
        }
    }

    fn abrir_pedidos(&mut self, world: &World) {
        for nome in std::mem::take(&mut self.abrir) {
            if let Some(a) = self.abas.iter().find(|a| a.nome == nome) {
                self.focus = Some(a.id);
            } else if let Some(r) = world
                .get_resource::<SfxLibrary>()
                .and_then(|l| l.recipes.get(&nome))
            {
                self.abre_aba(nome, r.clone(), Some(r.clone()));
            }
        }
    }

    fn adota_do_mundo(&mut self, world: &World) {
        let Some(lib) = world.get_resource::<SfxLibrary>() else {
            self.nomes.clear();
            return;
        };
        self.nomes.clear();
        self.nomes.extend(lib.recipes.keys().cloned());
        if !mesmos_takes(&self.takes, &lib.takes) {
            self.takes = Arc::new(lib.takes.clone());
            for a in &mut self.abas {
                a.takes = self.takes.clone();
                a.pede();
            }
        }
        let mut sumiram = Vec::new();
        for a in &mut self.abas {
            match (&a.escrita, lib.recipes.get(&a.nome)) {
                (Some(_), None) => sumiram.push(a.id),
                (Some(e), Some(w)) if e != w => a.adota(w.clone()),
                _ => {}
            }
        }
        for id in sumiram {
            self.fecha(id);
        }
    }

    fn escreve_no_mundo(&mut self, world: &mut World) {
        for a in &mut self.abas {
            if a.escrita.as_ref() == Some(&a.receita) {
                continue;
            }
            if !world.contains_resource::<SfxLibrary>() {
                world.insert_resource(SfxLibrary::default());
            }
            if let Some(mut lib) = world.get_resource_mut::<SfxLibrary>() {
                lib.recipes.insert(a.nome.clone(), a.receita.clone());
            }
            a.escrita = Some(a.receita.clone());
        }
    }

    // ── O que os testes e a tela do passo 4 leem ──

    #[must_use]
    pub fn ids(&self) -> Vec<u64> {
        self.abas.iter().map(|a| a.id).collect()
    }

    #[must_use]
    pub fn nome(&self, id: u64) -> Option<&str> {
        self.aba(id).map(|a| a.nome.as_str())
    }

    #[must_use]
    pub fn receita(&self, id: u64) -> Option<&Recipe> {
        self.aba(id).map(|a| &a.receita)
    }

    #[must_use]
    pub fn em_voo(&self, id: u64) -> bool {
        self.aba(id).is_some_and(Aba::em_voo)
    }

    /// O som do arquivo, sem limitador, do render mais novo que chegou.
    #[must_use]
    pub fn pcm(&self, id: u64) -> Option<&[f32]> {
        self.aba(id)?.pronto.as_ref().map(|f| &*f.pcm.samples)
    }

    #[must_use]
    pub fn selecionada(&self, id: u64) -> Option<LayerId> {
        self.aba(id)?.sel
    }

    #[must_use]
    pub fn solo(&self, id: u64) -> Option<LayerId> {
        self.aba(id)?.solo
    }

    /// Onde o alvo foi pintado por último.
    #[must_use]
    pub fn rect_de(&self, id: u64, alvo: Alvo) -> Option<Rect> {
        self.aba(id)?
            .alvos
            .iter()
            .find(|(a, _)| *a == alvo)
            .map(|(_, r)| *r)
    }

    /// Os medidores do render mais novo que chegou.
    #[must_use]
    pub fn medida(&self, id: u64) -> Option<&Medida> {
        self.aba(id)?.pronto.as_ref().map(|f| &f.medida)
    }

    /// A área que a aba recebeu na última pintura.
    #[must_use]
    pub fn area(&self) -> Option<Rect> {
        self.area
    }

    /// O Use in Game foi ao objeto ligado e ele não existe mais.
    #[must_use]
    pub fn objeto_sumiu(&self, id: u64) -> bool {
        self.aba(id).is_some_and(|a| a.objeto_sumiu)
    }

    /// O que a barra diz do som aprovado.
    #[must_use]
    pub fn estado_do_aprovado(&self, id: u64) -> Option<EstadoDoAprovado> {
        self.aba(id).map(Aba::estado_do_aprovado)
    }

    /// O som como se ouve, do render mais novo que chegou.
    #[must_use]
    pub fn escuta(&self, id: u64) -> Option<&[f32]> {
        self.aba(id)?.pronto.as_ref().map(|f| f.escuta.as_slice())
    }
}

fn mesmos_takes(a: &Takes, b: &Takes) -> bool {
    a.iter().map(|t| t.id).eq(b.iter().map(|t| t.id))
}
