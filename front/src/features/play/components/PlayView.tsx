import { OfferCarousel, type OfferItem } from "./OfferCarousel";
import { ModeSelector, type GameMode } from "./ModeSelector";
import { PlayButton } from "./PlayButton";
import { useOffers } from "@/features/play/hooks/useOffers";
import {
	PLAY_BLOCK_SPACED,
	PLAY_BUTTON_WRAP,
	PLAY_LOBBY,
	PLAY_SCROLL,
	PLAY_SPACER,
} from "./playLayout";

interface PlayViewProps {
	onOfferClick?: (offer: OfferItem) => void;
	onSwipeToggle?: (disabled: boolean) => void;
	onJoinClick?: () => void;
	onCreateClick?: () => void;
	onLiveClick?: () => void;
	onModeChange?: (mode: GameMode, option: string) => void;
	joinLabel?: string;
	disabled?: boolean;
	actionArea?: React.ReactNode;
	footer?: React.ReactNode;
	hideModeSelector?: boolean;
	playerElo?: number;
}

export function PlayView({
	onOfferClick,
	onSwipeToggle,
	onJoinClick,
	onCreateClick,
	onLiveClick,
	onModeChange,
	joinLabel = "Jouer",
	disabled = false,
	actionArea,
	footer,
	hideModeSelector = false,
	playerElo,
}: PlayViewProps) {
	const offers = useOffers();

	return (
		<div className={PLAY_SCROLL}>
			<div className={PLAY_BLOCK_SPACED}>
				<OfferCarousel
					offers={offers}
					autoPlayInterval={4000}
					onOfferClick={onOfferClick}
				/>
			</div>

			{!hideModeSelector && (
				<div className={PLAY_LOBBY}>
					<ModeSelector
						onSwipeToggle={onSwipeToggle}
						onModeChange={onModeChange}
						playerElo={playerElo}
					/>
				</div>
			)}

			<div className={PLAY_BUTTON_WRAP}>
				{actionArea ?? (
					<PlayButton
						onJoinClick={onJoinClick}
						onCreateClick={onCreateClick}
						onLiveClick={onLiveClick}
						joinLabel={joinLabel}
						disabled={disabled}
					/>
				)}
			</div>

			{footer}

			<div className={PLAY_SPACER} />
		</div>
	);
}
