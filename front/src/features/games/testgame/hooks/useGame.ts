import type { Card, CardRank, StackedCards } from '@/features/games/testgame/types/cardTypes';

import {  addCardsToGroup, getRank, groupsCardsByRank, randomisedDeck } from '@/features/games/testgame/utils/gameUtils';
import { useEffect, useState } from 'react';
import { findCompletedBook, handleAskCard } from '../utils/gameLogic';
import { useGameEventListener } from './useGameEventListener';
import type { GameEventMap } from '../utils/gameEvents';

type GamePhase = 'WAITING' | 'STARTING' | 'START_TURN' | 'PLAYING' | 'DRAWING' | 'ANIMATION' | 'GAME_OVER';

export type GameState = {
	deck: Card[];
	playerCards: StackedCards[];
	playerScore: number;
	opponentCards: StackedCards[];
	opponentScore: number;
	gameMessage: string;
	turn: 'player' | 'opponent';
	moveCount: number;
	animatingBook: {
		rank: CardRank;
		owner: 'player' | 'opponent';
	} | null;
	phase: GamePhase;
	currentAction: {rank: CardRank | null, asker: 'player' | 'opponent'} | null;
}

function createInitialGameState(initialCardNumber: number) {

	const shuffledDeck = randomisedDeck();
	const playerCards: Card[] = [];

	for (let i = 0; i < initialCardNumber; i++) {
		const card = shuffledDeck.pop();
		if (card) {
			playerCards.push(card);
		}
	}

	const opponentCards: Card[] = [];

	for (let i = 0; i < initialCardNumber; i++) {
		const card = shuffledDeck.pop();
		if (card) {
			opponentCards.push(card);
		}
	}

	const playerGroupCards: StackedCards[] = groupsCardsByRank(playerCards);
	const opponentGroupCards: StackedCards[] = groupsCardsByRank(opponentCards);


	return {
		deck: shuffledDeck,
		playerCards: playerGroupCards,
		opponentCards: opponentGroupCards,
		playerScore: 0,
		opponentScore: 0,
		gameMessage: 'Game started !',
		turn: 'player' as 'player' | 'opponent',
		moveCount: 0,
		animatingBook: null,
		phase: 'START_TURN' as GamePhase,
		currentAction: null,
	};
}

