//! ⭐⭐⭐ **O TERCEIRO PEDAÇO DO QUADRO — o que corre entre o nosso encode e a placa, e que
//! nenhuma sonda desta casa media.**
//!
//! ⚠️ Nasceu da pergunta do dono de 2026-09-21 (*«não aguenta arredondar 90 000 mas aguenta
//! 20 000. talvez seja o limite normal. avalie»*). A escada do carimbo
//! (`ph2d-app-motion`, `audit_the_corner_radius_cost`) mede `3,5 ms` de CPU a `102 400` cópias com
//! as quinas redondas, e o quadro que o dono vê mede `19,53 ms` ⇒ **`~16 ms` não estavam
//! atribuídos a ninguém**, e sem os atribuir não há como dizer se o tecto é da MÁQUINA ou nosso
//! (`CLAUDE.md` §0.0).
//!
//! ⭐ Entre os dois há um passo de **CPU** que não é nosso: o [`vello_encoding::Resolver`] percorre
//! os fluxos da cena e monta o `layout`, as rampas e o atlas de imagens que a placa vai ler. O
//! `vello::Renderer` corre-o dentro do `render_to_texture`, logo ele conta no relógio do quadro e
//! **não** aparece em nenhuma medição de encode.
//!
//! ⛔ **Esta porta existe porque o `Resolver` não é alcançável de fora desta crate** — o
//! `architecture` desta casa dá à `ph2d-vector` a exclusividade do `vello::*`, e o `vello` nem
//! sequer o re-exporta (é por isso que o `Cargo.toml` declara o `vello_encoding` à parte, com a
//! razão escrita ao lado). Quem mede a cena do Motion vive na `ph2d-app-motion`, três camadas
//! acima; sem esta porta a única saída seria construir uma cena SUCEDÂNEA aqui dentro, e *uma
//! sonda que mede um sucedâneo mede outro programa, para sempre*.
//!
//! ⚠️ **Ela não tem relógio nenhum**, de propósito: quem cronometra é a sonda, que imprime a carga
//! da máquina ao lado do número (`CLAUDE.md` §5.0). Aqui só se corre o trabalho e se devolve o
//! TAMANHO dele — que é a grandeza imune à máquina, e a que diz se duas leituras são comparáveis.

use crate::VectorScene;

/// **O tamanho do que o passo de resolução preparou** — a população, ao lado do relógio de quem
/// mede.
///
/// ⚠️ Ela viaja com a medição porque *uma leitura de relógio sem a população ao lado não se
/// compara com nenhuma outra*: a mesma cena com outra forma resolve outro tamanho, e duas tabelas
/// sem esta coluna leem-se como a mesma experiência.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolveSize {
    /// Quantos bytes o buffer da cena ocupa — o que atravessa o barramento para a placa.
    pub scene_bytes: usize,
    /// Quantos objectos de desenho a cena tem (uma cópia do carimbo é um).
    pub draw_objects: usize,
    /// Quantos segmentos de caminho a cena tem no total (`cópias × segmentos da forma`).
    pub path_segments: usize,
}

/// **O passo de resolução do Vello, isolado e reutilizável.**
///
/// ⚠️ **Um resolvedor só, fora do laço** — é o que o `Renderer` faz, e é o que torna o atlas de
/// imagens persistente entre quadros. *Um resolvedor por medição mede o nascimento do atlas e não
/// o quadro em regime*, que é a mesma armadilha do quadro FRIO que a escada do carimbo já pagou.
pub struct SceneResolver {
    inner: vello_encoding::Resolver,
    packed: Vec<u8>,
}

