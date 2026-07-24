//! **A cena pronta para o smoke da TIRA COM MÃOS** (`PH2D_FLIP_STRIP_SMOKE=1`).
//!
//! Três gestos novos para julgar: **arrastar a caixa** (move a chave no tempo),
//! **arrastar a borda direita dela** (estica o hold) e o **Pin** (light table — o
//! desenho que fica visível como vulto mesmo longe do playhead).
//!
//! A cena é a **BOLA QUICANDO** — o flipbook canônico: 4 poses inconfundíveis
//! (alto-esquerda → caindo → ESMAGADA no chão → alto-direita), cada uma numa cor.
//! As duas cenas anteriores (barras) REPROVARAM no smoke por leitura: *"só vejo 4
//! linhas"* e *"não há retângulo nenhum"* — barras finas espalhadas leem como
//! objetos avulsos, e "retângulo" colidia com as células da tira. Agora o canvas
//! tem UMA bola (nunca chamada de retângulo) e as células da tira são as únicas
//! "caixas" do roteiro.
//!
//! As quatro chaves são deliberadamente DESIGUAIS em exposição (4 · 1 · 6 · 2): a
//! largura da caixa é a duração, e o gesto de esticar precisa de contra o que ser
//! comparado.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_STRIP_SMOKE").is_some())
}

/// As quatro poses da bola: `(quadro, cor, centro x, centro y, raio x, raio y)`.
///
/// A **exposição não é autorada aqui**: ela É a distância até a chave seguinte
/// (4, 1 e 6); só a última precisa ser dita (ver o `set_exposure` em `stage`).
///
/// O `x` do centro CRESCE a cada pose (a bola VIAJA — é o que faz a cena ler como
/// um desenho mudando no tempo, não como objetos numa cena) e a pose do chão é
/// **mais larga que alta** (squash — reconhecível de longe, que é o que o teste
/// do Pin precisa).
const POSES: [(i32, Rgba, f32, f32, f32, f32); 4] = [
    (0, Rgba::new(0.95, 0.35, 0.35, 1.0), -1.10, 0.55, 0.42, 0.42),
    (
        4,
        Rgba::new(0.95, 0.85, 0.35, 1.0),
        -0.35,
        -0.35,
        0.42,
        0.42,
    ),
    (5, Rgba::new(0.40, 0.85, 0.95, 1.0), 0.25, -0.78, 0.62, 0.22),
    (11, Rgba::new(0.60, 0.95, 0.45, 1.0), 1.05, 0.50, 0.42, 0.42),
];

/// A exposição da ÚLTIMA chave (a sentinela que fecha o vão). O roteiro promete 2.
const LAST_EXPOSURE: u32 = 2;

/// Círculo unitário em 12 pontos (30° cada), sem trig em runtime — os valores são
/// `(cos, sin)` exatos o bastante para um desenho de smoke.
const UNIT_RING: [(f32, f32); 12] = [
    (1.0, 0.0),
    (0.866, 0.5),
    (0.5, 0.866),
    (0.0, 1.0),
    (-0.5, 0.866),
    (-0.866, 0.5),
    (-1.0, 0.0),
    (-0.866, -0.5),
    (-0.5, -0.866),
    (0.0, -1.0),
    (0.5, -0.866),
    (0.866, -0.5),
];

/// A bola: um anel grosso (elipse em 12 pontos, fechado). Grosso de propósito —
/// o VULTO do onion é a silhueta 100% recolorida numa tinta escura a meia
/// opacidade, e um traço fino vira um sussurro que ninguém acha na tela.
fn ball(colour: Rgba, cx: f32, cy: f32, rx: f32, ry: f32) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &(ux, uy) in UNIT_RING.iter().chain(std::iter::once(&UNIT_RING[0])) {
        s.push_point(Point {
            pos: Vec2::new(cx + ux * rx, cy + uy * ry),
            width: 0.30,
            opacity: 1.0,
            color: colour,
        });
    }
    s.hardness = 0.8;
    s
}

/// O chão: uma linha cinza fixa, para a pose ESMAGADA ter onde existir.
fn floor_stroke() -> FlipStroke {
    let mut s = FlipStroke::new();
    let grey = Rgba::new(0.55, 0.55, 0.55, 1.0);
    for x in [-1.6f32, 1.6] {
        s.push_point(Point {
            pos: Vec2::new(x, -1.10),
            width: 0.12,
            opacity: 1.0,
            color: grey,
        });
    }
    s.hardness = 0.9;
    s
}

