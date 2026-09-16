//! ⭐⭐⭐ **A MEMÓRIA DO PLANO** — os dois estabilizadores do [`crate::Verb::Plane`]
//! (`SPEC_pincel_de_plano.md` §6), e a alavanca do aparar (§14.7).
//!
//! # O que eles fazem
//!
//! Cada dab ajusta um plano à superfície debaixo dele; sem memória, dabs vizinhos ajustam planos
//! DIFERENTES e cada um encosta o barro ao seu, deixando degraus. A memória dá ao plano uma
//! lembrança do traço:
//!
//! - **a firmeza da NORMAL** (§6.1): a normal nova é interpolada com a última publicada, entra
//!   numa memória circular, e a publicada é a média normalizada da memória;
//! - **a firmeza do CENTRO** (§6.2): o centro novo é puxado para o plano publicado no dab anterior,
//!   entra noutra memória, e o publicado guarda o desvio MÉDIO dos centros recentes.
//!
//! ⚠️ **São dois filtros em série em cada metade** (uma interpolação com o anterior **e** uma média
//! móvel), e a do centro mede contra a normal **já estabilizada** — é por isso que os autores do
//! alvo avisam que estabilizar o centro sem a normal dá resultados estranhos (§6.3).
//!
//! # ⛔⛔ Porque ela não é opcional (§14.7, medido no próprio alvo)
//!
//! Com os valores de fábrica do perfil *aparar* e um traço arrastado, oito passagens levam o relevo
//! a `0,029` da rugosidade inicial; com a firmeza da normal a `0`, a `1,380` — **mais rugoso que em
//! repouso**, com uma vala de `4,5×` a amplitude. *Com o acumular ligado o plano segue a superfície
//! que ele próprio corta, e sem a normal presa a inclinação dele acompanha-a.*
//!
//! # ⚠️ Três leis de ciclo de vida
//!
//! - **A memória nasce CHEIA no primeiro dab do traço** — o inerte, que não move nada (a errata
//!   Q6 do R-pré: semeada no primeiro dab que MOVE, a reprodução do G-15 erra `3,0e-03`);
//! - **cada traço novo esquece** (`SculptStroke::begin`) — e é por isso que levantar a caneta entre
//!   passagens apara PIOR (§14.5: `11,9 %` contra `2,9 %`);
//! - **uma memória por passe de SIMETRIA**: a cópia espelhada tem normais espelhadas, e uma
//!   memória partilhada misturaria as duas metades da peça.

/// O comprimento máximo da memória, em dabs (espec §6.1). ⚠️ Facto de comportamento lido do alvo
/// e **não confirmado por medição nossa** — nenhum traço do corpus passa de `8` dabs (§13).
pub const MEMORIA_LIMITE: usize = 20;

/// Quantos dabs a memória guarda com esta firmeza: `1 + firmeza × (LIMITE − 1)`, **truncado**.
#[must_use]
pub fn comprimento_da_memoria(firmeza: f32) -> usize {
    let f = if firmeza.is_finite() {
        firmeza.clamp(0.0, 1.0)
    } else {
        0.0
    };
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let extra = (f * (MEMORIA_LIMITE - 1) as f32) as usize;
    1 + extra
}

/// A memória de UM passe de simetria ao longo de um traço.
#[derive(Clone, Debug, Default)]
pub(crate) struct MemoriaDoPlano {
    normais: Vec<[f32; 3]>,
    pos_normal: usize,
    centros: Vec<[f32; 3]>,
    pos_centro: usize,
    /// O plano que o dab anterior publicou — `None` antes do primeiro dab.
    publicado: Option<([f32; 3], [f32; 3])>,
}

impl MemoriaDoPlano {
    /// O traço acabou: a memória esquece (e fica com a capacidade).
    pub(crate) fn esquecer(&mut self) {
        self.normais.clear();
        self.centros.clear();
        self.pos_normal = 0;
        self.pos_centro = 0;
        self.publicado = None;
    }

