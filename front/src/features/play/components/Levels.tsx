import { Crown } from "lucide-react";
import {
	levelFromXp,
	levelProgress,
	MAX_LEVEL,
	MAX_XP,
} from "@/features/play/utils/xp";

interface LevelsProps {
	level?: number;
	progress?: number;
	xp?: number;
}

export function Levels({ level, progress, xp }: LevelsProps) {
	const resolvedLevel = xp != null ? levelFromXp(xp) : level ?? 1;
	const resolvedProgress = xp != null ? levelProgress(xp, resolvedLevel) : progress ?? 0;
	const clampedProgress = Math.max(0, Math.min(100, resolvedProgress));
	const isMax = xp != null ? xp >= MAX_XP : resolvedLevel >= MAX_LEVEL;

	if (isMax) {
		return (
			<div className="inline-flex items-center gap-2 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/90 py-1.5 pr-4 pl-1.5 shadow-[0_4px_20px_rgba(0,0,0,0.4)] backdrop-blur-lg select-none">
				<div className="flex h-5 min-w-5 shrink-0 items-center justify-center gap-0.5 rounded-[2px] bg-linear-to-br from-[#60a5fa] to-[#3b82f6] px-1.5 text-[10px] font-extrabold tracking-tight text-white shadow-[0_2px_8px_rgba(96,165,250,0.5)]">
					<Crown className="size-3" />
					MAX
				</div>
				<span className="text-[10px] font-semibold tracking-wide text-[#93c5fd]">
					Niveau maximum
				</span>
			</div>
		);
	}

	return (
		<div className="inline-flex items-center gap-3 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/90 py-1.5 pr-4 pl-1.5 shadow-[0_4px_20px_rgba(0,0,0,0.4)] backdrop-blur-lg select-none">
			<div className="flex h-5 w-5 shrink-0 items-center justify-center rounded-[2px] bg-linear-to-br from-[#60a5fa] to-[#3b82f6] text-sm font-extrabold text-white shadow-[0_2px_8px_rgba(96,165,250,0.4)]">
				<span>{resolvedLevel}</span>
			</div>

			<div className="h-2 w-20 overflow-hidden rounded-[2px] bg-white/10">
				<div
					className="h-full rounded-[2px] bg-[#60a5fa] transition-[width] duration-[400ms] ease-[cubic-bezier(0.25,1,0.5,1)]"
					style={{ width: `${clampedProgress}%` }}
				/>
			</div>
		</div>
	);
}