/// **Monta a cena inteira** — porta única: o gate encena por AQUI, senão a
/// mensagem impressa descreveria uma cena que ninguém mais produz. Devolve a
/// camada da BOLA (a ativa — criada por último, que é o fallback do bridge).
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> ph2d_flip::LayerId {
    obj.fps = 12.0;
    // Fantasmas LIGADOS e apertados (±1): é o alcance curto que faz o Pin valer a
    // pena — com ±8 todo mundo já apareceria e o gesto não provaria nada.
    obj.onion.enabled = true;
    obj.onion.frames_before = 1;
    obj.onion.frames_after = 1;
    // ⚠️ SEM esmaecer por distância: um pin fixa a chave 11 vista da chave 0
    // (Δ=11), e com `fade = 1/Δ` o alpha cai a 0,5/11 e é clampado no piso
    // `GHOST_MIN_ALPHA = 0.1` — o vulto que o TESTE 3 manda olhar seria
    // invisível por construção. `fade` é setting autorável (o USE_FADE do GP);
    // desligá-lo é encenação legítima, não maquiagem.
    obj.onion.fade = false;
    // Metade do default (0,5): no smoke de 2026-07-24 o Enio aprovou os gestos e
    // pediu o vulto mais discreto — *"Reduza opacidade do ghost para metade"*.
    // Ainda acima do piso GHOST_MIN_ALPHA (0,1), então o pin segue visível.
    obj.onion.opacity = 0.25;

    // O chão PRIMEIRO: a camada ativa (título da tira, alvo dos gestos) é a
    // ÚLTIMA — tem de ser a da bola.
    let floor = obj.add_layer("Chao");
    if let Some(d) = obj.insert_frame(floor, 0, Hold::Implicit, KeyKind::Keyframe) {
        obj.drawing_mut(d)
            .expect("desenho recém-criado")
            .strokes
            .push(floor_stroke());
    }

    let l = obj.add_layer("Bola");
    for &(key, colour, cx, cy, rx, ry) in &POSES {
        if let Some(d) = obj.insert_frame(l, key, Hold::Implicit, KeyKind::Keyframe) {
            obj.drawing_mut(d)
                .expect("desenho recém-criado")
                .strokes
                .push(ball(colour, cx, cy, rx, ry));
        }
    }
    // ⚠️ Só DEPOIS de todas as chaves existirem, e só na última: `set_exposure`
    // EMPURRA as seguintes, então autorar exposição no meio do laço moveria as
    // chaves que ainda vão nascer.
    obj.set_exposure(l, POSES[POSES.len() - 1].0, LAST_EXPOSURE);
    l
}

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_strip_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let tool_ok = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Strip Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[strip-smoke] cena montada: a bola quicando em 4 chaves (0, 4, 5, 11; \
             exposicoes 4, 1, 6, 2). Ferramenta flip ativa: {}.",
            if tool_ok {
                "sim"
            } else {
                "NAO (PARE: sem ela a faixa Frames nao aparece)"
            }
        );
        eprintln!(
            "\n\
             ============================================================\n\
             ANTES DE TUDO, confira DUAS coisas, nesta ordem:\n\
             ============================================================\n\
             1. Este terminal imprimiu, logo acima, a linha comecando com\n\
                '[strip-smoke] cena montada'. Se NAO imprimiu, PARE: o\n\
                smoke nao rodou (arvore ou variavel de ambiente errada).\n\
             2. Na parte de BAIXO da janela do app existe uma faixa com o\n\
                titulo 'Frames - Bola'. Dentro dela, abaixo da fileira de\n\
                botoes, ha QUATRO CAIXAS lado a lado com os numeros\n\
                4, 1, 6 e 2 dentro. Se essa faixa nao existir, PARE e me\n\
                diga o que voce ve no lugar.\n\
             \n\
             O que esta na tela: UMA bola quicando, desenhada em 4\n\
             momentos -- um desenho por caixa: vermelha no alto a\n\
             esquerda, amarela caindo, ciano ESMAGADA no chao, verde no\n\
             alto a direita. Voce esta no primeiro desenho (a bola\n\
             VERMELHA, na cor real). Perto dela ha um VULTO azul-escuro\n\
             meio transparente: e' o desenho SEGUINTE, mostrado como\n\
             referencia (o 'papel vegetal' do animador). Vulto\n\
             esverdeado = desenho anterior; azulado = seguinte. So o\n\
             desenho em que voce esta tem a cor verdadeira.\n\
             \n\
             ------------------------------------------------------------\n\
             AQUECIMENTO (10 s): clique nas quatro caixas, uma por uma\n\
             ------------------------------------------------------------\n\
             A cada clique a bola PULA para a pose daquele desenho (e\n\
             muda de cor), e os vultos mudam junto. Se isso funciona,\n\
             va aos quatro testes.\n\
             \n\
             ------------------------------------------------------------\n\
             TESTE 1 -- ARRASTAR UMA CAIXA muda o desenho de lugar no tempo\n\
             ------------------------------------------------------------\n\
             Segure o MEIO de uma caixa e arraste para o lado.\n\
             Enquanto arrasta, um CONTORNO mostra onde ela vai parar;\n\
             ela so muda de lugar quando voce SOLTA. Ao chegar na\n\
             vizinha, ENCOSTA e para -- nao passa por cima, nao troca de\n\
             lugar, nao some. (So clicar, sem arrastar, continua pulando\n\
             para aquele desenho; tremidinha de mao no clique nao move.)\n\
             \n\
             ------------------------------------------------------------\n\
             TESTE 2 -- ARRASTAR A BEIRADA DIREITA muda quanto tempo dura\n\
             ------------------------------------------------------------\n\
             Va com o mouse na BEIRADA DIREITA da caixa mais larga (a de\n\
             numero 6). Uma barrinha clara aparece ali; arraste-a.\n\
             A caixa estica ou encolhe, o numero acompanha, e as caixas\n\
             SEGUINTES sao empurradas junto.\n\
             Na caixa mais fina (a de numero 1) a barrinha NAO aparece --\n\
             de proposito: estreita demais para os dois gestos, ela\n\
             continua servindo para arrastar; a duracao dela muda pela\n\
             caixa 'Hold' na fileira de botoes.\n\
             \n\
             ------------------------------------------------------------\n\
             TESTE 3 -- o botao 'Pin' deixa um desenho visivel de longe\n\
             ------------------------------------------------------------\n\
             Clique na PRIMEIRA caixa: a bola verde (a ultima) nao\n\
             aparece nem como vulto -- esta longe demais.\n\
             Agora clique na ULTIMA caixa, aperte 'Pin' na fileira de\n\
             botoes, e volte a primeira caixa.\n\
             O vulto da bola verde agora aparece, mesmo de longe -- e o\n\
             vulto do vizinho continua la. A caixa fixada ganha um\n\
             pontinho no canto de baixo. Apertar 'Pin' de novo desfaz.\n\
             \n\
             ------------------------------------------------------------\n\
             TESTE 4 -- VARIAS CAIXAS de uma vez (marcar e arrastar)\n\
             ------------------------------------------------------------\n\
             Segure SHIFT e clique na PRIMEIRA e na ULTIMA caixa: as\n\
             duas ficam marcadas (cor de destaque). Agora segure o MEIO\n\
             de uma das marcadas e arraste: aparecem DOIS contornos --\n\
             um para cada marcada -- e, ao soltar, as duas mudam de\n\
             lugar JUNTAS, a mesma distancia. O destaque acompanha as\n\
             caixas movidas. As caixas NAO marcadas ficam onde estao, e\n\
             o grupo ENCOSTA nelas e para, como no Teste 1. Arrastar uma\n\
             caixa nao marcada move so ela. Para desmarcar, clique numa\n\
             caixa sem Shift.\n\
             \n\
             ============================================================\n\
             Se algo nao fizer o que esta escrito, me diga O QUE\n\
             ACONTECEU -- e' so isso que eu preciso.\n\
             ============================================================\n"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_flip::FlipDoc;

    fn ball_bbox(points: &[Vec2]) -> (f32, f32, f32) {
        let (mut min_x, mut max_x) = (f32::MAX, f32::MIN);
        let (mut min_y, mut max_y) = (f32::MAX, f32::MIN);
        for p in points {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }
        ((min_x + max_x) * 0.5, max_x - min_x, max_y - min_y)
    }

    /// 🔴 **A cena contém o que a mensagem promete.** Uma mensagem que descreve
    /// outra cena manda o Enio procurar o que não existe — e ele julga o produto
    /// pelo que leu. (As duas cenas anteriores caíram exatamente aí.)
    #[test]
    fn the_smoke_scene_shows_what_its_message_promises() {
        let mut doc = FlipDoc::default();
        let oid = doc.push_object("Strip Smoke");
        let obj = doc.object_mut(oid).expect("objeto");
        let l = stage(obj);
        let layer = obj.layer(l).expect("camada");
        assert_eq!(layer.name, "Bola", "o título da tira que o roteiro nomeia");
        let cells = layer.cells();

        let keys: Vec<i32> = cells.iter().map(|(k, _, _)| *k).collect();
        assert_eq!(keys, vec![0, 4, 5, 11], "as 4 chaves da mensagem");
        let exposures: Vec<u32> = cells.iter().map(|(_, _, e)| *e).collect();
        assert_eq!(
            exposures,
            vec![4, 1, 6, 2],
            "os números DENTRO das caixas — a largura da caixa É esse número"
        );
        // Desiguais: caixas do mesmo tamanho não mostram que a largura significa
        // algo, e o gesto de esticar não teria contra o que ser julgado.
        assert!(
            exposures
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                >= 3,
            "as exposições precisam ser visivelmente diferentes"
        );

        // 🔴 **A cena tem de parecer UMA animação.** A bola VIAJA (x do centro
        // estritamente crescente) e a pose do chão é ESMAGADA (mais larga que
        // alta) — as duas propriedades que fazem "trocar de desenho" ser visível
        // e o vulto fixado ser reconhecível de longe. As poses no ar são
        // redondas (bbox ~quadrada), senão o squash não se destaca.
        let boxes: Vec<(f32, f32, f32)> = cells
            .iter()
            .map(|(_, d, _)| ball_bbox(obj.drawing(*d).expect("arte").strokes[0].positions()))
            .collect();
        for pair in boxes.windows(2) {
            assert!(
                pair[1].0 > pair[0].0,
                "a bola viaja para a direita a cada desenho ({:?})",
                boxes.iter().map(|b| b.0).collect::<Vec<_>>()
            );
        }
        let (_, sw, sh) = boxes[2];
        assert!(
            sw > 1.5 * sh,
            "a pose do chão é ESMAGADA (largura {sw:.2} vs altura {sh:.2})"
        );
        for (i, &(_, w, h)) in boxes.iter().enumerate() {
            if i != 2 {
                assert!(
                    (w - h).abs() < 0.05,
                    "pose no ar {i} é redonda (bbox {w:.2}×{h:.2})"
                );
            }
        }

        // 🔴 **O vulto do Pin tem de ser VISÍVEL.** Com `fade = 1/Δ`, a chave 11
        // fixada e vista da chave 0 sai a 0,5/11 → clampada no piso
        // `GHOST_MIN_ALPHA` (0,1) — invisível sobre o fundo. O roteiro promete um
        // vulto que aparece; a cena desliga o esmaecer para cumprir.
        assert!(obj.onion.enabled, "fantasmas ligados desde o 1º frame");
        assert_eq!(
            (obj.onion.frames_before, obj.onion.frames_after),
            (1, 1),
            "alcance ±1: é o alcance curto que dá ao Pin algo a provar"
        );
        assert!(
            !obj.onion.fade,
            "sem fade por distância — o vulto fixado a Δ=11 seria invisível"
        );
        // A metade pedida no smoke de 2026-07-24 — e ainda acima do piso, senão o
        // clamp comeria a redução e o Pin voltaria a sumir.
        assert_eq!(obj.onion.opacity, 0.25, "o vulto é discreto por ordem");
        assert!(
            obj.onion.opacity > ph2d_flip::GHOST_MIN_ALPHA,
            "a opacidade pedida tem de sobreviver ao clamp do piso"
        );
    }

    /// 🔴 **A cena arma o caso que o Pin existe para resolver**: no quadro 0, com
    /// alcance ±1, a ÚLTIMA chave está fora — então fixá-la muda o que se vê, e o
    /// smoke tem um veredito. Com um alcance generoso a bola verde já apareceria
    /// e o gesto não provaria nada (a fixture não conteria o fenômeno).
    #[test]
    fn the_last_key_is_out_of_ghost_range_so_pinning_it_changes_the_screen() {
        let mut doc = FlipDoc::default();
        let oid = doc.push_object("Strip Smoke");
        let obj = doc.object_mut(oid).expect("objeto");
        let l = stage(obj);
        let obj = doc.object(oid).expect("objeto");
        let layer = obj.layer(l).expect("camada");

        let plain = ph2d_flip::ghosts(layer, 0, &obj.onion, &[], &[]);
        assert!(
            !plain.iter().any(|g| g.key == 11),
            "sem pin a última chave está fora do alcance (é a premissa do roteiro)"
        );
        let pinned = ph2d_flip::ghosts(layer, 0, &obj.onion, &[], &[11]);
        assert!(
            pinned.iter().any(|g| g.key == 11),
            "com pin ela aparece — é isso que o smoke manda olhar"
        );
        assert!(
            pinned.iter().any(|g| g.key == 4),
            "e a vizinha CONTINUA lá: um pin acompanha os vizinhos, não os substitui"
        );
    }
}
