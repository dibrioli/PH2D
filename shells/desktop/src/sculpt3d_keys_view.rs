//! ⭐⭐ **O TECLADO DA CÂMERA** — a divisão em quatro e as seis vistas nomeadas.
//!
//! Irmão do [`crate::sculpt3d`]`::keys` pelo tecto de LOC, e o corte é o que a
//! nota daquele módulo já desenhava: lá *o que a mão escolhe sobre o BARRO* (o
//! verbo, o nível, a máscara, o espelho, a luz), aqui *o que ela escolhe sobre a
//! VISTA*.
//!
//! ⚠️ **Uma função LIVRE e não um método do `App`**, ao contrário do irmão: tudo
//! o que estas teclas tocam é a cena, e um método obrigaria a re-emprestá-la —
//! quem já a tem na mão passa-a.

use crate::app_state::App;
use crate::sculpt3d::Sculpt3dScene;

/// As teclas da CÂMERA. `true` se consumiu.
pub(crate) fn camera_key(
    scene: &mut Sculpt3dScene,
    code: winit::keyboard::KeyCode,
    ctrl: bool,
) -> bool {
    // ⭐⭐ **`Escape` FECHA O MENU DE VISTAS — e só ele.**
    //
    // ⚠️ **Devolve `false` sem menu aberto**, e é isso que a mantém invisível:
    // `Escape` é a tecla de desistir de meio mundo, e um handler que a
    // reclamasse sempre roubaria o cancelar de quem vem a seguir no roteador.
    // *Um popup só possui a tecla enquanto está aberto.*
    if code == winit::keyboard::KeyCode::Escape {
        return scene.close_view_menu();
    }
    // ⭐⭐ **AS SEIS VISTAS NOMEADAS** (2026-09-08) — `Numpad1` frente · `Numpad3` direita ·
    // `Numpad7` topo, e **`Ctrl`** dá a oposta.
    //
    // ⚠️ **A tabela é a do módulo vizinho, lida e não re-decidida**
    // ([`crate::field3d_views::view_for_key`]): é a memória de dedo do Blender, e ter duas
    // tabelas faria `Numpad3` significar coisas diferentes em dois cantos do mesmo app.
    //
    // ⚠️ **Os eixos, esses, são os NOSSOS** — o Blender é `Z` para cima e esta casa é `Y`
    // (ver `Camera3d::UP`); o que se herda é a tecla, nunca o eixo.
    if let Some(v) = crate::field3d_views::view_for_key(code, ctrl) {
        scene.aim_view(v);
        eprintln!("[sculpt3d] vista: {}", v.key());
        return true;
    }
    false
}

impl App {
    /// ⭐⭐⭐ **`Ctrl+Alt+Q` ABRE E FECHA A DIVISÃO** — a MESMA tecla do módulo de
    /// modelagem, que é a do Blender para o *Toggle Quad View*.
    ///
    /// ⛔⛔ **REPORT DO ENIO, 2026-09-08:** *«o atalho das 4 viewports não
    /// funciona. E já existia um atalho para isso, se não me engano
    /// Ctrl+Alt+Q.»* — **as duas metades certas, e a segunda explica a
    /// primeira.**
    ///
    /// A tecla que shipou era `Ctrl` + crase, e ela estava **morta**: o
    /// `sculpt3d_key` tem um **catch-all** (`if ctrl { if code != KeyZ { return
    /// false } }`) que devolve `false` para todo `Ctrl+` que não seja o desfazer,
    /// e o meu braço vinha **depois** dele. ⚠️ *O meu próprio gate contra teclas
    /// mortas não o viu: ele compara arms `if code == K::…` entre si, e o que
    /// sombreia aqui é um bloco de modificador que engole o espaço inteiro.*
    ///
    /// ⇒ e a cura da tecla **não é reposicionar a minha**: é usar a que já
    /// existe. O `field3d_quad_key` faz exactamente isto para o canvas vizinho,
    /// e ter duas gramáticas para *«dividir a janela 3D»* nos dois módulos do
    /// mesmo app é a memória de dedo partida ao meio.
    ///
    /// ⚠️ **Os três modificadores exigidos POR NOME**, e não «pelo menos estes»:
    /// um `Ctrl+Alt+Shift+Q` é de outra pessoa, e engoli-lo é sequestro. Lei
    /// copiada do vizinho, à letra.
    ///
    /// ⚠️ **Ela mora FORA do `sculpt3d_key`**, e é o que a mantém viva: dali ela
    /// seria outra vez sombreada pelo catch-all. O despacho chama-a antes.
    pub(crate) fn sculpt3d_quad_key(&mut self, code: winit::keyboard::KeyCode) -> bool {
        if code != winit::keyboard::KeyCode::KeyQ
            || !self.modifiers.control_key()
            || !self.modifiers.alt_key()
            || self.modifiers.shift_key()
            || self.modifiers.super_key()
        {
            return false;
        }
        let Some(scene) = self.gfx.as_mut().and_then(|g| g.sculpt3d.as_mut()) else {
            return false;
        };
        // ⚠️⚠️ **SEM guarda de PONTEIRO, ao contrário da irmã do vizinho — e a
        // diferença é deliberada.** Lá ela existe porque aquele canvas convive
        // com outras superfícies que reclamam a mesma tecla; aqui **nenhuma**
        // das ~30 teclas deste módulo pergunta onde o rato está: quem decide é
        // o `sculpt3d_keys_live` (*o barro está na tela?*), e só ele.
        //
        // ⛔ Copiar a guarda junto com a tecla criaria a única tecla da
        // escultura que falha por causa de onde o cursor calhou de estar — e o
        // caso mais provável é o pior: o artista acabou de clicar num chip do
        // painel, o rato ficou lá, e a tecla não faz nada. *Herda-se a tecla,
        // não a moldura de quem a emprestou.*
        if !scene.clay_on_screen() {
            return false;
        }
        let aberta = scene.toggle_split();
        eprintln!(
            "[sculpt3d] viewports: {}",
            if aberta {
                "QUATRO (topo, direita, frente, artista)"
            } else {
                "UMA"
            }
        );
        true
    }
}

#[cfg(test)]
#[path = "sculpt3d_keys_view_tests.rs"]
mod tests;
