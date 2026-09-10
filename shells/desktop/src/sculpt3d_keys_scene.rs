//! ⭐ **OS VERBOS DA LISTA** — o que o `Shift` faz ao teclado da escultura.
//!
//! # Por que isto é um ficheiro irmão
//!
//! Corte feito na **INTEGRAÇÃO de 2026-09-10**, e o vermelho era **ACUMULADO**: o
//! `sculpt3d_keys.rs` chegou a `617 LOC` contra o teto de `600` somando a `line/sculpt3d` e a
//! `line/quadextract`, e **nenhuma das duas o estoura sozinha** — o mesmo mecanismo que o
//! `CLAUDE.md` §5.0 registou hoje para o `screens/hero.rs`. ⛔ A cura de um teto é **cortar por
//! responsabilidade**, nunca declarar a excepção `// ph2d-loc-cap:` que este gate oferece.
//!
//! E a fronteira já estava escrita no comentário do próprio bloco: *«OS VERBOS DA LISTA vêm ANTES
//! dos verbos do PINCEL»*. Aqui fala-se com a **cena** (nascer uma peça, duplicar, fundir, isolar,
//! ler a cavidade); depois dali fala-se com o **pincel**. ⭐ E a prova de que o corte é o certo é
//! que este bloco **não toca `self`** — só precisa da cena, logo é uma função livre e não um
//! método do `App`. Os irmãos `sculpt3d_keys_view` e `sculpt3d_keys_delete` nasceram da mesma lei.

use super::super::{Merge, Primitive, Sculpt3dScene};
use winit::keyboard::KeyCode;

/// Os verbos que o `Shift` arma. `true` = a tecla foi consumida.
pub(crate) fn sculpt3d_shift_verbs(scene: &mut Sculpt3dScene, code: KeyCode) -> bool {
    use winit::keyboard::KeyCode as K;
    let primitive = match code {
        K::Digit1 => Some(Primitive::Sphere),
        K::Digit2 => Some(Primitive::Cube),
        K::Digit3 => Some(Primitive::Cylinder),
        K::Digit4 => Some(Primitive::Torus),
        _ => None,
    };
    if let Some(kind) = primitive {
        let i = scene.add_primitive(kind);
        eprintln!(
            "[sculpt3d] + {} (peca {i}, a cena tem {}) -- Ctrl+Z a tira",
            kind.label(),
            scene.objects.len()
        );
        return true;
    }
    if code == K::KeyD {
        scene.duplicate_active();
        eprintln!(
            "[sculpt3d] DUPLICOU: a cena tem {} pecas -- a copia nasce AO LADO na tela",
            scene.objects.len()
        );
        return true;
    }
    // **FUNDIR.** ⚠️ No `Shift+J` porque *juntar* é o verbo, e porque o
    // `Shift` é onde os verbos da LISTA moram neste teclado (o `J` sozinho
    // é des-subdividir, que age numa peça). O log traz o número dos TRÊS
    // desfechos: a fusão não muda a silhueta da cena — as peças ficam
    // onde estavam —, então sem a contagem o artista vê a mesma imagem e
    // não tem como saber se a tecla fez alguma coisa.
    if code == K::KeyJ {
        match scene.merge_visible() {
            Merge::Done {
                pieces,
                verts,
                faces,
            } => eprintln!(
                "[sculpt3d] FUNDIDAS {pieces} pecas numa so' -- {verts} vertices / {faces} faces \
                     (elas nao ficam SOLDADAS: use V para reconstruir a casca) -- Ctrl+Z as separa"
            ),
            Merge::Nothing => eprintln!(
                "[sculpt3d] nao ha' o que fundir: e' preciso mais de UMA peca a' vista \
                     (Shift+I devolve a cena inteira)"
            ),
            Merge::Stack => eprintln!(
                "[sculpt3d] nao' funde com a pilha montada: a fusao troca a BASE, e todo nivel \
                     acima e' subdivisao dela -- ACHATE a pilha antes"
            ),
        }
        return true;
    }
    // **ISOLAR.** ⚠️ A resposta visual é a cena SUMIR menos uma peça — é
    // o *local view* do Blender, e é por isso que o log diz o que voltou
    // ou o que ficou: uma tela que perde quatro objetos sem uma linha
    // explicando é indistinguível de um crash de render.
    // **A CAVIDADE** — o canal que faz a escultura ser LIDA
    // (`docs/3D/05.1` §4, W10.1).
    //
    // ⚠️ **`Shift+C` e não `C`, e a única coisa que isto tira é um alias
    // acidental:** `C` sozinho limpa a máscara, e como o bloco do `shift`
    // cai adiante quando nada casa, hoje `Shift+C` também limpa. Nenhum
    // atalho DOCUMENTADO se move — e o mnemônico do artista é **C**avity,
    // que é o único que ele vai tentar antes de procurar.
    if code == K::KeyC {
        let amount = scene.cycle_cavity();
        if amount == 0.0 {
            eprintln!(
                "[sculpt3d] cavidade: DESLIGADA -- o barro liso da W3, ao byte                          (Shift+C liga)"
            );
        } else {
            eprintln!(
                "[sculpt3d] cavidade: {amount:.2} -- a fresta ESCURECE e a crista CLAREIA                          (Shift+C avanca; volta a zero depois de 1.00)"
            );
        }
        return true;
    }
    // **O ESPALHAMENTO SUB-SUPERFICIAL** (`docs/3D/05.1` §2a, W10.5) —
    // o terceiro dos três canais que o Enio nomeou, ao lado do AO e da
    // cavidade.
    //
    // ⚠️ `Shift+S` pelo mnemônico do artista (**S**kin / **S**cattering),
    // e ele estava livre no bloco de shift.
    if code == K::KeyS {
        let amount = scene.cycle_sss();
        if amount == 0.0 {
            eprintln!(
                "[sculpt3d] espalhamento: DESLIGADO -- o barro de sempre, ao byte                          (Shift+S liga)"
            );
        } else {
            eprintln!(
                "[sculpt3d] espalhamento: {amount:.2} -- a luz ATRAVESSA a borda da sombra,                          e o VERMELHO vai mais longe que o azul.                          O painel tem as duas pistas: 'Subsurface' e 'Scatter' (o alcance)."
            );
        }
        return true;
    }
    if code == K::KeyI {
        let on = scene.toggle_isolate();
        if on {
            eprintln!(
                "[sculpt3d] ISOLADA: as outras {} pecas sairam da vista (Shift+I devolve) \
                     -- o pincel nao alcanca o que nao se ve",
                scene.objects.len().saturating_sub(1)
            );
        } else {
            eprintln!(
                "[sculpt3d] a cena inteira voltou: {} pecas a' vista",
                scene.objects.len()
            );
        }
        return true;
    }
    false
}
