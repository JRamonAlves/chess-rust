use crate::peca::{Peca, PecaData};
use crate::posicao::Posicao;

pub struct Rainha {
    pub data: PecaData,
}

impl Peca for Rainha {
    fn mover(&self, posicao: &Posicao) -> bool {
        todo!()
    }

    fn possiveis_movimentos(&self) -> Vec<crate::posicao::Posicao> {
        todo!()
    }
}
