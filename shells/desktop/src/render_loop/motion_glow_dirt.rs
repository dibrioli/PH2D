//! **A MÁSCARA DE SUJIDADE DO HALO, resolvida contra a cena** — a metade de shell da última
//! célula P2 da folha 11 (doc 89).
//!
//! O nó guarda um NOME (`fx.glow`'s `dirt` text param, um `ParamWidget::Source`); o passe de
//! tela precisa de uma `TextureView` e do aspecto da imagem. Este arquivo é a costura entre os
//! dois, e ele é **uma função** de propósito: a shell é o único sítio do app onde a cena, o
//! atlas e as duas lojas de textura estão em mão ao mesmo tempo.
//!
//! ## As três fontes, e por que elas não custam nada aqui
//!
//! A célula precificava *"resolver as TRÊS fontes (`Atlas` / `Individual` / `CookedTexture`)
//! até ao passe de tela"* e avisava que cobrir só a primeira daria *"uma feature que funciona
//! com umas imagens e falha em silêncio com outras"*. O aviso continua certo; o preço não:
//! [`sprite_appearance`](super::motion_bridge_objects_appearance) já responde às três por uma
//! porta só, com gate, desde que a folha 14 dissolveu a mesma cerca do outro lado. O que faltava
//! era o passo a jusante — um `texture_id` é o que o passe de SPRITES consome, e um passe de
//! TELA quer a view —, e ele é hoje
//! [`SpriteRenderer::texture_view_and_dims`](ph2d_render::SpriteRenderer::texture_view_and_dims).
//!
//! ⚠️ **O ASPECTO sai do sub-rect, não da textura.** Uma célula de atlas vive numa textura
//! partilhada quadrada, então `w/h` da view é `1` para toda sprite empacotada — usá-lo esticaria
//! qualquer máscara que viesse do atlas, e o defeito leria como *"a minha sujidade só fica certa
//! se eu importar a imagem sozinha"*, que é a falha-por-fonte que a célula nomeou.

use ph2d_ecs::{Name, SimWorld};
use ph2d_render::{DirtMask, Sprite, SpriteRenderer};

use super::motion_bridge::{Appearance, sprite_appearance};

/// O que a shell consegue dizer sobre a máscara **sem** o renderer emprestado — o par que a
/// resolução por NOME produz, e a entrada do passo que precisa de uma view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Resolved {
    pub(super) texture_id: u32,
    pub(super) uv_rect: [f32; 4],
}

/// A sprite NOMEADA `name`, resolvida à aparência dela. `None` quando o nome não existe na cena
/// (ou não é uma sprite, ou a textura ainda não carregou).
///
/// ⚠️ **A comparação de nome é a MESMA do publicador de objectos** — igualdade exacta sobre o
/// `Name`, sem trim do lado da cena. O trim acontece no NÓ (o campo de texto do painel), e é lá
/// que ele pertence: aparar aqui também faria dois objectos chamados `"Dirt"` e `"Dirt "`
/// colidirem só nesta feature.
pub(super) fn resolve(sim: &mut SimWorld, look: Appearance<'_>, name: &str) -> Option<Resolved> {
    // O mesmo par `(&Sprite, &Name)` que o publicador de objectos percorre — `world_mut()`
    // constrói o `QueryState` (que fica em cache no mundo) e a iteração é só leitura.
    let mut q = sim.world_mut().query::<(&Sprite, &Name)>();
    let world = sim.world();
    let found = q.iter(world).find(|(_, n)| n.0 == name).map(|(s, _)| *s)?;
    let (uv_rect, texture_id) = sprite_appearance(&found, look)?;
    Some(Resolved {
        uv_rect,
        texture_id,
    })
}

/// A máscara pronta para o passe — a view e o aspecto da IMAGEM (não da textura que a contém).
///
/// ⚠️ **A chave é o `texture_id` e o sub-rect JUNTOS**, não só o id: duas células diferentes do
/// atlas partilham o id `0`, então uma chave que fosse só o id não veria a troca entre duas
/// sprites empacotadas e o artista escolheria outra imagem sem nada mudar na tela. Os dois
/// cabem num `u64` porque o que distingue as células é o rect, e um hash dele basta para
/// *"mudou?"* — a chave nunca é lida como endereço de nada.
pub(super) fn mask<'a>(r: Resolved, renderer: &'a SpriteRenderer) -> Option<DirtMask<'a>> {
    let (view, w, h) = renderer.texture_view_and_dims(r.texture_id)?;
    Some(DirtMask {
        view,
        key: key_of(r),
        uv_rect: r.uv_rect,
        aspect: image_aspect(r.uv_rect, w, h),
    })
}

