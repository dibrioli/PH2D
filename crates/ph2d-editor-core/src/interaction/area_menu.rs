//! ⭐⭐⭐ **O QUE UM MÓDULO PÕE NA FILA** — um pulldown de área, com a face que ele mostra fechado.
//!
//! ⚠️ **Vive na `interaction/` e NÃO na `widget/`, e não é arrumação:** os dois portões de
//! `src/widget/` (a galeria e o HR-12) perguntam *«que semântica de acessibilidade este widget
//! liga?»* e *«onde é que a galeria o mostra?»*, e isto **não tem nenhuma das duas** — não pinta
//! nada. O chip que o mostra é um [`crate::widget::ToolRailEntry::Compound`], que a galeria já
//! cobre. *Dois opt-outs para o mesmo ficheiro é o sinal de que ele está no sítio errado.*
//!
//! ⚠️ E o vizinho certo é o [`super::types_menu`]: isto é *que menu a área contribui*, que é a
//! mesma pergunta que o [`super::ContextMenuKind`] responde do outro lado.

use crate::widget::ToolRailEntry;

/// ⭐⭐⭐ **UM PULLDOWN QUE A ÁREA CONTRIBUI PARA A FILA** — a metade 2 da **D2**.
///
/// O módulo que tem o canvas publica `N` destes por quadro
/// ([`crate::interaction::WidgetStore::set_area_commands`]); a fila desenha um chip por cada, na
/// posição `slot`, com o id [`crate::ids::area_menu_button`].
///
/// # ⚠️ Os TRÊS campos respondem a três perguntas diferentes, e trocá-los apaga a razão de existir
///
/// | campo | pergunta | exemplo |
/// |---|---|---|
/// | `label` | *o que é este grupo?* | `View` · `Gizmo` |
/// | `face` | ***qual é o estado AGORA?*** | `Front` · `Move` |
/// | `rows` | *o que ele abre* | as seis vistas + os três gestos de câmera |
///
/// ⭐ **A `face` é o que faz o chip valer o lugar FECHADO.** Um chip que diz sempre a mesma coisa
/// custa a mesma largura e não informa nada — e foi precisamente por isso que os agrupamentos são
/// por *leitura* e não por conveniência: dois grupos com a mesma face seriam um grupo só.
///
/// ⛔ **E é por isto que não há UM pulldown com tudo dentro:** juntar vista e gizmo dá 14 linhas
/// com uma face só — o depósito da foto 3, mudado de sítio (`docs/UI_New_and_Simple/`).
#[derive(Clone, Debug, Default)]
pub struct AreaMenu {
    /// A chave já **traduzida** — quem traduz é o módulo dono, que é quem conhece o vocabulário
    /// dele (HR-15: o `ph2d-editor-core` não pode ter o rótulo de um módulo escrito no corpo).
    pub label: String,
    /// A leitura do estado. Ver a tabela acima.
    pub face: String,
    /// Toda face que o pulldown PODE mostrar (já traduzidas) — a fila dá-lhe a largura da mais
    /// larga, para ele não mudar de tamanho quando o estado muda. Vazia ⇒ só a actual.
    pub faces: Vec<String>,
    /// O corpo, com os ids do **painel dono** — ver [`crate::interaction::ContextMenuKind::AreaCommands`].
    pub rows: Vec<ToolRailEntry>,
}

/// ⭐⭐ **A FILA dos pulldowns da área, com DOIS escritores** — o seletor de modo à frente
/// ([`crate::object_mode`], spec/06 F2) e os do módulo que tem o canvas atrás dele.
///
/// ⚠️ **Cada escritor só reescreve a SUA parte**, e por isso a ordem em que escrevem no quadro não
/// importa: o módulo publica em todo quadro (vazio incluído) e não sabe que o seletor existe, e o
/// seletor idem. ⛔ Duas listas lidas em sequência pelos leitores dariam dois `slot` para o mesmo
/// chip; aqui o `slot` continua a ser a POSIÇÃO numa lista só.
#[derive(Clone, Debug, Default)]
pub struct AreaMenus {
    list: Vec<AreaMenu>,
    /// Quantos da frente são do seletor de modo (0 ou 1).
    leading: usize,
}

impl AreaMenus {
    /// Os pulldowns, na ordem da fila.
    #[must_use]
    pub fn all(&self) -> &[AreaMenu] {
        &self.list
    }

    /// O 1.º pulldown (`slot` 0) é o seletor de modo?
    #[must_use]
    pub fn has_leading(&self) -> bool {
        self.leading == 1
    }

    /// Reescreve a parte do MÓDULO, e deixa a do seletor.
    pub fn set_module(&mut self, menus: Vec<AreaMenu>) {
        self.list.truncate(self.leading);
        self.list.extend(menus);
    }

    /// Reescreve a parte do SELETOR DE MODO, e deixa a do módulo.
    pub fn set_leading(&mut self, menu: Option<AreaMenu>) {
        if self.leading == 1 {
            self.list.remove(0);
        }
        self.leading = usize::from(menu.is_some());
        if let Some(m) = menu {
            self.list.insert(0, m);
        }
    }
}
