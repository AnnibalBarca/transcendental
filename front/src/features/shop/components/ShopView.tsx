import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { emitWalletUpdated } from "@/features/play/hooks/useWallet";
import { emitInventoryUpdated } from "@/features/home/skin-room/context/CosmeticProvider";
import { PacksSection } from "./PacksSection";
import { PackOpeningModal } from "./PackOpeningModal";
import { collectionOwnerLogin, isKnownShopSlot, itemKey } from "../utils";
import {
	shopService,
	type OpenPackResult,
	type ShopCollection,
	type ShopPack,
} from "../services/shopService";
import { toast } from "@/components/ui/toast";
import { ThemeButton } from "@/features/play/components/ThemeButton";
import { cosmeticImageUrl } from "@/utils/cosmeticImage";

import {
	SHOP_COLLECTION_CARD,
	SHOP_COLLECTION_GRID,
	SHOP_COLLECTION_ITEMS,
	SHOP_COLLECTION_PIECE,
	SHOP_COLLECTION_PIECE_IMG,
	SHOP_COLLECTION_SUBTITLE,
	SHOP_COLLECTION_TITLE,
	SHOP_NOTICE,
	SHOP_PANEL,
	SHOP_PANEL_HEADER,
	SHOP_SCROLL,
	SHOP_TITLE,
} from "./shopStyles";

// Namespaced Set key for a collection's "busy" state (see busyKey in
// ShopView), kept distinct from itemKey() so a collection purchase in
// flight can't collide with a single-item purchase key of the same id.
function collectionKey(id: string) {
	return `collection:${id}`;
}

// VITE_IMAGE_MINIO is the browser-facing MinIO base URL (same value the
// backend calls MINIO_PUBLIC_ENDPOINT — see services::storage::Storage in
// back/User-API). Static UI assets (like this coin icon) live in a plain
// "assets/" prefix on the bucket, separate from the "assets_key"-addressed
// cosmetic images (e.g. "base/1.png") returned per-item by GET /shop.
const COIN_URL = `${import.meta.env.VITE_IMAGE_MINIO}/assets/RoyalCoins.svg`;

/**
 * Renders the bundle/collection grid (one card per teammate's "crew"
 * skin bundle, e.g. "Collection almeekel" = 5 pieces at a discount).
 * `ownedKeys` (built in ShopView from GET /shop's `owned` list) drives
 * both the "complete" ribbon and disabling the buy button once every
 * piece in the bundle is already owned.
 */
function ShopCollectionSection({
	collections,
	ownedKeys,
	busyKey,
	onBuyCollection,
}: {
	collections: ShopCollection[];
	ownedKeys: Set<string>;
	busyKey: string | null;
	onBuyCollection: (collection: ShopCollection) => void;
}) {
	const { t } = useTranslation();

	return (
		<section className={SHOP_PANEL}>
			<div className={SHOP_PANEL_HEADER}>
				<div>
					<h2 className={SHOP_TITLE}>{t("shop.collections")}</h2>
				</div>
			</div>
			<div className={SHOP_COLLECTION_GRID}>
				{collections.length === 0 && (
					<article className={SHOP_COLLECTION_CARD}>
						<p className={SHOP_COLLECTION_SUBTITLE}>
							{t("shop.noCollections")}
						</p>
					</article>
				)}
				{collections.map((collection) => {
					const key = collectionKey(collection.id);
					const complete =
						collection.items.length > 0 &&
						collection.items.every((piece) =>
							ownedKeys.has(itemKey(piece.item_id, piece.item_type)),
						);
					const collectionLabel = `${t("shop.collectionOf")} ${collectionOwnerLogin(collection.title)}`;
					return (
						<article
							key={collection.id}
							className="relative flex min-h-28 flex-col items-center justify-center gap-2 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/70 px-[18px] py-5 text-center text-white shadow-[0_10px_30px_rgba(0,0,0,0.3)]"
						>
							<span className="absolute top-2 left-2 flex items-center gap-1 rounded-full bg-[rgba(10,8,25,0.75)] px-2.5 py-1 text-[13px] font-black text-white shadow-[0_2px_10px_rgba(0,0,0,0.4)] backdrop-blur-sm">
								<img
									src={COIN_URL}
									alt=""
									className="h-3.5 w-3.5"
								/>
								{collection.price}
							</span>
							<h3 className={SHOP_COLLECTION_TITLE}>{collectionLabel}</h3>
							{collection.items.length > 0 && (
								<div className={SHOP_COLLECTION_ITEMS}>
									{collection.items.map((piece) => {
										const pieceLabel = isKnownShopSlot(piece.item_type)
											? `${t(`shop.slot.${piece.item_type}`)} ${collectionOwnerLogin(piece.title)}`
											: piece.title;
										return (
											<div
												key={`${piece.item_type}:${piece.item_id}`}
												className={SHOP_COLLECTION_PIECE}
											>
											<img
												src={cosmeticImageUrl(piece.item_type, piece.item_id)}
												alt={pieceLabel}
												className={SHOP_COLLECTION_PIECE_IMG}
												loading="lazy"
											/>
												<span>{pieceLabel}</span>
											</div>
										);
									})}
								</div>
							)}
							<ThemeButton
								type="button"
								texturePosition="center 98%"
								textureZoom={130}
								className="w-full px-4 py-2 text-[13px]"
								disabled={
									complete || busyKey === key || collection.items.length === 0
								}
								onClick={() => onBuyCollection(collection)}
							>
								{complete
									? t("shop.owned")
									: busyKey === key
										? "..."
										: t("shop.buyPack")}
							</ThemeButton>
						</article>
					);
				})}
			</div>
		</section>
	);
}

