//! **O PENTE DE TOPOLOGIA, conduzido pelo traço** — a fiação da [`ph2d_rake`].
//!
//! Irmão (`#[path]`) do [`super`], e o corte é o SUJEITO: a crate-folha tem a
//! LEI (que não sabe o que é uma malha nem um pincel) e aqui fica *quem a chama,
//! com que pegada, e o que se grava para o desfazer*.
//!
//! # ⛔⛔ Porque ele é o PRIMEIRO ACTO DO CARIMBO e não uma chamada à parte
//!
//! A espec §14.5 manda-o correr **dentro do laço por-carimbo**, sobre a malha
//! que o refino acabou de produzir — compor as duas metades em série falha a
//! barra em todas as granularidades medidas, e grosseira sai com o **sinal
//! trocado**. Ele podia ter sido uma porta pública chamada pelo braço do
//! carimbo logo a seguir ao refino; ⛔ **não pode**, por duas razões que só se
//! vêem de dentro:
//!
//! 1. **O DESFAZER.** O `capture` é `pub(super)` e a janela do traço é dele: um
//!    pente de fora moveria vértices sem gravar o `pre`, e o `Ctrl+Z` desfaria o
//!    carimbo deixando o penteado lá. *É a família que este módulo já pagou
//!    duas vezes* — o tecido em 05/09 e a pose em 14/09.
//! 2. **A JANELA DA GPU.** O [`super::SculptStroke::dab`] limpa
//!    `call_moved`/`call_refreshed` no início da CHAMADA; um pente que corresse
//!    antes veria a janela dele apagada, e o que ele moveu fora da pegada do
//!    verbo ficaria com normais velhas na tela, *com todos os gates de CPU
//!    verdes*.
//!
//! # ⭐ A direcção sai de graça
//!
//! Ela é o [`crate::Dab::path`], que o traço **já** deriva da diferença entre
//! centros de carimbos consecutivos. ⇒ a inércia que a espec §4.3 mede — *com
//! menos de dois carimbos o alvo não faz nada, não inventa uma direcção nem usa
//! a última* — cai por construção: no primeiro carimbo o `path` é nulo e a lei
//! devolve `0`.

/// **Quanto pente este traço leva de facto.**
///
/// ⛔⛔ Com a topologia dinâmica DESARMADA o pente é **inerte**, e isso não é
/// uma cerca defensiva: é lei, e o oráculo confirmou-a depois de ela ser
/// escrita por raciocínio de domínio — a célula `porta/c_nodyn` do corpus
/// mede o alvo com o interruptor desligado e o pente dele **não move um
/// vértice** (`max |p100 − p000| = 0,000e0`).
///
/// ⚠️ **Ela vive AQUI e não na crate da app**, e a razão é medida: enquanto
/// viveu lá, a bancada de paridade não lhe chegava e media um caminho que o
/// produto **não consegue percorrer** — lia `4,507e-2` de desvio naquela
/// célula sobre um produto que estaria exacto. *Uma sonda que mede um
/// sucedâneo mede outro programa.* Hoje a app **delega** aqui.
pub fn pente_do_traco(dyntopo_armado: bool, pente: f32) -> f32 {
    if dyntopo_armado { pente } else { 0.0 }
}

use ph2d_mesh::Mesh;

use crate::{Brush, SculptStroke};

impl SculptStroke {
    /// ⭐⭐⭐ **A DIRECÇÃO DO TRAÇO até este centro — e ela tem DOIS leitores.**
    ///
    /// Ela é a diferença entre CENTROS de carimbos consecutivos (ver
    /// [`crate::Dab::path`] para a lição de 2D que escolheu os centros em vez de
    /// uma tangente suavizada: **52,4°** de atraso num heading por média móvel,
    /// com o atraso a ESCALAR com o pincel).
    ///
    /// ⛔⛔ **Porque ela é uma porta e não duas linhas em cada sítio:** o
    /// deslocamento lê-a pelo `Dab::path`, e o **passe de topologia** lê-a antes
    /// disso — ele corre *antes* do carimbo, logo não tem `Dab` nenhum de onde a
    /// tirar. Escrita nos dois sítios, ela divergia no dia em que um deles
    /// ganhasse uma cerca; e as duas metades do pente ficariam a pentear em
    /// direcções diferentes **sem nada acusar**, porque cada uma passa nos
    /// gates da outra.
    ///
    /// ⚠️ **O primeiro carimbo devolve o vector NULO**, e é isso que faz a
    /// inércia da espec §4.3 cair por construção: sem dois centros não há
    /// direcção, e nem o deslocamento nem o refino inventam uma.
    #[must_use]
    pub fn direccao_do_traco(&self, centro: [f32; 3]) -> [f32; 3] {
        match self.last_center {
            Some(p) => [centro[0] - p[0], centro[1] - p[1], centro[2] - p[2]],
            None => [0.0; 3],
        }
    }

