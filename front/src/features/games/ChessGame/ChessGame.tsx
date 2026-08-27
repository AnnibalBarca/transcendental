import GameLayout from '../components/GameLayout';
import styles from '@/features/games/ChessGame/components/styles/ChessGame.module.css';
import ChessBoard from './components/ChessBoard';
import PromotionPicker from './components/PromotionPicker';
import PlayerCard from './components/PlayerCard';
import Cards from './components/Cards';
import { ThemeButton } from '@/features/play/components/ThemeButton';
import { SquareButton } from '@/features/play/components/SquareButton';
import { X } from 'lucide-react';
import { useChessGame } from './hooks/useChessGame';
import { roomService } from '@/features/room/services/roomService';
import { profileService } from '@/features/player/services/profileService';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import type { PieceType } from './types/chessTypes';
import type { CardId } from './types/cardTypes';
import { useAuth } from '@/features/auth/hooks/useAuth';

export default function ChessGame() {
	const {
		pieces,
		squareModifiers,
		myColor,
		currentPlayer,
		isMyTurn,
		status,
		winner,
		whiteTimeMs,
		blackTimeMs,
		whiteUsername,
		blackUsername,
		whitePicture,
		blackPicture,
		error,
		connected,
		endReason,
		firstMoveCountdown,
		lastMove,
		hand,
		cardState,
		cardActionUsedThisTurn,
		sendMove,
		sendCard,
		sendDiscard,
	} = useChessGame();

	const [showQuitModal, setShowQuitModal] = useState(false);
	const [pendingPromotion, setPendingPromotion] = useState<{ from: string; to: string } | null>(null);
	const [pendingCardTarget, setPendingCardTarget] = useState<CardId | null>(null);
	const [myPicture, setMyPicture] = useState<string | null>(null);
	const [eloMap, setEloMap] = useState<{ white?: number; black?: number }>({});
	const navigate = useNavigate();
	const { checkAuth } = useAuth();

	useEffect(() => {
    if (connected) return;


    const intervalId = setInterval(() => {
        checkAuth().catch(() => {});
    }, 5000);

    return () => clearInterval(intervalId);
	}, [connected, checkAuth]);

	useEffect(() => {
		if (!whiteUsername && !blackUsername) return;
		let cancelled = false;
		Promise.all([
			whiteUsername
				? profileService.getPublicProfile(whiteUsername).then((p) => p.ranked_elo).catch(() => 0)
				: Promise.resolve(0),
			blackUsername
				? profileService.getPublicProfile(blackUsername).then((p) => p.ranked_elo).catch(() => 0)
				: Promise.resolve(0),
		]).then(([w, b]) => {
			if (!cancelled) setEloMap({ white: w, black: b });
		});
		return () => {
			cancelled = true;
		};
	}, [whiteUsername, blackUsername]);

	useEffect(() => {
		fetch("/api/user/profile-picture", {
			method: "GET",
			credentials: "include",
		})
			.then((res) => (res.ok ? res.json() : null))
			.then((data) => {
				if (data && typeof data.picture_id === "string" && data.picture_id) {
					setMyPicture(data.picture_id);
				}
			})
			.catch(() => {});
	}, []);

	const isReverse = myColor === "black";

	const topColor = isReverse ? "white" : "black";
	const bottomColor = isReverse ? "black" : "white";

	const topName = topColor === "white" ? (whiteUsername || "White") : (blackUsername || "Black");
	const bottomName = bottomColor === "white" ? (whiteUsername || "White") : (blackUsername || "Black");

	const topPicture = topColor === "white" ? whitePicture : blackPicture;
	const bottomPicture =
		(bottomColor === "white" ? whitePicture : blackPicture) ||
		(myColor === bottomColor ? myPicture : null);

	const endTitle = winner
		? myColor && winner === myColor
			? "You win!"
			: "You lose"
		: "Game ended";

	const handleMove = useCallback((from: string, to: string) => {
		const piece = pieces[from];
		const isPromotion =
			piece?.type === "pawn" &&
			((myColor === "white" && to.endsWith("8")) ||
				(myColor === "black" && to.endsWith("1")));

		if (isPromotion) {
			setPendingPromotion({ from, to });
			return;
		}
		sendMove(from, to);
	}, [pieces, myColor, sendMove]);

	const handlePromotionSelect = useCallback((type: PieceType) => {
		setPendingPromotion((pp) => {
			if (!pp) return pp;
			sendMove(pp.from, pp.to, type as string);
			return null;
		});
	}, [sendMove]);

	const handleQuit = useCallback(async () => {
		try {
			await roomService.leaveGame();
		} catch (e) {
		}
		window.location.href = "/play";
	}, []);

	const handleCardPlay = useCallback((cardId: CardId, target?: string) => {
		setPendingCardTarget(null);
		sendCard(cardId, target);
	}, [sendCard]);

	const handleTargetSquareSelect = useCallback((square: string) => {
		setPendingCardTarget((pending) => {
			if (!pending) return pending;
			sendCard(pending, square);
			return null;
		});
	}, [sendCard]);

	const quitButton = useMemo(
		() =>
			status !== "ended" ? (
				<SquareButton
					tone="red"
					className="h-14 aspect-square"
					onClick={() => setShowQuitModal(true)}
					aria-label="Quit game"
				>
					<X className="h-6 w-6" />
				</SquareButton>
			) : undefined,
		[status],
	);

	return (
		<GameLayout>
			<div className={styles.game}>
				{!connected && <p>Connecting to game server...</p>}
				{error && <p className={styles.error}>Error: {error}</p>}
				{status === "waiting" && <p>Waiting for opponent...</p>}
				{pendingCardTarget && (
					<div className={styles.targetBanner}>
						Cliquez sur une case de l'echiquier pour jouer {pendingCardTarget.replace(/_/g, " ")}
					</div>
				)}
				<PlayerCard
					username={topName}
					pictureId={topPicture}
					color={topColor}
					isCurrentPlayer={currentPlayer === topColor}
					timeMs={topColor === "white" ? whiteTimeMs : blackTimeMs}
					countdown={topColor === "white" ? firstMoveCountdown : null}
					elo={topColor === "white" ? eloMap.white ?? 0 : eloMap.black ?? 0}
					className={styles.topCard}
					rightSlot={quitButton}
				/>
				<ChessBoard
					pieces={pieces}
					squareModifiers={squareModifiers}
					isReverse={isReverse}
					isMyTurn={isMyTurn && !pendingCardTarget}
					lastMove={lastMove}
					onMove={handleMove}
					pendingCardTarget={pendingCardTarget}
					onTargetSquareSelect={handleTargetSquareSelect}
					deadlyZones={cardState.deadly_zones}
					fogRemaining={cardState.fog_remaining_turns}
				/>
				<PlayerCard
					username={bottomName}
					pictureId={bottomPicture}
					color={bottomColor}
					isCurrentPlayer={currentPlayer === bottomColor}
					timeMs={bottomColor === "white" ? whiteTimeMs : blackTimeMs}
					countdown={bottomColor === "white" ? firstMoveCountdown : null}
					elo={bottomColor === "white" ? eloMap.white ?? 0 : eloMap.black ?? 0}
					className={styles.bottomCard}
				/>
				<Cards
					hand={hand}
					isMyTurn={isMyTurn}
					cardActionUsedThisTurn={cardActionUsedThisTurn}
					onPlayCard={handleCardPlay}
					onDiscardCard={sendDiscard}
					onRequestTarget={(cardId) => setPendingCardTarget(cardId)}
					pendingCardTarget={pendingCardTarget}
				/>
			</div>
			{showQuitModal && (
				<div className={styles.overlay} onClick={() => setShowQuitModal(false)}>
					<div className={styles.quitModal} onClick={(e) => e.stopPropagation()}>
						<h2 className={styles.quitModalTitle}>Abandon game?</h2>
						<p className={styles.quitModalText}>
							Your opponent wins. The game will end.
						</p>
						<div className={styles.quitModalActions}>
							<ThemeButton
								type="button"
								texturePosition="center 98%"
								textureZoom={130}
								onClick={() => setShowQuitModal(false)}
								className="h-11 flex-1 px-5"
							>
								Cancel
							</ThemeButton>
							<ThemeButton
								type="button"
								tone="red"
								texturePosition="center 98%"
								textureZoom={130}
								onClick={handleQuit}
								className="h-11 flex-1 px-5"
							>
								Abandon
							</ThemeButton>
						</div>
					</div>
				</div>
			)}
			{status === "ended" && (
				<div className={styles.overlay}>
					<div className={styles.endCard}>
						<h2 className={styles.endTitle}>{endTitle}</h2>
						{endReason && (
							<p className={styles.endSubtitle}>{endReason}</p>
						)}
						<ThemeButton
							type="button"
							texturePosition="center 98%"
							textureZoom={130}
							onClick={() => navigate("/play")}
							className="h-11 min-w-[200px] px-6"
						>
							Back to Home
						</ThemeButton>
					</div>
				</div>
			)}
			{pendingPromotion && myColor && (
				<PromotionPicker
					color={myColor}
					onSelect={handlePromotionSelect}
					onCancel={() => setPendingPromotion(null)}
				/>
			)}
		</GameLayout>
	);
}
