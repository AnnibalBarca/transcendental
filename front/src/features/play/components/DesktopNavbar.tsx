import type { LucideIcon } from "lucide-react";
import { Store, Shirt, Gamepad2, Settings } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Levels } from "@/features/play/components/Levels";
import { Wallet } from "@/features/play/components/Wallet";

interface NavItem {
	id: string;
	index: number;
	icon: LucideIcon;
	labelKey: string;
}

const NAV_ITEMS: NavItem[] = [
	{ id: "jouer", index: 2, icon: Gamepad2, labelKey: "nav.play" },
	{ id: "vestiaire", index: 1, icon: Shirt, labelKey: "nav.wardrobe" },
	{ id: "boutique", index: 0, icon: Store, labelKey: "nav.shop" },
	{ id: "settings", index: 4, icon: Settings, labelKey: "nav.settings" },
];

interface DesktopNavbarProps {
	activeIndex: number;
	setActiveIndex: (index: number) => void;
	level?: number;
	progress?: number;
	xp?: number;
	wallet?: number | null;
}

export function DesktopNavbar({
	activeIndex,
	setActiveIndex,
	level,
	progress,
	xp,
	wallet,
}: DesktopNavbarProps) {
	const { t } = useTranslation();

	return (
		<header className="relative z-30 flex h-16 shrink-0 items-center justify-between gap-6 border-b border-[#334155]/60 bg-[#0f172a]/90 px-6 shadow-[0_10px_40px_rgba(0,0,0,0.6)] backdrop-blur-xl">
			<nav className="flex items-center gap-1.5">
				{NAV_ITEMS.map((item) => {
					const Icon = item.icon;
					const isActive = item.index === activeIndex;
					return (
						<button
							key={item.id}
							type="button"
							onClick={() => setActiveIndex(item.index)}
							className={`relative flex cursor-pointer items-center gap-2 rounded-[2px] px-4 py-2.5 text-sm font-semibold transition-colors duration-150 ${
								isActive
									? "bg-blue-900/60 text-white"
									: "text-white/55 hover:bg-white/[0.06] hover:text-white"
							}`}
						>
							<Icon className="h-5 w-5" />
							<span>{t(item.labelKey)}</span>
						</button>
					);
				})}
			</nav>

			<div className="flex shrink-0 items-center gap-3">
				<Levels level={level} progress={progress} xp={xp} />
				<Wallet balance={wallet ?? 0} />
			</div>
		</header>
	);
}
