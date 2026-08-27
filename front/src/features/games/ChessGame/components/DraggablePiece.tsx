import type { PieceType, Piece } from '@/features/games/ChessGame/types/chessTypes';
import styles from '@/features/games/ChessGame/components/styles/ChessBoard.module.css';
import { useDraggable } from '@dnd-kit/core';
import { renderPiece } from '@/features/games/ChessGame/components/utils/pieces';
import { memo } from 'react';

interface DraggableProps
{
	piece: Piece;
	draggableId: string;
	setMoveSelection: React.Dispatch<React.SetStateAction<string[]>>;
	onPieceClick: () => void;
	disabled?: boolean;
}

const DraggablePiece = memo(function DraggablePiece(props: DraggableProps)
{
	const { setNodeRef, listeners, attributes, isDragging} = useDraggable({id: props.draggableId, disabled: props.disabled});
	const dragProps = props.disabled ? {} : { ...attributes, ...listeners };
	
	return (
		<div
			ref={setNodeRef}
			{...dragProps}
			onClick={props.onPieceClick}
			className={styles.piece}
			style={{
				opacity: isDragging ? 0.5 : 1,
				touchAction: props.disabled ? 'auto' : 'none',
				transition: 'none'
			}}
		>
			{renderPiece(props.piece)}
		</div>
	);
});

export default DraggablePiece;
