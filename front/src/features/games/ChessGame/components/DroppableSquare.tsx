import styles from '@/features/games/ChessGame/components/styles/ChessBoard.module.css';
import { useDroppable } from '@dnd-kit/core';
import { memo } from 'react';

interface DroppableProps
{
	isBlack: boolean;
	square: string;
	children: React.ReactNode;
	isHighlighted: boolean;
	onSquareClick: () => void;
	isSelected: boolean;
	hasPiece: boolean;
	isLastMove: boolean;
	isDeadlyZone?: boolean;
	isTargetMode?: boolean;
	hasFog: boolean;
	battlefieldModifier?: any;
}

const DroppableSquare = memo(function DroppableSquare(
{
	isBlack,
	children, 
	square,
	isHighlighted,
	onSquareClick,
	hasPiece,
	isSelected,
	isLastMove,
	isDeadlyZone,
	isTargetMode,
	hasFog,
	battlefieldModifier,
}:	DroppableProps
)
{
	const {setNodeRef, isOver} = useDroppable({id: square});
	const formattedBgUrl = `${import.meta.env.VITE_IMAGE_MINIO}/card-effects/fog_0.gif`;

	return (
		<div
			ref={setNodeRef}
			onClick={onSquareClick}
			style={{ backgroundImage: hasFog ? `url(${formattedBgUrl})` : 'none'}}

			className={`
					${styles.square} 
					${isBlack ? styles.black : styles.white} 
					${isOver ? styles.over : ''} 
					${isHighlighted ? styles.highlight : ''}
					${isSelected ? styles.selected : ''}
					${hasPiece ? styles.hasPiece : styles.emptySquare}
					${isLastMove ? styles.lastMove : styles.emptySquare}
					${isDeadlyZone ? styles.deadlyZone : ''}
					${isTargetMode ? styles.targetMode : ''}
					${hasFog ? styles.hasFog : ''}
					${battlefieldModifier ? styles.battlefieldModifier : ''}
				`}
		>
				{children}
		</div>
	);
});

export default DroppableSquare;