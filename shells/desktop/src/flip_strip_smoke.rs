//! **A cena pronta para o smoke da TIRA COM MÃOS** (`PH2D_FLIP_STRIP_SMOKE=1`).
//!
//! Três gestos novos para julgar, e a cena existe para que nenhum deles precise ser montado
//! à mão: **arrastar a célula** (move a chave no tempo), **arrastar a borda direita dela**
//! (estica o hold) e o **Pin** (light table — o quadro que fica visível como fantasma mesmo
//! longe do playhead).
//!
//! As quatro chaves são deliberadamente DESIGUAIS em exposição (4 · 1 · 6 · 2): a largura da
//! célula é a duração, então uma tira de células iguais não mostraria que a largura *diz*
//! alguma coisa — e o gesto de esticar não teria contra o que ser comparado. E os desenhos
//! são grandes e de cores diferentes: para julgar um fantasma fixado é preciso reconhecê-lo
//! de longe.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_STRIP_SMOKE").is_some())
}

/// As quatro chaves: `(quadro, cor, altura da barra)`.
///
/// A **exposição não é autorada aqui**: ela É a distância até a chave seguinte (4, 1 e 6),
/// e é isso que a largura da célula desenha. Só a última precisa ser dita — sem sucessora,
/// ela não tem de onde tirar a sua (ver o `set_exposure` no fim de `stage`).
///
/// A altura da barra é o que distingue os desenhos na tela: para julgar um fantasma fixado
/// é preciso reconhecê-lo de longe.
const KEYS: [(i32, Rgba, f32); 4] = [
    (0, Rgba::new(0.95, 0.35, 0.35, 1.0), 0.6),
    (4, Rgba::new(0.95, 0.85, 0.35, 1.0), 1.2),
    (5, Rgba::new(0.40, 0.85, 0.95, 1.0), 1.8),
    (11, Rgba::new(0.60, 0.95, 0.45, 1.0), 2.4),
];

/// A exposição da ÚLTIMA chave (a sentinela que fecha o vão). O roteiro promete 2.
const LAST_EXPOSURE: u32 = 2;

/// Uma barra vertical na cor da chave — grande, chapada e inconfundível.
fn bar(colour: Rgba, height: f32, x: f32) -> FlipStroke {
    let mut s = FlipStroke::new();
    for i in 0..=8u8 {
        let t = f32::from(i) / 8.0;
        s.push_point(Point {
            pos: Vec2::new(x, -1.4 + t * height),
            width: 0.35,
            opacity: 1.0,
            color: colour,
        });
    }
    s.hardness = 0.8;
    s
}

