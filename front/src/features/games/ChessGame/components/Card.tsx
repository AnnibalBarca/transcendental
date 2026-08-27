import { useState } from 'react'
import styles from '@/features/games/ChessGame/components/styles/Card.module.css'

interface CardProps {
	handleCardClick?: (index: number) => void;
	index: number;
	title: string;
	backgroundUrl: string;
	fallbackUrl?: string;
	zoom: boolean;
	description?: string;
	rarity?: number;
}

// Halo selon la rareté : 0 = aucune, 1 = Épique, 2 = Légendaire.
const RARITY_BOX_SHADOW = [
	"",
	"0 0 12px rgba(167,139,250,0.7), inset 0 0 6px rgba(167,139,250,0.5)",
	"0 0 14px rgba(251,191,36,0.8), inset 0 0 7px rgba(251,191,36,0.6)",
];

export default function Card({
	handleCardClick,
	index,
	title,
	backgroundUrl,
	fallbackUrl,
	zoom,
	description,
	rarity = 0,
}: CardProps) {
	const [src, setSrc] = useState(backgroundUrl);
	const formattedUrl = src;

	return (
		<div
			onClick={handleCardClick ? () => handleCardClick(index) : undefined}
			className={`${styles.card} ${zoom ? styles.zoom : styles.unZoomed}`}
			style={{ boxShadow: RARITY_BOX_SHADOW[rarity] ?? RARITY_BOX_SHADOW[0] }}
		>
			<img
				src={formattedUrl}
				alt=""
				className={styles.cardBg}
				onError={() => {
					if (fallbackUrl && src !== fallbackUrl) setSrc(fallbackUrl);
				}}
			/>
			<div className={styles.cardDarken} />
			<div className={`${styles.cardHeader} ${zoom ? styles.zoom : styles.unZoomed}`}>
				<p className={`${styles.cardTitle} ${zoom ? styles.zoom : styles.unZoomed}`}>{title}</p>
			</div>
			{zoom && (
				<div className={`${styles.cardDescription}`}>
					<p>{description}</p>
				</div>
			)}
		</div>
	);
}
