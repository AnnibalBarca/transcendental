import { useState } from "react";
import { OfferCarousel, type OfferItem } from "./OfferCarousel";
import { GameClosedPopup } from "./GameClosedPopup";
import { JoinGamePanel } from "./JoinGamePanel";
import { MatchmakingPanel } from "./MatchmakingPanel";
import { useOffers } from "@/features/play/hooks/useOffers";
import {
	PLAY_BLOCK_SPACED,
	PLAY_SCROLL,
	PLAY_SPACER,
} from "./playLayout";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { useMatchmaking } from "@/features/games/hooks/useMatchmaking";

interface PlayViewProps {
	onOfferClick?: (offer: OfferItem) => void;
	onJoinClick?: () => void;
}

export function PlayViewMatchmaking({
	onOfferClick,
	onJoinClick,
}: PlayViewProps) {
	const { userState, roomId } = useAuth();
	const { cancel } = useMatchmaking();
	const [cancelling, setCancelling] = useState(false);
	const offers = useOffers();

	const handleQuitGame = () => {};

	const handleCancel = async () => {
		setCancelling(true);
		await cancel();
		setCancelling(false);
	};

	const isMatchmaking = userState === "matchmaking";
	const isPlaying = userState === "playing" && roomId && roomId !== "0";

	return (
		<div className={PLAY_SCROLL}>
			<div className={PLAY_BLOCK_SPACED}>
				<OfferCarousel
					offers={offers}
					autoPlayInterval={4000}
					onOfferClick={onOfferClick}
				/>
			</div>

			<div className={PLAY_BLOCK_SPACED}>
				{isPlaying ? (
					<JoinGamePanel onJoin={onJoinClick ?? (() => {})} />
				) : isMatchmaking ? (
					<MatchmakingPanel onCancel={handleCancel} cancelling={cancelling} />
				) : (
					<JoinGamePanel onJoin={onJoinClick ?? (() => {})} />
				)}
			</div>

			<div className={PLAY_SPACER} />

			{isPlaying && (
				<GameClosedPopup
					onRejoin={onJoinClick ?? (() => {})}
					onQuit={handleQuitGame}
				/>
			)}
		</div>
	);
}
