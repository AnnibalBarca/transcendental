export type CardSuit = 'hearts' | 'diamonds' | 'clubs' | 'spades';
export type CardRank = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13;

export type StackedCards = { rank: CardRank; suits: CardSuit[] };
export type Card = { rank: CardRank; suit: CardSuit };