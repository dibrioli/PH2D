//! ⭐⭐⭐ **A FASE DO QUADRO DA ROTA B** — quem varre a cena à procura de cataventos.
//!
//! A irmã assada é a [`ph2d_form_donation::baked_form::relight_stale`], chamada pela
//! `fase_relight_baked_forms` da shell. Esta corre **depois** dela, e a ordem é LEI (§ abaixo).
//!
//! ## ⛔ Porque esta fase está atrás da feature e a irmã NÃO está
//!
//! O cabeçalho da `fase_relight_baked_forms` declara, por escrito, a promessa da rota A: *um objecto
//! assado que voltou de um arquivo acende **sem o módulo 3D no build***. ⚠️ **A rota B não pode
//! prometer isso e não o promete:** ela RASTERIZA por quadro, logo precisa da malha — e a malha é o
//! módulo. *A assimetria não é um descuido de `cfg`; é a diferença entre as duas rotas.*
//!
//! ## ⛔⛔ A ORDEM contra a irmã, e porque ela não basta
//!
//! Um objecto vivo é também um objecto **assado** (ver [`acende_os_cataventos`]: a matéria e o slot
//! vêm do [`BakedForm`]), logo a `relight_stale` também o vê. Correr depois faz o vivo **ganhar** —
//! mas deixava o assado a pagar um passe de luz por cada quadro em que o rig mexe.
//! ⇒ esta fase **carimba** o `lit_with` do assado com o rig que acabou de usar, que é a verdade
//! literal (*ele FOI aceso com este rig*), e a irmã salta-o na próxima. *Uma cura que mente ao
//! carimbo seria um remendo; esta diz o que aconteceu.*

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, Mesh3D, SimWorld};
use ph2d_form_donation::baked_form::{BakedForm, PassesDaLuz, rig_stamp};
use ph2d_gpu::GpuContext;
use ph2d_render::SpriteRenderer;

use crate::donation::PoseDaForma;
use crate::vivo::{AlvoVivo, FormaViva, ObjectoVivo, acende_um_quadro};

/// **A MAQUINARIA que serve TODOS os cataventos** — contra os três argumentos que descrevem o
/// estado de cada um.
///
/// ⛔ **Ela não é açúcar para calar um lint** (o clippy acusou `8/7`): é o mesmo corte que o
/// [`ObjectoVivo`] já fez um nível abaixo, e a fronteira é a mesma pergunta — *isto descreve um
/// objecto ou serve a cena inteira?* A placa, o renderizador e os passes servem a cena; os mapas e
/// o mundo são o estado que ela percorre.
pub struct Bancada<'a> {
    /// A placa.
    pub gpu: &'a GpuContext,
    /// Quem possui os slots de sprite para onde a saída é copiada.
    pub renderer: &'a mut SpriteRenderer,
    /// Os passes da luz, cada um construído na primeira acendida dele.
    pub passes: &'a mut PassesDaLuz,
}

/// O que a fase fez neste quadro — o readout de que os gates vivem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cataventos {
    /// Quantos objectos acenderam pela rota B.
    pub acesos: u32,
    /// Quantos de facto voltaram a RASTERIZAR (os outros bateram no carimbo).
    pub rasterizados: u32,
    /// Quantas formas vivas foram largadas por o objecto ter perdido o componente.
    pub largadas: u32,
}