    /// Uma passagem do pente sobre a pegada de **uma** cópia de simetria.
    ///
    /// Enche `self.moved` e `self.region` como o [`Self::dab_core`] faz, e quem
    /// chama é que os despeja nas janelas da chamada — a mesma forma, para as
    /// duas não divergirem.
    pub(super) fn pentear_a_pegada(
        &mut self,
        mesh: &mut Mesh,
        brush: &Brush,
        centro: [f32; 3],
        direccao: [f32; 3],
    ) -> usize {
        // ⚠️ A pegada é a do PINCEL; o peso é a queda dele vezes o botão.
        let raio = brush.radius;
        let mut alvos = std::mem::take(&mut self.moved);
        alvos.clear();
        mesh.verts_in_sphere(centro, raio, &mut self.query, &mut alvos);
        if alvos.is_empty() {
            self.moved = alvos;
            return 0;
        }

        // ⚠️ **Tudo o que se lê da malha lê-se ANTES do empréstimo mutável** — a
        // normal de cada alvo e o anel dele —, e a cópia é da PEGADA e nunca da
        // malha: o pente é local por construção.
        let mut pesos: Vec<f32> = Vec::with_capacity(alvos.len());
        let mut normais: Vec<[f32; 3]> = Vec::with_capacity(alvos.len());
        let mut inicio: Vec<u32> = Vec::with_capacity(alvos.len() + 1);
        let mut anel: Vec<u32> = Vec::new();
        {
            let posicoes = mesh.positions();
            let normal_de = mesh.normals();
            let vizinhanca = &mesh.adjacency().vert_verts;
            for &v in &alvos {
                let p = posicoes[v as usize];
                let d = ((p[0] - centro[0]).powi(2)
                    + (p[1] - centro[1]).powi(2)
                    + (p[2] - centro[2]).powi(2))
                .sqrt();
                // ⚠️ **A queda NUA, sem a dureza.** A espec §5 dá a este pente
                // UM controlo e mais nenhum; passá-lo pelo `shaped_distance`
                // faria a dureza — que é do VERBO em mãos — mudar o alcance de
                // uma coisa que não é do verbo.
                pesos.push(brush.pente * brush.falloff.weight(d / raio.max(f32::MIN_POSITIVE)));
                normais.push(normal_de[v as usize]);
                inicio.push(u32::try_from(anel.len()).unwrap_or(u32::MAX));
                anel.extend_from_slice(vizinhanca.neighbours(v as usize));
            }
            inicio.push(u32::try_from(anel.len()).unwrap_or(u32::MAX));
        }

        // ⚠️⚠️ **A CAPTURA VEM ANTES DA ESCRITA**, e é isto que a separa de um
        // pente que o `Ctrl+Z` não desfaz: o `capture` lê `mesh.positions()`
        // para congelar o `pre`, e depois de escrever o `pre` passaria a ser a
        // malha já penteada. Ele é idempotente (carimbo por época), logo
        // capturar a pegada inteira — e não só quem de facto anda — é barato e
        // é o que garante que nenhum movido fica de fora.
        for &v in &alvos {
            self.capture(mesh, v);
        }

        let movidos = ph2d_rake::pentear(
            mesh.positions_mut(),
            &alvos,
            &normais,
            &pesos,
            &inicio,
            &anel,
            direccao,
        );
        if movidos == 0 {
            self.moved = alvos;
            return 0;
        }
        // ⚠️ **A região é a PEGADA e não só os movidos**, pela mesma razão que o
        // `last_refreshed` existe: mover um vértice muda a normal de todo vizinho
        // que partilha uma face com ele.
        mesh.refresh_region(&alvos, &mut self.region);
        self.moved = alvos;
        movidos
    }
}
