import styles from '@/features/games/ChessGame/components/styles/PlayerCard.module.css';
import ProfilePicture from "@/components/ui/ProfilePicture";
import { memo } from "react";

interface PlayerCardProps {
	username?: string;
	elo?: number;
	avatarUrl?: string;
	pictureId?: string | null;
	color: "white" | "black";
	isCurrentPlayer: boolean;
	timeMs: number;
	countdown?: number | null;
	className?: string;
	rightSlot?: React.ReactNode;
}

function formatTimer(ms: number): string {
	const totalSeconds = Math.max(0, Math.floor(ms / 1000));
	const minutes = Math.floor(totalSeconds / 60);
	const seconds = totalSeconds % 60;
	return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
}

const PlayerCard = memo(function PlayerCard({
	username = "Opponent",
	elo = 0,
	avatarUrl,
	pictureId,
	color,
	isCurrentPlayer,
	timeMs,
	countdown = null,
	className,
	rightSlot,
}: PlayerCardProps) {
	const label = color === "white" ? "White" : "Black";
	const displayName = username || label;
	const displayElo = elo > 0 ? `ELO ${elo}` : label;
	const IMG_BASE = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";

	return (
		<div
			className={`${styles.playerCard} ${isCurrentPlayer ? styles.activeTurn : ""} ${className ?? ""}`}
		>
			<div className={styles.cardBg}>
				<span
					aria-hidden
					className={styles.texture}
					style={{
						backgroundImage: `url("${IMG_BASE}/carte-png/breakthrough_0.png")`,
					}}
				/>
				<div className={styles.player}>
					<div
						className={styles.playerIcon}
						style={
							avatarUrl && !pictureId
								? {
										backgroundImage: `url(${avatarUrl})`,
										backgroundSize: "cover",
										backgroundPosition: "center",
									}
								: undefined
						}
					>
						{pictureId ? (
							<ProfilePicture
								pictureId={pictureId}
								size={51}
								style={{ width: "100%", height: "100%", borderRadius: "50%" }}
							/>
						) : null}
					</div>
					<div className={styles.playerInfo}>
						<p className={styles.playerName}>{displayName}</p>
						<p>
							<span className={styles.playerRanking}>{displayElo}</span>
						</p>
					</div>
				</div>

				<div className={styles.timer}>
					<p>{countdown !== null ? `00:${countdown.toString().padStart(2, "0")}` : formatTimer(timeMs)}</p>
				</div>
			</div>
			{rightSlot}
		</div>
	);
});

export default PlayerCard;
