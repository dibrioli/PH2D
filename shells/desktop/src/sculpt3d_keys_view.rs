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

use crate::sculpt3d::Sculpt3dScene;

/// As teclas da CÂMERA. `true` se consumiu.
pub(crate) fn camera_key(
    scene: &mut Sculpt3dScene,
    code: winit::keyboard::KeyCode,
    ctrl: bool,
) -> bool {
    use winit::keyboard::KeyCode as K;
    // ⭐⭐⭐ **ABRE E FECHA OS QUATRO VIEWPORTS** (2026-09-08).
    //
    // ⚠️ **`Ctrl` + CRASE, e o `Ctrl` não é enfeite:** a crase **sozinha já
    // tem dono neste mesmo ficheiro** — ela abre e fecha o painel (linha ~63,
    // e o roteiro da cena `=37` manda o artista usá-la). A primeira redacção
    // desta wave escreveu `K::Backquote` sem modificador e ficou **morta por
    // ordem de leitura**: o braço do painel devolve `true` antes. *Um atalho
    // que compila e nunca corre é o defeito mais barato de escrever e o mais
    // caro de encontrar.*
    //
    // ⚠️ **A crase é a família certa**, e a escolha foi por eliminação: os
    // dez dígitos são verbos, `G`/`H`/`T`/`S`/`A` verbos, `C`/`I`/`B`/`N`
    // máscara, `K`/`J`/`V`/`O`/`P`/`U` topologia, `X`/`Y`/`Z` o espelho,
    // `Q`/`E`/`R`/`F` a luz, `D` a doação e o `Numpad` as vistas.
    //
    // ⛔ **O `Ctrl+Alt+Q` do Blender é inexprimível aqui**: esta porta recebe
    // `ctrl` e `shift` e **não** recebe o `alt`, e alargá-la mexeria na
    // assinatura que os outros trinta braços leem.
    //
    // ⏳ **A superfície de PAINEL fica aberta e nomeada**: o molde é o
    // `SCULPT3D_WIREFRAME` (um interruptor de VISTA, que não pergunta nada ao
    // motor), e ela custa seis sítios — o campo no `Sculpt3dUi`, a entrada na
    // `TOGGLES`, a row no `populate`, a chave de i18n, o `apply_ui` e o
    // `panel_snapshot`. O gatilho é o primeiro report do dono a dizer que não
    // achou a divisão.
    if code == K::Backquote && ctrl {
        let aberta = scene.toggle_split();
        eprintln!(
            "[sculpt3d] viewports: {}",
            if aberta {
                "QUATRO (topo, direita, frente, artista)"
            } else {
                "UMA"
            }
        );
        return true;
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
