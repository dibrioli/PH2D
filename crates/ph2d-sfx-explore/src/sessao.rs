//! **O que o artista pediu e ouviu:** de onde parte o sorteio, a lista Heard, e a grade do Explore.

use std::sync::Arc;

use ph2d_sfx_dsp::RenderError;
use ph2d_sfx_recipe::{Como, Ouvido, Recipe, Takes, factory};

use crate::lei::{TAMANHO_MUTATE, TAMANHO_ROLL, varia};
use crate::ninhada::{AVULSO, FILHOS, Filho, Ninhada};

/// O recurso é o tamanho do arquivo do projeto: 10 000 receitas de 4 camadas são 2,4 MB e gravam em 3,4 ms; a um som por segundo,
/// quase 3 h seguidas no mesmo som (`docs/SfxLab/05_W4_plano.md` §6). O mais velho sem estrela sai primeiro, e o recém-ouvido
/// nunca. A estrela é escolha do artista: só as estrelas passam a lista deste teto.
pub const MAX_OUVIDOS: usize = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tipo {
    Roll,
    Mutate,
}

#[derive(Clone, Debug)]
struct Sorteio {
    tipo: Tipo,
    origem: Recipe,
    resultado: Recipe,
    como: Como,
}

/// O que sobrevive à aba: a lista Heard. O ponto de partida não: ele se deduz.
#[derive(Clone, Debug, Default)]
pub struct Sessao {
    numero: u64,
    ultimo: Option<Sorteio>,
    pub ouvidos: Vec<Ouvido>,
    /// Há ouvidos por escrever no mundo.
    pub sujo: bool,
}

impl Sessao {
    fn proximo(&mut self) -> u64 {
        self.numero += 1;
        self.numero
    }

    /// De onde o Roll parte: o último som ESCOLHIDO. `R` repetido não anda sem parar até o horror; qualquer mudança que não veio
    /// do próprio sorteio (mão, desfazer, Keep, Heard) vira o novo ponto de partida.
    fn origem_do_roll(&self, atual: &Recipe) -> Recipe {
        match &self.ultimo {
            Some(s) if s.resultado == *atual => s.origem.clone(),
            _ => atual.clone(),
        }
    }

    pub fn roll(&mut self, atual: &Recipe) -> Recipe {
        let origem = self.origem_do_roll(atual);
        self.sorteia(Tipo::Roll, origem)
    }

    /// Anda: parte do som atual.
    pub fn mutate(&mut self, atual: &Recipe) -> Recipe {
        self.sorteia(Tipo::Mutate, atual.clone())
    }

    /// Joga fora a tentativa e traz outra do mesmo gesto, do mesmo ponto de partida.
    pub fn next(&mut self, atual: &Recipe) -> Recipe {
        match self.ultimo.clone() {
            Some(s) if s.resultado == *atual => self.sorteia(s.tipo, s.origem),
            Some(s) => self.sorteia(s.tipo, atual.clone()),
            None => self.roll(atual),
        }
    }

    fn sorteia(&mut self, tipo: Tipo, origem: Recipe) -> Recipe {
        let numero = self.proximo();
        let (resultado, como) = match tipo {
            Tipo::Roll => (
                de_algum_lugar(&origem, numero, TAMANHO_ROLL),
                Como::Roll(numero),
            ),
            Tipo::Mutate => (
                de_algum_lugar(&origem, numero, TAMANHO_MUTATE),
                Como::Mutate(numero),
            ),
        };
        self.ultimo = Some(Sorteio {
            tipo,
            origem,
            resultado: resultado.clone(),
            como,
        });
        resultado
    }

    /// Como este som nasceu, para a tela e para a lista.
    #[must_use]
    pub fn como(&self, atual: &Recipe) -> Como {
        match &self.ultimo {
            Some(s) if s.resultado == *atual => s.como,
            _ => Como::Mao,
        }
    }

    /// O som tocou de verdade: é o único caminho para a lista. Igual a uma linha, não repete.
    pub fn ouviu(&mut self, receita: &Recipe, como: Como) {
        let limpa = sem_aprovado(receita);
        if self.ouvidos.iter().any(|o| o.receita == limpa) {
            return;
        }
        self.ouvidos.push(Ouvido {
            receita: limpa,
            como,
            estrela: false,
        });
        self.corta();
        self.sujo = true;
    }

    fn corta(&mut self) {
        while self.ouvidos.len() > MAX_OUVIDOS {
            let velhos = &self.ouvidos[..self.ouvidos.len() - 1];
            let Some(i) = velhos.iter().position(|o| !o.estrela) else {
                return;
            };
            self.ouvidos.remove(i);
        }
    }

