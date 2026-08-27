import type { MoneyPack } from "../data/moneyPacks";
import {
	SHOP_PACK_BODY,
	SHOP_PACK_CARD,
	SHOP_PACK_FOOTER,
	SHOP_PACK_META,
	SHOP_PACK_PRICE,
} from "./shopStyles";

const COIN_URL = `${import.meta.env.VITE_IMAGE_MINIO}/assets/RoyalCoins.svg`;

interface MoneyPackCardProps {
	pack: MoneyPack;
	index: number;
}

// One tile in MoneyPackGrid (currently unused/unmounted, see that file).
// Purely presentational: no purchase handler is wired up here, it just
// renders the pack's label/price and a row of coin icons whose count scales
// with the pack's position (1 coin for the cheapest, up to 4 for the
// priciest) so the visual price ladder reads at a glance.
export function MoneyPackCard({ pack, index }: MoneyPackCardProps) {
	const coinCount = index + 1;

	return (
		<article className={SHOP_PACK_CARD}>
			<div className="relative flex h-[66px] items-center justify-center overflow-hidden rounded-[2px]">
				<div className="relative z-[1] flex items-center justify-center gap-1.5">
					{Array.from({ length: coinCount }).map((_, i) => (
						<img
							key={i}
							src={COIN_URL}
							alt=""
							className="h-6 w-6 object-contain drop-shadow-[0_2px_6px_rgba(0,0,0,0.4)]"
						/>
					))}
				</div>
			</div>
			<div className={SHOP_PACK_BODY}>
				<p className={SHOP_PACK_META}>{pack.label}</p>
				<div className={SHOP_PACK_FOOTER}>
					<span className={SHOP_PACK_PRICE}>{pack.price}</span>
				</div>
			</div>
		</article>
	);
}