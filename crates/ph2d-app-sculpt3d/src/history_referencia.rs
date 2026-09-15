//! ⭐⭐⭐ **A SUPERFÍCIE DE REFERÊNCIA DA MULTIRESOLUÇÃO** — o que o pen-down
//! fotografa para os dois pincéis que medem deslocamento
//! (`SPEC_unblocked_brushes.md` §2), e a recusa em voz alta quando ela não
//! existe.
//!
//! ⚠️ **O corte é de RESPONSABILIDADE e não de tamanho:** o [`super::history`]
//! responde *o que um gesto guarda para o `Ctrl+Z`*; isto responde *contra que
//! superfície um deslocamento se mede*. As duas moravam juntas por só uma delas
//! ter nascido primeiro — e a segunda tem espec, gate e um pincel próprios.

use super::Sculpt3dScene;

impl Sculpt3dScene {
    /// ⭐⭐⭐ **A SUPERFÍCIE DE REFERÊNCIA do pincel de apagar deslocamento**,
    /// fotografada no pen-down (`SPEC_unblocked_brushes.md` §2).
    ///
    /// Por vértice do nível de cima, o ponto da **superfície-limite** da
    /// subdivisão do nível de baixo.
    ///
    /// ⭐⭐ **A equivalência que torna isto `O(anel)` e não uma avaliação de
    /// superfície:** subdividir **não muda** a superfície-limite, logo o limite
    /// do vértice `v` de `subdivide(nível_de_baixo)` é um ponto da
    /// superfície-limite do nível de baixo, na posição paramétrica de `v`. ⇒ a
    /// máscara de vértice do [`ph2d_mesh::limit_point`] chega — *não é preciso
    /// avaliar o limite em coordenadas arbitrárias*.
    ///
    /// ⛔⛔ **A REFERÊNCIA NÃO É A PREVISÃO**, e a diferença é a wave inteira
    /// (espec §2.1): `subdivide(base)` é a previsão de UM passo, e usá-la
    /// **encolheria a peça** a cada passagem. Medido no canto de um cubo: a
    /// previsão pousa em `0,2778` e o limite em `0,2500`.
    ///
    /// ⚠️ **O vértice sem limite publicado FICA ONDE ESTÁ** (anel misto
    /// tri/quad — ver [`ph2d_mesh::LimitPoint::None`]): a referência dele é a
    /// posição VIVA, logo o pincel não o move. *Pôr ali a previsão seria
    /// escolher uma superfície que não é a de esquema nenhum.*
    ///
    /// Devolve `None` quando não há nível de baixo — ali o deslocamento não
    /// existe, e quem diz isso em voz alta é o chamador.
    pub(crate) fn superficie_de_referencia(&self) -> Option<Vec<[f32; 3]>> {
        let stack = &self.objects[self.active].stack;
        let k = stack.level();
        let baixo = stack.level_mesh(k.checked_sub(1)?)?;
        let previsto = ph2d_mesh::subdivide(baixo);
        let topo = stack.mesh();
        // ⛔ **A contagem tem de bater**, e ela bate por construção (o `higher`
        // do multires faz exactamente este `subdivide`). Se não bater, a pilha
        // não descreve esta malha e a referência seria uma tabela desalinhada —
        // devolver `None` é a resposta certa.
        if previsto.vert_count() != topo.vert_count() {
            return None;
        }
        Some(
            (0..previsto.vert_count())
                .map(|v| match ph2d_mesh::limit_point(&previsto, v) {
                    ph2d_mesh::LimitPoint::At(q) => q,
                    ph2d_mesh::LimitPoint::None => topo.positions()[v],
                })
                .collect(),
        )
    }

    /// **Fotografa a referência que o gesto pede, e diz quando não há.**
    ///
    /// ⚠️ **Chamada no pen-down e só ali**, como a foto do desfazer ao lado: ela
    /// é função do nível de BAIXO, que o traço não toca.
    pub(crate) fn open_reference_stroke(&mut self) {
        if !self.brush.verb.precisa_de_referencia() {
            self.stroke.reference.clear();
            return;
        }
        match self.superficie_de_referencia() {
            Some(r) => self.stroke.reference = r,
            None => {
                self.stroke.reference.clear();
                // ⛔ **A RECUSA EM VOZ ALTA é o produto** (espec §4.3 e §5.6):
                // sem pilha o dado de entrada não existe, e *o irmão-filtro do
                // alvo estoirou publicamente por não verificar isto*.
                //
                // ⚠️ **Ela mudou-se para a porta única em 2026-09-15**
                // ([`crate::recusa`]): enquanto vivia aqui, ela era a resposta
                // de UM predicado e as dos irmãos não existiam — *uma razão
                // escrita ao lado do sítio que a descobre não é uma família, é
                // um caso*.
            }
        }
    }
}
