import { useDndContext } from '@dnd-kit/core';
import { renderPiece } from '@/features/games/ChessGame/components/utils/pieces';
import type { Piece } from '@/features/games/ChessGame/types/chessTypes';
import styles from '@/features/games/ChessGame/components/styles/ChessBoard.module.css';

export default function OverlayContent({ pieces }: { pieces: { [key: string]: Piece } }) {
	const { active } = useDndContext();
	const activeId = active?.id;

	const activePiece: Piece | null | undefined = activeId
		? Object.values(pieces).find(p => p && p.id === activeId)
		: null;

	if (!activePiece) return null;

	return (
		<div className={styles.pieceOverlay} >
			{renderPiece(activePiece)}
		</div>
	);
}