/// **O aspecto da IMAGEM, não o da textura que a contém** — a metade da lei que se mede sem
/// uma GPU, e a única em que as três fontes se distinguem.
///
/// ⚠️ **É aqui que "funciona com umas imagens e falha em silêncio com outras" mora.** Uma
/// `Individual` e uma `CookedTexture` são delas próprias (`uv_rect = [0,0,1,1]`) e o aspecto da
/// textura É o da imagem; uma célula de ATLAS vive numa textura partilhada QUADRADA, cujo `w/h`
/// é `1` para toda sprite empacotada. Ler o da textura daria a máscara certa nas duas primeiras
/// fontes e uma máscara esticada na terceira — o modo de falha por-fonte que a célula nomeou,
/// com as duas rotas indistinguíveis num teste que só usasse imagens individuais.
///
/// ⚠️ **O `uv_rect` é `[u0, v0, u1, v1]` — os dois CANTOS**, que é o que
/// [`ph2d_render::AtlasRegion::uv`] devolve. Lê-lo como `[x, y, w, h]` foi o defeito da 1.ª
/// versão desta feature, e ele é invisível em duas das três fontes: `Individual` e
/// `CookedTexture` devolvem `[0, 0, 1, 1]`, que se lê **igual** nas duas convenções. Só a célula
/// de atlas as separa.
///
/// Um rect degenerado devolve `1` (quadrado), que o enquadramento trata sem `NaN`.
#[must_use]
fn image_aspect(uv_rect: [f32; 4], tex_w: u32, tex_h: u32) -> f32 {
    let [u0, v0, u1, v1] = uv_rect;
    let (iw, ih) = ((u1 - u0) * tex_w as f32, (v1 - v0) * tex_h as f32);
    if ih > 0.0 && iw > 0.0 && (iw / ih).is_finite() {
        iw / ih
    } else {
        1.0
    }
}

/// A identidade estável de uma escolha — ver [`mask`].
fn key_of(r: Resolved) -> u64 {
    let mut h = 1469598103934665603_u64; // FNV-1a offset basis
    let mut eat = |b: u32| {
        h ^= u64::from(b);
        h = h.wrapping_mul(1099511628211);
    };
    eat(r.texture_id);
    for v in r.uv_rect {
        eat(v.to_bits());
    }
    h
}

/// **O SEXTO motivo de o halo não mudar** (`PH2D_GLOW_DIAG=1`) — o nome está escrito e a cena
/// não tem ninguém com ele.
///
/// ⚠️ **Não é erro, e é por isso que ele precisa de voz.** Escrever o nome antes de criar a
/// sprite é legítimo (uma referência para a frente, como o `motion.path`), então a resposta certa
/// é *nada acontece* — que é indistinguível a olho de *escolhi a imagem errada*, de *a
/// intensidade está a zero* e das cinco causas que o irmão [`super::motion_glow_layer::diag`] já
/// enumera. Uma linha de texto separa-as.
///
/// ⚠️ **Só imprime na MUDANÇA do nome**, pela mesma razão que o irmão: um diagnóstico por quadro
/// afoga o terminal e o artista deixa de o ler.
pub(super) fn diag_unresolved(name: &str) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static LAST: AtomicU64 = AtomicU64::new(0);
    if std::env::var_os("PH2D_GLOW_DIAG").is_none() {
        return;
    }
    let mut h = 1469598103934665603_u64;
    for b in name.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(1099511628211);
    }
    if LAST.swap(h, Ordering::Relaxed) != h {
        eprintln!(
            "[glow] dirt: nenhuma sprite NOMEADA {name:?} na cena -- a mascara fica desligada"
        );
    }
}

#[cfg(test)]
#[path = "motion_glow_dirt_tests.rs"]
mod tests;
