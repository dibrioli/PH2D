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
//! - **5** — a página de ajuda (Onda 2, item 5): as mesmas três formas correm o `ajuda.luau` e
//!   balançam para os lados. O Ctrl+clique em `sin` abre a página `math` no membro; um link na
//!   mesma página só rola; Alt+← volta ao script; F12 em `ph2d.set` abre a `ph2d`; o `PoseField` da
//!   assinatura leva à `types`; Ctrl+F conta; Ctrl+W fecha; o Ctrl sobre `local` não dá a mão.
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

use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_script::{LuauScript, ProjectScripts, ScriptEditRequests, ScriptId, ScriptValue};
use ph2d_vec_entities::entities::{VecEntityMap, sync};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecScene, ellipse, rectangle, regular_polygon, star};

/// Quantas cenas este roteador serve — o `max_level` do roteador da família (conferido em teste).
pub const CENAS: u32 = 7;

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

/// O nome do script do nível 5.
pub const AJUDA_NAME: &str = "ajuda.luau";

/// ⭐ **O script do nível 5** — a página de ajuda: `math.sin`, um pseudónimo (`m.cos`), um método
/// de texto (`s:upper()`), o `ph2d.set` e uma palavra da linguagem (`local`). Sem erros: as formas
/// mexem-se. As agulhas do condutor das fotos (`ph2d_app_code::smoke_drive::STEPS_5`) vivem neste
/// texto (conferido no teste da cena).
pub const AJUDA_LUAU: &str = r#"-- ajuda.luau — as TRÊS formas da esquerda correm ESTE script: balançam para os lados.
-- O quadrado da direita não tem script. Aqui aprende-se a PÁGINA DE AJUDA.
--
-- 1) Carregue Ctrl e passe o rato por cima de «sin», lá em baixo: o nome sublinha.
--    Ctrl+clique: a página «math» abre numa aba, já no «sin» (o que faz, o argumento, um exemplo).
-- 2) No texto da página, clique em «math.noise»: a página rola até ele, na mesma aba.
-- 3) Alt+← volta ao script, onde foi o Ctrl+clique (rolar dentro da página não conta).
-- 4) Cursor em «set», na linha do ph2d.set, e F12: a página «ph2d», no «set».
--    Na assinatura, clique em «PoseField»: a página «types». Alt+← volta ao «set».
-- 5) Ctrl+F na página e escreva entity: quantas vezes aparece. Ctrl+W fecha a página.
-- 6) Ctrl por cima de «local»: nada sublinha (uma palavra da linguagem não tem página).

ph2d.property("amplitude", 0.75, { min = 0, max = 2 }) -- quanto balança (metros)
ph2d.property("speed", 2.0, { min = 0, max = 10 }) -- quão depressa

local m = math
local s = "ajuda"

function init(self)
  self.base_x = ph2d.get(self.id, "x")
  self.t = 0
  print(s:upper())
end

function update(self, dt)
  self.t = self.t + dt
  local onda = math.sin(self.t * self.speed) + m.cos(self.t) * 0.2
  ph2d.set(self.id, "x", self.base_x + onda * self.amplitude)
end
"#;

pub const JOGO_NAME: &str = "jogo.luau";

/// ⭐ **O script do nível 6** — o círculo do jogo.
pub const JOGO_LUAU: &str = r#"-- jogo.luau — o CÍRCULO AZUL corre ESTE script. A cena já está a correr.
-- Setas: o círculo anda. Espaço: atira uma estrela (uma cópia do mestre «Bullet»), que sobe e
-- some passado 1,5 s. Clique na cena: o quadrado cinzento (que NÃO tem script) vai para lá.
-- Rebobinar (o botão de voltar ao início): tudo volta ao lugar e as estrelas somem.

ph2d.property("speed", 4, { min = 0, max = 10 }) -- quão depressa anda

function update(self, dt)
  local x = ph2d.get(self.id, "x") or 0
  local y = ph2d.get(self.id, "y") or 0
  if ph2d.key_down("left") then x = x - self.speed * dt end
  if ph2d.key_down("right") then x = x + self.speed * dt end
  if ph2d.key_down("up") then y = y + self.speed * dt end
  if ph2d.key_down("down") then y = y - self.speed * dt end
  ph2d.set(self.id, "x", x)
  ph2d.set(self.id, "y", y)

  if ph2d.key_pressed("space") then
    local estrela = ph2d.spawn("Bullet", x, y + 1)
    ph2d.attach_script(estrela, "bala.luau")
    self.tiros = (self.tiros or 0) + 1
    print("tiro", self.tiros)
  end

  if ph2d.mouse_pressed("left") then
    local mx, my = ph2d.mouse_position()
    local alvo = ph2d.find_by_name("Target")
    if alvo and mx and my then
      ph2d.set(alvo, "x", mx)
      ph2d.set(alvo, "y", my)
    end
  end
