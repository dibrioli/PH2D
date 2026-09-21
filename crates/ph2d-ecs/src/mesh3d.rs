//! `Mesh3D` — **o CATAVENTO**: a malha 3D que um sprite mantém VIVA, e a pose dela.
//!
//! A [`crate::BakedForm`] é a **rota A** (`docs/3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md`):
//! a forma é rasterizada **uma vez**, os canais viajam no documento, e a luz relê-os quando o rig
//! muda. Esta é a **rota B**: a malha continua a existir, rasteriza **por quadro** para o G-buffer,
//! e é isso que faz **virar o objecto e a luz acompanhar**.
//!
//! # ⭐⭐⭐ Porque a POSE mora aqui, e não no `Transform`
//!
//! ⛔⛔ **Isto é uma medição, não uma preferência.** O [`crate::Transform`] tem `rotation: f32` e
//! exprime **apenas** a rotação no plano do ecrã — e a §5.0 do catavento mediu que *essa* a rota A
//! já dá **exactamente**: uma rotação `R` em torno do eixo da vista leva as normais a `R·n` e a
//! imagem a `R·imagem`, e as duas são operações 2D sobre o plano já assado (`0,00°` de desacordo de
//! normais, contra `28,58°` de um plano fixo). **Fora do plano não há operação 2D nenhuma**
//! (`31,69°`), porque aparecem faces que não estavam na imagem.
//!
//! ⇒ *a rotação que justifica esta rota é exactamente a que FALTA àquele campo*, logo ela tem de
//! viajar aqui. Tabelas: `docs/Render3d/17_a_rota_b_o_catavento.md` §1.5.
//!
//! # ⛔ A PRESENÇA é a decisão, e não um `bool`
//!
//! O `02.2` diz que a escolha entre as duas rotas é *«uma propriedade do objeto»* — e ela é **ter
//! ou não ter esta componente**. Um campo `live: bool` ao lado seria um segundo sítio a dizer a
//! mesma coisa, e os dois divergiriam no primeiro dia em que alguém escrevesse um sem o outro.
//! ⚠️ Com as duas presentes ganha a **VIVA**: ela é o opt-in explícito, e a assada é o que estava
//! lá antes.
//!
//! # ⛔ O que ela NÃO faz
//!
//! Não põe geometria no ECS — a cena 3D continua dona, exactamente como no
//! [`crate::sculpt_piece_ref`]. E **não** carrega material: a lei que acende é global
//! (`material_da_forma()` não recebe argumentos) e a escolha por objecto que já existe e já é
//! gravada é a `Lei` do assado. *Um `MeshShading { sss, ao, cavity, material }` seria quatro knobs
//! sem consumidor.*

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// A malha 3D que este sprite mantém viva, com a orientação dela.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Mesh3D {
    /// A peça da cena 3D que doa a forma.
    ///
    /// ⚠️ **Um `u32` cru, e não um tipo do módulo de escultura** — a direcção da seta importa: a
    /// shell conhece os dois, e nenhum dos dois conhece o outro. Idêntico ao
    /// [`crate::Sculpt3dPieceRef`].
    pub piece: u32,
    /// A volta em torno do eixo vertical do ecrã — a pá do catavento a virar.
    pub yaw: f32,
    /// A inclinação para cima e para baixo.
    pub pitch: f32,
    /// ⭐⭐⭐ **VOLTAS POR SEGUNDO** — um catavento GIRA, e esta é a propriedade que o diz.
    ///
    /// ⚠️ **O ângulo efectivo é DERIVADO por quadro e NUNCA escrito de volta aqui** — ver a
    /// [`Self::yaw_em`]. É essa linha que a mantém fora do `Ctrl+Z`: um componente registado a ser
    /// reescrito a 60 Hz seria um passo de undo por quadro, o defeito que o `preview_drive` desta
    /// casa existe para impedir. ⇒ `yaw` é o que o artista AUTOROU, e o giro compõe-se com ele.
    ///
    /// ⛔ `0` deixa a peça parada no `yaw` autorado, **ao bit** (ver o gate da inércia).
    pub spin: f32,
}

