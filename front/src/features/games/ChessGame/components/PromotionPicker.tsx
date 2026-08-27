import styles from '@/features/games/ChessGame/components/styles/PromotionPicker.module.css';
import { renderPiece } from './utils/pieces';
import type { PieceType, PieceColor } from '@/features/games/ChessGame/types/chessTypes';

interface PromotionPickerProps {
	color: PieceColor;
	onSelect: (type: PieceType) => void;
	onCancel: () => void;
}

const CHOICES: PieceType[] = ["queen", "rook", "bishop", "knight"];

export default function PromotionPicker({
	color,
	onSelect,
	onCancel,
}: PromotionPickerProps) {
	return (
		<div className={styles.overlay} onClick={onCancel}>
			<div className={styles.card} onClick={(e) => e.stopPropagation()}>
				<h2 className={styles.title}>Promote pawn</h2>
				<div className={styles.choices}>
					{CHOICES.map((type) => (
						<button
							key={type}
							className={styles.choice}
							onClick={() => onSelect(type)}
							type="button"
							aria-label={`Promote to ${type}`}
						>
							{renderPiece({ id: `${color}_${type}_promo`, type, color })}
						</button>
					))}
				</div>
			</div>
		</div>
	);
}