end
"#;

pub const BALA_NAME: &str = "bala.luau";

/// ⭐ O script de cada estrela atirada — sobe, espera e apaga-se.
pub const BALA_LUAU: &str = r#"-- bala.luau — cada estrela que o círculo atira corre ESTE script.
function init(self)
  ph2d.wait(1.5)
  ph2d.despawn(self.id)
end

function update(self, dt)
  ph2d.set(self.id, "y", (ph2d.get(self.id, "y") or 0) + 6 * dt)
end
"#;

/// Onde as peças do jogo nascem (metros): o círculo, o alvo e a receita.
pub const JOGO_AT: [[f32; 2]; 3] = [[-4.0, -1.0], [3.0, 0.75], [6.0, 2.5]];

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
    if nivel == 6 {
        return monta_jogo(sim, cena, mapa);
    }
    if nivel == 7 {
        return monta_arrastar(sim, cena, mapa);
    }
    let (bob, broken) = {
        let mut lib = sim.world_mut().get_resource_or_init::<ProjectScripts>();
        let bob = match nivel {
            3 => lib.insert(ONDA_NAME, ONDA_LUAU),
            4 => lib.insert(PROBLEMAS_NAME, PROBLEMAS_LUAU),
            5 => lib.insert(AJUDA_NAME, AJUDA_LUAU),
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
    if let Some(aberto) = broken.or((3..=5).contains(&nivel).then_some(bob)) {
        sim.world_mut()
            .get_resource_or_init::<ScriptEditRequests>()
            .0
            .push(aberto);
    }
    out
}

/// ⭐ **O nível 6**: o círculo com o `jogo.luau`, o quadrado «Target» sem script e o mestre
/// «Bullet» (uma estrela marcada como receita). Devolve `[círculo, alvo, mestre]`.
pub fn monta_jogo(sim: &mut SimWorld, cena: &mut VecScene, mapa: &mut VecEntityMap) -> Vec<Entity> {
    let jogo = {
        let mut lib = sim.world_mut().get_resource_or_init::<ProjectScripts>();
        lib.insert(BALA_NAME, BALA_LUAU);
        lib.insert(JOGO_NAME, JOGO_LUAU)
    };
    let r = SHAPE_R;
    let pecas = [
        ("Ship", tinta(ellipse([0.0, 0.0], r, r), [96, 168, 235])),
        (
            "Target",
            tinta(rectangle([-r, -r], [r, r]), [130, 134, 146]),
        ),
        (
            "Bullet",
            tinta(star([0.0, 0.0], r * 0.4, r * 0.4, 5, 0.45), [240, 196, 72]),
        ),
    ];
    let ids: Vec<_> = pecas
        .into_iter()
        .map(|(nome, p)| (nome, cena.push_path(p)))
        .collect();
    sync(sim, cena, mapa);
    let mut out = Vec::new();
    for (i, (nome, pid)) in ids.into_iter().enumerate() {
        let Some(&bits) = mapa.get(&pid) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        let t = Transform::from_translation(ph2d_core::Vec2::new(JOGO_AT[i][0], JOGO_AT[i][1]));
        let mut em = sim.world_mut().entity_mut(e);
        em.insert((Name::new(nome), t));
        match i {
            0 => {
                em.insert(LuauScript::library(jogo));
            }
            2 => {
                em.insert(ph2d_ecs::MasterRoot);
            }
            _ => {}
        }
        out.push(e);
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    out
}

pub const ARRASTAR_NAME: &str = "arrastar.luau";

/// ⭐ **O script do nível 7** — o dono arrasta objectos da Hierarquia para ele.
pub const ARRASTAR_LUAU: &str = r#"-- arrastar.luau — o CÍRCULO AZUL corre ESTE script. Aqui você ARRASTA objetos da Hierarquia
-- (a lista de objetos) para dentro do código. Depois de cada passo: Ctrl+S, e a aba «Cena» para ver.

local tempo = 0

function update(self, dt)
  tempo = tempo + dt

  -- ① Com o Ctrl carregado, arraste «Door» da Hierarquia e solte NA LINHA VAZIA aqui em baixo:

  if door then
    ph2d.set(door, "y", math.sin(tempo * 2) * 1.5) -- a porta sobe e desce
  end

  -- ② Abra uma linha nova logo abaixo desta (End e Enter), escreva  ph2d.set(  , arraste «Gold
  --    Coin» SEM o Ctrl para logo depois do parêntese e termine com  , "rotation", tempo)
end
"#;

/// Onde as peças do nível 7 nascem (metros): o círculo, a porta e a moeda.
pub const ARRASTAR_AT: [[f32; 2]; 3] = [[-4.0, 0.0], [0.5, 0.0], [4.5, 0.5]];

/// ⭐ **O nível 7**: o círculo «Ship» com o `arrastar.luau` (aberto numa aba), a porta «Door» e a
/// moeda «Gold Coin» — sem script, para serem arrastadas para o código. Devolve `[Ship, Door, Coin]`.
pub fn monta_arrastar(
    sim: &mut SimWorld,
    cena: &mut VecScene,
    mapa: &mut VecEntityMap,
) -> Vec<Entity> {
    let script = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert(ARRASTAR_NAME, ARRASTAR_LUAU);
    let r = SHAPE_R;
    let pecas = [
        ("Ship", tinta(ellipse([0.0, 0.0], r, r), [96, 168, 235])),
        (
            "Door",
            tinta(
                rectangle([-r * 0.5, -r * 1.5], [r * 0.5, r * 1.5]),
                [176, 120, 80],
            ),
        ),
        (
            "Gold Coin",
            tinta(star([0.0, 0.0], r, r, 5, 0.45), [240, 196, 72]),
        ),
    ];
    let ids: Vec<_> = pecas
        .into_iter()
        .map(|(nome, p)| (nome, cena.push_path(p)))
        .collect();
    sync(sim, cena, mapa);
    let mut out = Vec::new();
    for (i, (nome, pid)) in ids.into_iter().enumerate() {
        let Some(&bits) = mapa.get(&pid) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        let at = ARRASTAR_AT[i];
        let mut em = sim.world_mut().entity_mut(e);
        em.insert((
            Name::new(nome),
            Transform::from_translation(ph2d_core::Vec2::new(at[0], at[1])),
        ));
        if i == 0 {
            em.insert(LuauScript::library(script));
        }
        out.push(e);
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    sim.world_mut()
        .get_resource_or_init::<ScriptEditRequests>()
        .0
        .push(script);
    out
}

/// O piloto das fotos (`PH2D_CODE_SMOKE_DEMO=1`, e só na cena 6): as variáveis, lidas uma vez.
#[must_use]
pub fn piloto_armado() -> bool {
    static ARMADO: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ARMADO.get_or_init(|| {
        let var = |k: &str| std::env::var(k).ok().map(|v| v.trim().to_owned());
        var("PH2D_CODE_SMOKE_DEMO").as_deref() == Some("1")
            && var("PH2D_CODE_SMOKE").as_deref() == Some("6")
    })
}

/// ⭐ **O piloto da cena 6**, quadro `n`: os EVENTOS que um jogador daria, postos no estado de
/// entrada REAL (o mesmo caminho do winit) — para as fotos, onde os eventos sintéticos do X não
/// chegam. Devolve o ponto do mundo sob o rato.
pub fn piloto(input: &mut ph2d_input::InputState, n: u64) -> [f64; 2] {
    use ph2d_input::{Event, Key, MouseButton};
    let direita = Key(0xF703);
    let espaco = Key(0x20);
    let eventos: &[Event] = match n {
        30 => &[Event::KeyDown(direita)],
        90 => &[Event::KeyUp(direita)],
        100 | 120 | 140 => &[Event::KeyDown(espaco), Event::KeyUp(espaco)],
        160 => &[
            Event::MouseButtonDown(MouseButton::Left),
            Event::MouseButtonUp(MouseButton::Left),
        ],
        _ => &[],
    };
    for e in eventos {
        input.apply_event(*e);
    }
    [-3.0, 2.0]
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

#[cfg(test)]
#[path = "code_smoke_tests.rs"]
mod tests;