    /// **Estabiliza o plano cru** de um dab e devolve `(centro, normal)` publicados.
    pub(crate) fn estabilizar(
        &mut self,
        centro: [f32; 3],
        normal: [f32; 3],
        firmeza_normal: f32,
        firmeza_centro: f32,
    ) -> ([f32; 3], [f32; 3]) {
        let Some((c_ant, n_ant)) = self.publicado else {
            // ⭐ O primeiro dab: as duas memórias nascem CHEIAS com o que ele leu.
            self.normais.clear();
            self.normais
                .resize(comprimento_da_memoria(firmeza_normal), normal);
            self.centros.clear();
            self.centros
                .resize(comprimento_da_memoria(firmeza_centro), centro);
            self.pos_normal = 0;
            self.pos_centro = 0;
            self.publicado = Some((centro, normal));
            return (centro, normal);
        };
        // §6.1 — interpolar com a última publicada (firmeza `1` ⇒ fica a antiga), guardar, e
        // publicar a média normalizada.
        //
        // ⭐ **A interpolada é NORMALIZADA antes de entrar** — a espec não o diz, e o oráculo
        // decide-o (fixturas `firmeza_normal_025`/`_05`): crua, a reprodução erra `1,175e-4` e
        // `6,604e-5`; normalizada, `2,98e-8` e `5,96e-8`. ⚠️ Com a firmeza a zero a normal entra
        // CRUA (a lerp devolve-a ao bit e ela já é unitária), para o caminho da 1.ª missão não
        // mudar um bit.
        let entra = if firmeza_normal > 0.0 {
            let l = lerp(normal, n_ant, firmeza_normal);
            media_normalizada(&[l]).unwrap_or(n_ant)
        } else {
            normal
        };
        guardar(&mut self.normais, &mut self.pos_normal, entra);
        // Com UM dab de memória publica-se o que entrou (já unitário).
        let n_pub = if self.normais.len() == 1 {
            entra
        } else {
            media_normalizada(&self.normais).unwrap_or(n_ant)
        };
        // §6.2 — puxar o centro novo para o plano publicado no dab anterior (firmeza `1` ⇒ fica a
        // projecção), guardar, e publicar mantendo o desvio MÉDIO dos centros guardados contra o
        // plano que passa pelo centro que entrou, com a normal JÁ estabilizada.
        let altura = dot(sub(centro, c_ant), n_ant);
        let projeccao = sub(centro, scale(n_ant, altura));
        let c_entra = lerp(centro, projeccao, firmeza_centro);
        guardar(&mut self.centros, &mut self.pos_centro, c_entra);
        let media = media_das_alturas(&self.centros, c_entra, n_pub);
        // `centro = entrou − n·(altura_dele − média)`, e a altura do que entrou contra um plano
        // que passa por ele é zero.
        let c_pub = [
            c_entra[0] + n_pub[0] * media,
            c_entra[1] + n_pub[1] * media,
            c_entra[2] + n_pub[2] * media,
        ];
        self.publicado = Some((c_pub, n_pub));
        (c_pub, n_pub)
    }
}

/// Guarda `v` na memória circular (a mais antiga sai).
fn guardar(memoria: &mut [[f32; 3]], pos: &mut usize, v: [f32; 3]) {
    if memoria.is_empty() {
        return;
    }
    memoria[*pos % memoria.len()] = v;
    *pos = (*pos + 1) % memoria.len();
}

fn media_normalizada(memoria: &[[f32; 3]]) -> Option<[f32; 3]> {
    let mut s = [0.0f64; 3];
    for n in memoria {
        for k in 0..3 {
            s[k] += f64::from(n[k]);
        }
    }
    let len = (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
    if !len.is_finite() || len == 0.0 {
        return None;
    }
    #[allow(clippy::cast_possible_truncation)]
    Some([
        (s[0] / len) as f32,
        (s[1] / len) as f32,
        (s[2] / len) as f32,
    ])
}

fn media_das_alturas(memoria: &[[f32; 3]], origem: [f32; 3], normal: [f32; 3]) -> f32 {
    if memoria.is_empty() {
        return 0.0;
    }
    let soma: f64 = memoria
        .iter()
        .map(|c| f64::from(dot(sub(*c, origem), normal)))
        .sum();
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    let media = (soma / memoria.len() as f64) as f32;
    media
}

fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::{MEMORIA_LIMITE, MemoriaDoPlano, comprimento_da_memoria};

    /// ⭐ **G-14 — a memória tem tecto**: satura em `20` dabs, e a firmeza zero é UM dab (sem
    /// memória nenhuma).
    #[test]
    fn a_memoria_da_normal_tem_tecto() {
        assert_eq!(comprimento_da_memoria(0.0), 1);
        assert_eq!(comprimento_da_memoria(1.0), MEMORIA_LIMITE);
        assert_eq!(MEMORIA_LIMITE, 20);
        assert_eq!(
            comprimento_da_memoria(0.5),
            10,
            "o comprimento e' TRUNCADO: 1 + 9,5"
        );
        assert_eq!(comprimento_da_memoria(7.0), 20, "a firmeza satura em 1");
        assert_eq!(comprimento_da_memoria(f32::NAN), 1);
    }

    /// ⭐ **As duas pontas da lei, à mão:** firmeza `0` publica o plano cru ao bit, e firmeza `1`
    /// prende a normal do primeiro dab e o centro no plano do anterior.
    #[test]
    fn firmeza_zero_e_o_plano_cru_e_firmeza_um_prende_o_plano() {
        let n0 = [0.0, 0.0, 1.0];
        let n1 = [0.6, 0.0, 0.8];
        let mut solta = MemoriaDoPlano::default();
        solta.estabilizar([0.0; 3], n0, 0.0, 0.0);
        assert_eq!(
            solta.estabilizar([0.1, 0.0, 0.3], n1, 0.0, 0.0),
            ([0.1, 0.0, 0.3], n1)
        );

        let mut presa = MemoriaDoPlano::default();
        presa.estabilizar([0.0; 3], n0, 1.0, 1.0);
        let (c, n) = presa.estabilizar([0.1, 0.0, 0.3], n1, 1.0, 1.0);
        assert_eq!(n, n0, "a firmeza 1 deixou a normal rodar");
        assert!(
            c[2].abs() < 1e-7,
            "a firmeza 1 deixou o centro sair do plano anterior: {c:?}"
        );

        presa.esquecer();
        assert_eq!(
            presa.estabilizar([0.0; 3], n1, 1.0, 1.0).1,
            n1,
            "o traço novo nao esqueceu"
        );
    }
}
