use crate::jogador::Jogador;
use crate::posicao::Posicao;

pub mod bispo;
pub mod cavalo;
pub mod peao;
pub mod rainha;
pub mod rei;
pub mod torre;

pub struct PecaData {
    pub jogador: Jogador,
    pub nome: String,
}

pub trait Peca {
    fn mover(&self, posicao: &Posicao) -> bool;
    fn possiveis_movimentos(&self) -> Vec<Posicao>;
}
