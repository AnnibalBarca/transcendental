export type UrgencyLevel = "safe" | "warning" | "critical" | "urgent" | "expiring";

export const OFFER_WRAPPER = "w-full max-w-full";

const CAROUSEL_BASE =
	"relative w-full max-w-full overflow-hidden rounded-[2px] border-2 bg-[rgba(14,11,30,0.6)] transition-[border-color,box-shadow] duration-400 ease-[ease]";

const CAROUSEL_URGENCY: Record<UrgencyLevel, string> = {
	safe: "border-[rgba(255,255,255,0.07)] shadow-[0_10px_40px_rgba(0,0,0,0.4)]",
	warning: "border-[rgba(251,191,36,0.6)] shadow-[0_10px_40px_rgba(251,191,36,0.15)]",
	critical: "border-[rgba(249,115,22,0.7)] shadow-[0_10px_40px_rgba(249,115,22,0.2)]",
	urgent: "border-[rgba(239,68,68,0.8)] shadow-[0_10px_40px_rgba(239,68,68,0.25)]",
	expiring:
		"border-[rgb(239,68,68)] shadow-[0_0_30px_rgba(239,68,68,0.4)] offer-anim-border",
};

export const offerCarousel = (level: UrgencyLevel) =>
	`${CAROUSEL_BASE} ${CAROUSEL_URGENCY[level]}`;

const BADGE_BASE =
	"absolute top-4 right-4 z-[3] rounded-[9999px] px-3.5 py-1.5 text-[15px] font-bold uppercase tracking-[0.5px] backdrop-blur-[8px]";

const BADGE_URGENCY: Record<UrgencyLevel, string> = {
	safe: "",
	warning: "border border-[rgba(251,191,36,0.3)] bg-[rgba(251,191,36,0.2)] text-[#fbbf24]",
	critical: "border border-[rgba(249,115,22,0.3)] bg-[rgba(249,115,22,0.2)] text-[#fb923c]",
	urgent: "border border-[rgba(239,68,68,0.3)] bg-[rgba(239,68,68,0.2)] text-[#f87171]",
	expiring:
		"border border-[rgba(239,68,68,0.5)] bg-[rgba(239,68,68,0.3)] text-[#fca5a5] offer-anim-badge",
};

export const offerBadge = (level: UrgencyLevel) =>
	`${BADGE_BASE} ${BADGE_URGENCY[level]}`;

export const OFFER_DOTS =
	"absolute top-4 left-4 z-[3] flex gap-2";

export const offerDot = (isActive: boolean) =>
	`w-2.5 h-2.5 rounded-[9999px] border-none p-0 cursor-pointer transition-all duration-300 ease-[ease] ${
		isActive ? "bg-white" : "bg-[rgba(255,255,255,0.5)]"
	}`;

export const OFFER_VIEWPORT = "w-full aspect-video overflow-hidden";

export const OFFER_TRACK = "flex h-full";

export const offerSlide = (isClickable: boolean) =>
	`relative flex h-full min-w-0 flex-[0_0_100%] items-end bg-cover bg-center ${
		isClickable ? "cursor-pointer" : ""
	}`;

export const OFFER_OVERLAY =
	"absolute inset-0 z-[1] bg-linear-to-t/srgb from-[rgba(14,11,30,0.9)] via-[rgba(14,11,30,0.3)] to-transparent";

export const OFFER_CONTENT =
	"relative z-[2] flex w-full flex-col gap-3.5 p-6";

export const OFFER_TITLE =
	"m-0 text-[15px] font-bold text-white [text-shadow:0_2px_8px_rgba(0,0,0,0.5)]";

export const OFFER_PRICE = "flex items-baseline gap-1 text-white";

export const offerPriceValue = (activeLevel: UrgencyLevel) =>
	`text-[15px] font-extrabold transition-[color] duration-400 ease-[ease] ${
		activeLevel === "critical" || activeLevel === "urgent" || activeLevel === "expiring"
			? "text-[#f87171]"
			: "text-[#fbbf24]"
	}`;

export const OFFER_PRICE_CURRENCY = "font-medium opacity-70";
