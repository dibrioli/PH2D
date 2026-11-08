//! O que a aba das camadas lê antes de pintar ([`Quadro`]), o campo do nome em edição e as fotos.

use std::cell::Cell;

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::frame_radius;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_physics_ecs::{LayerMatrix, MAX_LAYERS};
use ph2d_tokens::{ROW_H_PX, Radius, Spacing, TypeToken};

use super::{
    CAMADA, EDITANDO, Geometria, LENTE, Lente, NOVA, Onde, abre, camada_aberta, comeca_a_renomear,
    frase_do_pe, pares, rotulo, titulo_da_camada,
};
use crate::ids as i;

/// A célula sob o ponteiro (o despacho põe-na em `Hovered` pelo índice de cliques, em ecrã).
fn pairada(ctx: &PaintCtx, visiveis: &[usize]) -> Option<(usize, usize)> {
    let store = ctx.host.store();
    visiveis
        .iter()
        .enumerate()
        .flat_map(|(p, &a)| visiveis[..=p].iter().map(move |&b| (a, b)))
        .find(|&(a, b)| {
            let id = i::PHYSICS_LAYER_CELL[i::physics_layer_cell_index(a, b)];
            matches!(
                store.button_state(id),
                Some(ButtonState::Hovered | ButtonState::Pressed)
            )
        })
}

/// As fotos do smoke (`PH2D_PHYSICS_SMOKE_ABA`; a tela virtual não recebe rato): `abre` dá os nomes da
/// maquete às camadas 0..4 e abre a aba na 1; `pairar` põe também o ponteiro em (4, 1); `nome` abre o
/// nome da 2 em edição na lista; `default` tira o nome à 0 (lê-se «Default»).
pub(super) fn fotos(ctx: &mut PaintCtx) {
    thread_local! { static FEITO: Cell<bool> = const { Cell::new(false) }; }
    let Ok(caso) = std::env::var("PH2D_PHYSICS_SMOKE_ABA") else {
        return;
    };
    if FEITO.get() || crate::state::current() == crate::state::PhysicsSnapshot::default() {
        return;
    }
    FEITO.set(true);
    abre(Some(1));
    // `remove`: a Enemy (2) aberta, com a pergunta do «Remove» (destino Ground).
    if caso == "remove" {
        abre(Some(2));
        super::pergunta_o_destino(Some(0));
    }
    // `default`: a camada 0 perde o nome (lê-se «Default»); os outros casos dão os nomes da maquete B.
    let nomes: Vec<&str> = if caso == "default" {
        vec![""]
    } else {
        tr("card.physics.smoke_layer_names").split('|').collect()
    };
    for (n, nome) in nomes.into_iter().enumerate() {
        ctx.host
            .bus_mut()
            .push(ph2d_editor_core::action_bus::EditorAction::LayerRename {
                layer: n as u8,
                name: nome.to_owned(),
            });
    }
    // `muitas`: 27 camadas (0..26), a 7 com um nome de 40 letras, o ponteiro numa célula baixa (26, 2).
    if caso == "muitas" {
        for layer in 5..27u8 {
            let bus = ctx.host.bus_mut();
            bus.push(ph2d_editor_core::action_bus::EditorAction::LayerShow { layer });
        }
        let name = tr("card.physics.smoke_long_name").to_owned();
        let bus = ctx.host.bus_mut();
        bus.push(ph2d_editor_core::action_bus::EditorAction::LayerRename { layer: 7, name });
        // A grade rola até ao fim, para a célula do ponteiro (26, 2) estar à vista.
        super::ROLAGEM.set((usize::from(u8::MAX), 0));
    }
    if caso == "nome" {
        let campo = i::PHYSICS_LAYER_NAME[2];
        let store = ctx.host.store_mut();
        if let Some(InteractiveState::TextInput { text, caret, .. }) = store.get_mut(campo) {
            tr("card.physics.smoke_layer_rename").clone_into(text);
            *caret = text.len();
        }
        ph2d_editor_core::interaction::dispatch::focus_text_field(store, campo);
        comeca_a_renomear(2, Onde::Lista);
    }
}

