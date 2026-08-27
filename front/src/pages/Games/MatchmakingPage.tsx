import { useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useMatchmaking } from "@/features/games/hooks/useMatchmaking";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import { HomeBackground } from "@/components/HomeBackground";

export default function MatchmakingPage() {
	const { t } = useTranslation();
	const navigate = useNavigate();
	const { userState, roomId } = useAuth();
	const { status, join, cancel } = useMatchmaking();
	const startedRef = useRef(false);
	const prevUserStateRef = useRef<string | null>(null);

	const timeControl = sessionStorage.getItem("chess_time_control") ?? "10";

	useEffect(() => {
		if (startedRef.current) return;
		startedRef.current = true;
		join(timeControl);
	}, [join, timeControl]);

	useEffect(() => {
		if (userState === "playing" && roomId) {
			navigate("/game/chess");
		}
	}, [userState, roomId, navigate]);

	useEffect(() => {
		const prev = prevUserStateRef.current;
		prevUserStateRef.current = userState;

		const wasInGameFlow =
			prev === "matchmaking" || prev === "playing" || prev === "waiting";
		const isNowNone = userState === "none";

		if (startedRef.current && wasInGameFlow && isNowNone) {
			navigate("/play");
		}
	}, [userState, navigate]);

	const handleCancel = async () => {
		await cancel();
		navigate("/play");
	};

	return (
		<div className="relative flex min-h-screen items-center justify-center overflow-hidden bg-black p-4">
			<HomeBackground />
			<div className="relative z-10 flex min-w-[300px] flex-col items-center gap-6 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/80 p-8 text-center backdrop-blur">
				<h1 className="text-[1.75rem] font-bold text-white">
					{t("play.matchmaking")}
				</h1>

				{status.state === "loading" ? (
					<p className="text-base text-[#cccccc]">{t("play.joiningQueue")}</p>
				) : status.state === "error" ? (
					<>
						<p className="text-base text-[#ff6b6b]">{status.error}</p>
						<ThemeButton
							type="button"
							texturePosition="center 98%"
							textureZoom={130}
							className="px-6 py-3 text-base"
							onClick={() => join(timeControl)}
						>
							{t("play.retry")}
						</ThemeButton>
					</>
				) : (
					<>
						<div className="h-12 w-12 animate-[spin_1s_linear_infinite] rounded-full border-4 border-white/20 border-t-[#60a5fa]" />
						<p className="text-base text-[#cccccc]">{t("play.searching")}</p>
						<ThemeButton
							type="button"
							texturePosition="center 98%"
							textureZoom={130}
							className="px-6 py-3 text-base"
							onClick={handleCancel}
						>
							{t("play.cancel")}
						</ThemeButton>
					</>
				)}
			</div>
		</div>
	);
}