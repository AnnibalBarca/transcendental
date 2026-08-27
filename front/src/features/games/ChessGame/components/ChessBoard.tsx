import styles from '@/features/games/ChessGame/components/styles/ChessBoard.module.css';
import type { Piece } from '@/features/games/ChessGame/types/chessTypes';
import DroppableSquare from '@/features/games/ChessGame/components/DroppableSquare';
import OverlayContent from '@/features/games/ChessGame/components/OverlayContent';
import { memo, useState } from 'react';
import { snapCenterToCursor } from '@dnd-kit/modifiers';
import { DndContext, DragOverlay, MouseSensor, pointerWithin, TouchSensor, useSensor, useSensors } from '@dnd-kit/core';
import DraggablePiece from './DraggablePiece';
import { renderNinjaCloud, renderSquareEffect } from './utils/pieces';

interface ChessBoardProps {
	pieces: { [key: string]: Piece };
	squareModifiers?: { [key: string]: { effect: string; rarity: number } };
	isReverse?: boolean;
	isMyTurn?: boolean;
	lastMove?: string[];
	onMove?: (from: string, to: string) => void;
	pendingCardTarget?: string | null;
	onTargetSquareSelect?: (square: string) => void;
	deadlyZones?: string[];
	fogRemaining: number;
}

