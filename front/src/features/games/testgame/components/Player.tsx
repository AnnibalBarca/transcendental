import styles from '@/features/games/testgame/components/styles/Player.module.css'
import Cards from './Cards'

import type { CardRank, CardSuit } from '@/features/games/testgame/types/cardTypes';
import Button from '@/components/ui/Button/Button';
import { useState } from 'react';

interface PlayerProps {
	cards?: { rank: CardRank; suits: CardSuit[] }[];
	score: number;
	animatingBook: {
		rank: CardRank;
		owner: 'player' | 'opponent';
	} | null;
	askCard: (rank: CardRank, asker: 'player' | 'opponent') => void;
}

export default function Player(props: PlayerProps) {

	const [askingRank, setAskingRank] = useState<CardRank | null>(null);

	const handleAskingRank = () => {
		if (!askingRank) {
			return;
		}
		props.askCard(askingRank, 'player');
		setAskingRank(null);
	}


	return (
		<div className={styles.player}>

			<div className={styles.cardContainer}>
				{props.cards ? props.cards.map((card) => (
					<Cards 
						key={card.rank}
						onClick={() => { setAskingRank(card.rank) }}
						rank={card.rank}
						suits={card.suits} 
						isBook={props.animatingBook?.rank === card.rank}
					/>
				)) : null}
			</div>
			{askingRank && (
				<Button onClick={() => { handleAskingRank() }} variant="primary" className={styles.button}>
					Ask for a {askingRank}
				</Button>

			)}
			<p>Score: {props.score}</p>
			<p>Player</p>
			<img src="https://cdn-icons-png.flaticon.com/512/149/149071.png" alt="Player Avatar" style={{ width: '30px', height: '30px', borderRadius: '50%' }} />
		</div>
	)
}