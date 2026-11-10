//! ⭐⭐ **A cena do EDITOR DE SCRIPTS** (`PH2D_CODE_SMOKE`, CodeEditor E15). O roteador é da família
//! `ph2d-app-code` (`ph2d_app_code::smoke`); a cena mora aqui, ao lado da do script do artista, porque
//! precisa da cena vectorial (`SceneCtx`) — a shell compõe as duas.
//!
//! - **1** — TRÊS formas (círculo, estrela, hexágono) correm o MESMO `bob.luau` (sobem e descem em
//!   onda) e uma QUARTA (o quadrado) não tem script: fica parada — o controlo. A cena TOCA e nada está
//!   aberto: o dono aprende o caminho — escolher uma forma → cartão *Script* → «Edit script» — e
//!   muda a `amplitude` no texto; Ctrl+S e as três mudam na hora.
//! - **2** — o mesmo, mais um triângulo com o `script_quebrado.luau` do smoke da Onda 0 (o erro com o
//!   símbolo culpado), já aberto numa aba pelo MESMO pedido do «Edit script» (`ScriptEditRequests`).
//! - **3** — a inteligência (Onda 2, I15): as três formas correm o `onda.luau`, que tem um erro de
//!   digitação de propósito (`self.ampiltude`) — por isso estão PARADAS; a aba abre e a ESTRELA fica
//!   escolhida, com o seu valor próprio da `amplitude` (2) no Inspector. O dono corrige com Alt+Enter,
//!   escreve `ph2d.` e escolhe `set` (as formas giram) e renomeia a propriedade com F2 (o 2 da estrela
//!   vai com o nome). Cada passo tem a sua foto (`ph2d_app_code::smoke_drive`).
//! - **4** — o acabamento (Onda 2): as mesmas três formas correm o `problemas.luau`, com um erro
//!   (`math.sinn`) e dois avisos (um `local` que ninguém lê, antes dele; `self.ampiltude`). Com o Ctrl
//!   carregado o nome que o Ctrl+clique alcança sublinha (o `set`, não o `ampiltude`); o erro do
//!   Play abre sozinho a lista de problemas (o erro primeiro); o clique selecciona o culpado (parado no erro, o F8 é o Continue); um `ph2d.set`
//!   escolhido dentro do `entity` de outro funde os buracos.
//! - **5** — a página de ajuda (Onda 2, item 5): as mesmas três formas correm o `ajuda.luau` e
//!   balançam para os lados. O Ctrl+clique em `sin` abre a página `math` no membro; um link na
//!   mesma página só rola; Alt+← volta ao script; F12 em `ph2d.set` abre a `ph2d`; o `PropertyName`
//!   da assinatura leva à `types`; Ctrl+F conta; Ctrl+W fecha; o Ctrl sobre `local` não dá a mão.
//! - **6** — a API que funciona (Onda 2, item 6): um JOGO pequeno. O círculo corre o `jogo.luau`:
//!   as setas movem-no (`ph2d.key_down`), o espaço atira (`ph2d.spawn("Bullet")` + `attach_script`
//!   do `bala.luau`, que sobe e, com `ph2d.wait(1.5)`, se apaga), um clique na cena leva o quadrado
//!   sem script até ao rato (`mouse_position`, `find_by_name`, `ph2d.set` num objecto sem script). O
//!   rebobinar devolve tudo. Nenhuma aba abre: com uma aba de código activa o teclado é dela.
//!   `PH2D_CODE_SMOKE_DEMO=1` = o piloto das fotos ([`piloto`]), inerte sem a variável.
//! - **7** — arrastar objectos para o código (item 8, `docs/CodeEditor/07_arrastar_para_o_codigo.md`):
//!   o círculo «Ship» corre o `arrastar.luau`, já aberto numa aba; a porta «Door» e a moeda «Gold
//!   Coin» não têm script. Com o Ctrl, arrastar a «Door» para a linha vazia escreve
//!   `local door = ph2d.object("Door")` e a porta sobe e desce; sem o Ctrl, a «Gold Coin» dentro de um
//!   `ph2d.set(` faz a moeda girar. Renomear a «Door» na Hierarquia muda o script (um Ctrl+Z desfaz os
//!   dois); `ph2d.object("` sugere os nomes; um nome errado é um aviso; Ctrl+clique selecciona.
//! - **8** — as propriedades pelo nome (fase 8d): [`crate::code_smoke_props`].
//! - **9** — arrastar nomes do Inspector: [`crate::code_smoke_inspector`].
//! - **10** — um script que mexe no Kanban do projecto (`ph2d.kanban`): [`kanban`].
//! - **11** — um problema, um lugar (Onda A, doc 13 §4.1–2): as três formas correm o
//!   [`ONDA_A_LUAU`]: cada segundo cada uma imprime as moedas (o Console diz «Circle · 0:01»); aos 3 s
//!   o `self.sped` pára as três e os Problems mostram UMA linha com «×3» e, no «On», dois nomes e a
//!   conta, fundida com o aviso do texto da mesma linha.
//! - **13** — a Bancada (Onda B, doc 13 §4 itens 6–10): [`ph2d_code_scenes::bancada`].
//! - **12** — o depurador (Onda D, doc 13 §4): as três formas correm o [`DEPURA_LUAU`], aberto numa
//!   aba com um ponto de parada já posto (o condutor põe-no); o jogo pára no Circle, o Continue passa
//!   à Star; tirado o ponto, os valores ao vivo aparecem no fim das linhas.
//! - **14** — os quadros do depurador (plano 14): as três formas correm o [`DEPURA_LUAU`], parado em
//!   `self.t = …`; o Stopped em planilha (Sheet), o Next line acende `self.t` e `self.pos.x` em âmbar,
//!   e `View ▸ Card Style` troca para Cards e Strip.
//! - **15** — muitos problemas e muitos donos: [`ph2d_code_scenes::arena`].

