import { useState } from "react";
import { Check } from "lucide-react";
import { getRank, rankImageUrl } from "@/features/play/ranks";

export type GameMode = "ranked";

interface ModeSelectorProps {
	onModeChange?: (mode: GameMode, option: string) => void;
	onSwipeToggle?: (disabled: boolean) => void;
	playerElo?: number;
}

const TIME_OPTIONS = ["5 min", "10 min", "15 min"];

export function ModeSelector({
	onModeChange,
	onSwipeToggle,
	playerElo = 1500,
}: ModeSelectorProps) {
	const [selectedTime, setSelectedTime] = useState(() => {
		const stored = sessionStorage.getItem("chess_time_control");
		return stored && ["5", "10", "15"].includes(stored)
			? `${stored} min`
			: "10 min";
	});

	const rank = getRank(playerElo);

	const handleSelect = (option: string) => {
		setSelectedTime(option);
		onModeChange?.("ranked", option);
		onSwipeToggle?.(false);
	};

	return (
		<div className="flex w-full cursor-pointer justify-between overflow-hidden rounded-[2px] border border-white/10 bg-white/[0.04] backdrop-blur text-white">
			<div className="flex w-[160px] min-w-0 flex-col">
				{TIME_OPTIONS.map((opt) => (
					<button
						key={opt}
						type="button"
						className={`flex w-full flex-1 cursor-pointer items-center justify-center gap-1.5 border border-white/[0.12] bg-white/5 px-5 py-2.5 text-base font-semibold text-white/70 transition-[background,border-color,color] duration-200 first:rounded-tl-[2px] last:rounded-bl-[2px] hover:bg-white/10 hover:text-white ${
							opt === selectedTime
								? "border-[#fbbf24] bg-[rgba(251,191,36,0.18)] text-[#fbbf24]"
								: ""
						}`}
						onClick={() => handleSelect(opt)}
					>
						{opt === selectedTime && <Check size={16} className="text-[#fbbf24]" />}
						{opt}
					</button>
				))}
			</div>

			<div className="flex flex-1 flex-col items-center gap-1 border border-white/[0.08] border-l-0 px-8 py-3 text-white [background:linear-gradient(135deg,rgba(255,255,255,0.06),rgba(255,255,255,0.02))] rounded-r-[2px]">
				<img
					src={rankImageUrl(rank.image)}
					alt={rank.name}
					className="h-[120px] w-[120px] object-contain"
				/>
				<div className="text-[18px] font-bold text-[#fbbf24]">
					{playerElo} ELO
				</div>
			</div>
		</div>
	);
}