import type { GameState } from "../hooks/useGame";
import type { CardRank, StackedCards } from "../types/cardTypes";
import { addCardsToGroup } from "./gameUtils";

function stealCards(stealerCards: StackedCards[], targetCards: StackedCards[], rank: CardRank) : number
{
	const targetCardIndex = targetCards.findIndex(c => c.rank === rank);
	const transferredSuits = targetCards[targetCardIndex].suits;

	targetCards.splice(targetCardIndex, 1);
	addCardsToGroup(stealerCards, rank, transferredSuits);

	return transferredSuits.length;
}

export function findCompletedBook(pCards: StackedCards[], oCards: StackedCards[]) : {rank: CardRank, owner: 'player' | 'opponent'} | null
{
	const playerBook = pCards.find(c => c.suits.length === 4);
	
	if (playerBook) return {rank: playerBook.rank, owner: 'player'};

	const opponentBook = oCards.find(c => c.suits.length === 4);
	
	if (opponentBook) return {rank: opponentBook.rank, owner: 'opponent'};

	return null;
}

export function handleAskCard(prev: GameState) : GameState
{

	if (!prev.currentAction) return prev;

	const {rank, asker} = prev.currentAction;

	const opponent = asker === 'player' ? 'opponent' : 'player';

	const pCards = prev.playerCards.map(c => ({ ...c, suits: [...c.suits] }));
	const oCards = prev.opponentCards.map(c => ({ ...c, suits: [...c.suits] }));

	const stealerHand = opponent === 'opponent' ? pCards : oCards;
	const targetHand = opponent === 'opponent' ? oCards : pCards;
	const hasRank = targetHand.some(c => c.rank === rank);

	if (hasRank)
	{
		const count = stealCards(stealerHand, targetHand, rank!);

		return {
			...prev,
			playerCards: pCards,
			opponentCards: oCards,
			moveCount: prev.moveCount + 1,
			gameMessage: `${asker} took ${count} ${rank}'s from ${opponent}!`,
			currentAction: null,
			phase: 'START_TURN'
		}
	}

	return {
		...prev,
		phase: 'DRAWING',
		gameMessage: 'Go Fish !'
	};
}