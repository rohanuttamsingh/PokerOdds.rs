use std::collections::HashMap;
use itertools::Itertools;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Suit {
    Spades,
    Diamonds,
    Clubs,
    Hearts,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Card {
    suit: Suit,
    value: u8,
}

impl Card {
    pub fn build(suit: Suit, value: u8) -> Result<Card, &'static str> {
        if !(2..=14).contains(&value) {
            return Err("Value must be between 2 (Two) and 14 (Ace)");
        }
        Ok(Card { suit, value })
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum Hand {
    HighCard(u8),
    Pair(u8),
    TwoPair(u8, u8),
    ThreeOfAKind(u8),
    Straight(u8), // u8: High card
    Flush(u8), // u8: High card
    FullHouse(u8, u8),
    FourOfAKind(u8),
    StraightFlush(u8), // u8: High card
    RoyalFlush,
}

fn is_all_same_value(values: &[u8]) -> bool {
    values.iter().all(|x| *x == values[0])
}

pub fn best_hand(hand: &mut [Card]) -> Hand {
    hand.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    let values: Vec<u8> = hand.iter().map(|x| x.value).collect();
    let suits: Vec<Suit> = hand.iter().map(|x| x.suit).collect();
    let is_straight = values.as_slice() == [2, 3, 4, 5, 14] || match values.iter().as_slice().windows(2).position(|w| w[0] + 1 != w[1]) {
        Some(_) => false,
        None => true,
    };
    let is_flush = match suits.iter().position(|x| *x != suits[0]) {
        Some(_) => false,
        None => true,
    };
    let (is_four, quads) = match values.iter().as_slice().windows(4).any(|w| is_all_same_value(w)) {
        true => (true, Some(values[2])),
        false => (false, None),
    };
    let (is_full_house, full_house_trips, full_house_pair) = {
        if is_all_same_value(&values[..2]) && is_all_same_value(&values[2..]) {
            (true, Some(values[4]), Some(values[0]))
        } else if is_all_same_value(&values[..3]) && is_all_same_value(&values[3..]) {
            (true, Some(values[0]), Some(values[4]))
        } else {
            (false, None, None)
        }
    };
    let (is_three, three_trips) = match values.iter().as_slice().windows(3).any(|w| is_all_same_value(w)) {
        true => (true, Some(values[2])),
        false => (false, None),
    };
    let (is_two_pair, high_pair, low_pair) = {
        if is_all_same_value(&values[..2]) && is_all_same_value(&values[2..4]) {
            let high_pair = std::cmp::max(values[0], values[2]);
            let low_pair = std::cmp::min(values[0], values[2]);
            (true, Some(high_pair), Some(low_pair))
        } else if is_all_same_value(&values[..2]) && is_all_same_value(&values[3..]) {
            let high_pair = std::cmp::max(values[0], values[3]);
            let low_pair = std::cmp::min(values[0], values[3]);
            (true, Some(high_pair), Some(low_pair))
        } else if is_all_same_value(&values[1..3]) && is_all_same_value(&values[3..]) {
            let high_pair = std::cmp::max(values[1], values[3]);
            let low_pair = std::cmp::min(values[1], values[3]);
            (true, Some(high_pair), Some(low_pair))
        } else {
            (false, None, None)
        }
    };
    let (is_pair, pair) = match values.iter().as_slice().windows(2).position(|w| w[0] == w[1]) {
        Some(index) => (true, Some(values[index])),
        None => (false, None),
    };
    let high_card = hand[4].value;

    if is_straight && is_flush && high_card == 14 {
        Hand::RoyalFlush
    } else if is_straight && is_flush {
        Hand::StraightFlush(high_card)
    } else if is_four {
        Hand::FourOfAKind(quads.unwrap())
    } else if is_full_house {
        Hand::FullHouse(full_house_trips.unwrap(), full_house_pair.unwrap())
    } else if is_flush {
        Hand::Flush(high_card)
    } else if is_straight {
        Hand::Straight(high_card)
    } else if is_three {
        Hand::ThreeOfAKind(three_trips.unwrap())
    } else if is_two_pair {
        Hand::TwoPair(high_pair.unwrap(), low_pair.unwrap())
    } else if is_pair {
        Hand::Pair(pair.unwrap())
    } else {
        Hand::HighCard(high_card)
    }
}

pub struct Game {
    hole: [Card; 2],
    flop: [Card; 3],
    turn: Option<Card>,
    river: Option<Card>,
}

impl Game {
    fn get_used_cards(&self, mine: bool) -> Vec<Card> {
        let mut used_cards: Vec<Card> = Vec::new();
        used_cards.extend(self.flop);
        if let Some(turn) = self.turn {
            used_cards.push(turn);
        }
        if let Some(river) = self.river {
            used_cards.push(river);
        }
        // If we are looking from our perspective, then include our cards
        if mine {
            used_cards.extend(self.hole);
        }
        used_cards
    }
    
    fn get_unused_cards(&self, mine: bool) -> Vec<Card> {
        let used_cards = self.get_used_cards(mine);
        let mut unused_cards: Vec<Card> = Vec::new();
        for suit in [Suit::Spades, Suit::Diamonds, Suit::Clubs, Suit::Hearts] {
            for value in 2..=14 {
                let current_card = Card::build(suit, value).unwrap();
                if !used_cards.contains(&current_card) {
                    unused_cards.push(current_card);
                }
            }
        }
        unused_cards
    }

    pub fn get_best_hand_frequencies(&self, mine: bool) -> HashMap<Hand, u8> {
        let mut best_hand_frequencies: HashMap<Hand, u8> = HashMap::new();
        let unused_cards = self.get_unused_cards(mine);
        for cards in unused_cards.into_iter().combinations(2) {
            let mut flop = self.flop.to_vec();
            let mut hand = cards.clone();
            hand.append(&mut flop);
            let best_hand = best_hand(&mut hand);
            best_hand_frequencies.entry(best_hand).and_modify(|counter| *counter += 1).or_insert(1);
        }
        best_hand_frequencies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_card() {
        for suit in [Suit::Spades, Suit::Diamonds, Suit::Clubs, Suit::Hearts] {
            for value in 2..=14 {
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

    #[test]
    fn test_best_hand() {
        assert_eq!(best_hand(&mut [
            Card { suit: Suit::Clubs, value: 10 },
            Card { suit: Suit::Clubs, value: 13 },
            Card { suit: Suit::Clubs, value: 14 },
            Card { suit: Suit::Clubs, value: 12 },
            Card { suit: Suit::Clubs, value: 11 },
        ]), Hand::RoyalFlush);
        assert_eq!(best_hand(&mut [
            Card { suit: Suit::Diamonds, value: 3 },
            Card { suit: Suit::Diamonds, value: 7 },
            Card { suit: Suit::Diamonds, value: 4 },
            Card { suit: Suit::Diamonds, value: 5 },
            Card { suit: Suit::Diamonds, value: 6 },
        ]), Hand::StraightFlush(7));
        assert_eq!(best_hand(&mut [
            Card { suit: Suit::Spades, value: 4 },
            Card { suit: Suit::Clubs, value: 4 },
            Card { suit: Suit::Hearts, value: 13 },
            Card { suit: Suit::Hearts, value: 4 },
            Card { suit: Suit::Diamonds, value: 4 },
        ]), Hand::FourOfAKind(4));
        assert_eq!(best_hand(&mut [
            Card { suit: Suit::Hearts, value: 8 },
            Card { suit: Suit::Clubs, value: 3 },
            Card { suit: Suit::Hearts, value: 3 },
            Card { suit: Suit::Diamonds, value: 8 },
            Card { suit: Suit::Spades, value: 8 },
        ]), Hand::FullHouse(8, 3));
        assert_eq!(best_hand(&mut [
            Card { suit: Suit::Clubs, value: 13 },
            Card { suit: Suit::Clubs, value: 14 },
            Card { suit: Suit::Clubs, value: 8 },
            Card { suit: Suit::Clubs, value: 9 },
            Card { suit: Suit::Clubs, value: 4 },
        ]), Hand::Flush(14));
        assert_eq!(best_hand(&mut [
            Card { suit: Suit::Diamonds, value: 2 },
            Card { suit: Suit::Hearts, value: 14 },
            Card { suit: Suit::Spades, value: 3 },
            Card { suit: Suit::Diamonds, value: 5 },
            Card { suit: Suit::Diamonds, value: 4 },
        ]), Hand::Straight(14));
        assert_eq!(best_hand(&mut [
            Card { suit: Suit::Spades, value: 14 },
            Card { suit: Suit::Spades, value: 2 },
            Card { suit: Suit::Clubs, value: 14 },
            Card { suit: Suit::Diamonds, value: 9 },
            Card { suit: Suit::Hearts, value: 14 },
        ]), Hand::ThreeOfAKind(14));
        assert_eq!(best_hand(&mut [
            Card { suit: Suit::Diamonds, value: 11},
            Card { suit: Suit::Spades, value: 11},
            Card { suit: Suit::Hearts, value: 7},
            Card { suit: Suit::Diamonds, value: 4},
            Card { suit: Suit::Clubs, value: 7},
        ]), Hand::TwoPair(11, 7));
        assert_eq!(best_hand(&mut [
            Card {suit: Suit::Hearts, value: 4 },
            Card {suit: Suit::Spades, value: 2 },
            Card {suit: Suit::Hearts, value: 13 },
            Card {suit: Suit::Spades, value: 9 },
            Card {suit: Suit::Clubs, value: 13 },
        ]), Hand::Pair(13));
        assert_eq!(best_hand(&mut [
            Card {suit: Suit::Clubs, value: 8 },
            Card {suit: Suit::Clubs, value: 7 },
            Card {suit: Suit::Hearts, value: 3 },
            Card {suit: Suit::Diamonds, value: 2 },
            Card {suit: Suit::Spades, value: 4 },
        ]), Hand::HighCard(8));
    }

    #[test]
    fn test_get_used_cards() {
        let hole = [Card { suit: Suit::Diamonds, value: 12 }, Card { suit: Suit::Clubs, value: 11 }];
        let flop = [Card { suit: Suit::Spades, value: 10 }, Card { suit: Suit::Hearts, value: 8 }, Card { suit: Suit::Clubs, value: 3 }];
        let game = Game { hole, flop, turn: None, river: None };
        let used_cards = game.get_used_cards(false);
        for hole_card in hole {
            assert!(!used_cards.contains(&hole_card));
        }
        for flop_card in flop {
            assert!(used_cards.contains(&flop_card));
        }
        let used_cards = game.get_used_cards(true);
        for hole_card in hole {
            assert!(used_cards.contains(&hole_card));
        }
        for flop_card in flop {
            assert!(used_cards.contains(&flop_card));
        }

        let turn = Card { suit: Suit::Diamonds, value: 2 };
        let game = Game { hole, flop, turn: Some(turn), river: None };
        let used_cards = game.get_used_cards(false);
        for hole_card in hole {
            assert!(!used_cards.contains(&hole_card));
        }
        for flop_card in flop {
            assert!(used_cards.contains(&flop_card));
        }
        assert!(used_cards.contains(&turn));
        let used_cards = game.get_used_cards(true);
        for hole_card in hole {
            assert!(used_cards.contains(&hole_card));
        }
        for flop_card in flop {
            assert!(used_cards.contains(&flop_card));
        }
        assert!(used_cards.contains(&turn));

        let river = Card { suit: Suit::Clubs, value: 7 };
        let game = Game {hole, flop, turn: Some(turn), river: Some(river) };
        let used_cards = game.get_used_cards(false);
        for hole_card in hole {
            assert!(!used_cards.contains(&hole_card));
        }
        for flop_card in flop {
            assert!(used_cards.contains(&flop_card));
        }
        assert!(used_cards.contains(&turn));
        assert!(used_cards.contains(&river));
        let used_cards = game.get_used_cards(true);
        for hole_card in hole {
            assert!(used_cards.contains(&hole_card));
        }
        for flop_card in flop {
            assert!(used_cards.contains(&flop_card));
        }
        assert!(used_cards.contains(&turn));
        assert!(used_cards.contains(&river));
    }

    #[test]
    fn test_get_unused_cards() {
        let hole = [Card { suit: Suit::Diamonds, value: 12 }, Card { suit: Suit::Clubs, value: 11 }];
        let flop = [Card { suit: Suit::Spades, value: 10 }, Card { suit: Suit::Hearts, value: 8 }, Card { suit: Suit::Clubs, value: 3 }];
        let game = Game { hole, flop, turn: None, river: None };
        let unused_cards = game.get_unused_cards(false);
        for hole_card in hole {
            assert!(unused_cards.contains(&hole_card));
        }
        for flop_card in flop {
            assert!(!unused_cards.contains(&flop_card));
        }
        let unused_cards = game.get_unused_cards(true);
        for hole_card in hole {
            assert!(!unused_cards.contains(&hole_card));
        }
        for flop_card in flop {
            assert!(!unused_cards.contains(&flop_card));
        }

        let turn = Card { suit: Suit::Diamonds, value: 2 };
        let game = Game { hole, flop, turn: Some(turn), river: None };
        let unused_cards = game.get_unused_cards(false);
        for hole_card in hole {
            assert!(unused_cards.contains(&hole_card));
        }
        for flop_card in flop {
            assert!(!unused_cards.contains(&flop_card));
        }
        assert!(!unused_cards.contains(&turn));
        let unused_cards = game.get_unused_cards(true);
        for hole_card in hole {
            assert!(!unused_cards.contains(&hole_card));
        }
        for flop_card in flop {
            assert!(!unused_cards.contains(&flop_card));
        }
        assert!(!unused_cards.contains(&turn));

        let river = Card { suit: Suit::Clubs, value: 7 };
        let game = Game {hole, flop, turn: Some(turn), river: Some(river) };
        let unused_cards = game.get_unused_cards(false);
        for hole_card in hole {
            assert!(unused_cards.contains(&hole_card));
        }
        for flop_card in flop {
            assert!(!unused_cards.contains(&flop_card));
        }
        assert!(!unused_cards.contains(&turn));
        assert!(!unused_cards.contains(&river));
        let unused_cards = game.get_unused_cards(true);
        for hole_card in hole {
            assert!(!unused_cards.contains(&hole_card));
        }
        for flop_card in flop {
            assert!(!unused_cards.contains(&flop_card));
        }
        assert!(!unused_cards.contains(&turn));
        assert!(!unused_cards.contains(&river));
    }
}