impl Default for SceneResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneResolver {
    /// Um resolvedor novo, com o buffer de saída vazio.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: vello_encoding::Resolver::new(),
            packed: Vec::new(),
        }
    }

    /// **Resolve esta cena** — o trabalho de CPU que o `render_to_texture` faz antes de a placa ver
    /// um pixel — e devolve o tamanho do que saiu.
    ///
    /// ⚠️ O buffer é reaproveitado entre chamadas, exactamente como no `Renderer`: a primeira
    /// chamada paga o crescimento dele e as seguintes não. *Medir a primeira é medir o alocador.*
    pub fn resolve(&mut self, scene: &VectorScene) -> ResolveSize {
        let enc = scene.inner().encoding();
        let segmentos = enc.n_path_segments as usize;
        let (layout, _ramps, _images) = self.inner.resolve(enc, &mut self.packed);
        ResolveSize {
            scene_bytes: self.packed.len(),
            draw_objects: layout.n_draw_objects as usize,
            path_segments: segmentos,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SceneResolver;
    use crate::VectorScene;
    use crate::scene_prepared::PreparedFill;
    use vello::kurbo::{Affine, BezPath, Point};
    use vello::peniko::{Brush, Color, Fill};

    /// Um polígono fechado de `lados` lados, em coordenadas locais.
    fn poligono(lados: usize) -> BezPath {
        let mut p = BezPath::new();
        for i in 0..lados {
            #[expect(clippy::cast_precision_loss, reason = "meia dúzia de lados")]
            let a = std::f64::consts::TAU * i as f64 / lados as f64;
            let (x, y) = (a.cos(), a.sin());
            if i == 0 {
                p.move_to(Point::new(x, y));
            } else {
                p.line_to(Point::new(x, y));
            }
        }
        p.close_path();
        p
    }

    /// Uma cena com `n` carimbos da mesma forma, pela porta do produto.
    fn cena(lados: usize, n: usize) -> VectorScene {
        let preparada = PreparedFill::new(&poligono(lados), Fill::NonZero);
        let tinta = Brush::Solid(Color::new([1.0, 1.0, 1.0, 1.0]));
        let mut c = VectorScene::new();
        for i in 0..n {
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de cena")]
            let x = i as f64 * 0.01;
            c.fill_prepared(&preparada, Affine::translate((x, 0.0)), &tinta);
        }
        c
    }

    /// ⭐⭐ **A porta conta o que diz contar** — um objecto de desenho por carimbo, e os segmentos a
    /// crescerem com a POPULAÇÃO e com a FORMA.
    ///
    /// ⚠️ **Nada aqui é um número interno do Vello escrito à mão:** o custo de UMA forma é
    /// derivado de uma cena de um carimbo só, e o que se afirma é a MULTIPLICAÇÃO. *Uma barra
    /// copiada da versão de hoje do encodador envelhece na próxima subida do stack.*
    #[test]
    fn o_resolvedor_conta_um_objecto_de_desenho_por_carimbo() {
        let mut r = SceneResolver::new();
        let um = r.resolve(&cena(5, 1));
        assert_eq!(um.draw_objects, 1, "um carimbo é um objecto de desenho");
        assert!(um.path_segments > 0, "um pentágono tem segmentos");

        let muitos = r.resolve(&cena(5, 1_000));
        assert_eq!(muitos.draw_objects, 1_000);
        assert_eq!(
            muitos.path_segments,
            um.path_segments * 1_000,
            "os segmentos são os da forma vezes as cópias"
        );
        assert!(
            muitos.scene_bytes > um.scene_bytes,
            "mil cópias entregam mais bytes à placa do que uma"
        );
    }

    /// ⭐ **O CONTROLO da irmã de cima** — sem ele, uma porta que devolvesse contagens fixas
    /// passaria: *um gate que só vê uma forma não prova que ele vê a forma*.
    #[test]
    fn uma_forma_com_mais_lados_resolve_mais_segmentos() {
        let mut r = SceneResolver::new();
        let triangulo = r.resolve(&cena(3, 100)).path_segments;
        let dodecagono = r.resolve(&cena(12, 100)).path_segments;
        assert!(
            dodecagono > triangulo,
            "12 lados têm de resolver mais segmentos que 3 — leu {dodecagono} contra {triangulo}"
        );
    }

    /// ⚠️⚠️ **A PROPRIEDADE DE QUE A SONDA DEPENDE: o buffer de saída é REAPROVEITADO.**
    ///
    /// O `Renderer` guarda UM resolvedor entre quadros, e é isso que faz a PRIMEIRA chamada pagar o
    /// crescimento do buffer e as seguintes não. Uma sonda que cronometrasse a primeira mediria o
    /// alocador — e é por isso que ela resolve uma vez A FRIO antes de ligar o cronómetro.
    ///
    /// ⛔⛔ **A 1.ª redacção deste gate PROMETIA isto e media outra coisa**, e foi uma mutação
    /// SOBREVIVENTE que o disse: ela comparava a resposta de duas chamadas, e um resolvedor que
    /// deitasse o buffer fora a cada chamada devolve **exactamente a mesma resposta**, só que mais
    /// devagar. *Uma igualdade de resultados não afirma nada sobre reaproveitamento.*
    ///
    /// ⭐ O que discrimina — sem um relógio, que sob fan-out seria mais um membro da família de
    /// flakes de carga — é a **CAPACIDADE**: depois de uma cena grande, uma cena pequena não pode
    /// fazer o buffer encolher. Com o buffer deitado fora a cada chamada ele encolhe, e o gate
    /// reprova.
    #[test]
    fn o_buffer_de_saida_e_reaproveitado_entre_cenas() {
        let mut r = SceneResolver::new();
        let grande = r.resolve(&cena(5, 5_000));
        let capacidade = r.packed.capacity();
        assert!(
            capacidade >= grande.scene_bytes,
            "controlo: o buffer tem de caber a cena grande"
        );
        let pequena = r.resolve(&cena(5, 10));
        assert!(
            pequena.scene_bytes < grande.scene_bytes / 10,
            "controlo: a cena pequena tem de ser MESMO pequena — leu {} contra {}",
            pequena.scene_bytes,
            grande.scene_bytes
        );
        assert_eq!(
            r.packed.capacity(),
            capacidade,
            "a cena pequena não pode fazer o buffer encolher: ele é reaproveitado"
        );
    }

    /// ⭐ **E a resposta é determinística** — a metade que a de cima deixou de afirmar quando
    /// trocou de grandeza. *As duas juntas são o que a sonda precisa: a mesma cena dá o mesmo
    /// número, e o número não custa uma alocação nova.*
    #[test]
    fn resolver_a_mesma_cena_duas_vezes_da_a_mesma_resposta() {
        let mut r = SceneResolver::new();
        let c = cena(5, 500);
        let primeira = r.resolve(&c);
        let segunda = r.resolve(&c);
        assert_eq!(primeira, segunda);
    }
}
