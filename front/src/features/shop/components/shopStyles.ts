export const SHOP_SCROLL =
	"box-border flex h-full w-full flex-col gap-[18px] overflow-y-auto pt-16 pb-30";

const SHOP_COLUMN = "w-[min(95%,680px)] mx-auto box-border lg:w-[min(92%,1100px)]";

export const SHOP_PANEL = `${SHOP_COLUMN} px-3 flex flex-col gap-4`;

export const SHOP_PANEL_HEADER = "flex items-start justify-between px-1";

export const SHOP_KICKER =
	"mx-0 mt-0 mb-1.5 text-[12px] font-bold uppercase tracking-[0.14em] text-[rgba(255,255,255,0.45)]";

export const SHOP_TITLE =
	"m-0 text-[clamp(18px,3vw,24px)] font-extrabold leading-[1.1] text-white max-[560px]:text-[17px]";

export const SHOP_NOTICE = `${SHOP_COLUMN} rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/80 px-3.5 py-2.5 text-center text-[14px] font-bold text-white`;

const CARD_SURFACE =
	"relative flex flex-col overflow-hidden border border-[#334155]/60 bg-[#0f172a]/70 text-white";

const GRID_TWO = "grid grid-cols-2 gap-[14px] max-[560px]:grid-cols-[1fr_1fr] lg:grid-cols-3 xl:grid-cols-4";

export const SHOP_PACK_GRID = GRID_TWO;

export const SHOP_PACK_CARD = `${CARD_SURFACE} min-h-40 justify-between rounded-[2px] p-3 shadow-[0_10px_30px_rgba(0,0,0,0.3)]`;

export const SHOP_PACK_RANK =
	"absolute top-3 right-3 text-[12px] font-bold opacity-40";

const PACK_VISUAL_BASE =
	"relative flex h-[66px] items-center justify-center overflow-hidden rounded-[2px]";

const PACK_VISUAL_FALLBACK =
	"bg-[linear-gradient(135deg,rgba(255,255,255,0.08),rgba(255,255,255,0.02))]";

const PACK_VISUAL_ACCENT: Record<string, string> = {
	hot: "bg-[linear-gradient(135deg,#ff3b3b,#ff1f5a)]",
	warm: "bg-[linear-gradient(135deg,#ff7a18,#ffb320)]",
	gold: "bg-[linear-gradient(135deg,#f59e0b,#facc15)]",
	deep: "bg-[linear-gradient(135deg,#0f172a,#334155)]",
};

export const packVisual = (accent: string) =>
	`${PACK_VISUAL_BASE} ${PACK_VISUAL_ACCENT[accent] ?? PACK_VISUAL_FALLBACK}`;

export const SHOP_PACK_GLOW =
	"absolute top-auto right-auto bottom-[-30%] left-1/2 h-[150px] w-[150px] -translate-x-1/2 rounded-[50%] opacity-35 bg-[radial-gradient(circle,rgba(255,255,255,0.9),transparent_65%)]";

export const SHOP_PACK_ICON =
	"relative z-[1] text-[18px] font-extrabold text-[rgba(255,255,255,0.95)] [text-shadow:0_2px_14px_rgba(0,0,0,0.35)]";

export const SHOP_PACK_BODY = "flex flex-col gap-2";

export const SHOP_PACK_TITLE = "m-0 text-[16px] font-extrabold";

export const SHOP_PACK_META =
	"m-0 text-[13px] leading-[1.4] text-white/60";

export const SHOP_PACK_FOOTER = "flex items-baseline justify-between gap-3";

export const SHOP_PACK_AMOUNT = "text-[13px] font-bold text-white/70";

export const SHOP_PACK_PRICE = "text-[18px] font-black text-[#ef4444]";

export const SHOP_ITEM_GRID = GRID_TWO;

export const SHOP_ITEM_CARD = `${CARD_SURFACE} min-h-[150px] justify-between rounded-[2px] px-3 pt-[14px] pb-3 shadow-[0_10px_30px_rgba(0,0,0,0.3)]`;

export const SHOP_ITEM_TAG =
	"self-start rounded-[2px] bg-blue-900 px-2 py-[3px] text-[11px] font-extrabold uppercase tracking-[0.08em] text-white";

const ITEM_GLOW_BASE =
	"absolute top-[-40%] right-[-20%] bottom-auto left-auto h-[160px] w-[160px] rounded-[50%] opacity-35";

const ITEM_GLOW_ACCENT: Record<string, string> = {
	vetement: "bg-[radial-gradient(circle,rgba(96,165,250,0.9),transparent_65%)]",
	carte: "bg-[radial-gradient(circle,rgba(52,211,153,0.9),transparent_65%)]",
	terrain: "bg-[radial-gradient(circle,rgba(251,146,60,0.9),transparent_65%)]",
};

export const itemGlow = (accent: string) =>
	`${ITEM_GLOW_BASE} ${ITEM_GLOW_ACCENT[accent] ?? ITEM_GLOW_ACCENT.vetement}`;

export const SHOP_ITEM_TITLE = "relative z-[1] m-0 text-[16px] font-extrabold";

export const SHOP_ITEM_KIND =
	"relative z-[1] m-0 text-[13px] leading-[1.4] text-white/60";

export const SHOP_ITEM_FOOTER = "relative z-[1] flex items-center gap-1.5";

export const SHOP_ITEM_COIN = "h-[18px] w-[18px] text-[#f59e0b]";

export const SHOP_ITEM_PRICE = "text-[18px] font-black text-white";

export const SHOP_ITEM_BUY =
	"ml-auto cursor-pointer rounded-[2px] border border-[#334155]/60 bg-blue-900 px-3.5 py-1.5 text-[13px] font-extrabold text-white transition-colors hover:bg-blue-800 disabled:cursor-default disabled:bg-[#334155]/40 disabled:text-white/50";

export const SHOP_COLLECTION_GRID = "grid grid-cols-1 gap-[14px] pb-2.5 lg:grid-cols-2";

export const SHOP_COLLECTION_CARD =
	"flex min-h-28 flex-col items-center justify-center gap-2 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 px-[18px] py-5 text-center text-white shadow-[0_10px_30px_rgba(0,0,0,0.3)]";

export const SHOP_COLLECTION_TITLE = "m-0 text-[18px] font-extrabold";

export const SHOP_COLLECTION_SUBTITLE =
	"m-0 text-[14px] leading-[1.45] text-white/60";

export const SHOP_ITEM_IMG =
	"relative z-[1] mx-auto h-20 w-20 object-contain drop-shadow-[0_4px_10px_rgba(0,0,0,0.25)]";

export const SHOP_SLOT_TITLE =
	"m-0 text-[13px] font-extrabold uppercase tracking-[0.1em] text-[rgba(255,255,255,0.75)]";

export const SHOP_SLOT_SECTION = "flex flex-col gap-2.5";

export const SHOP_COLLECTION_ITEMS =
	"flex flex-wrap items-center justify-center gap-2.5";

export const SHOP_COLLECTION_PIECE =
	"flex w-16 flex-col items-center gap-1 text-[11px] text-white/60";

export const SHOP_COLLECTION_PIECE_IMG =
	"h-12 w-12 rounded-[2px] bg-white/[0.06] object-contain";

export const SHOP_COLLECTION_BUY =
	"cursor-pointer rounded-[2px] border border-[#334155]/60 bg-blue-900 px-4 py-2 text-[13px] font-extrabold text-white transition-colors hover:bg-blue-800 disabled:cursor-default disabled:bg-[#334155]/40 disabled:text-white/50";