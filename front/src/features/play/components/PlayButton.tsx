import { Eye, Plus } from "lucide-react";
import { useTranslation } from "react-i18next";
import { SquareButton } from "./SquareButton";
import { ThemeButton } from "./ThemeButton";

interface PlayButtonProps {
	onJoinClick?: () => void;
	onCreateClick?: () => void;
	onLiveClick?: () => void;
	joinLabel?: string;
	disabled?: boolean;
}

export function PlayButton({
	onJoinClick,
	onCreateClick,
	onLiveClick,
	joinLabel,
	disabled = false,
}: PlayButtonProps) {
	const { t } = useTranslation();
	const label = joinLabel ?? t("home.play");

	return (
		<div className="flex w-full items-center justify-center gap-4">
			<ThemeButton
				type="button"
				onClick={onJoinClick}
				disabled={disabled}
				texturePosition="center 95%"
				textureZoom={100}
				className="h-14 min-w-0 flex-1"
			>
				<span className="text-sm tracking-[2px] uppercase">{label}</span>
			</ThemeButton>

			<SquareButton
				type="button"
				onClick={onCreateClick}
				aria-label={t("home.createGame")}
				texturePosition="center 95%"
				textureZoom={100}
			>
				<Plus className="h-6 w-6 [stroke-width:2.5px]" />
			</SquareButton>

			{onLiveClick && (
				<SquareButton
					type="button"
					onClick={onLiveClick}
					aria-label={t("home.liveGames")}
					texture="/img/carte/beast_of_burden_0.svg"
					texturePosition="center 95%"
					textureZoom={100}
				>
					<Eye className="h-6 w-6 [stroke-width:2px]" />
				</SquareButton>
			)}
		</div>
	);
}
