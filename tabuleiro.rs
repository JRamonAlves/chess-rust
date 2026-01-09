use crate::casa::Casa;
use crate::{
    casa::OptionalPeca,
    posicao::{Coluna, Linha, Posicao},
};

pub struct Tabuleiro {
    pub casas: Vec<Casa>,
}

impl Tabuleiro {
    pub fn new() -> Self {
        let mut casas: Vec<Casa> = Vec::new();
        for i in 1..9 {
            for j in 1..9 {
                let casa = Casa {
                    posicao: Posicao::new(Linha::from(i), Coluna::from(j)),
                    peca: OptionalPeca::None,
                };
                casas.push(casa);
            }
        }

        Tabuleiro { casas: casas }
    }
}
