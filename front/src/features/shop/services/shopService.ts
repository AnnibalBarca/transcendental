import i18n from "@/i18n";

// Every call here goes through nginx -> Gateway-API (which resolves "/api/user/*"
// over a Redis stream to User-API, see back/Gateway-API/src/http/handlers/router.rs)
// -> the handlers in back/User-API/src/http/handlers/shop.rs / add_item.rs / remove_item.rs.
// `credentials: "include"` is required on every call: auth is carried by the
// httpOnly "access_token" cookie, read server-side by
// api_core::auth::validate_and_get_claims (no Authorization header is used).
const API_USER = "/api/user";

export interface ShopItem {
	item_id: string;
	item_type: string;
	title: string;
	price: number;
	asset_key: string;
}

export interface ShopCollectionItem {
	item_id: string;
	item_type: string;
	title: string;
	price: number;
	asset_key: string;
}

export interface ShopCollection {
	id: string;
	title: string;
	price: number;
	end_date: string;
	items: ShopCollectionItem[];
}

export interface OwnedItem {
	item_id: string;
	item_type: string;
}

export interface ShopData {
	items: ShopItem[];
	collections: ShopCollection[];
	slots: string[];
	packs?: ShopPack[];
	wallet?: number;
	owned?: OwnedItem[];
}

export interface ShopPack {
	type: string;
	kind: "skin" | "card";
	label: string;
	count: number;
	price: number;
}

export interface PackReward {
	item_id: string;
	item_type: string;
	title: string;
	price: number;
	is_duplicate: boolean;
	refunded: number;
	rarity?: number;
}

export interface OpenPackResult {
	pack_type: string;
	wallet: number;
	rewards: PackReward[];
}

export interface PurchaseResult {
	item_id: string;
	item_type: string;
	wallet: number;
}

export interface CollectionPurchaseResult {
	collection_id: string;
	granted: OwnedItem[];
	wallet: number;
}

/**
 * Every backend error handler returns a JSON error shape ({ error }) or
 * ({ message }) depending on which helper produced it (api_core's json_error
 * vs. a raw json!() call) — this normalizes both so the UI always has a
 * displayable string, falling back to a translated default if the body
 * can't be parsed at all (e.g. a raw 502 from nginx with no JSON body).
 */
async function extractErrorMessage(
	response: Response,
	defaultMsg: string,
): Promise<string> {
	try {
		const data = await response.json();
		if (typeof data?.error === "string" && data.error.trim().length > 0) {
			return data.error;
		}
		if (typeof data?.message === "string" && data.message.trim().length > 0) {
			return data.message;
		}
	} catch {
		void 0;
	}
	return defaultMsg;
}

export const shopService = {
	/**
	 * Entry point of the shop screen (called once from ShopView's mount
	 * effect). Hits GET /api/user/shop -> User-API's handle_get_shop
	 * (back/User-API/src/http/handlers/shop.rs), which returns in one
	 * payload: the active catalog (`items`), the bundle `collections`
	 * (each with its own resolved item images), the 5 cosmetic `slots`,
	 * and — only if the access_token cookie is valid — the caller's
	 * `wallet` balance and `owned` item list. Everything the shop UI
	 * needs is fetched in this single round trip.
	 */
	async getShop(): Promise<ShopData> {
		const response = await fetch(`${API_USER}/shop`, {
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(
				response,
				i18n.t("shop.loadError"),
			);
			throw new Error(msg);
		}

		return response.json();
	},

	/**
	 * Buys a single cosmetic item (one avatar slot). POSTs to
	 * /api/user/inventory, routed to add_item.rs::handle_add_item, which
	 * calls db::shop::purchase — an atomic SQL transaction that checks the
	 * price, rejects a duplicate ("already owned"), then does
	 * `UPDATE users SET wallet = wallet - price WHERE wallet >= price`
	 * (the balance check lives in the WHERE clause itself, so two
	 * concurrent purchases can't both succeed on an insufficient balance)
	 * before inserting the row into player_inventory.
	 * NOTE: not currently wired to any button in ShopView — the live UI
	 * only sells whole collections/packs, so this path is effectively
	 * dead code on the front end today.
	 */
	async purchaseItem(
		item_id: string,
		item_type: string,
	): Promise<PurchaseResult> {
		const response = await fetch(`${API_USER}/inventory`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ item_id, item_type }),
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(response, i18n.t("shop.genericError"));
			throw new Error(msg);
		}

		return response.json();
	},

	/**
	 * Buys an entire bundle (e.g. "Collection almeekel" = the 5 avatar
	 * pieces for one teammate's crew skin). Called from
	 * ShopView::handleBuyCollection. Backend: db::shop::purchase_collection
	 * — same atomic-wallet-debit pattern as purchaseItem, but it first
	 * diffs the collection's items against player_inventory so it only
	 * charges/grants pieces not already owned, and refuses the purchase
	 * outright if every piece is already owned.
	 */
	async purchaseCollection(
		collection_id: string,
	): Promise<CollectionPurchaseResult> {
		const response = await fetch(`${API_USER}/collections/purchase`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ collection_id }),
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(
				response,
				i18n.t("shop.buyCollectionError"),
			);
			throw new Error(msg);
		}

		return response.json();
	},

	// Opens a gacha-style card/skin pack. Not part of my (almeekel) backend
	// work — /api/user/packs/open is a separate handler added later by a
	// teammate — kept here only because ShopView calls it alongside the
	// two methods above.
	async openPack(pack_type: string): Promise<OpenPackResult> {
		const response = await fetch(`${API_USER}/packs/open`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ pack_type }),
			credentials: "include",
		});

		if (!response.ok) {
			const msg = await extractErrorMessage(
				response,
				i18n.t("shop.openError"),
			);
			throw new Error(msg);
		}

		return response.json();
	},
};
