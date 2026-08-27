import type { CardRank, StackedCards, CardSuit } from '@/features/games/testgame/types/cardTypes';

const generateDeckCards = () => {
	const suits = ['hearts', 'diamonds', 'clubs', 'spades'] as const;
	const ranks = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13] as const;

	const deckCards: { rank: CardRank; suit: CardSuit }[] = [];

	for (const rank of ranks) {
		for (const suit of suits) {
			deckCards.push({ rank, suit });
		}
	}

	return deckCards;
};

export const randomisedDeck = () => {
	const deck = generateDeckCards();
	for (let i = deck.length - 1; i > 0; i--) {
		const j = Math.floor(Math.random() * (i + 1));
		[deck[i], deck[j]] = [deck[j], deck[i]];
	}
	return deck;
}

export const groupsCardsByRank = (cards: { rank: CardRank; suit: CardSuit }[]) => {
	const transformedCards: { rank: CardRank; suits: CardSuit[] }[] = [];

	for (const card of cards) {
		const existingCard = transformedCards.find(c => c.rank === card.rank);
		if (existingCard) {
			existingCard.suits.push(card.suit);
		} else {
			transformedCards.push({ rank: card.rank, suits: [card.suit] });
		}
	}

	return transformedCards;
}

export const getNumberOfCards = (cardsGroup: StackedCards[]) => {
	return cardsGroup.reduce((total, card) => total + card.suits.length, 0);
};

export const addCardsToGroup = (cardsGroup: StackedCards[], rank: CardRank, suits: CardSuit[]) => {
	const existingCard = cardsGroup.find(c => c.rank === rank);
	if (existingCard) {
		existingCard.suits.push(...suits);
	} else {
		cardsGroup.push({ rank, suits });
	}
};

export const getRank = (cardsGroup: StackedCards[], rank: CardRank) => {
	return cardsGroup.find(card => card.rank === rank);
}