/// **Monta a camada** — porta única: o gate encena por AQUI, senão a mensagem impressa
/// descreveria uma tira que ninguém mais produz.
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> ph2d_flip::LayerId {
    let l = obj.add_layer("Bars");
    for (i, &(key, colour, height)) in KEYS.iter().enumerate() {
        if let Some(d) = obj.insert_frame(l, key, Hold::Implicit, KeyKind::Keyframe) {
            let x = -2.4 + i as f32 * 1.6;
            obj.drawing_mut(d)
                .expect("desenho recém-criado")
                .strokes
                .push(bar(colour, height, x));
        }
    }
    // ⚠️ Só DEPOIS de todas as chaves existirem, e só na última: `set_exposure` EMPURRA as
    // seguintes, então autorar exposição no meio do laço moveria as chaves que ainda vão
    // nascer — a tira sairia com outros quadros que a mensagem não descreve.
    obj.set_exposure(l, KEYS[KEYS.len() - 1].0, LAST_EXPOSURE);
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
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Strip Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
        obj.fps = 12.0;
        // Fantasmas LIGADOS e apertados (±1): é o alcance que faz o light table valer a
        // pena — com ±8 todo mundo já apareceria e o Pin não provaria nada.
        obj.onion.enabled = true;
        obj.onion.frames_before = 1;
        obj.onion.frames_after = 1;
        stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[strip-smoke] cena montada: 4 chaves em 0, 4, 5 e 11 (exposicoes 4, 1, 6 e 2)."
        );
        eprintln!(
            "\n\
             ============================================================\n\
             O QUE VOCE ESTA VENDO\n\
             ============================================================\n\
             No meio da tela: quatro barras coloridas (vermelha, amarela, ciano,\n\
             verde), uma mais alta que a outra. Cada barra e' UM desenho seu.\n\
             \n\
             Na faixa de baixo (a tira): quatro retangulos, um por desenho, e cada\n\
             um com a LARGURA do tempo que aquele desenho fica na tela. Por isso\n\
             eles tem tamanhos diferentes -- o numero dentro de cada um e' quantos\n\
             quadros ele dura (4, 1, 6 e 2).\n\
             \n\
             Sao TRES coisas para conferir. Cada uma leva uns 10 segundos.\n\
             \n\
             ------------------------------------------------------------\n\
             1) ARRASTAR UM RETANGULO -- muda o desenho de lugar no tempo\n\
             ------------------------------------------------------------\n\
             Segure o MEIO de um retangulo e arraste para o lado.\n\
             \n\
             Enquanto voce arrasta, aparece um CONTORNO mostrando onde ele vai\n\
             parar. Ele so muda de lugar de verdade quando voce SOLTA.\n\
             \n\
             Ao chegar no vizinho, ele ENCOSTA e para -- nao passa por cima, nao\n\
             troca de lugar, nao some.\n\
             \n\
             (So CLICAR, sem arrastar, continua fazendo o de sempre: pula para\n\
              aquele desenho. Uma tremidinha de mao no clique nao pode mover nada.)\n\
             \n\
             ------------------------------------------------------------\n\
             2) ARRASTAR A BEIRADA DIREITA -- muda quanto tempo ele dura\n\
             ------------------------------------------------------------\n\
             Chegue com o mouse na BEIRADA DIREITA do retangulo grande (o ciano,\n\
             o de 6). Uma barrinha clara aparece ali. Arraste ela.\n\
             \n\
             O retangulo estica ou encolhe, o numero dentro dele acompanha, e os\n\
             retangulos SEGUINTES sao empurrados junto (esticar um desenho nao\n\
             pode comer o proximo).\n\
             \n\
             No retangulo mais fino (o amarelo, de 1 quadro) essa barrinha NAO\n\
             aparece -- ali ele e' estreito demais para caber os dois gestos, e\n\
             continua servindo so para arrastar. Isso e' de proposito: se voce\n\
             quiser mudar a duracao dele, use a caixa 'Hold' la em cima.\n\
             \n\
             ------------------------------------------------------------\n\
             3) O BOTAO 'Pin' -- deixa um desenho visivel de longe\n\
             ------------------------------------------------------------\n\
             Agora voce esta no comeco. Repare que da para ver, apagadinho, o\n\
             desenho VIZINHO (a barra amarela) -- e' o fantasma de sempre. Os\n\
             outros dois estao longe demais e nao aparecem.\n\
             \n\
             Clique no ULTIMO retangulo (o verde) para ir ate ele. Aperte o botao\n\
             'Pin' na barra de cima. Volte para o primeiro retangulo (o vermelho).\n\
             \n\
             A barra VERDE agora tem de aparecer apagada, mesmo estando longe --\n\
             e a amarela tem de continuar aparecendo tambem. O retangulo que voce\n\
             fixou fica com um pontinho no canto de baixo. Apertar 'Pin' de novo\n\
             desfaz.\n\
             \n\
             (Isso serve para deixar um desenho de referencia na tela enquanto\n\
              voce trabalha em outro -- a mesa de luz.)\n\
             \n\
             ============================================================\n\
             Se qualquer uma das tres nao fizer o que esta escrito, me diga O QUE\n\
             ACONTECEU -- e' so isso que eu preciso.\n\
             ============================================================\n"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_flip::FlipDoc;

    /// 🔴 **A cena contém o que a mensagem promete.** Uma mensagem que descreve outra tira
    /// manda o Enio procurar o que não existe — e ele julga o produto pelo que leu.
    #[test]
    fn the_smoke_scene_shows_what_its_message_promises() {
        let mut doc = FlipDoc::default();
        let oid = doc.push_object("Strip Smoke");
        let obj = doc.object_mut(oid).expect("objeto");
        let l = stage(obj);
        let layer = obj.layer(l).expect("camada");
        let cells = layer.cells();

        let keys: Vec<i32> = cells.iter().map(|(k, _, _)| *k).collect();
        assert_eq!(keys, vec![0, 4, 5, 11], "as 4 chaves da mensagem");
        let exposures: Vec<u32> = cells.iter().map(|(_, _, e)| *e).collect();
        assert_eq!(
            exposures,
            vec![4, 1, 6, 2],
            "as exposições que a mensagem promete — a largura da célula É esse número"
        );
        // Elas TÊM de ser desiguais: uma tira de células iguais não mostra que a largura
        // significa alguma coisa, e o gesto de esticar não teria contra o que ser julgado.
        assert!(
            exposures
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                >= 3,
            "as exposições precisam ser visivelmente diferentes"
        );
        // E cada chave tem arte (um fantasma fixado só se julga se der para reconhecê-lo).
        for (_, drawing, _) in &cells {
            assert!(
                obj.drawing(*drawing).is_some_and(|d| !d.strokes.is_empty()),
                "toda chave da cena desenha alguma coisa"
            );
        }
    }

    /// 🔴 **A cena arma o caso que o Pin existe para resolver**: no quadro 0, com alcance
    /// ±1, a ÚLTIMA chave está fora — então fixá-la muda o que se vê, e o smoke tem um
    /// veredito. Com um alcance generoso a barra verde já apareceria e o gesto não provaria
    /// nada (a fixture não conteria o fenômeno).
    #[test]
    fn the_last_key_is_out_of_ghost_range_so_pinning_it_changes_the_screen() {
        let mut doc = FlipDoc::default();
        let oid = doc.push_object("Strip Smoke");
        let obj = doc.object_mut(oid).expect("objeto");
        obj.onion.frames_before = 1;
        obj.onion.frames_after = 1;
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