    /// Keep: o som ganha a estrela e vira o ponto de partida.
    pub fn keep(&mut self, atual: &Recipe) {
        let como = self.como(atual);
        self.ouviu(atual, como);
        let limpa = sem_aprovado(atual);
        if let Some(o) = self.ouvidos.iter_mut().find(|o| o.receita == limpa) {
            o.estrela = true;
            self.sujo = true;
        }
        self.ultimo = None;
    }

    pub fn alterna_estrela(&mut self, i: usize) {
        if let Some(o) = self.ouvidos.get_mut(i) {
            o.estrela = !o.estrela;
            self.sujo = true;
        }
    }

    /// A lista veio do projeto. O sorteio continua depois do maior número: reabrir não repete o que já se ouviu.
    pub fn adota(&mut self, ouvidos: Vec<Ouvido>) {
        let maior = ouvidos.iter().map(|o| numero_de(o.como)).max().unwrap_or(0);
        self.numero = self.numero.max(maior);
        self.ouvidos = ouvidos;
        self.sujo = false;
    }

    /// O mundo tem outra lista? Corre uma vez por imagem: compara a receita só da última, e `como` e estrela de todas.
    #[must_use]
    pub fn difere_de(&self, outra: &[Ouvido]) -> bool {
        self.ouvidos.len() != outra.len()
            || (self.ouvidos.iter().zip(outra))
                .any(|(a, b)| a.como != b.como || a.estrela != b.estrela)
            || self.ouvidos.last().map(|o| &o.receita) != outra.last().map(|o| &o.receita)
    }
}

fn numero_de(c: Como) -> u64 {
    match c {
        Como::Mao => 0,
        Como::Roll(n) | Como::Mutate(n) | Como::Variacao(n) => n,
    }
}

fn sem_aprovado(r: &Recipe) -> Recipe {
    let mut limpa = r.clone();
    limpa.approved = None;
    limpa
}

/// Som sem camadas não tem o que variar: o primeiro clique já soa, a partir de um som de fábrica.
fn de_algum_lugar(origem: &Recipe, numero: u64, tamanho: f64) -> Recipe {
    if !origem.layers.is_empty() {
        return varia(origem, numero, tamanho);
    }
    let fabrica = factory::all();
    let (_, r) = &fabrica[(numero % fabrica.len() as u64) as usize];
    varia(r, numero, tamanho)
}

/// As oito variações em volta do som atual. `tamanhos` é por filho: arrastar um não mexe nos outros.
#[derive(Debug)]
pub struct Grade {
    pub base: Recipe,
    numero0: u64,
    /// O `Change`: de 0 a [`TAMANHO_ROLL`].
    pub tamanho: f64,
    pub tamanhos: [f64; FILHOS],
    pub filhos: [Option<Filho>; FILHOS],
    /// Um render deste lugar está a caminho: o som de agora é velho, e não pode tocar no lugar do novo.
    esperando: [bool; FILHOS],
    /// O último que se ouviu: é ele que `Enter` e `K` pegam.
    pub ultimo: Option<usize>,
    pub erro: Option<RenderError>,
}

impl Grade {
    fn numero(&self, i: usize) -> u64 {
        self.numero0 + i as u64 + 1
    }

    #[must_use]
    pub fn como(&self, i: usize) -> Como {
        Como::Variacao(self.numero(i))
    }

    /// A receita do lugar `i`, de agora: não depende do render ter chegado.
    #[must_use]
    pub fn receita(&self, i: usize) -> Recipe {
        de_algum_lugar(&self.base, self.numero(i), self.tamanhos[i])
    }
}

/// A exploração de uma aba.
#[derive(Default)]
pub struct Exploracao {
    pub sessao: Sessao,
    pub grade: Option<Grade>,
    ninhada: Ninhada,
    /// O que tocar assim que chegar; `FILHOS` = o som pedido à lista.
    a_tocar: Option<usize>,
    /// O som da lista que está a chegar.
    avulso: Option<(Recipe, Como)>,
}

impl Exploracao {
    #[must_use]
    pub fn aberta(&self) -> bool {
        self.grade.is_some()
    }

    /// `Tab`.
    pub fn alterna(&mut self, atual: &Recipe, takes: &Arc<Takes>) {
        if self.grade.take().is_some() {
            self.ninhada.esquece();
            self.a_tocar = None;
            return;
        }
        self.semeia(atual, TAMANHO_MUTATE, takes);
    }

