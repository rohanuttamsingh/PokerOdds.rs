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

fn get_best_hand(hand: &[Card]) -> Hand {
    let mut hand = hand.to_vec();
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
    fn update_frequencies(&self, hand: &[Card], frequencies: &mut HashMap<Hand, u8>) {
        let best_hand = get_best_hand(&hand);
        frequencies.entry(best_hand).and_modify(|counter| *counter += 1).or_insert(1);
    }

    fn update_frequencies_with_unused_cards(&self, hand: &[Card], unused_cards: &[Card], frequencies: &mut HashMap<Hand, u8>) {
        let remaining_length = 5 - hand.len();
        for final_cards in unused_cards.to_vec().into_iter().combinations(remaining_length) {
            let mut this_hand = hand.to_vec();
            this_hand.extend(final_cards);
            self.update_frequencies(&this_hand, frequencies);
        }
    }

    fn update_frequencies_from_starting_cards(&self, starting_cards: &[Card], unused_cards: &[Card], max_unused_cards: usize, frequencies: &mut HashMap<Hand, u8>) {
        let mut remaining_length = 5 - starting_cards.len();
        let mut hand = starting_cards.to_vec();
        if remaining_length == 0 {
            self.update_frequencies(&hand, frequencies);
        } else if let Some(turn) = self.turn {
            hand.push(turn);
            remaining_length -= 1;
            if remaining_length == 0 {
                self.update_frequencies(&hand, frequencies)
            } else if let Some(river) = self.river {
                remaining_length -= 1;
                if remaining_length == 0 {
                    hand.push(river);
                    self.update_frequencies(&hand, frequencies);
                } else if remaining_length <= max_unused_cards {
                    self.update_frequencies_with_unused_cards(&hand, unused_cards, frequencies);
                }
            } else if remaining_length <= max_unused_cards {
                self.update_frequencies_with_unused_cards(&hand, unused_cards, frequencies);
            }
        } else if remaining_length <= max_unused_cards {
            self.update_frequencies_with_unused_cards(&hand, unused_cards, frequencies);
        }
    }

    fn get_used_cards(&self) -> Vec<Card> {
        let mut used_cards = self.hole.to_vec();
        used_cards.extend(self.flop);
        if let Some(turn) = self.turn {
            used_cards.push(turn);
        }
        if let Some(river) = self.river {
            used_cards.push(river);
        }
        used_cards
    }
    
    fn get_unused_cards(&self, used_cards: &[Card]) -> Vec<Card> {
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

    pub fn get_best_hand_frequenicies(&self) -> (HashMap<Hand, u8>, HashMap<Hand, u8>) {
        let mut my_hand_frequencies: HashMap<Hand, u8> = HashMap::new();
        let mut their_hand_frequencies: HashMap<Hand, u8> = HashMap::new();
        let used_cards = self.get_used_cards();
        let unused_cards = self.get_unused_cards(&used_cards);
        // Both of Hole: Only unused cards could be turn and flop
        let starting_cards = &self.hole;
        self.update_frequencies_from_starting_cards(starting_cards, &unused_cards, 2, &mut my_hand_frequencies);
        // 1 of hole: Only unused cards could be turn and flop
        for hole_card in self.hole {
            let starting_cards = &[hole_card];
            self.update_frequencies_from_starting_cards(starting_cards, &unused_cards, 2, &mut my_hand_frequencies);
        }
        // All of Flop: Unused cards could be 2 of turn, flop, or their 2 cards
        let starting_cards = &self.flop;
        self.update_frequencies_from_starting_cards(starting_cards, &unused_cards, 2, &mut their_hand_frequencies);
        // 2 of Flop: Unused cards could be 3 of turn, flop, or their 2 cards
        for starting_cards in self.flop.to_vec().into_iter().combinations(2) {
            self.update_frequencies_from_starting_cards(&starting_cards, &unused_cards, 3, &mut their_hand_frequencies);
        }
        // 1 of Flop: Unused cards could be turn, flop, or their 2 cards
        for flop_card in self.flop {
            let starting_cards = &[flop_card];
            self.update_frequencies_from_starting_cards(starting_cards, &unused_cards, 4, & mut their_hand_frequencies);
        }


        (my_hand_frequencies, their_hand_frequencies)
    }

    pub fn get_best_hand_frequencies(&self) -> (HashMap<Hand, u8>, HashMap<Hand, u8>) {
        // Important note: Some cases appear missing, but these are intentionally left out because
        // they are infeasible, such as using 1 hole card and 1 flop card
        let mut my_hand_frequencies: HashMap<Hand, u8> = HashMap::new();
        let mut their_hand_frequencies: HashMap<Hand, u8> = HashMap::new();
        let mut used_cards = self.hole.to_vec();
        used_cards.append(&mut self.flop.to_vec());
        if let Some(turn) = self.turn {
            used_cards.push(turn);
        }
        if let Some(river) = self.river {
            used_cards.push(river);
        }
        let unused_cards = self.get_unused_cards(&used_cards);
        // Hole + Flop
        let mut hand = self.hole.to_vec();
        hand.extend(&self.flop);
        let best_hand = get_best_hand(&mut hand);
        my_hand_frequencies.insert(best_hand, 1);
        // Hole + 2 of Flop
        let mut hand = self.hole.to_vec();
        for two_of_flop in self.flop.into_iter().combinations(2) {
            hand.extend(two_of_flop);
            if let Some(turn) = self.turn {
                let mut this_hand = hand.clone();
                this_hand.push(turn);
                self.update_frequencies(&this_hand, &mut my_hand_frequencies);
            }
            if let Some(river) = self.river {
                let mut this_hand = hand.clone();
                this_hand.push(river);
                self.update_frequencies(&this_hand, &mut my_hand_frequencies);
            }
            if self.turn.is_none() && self.river.is_none() {
                for final_card in unused_cards.clone() {
                    let mut this_hand = hand.clone();
                    this_hand.push(final_card);
                    self.update_frequencies(&this_hand, &mut my_hand_frequencies);
                }
            }
        }
        // Hole + 1 of Flop
        let mut hand = self.hole.to_vec();
        for one_of_flop in self.flop {
            hand.push(one_of_flop);
            if let Some(turn) = self.turn {
                let mut turn_hand = hand.clone();
                turn_hand.push(turn);
                if let Some(river) = self.river {
                    let mut this_hand = turn_hand.clone();
                    this_hand.push(river);
                    self.update_frequencies(&this_hand, &mut my_hand_frequencies);
                }
                else {
                    for final_card in unused_cards.clone() {
                        let mut this_hand = turn_hand.clone();
                        this_hand.push(final_card);
                        self.update_frequencies(&this_hand, &mut my_hand_frequencies);
                    }
                }
            }
            if self.turn.is_none() && self.river.is_none() {
                for mut final_two_cards in unused_cards.clone().into_iter().combinations(2) {
                    let mut this_hand = hand.clone();
                    this_hand.append(&mut final_two_cards);
                    self.update_frequencies(&this_hand, &mut my_hand_frequencies);
                }
            }
        }
        // 1 of Hole + Flop
        for hole_card in self.hole {
            let mut hand = vec![hole_card];
            hand.extend(&self.flop);
            if let Some(turn) = self.turn {
                let mut this_hand = hand.clone();
                this_hand.push(turn);
                self.update_frequencies(&this_hand, &mut my_hand_frequencies);
            }
            if let Some(river) = self.river {
                let mut this_hand = hand.clone();
                this_hand.push(river);
                self.update_frequencies(&this_hand, &mut my_hand_frequencies);
            }
            if self.turn.is_none() && self.river.is_none() {
                for final_card in unused_cards.clone() {
                    let mut this_hand = hand.clone();
                    this_hand.push(final_card);
                    self.update_frequencies(&this_hand, &mut my_hand_frequencies);
                }
            }
        }
        // 1 of Hole + 2 of Flop
        for hole_card in self.hole {
            let mut hand = vec![hole_card];
            if let Some(turn) = self.turn {
                let mut turn_hand = hand.clone();
                turn_hand.push(turn);
                if let Some(river) = self.river {
                    let mut this_hand = turn_hand.clone();
                    this_hand.push(river);
                    self.update_frequencies(&this_hand, &mut my_hand_frequencies);
                }
            }
            if self.turn.is_none() && self.river.is_none() {
                for two_of_flop in self.flop.into_iter().combinations(2) {
                    hand.extend(two_of_flop);
                    for mut final_two_cards in unused_cards.clone().into_iter().combinations(2) {
                        let mut this_hand = hand.clone();
                        this_hand.append(&mut final_two_cards);
                        self.update_frequencies(&this_hand, &mut my_hand_frequencies);
                    }
                }
            }
        }
        // 3 of Flop
        let hand = self.flop.to_vec();
        if let Some(turn) = self.turn {
            let mut turn_hand = hand.clone();
            turn_hand.push(turn);
            if let Some(river) = self.river {
                let mut this_hand = turn_hand.clone();
                this_hand.push(river);
                self.update_frequencies(&this_hand, &mut their_hand_frequencies);
            }
            for final_card in unused_cards.clone() {
                let mut this_hand = turn_hand.clone();
                this_hand.push(final_card);
                self.update_frequencies(&this_hand, &mut their_hand_frequencies);
            }
        }
        if let Some(river) = self.river {
            let mut river_hand = hand.clone();
            river_hand.push(river);
            for final_card in unused_cards.clone() {
                let mut this_hand = river_hand.clone();
                this_hand.push(final_card);
                self.update_frequencies(&this_hand, &mut their_hand_frequencies);
            }
        }
        for mut final_two_cards in unused_cards.clone().into_iter().combinations(2) {
            let mut this_hand = hand.clone();
            this_hand.append(&mut final_two_cards);
            self.update_frequencies(&this_hand, &mut their_hand_frequencies);
        }
        // 2 of Flop
        for two_of_flop in self.flop.into_iter().combinations(2) {
            let hand = two_of_flop.clone();
            if let Some(turn) = self.turn {
                let mut turn_hand = hand.clone();
                turn_hand.push(turn);
                if let Some(river) = self.river {
                    let mut turn_river_hand = turn_hand.clone();
                    turn_river_hand.push(river);
                    for final_card in unused_cards.clone() {
                        let mut this_hand = turn_river_hand.clone();
                        this_hand.push(final_card);
                        self.update_frequencies(&this_hand, &mut their_hand_frequencies);
                    }
                }
                for mut final_two_cards in unused_cards.clone().into_iter().combinations(2) {
                    let mut this_hand = turn_hand.clone();
                    this_hand.append(&mut final_two_cards);
                    self.update_frequencies(&this_hand, &mut their_hand_frequencies);
                }
            }
            if let Some(river) = self.river {
                let mut river_hand = hand.clone();
                river_hand.push(river);
                for mut final_two_cards in unused_cards.clone().into_iter().combinations(2) {
                    let mut this_hand = river_hand.clone();
                    this_hand.append(&mut final_two_cards);
                    self.update_frequencies(&this_hand, &mut their_hand_frequencies);
                }
            }
        }
        // 1 of Flop
        for one_of_flop in self.flop {
            if let Some(turn) = self.turn {
                if let Some(river) = self.river {
                    let hand = vec![one_of_flop, turn, river];
                    for mut final_two_cards in unused_cards.clone().into_iter().combinations(2) {
                        let mut this_hand = hand.clone();
                        this_hand.append(&mut final_two_cards);
                        self.update_frequencies(&this_hand, &mut their_hand_frequencies);
                    }
                }
            }
        }
        (my_hand_frequencies, their_hand_frequencies)
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
    fn test_get_best_hand() {
        assert_eq!(get_best_hand(&mut [
            Card { suit: Suit::Clubs, value: 10 },
            Card { suit: Suit::Clubs, value: 13 },
            Card { suit: Suit::Clubs, value: 14 },
            Card { suit: Suit::Clubs, value: 12 },
            Card { suit: Suit::Clubs, value: 11 },
        ]), Hand::RoyalFlush);
        assert_eq!(get_best_hand(&mut [
            Card { suit: Suit::Diamonds, value: 3 },
            Card { suit: Suit::Diamonds, value: 7 },
            Card { suit: Suit::Diamonds, value: 4 },
            Card { suit: Suit::Diamonds, value: 5 },
            Card { suit: Suit::Diamonds, value: 6 },
        ]), Hand::StraightFlush(7));
        assert_eq!(get_best_hand(&mut [
            Card { suit: Suit::Spades, value: 4 },
            Card { suit: Suit::Clubs, value: 4 },
            Card { suit: Suit::Hearts, value: 13 },
            Card { suit: Suit::Hearts, value: 4 },
            Card { suit: Suit::Diamonds, value: 4 },
        ]), Hand::FourOfAKind(4));
        assert_eq!(get_best_hand(&mut [
            Card { suit: Suit::Hearts, value: 8 },
            Card { suit: Suit::Clubs, value: 3 },
            Card { suit: Suit::Hearts, value: 3 },
            Card { suit: Suit::Diamonds, value: 8 },
            Card { suit: Suit::Spades, value: 8 },
        ]), Hand::FullHouse(8, 3));
        assert_eq!(get_best_hand(&mut [
            Card { suit: Suit::Clubs, value: 13 },
            Card { suit: Suit::Clubs, value: 14 },
            Card { suit: Suit::Clubs, value: 8 },
            Card { suit: Suit::Clubs, value: 9 },
            Card { suit: Suit::Clubs, value: 4 },
        ]), Hand::Flush(14));
        assert_eq!(get_best_hand(&mut [
            Card { suit: Suit::Diamonds, value: 2 },
            Card { suit: Suit::Hearts, value: 14 },
            Card { suit: Suit::Spades, value: 3 },
            Card { suit: Suit::Diamonds, value: 5 },
            Card { suit: Suit::Diamonds, value: 4 },
        ]), Hand::Straight(14));
        assert_eq!(get_best_hand(&mut [
            Card { suit: Suit::Spades, value: 14 },
            Card { suit: Suit::Spades, value: 2 },
            Card { suit: Suit::Clubs, value: 14 },
            Card { suit: Suit::Diamonds, value: 9 },
            Card { suit: Suit::Hearts, value: 14 },
        ]), Hand::ThreeOfAKind(14));
        assert_eq!(get_best_hand(&mut [
            Card { suit: Suit::Diamonds, value: 11},
            Card { suit: Suit::Spades, value: 11},
            Card { suit: Suit::Hearts, value: 7},
            Card { suit: Suit::Diamonds, value: 4},
            Card { suit: Suit::Clubs, value: 7},
        ]), Hand::TwoPair(11, 7));
        assert_eq!(get_best_hand(&mut [
            Card {suit: Suit::Hearts, value: 4 },
            Card {suit: Suit::Spades, value: 2 },
            Card {suit: Suit::Hearts, value: 13 },
            Card {suit: Suit::Spades, value: 9 },
            Card {suit: Suit::Clubs, value: 13 },
        ]), Hand::Pair(13));
        assert_eq!(get_best_hand(&mut [
            Card {suit: Suit::Clubs, value: 8 },
            Card {suit: Suit::Clubs, value: 7 },
            Card {suit: Suit::Hearts, value: 3 },
            Card {suit: Suit::Diamonds, value: 2 },
            Card {suit: Suit::Spades, value: 4 },
        ]), Hand::HighCard(8));
    }

    #[test]
    fn test_get_used_and_unused_cards() {
        let hole = [Card { suit: Suit::Diamonds, value: 12 }, Card { suit: Suit::Clubs, value: 11 }];
        let flop = [Card { suit: Suit::Spades, value: 10 }, Card { suit: Suit::Hearts, value: 8 }, Card { suit: Suit::Clubs, value: 3 }];
        let game = Game { hole, flop, turn: None, river: None };
        let used_cards = game.get_used_cards();
        let unused_cards = game.get_unused_cards(&used_cards);
        assert_eq!(used_cards.len(), 5);
        assert_eq!(unused_cards.len(), 52 - 5);
        for card in hole {
            assert!(used_cards.contains(&card));
            assert!(!unused_cards.contains(&card));
        }
        for card in flop {
            assert!(used_cards.contains(&card));
            assert!(!unused_cards.contains(&card));
        }

        let turn = Card { suit: Suit::Hearts, value: 4 };
        let game = Game { hole, flop, turn: Some(turn), river: None };
        let used_cards = game.get_used_cards();
        let unused_cards = game.get_unused_cards(&used_cards);
        assert_eq!(used_cards.len(), 6);
        assert_eq!(unused_cards.len(), 52 - 6);
        for card in hole {
            assert!(used_cards.contains(&card));
            assert!(!unused_cards.contains(&card));
        }
        for card in flop {
            assert!(used_cards.contains(&card));
            assert!(!unused_cards.contains(&card));
        }
        assert!(used_cards.contains(&turn));
        assert!(!unused_cards.contains(&turn));

        let river = Card { suit: Suit::Diamonds, value: 9 };
        let game = Game { hole, flop, turn: Some(turn), river: Some(river) };
        let used_cards = game.get_used_cards();
        let unused_cards = game.get_unused_cards(&used_cards);
        assert_eq!(used_cards.len(), 7);
        assert_eq!(unused_cards.len(), 52 - 7);
        for card in hole {
            assert!(used_cards.contains(&card));
            assert!(!unused_cards.contains(&card));
        }
        for card in flop {
            assert!(used_cards.contains(&card));
            assert!(!unused_cards.contains(&card));
        }
        assert!(used_cards.contains(&turn));
        assert!(!unused_cards.contains(&turn));
        assert!(used_cards.contains(&river));
        assert!(!unused_cards.contains(&river));
    }
}