export function useGame(initialCardNumber: number)
{
	const [gameState, setGameState] = useState<GameState>(
		() => createInitialGameState(initialCardNumber)
	);

	const askCard = (rank: CardRank, asker: 'player' | 'opponent') => {

		if (gameState.phase !== 'PLAYING') return; 

		if (gameState.turn !== asker) {
			setGameState(prevState => ({
				...prevState,
				gameMessage: `It's not ${asker}'s turn!`,
			}));
			return;
		}

		const currentAskerCards = asker === 'player' ?
			gameState.playerCards
			:
			gameState.opponentCards;

		if (getRank(currentAskerCards, rank) === undefined) {
			return ;
		}

		setGameState(prev => ({
			...prev,
			gameMessage: `${asker} is asking for rank ${rank}...`,
			phase: 'ANIMATION',
			currentAction: {rank, asker}
		}));

		setTimeout(() => {
			setGameState(prev => handleAskCard(prev));
		}, 2000);
	}

	const confirmBookAnimation = (e: GameEventMap['ANIMATION:BOOK_COMPLETE']) =>
	{

		setGameState(prev => {


			if(!prev.animatingBook) return prev;

			const {rank, owner} = prev.animatingBook;

			const pCards = prev.playerCards.filter(c => !(owner === 'player' && c.rank === rank));
			const oCards = prev.opponentCards.filter(c => !(owner === 'opponent' && c.rank === rank));

			const newPlayerScore = owner === 'player' ? prev.playerScore + 1 : prev.playerScore;
			const newOpponentScore = owner === 'opponent' ? prev.opponentScore + 1 : prev.opponentScore;

			return {
				...prev,
				playerCards: pCards,
				opponentCards: oCards,
				playerScore: newPlayerScore,
				opponentScore: newOpponentScore,
				animatingBook: null,
				gameMessage: '',
				phase: 'START_TURN',
			};
		})
	}
	useGameEventListener({
		onBookComplete: confirmBookAnimation,
	})

	const handleDrawing = () => {

		if (gameState.deck.length === 0)
		{
			setGameState(prev => ({
				...prev,
				phase: 'START_TURN'
			}))
			return ;
		}

		const newDeck = [...gameState.deck]
		const drawnCard = newDeck.pop()!;

		setTimeout(() => {
			setGameState(prev => {

				if (!prev.currentAction) return prev;

				const { rank, asker } = prev.currentAction;
				const opponent = asker === 'player' ? 'opponent' : 'player';

				const pCards = prev.playerCards.map(c => ({ ...c, suits: [...c.suits] }));
				const oCards = prev.opponentCards.map(c => ({ ...c, suits: [...c.suits] }));

				const askerCards = asker === 'player' ? pCards : oCards;

				addCardsToGroup(askerCards, drawnCard.rank, [drawnCard.suit]);

				if (rank === null) {
					return {
						...prev,
						phase: 'START_TURN',
						gameMessage: `${asker} drew a card !`,
						deck: newDeck,
						playerCards: pCards,
						opponentCards: oCards,
						turn: prev.turn,
						currentAction: null
					}
				}

				const luckyDraw = drawnCard.rank === rank;
				const nextTurn = luckyDraw ? asker : opponent;

				const message = luckyDraw ? `${asker} draw the requested card ${rank} !`
					:
					`${asker} draw a card. Turn passes to ${opponent}`

				return {
					...prev,
					phase: 'START_TURN',
					gameMessage: message,
					deck: newDeck,
					playerCards: pCards,
					opponentCards: oCards,
					turn: nextTurn,
					currentAction: null
				}
			});
		}, 1500);

	}

	const handleOpponentAI = () => {
		if (gameState.turn !== 'opponent') return;

		if (gameState.opponentCards.length === 0) return;

		setTimeout(() => {
			const randomCard = gameState.opponentCards[Math.floor(Math.random() * gameState.opponentCards.length)];


			askCard(randomCard.rank, 'opponent');
		}, 1200);
	}

	const handleStartTurn = () => {
	

		const hand: StackedCards[] = gameState.turn === 'player' ? gameState.playerCards : gameState.opponentCards;
		const deck: Card[] = gameState.deck;

		const completedBook = findCompletedBook(gameState.playerCards, gameState.opponentCards);

		if (completedBook)
		{
			setGameState(prev =>( {
				...prev,
				phase: 'ANIMATION',
				gameMessage: `${completedBook.owner} completed a book of ${completedBook.rank} !`,
				animatingBook: completedBook,
			}))
		}

		if (deck.length === 0 
			&& (gameState.playerCards.length === 0 || gameState.opponentCards.length === 0))
		{
			setGameState(prev => ({
				...prev,
				phase: 'GAME_OVER',
			}));
			return;
		}

		if (hand.length === 0 && deck.length === 0)
		{
			setGameState(prev => ({
				...prev,
				phase: 'GAME_OVER'
			}));
			return ;
		}
		
		if (hand.length === 0)
		{
			setGameState(prev => ({
				...prev,
				phase: 'DRAWING',
				currentAction: {rank: null, asker: gameState.turn}
			}));
			return;
		}

		setGameState(prev => ({
			...prev,
			phase: 'PLAYING'
		}));
	}

	useEffect(() => {


		if (gameState.phase === 'STARTING' || gameState.phase === 'WAITING' || gameState.phase === 'ANIMATION')  return ;

		if (gameState.phase === 'GAME_OVER')
		{
			return ;
		}

		if (gameState.phase === 'START_TURN')
		{
			handleStartTurn();
			return ;
		}

		if (gameState.phase === 'DRAWING')
		{
			handleDrawing();
			return ;
		}

		if (gameState.turn === 'opponent') {
			handleOpponentAI();
			return ;
		}

	}, [gameState.phase]);

	return {
		deckLength: gameState.deck.length,
		playerCards: gameState.playerCards,
		playerScore: gameState.playerScore,
		opponentCards: gameState.opponentCards,
		opponentScore: gameState.opponentScore,
		gameMessage: gameState.gameMessage,
		moveCount: gameState.moveCount,
		animatingBook: gameState.animatingBook,
		askCard,
		turn: 'player',
	};
}