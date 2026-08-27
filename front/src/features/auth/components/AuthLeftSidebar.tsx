import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import NeonSpotGrid from "@/features/auth/components/NeonSpotGrid";

interface Stats {
	matches_played: number;
	users_online: number;
	active_rooms: number;
}

function formatCount(value: number): string {
	if (value >= 1_000_000) {
		const m = value / 1_000_000;
		return `${Number.isInteger(m) ? m : m.toFixed(1)}M+`;
	}
	if (value >= 1_000) {
		const k = value / 1_000;
		return `${Number.isInteger(k) ? k : k.toFixed(1)}K+`;
	}
	return String(value);
}

export default function AuthLeftSidebar() {
	const { t } = useTranslation();
	const [stats, setStats] = useState<Stats | null>(null);
	const ghostRef = useRef<HTMLImageElement>(null);
	const mainRef = useRef<HTMLImageElement>(null);
	const wrapRef = useRef<HTMLDivElement>(null);
	const offset = useRef({ x: 0, y: 0 });
	const reduced = useRef(false);

	useEffect(() => {
		reduced.current = window.matchMedia(
			"(prefers-reduced-motion: reduce)",
		).matches;

		const onMove = (e: MouseEvent) => {
			const rect = wrapRef.current?.getBoundingClientRect();
			if (!rect) return;
			offset.current.x = e.clientX - (rect.left + rect.width / 2);
			offset.current.y = e.clientY - (rect.top + rect.height / 2);
		};

		window.addEventListener("mousemove", onMove);
		return () => window.removeEventListener("mousemove", onMove);
	}, []);

	useEffect(() => {
		if (reduced.current) return;

		let raf = 0;
		const tick = () => {
			const dx = offset.current.x;
			const dy = offset.current.y;

			if (mainRef.current) {
				mainRef.current.style.transform = `translate3d(${(dx * 0.03).toFixed(2)}px, ${(dy * 0.03).toFixed(2)}px, 0)`;
			}
			if (ghostRef.current) {
				ghostRef.current.style.transform = `translate3d(${(-dx * 0.05).toFixed(2)}px, ${(-dy * 0.05).toFixed(2)}px, 0)`;
			}

			raf = requestAnimationFrame(tick);
		};

		raf = requestAnimationFrame(tick);
		return () => cancelAnimationFrame(raf);
	}, []);

	useEffect(() => {
		const loadStats = async () => {
			try {
				const res = await fetch("/api/auth/stats");
				if (res.ok) {
					const data = await res.json();
					if (
						typeof data.matches_played === "number" &&
						typeof data.users_online === "number" &&
						typeof data.active_rooms === "number"
					) {
						setStats(data);
					}
				}
			} catch {
				void 0;
			}
		};
		loadStats();
		const interval = window.setInterval(loadStats, 30_000);
		return () => window.clearInterval(interval);
	}, []);

	const iconSrc = `${import.meta.env.VITE_IMAGE_MINIO}/assets/img_home.png`;

	return (
		<div
			className="relative w-full hidden flex-col justify-around h-full flex-1 border-r-2 border-[#072D3A] text-white lg:flex"
		>
			<div className="absolute inset-0 z-0">
				<NeonSpotGrid />
			</div>

			<div className="flex justify-center w-full relative z-10">
				<div ref={wrapRef} className="relative">
					<img
						ref={mainRef}
						alt="icon"
						src={iconSrc}
						className="relative h-[min(560px,84vh)] w-auto opacity-90"
					/>
					<img
						ref={ghostRef}
						alt=""
						src={iconSrc}
						className="pointer-events-none absolute inset-0 z-10 h-[min(560px,84vh)] w-auto opacity-40"
					/>
				</div>
			</div>

			<div className="flex flex-row items-center justify-around text-center text-[0.9rem] font-bold">
				<div>
					<p className="text-[#06B6D4]">
						{stats ? formatCount(stats.users_online) : "0"}
					</p>
					<p className="text-[#52545A]">{t("auth.onlineNow")}</p>
				</div>
				<div>
					<p className="text-[#06B6D4]">
						{stats ? formatCount(stats.matches_played) : "0"}
					</p>
					<p className="text-[#52545A]">{t("auth.matchesPlayed")}</p>
				</div>
				<div>
					<p className="text-[#06B6D4]">
						{stats ? formatCount(stats.active_rooms) : "0"}
					</p>
					<p className="text-[#52545A]">{t("auth.activeRooms")}</p>
				</div>
			</div>
		</div>
	);
}
