import { useNavigate } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { useAuth } from "@/features/auth/hooks/useAuth";
import type { LiveGame } from "@/features/player/hooks/usePlayerEvents";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import { HomeBackground } from "@/components/HomeBackground";

const KIND_LABEL: Record<LiveGame["kind"], string> = {
	ranked: "Classé",
	casual: "Amical",
};

export default function LiveGamesPage() {
	const navigate = useNavigate();
	const { liveGames } = useAuth();

	return (
		<div className="relative flex min-h-screen flex-col items-center overflow-hidden bg-black pt-6 pb-12 text-white">
			<HomeBackground />
			<button
				type="button"
				className="absolute top-4 left-4 z-10 flex h-11 cursor-pointer items-center gap-2 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/80 px-4 text-sm font-semibold text-white transition-colors duration-150 hover:border-[#60a5fa] hover:text-[#60a5fa]"
				onClick={() => navigate(-1)}
			>
				<ArrowLeft className="h-4 w-4" />
				Back
			</button>

			<div className="relative z-10 flex w-full max-w-[min(80%,400px)] flex-col items-center">
				<h1 className="mt-4 text-center text-[1.8rem] font-bold uppercase tracking-[1px] text-white/80">
					Live games
				</h1>
				<p className="mb-5 mt-2 text-[0.9rem] text-[#94a3b8]">
					{liveGames.length === 0
						? "No games in progress"
						: `${liveGames.length} game${liveGames.length > 1 ? "s" : ""} in progress`}
				</p>

				<div className="flex w-full flex-col gap-2.5">
					{liveGames.map((game) => (
						<div
							key={game.game_id}
							className="flex items-center justify-between gap-3 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 px-3.5 py-3"
						>
							<div className="flex min-w-0 flex-col gap-0.5">
								<div className="flex items-center gap-1.5 whitespace-nowrap text-[0.95rem] font-semibold">
									<span className="max-w-[110px] overflow-hidden text-ellipsis">
										{game.player1 ?? "Player 1"}
									</span>
									<span className="text-[0.8rem] font-normal text-[#94a3b8]">
										vs
									</span>
									<span className="max-w-[110px] overflow-hidden text-ellipsis">
										{game.player2 ?? "Player 2"}
									</span>
								</div>
								<span className="text-xs text-[#60a5fa]">
									{KIND_LABEL[game.kind] ?? game.kind}
									{game.label ? ` · ${game.label}` : ""}
								</span>
							</div>
							<ThemeButton
								type="button"
								texturePosition="center 98%"
								textureZoom={130}
								className="px-3.5 py-2 text-[0.85rem] uppercase"
								onClick={() => navigate(`/game/spectate/${game.game_id}`)}
							>
								Watch
							</ThemeButton>
						</div>
					))}
				</div>
			</div>
		</div>
	);
}