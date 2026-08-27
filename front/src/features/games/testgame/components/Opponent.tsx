import styles from '@/features/games/testgame/components/styles/Opponent.module.css'
import type { CardRank } from '../types/cardTypes';
import { emitGameEvent } from '../utils/gameEvents';
import { type JSX } from 'react';
import { SpadeSuit } from '@/components/icons/SpadeSuit';
import { ClubSuit } from '@/components/icons/ClubSuit';
import { DiamondSuit } from '@/components/icons/DiamondSuit';
import { HeartSuit } from '@/components/icons/HeartSuit';

interface OpponentProps
{
	cardNumber?: number;
	score: number;
	animatingBook: {
		rank: CardRank;
		owner: 'player' | 'opponent';
	} | null;
}

export default function Opponent(props: OpponentProps) {

	const suitIcons: Record<string, JSX.Element> = {
		'hearts': <HeartSuit fill="red" width={13} />,
		'diamonds': <DiamondSuit fill="red" width={13} />,
		'clubs': <ClubSuit fill="black" width={13} />,
		'spades': <SpadeSuit fill="black" width={13} />
	};

	const isOpponentAnimating = props.animatingBook && props.animatingBook.owner === 'opponent';
	const suits = Object.keys(suitIcons);
	return (
		<div className={styles.opponent}>

			<div className={styles.cardContainer}>
				{props.cardNumber !== 0 && props.cardNumber !== undefined &&
					[...Array(props.cardNumber)].map((_, index) => (
						<div key={index} className={styles.cardBack}>
							<div className={styles.cardBackInner}></div>
						</div>
					))
				}

				{isOpponentAnimating && props.animatingBook && (
                    <div className={styles.flyingBookGroup}>
                        {suits.map((suit, index) => {
                            const isLastCard = index === suits.length - 1;

                            return (
                                <div 
                                    key={suit} 
                                    className={`${styles.card} ${styles.flyToCenter} ${styles.flyToCenter}`}
                                    onAnimationEnd={(e) => {
                                        if (isLastCard && e.animationName.includes('flySpreadAndFade')) {
                                            emitGameEvent({
                                                type: 'ANIMATION:BOOK_COMPLETE',
                                                detail: {
                                                    rank: props.animatingBook!.rank,
                                                    owner: 'opponent'
                                                }
                                            });
                                        }
                                    }}
                                >
                                    <div className={styles.cardIndex}>
                                        <span className={styles.rank}>{props.animatingBook!.rank}</span>
                                        {suitIcons[suit]}
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                )}

			</div>

			<p>Score: {props.score}</p>
			<p>Opponent</p>
			<img src="https://cdn-icons-png.flaticon.com/512/149/149071.png" alt="Opponent Avatar" style={{ width: '30px', height: '30px', borderRadius: '50%' }} />
		</div>
	)
}