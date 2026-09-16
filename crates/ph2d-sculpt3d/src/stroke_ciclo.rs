//! **O CICLO DE VIDA DE UM TRAÇO** — o que ele congela ao começar, o que ele
//! esquece, e as duas portas que armam a base persistente.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e foi forçado pelo tecto de LOC**
//! (2026-09-15, `stroke.rs` a `707` contra `700`): o pai declara *o que um traço
//! É* — os campos e os módulos que os lêem —, e aqui mora *quando ele nasce e
//! quando ele esquece*. ⛔ Uma entrada no `FILE_OVERAGE_OK` não era saída: o
//! `CLAUDE.md` §5.0 declara que a cura de um tecto é o corte, e este ficheiro
//! oscilava no limite havia dias.
//!
//! ⚠️ **Ele é FILHO do [`super`] e não um irmão**, pela razão de sempre nesta
//! crate: o `begin` toca praticamente todo campo privado de
//! [`crate::SculptStroke`], e a privacidade do Rust é visível aos descendentes.

use super::*;

impl SculptStroke {
    /// **Quantos vértices o último dab tocou** — sonda, para uma medição de
    /// custo poder dizer se duas colunas fizeram o mesmo trabalho.
    #[doc(hidden)]
    #[must_use]
    pub fn footprint_len(&self) -> usize {
        self.footprint.len()
    }
}

impl SculptStroke {
    /// Congela o `pre`: começa um traço novo sobre `mesh`.
    ///
    /// Não copia a malha — a captura é **preguiçosa, por vértice tocado**. Um
    /// traço numa malha de 5 M vértices que toca 20 mil paga 20 mil, não 5 M.
    /// **Re-sorteia o [`crate::FilterKind::Random`] sem mover nada.**
    ///
    /// ⚠️ **Nasce em `0` e o crate nunca a move**, de propósito: a semente da
    /// referência é `rand()` por invocação, o que faria de todo gate deste
    /// verbo uma medição de sorte. O determinismo é a decisão; variar é gesto
    /// do shell.
    ///
    /// ⚠️ **E ela é MENOS necessária aqui do que na referência**, pelo motivo
    /// que o `stroke_filter.rs` mede: o sorteio hasheia os BITS DA
    /// POSIÇÃO congelada, então um segundo gesto sobre uma malha já perturbada
    /// re-sorteia sozinho. Ela cobre só o caso de re-rolar a MESMA pose.
    pub fn set_filter_seed(&mut self, seed: u32) {
        self.filter_seed = seed;
    }

    /// **CONGELA A BASE PERSISTENTE** nas posições de agora (espec §6.4, o
    /// operador *Set Persistent Base*).
    ///
    /// ⚠️⚠️ **A ORDEM é a lei:** para a opção morder, a base tem de ser gravada
    /// **ANTES** do traço que a há-de contradizer. Gravá-la DEPOIS de um traço é
    /// um **no-op exacto** — ali ela É o repouso do traço seguinte, e as quatro
    /// leituras da §6.4 não mudam. *A experiência «deformar → gravar → repetir»
    /// não mede nada, e é a que ocorre primeiro a quem a desenha.*
    /// ⚠️ **Recebe as POSIÇÕES e não a `Mesh`**, e não é arrumação: quem chama
    /// tem a malha e o traço no mesmo `self`, e pedir as duas de uma vez é um
    /// empréstimo duplo. *A porta tem de caber no sítio onde o gesto acontece.*
    pub fn set_persistent_base(&mut self, positions: &[[f32; 3]]) {
        self.persistent_base.clear();
        self.persistent_base.extend_from_slice(positions);
    }

    /// **APAGA a base persistente** — o gesto oposto, e ele existe porque uma
    /// base gravada é invisível: sem ele o artista não tem como voltar ao pano
    /// que acumula sem trocar de cena.
    pub fn clear_persistent_base(&mut self) {
        self.persistent_base.clear();
    }