pub(super) fn pairar_da_foto(ctx: &mut PaintCtx) {
    let (a, b) = match std::env::var("PH2D_PHYSICS_SMOKE_ABA").as_deref() {
        Ok("pairar") => (4, 1),
        Ok("muitas") => (26, 2),
        _ => return,
    };
    let id = i::PHYSICS_LAYER_CELL[i::physics_layer_cell_index(a, b)];
    if let Some(InteractiveState::Button { state, .. }) = ctx.host.store_mut().get_mut(id) {
        *state = ButtonState::Hovered;
    }
}

/// O nome em edição: (camada, id, texto, caret, (âncora, estado), onde se pinta).
pub(super) type Campo = (
    usize,
    ph2d_a11y::NodeId,
    String,
    usize,
    (Option<usize>, ph2d_editor_core::widget::TextInputState),
    Rect,
);

fn campo_em_edicao(
    ctx: &PaintCtx,
    editando: Option<(usize, Onde)>,
    visiveis: &[usize],
    g: &Geometria,
    lista_w: f32,
) -> Option<Campo> {
    editando.map(|(n, onde)| {
        let id = i::PHYSICS_LAYER_NAME[n];
        let store = ctx.host.store();
        let (texto, caret, anchor, estado) = match store.get(id) {
            Some(InteractiveState::TextInput {
                text,
                caret,
                selection_anchor,
                state,
            }) => (text.clone(), *caret, *selection_anchor, *state),
            _ => (String::new(), 0, None, Default::default()),
        };
        let p = visiveis.iter().position(|&v| v == n).unwrap_or(0);
        let r = match onde {
            Onde::Lista => g.linha_da_lista(p),
            // ⭐ O campo da grade tem a largura da lista: «Enemy» já não vira «nemy».
            Onde::Grade => {
                let r = g.rotulo_da_linha(p);
                Rect::new(r.x, r.y, r.w.max(lista_w), r.h)
            }
        };
        (n, id, texto, caret, (anchor, estado), r)
    })
}

/// As dicas dos rótulos e do «+ Add» (as células não têm: a frase do pé diz o par).
fn dicas(ctx: &mut PaintCtx, visiveis: &[usize], cheia: bool) {
    {
        let nomes = crate::state::layer_names();
        let store = ctx.host.store_mut();
        for &a in visiveis {
            let dica = super::dica_do_rotulo(&nomes, a);
            // A frase do pé diz o par: a célula não leva dica flutuante.
            for id in [
                i::PHYSICS_LAYER_ROW_LABEL[a],
                i::PHYSICS_LAYER_COL_LABEL[a],
                i::PHYSICS_LAYER_LIST[a],
            ] {
                if store.tooltip_for(id) != Some(dica.as_str()) {
                    store.set_tooltip(id, dica.clone());
                }
            }
        }
        let dica = if cheia {
            tr("card.physics.layer_add_full")
        } else {
            tr("card.physics.tab_add_tip")
        };
        if store.tooltip_for(i::PHYSICS_LAYER_ADD) != Some(dica) {
            store.set_tooltip(i::PHYSICS_LAYER_ADD, dica);
        }
        let dica = if super::camada_aberta() == 0 {
            tr("card.physics.tab_remove_default_tip")
        } else {
            tr("card.physics.tab_remove_tip")
        };
        if store.tooltip_for(i::PHYSICS_LAYER_REMOVE) != Some(dica) {
            store.set_tooltip(i::PHYSICS_LAYER_REMOVE, dica);
        }
    }
}

