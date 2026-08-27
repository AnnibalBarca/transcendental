import { useEffect, useState } from "react";
import { shopService } from "@/features/shop/services/shopService";

export function useWallet(): number | null {
	const [wallet, setWallet] = useState<number | null>(null);

	useEffect(() => {
		let cancelled = false;

		const fetchWallet = () => {
			shopService
				.getShop()
				.then((data) => {
					if (!cancelled) setWallet(data.wallet ?? null);
				})
				.catch(() => {
					if (!cancelled) setWallet(null);
				});
		};

		fetchWallet();
		window.addEventListener("wallet:updated", fetchWallet);

		return () => {
			cancelled = true;
			window.removeEventListener("wallet:updated", fetchWallet);
		};
	}, []);

	return wallet;
}

export function emitWalletUpdated() {
	window.dispatchEvent(new Event("wallet:updated"));
}
