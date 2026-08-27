import styles from '@/features/games/testgame/components/styles/Deck.module.css'

interface DeckProps
{
	remainingCards: number;
}

export default function Deck(props: DeckProps)
{
	return (
		<div className={styles.deck}>
			<div className={styles.cardBack}>
				<div className={styles.cardBackInner}>
				</div>
			</div>
			<div className={styles.countCards}>{props.remainingCards}</div>
		</div>
	)
}