    pub fn begin(&mut self, mesh: &Mesh) {
        let n = mesh.vert_count();
        if self.slot.len() != n {
            self.slot = vec![u32::MAX; n];
            self.stamp = vec![0; n];
            self.epoch = 0;
        }
        self.epoch = self.epoch.wrapping_add(1);
        // O carimbo 0 é o "nunca visto" do vetor recém-criado, então a época
        // nunca pode valer 0 — a mesma regra do `QueryScratch`, e sem ela um
        // traço a cada 4 bilhões nasceria achando que já capturou tudo.
        if self.epoch == 0 {
            self.epoch = 1;
            self.stamp.fill(0);
        }
        self.touched.clear();
        self.base_pos.clear();
        self.base_nrm.clear();
        // ⚠️ **E a fotografia das normais** — ela é do GESTO, e herdá-la faria o
        // raio do traço novo apontar como a superfície estava no anterior.
        self.nrm0_do_pen_down.clear();
        self.base_mask.clear();
        self.accum.clear();
        self.target.clear();
        // ⚠️ **Um traço novo não herda a direção do anterior** — sem esta linha o
        // primeiro dab apontaria para onde a mão ia no gesto passado, que é um
        // lugar arbitrário.
        self.last_center = None;
        // ⚠️ **Nem a pegada congelada** — ela é do GESTO, e um traço novo
        // escolhe a dele no próprio pen-down.
        self.pegada_ancorada.clear();
        // ⚠️ **Nem a inclinação**, e a referência faz o mesmo (*Clay Thumb*:
        // a inclinação volta a zero no primeiro passo do traço).
        // Sem ela o segundo traço começaria de onde o primeiro parou, e o
        // artista veria a mesma ferramenta cavar mais fundo por ter sido usada
        // antes.
        self.thumb_tilt_deg = 0.0;
        // ⚠️ **Nem a abertura do V**, pela mesma razão — e no modo dinâmico ela
        // é MEMÓRIA, então herdá-la faria o primeiro dab do traço novo raspar
        // com o ângulo que a superfície tinha noutro lugar.
        self.scrape_angle_deg = 0.0;
        self.scrape = None;
        // ⚠️ **E a direcção do pincel de plano morre com o traço**, pela razão
        // gémea: herdá-la faria o PRIMEIRO dab do gesto novo já mover barro, e a
        // espec §1 mede-o em `0` de `2 401`. *O bit é do TRAÇO, não da sessão.*
        self.plano_teve_direccao = false;
        self.plano = None;
        // ⚠️ **O `b` do HC morre com o traço**, e é o que faz dele o *"array
        // zerado no início do traço"* do *Surface Smooth*: dentro de um gesto
        // ele PERSISTE entre dabs (a lei da referência), entre gestos não.
        self.hc_b.clear();
        // ⚠️ **A região de tecido morre com o traço**, e é isso que a torna
        // barata: ela é medida uma vez por gesto. Herdá-la faria o traço novo
        // simular a região do anterior, num lugar onde o artista já não está.
        self.cloth.clear();
        self.cloth_ref.clear();
        // ⚠️ E a do FILTRO pela mesma razão: ela é do gesto, não do programa.
        self.cloth_filter = None;
        // ⚠️ A cadeia da pose é do TRAÇO: um traço novo acha o pivô outra vez.
        self.pose = None;
        self.pose_construcoes = 0;
        // ⚠️ E a do CONTORNO pela mesma razão: o censo de bordas e a cadeia são
        // do TRAÇO, e um traço novo aponta outra borda.
        self.boundary = None;
        self.boundary_construcoes = 0;
        // ⚠️⚠️ **E o INDICADOR esquece TUDO, adjacência incluída** — este é o
        // único momento em que a malha pode ter mudado de uma forma que a chave
        // dele não vê (as ligações entre peças dependem das POSIÇÕES, e lê-las
        // seria varrer a malha inteira por quadro, que é o custo que ele existe
        // para não pagar).
        self.pose_previa.esquecer();
        self.boundary_previa.esquecer();
    }
}
