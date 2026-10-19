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
//!   carregado o nome que o Ctrl+clique alcança sublinha (o `set`, não o `ampiltude`); Ctrl+Shift+M
//!   abre a lista de problemas (o erro primeiro); o clique e o F8 seleccionam o culpado; um `ph2d.set`
//!   escolhido dentro do `entity` de outro funde os buracos.

use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_script::{LuauScript, ProjectScripts, ScriptEditRequests, ScriptId, ScriptValue};
use ph2d_vec_entities::entities::{VecEntityMap, sync};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecScene, ellipse, rectangle, regular_polygon, star};

/// Quantas cenas este roteador serve — o `max_level` do roteador da família (conferido em teste).
pub const CENAS: u32 = 4;

/// O nome do script das formas nos scripts do projeto.
pub const BOB_NAME: &str = "bob.luau";
/// O nome do script partido do nível 2.
pub const BROKEN_NAME: &str = "script_quebrado.luau";

/// ⭐ **O script das três formas** — os comentários são para o DONO e dizem o que mexer.
pub const BOB_LUAU: &str = r#"-- bob.luau — sobe e desce. As TRÊS formas da esquerda correm ESTE script.
--
-- Experimente: troque o 1.0 da amplitude por 2 e carregue Ctrl+S.
-- As três passam a subir mais, na hora — a cena continua a tocar.
-- O quadrado da direita NÃO usa script: ele fica parado.

ph2d.property("amplitude", 1.0, { min = 0, max = 4 }) -- quanto sobe (metros)
ph2d.property("speed", 2.0, { min = 0, max = 10 }) -- quão depressa

function init(self)
  self.base_x = ph2d.get(self.id, "x")
  self.base_y = ph2d.get(self.id, "y")
  self.t = 0
end

function update(self, dt)
  self.t = self.t + dt
  -- a posição em x desencontra as três: uma onda, não um coro
  local s = math.sin(self.t * self.speed + self.base_x)
  ph2d.set(self.id, "y", self.base_y + s * self.amplitude)
end
"#;

/// O nome do script do nível 3.
pub const ONDA_NAME: &str = "onda.luau";

/// O valor PRÓPRIO da `amplitude` da estrela no nível 3 — o que o Inspector mostra e o rename leva.
pub const STAR_AMPLITUDE: f64 = 2.0;

/// ⭐ **O script do nível 3** — o erro `ampiltude` é de propósito, e os comentários são o roteiro do
/// dono. As agulhas do condutor das fotos (`ph2d_app_code::smoke_drive::STEPS`) vivem neste texto
/// (conferido no teste da cena).
pub const ONDA_LUAU: &str = r#"-- onda.luau — as TRÊS formas da esquerda correm ESTE script, e ele tem um erro:
-- por isso estão paradas. O quadrado da direita não tem script.
--
-- 1) «ampiltude», lá em baixo, está sublinhada: o certo é «amplitude».
--    Cursor nela, Alt+Enter, Enter, Ctrl+S: as três sobem e descem.
-- 2) Na linha marcada com 2), escreva ph2d. e escolha set na lista.
-- 3) F2 sobre «amplitude» muda o nome; a estrela leva o valor dela (2).

ph2d.property("amplitude", 1.0, { min = 0, max = 4 }) -- quanto sobe (metros)
ph2d.property("speed", 2.0, { min = 0, max = 10 }) -- quão depressa

function init(self)
  self.base_x = ph2d.get(self.id, "x")
  self.base_y = ph2d.get(self.id, "y")
  self.t = 0
end

function update(self, dt)
  self.t = self.t + dt
  local s = math.sin(self.t * self.speed + self.base_x)
  ph2d.set(self.id, "y", self.base_y + s * self.ampiltude)
  -- 2) escreva aqui, numa linha nova
end
"#;

/// O nome do script do nível 4.
pub const PROBLEMAS_NAME: &str = "problemas.luau";