impl Mesh3D {
    /// **O ângulo que este quadro vai rasterizar** — o autorado mais o giro acumulado.
    ///
    /// ⚠️ Ela existe como porta e não como duas linhas no sítio que a lê porque *o ângulo
    /// efectivo* é uma pergunta com mais de um consumidor à espera (o passe, o gizmo do futuro e
    /// todo gate que queira afirmar a lei sem montar um quadro).
    #[must_use]
    pub fn yaw_em(&self, segundos: f32) -> f32 {
        // ⚠️ `mul_add` NÃO: a inércia em `spin = 0` tem de ser `yaw` **ao bit**, e ela é — mas por
        // `0.0 * t + yaw`, que é exacto nas duas formas. O que a escrita simples garante a mais é
        // ser a mesma conta que um leitor faria à mão ao conferir o gate.
        self.yaw + self.spin * std::f32::consts::TAU * segundos
    }
}

impl SimComponent for Mesh3D {}

#[cfg(test)]
mod tests {
    use super::Mesh3D;

    /// ⭐ **A INÉRCIA: com `spin = 0` o ângulo efectivo é o autorado AO BIT**, em todo instante.
    ///
    /// ⚠️ Ela não é decoração: é o que garante que acrescentar o giro ao componente não move uma
    /// peça que ninguém mandou girar. *Uma lei nova cuja omissão não é byte-idêntica re-baseia, em
    /// silêncio, tudo o que já estava gravado.*
    ///
    /// **Mutação que deve sangrar:** `self.yaw + self.spin * …` → `self.yaw + 1.0e-7 + …`.
    #[test]
    fn sem_giro_o_angulo_e_o_autorado_ao_bit() {
        let m = Mesh3D {
            piece: 0,
            yaw: 0.734_215_9,
            pitch: -0.2,
            spin: 0.0,
        };
        // ⚠️ Inclui instantes GRANDES de propósito: `0 * t` é exacto em `f32` para todo `t`
        // finito, e é essa exactidão que a lei promete — não uma tolerância.
        for t in [0.0_f32, 1.0, 7.5, 3600.0, 1.0e6] {
            assert_eq!(
                m.yaw_em(t).to_bits(),
                m.yaw.to_bits(),
                "com spin = 0 o angulo em t = {t} tem de ser o autorado, ao bit"
            );
        }
    }

    /// **UMA VOLTA POR SEGUNDO dá uma volta por segundo** — a unidade do campo, afirmada.
    ///
    /// ⚠️ **A régua é a VOLTA e não o radiano**, porque é a volta que o nome do campo promete: um
    /// `spin` em radianos/s passaria neste teste com `TAU` a menos e o artista escreveria `6,28`
    /// para uma volta. *Uma unidade que só existe no nome do campo não é uma unidade.*
    ///
    /// **Mutação que deve sangrar:** `std::f32::consts::TAU` → `1.0`.
    #[test]
    fn uma_volta_por_segundo_e_uma_volta_por_segundo() {
        let m = Mesh3D {
            piece: 0,
            yaw: 0.0,
            pitch: 0.0,
            spin: 1.0,
        };
        let uma_volta = std::f32::consts::TAU;
        assert!(
            (m.yaw_em(1.0) - uma_volta).abs() < 1.0e-6,
            "spin = 1 tem de dar TAU em 1 s, deu {}",
            m.yaw_em(1.0)
        );
        // E o autorado COMPÕE-SE com o giro, nunca é substituído por ele — a metade sem a qual
        // alguém poderia escrever `spin * TAU * t` e passar a primeira.
        let com_pose = Mesh3D { yaw: 0.5, ..m };
        assert!(
            (com_pose.yaw_em(1.0) - (0.5 + uma_volta)).abs() < 1.0e-6,
            "o yaw autorado tem de somar ao giro"
        );
    }
}