pub use ph2d_code_scenes::*;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_script::{LuauScript, ProjectScripts, ScriptEditRequests, ScriptId, ScriptValue};
use ph2d_vec_entities::entities::{VecEntityMap, sync};
use ph2d_vec_scene::VecScene;

/// Monta o nível `nivel` nos três empréstimos que ele usa. Devolve as entidades das formas, pela
/// ordem de [`SHAPE_X`].
pub fn monta_em(
    sim: &mut SimWorld,
    cena: &mut VecScene,
    mapa: &mut VecEntityMap,
    nivel: u32,
) -> Vec<Entity> {
    let nivel = nivel.clamp(1, CENAS);
    if nivel == 6 {
        return monta_jogo(sim, cena, mapa);
    }
    if nivel == 7 {
        return monta_arrastar(sim, cena, mapa);
    }
    if nivel == 8 {
        return crate::code_smoke_props::monta(sim, cena, mapa);
    }
    if nivel == 10 {
        return kanban::monta(sim);
    }
    if nivel == 13 {
        return bancada::monta(sim, cena, mapa);
    }
    if nivel == 15 {
        return arena::monta(sim, cena, mapa);
    }
    let (bob, broken) = {
        let mut lib = sim.world_mut().get_resource_or_init::<ProjectScripts>();
        let bob = match nivel {
            3 => lib.insert(ONDA_NAME, ONDA_LUAU),
            4 => lib.insert(PROBLEMAS_NAME, PROBLEMAS_LUAU),
            5 => lib.insert(AJUDA_NAME, AJUDA_LUAU),
            11 => lib.insert(ONDA_A_NAME, ONDA_A_LUAU),
            12 | 14 => lib.insert(DEPURA_NAME, DEPURA_LUAU),
            _ => lib.insert(BOB_NAME, BOB_LUAU),
        };
        let broken = (nivel == 2).then(|| lib.insert(BROKEN_NAME, BROKEN_LUAU));
        (bob, broken)
    };
    let ids: Vec<_> = formas(nivel)
        .into_iter()
        .map(|(nome, p)| (nome, cena.push_path(p)))
        .collect();
    // ⚠️ O `sync` aqui e não no próximo quadro: é ele que dá entidade a cada caminho, e o script
    // pendura-se na entidade.
    sync(sim, cena, mapa);
    let script_of = |i: usize| -> Option<ScriptId> {
        match i {
            0..=2 => Some(bob),
            4 => broken,
            _ => None,
        }
    };
    let mut out = Vec::new();
    for (i, (nome, pid)) in ids.into_iter().enumerate() {
        let Some(&bits) = mapa.get(&pid) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        #[allow(clippy::cast_possible_truncation)]
        let t = Transform::from_translation(ph2d_core::Vec2::new(SHAPE_X[i] as f32, SHAPE_Y));
        let mut em = sim.world_mut().entity_mut(e);
        em.insert((Name::new(nome), t));
        if let Some(s) = script_of(i) {
            let mut script = LuauScript::library(s);
            if nivel == 3 && i == 1 {
                script
                    .own
                    .insert("amplitude".into(), ScriptValue::Number(STAR_AMPLITUDE));
            }
            em.insert(script);
        }
        out.push(e);
    }
    if let Some(aberto) = broken.or(((3..=5).contains(&nivel) || nivel >= 11).then_some(bob)) {
        sim.world_mut()
            .get_resource_or_init::<ScriptEditRequests>()
            .0
            .push(aberto);
    }
    out
}