/// ⭐⭐⭐ **Acende toda entidade com [`Mesh3D`] pela rota B, e larga a VRAM de quem já não a tem.**
///
/// ⚠️ **A matéria e o slot vêm do objecto ASSADO, e isso é o desenho e não uma dependência
/// acidental:** o albedo que a luz multiplica é a ARTE do sprite (a mesma nas duas rotas) e o slot
/// individual é para onde a saída vai. A rota B acrescenta **uma** coisa a um objecto misto que já
/// existe — a forma deixa de ser um `Vec<f32>` gravado e passa a ser rasterizada por quadro.
/// ⛔ Um objecto com [`Mesh3D`] e **sem** bake é saltado em silêncio de propósito: ele não tem
/// albedo nem ranhura, e inventá-los aqui seria uma segunda cópia do `bake_one`.
pub fn acende_os_cataventos(
    cena: &mut crate::Sculpt3dScene,
    assados: &mut BTreeMap<u64, BakedForm>,
    vivas: &mut BTreeMap<u64, FormaViva>,
    bancada: Bancada<'_>,
    sim: &mut SimWorld,
    // ⭐⭐ **O relógio do DOCUMENTO e não o da parede**, e a escolha é o que faz um catavento
    // obedecer ao transporte: rebobinar devolve a pá ao sítio, e um scrub mostra o quadro que o
    // artista pediu. *Com o relógio de parede a peça continuaria a girar com a régua parada.*
    segundos: f32,
) -> Cataventos {
    let Bancada {
        gpu,
        renderer,
        passes,
    } = bancada;
    let mut q = sim.world_mut().query::<(Entity, &Mesh3D)>();
    let pedidos: Vec<(u64, PoseDaForma)> = q
        .iter(sim.world())
        .map(|(e, m)| {
            (
                e.to_bits(),
                PoseDaForma {
                    // ⚠️ **DERIVADO, nunca escrito de volta** — ver [`Mesh3D::yaw_em`] e o degrau
                    // `163`: um componente registado reescrito por quadro é um passo de `Ctrl+Z`
                    // por quadro.
                    yaw: m.yaw_em(segundos),
                    pitch: m.pitch,
                },
            )
        })
        .collect();

    // ⚠️ **A VRAM é largada ANTES de acender**, senão um quadro em que o artista tira o componente
    // de um objecto e o põe noutro guarda as duas. Ver [`FormaViva::bytes`]: cada uma é memória de
    // placa que ninguém mais tem a quem perguntar.
    let antes = vivas.len();
    vivas.retain(|bits, _| pedidos.iter().any(|(b, _)| b == bits));
    let largadas = u32::try_from(antes - vivas.len()).unwrap_or(u32::MAX);

    let rig = *cena.rig();
    let carimbo_do_rig = rig_stamp(&rig);
    let mut out = Cataventos {
        largadas,
        ..Cataventos::default()
    };
    for (bits, pose) in pedidos {
        let Some(assado) = assados.get_mut(&bits) else {
            continue;
        };
        // ⚠️ **`remove` e volta a inserir, em vez de `and_modify`:** a [`FormaViva::garante`] toma
        // a anterior **por valor** (ela devolve-a intacta quando o tamanho bate, e é isso que faz o
        // caso comum não alocar nada). Tirá-la do mapa é a única forma de lha dar sem uma variante
        // vazia a existir só para um `mem::replace` a poder segurar.
        let nova = FormaViva::garante(vivas.remove(&bits), gpu, assado.size);
        let viva = vivas.entry(bits).or_insert(nova);
        let antes_r = viva.rasterizacoes;
        let r = acende_um_quadro(
            cena,
            gpu,
            renderer,
            passes,
            &rig,
            ObjectoVivo {
                alvo: AlvoVivo {
                    size: assado.size,
                    base: &assado.base,
                    texture_id: assado.texture_id,
                },
                viva,
                pose,
                // ⚠️ **Do ASSADO e não do ecrã de agora** — ver o campo: o enquadramento é uma
                // memória do gesto, e recalculá-lo por quadro faria a peça deslizar dentro do
                // sprite assim que o artista arrastasse o canvas 2D.
                recorte: assado.recorte,
            },
        );
        if r.is_ok() {
            out.acesos += 1;
            out.rasterizados += viva.rasterizacoes - antes_r;
            // Ver o cabeçalho do módulo: o carimbo diz a verdade, e é ele que impede a irmã
            // assada de re-acender por cima no quadro seguinte.
            assado.lit_with = Some(carimbo_do_rig);
        }
    }
    out
}