/// Tudo o que a aba lê antes de pintar (o store e as medidas do texto), uma vez por quadro.
pub(super) struct Quadro {
    pub(super) area: Rect,
    pub(super) g: Geometria,
    pub(super) visiveis: Vec<usize>,
    pub(super) rotulos: Vec<String>,
    pub(super) contagens: Vec<String>,
    pub(super) aberta_n: usize,
    pub(super) sob: Option<(usize, usize)>,
    pub(super) editando: Option<(usize, Onde)>,
    pub(super) font: f32,
    pub(super) raio: f32,
    pub(super) contagem_w: f32,
    pub(super) cabeca_dir_w: f32,
    pub(super) pares_w: f32,
    pub(super) add_w: f32,
    pub(super) add_aceso: bool,
    pub(super) matriz: LayerMatrix,
    pub(super) add_rotulo: &'static str,
    pub(super) pe: String,
    pub(super) par: Option<String>,
    pub(super) titulo: String,
    pub(super) pares_da_camada: String,
    pub(super) cabeca_dir: String,
    pub(super) campo: Option<Campo>,
    pub(super) camada: super::camada::Camada,
    pub(super) pergunta: Option<super::remove::Pergunta>,
    pub(super) remove_w: f32,
    pub(super) remove_aceso: bool,
}

impl Quadro {
    pub(super) fn de(ctx: &mut PaintCtx, area: Rect) -> Self {
        let theme = ctx.host.theme();
        let nomes = crate::state::layer_names();
        let visiveis = crate::state::visible_layers();
        if !visiveis.contains(&camada_aberta()) {
            CAMADA.set(visiveis.first().copied().unwrap_or(0));
        }
        let rotulos: Vec<String> = (0..MAX_LAYERS).map(|n| rotulo(&nomes, n)).collect();
        let font = TypeToken::Sm.px();
        let largura = |ctx: &mut PaintCtx, s: &str| ctx.text_system.prefix_width(s, font);
        let rotulo_w = visiveis
            .iter()
            .map(|&n| largura(ctx, &rotulos[n]))
            .fold(0.0_f32, f32::max)
            + Spacing::Md.px();
        let add_rotulo = tr("card.physics.tab_add");
        let botao_w = largura(ctx, add_rotulo).max(largura(ctx, tr("card.physics.tab_remove")))
            + Spacing::Lg.px() * 2.0;
        let contagem_w = largura(ctx, &MAX_LAYERS.to_string().repeat(2));
        let lista_w = (rotulo_w + contagem_w + Spacing::Lg.px() * 2.0)
            .max(largura(ctx, tr("card.physics.tab_layers")) + Spacing::Lg.px() * 2.0)
            .max(ROW_H_PX * 6.0) // LITERAL-PX-OK: a lista tem pelo menos seis linhas de largura
            // «+ Add layer» e «Remove» com a mesma largura, lado a lado dentro do cartão (onda 9e); se a aba
            // não deixa (a lista passaria de um terço dela), um debaixo do outro.
            .max({
                let lado_a_lado = Geometria::lista_para_os_botoes(botao_w);
                let empilhados = botao_w + Geometria::MARGEM_DA_LISTA * 2.0;
                if lado_a_lado <= area.w / 3.0 {
                    lado_a_lado
                } else {
                    empilhados
                } // LITERAL-PX-OK: um terço da aba
            });
        let aberta_n = camada_aberta();
        let nomes_dos_objetos: Vec<String> = crate::state::layer_objects(aberta_n)
            .into_iter()
            .map(|(_, n)| n)
            .collect();
        let camada = super::camada::Camada::monta(
            Geometria::largura_direita(area, lista_w),
            &visiveis,
            &rotulos,
            &nomes_dos_objetos,
            super::MAIS.get(),
            &mut |s| ctx.text_system.prefix_width(s, font),
        );
        let pergunta = super::removendo().map(|destino| {
            super::remove::Pergunta::monta(
                Geometria::largura_direita(area, lista_w),
                aberta_n,
                &visiveis,
                &rotulos,
                &nomes_dos_objetos,
                destino,
                &mut |s| ctx.text_system.prefix_width(s, font),
            )
        });
        let altura = pergunta.as_ref().map_or(camada.altura, |p| p.altura);
        let g = Geometria::de(area, rotulo_w, lista_w, visiveis.len(), altura)
            .com_botoes(botao_w)
            .rolada(super::ROLAGEM.get());
        super::GEOMETRIA.set(Some(g));
        let camada = camada.desloca(g.camada.x, g.camada.y);
        let pergunta = pergunta.map(|p| p.desloca(g.camada.x, g.camada.y));
        let remove_w = g.botao_w;
        let remove_aceso =
            ctx.host.store().button_state(i::PHYSICS_LAYER_REMOVE) == Some(ButtonState::Hovered);
        let matriz = LayerMatrix::from_rows(crate::state::current().vivo.layer_matrix);
        let sob = pairada(ctx, &visiveis);
        let editando = EDITANDO.get();
        let raio = frame_radius(theme, Radius::Sm.px());
        let par = super::par_do_pe(sob, &nomes);
        let pe = frase_do_pe(sob, &nomes, &matriz, &visiveis, NOVA.get());
        let (titulo, pares_da_camada) = titulo_da_camada(
            aberta_n,
            &nomes,
            &matriz,
            &visiveis,
            crate::state::layer_count(aberta_n),
        );
        let n_pares = pares(&visiveis, &matriz).0;
        let cabeca_dir = tr_with("card.physics.tab_pairs", &[("n", &n_pares.to_string())]);
        let cabeca_dir_w = largura(ctx, &cabeca_dir);
        let pares_w = largura(ctx, &pares_da_camada);
        let add_w = g.botao_w;
        let cheia = visiveis.len() >= MAX_LAYERS;
        let add_aceso =
            ctx.host.store().button_state(i::PHYSICS_LAYER_ADD) == Some(ButtonState::Hovered);

        // O campo em edição lê o store antes de a cena ser emprestada.
        let campo = campo_em_edicao(ctx, editando, &visiveis, &g, lista_w);
        dicas(ctx, &visiveis, cheia);
        let contagens: Vec<String> = visiveis
            .iter()
            .map(|&n| crate::state::layer_count(n).to_string())
            .collect();
        Self {
            area,
            g,
            visiveis,
            rotulos,
            contagens,
            aberta_n,
            sob,
            editando,
            font,
            raio,
            contagem_w,
            cabeca_dir_w,
            pares_w,
            add_w,
            add_aceso,
            matriz,
            add_rotulo,
            pe,
            par,
            titulo,
            pares_da_camada,
            cabeca_dir,
            campo,
            camada,
            pergunta,
            remove_w,
            remove_aceso,
        }
    }

