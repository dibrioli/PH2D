//! As recusas do solver (antes de alocar nada).

#[derive(Clone, Debug, PartialEq)]
pub enum ErroSolver {
    /// `aritmetica: F16` num dispositivo sem `SHADER_F16`.
    SemShaderF16,
    ConfigInvalida(&'static str),
    /// A grade passa o `max_texture_dimension_2d` do dispositivo (um limite do HARDWARE).
    GradeGrandeDemais {
        lado: u32,
        limite: u32,
    },
}

impl std::fmt::Display for ErroSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SemShaderF16 => write!(
                f,
                "aritmética f16 pedida e o dispositivo não tem SHADER_F16"
            ),
            Self::ConfigInvalida(o) => write!(f, "configuração inválida: {o}"),
            Self::GradeGrandeDemais { lado, limite } => {
                write!(
                    f,
                    "grade de {lado} texels passa o limite do dispositivo ({limite})"
                )
            }
        }
    }
}

impl std::error::Error for ErroSolver {}