/// A porta da shell: monta e anuncia. Devolve o objecto escolhido: nenhum nos níveis 1–2 (o dono
/// aprende a escolher), a ESTRELA no 3 (o Inspector mostra o valor próprio dela).
pub fn montar(cx: crate::scene_ctx::SceneCtx<'_>, nivel: u32) -> Option<u64> {
    let nivel = nivel.clamp(1, CENAS);
    if nivel == 8 {
        return crate::code_smoke_props::montar(cx);
    }
    if nivel == 9 {
        return crate::code_smoke_inspector::montar(cx);
    }
    if nivel == 10 {
        return kanban::montar(cx);
    }
    let formas = monta_em(cx.sim, cx.vec_scene, cx.vec_entities, nivel);
    if nivel == 3 {
        println!(
            "[code-smoke] =3 {} formas: o círculo, a estrela e o hexágono correm o onda.luau, já aberto \
             numa aba — e PARADAS: o script tem «ampiltude» em vez de «amplitude». A estrela fica \
             escolhida: o Inspector mostra a amplitude dela (2). O quadrado cinzento não tem script.",
            formas.len()
        );
        return formas.get(1).map(|e| e.to_bits());
    }
    if nivel == 4 {
        println!(
            "[code-smoke] =4 {} formas: o círculo, a estrela e o hexágono correm o problemas.luau, já \
             aberto numa aba — PARADAS: o script tem um erro («math.sinn») e dois avisos. Ctrl + rato \
             sobre «set» sublinha; a lista de problemas abre-se sozinha; um clique numa linha leva ao problema.",
            formas.len()
        );
        return None;
    }
    if nivel == 6 {
        println!(
            "[code-smoke] =6 o JOGO: o círculo azul corre o jogo.luau. Setas = andar; Espaço = atirar \
             uma estrela (sobe e some em 1,5 s); clique na cena = o quadrado cinzento vai para o rato. \
             Rebobinar devolve tudo ao lugar. O console (Edit script no cartão do círculo) conta os tiros."
        );
        return formas.first().map(|e| e.to_bits());
    }
    if nivel == 7 {
        println!(
            "[code-smoke] =7 arrastar para o código: o círculo «Ship» corre o arrastar.luau, já aberto \
             numa aba; «Door» e «Gold Coin» não têm script. Ctrl + arrastar a «Door» da Hierarquia para \
             a linha vazia do update: a porta sobe e desce (Ctrl+S, aba «Cena»)."
        );
        return None;
    }
    if nivel == 11 {
        println!(
            "[code-smoke] =11 {} formas: o círculo, a estrela e o hexágono correm o onda_a.luau, já \
             aberto numa aba. Play: cada segundo o Console mostra «coins: N» com «Circle · 0:01»; aos \
             3 s as três param e os Problems abrem com UMA linha: «… — ×3 in Play», On «Circle, \
             Star +1».",
            formas.len()
        );
        return None;
    }
    if nivel == 12 {
        println!(
            "[code-smoke] =12 {} formas: o círculo, a estrela e o hexágono correm o depura.luau, já \
             aberto numa aba com um ponto vermelho na linha «local swing». O jogo pára no Circle: \
             o cartão «Stopped» mostra as variáveis; Continue passa à Star; clique no ponto para o \
             tirar e Continue: os valores aparecem no fim das linhas.",
            formas.len()
        );
        return None;
    }
    if nivel == 14 {
        println!(
            "[code-smoke] =14 os quadros do depurador: as três formas correm o depura.luau, parado no \
             Circle na linha «self.t = …». O quadro Stopped é uma planilha (Name · Value · Type); Next \
             line acende em âmbar o que mudou; View ▸ Card Style troca para Cards e Strip."
        );
        return None;
    }
    if nivel == 15 {
        println!(
            "[code-smoke] =15 oito formas correm o arena.luau (sete propriedades, seis com valor \
             próprio em âmbar na grelha Who has what) e o script tem uns dez problemas na lista."
        );
        return None;
    }
    if nivel == 13 {
        println!(
            "[code-smoke] =13 a Bancada: Hero e Hero 2 correm o player.luau (o Hero 2 com speed 180), \
             cinco moedas o coin.luau, e o door.luau não é de ninguém. A aba do player.luau abre: In \
             this script, as marcas da margem, a grelha Who has what com o mapa, e a frase do pé."
        );
        return formas.get(1).map(|e| e.to_bits());
    }
    if nivel == 5 {
        println!(
            "[code-smoke] =5 {} formas: o círculo, a estrela e o hexágono correm o ajuda.luau, já \
             aberto numa aba, e balançam para os lados. Ctrl+clique em «sin» abre a página de ajuda \
             «math»; F12 em «set» abre a «ph2d»; Alt+← volta; Ctrl+W fecha a página.",
            formas.len()
        );
        return None;
    }
    println!(
        "[code-smoke] ={nivel} {} formas: o círculo, a estrela e o hexágono correm o MESMO \
         bob.luau (sobem e descem em onda); o quadrado cinzento não tem script e fica parado.{}",
        formas.len(),
        if nivel >= 2 {
            " O triângulo vermelho corre o script_quebrado.luau, já aberto numa aba: a linha 3 \
             marcada na calha e o «=» a mais sublinhado."
        } else {
            " Nada está aberto: escolha uma forma, desça ao cartão Script e carregue em «Edit script»."
        }
    );
    None
}

#[path = "code_smoke_kanban.rs"]
pub mod kanban;

#[cfg(test)]
#[path = "code_smoke_tests.rs"]
mod tests;
