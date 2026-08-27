import type { LucideIcon } from "lucide-react";
import { motion } from "framer-motion";
import { Store, Shirt, Gamepad2, Users, Settings } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useFriendContext } from "@/features/friends/context/FriendContext";

interface NavItem {
	id: string;
	icon: LucideIcon;
	labelKey: string;
	badge?: string;
}

const NAV_ITEMS: NavItem[] = [
	{ id: "boutique", icon: Store, labelKey: "nav.shop" },
	{ id: "vestiaire", icon: Shirt, labelKey: "nav.wardrobe" },
	{ id: "jouer", icon: Gamepad2, labelKey: "nav.play" },
	{ id: "social", icon: Users, labelKey: "nav.social" },
	{ id: "settings", icon: Settings, labelKey: "nav.settings" },
];

interface NavbarProps {
	activeIndex: number;
	setActiveIndex: (index: number) => void;
	floatIndex: number;
}

export function Navbar({
	activeIndex,
	setActiveIndex,
	floatIndex,
}: NavbarProps) {
	const { t } = useTranslation();
	const { unreadCounts } = useFriendContext();
	const totalUnread = Object.values(unreadCounts).reduce((a, b) => a + b, 0);
	const itemWidth = 100 / NAV_ITEMS.length;
	const basePillWidth = itemWidth + 8;

	const lastIndex = NAV_ITEMS.length - 1;
	const distanceToEdge = Math.min(floatIndex, lastIndex - floatIndex);
	const edgeFactor = Math.min(1, 0.8 + Math.max(0, distanceToEdge) * 0.2);

	const pillWidth = basePillWidth * edgeFactor;
	const centerPercent = floatIndex * itemWidth + itemWidth / 2;
	const pillLeft = Math.max(
		0,
		Math.min(100 - pillWidth, centerPercent - pillWidth / 2),
	);

	return (
		<nav className="fixed right-0 bottom-0 left-0 z-50 flex h-20 items-stretch border-t border-[#334155]/60 bg-[#0f172a]/90 shadow-[0_-10px_40px_rgba(0,0,0,0.6)] backdrop-blur-xl">
			<motion.div
				className="pointer-events-none absolute top-0.5 bottom-0.5 rounded-[2px] bg-blue-900/60"
				style={{ left: `${pillLeft}%`, width: `${pillWidth}%` }}
			/>
			{NAV_ITEMS.map((item, index) => {
				const Icon = item.icon;
				const isActive = index === activeIndex;
				const distance = Math.abs(floatIndex - index);
				return (
					<button
						key={item.id}
						onClick={() => setActiveIndex(index)}
						className="relative flex flex-1 cursor-pointer items-center justify-center bg-transparent text-white"
						type="button"
					>
						{item.id === "social" && totalUnread > 0 && (
							<span className="absolute -top-2 -right-2 z-[1] rounded-full bg-linear-to-r from-pink-500 to-rose-500 px-1.5 py-0.5 text-[9px] font-bold text-white">
								{totalUnread > 99 ? "99+" : totalUnread}
							</span>
						)}
						{item.badge && item.id !== "social" && (
							<span className="absolute -top-2 -right-2 z-[1] rounded-full bg-linear-to-r from-pink-500 to-rose-500 px-1.5 py-0.5 text-[9px] font-bold text-white">{item.badge}</span>
						)}
						<motion.div
							className="relative flex flex-col items-center justify-center gap-0.5"
							animate={{
								scale: isActive ? 1.1 : 1,
								y: isActive ? -4 : 0,
							}}
							transition={{ type: "spring", stiffness: 300, damping: 30 }}
						>
							<Icon
								className="h-6 w-6"
								style={{
									opacity: distance < 1 ? 1 - distance * 0.5 : 0.5,
								}}
							/>
							<motion.span
								className="overflow-hidden text-[11px] font-semibold text-white"
								animate={{
									opacity: isActive ? 1 : 0,
									y: isActive ? 0 : 8,
									height: isActive ? "auto" : 0,
								}}
								transition={{ type: "spring", stiffness: 300, damping: 30 }}
							>
								{t(item.labelKey)}
							</motion.span>
						</motion.div>
					</button>
				);
			})}
		</nav>
	);
}