/**
 * Root shop screen (mounted by ShopPage). Owns all shop state: the fetched
 * catalog/collections, the set of item keys the player already owns (used
 * to grey out already-bought pieces without waiting on a second request),
 * and which purchase button is currently "busy" (in-flight request, keyed
 * by pack type / collectionKey so only that one button shows a spinner).
 */
export function ShopView() {
	const { t } = useTranslation();
	const [collections, setCollections] = useState<ShopCollection[]>([]);
	const [packs, setPacks] = useState<ShopPack[]>([]);
	const [ownedKeys, setOwnedKeys] = useState<Set<string>>(new Set());
	const [busyKey, setBusyKey] = useState<string | null>(null);
	const [loading, setLoading] = useState(true);
	const [openingPack, setOpeningPack] = useState<ShopPack | null>(null);
	const [packResult, setPackResult] = useState<OpenPackResult | null>(null);

	// Single fetch on mount: loads the whole shop payload (see
	// shopService.getShop's comment) and derives ownedKeys from it. The
	// `cancelled` flag guards against setting state after the component
	// unmounted mid-request (e.g. the player navigates away from /shop
	// before the response lands).
	useEffect(() => {
		let cancelled = false;
		shopService
			.getShop()
			.then((data) => {
				if (cancelled) return;
				setCollections(data.collections);
				setPacks(data.packs ?? []);
				setOwnedKeys(
					new Set(
						(data.owned ?? []).map((o) => itemKey(o.item_id, o.item_type)),
					),
				);
			})
			.catch((error: Error) => {
				if (!cancelled) 
				{
					toast.add(
						{
							title: t("shop.loadError"),
							description: error.message,
							type: "error",
						}
					)
				}
			})
			.finally(() => {
				if (!cancelled) setLoading(false);
			});
		return () => {
			cancelled = true;
		};
	}, [t]);

	// Pack-opening flow (not my code — added later on top of the shop
	// skeleton by a teammate for the gacha-pack feature). Kept for context:
	// on success it merges newly-granted, non-duplicate rewards into
	// ownedKeys and opens PackOpeningModal to reveal them.
	const handleOpenPack = useCallback(async (pack: ShopPack) => {
		setBusyKey(pack.type);
		try {
			const result = await shopService.openPack(pack.type);
			emitWalletUpdated();
			emitInventoryUpdated();
			setOwnedKeys((prev) => {
				const next = new Set(prev);
				for (const reward of result.rewards) {
					if (!reward.is_duplicate) {
						next.add(itemKey(reward.item_id, reward.item_type));
					}
				}
				return next;
			});
			setOpeningPack(pack);
			setPackResult(result);
			toast.add(
				{
					title: t("shop.collectionBought"),
					description: t("shop.collectionBoughtDesc", { title: pack.label }),
					type: "success",
				}
			)
		} catch (error) {
			toast.add(
				{
					title: t("shop.openError"),
					description: error instanceof Error ? error.message : t("shop.openErrorUnknown"),
					type: "error",
				}
			)
		} finally {
			setBusyKey(null);
		}
	}, [t]);

	// Buy-a-bundle flow, wired to each ShopCollectionSection card's button.
	// Marks only that collection's key as busy (not the whole shop), calls
	// the backend, and on success merges the granted pieces into ownedKeys
	// so the card flips to "owned" immediately — no need to re-fetch
	// GET /shop just to reflect the purchase. Also fires
	// emitWalletUpdated/emitInventoryUpdated, two tiny pub/sub events other
	// screens (header wallet display, skin room) listen to so they refresh
	// without this component knowing anything about them.
	const handleBuyCollection = useCallback(async (collection: ShopCollection) => {
		const key = collectionKey(collection.id);
		setBusyKey(key);
		try {
			const result = await shopService.purchaseCollection(collection.id);
			emitWalletUpdated();
			emitInventoryUpdated();
			setOwnedKeys((prev) => {
				const next = new Set(prev);
				for (const granted of result.granted) {
					next.add(itemKey(granted.item_id, granted.item_type));
				}
				return next;
			});
			toast.add(
				{
					title: t("shop.collectionBought"),
					description: t("shop.collectionBoughtDesc", {
						title: `${t("shop.collectionOf")} ${collectionOwnerLogin(collection.title)}`,
					}),
					type: "success",
				}
			)
		} catch (error) {
			toast.add(
				{
					title: t("shop.buyCollectionError"),
					description: error instanceof Error ? error.message : t("shop.buyCollectionErrorUnknown"),
					type: "error",
				}
			)
		} finally {
			setBusyKey(null);
		}
	}, [t]);

	return (
		<div className={SHOP_SCROLL}>
			{loading ? (
				<div className={SHOP_NOTICE}>{t("shop.loading")}</div>
			) : (
				<>
					<PacksSection
						packs={packs}
						busy={busyKey !== null}
						onOpen={handleOpenPack}
					/>
					<ShopCollectionSection
						collections={collections}
						ownedKeys={ownedKeys}
						busyKey={busyKey}
						onBuyCollection={handleBuyCollection}
					/>
				</>
			)}

			{openingPack && packResult && (
				<PackOpeningModal
					kind={openingPack.kind}
					rewards={packResult.rewards}
					wallet={packResult.wallet}
					onClose={() => {
						setOpeningPack(null);
						setPackResult(null);
					}}
				/>
			)}
		</div>
	);
}