    pub(super) fn linha_acesa(&self, n: usize) -> bool {
        self.sob.map(|s| s.0) == Some(n) || (self.sob.is_none() && self.aberta_n == n)
    }

    pub(super) fn coluna_acesa(&self, n: usize) -> bool {
        self.sob.map(|s| s.1) == Some(n)
    }

    /// Escreve a [`Lente`].
    pub(super) fn lente(&self) {
        let (visiveis, rotulos, pe, titulo, pares_da_camada) = (
            &self.visiveis,
            &self.rotulos,
            &self.pe,
            &self.titulo,
            &self.pares_da_camada,
        );
        let linha_acesa = |n| self.linha_acesa(n);
        let coluna_acesa = |n| self.coluna_acesa(n);
        LENTE.with(|l| {
            *l.borrow_mut() = Lente {
                pe: pe.clone(),
                linhas: visiveis
                    .iter()
                    .copied()
                    .filter(|&n| linha_acesa(n))
                    .collect(),
                colunas: visiveis
                    .iter()
                    .copied()
                    .filter(|&n| coluna_acesa(n))
                    .collect(),
                lista: visiveis
                    .iter()
                    .map(|&n| (n, rotulos[n].clone(), crate::state::layer_count(n)))
                    .collect(),
                titulo: (titulo.clone(), pares_da_camada.clone()),
                frase_da_camada: (visiveis.len() <= 1)
                    .then(|| tr("card.physics.tab_only_default").to_owned()),
            };
        });
    }
}
