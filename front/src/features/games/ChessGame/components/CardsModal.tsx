import styles from '@/features/games/ChessGame/components/styles/CardsModal.module.css';
import { useTranslation } from 'react-i18next';
import { useEffect, useRef, useState } from 'react';
import Card from './Card';
import { ThemeButton } from '@/features/play/components/ThemeButton';
import type { CardId, CardRegister } from '@/features/games/ChessGame/types/cardTypes';
import { CHESS_CARDS, cardImageUrl } from '@/features/games/ChessGame/types/cardTypes';

interface CardsModalProps {
	isOpen: boolean;
	close: () => void;
	hand: CardRegister[];
	activeIndex: number | null;
	isMyTurn: boolean;
	cardActionUsedThisTurn: boolean;
	onPlayCard: (cardId: CardId, target?: string) => void;
	onDiscardCard: (cardId: CardId) => void;
	onRequestTarget: (cardId: CardId) => void;
	pendingCardTarget: CardId | null;
}

export default function CardsModal({
	isOpen,
	close,
	hand,
	activeIndex,
	isMyTurn,
	cardActionUsedThisTurn,
	onPlayCard,
	onDiscardCard,
	onRequestTarget,
	pendingCardTarget,
}: CardsModalProps) {
	const { t } = useTranslation();
	const carouselRef = useRef<HTMLDivElement>(null);
	const [centeredIndex, setCenteredIndex] = useState<number>(activeIndex ?? 0);
	const [isDragging, setIsDragging] = useState(false);
	const dragStart = useRef<{ x: number; left: number } | null>(null);
	const centeredCard = hand[centeredIndex];
	const centeredData = centeredCard ? CHESS_CARDS[centeredCard.id] : null;
	const isPendingTarget = centeredCard ? pendingCardTarget === centeredCard.id : false;

	const updateCenteredIndex = () => {
		if (!carouselRef.current || hand.length === 0) return;
		const container = carouselRef.current;
		const containerCenter = container.scrollLeft + container.clientWidth / 2;
		let closestIndex = 0;
		let closestDistance = Infinity;

		Array.from(container.children).forEach((child, index) => {
			const rect = (child as HTMLElement).getBoundingClientRect();
			const childCenter = rect.left + rect.width / 2;
			const containerRect = container.getBoundingClientRect();
			const childCenterRelative = childCenter - containerRect.left + container.scrollLeft;
			const distance = Math.abs(childCenterRelative - containerCenter);
			if (distance < closestDistance) {
				closestDistance = distance;
				closestIndex = index;
			}
		});

		setCenteredIndex(closestIndex);
	};

	useEffect(() => {
		if (isOpen && carouselRef.current && activeIndex !== null) {
			const targetCard = carouselRef.current.children[activeIndex] as HTMLElement;
			if (targetCard) {
				targetCard.scrollIntoView({
					behavior: 'instant',
					inline: 'center',
					block: 'nearest',
				});
				setCenteredIndex(activeIndex);
			}
		}
	}, [isOpen, activeIndex]);

	useEffect(() => {
		const container = carouselRef.current;
		if (!container || !isOpen) return;

		const handleScroll = () => {
			window.requestAnimationFrame(updateCenteredIndex);
		};

		container.addEventListener('scroll', handleScroll, { passive: true });
		updateCenteredIndex();
		return () => container.removeEventListener('scroll', handleScroll);
	}, [isOpen, hand.length]);

	useEffect(() => {
		const handleKeyDown = (event: KeyboardEvent) => {
			if (event.key === 'Escape') {
				close();
			}
		};

		if (isOpen) {
			window.addEventListener('keydown', handleKeyDown);
		}

		return () => {
			window.removeEventListener('keydown', handleKeyDown);
		};
	}, [isOpen, close]);

	if (!isOpen || activeIndex === null || hand.length === 0 || !centeredData) return null;

	const isPlayable = centeredCard?.playable !== false;

	const actionBlocked = !isMyTurn || cardActionUsedThisTurn;

	const handlePlay = () => {
		if (actionBlocked || !centeredCard || !isPlayable) return;
		if (centeredData.needsTarget) {
			onRequestTarget(centeredCard.id);
		} else {
			onPlayCard(centeredCard.id);
		}
	};

	const handleDiscard = () => {
		if (actionBlocked || !centeredCard) return;
		onDiscardCard(centeredCard.id);
		close();
	};

	const onPointerDown = (e: React.PointerEvent) => {
		if (!carouselRef.current) return;
		dragStart.current = { x: e.clientX, left: carouselRef.current.scrollLeft };
		setIsDragging(true);
		try {
			carouselRef.current.setPointerCapture(e.pointerId);
		} catch {
			/* ignore */
		}
	};

	const onPointerMove = (e: React.PointerEvent) => {
		if (!isDragging || !dragStart.current || !carouselRef.current) return;
		const dx = e.clientX - dragStart.current.x;
		carouselRef.current.scrollLeft = dragStart.current.left - dx;
	};

	const endDrag = () => {
		dragStart.current = null;
		setIsDragging(false);
	};

	return (
		<div onClick={close} className={styles.overlay}>
			<div onClick={(e) => e.stopPropagation()} className={styles.modal}>
				<div
					ref={carouselRef}
					className={`${styles.cards} ${isDragging ? styles.dragging : ''}`}
					onPointerDown={onPointerDown}
					onPointerMove={onPointerMove}
					onPointerUp={endDrag}
					onPointerCancel={endDrag}
					onPointerLeave={endDrag}
				>
					{hand.map((rawCard, index) => {
						const isCentered = index === centeredIndex;
						return (
							<div
								key={`${rawCard.id}-${index}`}
								className={`${styles.cardWrapper} ${isCentered ? styles.centeredCard : styles.dimmedCard}`}
							>
								<Card
									description={t(`cards.${rawCard.id}.description`)}
									zoom={true}
									backgroundUrl={cardImageUrl(rawCard.id)}
									title={t(`cards.${rawCard.id}.title`)}
									index={index}
								/>
							</div>
						);
					})}
				</div>

				<div className={styles.actions}>
					{isPlayable && centeredData?.needsTarget ? (
						<ThemeButton
							type="button"
							texturePosition="center 98%"
							textureZoom={130}
							onClick={handlePlay}
							disabled={actionBlocked || isPendingTarget}
							className="h-12 min-w-[220px] px-6"
						>
							{isPendingTarget
								? t("games.selectSquare")
								: isMyTurn && !cardActionUsedThisTurn
									? t("games.chooseSquare")
									: t("games.actionUsed")}
						</ThemeButton>
					) : isPlayable ? (
						<ThemeButton
							type="button"
							texturePosition="center 98%"
							textureZoom={130}
							onClick={handlePlay}
							disabled={actionBlocked}
							className="h-12 min-w-[220px] px-6"
						>
							{isMyTurn && !cardActionUsedThisTurn
								? t("games.play", { title: t(`cards.${centeredCard?.id}.title`) })
								: t("games.actionUsed")}
						</ThemeButton>
					) : null}
					<ThemeButton
						type="button"
						texturePosition="center 98%"
						textureZoom={130}
						onClick={handleDiscard}
						disabled={actionBlocked}
						className="h-12 min-w-[220px] px-6"
					>
						{isMyTurn && !cardActionUsedThisTurn
							? t("games.discard", { title: t(`cards.${centeredCard?.id}.title`) })
							: t("games.actionUsed")}
					</ThemeButton>
					<ThemeButton
						type="button"
						texturePosition="center 98%"
						textureZoom={130}
						onClick={close}
						className="h-10 min-w-[120px] px-5 text-sm"
					>
						{t("shop.close")}
					</ThemeButton>
				</div>
			</div>
		</div>
	);
}