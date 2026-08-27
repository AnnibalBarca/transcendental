import { useEffect, useState } from "react";
import { shopService, type ShopCollection } from "@/features/shop/services/shopService";
import { cosmeticImageUrl } from "@/utils/cosmeticImage";
import type { OfferItem } from "@/features/play/components/OfferCarousel";

function mapCollectionToOffer(collection: ShopCollection): OfferItem {
	const expiresAt = Number.isNaN(new Date(collection.end_date).getTime())
		? undefined
		: new Date(collection.end_date).getTime();
	const first = collection.items[0];
	const backgroundImage = first
		? cosmeticImageUrl(first.item_type, first.item_id)
		: "";

	return {
		id: collection.id,
		backgroundImage,
		price: collection.price,
		timeRemaining: "",
		expiresAt,
		title: collection.title,
	};
}

export function useOffers(): OfferItem[] {
	const [offers, setOffers] = useState<OfferItem[]>([]);

	useEffect(() => {
		let cancelled = false;

		shopService
			.getShop()
			.then((data) => {
				if (cancelled) return;
				setOffers(data.collections.map(mapCollectionToOffer));
			})
			.catch(() => {
				if (!cancelled) setOffers([]);
			});

		return () => {
			cancelled = true;
		};
	}, []);

	return offers;
}
