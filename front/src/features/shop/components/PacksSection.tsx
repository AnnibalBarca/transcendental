import type { ShopPack } from "../services/shopService";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import { useTranslation } from "react-i18next";

interface PacksSectionProps {
	packs: ShopPack[];
	busy: boolean;
	onOpen: (pack: ShopPack) => void;
}

const COIN_URL = `${import.meta.env.VITE_IMAGE_MINIO}/assets/RoyalCoins.svg`;

const PACK_IMAGE: Record<string, string> = {
	skin_1: `${import.meta.env.VITE_IMAGE_MINIO}/pack/skin_1.svg`,
	skin_3: `${import.meta.env.VITE_IMAGE_MINIO}/pack/skin_3.svg`,
	card_2: `${import.meta.env.VITE_IMAGE_MINIO}/pack/card_2.svg`,
	card_5: `${import.meta.env.VITE_IMAGE_MINIO}/pack/card_5.svg`,
};

export function PacksSection({ packs, busy, onOpen }: PacksSectionProps) {
	const { t } = useTranslation();
	return (
		<section className="mx-auto flex w-[min(95%,680px)] flex-col gap-4 px-3 lg:w-[min(92%,1100px)]">
			<div className="flex items-start justify-between px-1">
				<div>
					<p className="mx-0 mt-0 mb-1.5 text-[12px] font-bold tracking-[0.14em] text-[rgba(255,255,255,0.45)] uppercase">
						{t("shop.packsKicker")}
					</p>
					<h2 className="m-0 text-[clamp(18px,3vw,24px)] font-extrabold leading-[1.1] text-white">
						{t("shop.packsToOpen")}
					</h2>
				</div>
			</div>

			<div className="grid grid-cols-2 gap-[14px] lg:grid-cols-3 xl:grid-cols-4">
				{packs.map((pack) => (
					<article
						key={pack.type}
						className="relative flex flex-col overflow-hidden rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 text-white shadow-[0_10px_30px_rgba(0,0,0,0.3)]"
					>
						<div className="relative flex aspect-square w-full items-center justify-center overflow-hidden bg-white/[0.04]">
							{PACK_IMAGE[pack.type] ? (
								<img
									src={PACK_IMAGE[pack.type]}
									alt={pack.label}
									className="h-full w-full object-cover"
								/>
							) : (
<span className="text-3xl font-black text-white/30">
								{pack.label}
							</span>
							)}

							{ }
							<span className="absolute top-2 left-2 flex items-center gap-1 rounded-full bg-[rgba(10,8,25,0.75)] px-2.5 py-1 text-[13px] font-black text-white shadow-[0_2px_10px_rgba(0,0,0,0.4)] backdrop-blur-sm">
								<img
									src={COIN_URL}
									alt=""
									className="h-3.5 w-3.5"
								/>
								{pack.price}
							</span>
						</div>

						<div className="flex items-center justify-center p-2">
							<ThemeButton
								type="button"
								disabled={busy}
								texturePosition="center 98%"
								textureZoom={130}
								className="w-full px-3.5 py-2 text-[13px]"
								onClick={() => onOpen(pack)}
							>
								{busy ? t("shop.opening") : t("shop.buy")}
							</ThemeButton>
						</div>
					</article>
				))}
			</div>
		</section>
	);
}
