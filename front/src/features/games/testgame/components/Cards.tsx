import styles from '@/features/games/testgame/components/styles/Cards.module.css'
import Button from '@/components/ui/Button/Button'
import { DiamondSuit } from '@/components/icons/DiamondSuit';
import type { JSX } from 'react';
import { HeartSuit } from '@/components/icons/HeartSuit';
import { ClubSuit } from '@/components/icons/ClubSuit';
import { SpadeSuit } from '@/components/icons/SpadeSuit';
import { emitGameEvent } from '../utils/gameEvents';

type CardSuit = 'hearts' | 'diamonds' | 'clubs' | 'spades';

interface CardsProps
{
	rank: 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13;
	suits: CardSuit[];
	onClick?: () => void;
	isBook: boolean;
}

export default function Cards(props: CardsProps)
{

	const suitIcons: Record<string, JSX.Element> = {
		'hearts': <HeartSuit fill="red" width={13} />,
		'diamonds': <DiamondSuit fill="red" width={13} />,
		'clubs': <ClubSuit fill="black" width={13} />,
		'spades': <SpadeSuit fill="black" width={13} />
	};

	const cardRanks: Record<number, string> = {
		1: 'A',
		11: 'J',
		12: 'Q',
		13: 'K'
	}

	return (
		<div className={styles.cardFamily}>
			{props.suits.map((suit, index) => (
				<Button 
					onClick={props.onClick} 
					key={index} variant="unstyled" 
					className={`${styles.card}
						${props.isBook ? styles.flyToCenter : ''}
					`}
					onAnimationEnd={(e) =>{
						if (props.isBook && index === props.suits.length - 1 && e.animationName.includes('flySpreadAndFade'))
						{
							emitGameEvent({
								type: 'ANIMATION:BOOK_COMPLETE',
								detail: {
									rank: props.rank,
									owner: 'player'
								}
							});
						}
					}}
				>
					<div className={styles.cardIndex}>
						<span className={styles.rank}>{cardRanks[props.rank] || props.rank}</span>
						{suitIcons[suit]}
					</div>
				</Button>
			))}
		</div>
	);

}