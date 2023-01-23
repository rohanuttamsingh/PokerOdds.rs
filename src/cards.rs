#[derive(Debug, Copy, Clone)]
pub enum Suit {
    Spades,
    Diamonds,
    Clubs,
    Hearts,
}

#[derive(Debug)]
pub struct Card {
    suit: Suit,
    value: u8,
}

impl Card {
    pub fn build(suit: Suit, value: u8) -> Result<Card, &'static str> {
        if !(1..=13).contains(&value) {
            return Err("Value must be between 1 (Ace) and 13 (King)");
        }
        Ok(Card { suit, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_card() {
        for suit in [Suit::Spades, Suit::Diamonds, Suit::Clubs, Suit::Hearts] {
            for value in 1..=13 {
                match Card::build(suit, value) {
                    Ok(_) => (),
                    Err(_) => panic!("{suit:?} {value} should be a valid card"),
                }
            }
        }
    }

    #[test]
    fn too_low_value() {
        let value = 0;
        for suit in [Suit::Spades, Suit::Diamonds, Suit::Clubs, Suit::Hearts] {
            match Card::build(suit, value) {
                Ok(_) => panic!("{suit:?} {value} is not a valid card"),
                Err(_) => (),
            }
        }
    }

    #[test]
    fn too_high_value() {
        let value = 100;
        for suit in [Suit::Spades, Suit::Diamonds, Suit::Clubs, Suit::Hearts] {
            match Card::build(suit, value) {
                Ok(_) => panic!("{suit:?} {value} is not a valid card"),
                Err(_) => (),
            }
        }
    }
}