const ChessBoard = memo(function ChessBoard(props: ChessBoardProps) {
	const { pieces, squareModifiers = {}, isReverse = false, isMyTurn = true, lastMove = [], onMove, pendingCardTarget, onTargetSquareSelect, deadlyZones = [], fogRemaining } = props;

	const [moveSelection, setMoveSelection] = useState<string[]>([]);
	const [selectedSquare, setSelectedSquare] = useState<string | null>(null);
	const BOARD_SIZE = 64;
	const BOARD_ARRAY = Array.from({ length: BOARD_SIZE });
	const COLUMNS = "abcdefgh";

	const onDragStart = (e: any) => {
		const { active } = e;
		if (!isMyTurn) return;

		const pieceSquare = Object.keys(pieces).find(key => pieces[key]?.id === active.id);

		if (!pieceSquare) return;

		setSelectedSquare(pieceSquare);
		setMoveSelection(pieces[pieceSquare].moveset || []);
	};

	const onDragEnd = (e: any) => {
		if (!isMyTurn) {
			setMoveSelection([]);
			setSelectedSquare(null);
			return;
		}

		const { active, over } = e;

		if (!over) {
			setMoveSelection([]);
			setSelectedSquare(null);
			return;
		}

		const pieceId = active.id;
		const targetId = over.id;


		if (!targetId) {
			setMoveSelection([]);
			setSelectedSquare(null);
			return;
		}

		const sourceId = Object.keys(pieces).find(key => pieces[key]?.id === pieceId);
		if (!sourceId || sourceId === targetId) {
			setMoveSelection([]);
			setSelectedSquare(null);
			return;
		}

		const movedPiece = pieces[sourceId];
		if (!movedPiece?.moveset || !movedPiece.moveset.includes(targetId)) {
			setMoveSelection([]);
			setSelectedSquare(null);
			return;
		}

		if (onMove) {
			onMove(sourceId, targetId);
		}

		setMoveSelection([]);
		setSelectedSquare(null);
	};

	const mouseSensor = useSensor(MouseSensor, {
		activationConstraint: {
			distance: 5,
		},
	});

	const touchSensor = useSensor(TouchSensor, {
		activationConstraint: {
			delay: 0,
			tolerance: 5,
		},
	});

	const sensors = useSensors(mouseSensor, touchSensor);

	const handleSquareClick = (dest: string) => {
		if (pendingCardTarget) {
			const myColor = isReverse ? "black" : "white";
			const targetPiece = pieces[dest];
			if (pendingCardTarget === "0") {
				if (targetPiece) {
					return;
				}
			}
			if (pendingCardTarget === "13" || pendingCardTarget === "17") {
				if (!targetPiece || targetPiece.color !== myColor) {
					return;
				}
			}
			if (pendingCardTarget === "19") {
				if (!targetPiece || targetPiece.type !== "pawn") {
					return;
				}
			}
			if (pendingCardTarget === "20" || pendingCardTarget === "21" || pendingCardTarget === "22") {
				const expected = pendingCardTarget === "20" ? "knight" : pendingCardTarget === "21" ? "rook" : "bishop";
				if (!targetPiece || targetPiece.color !== myColor || targetPiece.type !== expected) {
					return;
				}
			}
			if (pendingCardTarget === "23" || pendingCardTarget === "29") {
				if (!targetPiece || targetPiece.color !== myColor || targetPiece.type !== "pawn") {
					return;
				}
			}
			if (pendingCardTarget === "27") {
				if (!targetPiece || targetPiece.color !== myColor || targetPiece.type === "king") {
					return;
				}
			}
			if (pendingCardTarget === "30") {
				if (targetPiece) {
					return;
				}
			}
			if (onTargetSquareSelect) {
				onTargetSquareSelect(dest);
			}
			return;
		}

		if (!isMyTurn) return;
		if (!selectedSquare) return;

		const destPiece = pieces[dest];
		const selectedPiece = pieces[selectedSquare];

		if (destPiece && destPiece.color === selectedPiece?.color) {
			setSelectedSquare(dest);
			setMoveSelection(destPiece.moveset || []);
			return;
		}


		if (!selectedPiece) {
			return;
		}

		if (!selectedPiece.moveset || !selectedPiece.moveset.includes(dest)) {
			setSelectedSquare(null);
			setMoveSelection([]);
			return;
		}

		if (onMove) {
			onMove(selectedSquare, dest);
		}

		setSelectedSquare(null);
		setMoveSelection([]);
	};

	const visibleSquares = new Set<string>();

	if (fogRemaining > 0) {
		Object.values(pieces).forEach(piece => {
			if (piece?.moveset) {
				piece.moveset.forEach(square => visibleSquares.add(square));
			}
		});
	}

	return (
		<>
			<DndContext onDragStart={onDragStart} onDragEnd={onDragEnd} sensors={sensors} collisionDetection={pointerWithin}>
				<div className={styles.chessBoard}>
					{
						BOARD_ARRAY.map((_, index) => {
							let x = index % 8;
							let y = Math.floor(index / 8);

							if (isReverse) {
								x = 7 - x;
								y = 7 - y;
							}
							const isBlack = (x + y) % 2 === 1;
							const currentSquare = `${COLUMNS[x]}${8 - y}`;
							const currentPiece = pieces[currentSquare];
							const squareMod = squareModifiers[currentSquare];
							const isHighlighted = moveSelection ? moveSelection.includes(currentSquare) : false;
							const isLastMove = lastMove ? lastMove.includes(currentSquare) : false;
							const isDeadlyZone = deadlyZones.includes(currentSquare) || squareMod?.effect === "deadlyzone";
							const isBattlefield = squareMod?.effect === "battlefield";
							const hasFog = fogRemaining > 0 && !visibleSquares.has(currentSquare) && !currentPiece;

							return (
								<DroppableSquare
									key={currentSquare}
									square={currentSquare}
									isBlack={isBlack}
									isHighlighted={isHighlighted}
									hasPiece={!!currentPiece}
									isLastMove={isLastMove}
									isSelected={selectedSquare === currentSquare}
									isDeadlyZone={isDeadlyZone}
									isTargetMode={!!pendingCardTarget}
									hasFog={hasFog}
									battlefieldModifier={isBattlefield ? squareMod : undefined}
									onSquareClick={() => handleSquareClick(currentSquare)}
								>
									{
										isReverse
											? x === 7 && <span className={styles.coordY}>{8 - y}</span>
											: x === 0 && <span className={styles.coordY}>{8 - y}</span>
									}

									{
										isReverse
											? y === 0 && <span className={styles.coordX}>{COLUMNS[x]}</span>
											: y === 7 && <span className={styles.coordX}>{COLUMNS[x]}</span>
									}
								{squareMod &&
									renderSquareEffect(squareMod.effect, squareMod.rarity)}
							{currentPiece &&
									<DraggablePiece
										onPieceClick={() => {
											if (pendingCardTarget) return;
											if (!isMyTurn) return;
											setSelectedSquare(currentSquare);
											setMoveSelection(currentPiece.moveset!);
										}}
											setMoveSelection={setMoveSelection}
										piece={currentPiece}
										draggableId={currentPiece.id}
										disabled={!!pendingCardTarget}
									/>}
								{currentPiece && renderNinjaCloud(currentPiece)}
								</DroppableSquare>
							);
						})}
				</div>
				<DragOverlay modifiers={[snapCenterToCursor]} dropAnimation={null}>
					<OverlayContent pieces={pieces} />
				</DragOverlay>
			</DndContext>
		</>
	);
});

export default ChessBoard;