/// ⭐ **O script do nível 4** — um erro (`math.sinn`) entre dois avisos (o `local` que ninguém lê,
/// ANTES dele, e o `self.ampiltude`, depois): a lista ordena por gravidade, o erro sobe e o F8 a
/// seguir a ele vai para cima. As agulhas do condutor das fotos
/// (`ph2d_app_code::smoke_drive::STEPS_4`) vivem neste texto (conferido no teste da cena).
pub const PROBLEMAS_LUAU: &str = r#"-- problemas.luau — as TRÊS formas da esquerda correm ESTE script. Ele tem um erro e dois
-- avisos de propósito (por isso as formas estão paradas).
--
-- 1) Carregue Ctrl e passe o rato por cima de «set», lá em baixo: o nome sublinha e o
--    ponteiro vira uma mão (o Ctrl+clique leva à ajuda). Por cima de «ampiltude», não.
-- 2) Ctrl+Shift+M abre a lista de problemas: o erro primeiro, depois os avisos.
--    Clique numa linha da lista; F8 salta para o problema seguinte.
-- 3) Na linha marcada com 3), escreva ph2d. e escolha set; com «entity» escolhido,
--    outra vez ph2d. e set. O Tab passa pelos buracos de dentro e depois pelos de fora.

ph2d.property("amplitude", 1.0, { min = 0, max = 4 }) -- quanto sobe (metros)
ph2d.property("speed", 2.0, { min = 0, max = 10 }) -- quão depressa

function init(self)
  self.base_y = ph2d.get(self.id, "y")
  self.t = 0
end

function update(self, dt)
  local sobra = 3 -- aviso: ninguém a lê
  self.t = self.t + dt
  local s = math.sinn(self.t * self.speed) -- erro: é «sin»
  ph2d.set(self.id, "y", self.base_y + s * self.ampiltude) -- aviso: é «amplitude»
  -- 3) escreva aqui, numa linha nova
end
"#;

/// O script partido do smoke da Onda 0 — o mesmo ficheiro que o dono já viu no cartão.
pub const BROKEN_LUAU: &str =
    include_str!("../../../docs/CodeEditor/ferramentas/smoke/script_quebrado.luau");

/// `x` de cada forma: as três do script, o quadrado (controlo) e o triângulo do nível 2.
pub const SHAPE_X: [f64; 5] = [-5.0, -2.5, 0.0, 2.75, 5.25];
/// O `y` de repouso: acima do meio, para a onda de amplitude 2 caber entre a régua e a timeline
/// (medido na foto a 1930×1040).
pub const SHAPE_Y: f32 = 0.75;
/// O meio-tamanho de cada forma (metros).
pub const SHAPE_R: f64 = 0.8;

fn tinta(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

/// As formas, centradas na origem (a pose é o `Transform`, que o script mexe): `(nome, caminho)`.
fn formas(nivel: u32) -> Vec<(&'static str, VecPath)> {
    let r = SHAPE_R;
    let mut v = vec![
        ("Circle", tinta(ellipse([0.0, 0.0], r, r), [96, 168, 235])),
        (
            "Star",
            tinta(star([0.0, 0.0], r, r, 5, 0.45), [240, 196, 72]),
        ),
        (
            "Hexagon",
            tinta(regular_polygon([0.0, 0.0], r, r, 6), [150, 120, 230]),
        ),
        (
            "Square (no script)",
            tinta(rectangle([-r, -r], [r, r]), [130, 134, 146]),
        ),
    ];
    if nivel == 2 {
        v.push((
            "Triangle (broken script)",
            tinta(regular_polygon([0.0, 0.0], r, r, 3), [232, 96, 80]),
        ));
    }
    v
}

/// Monta o nível `nivel` nos três empréstimos que ele usa. Devolve as entidades das formas, pela
/// ordem de [`SHAPE_X`].
pub fn monta_em(
    sim: &mut SimWorld,
    cena: &mut VecScene,
    mapa: &mut VecEntityMap,
    nivel: u32,
) -> Vec<Entity> {
    let nivel = nivel.clamp(1, CENAS);
    let (bob, broken) = {
        let mut lib = sim.world_mut().get_resource_or_init::<ProjectScripts>();
        let bob = match nivel {
            3 => lib.insert(ONDA_NAME, ONDA_LUAU),
            4 => lib.insert(PROBLEMAS_NAME, PROBLEMAS_LUAU),
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
    if let Some(aberto) = broken.or((nivel >= 3).then_some(bob)) {
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
             sobre «set» sublinha; Ctrl+Shift+M abre a lista de problemas; F8 salta entre eles.",
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

#[cfg(test)]
#[path = "code_smoke_tests.rs"]
mod tests;