    /// Oito novas em volta de `base`.
    pub fn semeia(&mut self, base: &Recipe, tamanho: f64, takes: &Arc<Takes>) {
        self.a_tocar = None;
        let numero0 = self.sessao.numero;
        self.sessao.numero += FILHOS as u64;
        let g = Grade {
            base: base.clone(),
            numero0,
            tamanho,
            tamanhos: [tamanho; FILHOS],
            filhos: Default::default(),
            esperando: [true; FILHOS],
            ultimo: None,
            erro: None,
        };
        for i in 0..FILHOS {
            self.ninhada.pede(i, g.receita(i), takes.clone());
        }
        self.grade = Some(g);
    }

    /// Uma variação com outro tamanho, sem mudar as outras.
    pub fn redimensiona(&mut self, i: usize, tamanho: f64, takes: &Arc<Takes>) {
        let Some(g) = self.grade.as_mut() else {
            return;
        };
        if g.tamanhos[i] == tamanho {
            return;
        }
        g.tamanhos[i] = tamanho;
        g.esperando[i] = true;
        let r = g.receita(i);
        self.ninhada.pede(i, r, takes.clone());
    }

    /// Toca a variação assim que ela existir.
    pub fn toca(&mut self, i: usize) {
        if let Some(g) = self.grade.as_mut() {
            g.ultimo = Some(i);
        }
        self.a_tocar = Some(i);
    }

    /// `N` no Explore: a próxima; depois da oitava, oito novas.
    pub fn proximo_filho(&mut self, takes: &Arc<Takes>) {
        let Some(g) = self.grade.as_ref() else {
            return;
        };
        let i = g.ultimo.map_or(0, |u| u + 1);
        if i >= FILHOS {
            let (base, tamanho) = (g.base.clone(), g.tamanho);
            self.semeia(&base, tamanho, takes);
            self.toca(0);
        } else {
            self.toca(i);
        }
    }

    /// A linha `i` da lista, ouvida sem virar o som atual.
    pub fn ouve_a_linha(&mut self, i: usize, takes: &Arc<Takes>) {
        let Some(o) = self.sessao.ouvidos.get(i) else {
            return;
        };
        self.avulso = Some((o.receita.clone(), o.como));
        self.ninhada.pede(AVULSO, o.receita.clone(), takes.clone());
        self.a_tocar = Some(AVULSO);
    }

    /// O que tocar nesta imagem, se algo chegou. A chegada entra na grade, e o som que toca entra na lista Heard.
    pub fn serve(
        &mut self,
        atual: &Recipe,
        em_gesto: bool,
        takes: &Arc<Takes>,
    ) -> Option<Vec<f32>> {
        self.refaz_se_o_atual_mudou(atual, em_gesto, takes);
        let mut do_avulso = None;
        for (lugar, r) in self.ninhada.recebe() {
            match (lugar, r) {
                (AVULSO, Ok(f)) => do_avulso = Some(f),
                (i, Ok(f)) => {
                    if let Some(g) = self.grade.as_mut() {
                        g.filhos[i] = Some(f);
                        g.esperando[i] = false;
                        g.erro = None;
                    }
                }
                (lugar, Err(e)) => {
                    if self.a_tocar == Some(lugar) {
                        self.a_tocar = None;
                    }
                    if let Some(g) = self.grade.as_mut() {
                        g.erro = Some(e);
                        if let Some(esperando) = g.esperando.get_mut(lugar) {
                            *esperando = false;
                        }
                    }
                }
            }
        }
        match self.a_tocar? {
            i if i < FILHOS && self.grade.as_ref()?.esperando[i] => None,
            AVULSO => {
                let f = do_avulso?;
                self.a_tocar = None;
                self.avulso = None;
                Some(f.nivelada)
            }
            i => {
                let g = self.grade.as_ref()?;
                let f = g.filhos[i].as_ref()?;
                let (som, receita, como) = (f.nivelada.clone(), f.receita.clone(), g.como(i));
                self.a_tocar = None;
                self.sessao.ouviu(&receita, como);
                Some(som)
            }
        }
    }

    /// Mudou fora da grade (desfazer, mão, Heard): as oito passam a ser em volta do som novo.
    fn refaz_se_o_atual_mudou(&mut self, atual: &Recipe, em_gesto: bool, takes: &Arc<Takes>) {
        let Some(g) = self.grade.as_ref() else {
            return;
        };
        if em_gesto || sem_aprovado(&g.base) == sem_aprovado(atual) {
            return;
        }
        let tamanho = g.tamanho;
        self.semeia(atual, tamanho, takes);
    }

    #[must_use]
    pub fn em_voo(&self) -> bool {
        self.a_tocar.is_some() || self.ninhada.em_voo()
    }
}
