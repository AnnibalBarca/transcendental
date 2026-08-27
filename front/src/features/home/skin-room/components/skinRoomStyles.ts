const TILE_BG = "bg-[#0f172a]";

const TILE_RADIUS = "rounded-[2px]";

const tileBorder = (isActive: boolean) =>
	`border-2 ${isActive ? "border-[#60a5fa]" : "border-[#334155]/60"}`;

export const SKIN_SCROLL = "flex w-full h-full max-h-[80vh] justify-center overflow-y-auto pt-16 gap-y-[25px]";

export const SKIN_ROOM = "w-[clamp(200px,90vw,580px)]";

export const EQUIPMENT_BOARD = "flex aspect-square w-[clamp(200px,70vw,450px)] mx-auto";

export const CHARACTER_IMAGE = `${tileBorder(false)} ${TILE_RADIUS} object-cover p-[5px] h-full w-full`;

export const EQUIPMENT_COLUMN = "flex flex-col items-center justify-center shrink-0 w-[25%] h-full min-h-0 min-w-0";

export const equipmentSlot = (isActive: boolean) =>
	`${TILE_BG} ${tileBorder(isActive)} ${TILE_RADIUS} flex aspect-square h-[calc(25%)] cursor-pointer min-h-0 min-w-0`;

export const ITEM_LIST = "flex flex-wrap justify-center list-none m-0 p-4 gap-[2vw]";

export const itemTile = (isActive: boolean) =>
	`${TILE_BG} ${tileBorder(isActive)} ${TILE_RADIUS} aspect-square w-[15%] cursor-pointer overflow-hidden min-h-0 min-w-0"`;

export const TILE_IMAGE = "h-full w-full object-cover";

export const CARD_DECK_TITLE = "m-0 text-[3rem] font-extrabold";

export const CARD_LIST =
  "flex flex-wrap justify-center list-none m-0 p-4 gap-[2vw] bg-[#0f172a] rounded-[2px] border border-[#334155]/60 p-4";

export const cardtitle = (owned: boolean) =>
  `${TILE_BG} ${TILE_RADIUS} w-[clamp(48px,12vw,120px)] aspect-[2/3] overflow-hidden [content-visibility:auto] [contain-intrinsic-size:80px_120px] ${
    owned
      ? ""
      : "opacity-50"
  }`

export const CARD_ITEM =
  "box-border object-cover cursor-pointer overflow-hidden min-h-0 min-w-0 w-full h-full flex flex-col"

export const TEXT_SOFT = "font-sans text-[clamp(0.5rem,1rem,2rem)] leading-relaxed tracking-wide text-white/70 text-center mx-10 mb-10";

export const BUTTON_ROW = "flex justify-center gap-4 mt-6 mb-6 w-full px-6 text-[clamp(0.5rem,2vw,2rem)]";

export const BUTTON_PRIMARY =
  "flex-1 min-w-0 px-4 py-4 rounded-[2px] bg-blue-900 text-white font-sans text-base text-center active:scale-95 truncate";

export const BUTTON_SECONDARY =
  "flex-1 min-w-0 px-4 py-4 rounded-[2px] bg-transparent text-white font-sans text-base text-center border-2 border-[#334155]/60 cursor-pointer truncate";


export const CAROUSSEL_CARD = "w-[clamp(50px,50%,175px)] mx-auto"

export const MODAL_CARD_ITEM = "w-full flex justify-center"

// L'image — se base sur le carousel parent
export const modalcardtitle = (owned: boolean) =>
  `${TILE_BG} ${TILE_RADIUS} w-[clamp(120px,60%,350px)] aspect-[2/3] overflow-hidden object-cover mx-auto ${
    owned
      ? ""
      : "opacity-50"
  }`

// Le popup global — inchangé
export const CARD_MODAL_POP_UP =
  ` w-[clamp(250px,80%,400px)]  text-[3rem] bg-[#0f172a]/90 backdrop-blur m-auto rounded-[2px] border border-[#334155]/60 p-5 min-w-[300px]`
// export const CARD_LIST = "flex flex-wrap justify-center list-none m-0 p-4 gap-[2vw]";

// export const cardtitle = (owned: boolean) =>
//   `${TILE_BG} ${owned ? "bg-[#FF0000]" : "bg-[#00FFFF]"} ${TILE_RADIUS}`;

// export const CARD_ITEM = "box-border w-[20%] aspect-[2/3] cursor-pointer overflow-hidden min-h-0 min-w-0";
