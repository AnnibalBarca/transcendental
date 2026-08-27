const IMG_BASE = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";
import { useTranslation } from "react-i18next";

interface MatchmakingPanelProps {
	onCancel: () => void;
	cancelling?: boolean;
}

export function MatchmakingPanel({
	onCancel,
	cancelling = false,
}: MatchmakingPanelProps) {
	const { t } = useTranslation();
	return (
		<div className="flex flex-col items-center justify-center gap-4 p-8 text-center">
			<div className="h-14 w-14 animate-spin rounded-full border-4 border-white/15 border-t-[#60a5fa]" />
			<h2 className="m-0 text-2xl font-bold text-white">
				{t("play.searchingForOpponent")}
			</h2>
			<p className="m-0 text-base text-white/70">
				{t("play.autoStart")}
			</p>
			<button
				className="relative mt-2 cursor-pointer overflow-hidden rounded-[2px] bg-linear-to-br from-blue-900 via-blue-950 to-slate-950 px-6 py-3 text-base font-semibold text-white transition-[filter] duration-150 hover:brightness-95 active:brightness-90 disabled:cursor-not-allowed disabled:opacity-60"
				onClick={onCancel}
				type="button"
				disabled={cancelling}
			>
				<span
					aria-hidden
					className="pointer-events-none absolute inset-0 mix-blend-screen opacity-50"
					style={{
						backgroundImage: `url("${IMG_BASE}/carte/breakthrough_0.svg")`,
						backgroundSize: "300%",
						backgroundPosition: "center 20%",
						filter: "grayscale(100%)",
					}}
				/>
				<span className="relative">
					{cancelling ? t("play.cancelling") : t("play.cancelMatchmaking")}
				</span>
			</button>
		</div>
	);
}