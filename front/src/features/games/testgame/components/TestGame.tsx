import styles from '@/features/games/testgame/components/styles/TestGame.module.css'
import Player from './Player'
import Deck from './Deck'
import Opponent from './Opponent'
import { useGame } from '../hooks/useGame';
import { getNumberOfCards } from '../utils/gameUtils';

export default function TestGame()
{
	const { 
		deckLength,
		playerCards,
		opponentCards,
		gameMessage,
		opponentScore,
		playerScore,
		animatingBook,
		askCard,
	} = useGame(7);
	
	return (
		<div className={styles.game}>
			<Opponent animatingBook={animatingBook?.owner === 'opponent' ? animatingBook : null} cardNumber={getNumberOfCards(opponentCards)} score={opponentScore} />
			<div className={styles.gameCenter}>
				<Deck remainingCards={deckLength} />
				<p className={styles.gameInfo}>{gameMessage}</p>
			</div>
			<Player animatingBook={animatingBook?.owner === 'player' ? animatingBook : null} cards={playerCards} askCard={askCard} score={playerScore}/>
			</div>
	)
}