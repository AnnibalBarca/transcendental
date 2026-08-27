import { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { motion } from "framer-motion";
import { Lottie } from "lottie-react";
import { X } from "lucide-react";
import { cardImageUrl, type CardId } from "@/data/cards";
import type { PackReward } from "../services/shopService";
import { cosmeticImageUrl } from "@/utils/cosmeticImage";
import { useCardTilt } from "@/features/shop/hook/useCardTilt";
import { useTranslation } from "react-i18next";
import pk from "./PackOpening.module.css";

// iOS 13+ expose DeviceOrientationEvent.requestPermission, pas dans les autres navigateurs
type DeviceOrientationEventiOS = typeof DeviceOrientationEvent & {
	requestPermission?: () => Promise<"granted" | "denied">;
};

const IMG_BASE =
	(import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";

const COIN_IMG = `${import.meta.env.VITE_IMAGE_MINIO}/assets/RoyalCoins.svg`;

interface RarityDef {
	name: string;
	color: string;
	glow: string;
}

const RARITIES: RarityDef[] = [
	{ name: "shop.rarity.common", color: "#94a3b8", glow: "rgba(148,163,184,0.55)" },
	{ name: "shop.rarity.epic", color: "#a78bfa", glow: "rgba(167,139,250,0.65)" },
	{ name: "shop.rarity.legendary", color: "#fbbf24", glow: "rgba(251,191,36,0.7)" },
];

function rarityOf(
	kind: "skin" | "card",
	price: number,
	rarity?: number,
): RarityDef {
	if (rarity != null && rarity >= 0 && rarity < RARITIES.length) {
		return RARITIES[rarity];
	}

	if (kind === "card") {
		if (price >= 1000) return RARITIES[2];
		if (price >= 800) return RARITIES[1];
		return RARITIES[0];
	}
	if (price >= 250) return RARITIES[2];
	if (price >= 150) return RARITIES[1];
	return RARITIES[0];
}

interface PackOpeningModalProps {
	kind: "skin" | "card";
	rewards: PackReward[];
	wallet: number;
	onClose: () => void;
}

const CARD_W = "w-[min(62vw,300px)]";
const CARD_H = "aspect-[2/3]";

const FLIP_MS = 600;
const LEAVE_MS = 520;

export function PackOpeningModal({
	kind,
	rewards,
	onClose,
}: PackOpeningModalProps) {
	const { t } = useTranslation();
	const [index, setIndex] = useState(0);
	const [flipped, setFlipped] = useState(false);
	const [leaving, setLeaving] = useState(false);
	const [introDone, setIntroDone] = useState(false);

	const lockRef = useRef(false);
	const timerRef = useRef<number | null>(null);

	const total = rewards.length;
	const done = index >= total;
	const current = done ? null : rewards[index];
	const rarity = current
		? rarityOf(kind, current.price, current.rarity)
		: RARITIES[0];

	const {
		elRef: cardRef,
		needsIOSPermission,
		requestIOSPermission,
		handleMouseMove,
		handleMouseLeave,
	} = useCardTilt<HTMLDivElement>(!done && !!current);

	useEffect(() => {
		return () => {
			if (timerRef.current !== null) window.clearTimeout(timerRef.current);
		};
	}, []);

	useEffect(() => {
		if (done) onClose();
	}, [done, onClose]);

	useEffect(() => {
		if (introDone) return;
		const timeoutId = window.setTimeout(() => setIntroDone(true), 3500);
		return () => window.clearTimeout(timeoutId);
	}, [introDone]);

	const schedule = (fn: () => void, ms: number) => {
		if (timerRef.current !== null) window.clearTimeout(timerRef.current);
		timerRef.current = window.setTimeout(() => {
			timerRef.current = null;
			fn();
		}, ms);
	};

	const handleClick = () => {
		if (done || !current || lockRef.current || !introDone) return;

		if (!flipped) {
			lockRef.current = true;
			setFlipped(true);
			schedule(() => {
				lockRef.current = false;
			}, FLIP_MS);
			return;
		}

		lockRef.current = true;
		setLeaving(true);
		schedule(() => {
			setIndex((i) => i + 1);
			setFlipped(false);
			setLeaving(false);
			lockRef.current = false;
		}, LEAVE_MS);
	};

const rewardImage = (reward: PackReward) =>
		kind === "card"
			? cardImageUrl(reward.item_id as CardId)
			: cosmeticImageUrl(reward.item_type, reward.item_id);

	return createPortal(
		<div
			className="fixed inset-0 z-[100] flex items-center justify-center overflow-hidden"
			onClick={handleClick}
		>
			<div className="absolute inset-0 bg-[#0b0b0d]" />

			<button
				type="button"
				onClick={(e) => {
					e.stopPropagation();
					onClose();
				}}
				className="absolute top-4 right-4 z-30 cursor-pointer rounded-full bg-white/10 p-2 text-white/70 transition-colors hover:bg-white/20 hover:text-white"
				aria-label={t("shop.close")}
			>
				<X className="size-5" />
			</button>

{needsIOSPermission && (
				<button
					type="button"
					onClick={(e) => {
						e.stopPropagation();
						requestIOSPermission();
					}}
					className="pointer-events-auto absolute top-4 left-4 z-30 cursor-pointer rounded-full bg-white/10 px-3 py-1.5 text-xs text-white/80 backdrop-blur transition-colors hover:bg-white/20 hover:text-white"
				>
					{t("shop.enable3d")}
				</button>
			)}

			{!introDone ? (
				<div className="flex flex-col items-center gap-6">
					<Lottie
						src="/lottie/pack-open.json"
						loop={false}
						autoplay
						className="h-72 w-72"
						subscriptions={{
							complete: () => setIntroDone(true),
							error: () => setIntroDone(true),
						}}
					/>
					<p className="text-sm font-semibold text-white/70">
						{t("shop.openingPack")}
					</p>
				</div>
			) : (
				<div className="relative flex flex-col items-center">
					<p className="mb-5 text-xs font-bold tracking-[0.3em] text-white/40 uppercase">
						{kind === "card" ? t("shop.card") : t("shop.item")} {index + 1} / {total}
					</p>

					{!done && current && (
						<div
							ref={cardRef}
							className={`relative ${CARD_W} ${CARD_H}`}
							style={{
								perspective: 1200,
								transformStyle: "preserve-3d",
							}}
							onMouseMove={handleMouseMove}
							onMouseLeave={handleMouseLeave}
						>
							<div
								className={pk.tiltWrap}
								style={{ transformStyle: "preserve-3d" }}
							>
								<motion.div
									key={`${index}:${current.item_id}`}
									className={`relative ${CARD_W} ${CARD_H}`}
									style={{
										zIndex: 99,
										transformStyle: "preserve-3d",
									}}
									animate={{
										scale: leaving ? 1.2 : 1,
										opacity: leaving ? 0 : 1,
										y: leaving ? -170 : 0,
									}}
									transition={{
										duration: leaving ? LEAVE_MS / 1000 : 0.3,
										ease: "easeInOut",
									}}
								>
								<motion.button
									type="button"
									className={`relative ${CARD_W} ${CARD_H} cursor-pointer outline-none`}
									style={{
										transformStyle: "preserve-3d",
										borderRadius: 14,
										backgroundColor: "#1c1c26",
										boxShadow:
											"0 0 80px rgba(255,255,255,0.45), 0 0 200px rgba(255,255,255,0.25)",
									}}
									animate={{ rotateY: flipped ? 180 : 0 }}
									transition={{
										duration: FLIP_MS / 1000,
										ease: "easeInOut",
									}}
								>
								<div
									className={`absolute inset-0 ${CARD_W} ${CARD_H} ${pk.face}`}
									style={{ backfaceVisibility: "hidden" }}
								>
									<div className={pk.back}>
										<img
											src={`${IMG_BASE}/card/backcard.svg`}
											alt=""
											className="h-full w-full object-cover"
										/>
									</div>
								</div>

								<div
									className={`absolute inset-0 ${CARD_W} ${CARD_H} ${pk.face}`}
									style={
										{
											backfaceVisibility: "hidden",
											transform: "rotateY(180deg)",
											"--glow": rarity.color,
										} as React.CSSProperties
									}
								>
									{kind === "card" ? (
										<>
												<div
													className="absolute inset-0"
													style={{
														backgroundImage: rewardImage(current)
															? `url(${rewardImage(current)})`
															: undefined,
														backgroundSize: "cover",
														backgroundPosition: "center",
														backgroundRepeat: "no-repeat",
														backgroundColor: "#181236",
														borderRadius: 14,
													}}
												/>
												<p
													className="absolute top-7 right-0 left-0 z-[1] px-3 text-center font-bold uppercase"
													style={{
														color: rarity.color,
														fontSize: "0.95rem",
														textShadow: "0 1px 3px rgba(0,0,0,0.85)",
													}}
												>
													{current.title}
												</p>
												<div className="absolute top-2 left-2 z-[1]">
													<span
														className="rounded-full px-2 py-0.5 text-[10px] font-extrabold tracking-widest uppercase"
														style={{
															backgroundColor: `${rarity.color}26`,
															color: rarity.color,
															border: `1px solid ${rarity.color}66`,
															backdropFilter: "blur(4px)",
														}}
													>
														{t(rarity.name)}
													</span>
												</div>
											</>
										) : (
											<div className="flex h-full w-full flex-col items-center justify-center gap-3 p-4">
												{rewardImage(current) ? (
													<img
														src={rewardImage(current)}
														alt={current.title}
														className="h-1/2 max-h-40 w-auto object-contain drop-shadow-[0_8px_24px_rgba(0,0,0,0.7)]"
													/>
												) : (
													<div className="flex h-1/2 items-center justify-center text-6xl font-black text-white/70">
														?
													</div>
												)}
												<span
													className="px-2 text-center text-base leading-snug font-extrabold"
													style={{ color: rarity.color }}
												>
													{current.title}
												</span>
												<span className="rounded-full px-2.5 py-1 text-[11px] font-extrabold tracking-widest text-white/80 uppercase">
													{t(rarity.name)}
												</span>
											</div>
										)}

										<div className="absolute bottom-2 left-1/2 z-[1] -translate-x-1/2">
											{current.is_duplicate ? (
												<span className="flex items-center gap-1 rounded-full bg-rose-500/85 px-2.5 py-1 text-xs font-extrabold text-white shadow-[0_2px_8px_rgba(0,0,0,0.4)]">
													{t("shop.duplicate", { amount: current.refunded })}
													<img src={COIN_IMG} alt="" className="h-3.5 w-3.5" />
												</span>
											) : (
												<span className="rounded-full bg-emerald-500/85 px-2.5 py-1 text-xs font-extrabold text-white shadow-[0_2px_8px_rgba(0,0,0,0.4)]">
													{t("shop.new")}
												</span>
											)}
										</div>

										{kind === "card" && (
											<div className={pk.shine} aria-hidden="true" />
										)}
									</div>
							</motion.button>
								</motion.div>
							</div>
					</div>
					)}

					{!done && current && (
						<p className="mt-6 text-xs text-white/40">
{flipped && !leaving
							? t("shop.clickContinue")
							: t("shop.clickFlip")}
						</p>
					)}
				</div>
			)}
		</div>,
		document.body,
	);
}
