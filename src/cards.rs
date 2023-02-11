#[derive(Debug, Copy, Clone, PartialEq)]
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

#[derive(Debug, PartialEq, PartialOrd)]
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

pub fn best_hand(mut hand: [Card; 5]) -> Hand {
    hand.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    let values: Vec<u8> = hand.iter().map(|x| x.value).collect();
    let suits: Vec<Suit> = hand.iter().map(|x| x.suit).collect();
    let is_straight = match values.iter().as_slice().windows(2).position(|w| w[0] + 1 != w[1]) {
        Some(_) => false,
        None => true,
    };
    let is_flush = match suits.iter().position(|x| *x != suits[0]) {
        Some(_) => false,
        None => true,
    };
    let (is_four, quads) = match values.iter().as_slice().windows(4).any(|w| w.iter().all(|x| *x == w[0])) {
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
    let (is_three, three_trips) = match values.iter().as_slice().windows(3).any(|w| w.iter().all(|x| *x == w[0])) {
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

    if is_flush && values == vec![1, 10, 11, 12, 13] {
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

    #[test]
    fn test_best_hand() {
        assert_eq!(best_hand([
            Card { suit: Suit::Clubs, value: 10 },
            Card { suit: Suit::Clubs, value: 13 },
            Card { suit: Suit::Clubs, value: 1 },
            Card { suit: Suit::Clubs, value: 12 },
            Card { suit: Suit::Clubs, value: 11 },
        ]), Hand::RoyalFlush);
        assert_eq!(best_hand([
            Card { suit: Suit::Diamonds, value: 3 },
            Card { suit: Suit::Diamonds, value: 7 },
            Card { suit: Suit::Diamonds, value: 4 },
            Card { suit: Suit::Diamonds, value: 5 },
            Card { suit: Suit::Diamonds, value: 6 },
        ]), Hand::StraightFlush(7));
        assert_eq!(best_hand([
            Card { suit: Suit::Spades, value: 4 },
            Card { suit: Suit::Clubs, value: 4 },
            Card { suit: Suit::Hearts, value: 13 },
            Card { suit: Suit::Hearts, value: 4 },
            Card { suit: Suit::Diamonds, value: 4 },
        ]), Hand::FourOfAKind(4));
        assert_eq!(best_hand([
            Card { suit: Suit::Hearts, value: 8 },
            Card { suit: Suit::Clubs, value: 3 },
            Card { suit: Suit::Hearts, value: 3 },
            Card { suit: Suit::Diamonds, value: 8 },
            Card { suit: Suit::Spades, value: 8 },
        ]), Hand::FullHouse(8, 3));
        assert_eq!(best_hand([
            Card { suit: Suit::Clubs, value: 13 },
            Card { suit: Suit::Clubs, value: 1 },
            Card { suit: Suit::Clubs, value: 8 },
            Card { suit: Suit::Clubs, value: 9 },
            Card { suit: Suit::Clubs, value: 4 },
        ]), Hand::Flush(13));
        assert_eq!(best_hand([
            Card { suit: Suit::Diamonds, value: 10 },
            Card { suit: Suit::Hearts, value: 9 },
            Card { suit: Suit::Spades, value: 6 },
            Card { suit: Suit::Diamonds, value: 7 },
            Card { suit: Suit::Diamonds, value: 8 },
        ]), Hand::Straight(10));
        assert_eq!(best_hand([
            Card { suit: Suit::Spades, value: 1 },
            Card { suit: Suit::Spades, value: 2 },
            Card { suit: Suit::Clubs, value: 1 },
            Card { suit: Suit::Diamonds, value: 9 },
            Card { suit: Suit::Hearts, value: 1 },
        ]), Hand::ThreeOfAKind(1));
        assert_eq!(best_hand([
            Card { suit: Suit::Diamonds, value: 11},
            Card { suit: Suit::Spades, value: 11},
            Card { suit: Suit::Hearts, value: 7},
            Card { suit: Suit::Diamonds, value: 4},
            Card { suit: Suit::Clubs, value: 7},
        ]), Hand::TwoPair(11, 7));
        assert_eq!(best_hand([
            Card {suit: Suit::Hearts, value: 4 },
            Card {suit: Suit::Spades, value: 2 },
            Card {suit: Suit::Hearts, value: 13 },
            Card {suit: Suit::Spades, value: 9 },
            Card {suit: Suit::Clubs, value: 13 },
        ]), Hand::Pair(13));
        assert_eq!(best_hand([
            Card {suit: Suit::Clubs, value: 8 },
            Card {suit: Suit::Clubs, value: 7 },
            Card {suit: Suit::Hearts, value: 3 },
            Card {suit: Suit::Diamonds, value: 1 },
            Card {suit: Suit::Spades, value: 4 },
        ]), Hand::HighCard(8));
    }
}
