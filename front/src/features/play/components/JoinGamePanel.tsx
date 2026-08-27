import { Gamepad2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { ThemeButton } from "./ThemeButton";

interface JoinGamePanelProps {
	onJoin: () => void;
}

export function JoinGamePanel({ onJoin }: JoinGamePanelProps) {
	const { t } = useTranslation();
	return (
		<div className="flex flex-col items-center justify-center gap-4 p-8 text-center">
			<div className="text-5xl leading-none">♟️</div>
			<h2 className="m-0 text-2xl font-bold text-white">{t("play.gameReady")}</h2>
			<p className="m-0 text-base text-white/70">
				{t("play.opponentWaiting")}
			</p>
			<ThemeButton
				type="button"
				onClick={onJoin}
				texturePosition="center 98%"
				textureZoom={130}
				className="mt-2 px-8 py-3.5 text-[1.1rem]"
			>
				<Gamepad2 className="h-5 w-5 [stroke-width:2px]" />
				Rejoin game
			</ThemeButton>
		</div>
	);
}
