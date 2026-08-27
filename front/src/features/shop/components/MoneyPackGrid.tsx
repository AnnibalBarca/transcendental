import { MONEY_PACKS } from "../data/moneyPacks";
import { MoneyPackCard } from "./MoneyPackCard";
import {
	SHOP_KICKER,
	SHOP_PACK_GRID,
	SHOP_PANEL,
	SHOP_PANEL_HEADER,
	SHOP_TITLE,
} from "./shopStyles";

// Renders MONEY_PACKS (hardcoded, front-only data — no backend endpoint
// backs a real-money purchase) as a grid of MoneyPackCard.
//
// DEAD CODE: this component is not imported/rendered anywhere (ShopView
// only mounts PacksSection + the collections section). It's a leftover
// from an early "buy in-game currency with real money" concept that was
// dropped — the actual currency ("cacahouètes") is earned by playing, not
// bought. Left here undeleted for now; safe to remove if confirmed unused
// at the next cleanup pass.
export function MoneyPackGrid() {
	return (
		<section className={SHOP_PANEL}>
			<div className={SHOP_PANEL_HEADER}>
				<div>
					<p className={SHOP_KICKER}></p>
					<h2 className={SHOP_TITLE}>Cacahouètes</h2>
				</div>
			</div>
			<div className={SHOP_PACK_GRID}>
				{MONEY_PACKS.map((pack, index) => (
					<MoneyPackCard key={pack.id} pack={pack} index={index} />
				))}
			</div>
		</section>
	);
}